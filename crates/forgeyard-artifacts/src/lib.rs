#![allow(clippy::collapsible_if)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub name: String,
    pub blake3_hash: String,
    pub size_bytes: u64,
    pub created_at: u64,
}

#[async_trait]
pub trait ArtifactRegistry: Send + Sync {
    async fn register(&self, name: &str, file_path: &str) -> Result<ArtifactMetadata, String>;
    async fn verify(&self, metadata: &ArtifactMetadata) -> Result<bool, String>;
    async fn list_artifacts(&self) -> Result<Vec<ArtifactMetadata>, String>;
    async fn evict_old_artifacts(&self, max_total_bytes: u64) -> Result<usize, String>;
}

pub struct LocalArtifactRegistry {
    pub storage_dir: PathBuf,
}

impl LocalArtifactRegistry {
    pub fn new(storage_dir: impl Into<PathBuf>) -> Self {
        Self {
            storage_dir: storage_dir.into(),
        }
    }

    pub async fn compute_blake3(path: &Path) -> Result<(String, u64), String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(mut ring) = io_uring::IoUring::new(16) {
                if let Ok(file) = std::fs::File::open(path) {
                    use std::os::unix::io::AsRawFd;
                    let fd = io_uring::types::Fd(file.as_raw_fd());
                    let mut hasher = blake3::Hasher::new();
                    let mut buffer = [0u8; 16384];
                    let mut total_bytes = 0u64;
                    let mut offset = 0u64;

                    loop {
                        let read_e = io_uring::opcode::Read::new(fd, buffer.as_mut_ptr(), buffer.len() as u32)
                            .offset(offset)
                            .build();
                        unsafe {
                            let _ = ring.submission().push(&read_e);
                        }
                        if ring.submit_and_wait(1).is_err() {
                            break;
                        }
                        
                        let mut cqe_found = false;
                        for cqe in ring.completion() {
                            let n = cqe.result();
                            if n <= 0 {
                                cqe_found = false;
                                break;
                            }
                            let bytes_read = n as usize;
                            hasher.update(&buffer[..bytes_read]);
                            total_bytes += bytes_read as u64;
                            offset += bytes_read as u64;
                            cqe_found = true;
                        }
                        if !cqe_found {
                            break;
                        }
                    }
                    if total_bytes > 0 {
                        let hash_hex = hasher.finalize().to_hex().to_string();
                        return Ok((hash_hex, total_bytes));
                    }
                }
            }
        }

        // Standard Tokio async fallback
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|e| format!("Failed to open file for hashing: {}", e))?;

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 16384];
        let mut total_bytes = 0u64;

        loop {
            let n = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
            total_bytes += n as u64;
        }

        let hash_hex = hasher.finalize().to_hex().to_string();
        Ok((hash_hex, total_bytes))
    }
}

#[async_trait]
impl ArtifactRegistry for LocalArtifactRegistry {
    async fn register(&self, name: &str, file_path: &str) -> Result<ArtifactMetadata, String> {
        let source_path = Path::new(file_path);
        if !source_path.exists() {
            return Err(format!("Source artifact file does not exist: {}", file_path));
        }

        tokio::fs::create_dir_all(&self.storage_dir)
            .await
            .map_err(|e| e.to_string())?;

        let (blake3_hash, size_bytes) = Self::compute_blake3(source_path).await?;
        let dest = self.storage_dir.join(&blake3_hash);

        if !dest.exists() {
            tokio::fs::copy(source_path, &dest)
                .await
                .map_err(|e| format!("Failed to copy artifact into CAS store: {}", e))?;
        }

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let metadata = ArtifactMetadata {
            name: name.to_string(),
            blake3_hash,
            size_bytes,
            created_at,
        };

        // Write sidecar metadata file
        let meta_path = self.storage_dir.join(format!("{}.meta.json", metadata.blake3_hash));
        if let Ok(meta_json) = serde_json::to_string_pretty(&metadata) {
            let _ = tokio::fs::write(meta_path, meta_json).await;
        }

        Ok(metadata)
    }

    async fn verify(&self, metadata: &ArtifactMetadata) -> Result<bool, String> {
        let path = self.storage_dir.join(&metadata.blake3_hash);
        if !path.exists() {
            return Ok(false);
        }

        let (actual_hash, actual_size) = Self::compute_blake3(&path).await?;
        Ok(actual_hash == metadata.blake3_hash && actual_size == metadata.size_bytes)
    }

    async fn list_artifacts(&self) -> Result<Vec<ArtifactMetadata>, String> {
        let mut entries = tokio::fs::read_dir(&self.storage_dir)
            .await
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(meta) = serde_json::from_str::<ArtifactMetadata>(&content) {
                        results.push(meta);
                    }
                }
            }
        }

        Ok(results)
    }

    async fn evict_old_artifacts(&self, max_total_bytes: u64) -> Result<usize, String> {
        let mut artifacts = self.list_artifacts().await?;
        artifacts.sort_by_key(|a| a.created_at);

        let mut total_size: u64 = artifacts.iter().map(|a| a.size_bytes).sum();
        let mut evicted_count = 0;

        for meta in artifacts {
            if total_size <= max_total_bytes {
                break;
            }

            let file_path = self.storage_dir.join(&meta.blake3_hash);
            let meta_path = self.storage_dir.join(format!("{}.meta.json", meta.blake3_hash));

            let _ = tokio::fs::remove_file(file_path).await;
            let _ = tokio::fs::remove_file(meta_path).await;

            total_size = total_size.saturating_sub(meta.size_bytes);
            evicted_count += 1;
        }

        Ok(evicted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_artifact_registration_and_verification() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("dummy.bin");
        tokio::fs::write(&file_path, b"hello world artifact").await.unwrap();

        let registry = LocalArtifactRegistry::new(dir.path().join("store"));
        let meta = registry.register("dummy.bin", file_path.to_str().unwrap()).await.unwrap();

        assert_eq!(meta.name, "dummy.bin");
        assert!(meta.size_bytes > 0);

        let is_valid = registry.verify(&meta).await.unwrap();
        assert!(is_valid);
    }
}
