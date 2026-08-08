use async_trait::async_trait;
use forgeyard_model::{DetectionEvidence, TechnologyKind};
use ignore::WalkBuilder;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Deserialize)]
#[allow(dead_code)]
struct CargoToml {
    package: Option<Package>,
    workspace: Option<Workspace>,
    dependencies: Option<std::collections::BTreeMap<String, toml::Value>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Package {
    name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Workspace {
    members: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum DetectorError {
    #[error("Detection failed: {0}")]
    Failed(String),
}

#[async_trait]
pub trait Detector: Send + Sync {
    async fn detect(&self, workspace_root: &Path) -> Result<Option<DetectionEvidence>, DetectorError>;
}

pub struct ComprehensiveDetector;

#[async_trait]
impl Detector for ComprehensiveDetector {
    async fn detect(&self, workspace_root: &Path) -> Result<Option<DetectionEvidence>, DetectorError> {
        let walker = WalkBuilder::new(workspace_root)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .build();

        let mut files_to_process = Vec::new();
        for entry in walker.flatten() {
            files_to_process.push(entry.into_path());
        }

        let (has_rust, has_node, has_docker, has_ios, has_android, frameworks) = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;
            
            files_to_process.into_par_iter().fold(
                || (false, false, false, false, false, HashSet::new()),
                |mut acc, path| {
                    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                    match file_name.as_ref() {
                        "Cargo.toml" => {
                            acc.0 = true;
                            acc.5.insert("cargo".to_string());
                            #[allow(clippy::collapsible_if)]
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(parsed) = toml::from_str::<CargoToml>(&content) {
                                    if let Some(deps) = parsed.dependencies {
                                        if deps.contains_key("dioxus") { acc.5.insert("dioxus".to_string()); }
                                        if deps.contains_key("axum") { acc.5.insert("axum".to_string()); }
                                        if deps.contains_key("actix-web") { acc.5.insert("actix-web".to_string()); }
                                        if deps.contains_key("tokio") { acc.5.insert("tokio".to_string()); }
                                        if deps.contains_key("tauri") { acc.5.insert("tauri".to_string()); }
                                    }
                                }
                            }
                        }
                        "package.json" => {
                            acc.1 = true;
                            acc.5.insert("npm".to_string());
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if content.contains("react") { acc.5.insert("react".to_string()); }
                                if content.contains("vue") { acc.5.insert("vue".to_string()); }
                                if content.contains("next") { acc.5.insert("nextjs".to_string()); }
                            }
                        }
                        "Dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => {
                            acc.2 = true;
                            acc.5.insert("docker".to_string());
                        }
                        "Podfile" | "Package.swift" => {
                            acc.3 = true;
                            acc.5.insert("ios".to_string());
                        }
                        "build.gradle" | "build.gradle.kts" => {
                            acc.4 = true;
                            acc.5.insert("android".to_string());
                        }
                        _ => {
                            if file_name.ends_with(".xcodeproj") || file_name.ends_with(".xcworkspace") {
                                acc.3 = true;
                                acc.5.insert("xcode".to_string());
                            }
                        }
                    }
                    acc
                }
            ).reduce(
                || (false, false, false, false, false, HashSet::new()),
                |mut a, b| {
                    a.0 |= b.0; // has_rust
                    a.1 |= b.1; // has_node
                    a.2 |= b.2; // has_docker
                    a.3 |= b.3; // has_ios
                    a.4 |= b.4; // has_android
                    a.5.extend(b.5); // frameworks
                    a
                }
            )
        }).await.map_err(|e| DetectorError::Failed(e.to_string()))?;

        if !has_rust && !has_node && !has_docker && !has_ios && !has_android {
            return Ok(None);
        }

        let kind = if has_rust {
            TechnologyKind::Rust
        } else if has_node {
            TechnologyKind::Node
        } else {
            TechnologyKind::Unknown
        };

        let mut intended_targets = Vec::new();
        if frameworks.contains("axum") || frameworks.contains("actix-web") || has_docker {
            intended_targets.push("linux-x86_64".to_string());
        }
        if frameworks.contains("dioxus") || frameworks.contains("react") || frameworks.contains("vue") || frameworks.contains("nextjs") {
            intended_targets.push("web".to_string());
        }
        if has_ios || frameworks.contains("xcode") {
            intended_targets.push("ios".to_string());
        }
        if has_android {
            intended_targets.push("android".to_string());
        }
        
        if intended_targets.is_empty() {
            intended_targets.push("linux-x86_64".to_string());
        }

        let test_suites = if has_rust { vec!["cargo test".to_string()] } else { vec![] };

        Ok(Some(DetectionEvidence {
            kind,
            frameworks: frameworks.into_iter().collect(),
            intended_targets,
            test_suites,
        }))
    }
}

pub struct WorkspaceAnalyzer {
    detectors: Vec<Box<dyn Detector>>,
}

impl Default for WorkspaceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceAnalyzer {
    pub fn new() -> Self {
        Self {
            detectors: vec![Box::new(ComprehensiveDetector)],
        }
    }

    pub async fn analyze(&self, workspace_root: &Path) -> Result<Vec<DetectionEvidence>, DetectorError> {
        let mut evidence = Vec::new();
        for detector in &self.detectors {
            if let Some(e) = detector.detect(workspace_root).await? {
                evidence.push(e);
            }
        }
        Ok(evidence)
    }
}
