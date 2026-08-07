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

    pub fn inject_into_config(
        mut config: ForgeyardConfig,
        workspace_root: impl AsRef<Path>,
    ) -> ForgeyardConfig {
        if Self::detect(workspace_root) && !config.pipelines.contains_key("xcode_release") {
            config
                .pipelines
                .insert("xcode_release".to_string(), Self::generate_default_pipeline());
        }
        config
    }
}
