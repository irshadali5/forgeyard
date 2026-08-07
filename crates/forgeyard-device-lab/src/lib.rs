use async_trait::async_trait;

pub struct DeviceCapabilities {
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
}

#[async_trait]
impl DeviceLab for LocalAndroidDeviceLab {
    async fn list_devices(&self) -> Result<Vec<DeviceCapabilities>, String> {
        // Mocking `adb devices` discovery for now
        let output = std::process::Command::new("adb")
            .arg("devices")
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut devices = Vec::new();

            for line in stdout.lines() {
                if line.ends_with("device") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(id) = parts.first() {
                        devices.push(DeviceCapabilities {
                            os: "Android".to_string(),
                            version: id.to_string(), // Hack: storing id in version for now
                            architecture: "arm64-v8a".to_string(), // In reality we'd `adb -s <id> shell getprop`
                        });
                    }
                }
            }
            return Ok(devices);
        }
        
        Err("Failed to execute adb".to_string())
    }

    async fn acquire_device(&self, _requirements: &DeviceCapabilities) -> Result<DeviceSession, String> {
        let available = self.list_devices().await?;
        let mut locked = self.locked_devices.lock().await;

        for dev in available {
            // Find a device matching architecture/os, but right now we just grab the first available
            let id = dev.version; // Hack: list_devices pushes id to version for now
            if !locked.contains(&id) {
                locked.insert(id.clone());
                return Ok(DeviceSession { id });
            }
        }
        
        Err("No available devices".to_string())
    }

    async fn release_device(&self, session: DeviceSession) -> Result<(), String> {
        let mut locked = self.locked_devices.lock().await;
        locked.remove(&session.id);
        Ok(())
    }
}
