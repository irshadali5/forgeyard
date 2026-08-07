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

    pub fn inject_into_config(
        mut config: ForgeyardConfig,
        workspace_root: impl AsRef<Path>,
    ) -> ForgeyardConfig {
        if Self::detect(workspace_root) && !config.pipelines.contains_key("oci_build") {
            config
                .pipelines
                .insert("oci_build".to_string(), Self::generate_default_pipeline());
        }
        config
    }
}
