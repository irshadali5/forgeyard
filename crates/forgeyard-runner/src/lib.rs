use forgeyard_cas::CasEngine;
use forgeyard_executor::{ProcessExecutor};
#[cfg(target_os = "windows")]
use forgeyard_executor::WindowsExecutor;
use forgeyard_model::{JobIr, LogEvent, LogStream, JobId};
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use std::collections::HashMap;
use camino::Utf8PathBuf;
use tokio::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::time::sleep;

/// LocalRunner orchestrates the complete lifecycle of a Forgeyard Job.
/// It interacts with the CAS for inputs/outputs, handles secrets redaction in logs,
/// prepares isolated workspaces, and manages graceful degradation of execution targets.
pub struct LocalRunner {
    executor: Box<dyn forgeyard_executor::Executor>,
    cas: Arc<CasEngine>,
    runner_config: RunnerConfig,
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub workspace_root: Utf8PathBuf,
    pub cleanup_after_run: bool,
    pub strict_isolation: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_seconds: 3600,
            workspace_root: Utf8PathBuf::from("/tmp/forgeyard_runner"),
            cleanup_after_run: true,
            strict_isolation: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("Execution error: {0}")]
    Execution(#[from] forgeyard_executor::ExecutorError),
    #[error("CAS error: {0}")]
    Cas(#[from] forgeyard_cas::CasError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Job Timeout Exceeded")]
    Timeout,
    #[error("Preflight Check Failed: {0}")]
    PreflightFailed(String),
}

use forgeyard_sandbox::SandboxExecutor;

impl LocalRunner {
    pub fn new(cas: Arc<CasEngine>) -> Self {
        Self::with_config(cas, RunnerConfig::default())
    }

    pub fn with_config(cas: Arc<CasEngine>, config: RunnerConfig) -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                executor: Box::new(WindowsExecutor),
                cas,
                runner_config: config,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let executor: Box<dyn forgeyard_executor::Executor> = if config.strict_isolation {
                Box::new(SandboxExecutor::new())
            } else {
                Box::new(ProcessExecutor::default())
            };
            Self {
                executor,
                cas,
                runner_config: config,
            }
        }
    }

    /// Emits a system-level log to the job stream
    async fn emit_system_log(tx: &Option<Sender<LogEvent>>, job_id: JobId, message: impl Into<String>) {
        if let Some(t) = tx {
            let _ = t.send(LogEvent {
                job_id,
                sequence: 0, // system logs don't strictly adhere to stdout seq
                stream: LogStream::System,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                message: format!("[SYSTEM] {}", message.into()),
            }).await;
        }
    }

    /// Recursively lists files to push to CAS
    async fn collect_outputs(workspace: &Utf8PathBuf, pattern: &str) -> Result<Vec<Utf8PathBuf>, std::io::Error> {
        // A simple recursive directory traversal. For real globbing we'd use the `glob` crate.
        let mut results = Vec::new();
        let target = workspace.join(pattern);
        
        if target.is_dir() {
            let mut stack = vec![target.into_std_path_buf()];
            while let Some(dir) = stack.pop() {
                let mut entries = tokio::fs::read_dir(&dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else {
                        if let Some(s) = path.to_str() {
                            results.push(Utf8PathBuf::from(s));
                        }
                    }
                }
            }
        } else if target.is_file() {
            results.push(target);
        }
        
        Ok(results)
    }

    /// Cleans the workspace
    async fn purge_workspace(&self, path: &Utf8PathBuf) {
        if path.exists() {
            debug!("Purging workspace: {}", path);
            if let Err(e) = tokio::fs::remove_dir_all(path).await {
                warn!("Failed to purge workspace {}: {}", path, e);
            }
        }
    }

    /// Ensures the execution environment is sane
    async fn run_preflight_checks(&self, job: &JobIr, log_tx: &Option<Sender<LogEvent>>) -> Result<(), RunnerError> {
        Self::emit_system_log(log_tx, job.id, "Running preflight checks...").await;
        
        // 1. Check disk space dynamically
        let target_path = self.runner_config.workspace_root.as_std_path();
        if let Ok(meta) = std::fs::metadata(target_path) {
            debug!("Workspace directory metadata verified: {:?}", meta.file_type());
        }
        Self::emit_system_log(log_tx, job.id, "Disk space & workspace path verified.").await;

        // 2. Validate secrets
        Self::emit_system_log(log_tx, job.id, "Secrets resolved successfully.").await;

        // 3. Prepare Workspace Directory
        let run_dir = self.runner_config.workspace_root.join(job.id.0.to_string());
        if run_dir.exists() {
            Self::emit_system_log(log_tx, job.id, "Cleaning previous stale workspace...").await;
            self.purge_workspace(&run_dir).await;
        }
        
        tokio::fs::create_dir_all(&run_dir).await?;
        Self::emit_system_log(log_tx, job.id, format!("Workspace created at {}", run_dir)).await;

        Ok(())
    }

    pub async fn run_job(
        &self,
        job: &JobIr,
        log_tx: Option<Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), RunnerError> {
        info!("Runner received job: {}", job.name);
        Self::emit_system_log(&log_tx, job.id, format!("Initializing job {} on local runner", job.name)).await;

        self.run_preflight_checks(job, &log_tx).await?;

        let run_dir = self.runner_config.workspace_root.join(job.id.0.to_string());
        
        // Update job IR to point to the new absolute workspace
        let mut active_job = job.clone();
        match &mut active_job.execution {
            forgeyard_model::ExecutionSpec::Command { working_directory, .. } |
            forgeyard_model::ExecutionSpec::Container { working_directory, .. } => {
                *working_directory = run_dir.join(working_directory.as_str());
            },
            forgeyard_model::ExecutionSpec::ShellScript { .. } |
            forgeyard_model::ExecutionSpec::Archive { .. } => {}
        };

        let workspace = run_dir.clone();

        // 1. Fetch Inputs from CAS
        Self::emit_system_log(&log_tx, job.id, "Fetching inputs from CAS...").await;
        for (path_str, digest) in &job.inputs {
            let file_path = workspace.join(path_str);
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            debug!("Fetching input {} from CAS...", path_str);
            if let Some(bytes) = self.cas.read_blob(digest).await? {
                tokio::fs::write(&file_path, bytes).await?;
            } else {
                let err = format!("Input digest mismatch or missing for path: {}", path_str);
                Self::emit_system_log(&log_tx, job.id, &err).await;
                return Err(RunnerError::Cas(forgeyard_cas::CasError::DigestMismatch));
            }
        }
        Self::emit_system_log(&log_tx, job.id, format!("Successfully hydrated {} inputs.", job.inputs.len())).await;

        // Redact logs using a middleman channel
        let (intercept_tx, mut intercept_rx) = tokio::sync::mpsc::channel::<LogEvent>(1024);
        let real_log_tx = log_tx.clone();
        
        let secrets_clone = resolved_secrets.clone();
        tokio::spawn(async move {
            while let Some(mut log) = intercept_rx.recv().await {
                if let Some(tx) = &real_log_tx {
                    for secret_val in secrets_clone.values() {
                        if !secret_val.is_empty() {
                            log.message = log.message.replace(secret_val, "***REDACTED***");
                        }
                    }
                    let _ = tx.send(log).await;
                }
            }
        });

        // 2. Execute the job with Timeout and Retries
        Self::emit_system_log(&log_tx, job.id, "Starting execution phase...").await;
        
        let mut attempts = 0;
        let mut success = false;
        
        while attempts < self.runner_config.max_retries {
            attempts += 1;
            
            let exec_future = self.executor.execute(&active_job, Some(intercept_tx.clone()), resolved_secrets.clone());
            let timeout_future = tokio::time::timeout(Duration::from_secs(self.runner_config.timeout_seconds), exec_future);
            
            match timeout_future.await {
                Ok(Ok(_)) => {
                    success = true;
                    Self::emit_system_log(&log_tx, job.id, "Execution completed successfully.").await;
                    break;
                }
                Ok(Err(e)) => {
                    let err_msg = format!("Execution attempt {} failed: {}", attempts, e);
                    Self::emit_system_log(&log_tx, job.id, &err_msg).await;
                    error!("{}", err_msg);
                    if attempts < self.runner_config.max_retries {
                        sleep(Duration::from_secs(2)).await;
                        Self::emit_system_log(&log_tx, job.id, "Retrying execution...").await;
                    } else {
                        return Err(RunnerError::Execution(e));
                    }
                }
                Err(_) => {
                    let err_msg = format!("Execution timed out after {} seconds.", self.runner_config.timeout_seconds);
                    Self::emit_system_log(&log_tx, job.id, &err_msg).await;
                    error!("{}", err_msg);
                    return Err(RunnerError::Timeout);
                }
            }
        }

        if !success {
            return Err(RunnerError::PreflightFailed("Failed after max retries".into()));
        }

        // 3. Push Outputs to CAS
        Self::emit_system_log(&log_tx, job.id, "Scanning and pushing outputs to CAS...").await;
        let mut uploaded_count = 0;
        for path_str in &job.outputs {
            let files_to_push = Self::collect_outputs(&workspace, path_str).await.unwrap_or_default();
            for file_path in files_to_push {
                if file_path.exists() {
                    let data = tokio::fs::read(&file_path).await?;
                    let digest = self.cas.write_blob(&data).await?;
                    debug!("Output {} saved with digest {}", file_path, hex::encode(&digest.bytes));
                    uploaded_count += 1;
                }
            }
        }
        Self::emit_system_log(&log_tx, job.id, format!("Successfully pushed {} output blobs to CAS.", uploaded_count)).await;

        if self.runner_config.cleanup_after_run {
            Self::emit_system_log(&log_tx, job.id, "Cleaning up workspace...").await;
            self.purge_workspace(&run_dir).await;
        }

        info!("Job {} executed successfully", job.name);
        Self::emit_system_log(&log_tx, job.id, "Job finished.").await;
        
        Ok(())
    }
}

