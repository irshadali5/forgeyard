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
}

pub struct GradleTaskBuilder {
    pub build_variant: String,
    pub tasks: Vec<String>,
}

impl GradleTaskBuilder {
    pub fn new(variant: &str) -> Self {
        Self {
            build_variant: variant.to_string(),
            tasks: vec![format!("assemble{}", variant), format!("test{}UnitTest", variant)],
        }
    }

    pub fn to_command(&self) -> Vec<String> {
        let mut cmd = vec!["./gradlew".to_string()];
        cmd.extend(self.tasks.clone());
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_android_adapter_detection() {
        let dir = tempdir().unwrap();
        assert!(!AndroidAdapter::detect(dir.path()));

        std::fs::write(dir.path().join("build.gradle.kts"), b"// gradle").unwrap();
        assert!(AndroidAdapter::detect(dir.path()));
    }

    #[test]
    fn test_gradle_task_builder() {
        let builder = GradleTaskBuilder::new("Release");
        let cmd = builder.to_command();
        assert_eq!(cmd[0], "./gradlew");
        assert_eq!(cmd[1], "assembleRelease");
    }
}
