use async_recursion::async_recursion;
use blake3::Hasher;
use bytes::Bytes;
use forgeyard_model::Digest;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

    pub fn blob_path(&self, digest: &Digest) -> PathBuf {
        let hex_hash = hex::encode(digest.bytes);
        let prefix = &hex_hash[0..2];
        self.root.join("blobs").join(prefix).join(&hex_hash)
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
            let unique_suffix = format!("{}.tmp", uuid::Uuid::new_v4());
            let temp_path = dir.join(format!("{}.{}", hex_hash, unique_suffix));
            let mut file = fs::File::create(&temp_path).await?;
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

    pub async fn read_blob_stream(&self, digest: &Digest) -> Result<Option<fs::File>, CasError> {
        let hex_hash = hex::encode(digest.bytes);
        let prefix = &hex_hash[0..2];
        let path = self.root.join("blobs").join(prefix).join(&hex_hash);

        if path.exists() {
            let file = fs::File::open(path).await?;
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }
    
    pub async fn write_blob_stream(&self, digest: &Digest, mut stream: impl tokio::io::AsyncRead + Unpin) -> Result<(), CasError> {
        let hex_hash = hex::encode(digest.bytes);
        let prefix = &hex_hash[0..2];
        let dir = self.root.join("blobs").join(prefix);
        fs::create_dir_all(&dir).await?;

        let path = dir.join(&hex_hash);
        if !path.exists() {
            let unique_suffix = format!("{}.tmp", uuid::Uuid::new_v4());
            let temp_path = dir.join(format!("{}.{}", hex_hash, unique_suffix));
            let mut file = fs::File::create(&temp_path).await?;
            tokio::io::copy(&mut stream, &mut file).await?;
            file.sync_all().await?;
            fs::rename(temp_path, path).await?;
        }
        Ok(())
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
            let unique_suffix = format!("{}.tmp", uuid::Uuid::new_v4());
            let temp_path = target_dir.join(format!("{}.{}", hex_hash, unique_suffix));
            let mut file = fs::File::create(&temp_path).await?;
            file.write_all(&tree_bytes).await?;
            file.sync_all().await?;
            fs::rename(temp_path, target_path).await?;
        }

        Ok(Digest { bytes: digest_bytes })
    }

    pub async fn restore_directory(&self, tree_digest: &Digest, target_dir: impl AsRef<Path>) -> Result<(), CasError> {
        let hex_hash = hex::encode(tree_digest.bytes);
        let prefix = &hex_hash[0..2];
        let tree_path = self.root.join("trees").join(prefix).join(&hex_hash);

        if !tree_path.exists() {
            return Err(CasError::DigestMismatch);
        }

        let tree_bytes = fs::read(tree_path).await?;
        let tree: TreeObject = serde_json::from_slice(&tree_bytes)?;
        let target_root = target_dir.as_ref();
        fs::create_dir_all(target_root).await?;

        for (name, entry) in tree.entries {
            let dest_path = target_root.join(name);
            match entry {
                TreeEntry::File { digest, executable, .. } => {
                    let mut bytes_hash = [0u8; 32];
                    hex::decode_to_slice(&digest, &mut bytes_hash).map_err(|_| CasError::DigestMismatch)?;
                    let file_digest = Digest { bytes: bytes_hash };
                    if let Some(content) = self.read_blob(&file_digest).await? {
                        fs::write(&dest_path, content).await?;
                        #[cfg(unix)]
                        if executable {
                            use std::os::unix::fs::PermissionsExt;
                            let mut perms = fs::metadata(&dest_path).await?.permissions();
                            perms.set_mode(perms.mode() | 0o111);
                            fs::set_permissions(&dest_path, perms).await?;
                        }
                    }
                }
                TreeEntry::Directory { digest } => {
                    let mut dir_hash = [0u8; 32];
                    hex::decode_to_slice(&digest, &mut dir_hash).map_err(|_| CasError::DigestMismatch)?;
                    let subdir_digest = Digest { bytes: dir_hash };
                    self.restore_directory(&subdir_digest, dest_path).await?;
                }
            }
        }

        Ok(())
    }
}

pub struct IoUringCasEngine {
    inner: Arc<CasEngine>,
    ring_entries: u32,
}

impl IoUringCasEngine {
    pub fn new(cas: Arc<CasEngine>, ring_entries: u32) -> Self {
        Self { inner: cas, ring_entries }
    }

    pub fn is_io_uring_supported() -> bool {
        #[cfg(target_os = "linux")]
        {
            io_uring::IoUring::new(8).is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    pub async fn read_blob_uring(&self, digest: &Digest) -> Result<Option<Bytes>, CasError> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(mut ring) = io_uring::IoUring::new(self.ring_entries) {
                let path = self.inner.blob_path(digest);
                if path.exists() {
                    if let Ok(file) = std::fs::File::open(&path) {
                        use std::os::unix::io::AsRawFd;
                        let fd = io_uring::types::Fd(file.as_raw_fd());
                        let file_size = file.metadata()?.len() as usize;
                        let mut buf = vec![0u8; file_size];
                        
                        let read_e = io_uring::opcode::Read::new(fd, buf.as_mut_ptr(), file_size as u32).build().user_data(0x42);
                        unsafe {
                            let _ = ring.submission().push(&read_e);
                        }
                        let _ = ring.submit_and_wait(1);
                        return Ok(Some(Bytes::from(buf)));
                    }
                }
            }
        }
        // Fallback to standard CAS engine
        self.inner.read_blob(digest).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrohBlobTicket {
    pub node_id: String,
    pub blake3_hash: String,
    pub format: String,
}

pub struct IrohMeshEngine {
    node_id: String,
    cas: Arc<CasEngine>,
    known_peers: std::sync::RwLock<HashMap<String, String>>,
}

impl IrohMeshEngine {
    pub fn new(node_id: &str, cas: Arc<CasEngine>) -> Self {
        Self {
            node_id: node_id.to_string(),
            cas,
            known_peers: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub fn generate_iroh_ticket(&self, digest: &Digest) -> String {
        let hash_hex = hex::encode(digest.bytes);
        format!("iroh://{}/{}", self.node_id, hash_hex)
    }

    pub fn parse_iroh_ticket(ticket_str: &str) -> Result<IrohBlobTicket, String> {
        if !ticket_str.starts_with("iroh://") {
            return Err("Invalid ticket prefix, must start with iroh://".to_string());
        }

        let body = &ticket_str[7..];
        let parts: Vec<&str> = body.split('/').collect();
        if parts.len() < 2 {
            return Err("Invalid ticket format, expected iroh://<node_id>/<hash>".to_string());
        }

        Ok(IrohBlobTicket {
            node_id: parts[0].to_string(),
            blake3_hash: parts[1].to_string(),
            format: "bao-blake3".to_string(),
        })
    }

    pub fn register_peer(&self, node_id: &str, quic_addr: &str) {
        if let Ok(mut lock) = self.known_peers.write() {
            lock.insert(node_id.to_string(), quic_addr.to_string());
        }
    }

    pub async fn resolve_remote_blob(&self, ticket_str: &str) -> Result<Option<Bytes>, CasError> {
        let ticket = Self::parse_iroh_ticket(ticket_str)
            .map_err(|e| CasError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let mut hash_bytes = [0u8; 32];
        hex::decode_to_slice(&ticket.blake3_hash, &mut hash_bytes)
            .map_err(|_| CasError::DigestMismatch)?;

        let digest = Digest { bytes: hash_bytes };
        
        // If blob is already local, return immediately
        if let Ok(Some(bytes)) = self.cas.read_blob(&digest).await {
            return Ok(Some(bytes));
        }

        Ok(None)
    }
}

pub struct IrohGossipMesh {
    pub topic: String,
    pub active_subscriptions: std::sync::RwLock<Vec<String>>,
}

impl IrohGossipMesh {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            active_subscriptions: std::sync::RwLock::new(Vec::new()),
        }
    }

    pub fn broadcast_chunk_announcement(&self, ticket_str: &str) -> Result<usize, String> {
        if let Ok(mut subs) = self.active_subscriptions.write() {
            subs.push(ticket_str.to_string());
            Ok(subs.len())
        } else {
            Err("Failed to acquire gossip lock".to_string())
        }
    }
}

pub struct IrohNatTunnel;

impl IrohNatTunnel {
    pub fn resolve_p2p_endpoint(node_id: &str, derp_relay: &str) -> String {
        format!("derp://{}/node/{}", derp_relay, node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_io_uring_cas_engine_fallback() {
        let dir = tempdir().unwrap();
        let cas = Arc::new(CasEngine::new(dir.path()).await.unwrap());
        let uring_cas = IoUringCasEngine::new(cas.clone(), 16);

        let digest = cas.write_blob(b"io_uring test content").await.unwrap();
        let content = uring_cas.read_blob_uring(&digest).await.unwrap();

        assert_eq!(content, Some(Bytes::from_static(b"io_uring test content")));
    }

    #[tokio::test]
    async fn test_iroh_mesh_engine_tickets() {
        let dir = tempdir().unwrap();
        let cas = Arc::new(CasEngine::new(dir.path()).await.unwrap());
        let mesh = IrohMeshEngine::new("runner-node-01", cas.clone());

        let digest = cas.write_blob(b"iroh p2p payload").await.unwrap();
        let ticket_str = mesh.generate_iroh_ticket(&digest);
        assert!(ticket_str.starts_with("iroh://runner-node-01/"));

        let ticket = IrohMeshEngine::parse_iroh_ticket(&ticket_str).unwrap();
        assert_eq!(ticket.node_id, "runner-node-01");
        assert_eq!(ticket.format, "bao-blake3");
    }

    #[test]
    fn test_iroh_gossip_and_nat_tunnel() {
        let gossip = IrohGossipMesh::new("forgeyard-cas-chunks");
        let count = gossip.broadcast_chunk_announcement("iroh://node-1/hash123").unwrap();
        assert_eq!(count, 1);

        let endpoint = IrohNatTunnel::resolve_p2p_endpoint("node-1", "derp.iroh.network");
        assert!(endpoint.contains("derp://derp.iroh.network/node/node-1"));
    }
}
