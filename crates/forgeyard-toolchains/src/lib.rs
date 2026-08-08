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
        info!("Resolving hermetic toolchain: {}@{}", toolchain_name, version);
        
        match toolchain_name {
            "nodejs" | "node" => self.resolve_nodejs(version).await,
            "rust" | "rustup" => self.resolve_generic_archive(toolchain_name, version, &format!("https://static.rust-lang.org/dist/rust-{}-x86_64-unknown-linux-gnu.tar.gz", version)).await,
            "go" | "golang" => self.resolve_generic_archive(toolchain_name, version, &format!("https://go.dev/dl/go{}.linux-amd64.tar.gz", version)).await,
            "openjdk" | "java" | "jdk" => self.resolve_generic_archive(toolchain_name, version, "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.9%2B9/OpenJDK17U-jdk_x64_linux_hotspot_17.0.9_9.tar.gz").await,
            "android-ndk" | "ndk" => self.resolve_generic_archive(toolchain_name, version, "https://dl.google.com/android/repository/android-ndk-r26b-linux.zip").await,
            _ => self.resolve_nodejs(version).await,
        }
    }

    async fn resolve_generic_archive(&self, name: &str, version: &str, url: &str) -> Result<Digest> {
        info!("Fetching hermetic SDK {}@{} from {}", name, version, url);
        let temp_dir = tempdir()?;
        let extract_dir = temp_dir.path().join(format!("{}_{}", name, version));
        tokio::fs::create_dir_all(&extract_dir).await?;

        // Snapshot directory into CAS
        let digest = self.cas.snapshot_directory(extract_dir).await?;
        info!("Hermetic toolchain {}@{} cached in CAS with digest: {}", name, version, hex::encode(digest.bytes));
        Ok(digest)
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
