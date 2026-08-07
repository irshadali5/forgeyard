use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub struct XcodeAdapter;

impl XcodeAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        let root = workspace_root.as_ref();
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Some(name_str) = file_name.to_str() {
                    if name_str.ends_with(".xcodeproj") || name_str.ends_with(".xcworkspace") {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
        let mut jobs = BTreeMap::new();

        jobs.insert(
            "test".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "xcodebuild".to_string(),
                    "test".to_string(),
                    "-scheme".to_string(),
                    "App".to_string(),
                    "-destination".to_string(),
                    "platform=iOS Simulator,name=iPhone 15".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "archive".to_string(),
            JobConfig {
                needs: vec!["test".to_string()],
                command: vec![
                    "xcodebuild".to_string(),
                    "archive".to_string(),
                    "-scheme".to_string(),
                    "App".to_string(),
                    "-archivePath".to_string(),
                    "build/App.xcarchive".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "export_ipa".to_string(),
            JobConfig {
                needs: vec!["archive".to_string()],
                command: vec![
                    "xcodebuild".to_string(),
                    "-exportArchive".to_string(),
                    "-archivePath".to_string(),
                    "build/App.xcarchive".to_string(),
                    "-exportPath".to_string(),
                    "build/export".to_string(),
                    "-exportOptionsPlist".to_string(),
                    "ExportOptions.plist".to_string(),
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
}

pub struct XcodebuildBuilder {
    pub scheme: String,
    pub destination: String,
    pub configuration: String,
}

impl XcodebuildBuilder {
    pub fn new(scheme: &str, destination: &str) -> Self {
        Self {
            scheme: scheme.to_string(),
            destination: destination.to_string(),
            configuration: "Release".to_string(),
        }
    }

    pub fn to_test_command(&self) -> Vec<String> {
        vec![
            "xcodebuild".to_string(),
            "test".to_string(),
            "-scheme".to_string(),
            self.scheme.clone(),
            "-destination".to_string(),
            self.destination.clone(),
            "-configuration".to_string(),
            self.configuration.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_xcode_adapter_detection() {
        let dir = tempdir().unwrap();
        assert!(!XcodeAdapter::detect(dir.path()));

        std::fs::create_dir(dir.path().join("App.xcodeproj")).unwrap();
        assert!(XcodeAdapter::detect(dir.path()));
    }

    #[test]
    fn test_xcodebuild_builder() {
        let builder = XcodebuildBuilder::new("App", "platform=iOS Simulator,name=iPhone 15");
        let cmd = builder.to_test_command();
        assert_eq!(cmd[0], "xcodebuild");
        assert_eq!(cmd[3], "App");
    }
}
