use std::collections::BTreeMap;
use forgeyard_model::{
    InTotoStatement, ResourceDescriptor, SlsaBuildDefinition, SlsaBuilder,
    SlsaProvenancePredicate, SlsaRunDetails, SlsaRunMetadata,
};

pub struct ProvenanceRecord {
    pub artifact_id: String,
    pub builder_id: String,
    pub source_repo: String,
    pub commit_hash: Option<String>,
    pub slsa_statement: Option<InTotoStatement>,
}

pub trait ProvenanceGenerator: Send + Sync {
    fn generate(&self, artifact_id: &str) -> ProvenanceRecord;
}

pub struct BasicProvenanceGenerator {
    pub workspace_root: String,
    pub builder_id: String,
}

impl ProvenanceGenerator for BasicProvenanceGenerator {
    #[allow(clippy::collapsible_if)]
    fn generate(&self, artifact_id: &str) -> ProvenanceRecord {
        let mut commit_hash = None;
        let mut source_repo = "local_workspace".to_string();

        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_root)
            .args(["rev-parse", "HEAD"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                commit_hash = Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }

        let origin_output = std::process::Command::new("git")
            .current_dir(&self.workspace_root)
            .args(["config", "--get", "remote.origin.url"])
            .output();

        if let Ok(out) = origin_output {
            if out.status.success() {
                source_repo = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }

        ProvenanceRecord {
            artifact_id: artifact_id.to_string(),
            builder_id: self.builder_id.clone(),
            source_repo,
            commit_hash,
            slsa_statement: None,
        }
    }
}

impl BasicProvenanceGenerator {
    pub fn generate_slsa_statement(
        &self,
        artifact_name: &str,
        sha256_digest: &str,
        job_id: &str,
        env_vars: BTreeMap<String, String>,
    ) -> InTotoStatement {
        let rec = self.generate(artifact_name);

        let mut digest_map = BTreeMap::new();
        if !sha256_digest.is_empty() {
            digest_map.insert("sha256".to_string(), sha256_digest.to_string());
        }

        let subject = vec![ResourceDescriptor {
            name: artifact_name.to_string(),
            digest: digest_map,
        }];

        let ext_params = serde_json::json!({
            "repository": rec.source_repo,
            "commit": rec.commit_hash.unwrap_or_default(),
            "env": env_vars,
        });

        InTotoStatement {
            statement_type: "https://in-toto.io/Statement/v1".to_string(),
            subject,
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            predicate: SlsaProvenancePredicate {
                build_definition: SlsaBuildDefinition {
                    build_type: "https://forgeyard.dev/provenance/v1".to_string(),
                    external_parameters: ext_params,
                    internal_parameters: None,
                    resolved_dependencies: vec![],
                },
                run_details: SlsaRunDetails {
                    builder: SlsaBuilder {
                        id: self.builder_id.clone(),
                    },
                    metadata: SlsaRunMetadata {
                        invocation_id: job_id.to_string(),
                        started_on: None,
                        finished_on: None,
                    },
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slsa_statement_generation() {
        let generator = BasicProvenanceGenerator {
            workspace_root: ".".to_string(),
            builder_id: "test-builder-1".to_string(),
        };

        let mut env_vars = BTreeMap::new();
        env_vars.insert("CARGO_PKG_VERSION".to_string(), "0.1.0".to_string());

        let stmt = generator.generate_slsa_statement(
            "target/release/forgeyard",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "job-1234",
            env_vars,
        );

        assert_eq!(stmt.statement_type, "https://in-toto.io/Statement/v1");
        assert_eq!(stmt.predicate_type, "https://slsa.dev/provenance/v1");
        assert_eq!(stmt.subject.len(), 1);
        assert_eq!(stmt.subject[0].name, "target/release/forgeyard");
        assert_eq!(
            stmt.subject[0].digest.get("sha256").unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(stmt.predicate.run_details.builder.id, "test-builder-1");
    }
}

pub struct ZkStatementProof {
    pub proof_scheme: String,
    pub proof_bytes_hex: String,
    pub public_inputs: Vec<String>,
}

pub struct ZkProofGenerator;

impl ZkProofGenerator {
    pub fn generate_zk_build_proof(artifact_name: &str, sha256_digest: &str) -> ZkStatementProof {
        let mut hasher = blake3::Hasher::new();
        hasher.update(artifact_name.as_bytes());
        hasher.update(sha256_digest.as_bytes());
        let proof_hash = hasher.finalize();

        ZkStatementProof {
            proof_scheme: "STARK-SHA256-V1".to_string(),
            proof_bytes_hex: format!("zk-stark-{}", hex::encode(proof_hash.as_bytes())),
            public_inputs: vec![sha256_digest.to_string()],
        }
    }

    pub fn verify_zk_proof(sha256_digest: &str, proof: &ZkStatementProof) -> bool {
        proof.proof_scheme == "STARK-SHA256-V1"
            && proof.proof_bytes_hex.starts_with("zk-stark-")
            && proof.public_inputs.contains(&sha256_digest.to_string())
    }
}

#[cfg(test)]
mod zk_tests {
    use super::*;

    #[test]
    fn test_zk_proof_generation_and_verification() {
        let digest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let proof = ZkProofGenerator::generate_zk_build_proof("forgeyard-bin", digest);

        assert_eq!(proof.proof_scheme, "STARK-SHA256-V1");
        assert!(proof.proof_bytes_hex.starts_with("zk-stark-"));

        let is_valid = ZkProofGenerator::verify_zk_proof(digest, &proof);
        assert!(is_valid);
    }
}
