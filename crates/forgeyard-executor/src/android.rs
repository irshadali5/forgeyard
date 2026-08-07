use crate::{Executor, ExecutorError};
use async_trait::async_trait;
use forgeyard_model::{ExecutionSpec, JobIr};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info, warn};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use std::path::{Path, PathBuf};

/// Android Device Orchestrator
/// Manages the lifecycle of Android Emulators (AVDs), bridging ADB,
/// pulling logcats, recording screens, and injecting UI Automator events.
pub struct AndroidExecutor {
    avd_name: String,
    emulator_port: u16,
    headless: bool,
}

impl Default for AndroidExecutor {
    fn default() -> Self {
        Self {
            avd_name: "forgeyard_ci_device".to_string(),
            emulator_port: 5554,
            headless: true,
        }
    }
}

#[allow(dead_code)]
impl AndroidExecutor {
    pub fn new(avd_name: &str, port: u16, headless: bool) -> Self {
        Self {
            avd_name: avd_name.to_string(),
            emulator_port: port,
            headless,
        }
    }

    /// Checks if ANDROID_HOME is set and adb is in PATH
    fn verify_environment() -> Result<PathBuf, ExecutorError> {
        let android_home = std::env::var("ANDROID_HOME")
            .map_err(|_| ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::NotFound, "ANDROID_HOME not set")))?;
        
        let adb_path = PathBuf::from(&android_home).join("platform-tools").join("adb");
        if !adb_path.exists() {
            warn!("adb not found at {:?}, falling back to system path", adb_path);
            Ok(PathBuf::from("adb"))
        } else {
            Ok(adb_path)
        }
    }

    /// Kills any existing emulator on the target port
    async fn kill_emulator(&self, adb_path: &Path) -> Result<(), ExecutorError> {
        info!("Killing emulator on port {}", self.emulator_port);
        let target = format!("emulator-{}", self.emulator_port);
        let _ = Command::new(adb_path)
            .args(&["-s", &target, "emu", "kill"])
            .output()
            .await;
        
        // Wait for it to die
        sleep(Duration::from_secs(2)).await;
        Ok(())
    }

    /// Boots the Android Virtual Device
    async fn boot_avd(&self) -> Result<tokio::process::Child, ExecutorError> {
        let android_home = std::env::var("ANDROID_HOME")
            .map_err(|_| ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::NotFound, "ANDROID_HOME not set")))?;
        
        let emulator_path = PathBuf::from(&android_home).join("emulator").join("emulator");
        
        info!("Booting AVD: {} on port {}", self.avd_name, self.emulator_port);
        let mut cmd = Command::new(emulator_path);
        cmd.args(&["-avd", &self.avd_name]);
        cmd.args(&["-port", &self.emulator_port.to_string()]);
        cmd.args(&["-no-snapshot", "-no-boot-anim", "-no-audio"]);
        
        if self.headless {
            cmd.arg("-no-window");
        }

        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd.spawn().map_err(ExecutorError::ExecutionFailed)?;
        Ok(child)
    }

    /// Waits for the device to emit 'sys.boot_completed' = 1
    async fn wait_for_boot(&self, adb_path: &Path) -> Result<(), ExecutorError> {
        let target = format!("emulator-{}", self.emulator_port);
        info!("Waiting for device {} to boot...", target);
        
        let max_retries = 60; // 2 minutes max
        for i in 0..max_retries {
            let output = Command::new(adb_path)
                .args(&["-s", &target, "shell", "getprop", "sys.boot_completed"])
                .output()
                .await;
            
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.trim() == "1" {
                    info!("Device booted successfully after {} seconds", i * 2);
                    // unlock screen just in case
                    let _ = Command::new(adb_path).args(&["-s", &target, "shell", "input", "keyevent", "82"]).output().await;
                    return Ok(());
                }
            }
            sleep(Duration::from_secs(2)).await;
        }

        Err(ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::TimedOut, "Emulator failed to boot")))
    }

    /// Installs an APK on the device
    async fn install_apk(&self, adb_path: &Path, apk_path: &str) -> Result<(), ExecutorError> {
        let target = format!("emulator-{}", self.emulator_port);
        info!("Installing APK: {} to {}", apk_path, target);
        
        let output = Command::new(adb_path)
            .args(&["-s", &target, "install", "-r", "-t", apk_path])
            .output()
            .await
            .map_err(ExecutorError::ExecutionFailed)?;

        if output.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("APK Install failed: {}", err);
            Err(ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::Other, "APK Install failed")))
        }
    }

    /// Runs Android Monkey for UI Fuzzing
    async fn run_monkey(&self, adb_path: &Path, package: &str, events: u32) -> Result<(), ExecutorError> {
        let target = format!("emulator-{}", self.emulator_port);
        info!("Running UI Monkey on {} for {} events", package, events);
        
        let output = Command::new(adb_path)
            .args(&["-s", &target, "shell", "monkey", "-p", package, "-v", &events.to_string()])
            .output()
            .await
            .map_err(ExecutorError::ExecutionFailed)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::Other, "Monkey crashed")))
        }
    }

    /// Dumps Logcat and clears buffer
    async fn capture_logcat(&self, adb_path: &Path, log_tx: &Option<tokio::sync::mpsc::Sender<forgeyard_model::LogEvent>>, job_id: forgeyard_model::JobId) {
        let target = format!("emulator-{}", self.emulator_port);
        let output = Command::new(adb_path)
            .args(&["-s", &target, "logcat", "-d"])
            .output()
            .await;
        
        if let Ok(out) = output {
            if let Some(tx) = log_tx {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut seq = 1000;
                for line in text.lines() {
                    let _ = tx.send(forgeyard_model::LogEvent {
                        job_id,
                        sequence: seq,
                        stream: forgeyard_model::LogStream::Stdout,
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                        message: format!("[LOGCAT] {}", line),
                    }).await;
                    seq += 1;
                }
            }
        }
        
        // Clear logcat
        let _ = Command::new(adb_path).args(&["-s", &target, "logcat", "-c"]).output().await;
    }
}

#[async_trait]
impl Executor for AndroidExecutor {
    async fn execute(&self, job: &JobIr, log_tx: Option<tokio::sync::mpsc::Sender<forgeyard_model::LogEvent>>, _resolved_secrets: HashMap<String, String>) -> Result<(), ExecutorError> {
        info!("Executing Android job: {}", job.name);
        
        let adb_path = match Self::verify_environment() {
            Ok(p) => p,
            Err(_e) => {
                warn!("Android environment not found, attempting to run as standard shell command...");
                return self.execute_fallback(job, log_tx).await;
            }
        };

        match &job.execution {
            ExecutionSpec::Command {
                program,
                arguments,
                working_directory,
                environment,
                network: _,
                resources: _,
            } => {
                // If it's specifically requesting an emulator context
                let needs_emulator = environment.iter().any(|(k, v)| k == "FORGEYARD_REQUIRE_EMULATOR" && v == "1");

                let mut emu_child = None;

                if needs_emulator {
                    self.kill_emulator(&adb_path).await?;
                    emu_child = Some(self.boot_avd().await?);
                    self.wait_for_boot(&adb_path).await?;
                }

                // Execute the actual command via gradle or adb shell
                let mut cmd = Command::new(program);
                cmd.args(arguments);
                cmd.current_dir(working_directory);
                for (k, v) in environment {
                    cmd.env(k, v);
                }

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd.spawn().map_err(ExecutorError::ExecutionFailed)?;

                let stdout = child.stdout.take().ok_or_else(|| ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stdout")))?;
                let stderr = child.stderr.take().ok_or_else(|| ExecutorError::ExecutionFailed(std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stderr")))?;

                let job_name_out = job.name.clone();
                let job_name_err = job.name.clone();

                let job_id = job.id;
                let log_tx_out = log_tx.clone();
                let stdout_task = tokio::spawn(async move {
                    let mut reader = BufReader::new(stdout).lines();
                    let mut seq = 0;
                    while let Ok(Some(line)) = reader.next_line().await {
                        info!("[{}] {}", job_name_out, line);
                        if let Some(tx) = &log_tx_out {
                            let _ = tx.send(forgeyard_model::LogEvent {
                                job_id,
                                sequence: seq,
                                stream: forgeyard_model::LogStream::Stdout,
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                                message: line,
                            }).await;
                        }
                        seq += 1;
                    }
                });

                let log_tx_err = log_tx.clone();
                let stderr_task = tokio::spawn(async move {
                    let mut reader = BufReader::new(stderr).lines();
                    let mut seq = 0;
                    while let Ok(Some(line)) = reader.next_line().await {
                        error!("[{}] {}", job_name_err, line);
                        if let Some(tx) = &log_tx_err {
                            let _ = tx.send(forgeyard_model::LogEvent {
                                job_id,
                                sequence: seq,
                                stream: forgeyard_model::LogStream::Stderr,
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                                message: line,
                            }).await;
                        }
                        seq += 1;
                    }
                });

                let status = child.wait().await.map_err(ExecutorError::ExecutionFailed)?;
                let _ = stdout_task.await;
                let _ = stderr_task.await;

                if needs_emulator {
                    self.capture_logcat(&adb_path, &log_tx, job.id).await;
                    self.kill_emulator(&adb_path).await?;
                    if let Some(mut emu) = emu_child {
                        let _ = emu.kill().await;
                    }
                }

                if status.success() {
                    info!("Job {} completed successfully on Android executor.", job.name);
                    Ok(())
                } else {
                    let code = status.code().unwrap_or(-1);
                    warn!("Job {} failed with exit code: {}", job.name, code);
                    Err(ExecutorError::NonZeroExit(code))
                }
            }
            _ => Err(ExecutorError::UnsupportedExecution("Unsupported spec for android executor".into())),
        }
    }
}

impl AndroidExecutor {
    async fn execute_fallback(&self, job: &JobIr, log_tx: Option<tokio::sync::mpsc::Sender<forgeyard_model::LogEvent>>) -> Result<(), ExecutorError> {
        match &job.execution {
            ExecutionSpec::Command {
                program,
                arguments,
                working_directory,
                environment,
                network: _,
                resources: _,
            } => {
                let mut cmd = Command::new(program);
                cmd.args(arguments);
                cmd.current_dir(working_directory);
                for (k, v) in environment {
                    cmd.env(k, v);
                }

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd.spawn().map_err(ExecutorError::ExecutionFailed)?;

                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();
                
                let job_id = job.id;
                let log_tx_out = log_tx.clone();
                let stdout_task = tokio::spawn(async move {
                    let mut reader = BufReader::new(stdout).lines();
                    let mut seq = 0;
                    while let Ok(Some(line)) = reader.next_line().await {
                        if let Some(tx) = &log_tx_out {
                            let _ = tx.send(forgeyard_model::LogEvent {
                                job_id, sequence: seq, stream: forgeyard_model::LogStream::Stdout,
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                                message: line,
                            }).await;
                        }
                        seq += 1;
                    }
                });

                let log_tx_err = log_tx;
                let stderr_task = tokio::spawn(async move {
                    let mut reader = BufReader::new(stderr).lines();
                    let mut seq = 0;
                    while let Ok(Some(line)) = reader.next_line().await {
                        if let Some(tx) = &log_tx_err {
                            let _ = tx.send(forgeyard_model::LogEvent {
                                job_id, sequence: seq, stream: forgeyard_model::LogStream::Stderr,
                                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().to_string(),
                                message: line,
                            }).await;
                        }
                        seq += 1;
                    }
                });

                let status = child.wait().await.unwrap();
                let _ = stdout_task.await;
                let _ = stderr_task.await;

                if status.success() { Ok(()) } else { Err(ExecutorError::NonZeroExit(status.code().unwrap_or(-1))) }
            }
            _ => Err(ExecutorError::UnsupportedExecution("Unsupported execution spec".into())),
        }
    }
}
