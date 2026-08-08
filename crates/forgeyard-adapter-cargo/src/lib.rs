use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use guppy::graph::{DependencyDirection, PackageGraph, PackageMetadata};
use guppy::MetadataCommand;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoPackageInfo {
    pub name: String,
    pub version: String,
    pub manifest_path: String,
    pub is_workspace_member: bool,
    pub direct_dependencies: Vec<String>,
    pub transitive_dependency_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoGraphSummary {
    pub total_packages: usize,
    pub workspace_members: Vec<String>,
    pub topological_order: Vec<String>,
    pub detected_frameworks: Vec<String>,
}

/// High-performance Cargo dependency graph tracker and query engine powered by `guppy`.
pub struct CargoGraphTracker {
    graph: Option<PackageGraph>,
    root_path: PathBuf,
}

impl CargoGraphTracker {
    /// Creates a new `CargoGraphTracker` by inspecting the Cargo workspace at `workspace_root`.
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        let root_path = workspace_root.as_ref().to_path_buf();
        let graph = MetadataCommand::new()
            .current_dir(&root_path)
            .build_graph()
            .ok();

        Self { graph, root_path }
    }

    /// Creates a `CargoGraphTracker` from raw cargo metadata JSON output.
    pub fn from_json(json_str: &str, workspace_root: impl AsRef<Path>) -> Result<Self, guppy::Error> {
        let graph = PackageGraph::from_json(json_str)?;
        Ok(Self {
            graph: Some(graph),
            root_path: workspace_root.as_ref().to_path_buf(),
        })
    }

    /// Checks if a valid `PackageGraph` is loaded.
    pub fn is_available(&self) -> bool {
        self.graph.is_some()
    }

    /// Returns the root path of the workspace.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Returns metadata for all workspace member packages.
    pub fn workspace_members(&self) -> Vec<CargoPackageInfo> {
        let Some(graph) = &self.graph else {
            return Vec::new();
        };

        graph
            .workspace()
            .iter()
            .map(|pkg| self.package_metadata_to_info(pkg, true))
            .collect()
    }

    /// Returns all package names in the workspace sorted in topological dependency order.
    pub fn topological_workspace_order(&self) -> Vec<String> {
        let Some(graph) = &self.graph else {
            return Vec::new();
        };

        let set = graph.query_workspace().resolve();
        let workspace_ids: HashSet<_> = graph.workspace().iter().map(|p| p.id()).collect();
        set.package_ids(DependencyDirection::Forward)
            .filter(|id| workspace_ids.contains(id))
            .filter_map(|id| graph.metadata(id).ok())
            .map(|pkg| pkg.name().to_string())
            .collect()
    }

    /// Queries all transitive dependencies for a package by name.
    pub fn query_transitive_deps(&self, package_name: &str) -> Vec<String> {
        let Some(graph) = &self.graph else {
            return Vec::new();
        };

        let Some(pkg) = graph.packages().find(|p| p.name() == package_name) else {
            return Vec::new();
        };

        if let Ok(query) = graph.query_forward([pkg.id()]) {
            let set = query.resolve();
            return set
                .package_ids(DependencyDirection::Forward)
                .filter_map(|id| graph.metadata(id).ok())
                .map(|p| p.name().to_string())
                .collect();
        }
        Vec::new()
    }

    /// Queries all reverse dependencies (downstream packages depending on `package_name`).
    pub fn query_reverse_deps(&self, package_name: &str) -> Vec<String> {
        let Some(graph) = &self.graph else {
            return Vec::new();
        };

        let Some(pkg) = graph.packages().find(|p| p.name() == package_name) else {
            return Vec::new();
        };

        if let Ok(query) = graph.query_reverse([pkg.id()]) {
            let set = query.resolve();
            return set
                .package_ids(DependencyDirection::Forward)
                .filter_map(|id| graph.metadata(id).ok())
                .map(|p| p.name().to_string())
                .collect();
        }
        Vec::new()
    }

    /// Determines which workspace packages are impacted by a list of modified file paths.
    pub fn query_impacted_workspace_packages(&self, changed_files: &[impl AsRef<Path>]) -> Vec<String> {
        let Some(graph) = &self.graph else {
            return Vec::new();
        };

        let mut affected_pkg_ids = HashSet::new();

        for file in changed_files {
            let file_path = file.as_ref();
            for pkg in graph.workspace().iter() {
                if let Some(parent) = pkg.manifest_path().parent() {
                    if file_path.starts_with(parent.as_std_path()) {
                        affected_pkg_ids.insert(pkg.id());
                    }
                }
            }
        }

        if affected_pkg_ids.is_empty() {
            return Vec::new();
        }

        if let Ok(query) = graph.query_reverse(affected_pkg_ids) {
            let set = query.resolve();
            let workspace_ids: HashSet<_> = graph.workspace().iter().map(|p| p.id()).collect();
            return set
                .package_ids(DependencyDirection::Forward)
                .filter(|id| workspace_ids.contains(id))
                .filter_map(|id| graph.metadata(id).ok())
                .map(|pkg| pkg.name().to_string())
                .collect();
        }
        Vec::new()
    }

    /// Queries the graph to detect popular frameworks and ecosystem libraries.
    pub fn detect_frameworks(&self) -> HashSet<String> {
        let mut frameworks = HashSet::new();
        let Some(graph) = &self.graph else {
            return frameworks;
        };

        let known = [
            "dioxus", "axum", "actix-web", "tokio", "tauri", "sqlx", "tonic", "diesel",
            "rayon", "quinn", "reqwest", "yew", "leptos", "warp", "rocket", "serde",
        ];

        for pkg in graph.packages() {
            let name = pkg.name();
            if known.contains(&name) {
                frameworks.insert(name.to_string());
            }
        }

        frameworks
    }

    /// Returns a structured graph summary.
    pub fn summary(&self) -> CargoGraphSummary {
        let Some(graph) = &self.graph else {
            return CargoGraphSummary {
                total_packages: 0,
                workspace_members: Vec::new(),
                topological_order: Vec::new(),
                detected_frameworks: Vec::new(),
            };
        };

        CargoGraphSummary {
            total_packages: graph.package_count(),
            workspace_members: graph.workspace().iter().map(|p| p.name().to_string()).collect(),
            topological_order: self.topological_workspace_order(),
            detected_frameworks: self.detect_frameworks().into_iter().collect(),
        }
    }

    fn package_metadata_to_info(&self, pkg: PackageMetadata, is_workspace_member: bool) -> CargoPackageInfo {
        let direct_deps = pkg
            .direct_links()
            .map(|link| link.to().name().to_string())
            .collect();

        let transitive_count = if let Ok(query) = self.graph.as_ref().unwrap().query_forward([pkg.id()]) {
            query.resolve().len().saturating_sub(1)
        } else {
            0
        };

        CargoPackageInfo {
            name: pkg.name().to_string(),
            version: pkg.version().to_string(),
            manifest_path: pkg.manifest_path().to_string(),
            is_workspace_member,
            direct_dependencies: direct_deps,
            transitive_dependency_count: transitive_count,
        }
    }
}

pub struct CargoAdapter;

impl CargoAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        workspace_root.as_ref().join("Cargo.toml").exists()
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
        Self::generate_pipeline_for_workspace(".")
    }

    pub fn generate_pipeline_for_workspace(workspace_root: impl AsRef<Path>) -> PipelineConfig {
        let tracker = CargoGraphTracker::new(workspace_root);
        let mut jobs = BTreeMap::new();

        jobs.insert(
            "format".to_string(),
            JobConfig {
                needs: vec![],
                command: vec![
                    "cargo".to_string(),
                    "fmt".to_string(),
                    "--all".to_string(),
                    "--".to_string(),
                    "--check".to_string(),
                ],
                matrix: None,
            },
        );

        let check_cmd = if tracker.is_available() {
            vec!["cargo".to_string(), "check".to_string(), "--workspace".to_string(), "--all-targets".to_string()]
        } else {
            vec!["cargo".to_string(), "check".to_string(), "--workspace".to_string()]
        };

        jobs.insert(
            "check".to_string(),
            JobConfig {
                needs: vec!["format".to_string()],
                command: check_cmd,
                matrix: None,
            },
        );

        jobs.insert(
            "clippy".to_string(),
            JobConfig {
                needs: vec!["format".to_string()],
                command: vec![
                    "cargo".to_string(),
                    "clippy".to_string(),
                    "--workspace".to_string(),
                    "--all-targets".to_string(),
                    "--".to_string(),
                    "-D".to_string(),
                    "warnings".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "test".to_string(),
            JobConfig {
                needs: vec!["check".to_string(), "clippy".to_string()],
                command: vec![
                    "cargo".to_string(),
                    "test".to_string(),
                    "--workspace".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "build".to_string(),
            JobConfig {
                needs: vec!["test".to_string()],
                command: vec![
                    "cargo".to_string(),
                    "build".to_string(),
                    "--release".to_string(),
                    "--target".to_string(),
                    "${{ matrix.target }}".to_string(),
                ],
                matrix: Some(vec![
                    "x86_64-unknown-linux-gnu".to_string(),
                    "x86_64-unknown-linux-musl".to_string(),
                    "wasm32-unknown-unknown".to_string(),
                    "x86_64-pc-windows-msvc".to_string(),
                    "aarch64-pc-windows-msvc".to_string(),
                    "aarch64-linux-android".to_string(),
                    "x86_64-linux-android".to_string(),
                ]),
            },
        );

        jobs.insert(
            "sbom".to_string(),
            JobConfig {
                needs: vec!["build".to_string()],
                command: vec![
                    "cargo".to_string(),
                    "cyclonedx".to_string(),
                    "--all".to_string(),
                    "--format".to_string(),
                    "json".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "package".to_string(),
            JobConfig {
                needs: vec!["sbom".to_string()],
                command: vec![
                    "tar".to_string(),
                    "-czvf".to_string(),
                    "release.tar.gz".to_string(),
                    "target/release/".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "package_wix".to_string(),
            JobConfig {
                needs: vec!["build".to_string()],
                command: vec![
                    "cargo".to_string(),
                    "wix".to_string(),
                    "--nocapture".to_string(),
                ],
                matrix: None,
            },
        );

        jobs.insert(
            "sign_windows".to_string(),
            JobConfig {
                needs: vec!["package_wix".to_string()],
                command: vec![
                    "signtool".to_string(),
                    "sign".to_string(),
                    "/a".to_string(),
                    "target/wix/release.msi".to_string(),
                ],
                matrix: None,
            },
        );

        PipelineConfig {
            triggers: vec![],
            stages: vec![
                "validate".to_string(),
                "build".to_string(),
                "package".to_string(),
            ],
            jobs,
        }
    }

    pub fn inject_into_config(
        mut config: ForgeyardConfig,
        workspace_root: impl AsRef<Path>,
    ) -> ForgeyardConfig {
        if Self::detect(&workspace_root) && !config.pipelines.contains_key("default") {
            config
                .pipelines
                .insert("default".to_string(), Self::generate_pipeline_for_workspace(&workspace_root));
        }
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_graph_tracker_current_workspace() {
        let tracker = CargoGraphTracker::new(".");
        assert!(tracker.is_available());

        let members = tracker.workspace_members();
        assert!(!members.is_empty());

        let topo = tracker.topological_workspace_order();
        assert!(!topo.is_empty());

        let frameworks = tracker.detect_frameworks();
        assert!(frameworks.contains("tokio") || frameworks.contains("serde"));

        let summary = tracker.summary();
        assert!(summary.total_packages > 0);
    }

    #[test]
    fn test_cargo_graph_tracker_transitive_and_reverse_queries() {
        let tracker = CargoGraphTracker::new(".");
        if tracker.is_available() {
            let members = tracker.workspace_members();
            if let Some(first) = members.first() {
                let transitive = tracker.query_transitive_deps(&first.name);
                let reverse = tracker.query_reverse_deps(&first.name);
                let _ = (transitive, reverse);
            }
        }
    }
}
