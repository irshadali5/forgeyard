use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub id: String,
    pub os: String,
    pub version: String,
    pub architecture: String,
}

pub struct DeviceSession {
    pub id: String,
}

#[async_trait]
pub trait DeviceLab: Send + Sync {
    async fn list_devices(&self) -> Result<Vec<DeviceCapabilities>, String>;
    async fn acquire_device(&self, requirements: &DeviceCapabilities) -> Result<DeviceSession, String>;
    async fn release_device(&self, session: DeviceSession) -> Result<(), String>;
}

pub struct LocalAndroidDeviceLab {
    pub locked_devices: tokio::sync::Mutex<std::collections::HashSet<String>>,
}

impl LocalAndroidDeviceLab {
    pub fn new() -> Self {
        Self {
            locked_devices: tokio::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }

    fn query_device_prop(id: &str, prop: &str) -> String {
        let output = std::process::Command::new("adb")
            .args(["-s", id, "shell", "getprop", prop])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !val.is_empty() {
                    return val;
                }
            }
        }
        "unknown".to_string()
    }
}

#[async_trait]
impl DeviceLab for LocalAndroidDeviceLab {
    async fn list_devices(&self) -> Result<Vec<DeviceCapabilities>, String> {
        let output = std::process::Command::new("adb")
            .arg("devices")
            .output()
            .map_err(|e| format!("Failed to execute adb: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines() {
            if line.ends_with("device") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(&id) = parts.first() {
                    let version = Self::query_device_prop(id, "ro.build.version.release");
                    let architecture = Self::query_device_prop(id, "ro.product.cpu.abi");

                    devices.push(DeviceCapabilities {
                        id: id.to_string(),
                        os: "Android".to_string(),
                        version,
                        architecture,
                    });
                }
            }
        }

        Ok(devices)
    }

    async fn acquire_device(&self, requirements: &DeviceCapabilities) -> Result<DeviceSession, String> {
        let available = self.list_devices().await?;
        let mut locked = self.locked_devices.lock().await;

        for dev in available {
            if dev.os.eq_ignore_ascii_case(&requirements.os) {
                if requirements.architecture.is_empty() || dev.architecture.contains(&requirements.architecture) {
                    if !locked.contains(&dev.id) {
                        locked.insert(dev.id.clone());
                        return Ok(DeviceSession { id: dev.id });
                    }
                }
            }
        }

        Err("No matching available devices in device lab".to_string())
    }

    async fn release_device(&self, session: DeviceSession) -> Result<(), String> {
        let mut locked = self.locked_devices.lock().await;
        locked.remove(&session.id);
        Ok(())
    }
}
