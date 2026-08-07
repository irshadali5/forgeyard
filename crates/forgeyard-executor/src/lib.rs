use forgeyard_model::{JobIr, LogEvent};
use tokio::sync::mpsc;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Failed to execute command: {0}")]
    ExecutionFailed(std::io::Error),
    #[error("Process exited with non-zero status: {0}")]
    NonZeroExit(i32),
    #[error("Execution method not supported: {0}")]
    UnsupportedExecution(String),
    #[error("Container daemon unavailable: {0}")]
    ContainerDaemonError(String),
    #[error("I/O error during execution setup: {0}")]
    SetupError(#[from] std::io::Error),
}

#[async_trait]
pub trait Executor: Send + Sync {
    async fn execute(
        &self,
        job: &JobIr,
        log_tx: Option<mpsc::Sender<LogEvent>>,
        resolved_secrets: HashMap<String, String>,
    ) -> Result<(), ExecutorError>;
}

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsExecutor;

#[cfg(target_os = "macos")]
pub mod apple;
#[cfg(target_os = "macos")]
pub use apple::AppleExecutor;

// If we are testing on linux/generic, we still compile AppleExecutor to mock/test, 
// so we won't strictly lock it behind target_os = "macos" in our workspace right now.
#[cfg(not(target_os = "macos"))]
pub mod apple;
#[cfg(not(target_os = "macos"))]
pub use apple::AppleExecutor;

pub mod android;
pub use android::AndroidExecutor;

pub mod process;
pub use process::ProcessExecutor;

pub mod container;
pub use container::ContainerExecutor;

