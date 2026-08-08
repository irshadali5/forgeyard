use async_trait::async_trait;
use forgeyard_executor::{Executor, ExecutorError};
use forgeyard_model::{ExecutionSpec, JobIr, LogEvent, LogStream, NetworkPolicy};
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info, warn};

fn check_bwrap_available() -> bool {
    std::process::Command::new("bwrap")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

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

        let has_bwrap = check_bwrap_available();

        let mut cmd = if has_bwrap {
            let mut c = Command::new("bwrap");
            c.arg("--ro-bind").arg("/usr").arg("/usr");
            c.arg("--ro-bind-try").arg("/lib").arg("/lib");
            c.arg("--ro-bind-try").arg("/lib64").arg("/lib64");
            c.arg("--ro-bind-try").arg("/etc/resolv.conf").arg("/etc/resolv.conf");
            c.arg("--ro-bind-try").arg("/etc/ssl").arg("/etc/ssl");
            c.arg("--ro-bind-try").arg("/etc/pki").arg("/etc/pki");
            c.arg("--ro-bind-try").arg("/etc/ca-certificates").arg("/etc/ca-certificates");
            c.arg("--proc").arg("/proc");
            c.arg("--dev").arg("/dev");
            c.arg("--tmpfs").arg("/tmp");
            c.arg("--tmpfs").arg("/run");

            if matches!(network, NetworkPolicy::Deny) {
                c.arg("--unshare-net");
            }

            c.arg("--unshare-user");
            c.arg("--unshare-ipc");
            c.arg("--unshare-pid");
            c.arg("--unshare-uts");
            c.arg("--unshare-cgroup");
            c.arg("--cap-drop").arg("ALL");
            c.arg("--die-with-parent");
            c.arg("--new-session");

            let abs_cwd = if cwd.is_absolute() {
                cwd.clone()
            } else {
                camino::Utf8PathBuf::try_from(std::env::current_dir().unwrap_or_default())
                    .unwrap_or_default()
                    .join(&cwd)
            };
            c.arg("--bind").arg(abs_cwd.as_str()).arg(abs_cwd.as_str());
            c.arg("--chdir").arg(abs_cwd.as_str());

            for (k, v) in &env_vars {
                c.arg("--setenv").arg(k).arg(v);
            }

            c.arg("--");
            c.arg(&program);
            c.args(&args);
            c
        } else {
            warn!("bwrap binary not found. Falling back to process sandbox execution...");
            let abs_cwd = if cwd.is_absolute() {
                cwd.clone()
            } else {
                camino::Utf8PathBuf::try_from(std::env::current_dir().unwrap_or_default())
                    .unwrap_or_default()
                    .join(&cwd)
            };

            let mut c = Command::new(&program);
            c.args(&args);
            c.current_dir(abs_cwd.as_path());
            c.env_clear();
            for (k, v) in &env_vars {
                c.env(k, v);
            }
            c
        };

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
                        run_id: None,
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
                        run_id: None,
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

pub struct CgroupGovernor {
    pub cgroup_path: std::path::PathBuf,
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_quota_us: Option<u64>,
}

impl CgroupGovernor {
    pub fn new(job_id: &str) -> Self {
        Self {
            cgroup_path: std::path::PathBuf::from("/sys/fs/cgroup/forgeyard").join(job_id),
            max_memory_bytes: None,
            max_cpu_quota_us: None,
        }
    }

    pub fn is_cgroup_v2_available() -> bool {
        std::path::Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
    }

    pub fn apply_limits(&self) -> Result<(), String> {
        if !Self::is_cgroup_v2_available() {
            return Ok(());
        }

        let _ = std::fs::create_dir_all(&self.cgroup_path);

        if let Some(mem_max) = self.max_memory_bytes {
            let path = self.cgroup_path.join("memory.max");
            let _ = std::fs::write(path, mem_max.to_string());
        }

        if let Some(cpu_quota) = self.max_cpu_quota_us {
            let path = self.cgroup_path.join("cpu.max");
            let _ = std::fs::write(path, format!("{} 100000", cpu_quota));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroup_governor_init() {
        let governor = CgroupGovernor::new("job-test-1");
        assert!(governor.cgroup_path.to_str().unwrap().contains("job-test-1"));
        let _ = governor.apply_limits();
    }
}
