use forgeyard_config::{ForgeyardConfig, JobConfig, PipelineConfig};
use std::collections::BTreeMap;
use std::path::Path;

pub struct CargoAdapter;

impl CargoAdapter {
    pub fn detect(workspace_root: impl AsRef<Path>) -> bool {
        workspace_root.as_ref().join("Cargo.toml").exists()
    }

    pub fn generate_default_pipeline() -> PipelineConfig {
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

        jobs.insert(
            "check".to_string(),
            JobConfig {
                needs: vec!["format".to_string()],
                command: vec![
                    "cargo".to_string(),
                    "check".to_string(),
                    "--workspace".to_string(),
                    "--all-targets".to_string(),
                ],
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

        // Rust Target Matrix (Phase 3 + Phase 5)
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

        // SBOM and Provenance
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

        // Web Packaging and Archives
        jobs.insert(
            "package".to_string(),
            JobConfig {
                needs: vec!["sbom".to_string()], // depends on SBOM and effectively on all build matrix jobs
                command: vec![
                    "tar".to_string(),
                    "-czvf".to_string(),
                    "release.tar.gz".to_string(),
                    "target/release/".to_string(),
                ],
                matrix: None,
            },
        );

        // Windows MSIX Packaging
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

        // Windows Signing
        jobs.insert(
            "sign_windows".to_string(),
            JobConfig {
                needs: vec!["package_wix".to_string()],
                command: vec![
                    "signtool".to_string(),
                    "sign".to_string(),
                    "/a".to_string(), // Automatically select best cert
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
        if Self::detect(workspace_root) && !config.pipelines.contains_key("default") {
            config
                .pipelines
                .insert("default".to_string(), Self::generate_default_pipeline());
        }
        config
    }
}
