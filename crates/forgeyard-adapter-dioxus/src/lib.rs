use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub struct DioxusAdapter;

impl DioxusAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        workspace_root.as_ref().join("Dioxus.toml").exists()
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
        let mut jobs = BTreeMap::new();

        jobs.insert(
            "build_web".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "dx".to_string(),
                    "build".to_string(),
                    "--release".to_string(),
                    "--platform".to_string(),
                    "web".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "build_desktop".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "dx".to_string(),
                    "build".to_string(),
                    "--release".to_string(),
                    "--platform".to_string(),
                    "desktop".to_string(),
                ],
                matrix: None,
            },
        );

        PipelineConfig {
            triggers: vec![],
            stages: vec![
                "build".to_string(),
            ],
            jobs,
        }
    }

    pub fn inject_into_config(
        mut config: ForgeyardConfig,
        workspace_root: impl AsRef<Path>,
    ) -> ForgeyardConfig {
        if Self::detect(workspace_root) && !config.pipelines.contains_key("dioxus_release") {
            config
                .pipelines
                .insert("dioxus_release".to_string(), Self::generate_default_pipeline());
        }
        config
    }
}
