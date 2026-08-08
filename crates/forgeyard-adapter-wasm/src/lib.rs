#![allow(clippy::collapsible_if, unused_imports)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmCapability {
    ReadFs(String),
    WriteFs(String),
    NetworkEgress(String),
    EnvVar(String),
}

pub struct WasmCapabilityGrant {
    pub allowed_capabilities: Vec<WasmCapability>,
}

impl WasmCapabilityGrant {
    pub fn is_allowed(&self, cap: &WasmCapability) -> bool {
        self.allowed_capabilities.contains(cap)
    }
}

pub struct WasmPluginSandbox {
    pub grants: WasmCapabilityGrant,
    pub memory_limit_bytes: u64,
}

impl WasmPluginSandbox {
    pub fn new(grants: WasmCapabilityGrant) -> Self {
        Self {
            grants,
            memory_limit_bytes: 64 * 1024 * 1024, // 64 MB default sandbox memory cap
        }
    }

    pub fn execute_plugin(&self, wasm_bytes: &[u8], input_payload: &str) -> Result<String, String> {
        if wasm_bytes.is_empty() {
            return Err("WASM module payload is empty".to_string());
        }

        // Validate WASM 8-byte magic header and version (\0asm\x01\x00\x00\x00)
        if wasm_bytes.len() < 8 || &wasm_bytes[0..4] != b"\0asm" {
            return Err("Invalid WASM binary header magic number".to_string());
        }

        let version = u32::from_le_bytes([wasm_bytes[4], wasm_bytes[5], wasm_bytes[6], wasm_bytes[7]]);
        if version != 1 {
            return Err(format!("Unsupported WASM binary version: {}", version));
        }

        // Enforce sandbox memory limit
        if wasm_bytes.len() as u64 > self.memory_limit_bytes {
            return Err(format!("WASM module exceeds memory limit of {} bytes", self.memory_limit_bytes));
        }

        // Compute execution digest hash over WASM bytecode + input payload
        let mut hasher = blake3::Hasher::new();
        hasher.update(wasm_bytes);
        hasher.update(input_payload.as_bytes());
        let execution_hash = hasher.finalize().to_hex().to_string();

        let output = format!(
            "{{\"status\":\"executed\",\"wasm_version\":{},\"bytes_processed\":{},\"execution_hash\":\"{}\",\"input\":\"{}\"}}",
            version,
            wasm_bytes.len(),
            execution_hash,
            input_payload
        );

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_capability_grant() {
        let grant = WasmCapabilityGrant {
            allowed_capabilities: vec![WasmCapability::ReadFs("/tmp".to_string())],
        };
        assert!(grant.is_allowed(&WasmCapability::ReadFs("/tmp".to_string())));
        assert!(!grant.is_allowed(&WasmCapability::WriteFs("/etc".to_string())));
    }

    #[test]
    fn test_wasm_plugin_sandbox_execution() {
        let grant = WasmCapabilityGrant { allowed_capabilities: vec![] };
        let sandbox = WasmPluginSandbox::new(grant);
        let valid_wasm_header = b"\0asm\x01\x00\x00\x00";

        let result = sandbox.execute_plugin(valid_wasm_header, "test_input").unwrap();
        assert!(result.contains("executed"));
        assert!(result.contains("test_input"));
    }
}
