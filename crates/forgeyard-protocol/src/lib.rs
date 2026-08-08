use forgeyard_model::JobIr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerCapabilities {
    pub os: String,
    pub arch: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerInfo {
    pub runner_id: Uuid,
    pub token: String,
    pub capabilities: RunnerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub runner_id: Uuid,
    pub active_jobs: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLeaseRequest {
    pub runner_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLeaseResponse {
    pub job: Option<JobIr>,
    pub resolved_secrets: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub runner_id: Uuid,
    pub job_id: forgeyard_model::JobId,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DaemonMessage {
    LeaseResponse(Box<JobLeaseResponse>),
    AcknowledgeHeartbeat,
    AcknowledgeResult,
    AcknowledgeLogBatch,
    ArtifactStreamReady { hash: String, exists: bool }, // signals that the daemon is ready to stream or already has it
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    Register(RunnerInfo),
    Heartbeat(Heartbeat),
    RequestLease(JobLeaseRequest),
    ReportResult(JobResult),
    LogBatch(Vec<forgeyard_model::LogEvent>),
    PullArtifact { hash: String },
    PushArtifact { hash: String }, // indicates agent will open a unistream to push this hash
}
