use forgeyard_model::Trigger;
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

pub struct GitHubWorkflowConverter;

impl GitHubWorkflowConverter {
    pub fn convert_yaml(workflow_name: &str, yaml_content: &str) -> ForgeyardConfig {
        let mut jobs = BTreeMap::new();
        let mut current_job_name = String::new();
        let mut current_cmds = Vec::new();
        let mut current_needs = Vec::new();
        let mut current_matrix = Vec::new();
        let mut stages = Vec::new();

        for line in yaml_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("run:") {
                let cmd = trimmed.trim_start_matches("run:").trim();
                current_cmds.push(cmd.to_string());
            } else if trimmed.starts_with("- run:") {
                let cmd = trimmed.trim_start_matches("- run:").trim();
                current_cmds.push(cmd.to_string());
            } else if trimmed.starts_with("needs:") {
                let dep_str = trimmed.trim_start_matches("needs:").trim();
                if dep_str.starts_with('[') && dep_str.ends_with(']') {
                    let inner = &dep_str[1..dep_str.len() - 1];
                    for item in inner.split(',') {
                        let clean = item.trim().trim_matches('\'').trim_matches('"');
                        if !clean.is_empty() {
                            current_needs.push(clean.to_string());
                        }
                    }
                } else if !dep_str.is_empty() {
                    current_needs.push(dep_str.to_string());
                }
            } else if trimmed.starts_with("matrix:") {
                current_matrix.push("target: [x86_64, arm64]".to_string());
            } else if trimmed.ends_with(':') && !trimmed.starts_with('-') && !trimmed.contains(' ') 
                && trimmed != "jobs:" && trimmed != "steps:" && trimmed != "strategy:" && trimmed != "matrix:" && trimmed != "env:" {
                
                if !current_job_name.is_empty() && !current_cmds.is_empty() {
                    stages.push(current_job_name.clone());
                    jobs.insert(current_job_name.clone(), JobConfig {
                        needs: std::mem::take(&mut current_needs),
                        command: std::mem::take(&mut current_cmds),
                        matrix: if current_matrix.is_empty() { None } else { Some(std::mem::take(&mut current_matrix)) },
                    });
                }
                current_job_name = trimmed.trim_end_matches(':').to_string();
            }
        }

        if !current_job_name.is_empty() {
            if current_cmds.is_empty() {
                current_cmds.push("cargo build --release".to_string());
            }
            stages.push(current_job_name.clone());
            jobs.insert(current_job_name, JobConfig {
                needs: current_needs,
                command: current_cmds,
                matrix: if current_matrix.is_empty() { None } else { Some(current_matrix) },
            });
        }

        if jobs.is_empty() {
            jobs.insert("build".to_string(), JobConfig {
                needs: vec![],
                command: vec!["cargo build".to_string()],
                matrix: None,
            });
            stages.push("build".to_string());
        }

        let mut pipelines = BTreeMap::new();
        pipelines.insert("default".to_string(), PipelineConfig {
            triggers: vec![Trigger::GitCommit],
            stages,
            jobs,
        });

        ForgeyardConfig {
            version: 1,
            project: ProjectConfig { name: workflow_name.to_string() },
            pipelines,
        }
    }
}

pub struct GitLabCIConverter;

impl GitLabCIConverter {
    pub fn convert_yaml(project_name: &str, yaml_content: &str) -> ForgeyardConfig {
        let mut jobs = BTreeMap::new();
        let mut current_job_name = String::new();
        let mut current_cmds = Vec::new();
        let mut current_needs = Vec::new();
        let mut stages = Vec::new();

        for line in yaml_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") && !trimmed.starts_with("- stage") && !trimmed.starts_with("- needs") {
                let cmd = trimmed.trim_start_matches("- ").trim();
                current_cmds.push(cmd.to_string());
            } else if trimmed.starts_with("needs:") {
                let dep_str = trimmed.trim_start_matches("needs:").trim();
                if dep_str.starts_with('[') && dep_str.ends_with(']') {
                    let inner = &dep_str[1..dep_str.len() - 1];
                    for item in inner.split(',') {
                        let clean = item.trim().trim_matches('\'').trim_matches('"');
                        if !clean.is_empty() {
                            current_needs.push(clean.to_string());
                        }
                    }
                }
            } else if trimmed.ends_with(':') && !trimmed.starts_with('-') && !trimmed.contains(' ') 
                && trimmed != "script:" && trimmed != "stages:" && trimmed != "before_script:" && trimmed != "after_script:" {
                
                if !current_job_name.is_empty() && !current_cmds.is_empty() {
                    stages.push(current_job_name.clone());
                    jobs.insert(current_job_name.clone(), JobConfig {
                        needs: std::mem::take(&mut current_needs),
                        command: std::mem::take(&mut current_cmds),
                        matrix: None,
                    });
                }
                current_job_name = trimmed.trim_end_matches(':').to_string();
            }
        }

        if !current_job_name.is_empty() {
            if current_cmds.is_empty() {
                current_cmds.push("cargo test".to_string());
            }
            stages.push(current_job_name.clone());
            jobs.insert(current_job_name, JobConfig {
                needs: current_needs,
                command: current_cmds,
                matrix: None,
            });
        }

        if jobs.is_empty() {
            jobs.insert("test".to_string(), JobConfig {
                needs: vec![],
                command: vec!["cargo test --all".to_string()],
                matrix: None,
            });
            stages.push("test".to_string());
        }

        let mut pipelines = BTreeMap::new();
        pipelines.insert("default".to_string(), PipelineConfig {
            triggers: vec![Trigger::GitCommit],
            stages,
            jobs,
        });

        ForgeyardConfig {
            version: 1,
            project: ProjectConfig { name: project_name.to_string() },
            pipelines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_workflow_converter() {
        let yaml = r#"
name: CI
jobs:
  build:
    run: cargo build --release
    run: cargo test
  deploy:
    needs: [build]
    run: cargo deploy
"#;
        let config = GitHubWorkflowConverter::convert_yaml("my-app", yaml);
        assert_eq!(config.project.name, "my-app");
        let default_pipe = config.pipelines.get("default").unwrap();
        assert_eq!(default_pipe.jobs.len(), 2);
        assert!(default_pipe.jobs.contains_key("build"));
        assert!(default_pipe.jobs.contains_key("deploy"));
        let deploy_job = default_pipe.jobs.get("deploy").unwrap();
        assert_eq!(deploy_job.needs, vec!["build"]);
    }

    #[test]
    fn test_gitlab_ci_converter() {
        let yaml = r#"
test_job:
  script:
    - cargo test --all
build_job:
  needs: [test_job]
  script:
    - cargo build --release
"#;
        let config = GitLabCIConverter::convert_yaml("gitlab-app", yaml);
        assert_eq!(config.project.name, "gitlab-app");
        let default_pipe = config.pipelines.get("default").unwrap();
        assert_eq!(default_pipe.jobs.len(), 2);
    }
}
