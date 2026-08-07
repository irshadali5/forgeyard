use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitRunRequest {
    pub workspace_path: String,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    #[serde(default)]
    pub override_branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitRunResponse {
    pub run_id: String,
    pub status: String,
    pub expected_jobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusInfo {
    pub job_name: String,
    pub state: String,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub runner_id: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStatusResponse {
    pub run_id: String,
    pub jobs: Vec<JobStatusInfo>,
    pub overall_state: String,
    pub total_duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLogsResponse {
    pub run_id: String,
    pub logs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTelemetryPayload {
    pub agent_id: String,
    pub os: String,
    pub arch: String,
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_usage_percent: f32,
    pub load_average: [f32; 3],
    pub active_jobs: usize,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTelemetryResponse {
    pub accepted: bool,
    pub update_available: Option<String>, // URL or version if update is needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactUploadRequest {
    pub job_id: String,
    pub path: String,
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactUploadResponse {
    pub artifact_id: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRunnerRequest {
    pub token: String,
    pub capabilities: Vec<String>,
    pub memory_limit: u64,
    pub cpu_shares: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRunnerResponse {
    pub runner_id: String,
    pub lease_timeout_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineDefinitionSync {
    pub project_name: String,
    pub config_yaml: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunAnalytics {
    pub run_id: String,
    pub job_durations: HashMap<String, u64>,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub total_artifact_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretCreateRequest {
    pub name: String,
    pub value: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretListResponse {
    pub secrets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerStatus {
    pub runner_id: String,
    pub os: String,
    pub arch: String,
    pub capabilities: Vec<String>,
    pub last_seen: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRunnersResponse {
    pub runners: Vec<RunnerStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListRunsResponse {
    pub runs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub job_name: String,
    pub file_path: String,
    pub hash: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListArtifactsResponse {
    pub artifacts: Vec<ArtifactInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatrixTarget {
    pub target: String,
    pub platform: String,
    pub needs_runner: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMatrixResponse {
    pub targets: Vec<MatrixTarget>,
}

pub struct ApiRequest {
    pub path: String,
    pub payload: Vec<u8>,
    pub headers: BTreeMap<String, String>,
}

pub struct ApiResponse {
    pub status: u16,
    pub payload: Vec<u8>,
    pub headers: BTreeMap<String, String>,
}

pub trait ApiRouter: Send + Sync {
    fn handle(&self, req: ApiRequest) -> ApiResponse;
}
