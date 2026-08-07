use forgeyard_model::{Provenance, SignedProvenance};

pub trait Signer: Send + Sync {
    fn sign_provenance(&self, provenance: Provenance) -> SignedProvenance;
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
}

impl Signer for LocalEd25519Signer {
    fn sign_provenance(&self, provenance: Provenance) -> SignedProvenance {
        use ed25519_dalek::Signer as _;
        
        let payload = serde_json::to_string(&provenance).unwrap_or_else(|_| "{}".to_string());
        let signature = self.signing_key.sign(payload.as_bytes());
        
        SignedProvenance {
            provenance,
            signature: hex::encode(signature.to_bytes()),
            key_id: self.key_id.clone(),
        }
    }
}
