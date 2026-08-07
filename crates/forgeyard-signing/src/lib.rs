use ed25519_dalek::{VerifyingKey, Verifier};
use forgeyard_model::{InTotoStatement, Provenance, SignedProvenance};

pub trait Signer: Send + Sync {
    fn sign_provenance(&self, provenance: Provenance) -> SignedProvenance;
    fn sign_statement(&self, statement: &InTotoStatement) -> String;
}

pub struct LocalEd25519Signer {
    pub key_id: String,
    pub signing_key: ed25519_dalek::SigningKey,
}

impl LocalEd25519Signer {
    pub fn generate_new(key_id: String) -> Self {
        use rand::rngs::OsRng;
        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Self {
            key_id,
            signing_key,
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn verify_statement_signature(
        verifying_key: &VerifyingKey,
        statement: &InTotoStatement,
        signature_hex: &str,
    ) -> bool {
        let payload = serde_json::to_string(statement).unwrap_or_default();
        if let Ok(sig_bytes) = hex::decode(signature_hex) {
            if let Ok(sig) = ed25519_dalek::Signature::from_slice(&sig_bytes) {
                return verifying_key.verify(payload.as_bytes(), &sig).is_ok();
            }
        }
        false
    }
}

impl Signer for LocalEd25519Signer {
    fn sign_provenance(&self, provenance: Provenance) -> SignedProvenance {
        use ed25519_dalek::Signer as _;
        
        let statement = provenance.statement.clone();
        let payload = serde_json::to_string(&provenance).unwrap_or_else(|_| "{}".to_string());
        let signature = self.signing_key.sign(payload.as_bytes());
        
        SignedProvenance {
            provenance,
            signature: hex::encode(signature.to_bytes()),
            key_id: self.key_id.clone(),
            statement,
        }
    }

    fn sign_statement(&self, statement: &InTotoStatement) -> String {
        use ed25519_dalek::Signer as _;
        let payload = serde_json::to_string(statement).unwrap_or_default();
        let signature = self.signing_key.sign(payload.as_bytes());
        hex::encode(signature.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use forgeyard_model::*;

    #[test]
    fn test_slsa_statement_signing_and_verification() {
        let signer = LocalEd25519Signer::generate_new("key-1".to_string());
        
        let stmt = InTotoStatement {
            statement_type: "https://in-toto.io/Statement/v1".to_string(),
            subject: vec![ResourceDescriptor {
                name: "bin/forgeyard".to_string(),
                digest: BTreeMap::from([("sha256".to_string(), "abc123hash".to_string())]),
            }],
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            predicate: SlsaProvenancePredicate {
                build_definition: SlsaBuildDefinition {
                    build_type: "https://forgeyard.dev/provenance/v1".to_string(),
                    external_parameters: serde_json::json!({"commit": "def456"}),
                    internal_parameters: None,
                    resolved_dependencies: vec![],
                },
                run_details: SlsaRunDetails {
                    builder: SlsaBuilder { id: "builder-9".to_string() },
                    metadata: SlsaRunMetadata {
                        invocation_id: "run-42".to_string(),
                        started_on: None,
                        finished_on: None,
                    },
                },
            },
        };

        let sig = signer.sign_statement(&stmt);
        assert!(!sig.is_empty());

        let is_valid = LocalEd25519Signer::verify_statement_signature(
            &signer.verifying_key(),
            &stmt,
            &sig,
        );
        assert!(is_valid);
    }
}
