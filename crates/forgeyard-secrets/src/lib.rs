use async_trait::async_trait;
use forgeyard_model::SecretReference;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("Secret not found: {0}")]
    NotFound(String),
    #[error("Backend error: {0}")]
    Backend(String),
}

#[async_trait]
pub trait SecretBackend: Send + Sync {
    async fn get(&self, reference: &SecretReference) -> Result<String, SecretError>;
}

pub struct EnvSecretBackend;

#[async_trait]
impl SecretBackend for EnvSecretBackend {
    async fn get(&self, reference: &SecretReference) -> Result<String, SecretError> {
        // Read directly from the host environment of the daemon
        std::env::var(&reference.name).map_err(|_| SecretError::NotFound(reference.name.clone()))
    }
}

pub struct DotEnvBackend;

#[async_trait]
impl SecretBackend for DotEnvBackend {
    async fn get(&self, reference: &SecretReference) -> Result<String, SecretError> {
        if let Ok(path) = dotenvy::dotenv() {
            if let Ok(iter) = dotenvy::from_path_iter(path) {
                for item in iter {
                    if let Ok((k, v)) = item {
                        if k == reference.name {
                            return Ok(v);
                        }
                    }
                }
            }
        }
        Err(SecretError::NotFound(reference.name.clone()))
    }
}

pub struct EncryptedVaultBackend {
    vault: std::sync::RwLock<HashMap<String, Vec<u8>>>,
    key: [u8; 32],
}

impl EncryptedVaultBackend {
    pub fn new(master_password: &str) -> Self {
        let key = blake3::derive_key("forgeyard-secret-vault-v1", master_password.as_bytes());
        Self {
            vault: std::sync::RwLock::new(HashMap::new()),
            key,
        }
    }

    pub fn insert_secret(&self, name: &str, secret_value: &str) {
        let mut encrypted = Vec::new();
        for (i, byte) in secret_value.as_bytes().iter().enumerate() {
            encrypted.push(byte ^ self.key[i % 32]);
        }
        if let Ok(mut lock) = self.vault.write() {
            lock.insert(name.to_string(), encrypted);
        }
    }
}

#[async_trait]
impl SecretBackend for EncryptedVaultBackend {
    async fn get(&self, reference: &SecretReference) -> Result<String, SecretError> {
        let lock = self.vault.read().map_err(|e| SecretError::Backend(e.to_string()))?;
        if let Some(encrypted) = lock.get(&reference.name) {
            let mut decrypted = Vec::new();
            for (i, byte) in encrypted.iter().enumerate() {
                decrypted.push(byte ^ self.key[i % 32]);
            }
            String::from_utf8(decrypted).map_err(|e| SecretError::Backend(e.to_string()))
        } else {
            Err(SecretError::NotFound(reference.name.clone()))
        }
    }
}

pub struct SecretBroker {
    backends: Vec<Box<dyn SecretBackend>>,
}

impl Default for SecretBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretBroker {
    pub fn new() -> Self {
        let vault = EncryptedVaultBackend::new("forgeyard_master_key_default");
        Self {
            backends: vec![
                Box::new(vault),
                Box::new(DotEnvBackend),
                Box::new(EnvSecretBackend),
            ],
        }
    }

    pub async fn resolve_job_secrets(&self, secrets: &[SecretReference]) -> Result<HashMap<String, String>, SecretError> {
        let mut resolved = HashMap::new();
        for sec in secrets {
            let mut found = false;
            for backend in &self.backends {
                if let Ok(val) = backend.get(sec).await {
                    resolved.insert(sec.name.clone(), val);
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(SecretError::NotFound(sec.name.clone()));
            }
        }
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encrypted_vault_backend() {
        let vault = EncryptedVaultBackend::new("my_secure_passphrase");
        vault.insert_secret("API_KEY", "super_secret_12345");

        let sec_ref = SecretReference {
            name: "API_KEY".to_string(),
            version: None,
            scope: forgeyard_model::SecretScope::Global,
            delivery: forgeyard_model::SecretDelivery::Environment,
        };

        let result = vault.get(&sec_ref).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "super_secret_12345");
    }
}
