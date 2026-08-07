use crate::{Executor, ExecutorError};
use async_trait::async_trait;
use forgeyard_model::{ExecutionSpec, JobIr, LogEvent, LogStream};
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info, warn};

// We conditionally include windows-sys so it doesn't break rust-analyzer on non-Windows
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
};

#[derive(Default)]
pub struct WindowsExecutor;

#[async_trait]
impl Executor for WindowsExecutor {
    async fn execute(
        &self,
        job: &JobIr,
        log_tx: Option<Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), ExecutorError> {
        info!("Executing Windows job: {}", job.name);
        
        let (program, args, cwd, mut env_vars) = match &job.execution {
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
                // On Windows, use powershell by default
                let program = "powershell".to_string();
                let args = vec!["-Command".to_string(), script.clone()];
                let cwd = camino::Utf8PathBuf::from(".");
                (program, args, cwd, BTreeMap::new())
            }
            _ => return Err(ExecutorError::UnsupportedExecution(
                "WindowsExecutor only supports Command and ShellScript specs".into(),
            )),
        };

        for (k, v) in resolved_secrets {
            env_vars.insert(k, v);
        }

        let mut cmd = Command::new(&program);
        cmd.args(&args);
        cmd.current_dir(cwd.as_path());
        
        for (k, v) in &env_vars {
            cmd.env(k, v);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        let mut child = cmd.spawn().map_err(ExecutorError::ExecutionFailed)?;

        // Windows Job Object Isolation (only compiled on Windows)
        #[cfg(target_os = "windows")]
        unsafe {
            let job_handle: HANDLE = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job_handle == 0 {
                warn!("Failed to create Windows Job Object, process is not isolated.");
            } else {
                // Determine resource limits from JobIr (if implemented, default none)
                // For now, no memory limits, but we bind the process to the job object so we can kill the whole tree later if cancelled.
                use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};
                let pid = child.id().unwrap_or(0);
                let proc_handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
                if proc_handle != 0 {
                    if AssignProcessToJobObject(job_handle, proc_handle) == 0 {
                        warn!("Failed to assign process to Job Object");
                    }
                    CloseHandle(proc_handle);
                }
            }
        }

        let stdout = child.stdout.take().ok_or_else(|| {
            ExecutorError::SetupError(std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stdout"))
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            ExecutorError::SetupError(std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stderr"))
        })?;

        let job_name_out = job.name.clone();
        let job_name_err = job.name.clone();
        let job_id = job.id.clone();
        let log_tx_out = log_tx.clone();

        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut seq = 0;
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("[{}] STDOUT: {}", job_name_out, line);
                if let Some(tx) = &log_tx_out {
                    let _ = tx.send(LogEvent {
                        job_id: job_id.clone(),
                        sequence: seq,
                        stream: LogStream::Stdout,
                        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                        message: line,
                    }).await;
                }
                seq += 1;
            }
        });

        let job_id_err = job.id.clone();
        let log_tx_err = log_tx;
        
        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            let mut seq = 0;
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("[{}] STDERR: {}", job_name_err, line);
                if let Some(tx) = &log_tx_err {
                    let _ = tx.send(LogEvent {
                        job_id: job_id_err.clone(),
                        sequence: seq,
                        stream: LogStream::Stderr,
                        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
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
            info!("Windows Job {} completed successfully.", job.name);
            Ok(())
        } else {
            let code = status.code().unwrap_or(-1);
            warn!("Windows Job {} failed with exit code: {}", job.name, code);
            Err(ExecutorError::NonZeroExit(code))
        }
    }
}
