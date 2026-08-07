use anyhow::Result;
use forgeyard_cas::CasEngine;
use forgeyard_model::Digest;
use std::sync::Arc;
use tempfile::tempdir;
use tracing::{info, debug};
use tokio::io::AsyncWriteExt;

pub struct ToolchainManager {
    cas: Arc<CasEngine>,
}

impl ToolchainManager {
    pub fn new(cas: Arc<CasEngine>) -> Self {
        Self { cas }
    }

    pub async fn resolve(&self, toolchain_name: &str, version: &str) -> Result<Digest> {
        info!("Resolving toolchain: {}@{}", toolchain_name, version);
        
        match toolchain_name {
            "nodejs" => self.resolve_nodejs(version).await,
            _ => anyhow::bail!("Unsupported toolchain: {}", toolchain_name),
        }
    }

    async fn resolve_nodejs(&self, version: &str) -> Result<Digest> {
        // e.g., https://nodejs.org/dist/v20.10.0/node-v20.10.0-linux-x64.tar.gz
        let url = format!(
            "https://nodejs.org/dist/v{}/node-v{}-linux-x64.tar.gz",
            version, version
        );
        
        let temp_dir = tempdir()?;
        let archive_path = temp_dir.path().join("node.tar.gz");
        
        info!("Downloading Node.js {} from {}", version, url);
        
        let mut response = reqwest::get(&url).await?.error_for_status()?;
        let mut file = tokio::fs::File::create(&archive_path).await?;
        
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }
        
        file.sync_all().await?;
        
        debug!("Download complete. Extracting archive...");
        
        let extract_dir = temp_dir.path().join("extract");
        tokio::fs::create_dir_all(&extract_dir).await?;
        
        let archive_path_clone = archive_path.clone();
        let extract_dir_clone = extract_dir.clone();
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            let tar_gz = std::fs::File::open(archive_path_clone)?;
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);
            archive.unpack(&extract_dir_clone)?;
            Ok(())
        }).await??;

        info!("Extraction complete. Snapshotting to CAS...");
        let digest = self.cas.snapshot_directory(extract_dir).await?;
        
        info!("Toolchain successfully cached with digest: {}", hex::encode(digest.bytes));
        Ok(digest)
    }
}
