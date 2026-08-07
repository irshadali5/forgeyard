use async_trait::async_trait;

pub struct Target {
    pub os: String,
    pub arch: String,
}

pub struct PackageContext {
    pub name: String,
    pub version: String,
}

pub struct ProducedArtifact {
    pub name: String,
    pub path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Packaging failed: {0}")]
    Failed(String),
}

#[async_trait]
pub trait Packager: Send + Sync {
    fn supports(&self, target: &Target) -> bool;
    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError>;
}

pub struct TarPackager {
    pub workspace_root: String,
    pub output_dir: String,
}

#[async_trait]
impl Packager for TarPackager {
    fn supports(&self, target: &Target) -> bool {
        target.os == "linux" || target.os == "macos"
    }

    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError> {
        let artifact_name = format!("{}-{}.tar.gz", context.name, context.version);
        let output_path = std::path::Path::new(&self.output_dir).join(&artifact_name);
        
        let status = std::process::Command::new("tar")
            .current_dir(&self.workspace_root)
            .arg("-czf")
            .arg(&output_path)
            .arg(".")
            .status()
            .map_err(|e| PackageError::Failed(format!("Failed to execute tar: {}", e)))?;
            
        if !status.success() {
            return Err(PackageError::Failed(format!("tar command exited with status: {}", status)));
        }

        Ok(vec![ProducedArtifact {
            name: artifact_name,
            path: output_path.to_string_lossy().to_string(),
        }])
    }
}

pub struct ZipPackager {
    pub workspace_root: String,
    pub output_dir: String,
}

#[async_trait]
impl Packager for ZipPackager {
    fn supports(&self, target: &Target) -> bool {
        target.os == "windows" || target.os == "wasm" || target.os == "web"
    }

    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError> {
        let artifact_name = format!("{}-{}.zip", context.name, context.version);
        let output_path = std::path::Path::new(&self.output_dir).join(&artifact_name);
        
        let status = std::process::Command::new("zip")
            .current_dir(&self.workspace_root)
            .arg("-r")
            .arg(&output_path)
            .arg(".")
            .status()
            .map_err(|e| PackageError::Failed(format!("Failed to execute zip: {}", e)))?;
            
        if !status.success() {
            return Err(PackageError::Failed(format!("zip command exited with status: {}", status)));
        }

        Ok(vec![ProducedArtifact {
            name: artifact_name,
            path: output_path.to_string_lossy().to_string(),
        }])
    }
}

pub struct MsiPackager {
    pub workspace_root: String,
    pub output_dir: String,
}

#[async_trait]
impl Packager for MsiPackager {
    fn supports(&self, target: &Target) -> bool {
        target.os == "windows"
    }

    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError> {
        let artifact_name = format!("{}-{}.msi", context.name, context.version);
        let output_path = std::path::Path::new(&self.output_dir).join(&artifact_name);
        
        let status = std::process::Command::new("cargo")
            .current_dir(&self.workspace_root)
            .arg("wix")
            .arg("--output")
            .arg(&output_path)
            .status()
            .map_err(|e| PackageError::Failed(format!("Failed to execute cargo wix: {}", e)))?;
            
        if !status.success() {
            return Err(PackageError::Failed(format!("cargo wix exited with status: {}", status)));
        }

        Ok(vec![ProducedArtifact {
            name: artifact_name,
            path: output_path.to_string_lossy().to_string(),
        }])
    }
}

pub struct ApkPackager {
    pub workspace_root: String,
    pub output_dir: String,
}

#[async_trait]
impl Packager for ApkPackager {
    fn supports(&self, target: &Target) -> bool {
        target.os == "android"
    }

    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError> {
        let artifact_name = format!("{}-{}-release.apk", context.name, context.version);
        
        let status = std::process::Command::new("./gradlew")
            .current_dir(&self.workspace_root)
            .arg("assembleRelease")
            .status()
            .map_err(|e| PackageError::Failed(format!("Failed to execute gradlew: {}", e)))?;
            
        if !status.success() {
            return Err(PackageError::Failed(format!("gradlew exited with status: {}", status)));
        }

        let default_output = std::path::Path::new(&self.workspace_root)
            .join("app/build/outputs/apk/release/app-release.apk");
        let final_output = std::path::Path::new(&self.output_dir).join(&artifact_name);
        
        if default_output.exists() {
            std::fs::copy(&default_output, &final_output).unwrap_or(0);
        }

        Ok(vec![ProducedArtifact {
            name: artifact_name,
            path: final_output.to_string_lossy().to_string(),
        }])
    }
}

pub struct AppPackager {
    pub workspace_root: String,
    pub output_dir: String,
}

#[async_trait]
impl Packager for AppPackager {
    fn supports(&self, target: &Target) -> bool {
        target.os == "macos" || target.os == "ios"
    }

    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError> {
        let artifact_name = format!("{}-{}.app", context.name, context.version);
        
        let status = std::process::Command::new("cargo")
            .current_dir(&self.workspace_root)
            .arg("bundle")
            .arg("--release")
            .status()
            .map_err(|e| PackageError::Failed(format!("Failed to execute cargo bundle: {}", e)))?;
            
        if !status.success() {
            return Err(PackageError::Failed(format!("cargo bundle exited with status: {}", status)));
        }

        let output_path = std::path::Path::new(&self.output_dir).join(&artifact_name);

        Ok(vec![ProducedArtifact {
            name: artifact_name,
            path: output_path.to_string_lossy().to_string(),
        }])
    }
}

pub struct DebianPackager {
    pub workspace_root: String,
    pub output_dir: String,
}

#[async_trait]
impl Packager for DebianPackager {
    fn supports(&self, target: &Target) -> bool {
        target.os == "linux" || target.os == "debian" || target.os == "ubuntu"
    }

    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>, PackageError> {
        let artifact_name = format!("{}_{}_amd64.deb", context.name, context.version);
        let output_path = std::path::Path::new(&self.output_dir).join(&artifact_name);
        
        let status = std::process::Command::new("cargo")
            .current_dir(&self.workspace_root)
            .arg("deb")
            .arg("-o")
            .arg(&output_path)
            .status()
            .map_err(|e| PackageError::Failed(format!("Failed to execute cargo deb: {}", e)))?;
            
        if !status.success() {
            return Err(PackageError::Failed(format!("cargo deb exited with status: {}", status)));
        }

        Ok(vec![ProducedArtifact {
            name: artifact_name,
            path: output_path.to_string_lossy().to_string(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packager_target_support() {
        let tar_packager = TarPackager { workspace_root: ".".into(), output_dir: ".".into() };
        let deb_packager = DebianPackager { workspace_root: ".".into(), output_dir: ".".into() };

        assert!(tar_packager.supports(&Target { os: "linux".into(), arch: "x86_64".into() }));
        assert!(deb_packager.supports(&Target { os: "debian".into(), arch: "x86_64".into() }));
    }
}
