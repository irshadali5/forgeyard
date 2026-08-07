use async_trait::async_trait;

pub struct ArtifactMetadata {
    pub hash: String,
    pub size_bytes: u64,
}

#[async_trait]
pub trait ArtifactRegistry: Send + Sync {
    async fn register(&self, path: &str) -> Result<ArtifactMetadata, String>;
}

pub struct LocalArtifactRegistry {
    pub storage_dir: std::path::PathBuf,
}

#[async_trait]
impl ArtifactRegistry for LocalArtifactRegistry {
    async fn register(&self, path: &str) -> Result<ArtifactMetadata, String> {
        use sha2::{Sha256, Digest};
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        let mut size_bytes = 0;

        loop {
            let n = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
            size_bytes += n as u64;
        }

        let hash = hex::encode(hasher.finalize());

        // Copy to storage dir
        let dest = self.storage_dir.join(&hash);
        if !dest.exists() {
            tokio::fs::copy(path, dest)
                .await
                .map_err(|e| format!("Failed to copy artifact: {}", e))?;
        }

        Ok(ArtifactMetadata {
            hash,
            size_bytes,
        })
    }
}
