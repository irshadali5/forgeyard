use async_trait::async_trait;
use std::path::PathBuf;

pub struct ToolchainSpec {
    pub language: String,
    pub version: String,
    pub target: Option<String>,
}

#[async_trait]
pub trait ToolchainProvider: Send + Sync {
    async fn acquire(&self, spec: &ToolchainSpec) -> Result<PathBuf, String>;
}

pub struct LocalToolchainProvider;

#[async_trait]
impl ToolchainProvider for LocalToolchainProvider {
    async fn acquire(&self, spec: &ToolchainSpec) -> Result<PathBuf, String> {
        if spec.language == "rust" {
            if let Some(target) = &spec.target {
                let status = std::process::Command::new("rustup")
                    .arg("target")
                    .arg("add")
                    .arg(target)
                    .arg("--toolchain")
                    .arg(&spec.version)
                    .status()
                    .map_err(|e| e.to_string())?;
                
                if !status.success() {
                    return Err(format!("Failed to add rustup target: {}", target));
                }
            }

            let output = std::process::Command::new("rustc")
                .arg(format!("+{}", spec.version))
                .arg("--print")
                .arg("sysroot")
                .output()
                .map_err(|e| e.to_string())?;
                
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path_str));
            } else {
                return Err(format!("Failed to acquire Rust toolchain: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        Err(format!("Unsupported language: {}", spec.language))
    }
}
