use anyhow::{Context, Result};
use forgeyard_model::{Digest, SourceInput};
use forgeyard_cas::CasEngine;
use std::sync::Arc;
use tempfile::tempdir;
use tracing::{info, debug};

pub struct IntakePipeline;

impl IntakePipeline {
    pub async fn process(input: SourceInput, cas: Arc<CasEngine>) -> Result<Digest> {
        match input {
            SourceInput::GitRepository { url, revision } => {
                info!("Intaking git repository from {}", url);
                let temp_dir = tempdir().context("Failed to create tempdir for git clone")?;
                let clone_path = temp_dir.path();
                
                // Perform the clone in a blocking task since git2 is synchronous
                let url_clone = url.clone();
                let rev_clone = revision.clone();
                let clone_path_buf = clone_path.to_path_buf();
                
                tokio::task::spawn_blocking(move || -> Result<()> {
                    let repo = git2::Repository::clone(&url_clone, &clone_path_buf)
                        .with_context(|| format!("Failed to clone {}", url_clone))?;
                    
                    if let Some(rev) = rev_clone {
                        let (object, reference) = repo.revparse_ext(&rev)?;
                        repo.checkout_tree(&object, None)?;
                        match reference {
                            Some(gref) => repo.set_head(gref.name().unwrap())?,
                            None => repo.set_head_detached(object.id())?,
                        }
                    }
                    Ok(())
                }).await??;

                debug!("Cloned repository into temp dir, now snapshotting...");
                let digest = cas.snapshot_directory(clone_path).await?;
                info!("Snapshot created with digest: {}", hex::encode(digest.bytes));
                
                Ok(digest)
            },
            SourceInput::WorkingDirectory(path) => {
                info!("Intaking local directory {:?}", path);
                let digest = cas.snapshot_directory(&path).await?;
                Ok(digest)
            },
            _ => anyhow::bail!("Unsupported SourceInput format"),
        }
    }
}
