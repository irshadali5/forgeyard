use async_recursion::async_recursion;
use blake3::Hasher;
use bytes::Bytes;
use forgeyard_model::Digest;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct CasEngine {
    root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum CasError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Digest mismatch")]
    DigestMismatch,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum TreeEntry {
    File { digest: String, size: u64, executable: bool },
    Directory { digest: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreeObject {
    pub entries: BTreeMap<String, TreeEntry>,
}

impl CasEngine {
    pub async fn new(root: impl AsRef<Path>) -> Result<Self, CasError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("blobs")).await?;
        fs::create_dir_all(root.join("trees")).await?;
        Ok(Self { root })
    }

    pub async fn write_blob(&self, data: &[u8]) -> Result<Digest, CasError> {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let digest_bytes: [u8; 32] = hash.into();
        let hex_hash = hex::encode(digest_bytes);

        let prefix = &hex_hash[0..2];
        let dir = self.root.join("blobs").join(prefix);
        fs::create_dir_all(&dir).await?;

        let path = dir.join(&hex_hash);
        if !path.exists() {
            let mut temp_path = path.clone();
            temp_path.set_extension("tmp");
            let mut file = fs::File::create(&temp_path).await?;
            // In a production system we would compress this with zlib or zstd
            file.write_all(data).await?;
            file.sync_all().await?;
            fs::rename(temp_path, path).await?;
        }

        Ok(Digest { bytes: digest_bytes })
    }

    pub async fn read_blob(&self, digest: &Digest) -> Result<Option<Bytes>, CasError> {
        let hex_hash = hex::encode(digest.bytes);
        let prefix = &hex_hash[0..2];
        let path = self.root.join("blobs").join(prefix).join(&hex_hash);

        if path.exists() {
            let mut file = fs::File::open(path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;
            Ok(Some(Bytes::from(buffer)))
        } else {
            Ok(None)
        }
    }

    pub async fn snapshot_directory(&self, path: impl AsRef<Path>) -> Result<Digest, CasError> {
        let root_path = path.as_ref().to_path_buf();
        self.snapshot_dir_recursive(root_path).await
    }

    #[async_recursion]
    async fn snapshot_dir_recursive(&self, dir_path: PathBuf) -> Result<Digest, CasError> {
        let mut tree = TreeObject {
            entries: BTreeMap::new(),
        };

        // We use std::fs for directory walking because ignore crate is sync.
        // For heavy IO, this should ideally be spawn_blocking.
        let walker = WalkBuilder::new(&dir_path)
            .hidden(false)
            .ignore(true)
            .git_ignore(true)
            .max_depth(Some(1))
            .build();

        for result in walker {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path == dir_path {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let metadata = std::fs::symlink_metadata(path)?;

            if metadata.is_dir() {
                let subdir_digest = self.snapshot_dir_recursive(path.to_path_buf()).await?;
                tree.entries.insert(
                    file_name,
                    TreeEntry::Directory {
                        digest: hex::encode(subdir_digest.bytes),
                    },
                );
            } else if metadata.is_file() {
                let data = fs::read(path).await?;
                let file_digest = self.write_blob(&data).await?;
                
                #[cfg(unix)]
                let executable = {
                    use std::os::unix::fs::PermissionsExt;
                    metadata.permissions().mode() & 0o111 != 0
                };
                #[cfg(not(unix))]
                let executable = false;

                tree.entries.insert(
                    file_name,
                    TreeEntry::File {
                        digest: hex::encode(file_digest.bytes),
                        size: metadata.len(),
                        executable,
                    },
                );
            }
        }

        let tree_bytes = serde_json::to_vec(&tree)?;
        let mut hasher = Hasher::new();
        hasher.update(&tree_bytes);
        let hash = hasher.finalize();
        let digest_bytes: [u8; 32] = hash.into();
        let hex_hash = hex::encode(digest_bytes);

        let prefix = &hex_hash[0..2];
        let target_dir = self.root.join("trees").join(prefix);
        fs::create_dir_all(&target_dir).await?;

        let target_path = target_dir.join(&hex_hash);
        if !target_path.exists() {
            let mut temp_path = target_path.clone();
            temp_path.set_extension("tmp");
            let mut file = fs::File::create(&temp_path).await?;
            file.write_all(&tree_bytes).await?;
            file.sync_all().await?;
            fs::rename(temp_path, target_path).await?;
        }

        Ok(Digest { bytes: digest_bytes })
    }
}
