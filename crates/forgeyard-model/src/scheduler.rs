use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use crate::RunnerId;
use crate::{OperatingSystem, Architecture};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    Os(OperatingSystem),
    Arch(Architecture),
    Rust(String), // "stable", "nightly"
    Docker,
    Podman,
    Kvm,
    Wine,
    QemuUser,
    AndroidSdk,
    AndroidNdk,
    AndroidEmulator,
    AndroidDevice,
    Xcode,
    IosSimulator,
    IosDevice,
    AppleCodesign,
    AppleNotary,
    WindowsSigning,
    Gpu(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityExpression {
    pub required: BTreeSet<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostPlatform {
    pub os: OperatingSystem,
    pub arch: Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapacity {
    pub memory_bytes: u64,
    pub cpu_shares: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustLevel {
    Untrusted,
    Trusted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunnerHealth {
    Healthy,
    Degraded,
    Unreachable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerDescriptor {
    pub id: RunnerId,
    pub host: HostPlatform,
    pub capabilities: BTreeSet<Capability>,
    pub resources: ResourceCapacity,
    pub trust_level: TrustLevel,
    pub labels: BTreeMap<String, String>,
    pub health: RunnerHealth,
    #[serde(default)]
    pub cached_fingerprints: BTreeSet<String>,
    #[serde(default)]
    pub installed_toolchains: BTreeSet<String>,
    #[serde(default)]
    pub network_latency_ms: u64,
}
