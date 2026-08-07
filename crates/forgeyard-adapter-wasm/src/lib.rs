use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub struct WasmAdapter;

impl WasmAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        let root = workspace_root.as_ref();
        if let Ok(content) = std::fs::read_to_string(root.join("Cargo.toml")) {
            if content.contains("wasm-bindgen") {
                return true;
            }
        }
        false
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
        let mut jobs = BTreeMap::new();

        jobs.insert(
            "build".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "wasm-pack".to_string(),
                    "build".to_string(),
                    "--target".to_string(),
                    "web".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "test".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "wasm-pack".to_string(),
                    "test".to_string(),
                    "--headless".to_string(),
                    "--chrome".to_string(),
                ],
                matrix: None,
            },
        );

        PipelineConfig {
            triggers: vec![],
            stages: vec![
                "test".to_string(),
                "build".to_string(),
            ],
            jobs,
        }
    }

    pub fn inject_into_config(
        mut config: ForgeyardConfig,
        workspace_root: impl AsRef<Path>,
    ) -> ForgeyardConfig {
        if Self::detect(workspace_root) && !config.pipelines.contains_key("wasm_release") {
            config
                .pipelines
                .insert("wasm_release".to_string(), Self::generate_default_pipeline());
        }
        config
    }
}
