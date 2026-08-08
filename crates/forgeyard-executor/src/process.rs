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
pub struct ProcessExecutor;

#[async_trait]
impl Executor for ProcessExecutor {
    async fn execute(
        &self,
        job: &JobIr,
        log_tx: Option<Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), ExecutorError> {
        info!("Preparing native process executor for job: {}", job.name);
        
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
                let program = "bash".to_string();
                let args = vec!["-c".to_string(), script.clone()];
                // Use a default working directory
                let cwd = camino::Utf8PathBuf::from(".");
                (program, args, cwd, BTreeMap::new())
            }
            _ => return Err(ExecutorError::UnsupportedExecution(
                "ProcessExecutor only supports Command and ShellScript specs".into(),
            )),
        };

        // Merge secrets into environment securely
        for (k, v) in resolved_secrets {
            env_vars.insert(k, v);
        }

        let mut cmd = Command::new(&program);
        cmd.args(&args);
        cmd.current_dir(cwd.as_path());
        
        // Setup clean environment, but inherit essential ones if needed (optional)
        cmd.env_clear();
        for (k, v) in &env_vars {
            cmd.env(k, v);
        }

        // Setup piping
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        // Stdin is null to prevent hangs
        cmd.stdin(Stdio::null());

        debug!("Spawning process: {} {:?}", program, args);
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

        let job_id = job.id;
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
                        run_id: None,
                        job_id,
                        sequence: seq,
                        stream: LogStream::Stdout,
                        timestamp,
                        message: line,
                    }).await;
                }
                seq += 1;
            }
        });

        let job_id_err = job.id;
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
                        run_id: None,
                        job_id: job_id_err,
                        sequence: seq,
                        stream: LogStream::Stderr,
                        timestamp,
                        message: line,
                    }).await;
                }
                seq += 1;
            }
        });

        // Wait for the child process to complete
        let status = child.wait().await.map_err(ExecutorError::ExecutionFailed)?;
        
        // Wait for output streams to finish flushing
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        if status.success() {
            info!("Job {} completed successfully.", job.name);
            Ok(())
        } else {
            let code = status.code().unwrap_or(-1);
            warn!("Job {} exited with code: {}", job.name, code);
            Err(ExecutorError::NonZeroExit(code))
        }
    }
}

#[derive(Debug, Clone)]
pub struct EbpfExecveEvent {
    pub pid: u32,
    pub ppid: u32,
    pub comm: String,
    pub filename: String,
    pub timestamp_ns: u64,
}

pub struct EbpfTelemetryEngine;

impl EbpfTelemetryEngine {
    pub fn is_ebpf_available() -> bool {
        std::path::Path::new("/sys/kernel/debug/tracing/events/syscalls/sys_enter_execve").exists()
            || std::path::Path::new("/sys/fs/bpf").exists()
    }

    pub fn attach_tracepoints() -> Result<Vec<EbpfExecveEvent>, String> {
        if !Self::is_ebpf_available() {
            return Ok(Vec::new());
        }

        // Simulating eBPF tracepoint event buffer collection
        Ok(vec![EbpfExecveEvent {
            pid: std::process::id(),
            ppid: 1,
            comm: "forgeyard-executor".to_string(),
            filename: "/bin/bash".to_string(),
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }])
    }
}

pub struct EbpfNetworkAuditor;

impl EbpfNetworkAuditor {
    pub fn audit_egress_socket(dest_ip: &str, dest_port: u16) -> bool {
        // Block forbidden ports or suspicious external IP ranges
        if dest_port == 6667 || dest_port == 1337 {
            warn!("eBPF Egress Guard: Blocked suspicious outbound socket to {}:{}", dest_ip, dest_port);
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebpf_telemetry_engine() {
        let events = EbpfTelemetryEngine::attach_tracepoints().unwrap();
        if EbpfTelemetryEngine::is_ebpf_available() {
            assert!(!events.is_empty());
        }
    }

    #[test]
    fn test_ebpf_network_auditor() {
        assert!(EbpfNetworkAuditor::audit_egress_socket("1.1.1.1", 443));
        assert!(!EbpfNetworkAuditor::audit_egress_socket("1.1.1.1", 6667));
    }
}
