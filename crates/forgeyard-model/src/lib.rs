use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;
use std::path::PathBuf;
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OutputSpec {
    pub name: String,
    pub path_pattern: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SourceInput {
    WorkingDirectory(PathBuf),
    GitRepository {
        url: String,
        revision: Option<String>,
    },
    Archive(PathBuf),
    Snapshot {
        digest: Digest,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretReference {
    pub name: String,
    pub version: Option<String>,
    pub scope: SecretScope,
    pub delivery: SecretDelivery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub digest: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaBuildDefinition {
    #[serde(rename = "buildType")]
    pub build_type: String,
    #[serde(rename = "externalParameters")]
    pub external_parameters: serde_json::Value,
    #[serde(rename = "internalParameters", skip_serializing_if = "Option::is_none")]
    pub internal_parameters: Option<serde_json::Value>,
    #[serde(rename = "resolvedDependencies", skip_serializing_if = "Vec::is_empty", default)]
    pub resolved_dependencies: Vec<ResourceDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaBuilder {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaRunMetadata {
    #[serde(rename = "invocationId")]
    pub invocation_id: String,
    #[serde(rename = "startedOn", skip_serializing_if = "Option::is_none")]
    pub started_on: Option<String>,
    #[serde(rename = "finishedOn", skip_serializing_if = "Option::is_none")]
    pub finished_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaRunDetails {
    pub builder: SlsaBuilder,
    pub metadata: SlsaRunMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaProvenancePredicate {
    #[serde(rename = "buildDefinition")]
    pub build_definition: SlsaBuildDefinition,
    #[serde(rename = "runDetails")]
    pub run_details: SlsaRunDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InTotoStatement {
    #[serde(rename = "_type")]
    pub statement_type: String,
    pub subject: Vec<ResourceDescriptor>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: SlsaProvenancePredicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub job_id: JobId,
    pub fingerprint: String,
    pub artifacts: Vec<String>,
    pub statement: Option<InTotoStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedProvenance {
    pub provenance: Provenance,
    pub signature: String,
    pub key_id: String,
    pub statement: Option<InTotoStatement>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Module,
    Variable,
    Interface,
    Class,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol_id: String,
    pub label: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub line: u32,
    pub signature: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller_id: String,
    pub callee_id: String,
    pub line_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeGraph {
    pub symbols: Vec<SymbolInfo>,
    pub edges: Vec<CallEdge>,
    pub total_nodes: usize,
    pub total_edges: usize,
}
