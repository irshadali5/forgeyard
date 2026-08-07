use crate::{Executor, ExecutorError};
use async_trait::async_trait;
use forgeyard_model::{ExecutionSpec, JobIr, LogEvent, LogStream, NetworkPolicy};
use std::collections::HashMap;
use std::process::Stdio;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info, warn};

#[derive(Default)]
pub struct ContainerExecutor {
    engine: String,
}

impl ContainerExecutor {
    pub fn new() -> Self {
        // Detect podman or docker
        let engine = if std::process::Command::new("podman").arg("--version").output().is_ok() {
            "podman".to_string()
        } else {
            "docker".to_string()
        };
        Self { engine }
    }
}

#[async_trait]
impl Executor for ContainerExecutor {
    async fn execute(
        &self,
        job: &JobIr,
        log_tx: Option<Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), ExecutorError> {
        info!("Preparing Linux container executor ({}) for job: {}", self.engine, job.name);

        let (image, program, args, cwd, mut env_vars, network, resources) = match &job.execution {
            ExecutionSpec::Container {
                image,
                program,
                arguments,
                working_directory,
                environment,
                network,
                resources,
            } => {
                (
                    image.name.clone(),
                    program.clone(),
                    arguments.clone(),
                    working_directory.clone(),
                    environment.clone(),
                    network.clone(),
                    resources.clone(),
                )
            }
            _ => {
                return Err(ExecutorError::UnsupportedExecution(
                    "ContainerExecutor only supports ExecutionSpec::Container".into(),
                ))
            }
        };

        // Merge secrets into environment securely
        for (k, v) in resolved_secrets {
            env_vars.insert(k, v);
        }

        let mut cmd = Command::new(&self.engine);
        cmd.arg("run");
        cmd.arg("--rm");
        
        // Networking
        match network {
            NetworkPolicy::Deny => {
                cmd.arg("--network=none");
            }
            NetworkPolicy::DependencyFetchOnly | NetworkPolicy::AllowAll => {
                // In a true implementation, DependencyFetchOnly might use an egress proxy network.
                // For now, we allow networking.
            }
        }

        // Resources
        if let Some(mem) = resources.memory_bytes {
            cmd.arg(format!("--memory={}", mem));
        }
        if let Some(cpu) = resources.cpu_shares {
            cmd.arg(format!("--cpu-shares={}", cpu));
        }

        // Environment variables
        for (k, v) in &env_vars {
            cmd.arg("-e");
            cmd.arg(format!("{}={}", k, v));
        }

        // Working directory (we assume it's mounted or exists in the image)
        cmd.arg("-w");
        cmd.arg(cwd.as_str());

        // Target Image
        cmd.arg(&image);

        // Program and Arguments
        cmd.arg(&program);
        cmd.args(&args);

        // Setup piping
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        debug!("Spawning container: {:?}", cmd);
        let mut child = cmd.spawn().map_err(ExecutorError::ExecutionFailed)?;

        let stdout = child.stdout.take().ok_or_else(|| {
            ExecutorError::SetupError(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to capture stdout",
            ))
        })?;

        let stderr = child.stderr.take().ok_or_else(|| {
            ExecutorError::SetupError(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to capture stderr",
            ))
        })?;

        let job_id = job.id.clone();
        let log_tx_out = log_tx.clone();
        let job_name_out = job.name.clone();

        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut seq = 0;
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("[{}] STDOUT: {}", job_name_out, line);
                if let Some(tx) = &log_tx_out {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                        .to_string();

                    let _ = tx.send(LogEvent {
                        job_id: job_id.clone(),
                        sequence: seq,
                        stream: LogStream::Stdout,
                        timestamp,
                        message: line,
                    }).await;
                }
                seq += 1;
            }
        });

        let job_id_err = job.id.clone();
        let log_tx_err = log_tx;
        let job_name_err = job.name.clone();

        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            let mut seq = 0;
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("[{}] STDERR: {}", job_name_err, line);
                if let Some(tx) = &log_tx_err {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                        .to_string();

                    let _ = tx.send(LogEvent {
                        job_id: job_id_err.clone(),
                        sequence: seq,
                        stream: LogStream::Stderr,
                        timestamp,
                        message: line,
                    }).await;
                }
                seq += 1;
            }
        });

        let status = child.wait().await.map_err(ExecutorError::ExecutionFailed)?;

        let _ = stdout_task.await;
        let _ = stderr_task.await;

        if status.success() {
            info!("Container Job {} completed successfully.", job.name);
            Ok(())
        } else {
            let code = status.code().unwrap_or(-1);
            warn!("Container Job {} exited with code: {}", job.name, code);
            Err(ExecutorError::NonZeroExit(code))
        }
    }
}
