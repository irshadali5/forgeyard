use crate::{Executor, ExecutorError};
use async_trait::async_trait;
use forgeyard_model::{ExecutionSpec, JobIr, LogEvent, LogStream};
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info, warn};

#[derive(Default)]
pub struct AppleExecutor;

impl AppleExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Executor for AppleExecutor {
    async fn execute(
        &self,
        job: &JobIr,
        log_tx: Option<Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), ExecutorError> {
        info!("Preparing Apple macOS/iOS executor for job: {}", job.name);
        
        let (mut program, mut args, cwd, mut env_vars) = match &job.execution {
            ExecutionSpec::Command {
                program,
                arguments,
                working_directory,
                environment,
                ..
            } => {
                (program.clone(), arguments.clone(), working_directory.clone(), environment.clone())
            }
            ExecutionSpec::ShellScript { script } => {
                let program = "bash".to_string();
                let args = vec!["-c".to_string(), script.clone()];
                // Use a default working directory
                let cwd = camino::Utf8PathBuf::from(".");
                (program, args, cwd, BTreeMap::new())
            }
            _ => return Err(ExecutorError::UnsupportedExecution(
                "AppleExecutor only supports Command and ShellScript specs".into(),
            )),
        };

        // Apple specific mapping: If the program is `xcodebuild` or `xcrun`, ensure we route it properly.
        // We can prepend `/usr/bin/xcrun` if we are executing a specific tool that requires Developer context.
        if program == "simctl" {
            args.insert(0, program);
            program = "/usr/bin/xcrun".to_string();
        }

        // Merge secrets into environment securely
        for (k, v) in resolved_secrets {
            env_vars.insert(k, v);
        }

        let mut cmd = Command::new(&program);
        cmd.args(&args);
        cmd.current_dir(cwd.as_path());
        
        cmd.env_clear();
        for (k, v) in &env_vars {
            cmd.env(k, v);
        }
        
        // Essential Apple environment variables
        cmd.env("DEVELOPER_DIR", "/Applications/Xcode.app/Contents/Developer");
        cmd.env("HOME", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()));
        cmd.env("PATH", std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin:/usr/sbin:/sbin".to_string()));

        // Setup piping
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        debug!("Spawning Apple process: {} {:?}", program, args);
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
            info!("Apple Job {} completed successfully.", job.name);
            Ok(())
        } else {
            let code = status.code().unwrap_or(-1);
            warn!("Apple Job {} exited with code: {}", job.name, code);
            Err(ExecutorError::NonZeroExit(code))
        }
    }
}
