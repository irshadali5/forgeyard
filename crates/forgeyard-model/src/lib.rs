use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }
    };
}

id_type!(RunId);
id_type!(JobId);
id_type!(RunnerId);
id_type!(ArtifactId);
id_type!(PipelineId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobState {
    Created,
    Ready,
    WaitingForRunner,
    Leased,
    Preparing,
    Running,
    UploadingOutputs,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
    Skipped,
    Cached,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatingSystem {
    Linux,
    Windows,
    MacOS,
    Android,
    IOS,
    Web,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Architecture {
    X86,
    X86_64,
    ArmV7,
    Aarch64,
    Wasm32,
    Universal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPlatform {
    pub os: OperatingSystem,
    pub arch: Architecture,
    pub abi: Option<String>,
    pub rust_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobIr {
    pub id: JobId,
    pub name: String,
    pub dependencies: Vec<JobId>,
    pub runner_requirements: scheduler::CapabilityExpression,
    pub execution: ExecutionSpec,
    pub timeout: Duration,
    pub cache: CachePolicy,
    pub inputs: BTreeMap<String, Digest>,
    pub outputs: Vec<String>,
    pub secrets: Vec<SecretReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CachePolicy {
    #[default]
    Disabled,
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicy {
    Deny,
    DependencyFetchOnly,
    AllowAll,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceRequest {
    pub memory_bytes: Option<u64>,
    pub cpu_shares: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerImage {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionSpec {
    Command {
        program: String,
        arguments: Vec<String>,
        working_directory: Utf8PathBuf,
        environment: BTreeMap<String, String>,
        network: NetworkPolicy,
        resources: ResourceRequest,
    },
    Container {
        image: ContainerImage,
        program: String,
        arguments: Vec<String>,
        working_directory: Utf8PathBuf,
        environment: BTreeMap<String, String>,
        network: NetworkPolicy,
        resources: ResourceRequest,
    },
    ShellScript {
        script: String,
    },
    Archive {
        format: ArchiveFormat,
        source: Utf8PathBuf,
        destination: Utf8PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    TarGz,
    Zip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineIr {
    pub pipeline_id: PipelineId,
    pub jobs: BTreeMap<JobId, JobIr>,
    pub edges: Vec<(JobId, JobId)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Trigger {
    Manual,
    SourceChanged,
    GitCommit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Digest {
    pub bytes: [u8; 32],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogStream {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub job_id: JobId,
    pub sequence: u64,
    pub stream: LogStream,
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretScope {
    Global,
    Project(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretDelivery {
    Environment,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretReference {
    pub name: String,
    pub version: Option<String>,
    pub scope: SecretScope,
    pub delivery: SecretDelivery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub job_id: JobId,
    pub fingerprint: String,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedProvenance {
    pub provenance: Provenance,
    pub signature: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TechnologyKind {
    Rust,
    Node,
    Android,
    Xcode,
    Generic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvidence {
    pub kind: TechnologyKind,
    pub frameworks: Vec<String>,
    pub intended_targets: Vec<String>,
    pub test_suites: Vec<String>,
}

pub mod scheduler;
