use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub struct OciAdapter;

impl OciAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        let root = workspace_root.as_ref();
        root.join("Dockerfile").exists() || root.join("Containerfile").exists()
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
        let mut jobs = BTreeMap::new();

        jobs.insert(
            "build".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "podman".to_string(),
                    "build".to_string(),
                    "-t".to_string(),
                    "app:latest".to_string(),
                    ".".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "scan".to_string(),
            JobConfig {
                needs: vec!["build".to_string()],
                command: vec![
                    "trivy".to_string(),
                    "image".to_string(),
                    "app:latest".to_string(),
                ],
                matrix: None,
            },
        );

        PipelineConfig {
            triggers: vec![],
            stages: vec![
                "build".to_string(),
                "scan".to_string(),
            ],
            jobs,
        }
    }
}

pub struct OciBuildBuilder {
    pub image_name: String,
    pub tag: String,
    pub platforms: Vec<String>,
    pub dockerfile_path: String,
}

impl OciBuildBuilder {
    pub fn new(image_name: &str, tag: &str) -> Self {
        Self {
            image_name: image_name.to_string(),
            tag: tag.to_string(),
            platforms: vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
            dockerfile_path: "Dockerfile".to_string(),
        }
    }

    pub fn to_command(&self) -> Vec<String> {
        let mut cmd = vec![
            "docker".to_string(),
            "buildx".to_string(),
            "build".to_string(),
            "-f".to_string(),
            self.dockerfile_path.clone(),
            "-t".to_string(),
            format!("{}:{}", self.image_name, self.tag),
        ];

        if !self.platforms.is_empty() {
            cmd.push("--platform".to_string());
            cmd.push(self.platforms.join(","));
        }

        cmd.push(".".to_string());
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_oci_adapter_detection() {
        let dir = tempdir().unwrap();
        assert!(!OciAdapter::detect(dir.path()));

        std::fs::write(dir.path().join("Dockerfile"), b"FROM alpine").unwrap();
        assert!(OciAdapter::detect(dir.path()));
    }

    #[test]
    fn test_oci_build_builder() {
        let builder = OciBuildBuilder::new("my-app", "v1.0.0");
        let cmd = builder.to_command();
        assert_eq!(cmd[0], "docker");
        assert_eq!(cmd[6], "my-app:v1.0.0");
    }
}
