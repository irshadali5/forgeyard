use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub struct AndroidAdapter;

impl AndroidAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        let root = workspace_root.as_ref();
        root.join("build.gradle").exists() || root.join("build.gradle.kts").exists()
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
        let mut jobs = BTreeMap::new();

        jobs.insert(
            "lint".to_string(),
            JobConfig {
                needs: vec![],
                command: vec!["./gradlew".to_string(), "lint".to_string()],
                matrix: None,
            },
        );

        jobs.insert(
            "test".to_string(),
            JobConfig {
                needs: vec!["lint".to_string()],
                command: vec!["./gradlew".to_string(), "testDebugUnitTest".to_string()],
                matrix: None,
            },
        );

        jobs.insert(
            "build_apk".to_string(),
            JobConfig {
                needs: vec!["test".to_string()],
                command: vec!["./gradlew".to_string(), "assembleRelease".to_string()],
                matrix: None,
            },
        );

        jobs.insert(
            "build_aab".to_string(),
            JobConfig {
                needs: vec!["test".to_string()],
                command: vec!["./gradlew".to_string(), "bundleRelease".to_string()],
                matrix: None,
            },
        );

        PipelineConfig {
            triggers: vec![],
            stages: vec![
                "validate".to_string(),
                "build".to_string(),
            ],
            jobs,
        }
    }

    pub fn inject_into_config(
        mut config: ForgeyardConfig,
        workspace_root: impl AsRef<Path>,
    ) -> ForgeyardConfig {
        if Self::detect(workspace_root) && !config.pipelines.contains_key("android_release") {
            config
                .pipelines
                .insert("android_release".to_string(), Self::generate_default_pipeline());
        }
        config
    }
}
