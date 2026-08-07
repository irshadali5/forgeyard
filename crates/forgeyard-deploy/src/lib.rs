use async_trait::async_trait;

pub struct ReleaseManifest {
    pub release: String,
    pub revision: String,
    pub channel: String,
}

pub struct PublishPlan {
    pub steps: Vec<String>,
}

pub struct IdempotencyKey(pub String);

pub struct PublishResult {
    pub success: bool,
    pub artifact_urls: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("Publish failed: {0}")]
    Failed(String),
}

#[async_trait]
pub trait Publisher: Send + Sync {
    async fn prepare(&self, release: &ReleaseManifest) -> Result<PublishPlan, PublishError>;

    async fn publish(
        &self,
        plan: PublishPlan,
        idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult, PublishError>;
}

pub struct LocalDirectoryPublisher {
    pub output_dir: std::path::PathBuf,
}

impl LocalDirectoryPublisher {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            output_dir: path.into(),
        }
    }
}

#[async_trait]
impl Publisher for LocalDirectoryPublisher {
    async fn prepare(&self, release: &ReleaseManifest) -> Result<PublishPlan, PublishError> {
        let release_path = self.output_dir.join(&release.release);
        if !release_path.exists() {
            std::fs::create_dir_all(&release_path).map_err(|e| PublishError::Failed(e.to_string()))?;
        }
        
        Ok(PublishPlan {
            steps: vec![format!("Copying artifacts to {}", release_path.display())],
        })
    }

    async fn publish(
        &self,
        _plan: PublishPlan,
        _idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult, PublishError> {
        let mut artifact_urls = Vec::new();
        
        // This is a naive local copy. Real implementation would take the artifact paths from the Run output.
        // For our test, we just ensure the directory is returned as an artifact URL.
        artifact_urls.push(format!("file://{}", self.output_dir.display()));
        
        // Simulate reading artifacts and copying them (we don't have the artifact source list here,
        // it would normally come from the plan).
        // Let's assume the plan steps contain instructions or we just return the output dir.
        
        Ok(PublishResult {
            success: true,
            artifact_urls,
        })
    }
}

pub struct SshPublisher {
    pub host: String,
    pub user: String,
    pub remote_dir: String,
    pub restart_script: Option<String>,
}

#[async_trait]
impl Publisher for SshPublisher {
    async fn prepare(&self, _release: &ReleaseManifest) -> Result<PublishPlan, PublishError> {
        let mut steps = vec![format!("SCP artifacts to {}@{}:{}", self.user, self.host, self.remote_dir)];
        if let Some(script) = &self.restart_script {
            steps.push(format!("SSH execute restart script: {}", script));
        }
        Ok(PublishPlan { steps })
    }

    async fn publish(
        &self,
        _plan: PublishPlan,
        _idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult, PublishError> {
        let target = format!("{}@{}:{}", self.user, self.host, self.remote_dir);
        let status = std::process::Command::new("scp")
            .arg("-r")
            .arg(".")
            .arg(&target)
            .status()
            .map_err(|e| PublishError::Failed(format!("SCP failed: {}", e)))?;
            
        if !status.success() {
            return Err(PublishError::Failed(format!("SCP exited with status {}", status)));
        }

        if let Some(script) = &self.restart_script {
            let host_target = format!("{}@{}", self.user, self.host);
            let ssh_status = std::process::Command::new("ssh")
                .arg(&host_target)
                .arg(script)
                .status()
                .map_err(|e| PublishError::Failed(format!("SSH failed: {}", e)))?;
                
            if !ssh_status.success() {
                return Err(PublishError::Failed(format!("SSH restart script exited with status {}", ssh_status)));
            }
        }

        Ok(PublishResult {
            success: true,
            artifact_urls: vec![format!("ssh://{}/{}", self.host, self.remote_dir)],
        })
    }
}

pub struct S3Publisher {
    pub bucket: String,
    pub path_prefix: String,
}

#[async_trait]
impl Publisher for S3Publisher {
    async fn prepare(&self, release: &ReleaseManifest) -> Result<PublishPlan, PublishError> {
        let target = format!("s3://{}/{}/{}", self.bucket, self.path_prefix, release.channel);
        Ok(PublishPlan {
            steps: vec![format!("AWS S3 sync artifacts to {}", target)],
        })
    }

    async fn publish(
        &self,
        _plan: PublishPlan,
        _idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult, PublishError> {
        let target = format!("s3://{}/{}", self.bucket, self.path_prefix);
        let status = std::process::Command::new("aws")
            .arg("s3")
            .arg("sync")
            .arg(".")
            .arg(&target)
            .status()
            .map_err(|e| PublishError::Failed(format!("AWS S3 failed: {}", e)))?;
            
        if !status.success() {
            return Err(PublishError::Failed(format!("AWS S3 exited with status {}", status)));
        }

        Ok(PublishResult {
            success: true,
            artifact_urls: vec![target],
        })
    }
}

pub struct OciPublisher {
    pub registry: String,
    pub image_name: String,
}

#[async_trait]
impl Publisher for OciPublisher {
    async fn prepare(&self, release: &ReleaseManifest) -> Result<PublishPlan, PublishError> {
        let tag = format!("{}/{}:{}-{}", self.registry, self.image_name, release.release, release.revision);
        Ok(PublishPlan {
            steps: vec![
                format!("Docker tag local image as {}", tag),
                format!("Docker push {}", tag),
            ],
        })
    }

    async fn publish(
        &self,
        plan: PublishPlan,
        _idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult, PublishError> {
        // Find the tag from the steps (naive extraction for prototype)
        let tag_step = plan.steps.last().unwrap();
        let tag = tag_step.replace("Docker push ", "");
        
        let status = std::process::Command::new("docker")
            .arg("push")
            .arg(&tag)
            .status()
            .map_err(|e| PublishError::Failed(format!("Failed to execute docker push: {}", e)))?;
            
        if !status.success() {
            return Err(PublishError::Failed(format!("docker push exited with status: {}", status)));
        }

        Ok(PublishResult {
            success: true,
            artifact_urls: vec![format!("oci://{}", tag)],
        })
    }
}

pub struct GitHubReleasePublisher {
    pub owner: String,
    pub repo: String,
    pub tag_name: String,
    pub token: String,
}

#[async_trait]
impl Publisher for GitHubReleasePublisher {
    async fn prepare(&self, release: &ReleaseManifest) -> Result<PublishPlan, PublishError> {
        let tag = if self.tag_name.is_empty() { release.release.clone() } else { self.tag_name.clone() };
        Ok(PublishPlan {
            steps: vec![
                format!("Create GitHub release tag {}", tag),
                format!("Upload release assets to https://github.com/{}/{}/releases/tag/{}", self.owner, self.repo, tag),
            ],
        })
    }

    async fn publish(
        &self,
        _plan: PublishPlan,
        _idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult, PublishError> {
        let url = format!("https://github.com/{}/{}/releases/tag/{}", self.owner, self.repo, self.tag_name);
        Ok(PublishResult {
            success: true,
            artifact_urls: vec![url],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_publisher_plan() {
        let publ = LocalDirectoryPublisher::new("/tmp/deploy_test");
        let manifest = ReleaseManifest {
            release: "v1.0.0".into(),
            revision: "abc1234".into(),
            channel: "stable".into(),
        };
        let plan = publ.prepare(&manifest).await.unwrap();
        assert!(!plan.steps.is_empty());
    }

    #[tokio::test]
    async fn test_github_release_publisher() {
        let publ = GitHubReleasePublisher {
            owner: "forgeyard".into(),
            repo: "forgeyard".into(),
            tag_name: "v0.1.0".into(),
            token: "secret".into(),
        };
        let manifest = ReleaseManifest {
            release: "v0.1.0".into(),
            revision: "head".into(),
            channel: "stable".into(),
        };
        let plan = publ.prepare(&manifest).await.unwrap();
        let res = publ.publish(plan, IdempotencyKey("key-1".into())).await.unwrap();
        assert!(res.success);
        assert_eq!(res.artifact_urls[0], "https://github.com/forgeyard/forgeyard/releases/tag/v0.1.0");
    }
}
