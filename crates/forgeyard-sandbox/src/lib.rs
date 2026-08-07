use async_trait::async_trait;
use forgeyard_executor::{Executor, ExecutorError};
use forgeyard_model::{ExecutionSpec, JobIr, LogEvent, LogStream, NetworkPolicy};
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info, warn};

/// A Linux Sandbox Executor utilizing `bwrap` (Bubblewrap).
/// Provides unprivileged namespace isolation for build jobs.
#[derive(Default)]
pub struct SandboxExecutor;

impl SandboxExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Executor for SandboxExecutor {
    async fn execute(
        &self,
        job: &JobIr,
        log_tx: Option<Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), ExecutorError> {
        info!("Executing Linux Sandbox job: {}", job.name);

        let (program, args, cwd, mut env_vars, network) = match &job.execution {
            ExecutionSpec::Command {
                program,
                arguments,
                working_directory,
                environment,
                network,
                ..
            } => (
                program.clone(),
                arguments.clone(),
                working_directory.clone(),
                environment.clone(),
                network.clone(),
            ),
            ExecutionSpec::ShellScript { script } => {
                let program = "bash".to_string();
                let args = vec!["-c".to_string(), script.clone()];
                let cwd = camino::Utf8PathBuf::from(".");
                (program, args, cwd, BTreeMap::new(), NetworkPolicy::AllowAll)
            }
            _ => {
                return Err(ExecutorError::UnsupportedExecution(
                    "SandboxExecutor only supports Command and ShellScript execution specs.".into(),
                ))
            }
        };

        for (k, v) in resolved_secrets {
            env_vars.insert(k, v);
        }

        let mut cmd = Command::new("bwrap");

        // Essential read-only binds for a functional minimal Linux environment
        cmd.arg("--ro-bind").arg("/usr").arg("/usr");
        cmd.arg("--ro-bind-try").arg("/lib").arg("/lib");
        cmd.arg("--ro-bind-try").arg("/lib64").arg("/lib64");
        cmd.arg("--ro-bind-try").arg("/etc/resolv.conf").arg("/etc/resolv.conf");
        cmd.arg("--ro-bind-try").arg("/etc/ssl").arg("/etc/ssl");
        cmd.arg("--ro-bind-try").arg("/etc/pki").arg("/etc/pki");
        cmd.arg("--ro-bind-try").arg("/etc/ca-certificates").arg("/etc/ca-certificates");

        // Useful API filesystems
        cmd.arg("--proc").arg("/proc");
        cmd.arg("--dev").arg("/dev");

        // Temporary directories
        cmd.arg("--tmpfs").arg("/tmp");
        cmd.arg("--tmpfs").arg("/run");

        // Isolate network if denied
        if matches!(network, NetworkPolicy::Deny) {
            cmd.arg("--unshare-net");
        }

        // Drop unnecessary privileges completely
        cmd.arg("--unshare-user");
        cmd.arg("--unshare-ipc");
        cmd.arg("--unshare-pid");
        cmd.arg("--unshare-uts");
        cmd.arg("--unshare-cgroup");
        cmd.arg("--cap-drop").arg("ALL");
        cmd.arg("--die-with-parent");
        cmd.arg("--new-session");

        // Mount the workspace read-write
        let abs_cwd = if cwd.is_absolute() {
            cwd.clone()
        } else {
            camino::Utf8PathBuf::try_from(std::env::current_dir().unwrap_or_default())
                .unwrap_or_default()
                .join(&cwd)
        };
        cmd.arg("--bind").arg(abs_cwd.as_str()).arg(abs_cwd.as_str());
        
        cmd.arg("--chdir").arg(abs_cwd.as_str());

        // Setup environment
        for (k, v) in &env_vars {
            cmd.arg("--setenv").arg(k).arg(v);
        }

        // Finally, the command to run
        cmd.arg("--");
        cmd.arg(&program);
        cmd.args(&args);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        debug!("Spawning sandbox: {:?}", cmd);
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
            info!("Sandbox Job {} completed successfully.", job.name);
            Ok(())
        } else {
            let code = status.code().unwrap_or(-1);
            warn!("Sandbox Job {} exited with code: {}", job.name, code);
            Err(ExecutorError::NonZeroExit(code))
        }
    }
}
