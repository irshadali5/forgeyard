pub struct ProvenanceRecord {
    pub artifact_id: String,
    pub builder_id: String,
    pub source_repo: String,
    pub commit_hash: Option<String>,
}

pub trait ProvenanceGenerator: Send + Sync {
    fn generate(&self, artifact_id: &str) -> ProvenanceRecord;
}

pub struct BasicProvenanceGenerator {
    pub workspace_root: String,
    pub builder_id: String,
}

impl ProvenanceGenerator for BasicProvenanceGenerator {
    fn generate(&self, artifact_id: &str) -> ProvenanceRecord {
        let mut commit_hash = None;
        let mut source_repo = "local_workspace".to_string();

        let output = std::process::Command::new("git")
            .current_dir(&self.workspace_root)
            .args(["rev-parse", "HEAD"])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                commit_hash = Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }

        let origin_output = std::process::Command::new("git")
            .current_dir(&self.workspace_root)
            .args(["config", "--get", "remote.origin.url"])
            .output();

        if let Ok(out) = origin_output {
            if out.status.success() {
                source_repo = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }

        ProvenanceRecord {
            artifact_id: artifact_id.to_string(),
            builder_id: self.builder_id.clone(),
            source_repo,
            commit_hash,
        }
    }
}
