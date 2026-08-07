use forgeyard_model::{ExecutionSpec, Trigger};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeyardConfig {
    pub version: u32,
    pub project: ProjectConfig,
    #[serde(default)]
    pub pipelines: BTreeMap<String, PipelineConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    #[serde(default)]
    pub triggers: Vec<Trigger>,
    #[serde(default)]
    pub stages: Vec<String>,
    #[serde(default)]
    pub jobs: BTreeMap<String, JobConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    #[serde(default)]
    pub needs: Vec<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub matrix: Option<Vec<String>>,
}

impl ForgeyardConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        ron::from_str(&content).map_err(ConfigError::Ron)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error reading config: {0}")]
    Io(std::io::Error),
    #[error("RON parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),
    #[error("Settings load error: {0}")]
    Settings(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    pub http_port: u16,
    pub quic_port: u16,
    pub db_path: String,
    pub artifact_dir: String,
    pub token: String,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            http_port: 8080,
            quic_port: 5000,
            db_path: ".forgeyard.db".to_string(),
            artifact_dir: ".artifacts".to_string(),
            token: "default_token".to_string(),
        }
    }
}

impl DaemonSettings {
    pub fn load() -> Result<Self, ConfigError> {
        let s = config::Config::builder()
            .set_default("http_port", 8080).unwrap()
            .set_default("quic_port", 5000).unwrap()
            .set_default("db_path", ".forgeyard.db").unwrap()
            .set_default("artifact_dir", ".artifacts").unwrap()
            .set_default("token", "default_token").unwrap()
            .add_source(config::File::with_name("daemon_config").required(false))
            .add_source(config::Environment::with_prefix("FORGEYARD_DAEMON"))
            .build()
            .map_err(|e| ConfigError::Settings(e.to_string()))?;
        
        s.try_deserialize().map_err(|e| ConfigError::Settings(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub daemon_url: Option<String>,
    pub token: String,
    pub max_concurrent_jobs: usize,
}

impl AgentSettings {
    pub fn load() -> Result<Self, ConfigError> {
        let s = config::Config::builder()
            .set_default("max_concurrent_jobs", 4).unwrap()
            .set_default("token", "default_token").unwrap()
            .add_source(config::File::with_name("agent_config").required(false))
            .add_source(config::Environment::with_prefix("FORGEYARD_AGENT"))
            .build()
            .map_err(|e| ConfigError::Settings(e.to_string()))?;
        
        s.try_deserialize().map_err(|e| ConfigError::Settings(e.to_string()))
    }
}
