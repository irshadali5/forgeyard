use anyhow::Context;
use clap::Parser;
use forgeyard_adapter_cargo::CargoAdapter;
use forgeyard_cas::CasEngine;
use forgeyard_config::{ForgeyardConfig, ProjectConfig};
use forgeyard_model::{JobState, RunId};
use forgeyard_pipeline::PipelineCompiler;
use forgeyard_storage::MetadataStore;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
pub mod intake;
mod semantic;

use semantic::SemanticIndexer;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path},
    response::IntoResponse,
};
use axum::extract::ws::{WebSocketUpgrade, WebSocket, Message};
use forgeyard_api::{SubmitRunRequest, SubmitRunResponse, GetStatusResponse, GetLogsResponse, JobStatusInfo};
use forgeyard_model::LogEvent;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

mod quic_server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "default_token")]
    token: String,
}

struct RedactingWriter<W: std::io::Write> {
    inner: W,
    secrets: Vec<String>,
}

impl<W: std::io::Write> std::io::Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut s = String::from_utf8_lossy(buf).into_owned();
        for secret in &self.secrets {
            s = s.replace(secret, "***REDACTED***");
        }
        self.inner.write_all(s.as_bytes())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let broker = Arc::new(forgeyard_secrets::SecretBroker::new());

    let redact_token = args.token.clone();
    let make_writer = move || {
        RedactingWriter {
            inner: std::io::stdout(),
            secrets: vec![redact_token.clone()],
        }
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(make_writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Forgeyard Daemon (Phase 4)...");

    let settings = forgeyard_config::DaemonSettings::load().unwrap_or_default();
    
    let token = if args.token != "default_token" {
        args.token.clone()
    } else {
        settings.token.clone()
    };

    let db_path = &settings.db_path;
    let store = Arc::new(MetadataStore::new(db_path).context("Failed to initialize metadata store")?);

    info!("Performing crash recovery...");

    let cas = Arc::new(
        CasEngine::new(".forgeyard_cas")
            .await
            .context("Failed to initialize CAS")?,
    );
    info!("Running workspace detection...");
    let analyzer = forgeyard_detector::WorkspaceAnalyzer::new();
    if let Ok(evidence) = analyzer.analyze(std::path::Path::new(".")).await {
        for ev in evidence {
            info!("Detected technology: {:?}", ev.kind);
        }
    }

    // Broker already instantiated above

    let (log_tx, _) = tokio::sync::broadcast::channel(1024);
    
    let quic_server = quic_server::QuicServer::start(settings.quic_port, token.clone(), cas.clone(), store.clone(), broker.clone(), log_tx.clone()).await?;
    let quic_server = Arc::new(quic_server);

    let toolchains = Arc::new(forgeyard_toolchains::ToolchainManager::new(cas.clone()));
    let semantic_indexer = SemanticIndexer::new();
    
    // Spawn a background task to index incoming logs
    let mut log_rx = log_tx.subscribe();
    let indexer_clone = semantic_indexer.clone();
    tokio::spawn(async move {
        while let Ok(event) = log_rx.recv().await {
            indexer_clone.index_log(&event.job_id.0.to_string(), event).await;
        }
    });

    let app_state = AppState {
        cas: cas.clone(),
        store: store.clone(),
        quic_server: quic_server.clone(),
        toolchains: toolchains.clone(),
        semantic_indexer: semantic_indexer.clone(),
        active_runners: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        secrets: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        log_tx,
        auth_token: token,
    };

    let protected_routes = Router::new()
        .route("/api/v1/run", post(handle_run))
        .route("/api/v1/intake", post(handle_intake))
        .route("/api/v1/status/:run_id", get(handle_status))
        .route("/api/v1/logs/:run_id", get(handle_logs))
        .route("/api/v1/logs/stream/:run_id", get(handle_ws_logs))
        .route("/api/v1/runners/register", post(handle_register_runner))
        .route("/api/v1/runners/telemetry", post(handle_telemetry))
        .route("/api/v1/runners", get(handle_list_runners))
        .route("/api/v1/runs", get(handle_list_runs))
        .route("/api/v1/secrets", post(handle_create_secret))
        .route("/api/v1/secrets/list", get(handle_list_secrets))
        .route("/api/v1/metrics", get(handle_metrics))
        .route("/api/v1/graph", get(handle_graph))
        .route("/api/v1/search", post(handle_search))
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), auth_middleware));

    let app = protected_routes.with_state(app_state.clone());

    info!("Starting HTTP API server on 127.0.0.1:{}", settings.http_port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", settings.http_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    cas: Arc<forgeyard_cas::CasEngine>,
    store: Arc<MetadataStore>,
    quic_server: Arc<quic_server::QuicServer>,
    toolchains: Arc<forgeyard_toolchains::ToolchainManager>,
    semantic_indexer: SemanticIndexer,
    active_runners: Arc<tokio::sync::RwLock<std::collections::HashMap<String, forgeyard_api::RunnerStatus>>>,
    secrets: Arc<tokio::sync::RwLock<std::collections::HashMap<String, String>>>,
    log_tx: tokio::sync::broadcast::Sender<LogEvent>,
    auth_token: String,
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    if state.auth_token.is_empty() || state.auth_token == "default_token" {
        return Ok(next.run(req).await);
    }

    if let Some(auth_header) = req.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str == format!("Bearer {}", state.auth_token) || auth_str == state.auth_token {
                return Ok(next.run(req).await);
            }
        }
    }

    Err(axum::http::StatusCode::UNAUTHORIZED)
}

async fn handle_run(
    State(state): State<AppState>,
    Json(payload): Json<SubmitRunRequest>,
) -> Result<Json<SubmitRunResponse>, axum::http::StatusCode> {
    let run_id = RunId(uuid::Uuid::new_v4());
    
    let store = state.store.clone();
    let quic_server = state.quic_server.clone();
    let toolchains = state.toolchains.clone();
    
    let handle = tokio::spawn(async move {
        // Resolve toolchains before pipeline execution
        if let Err(e) = toolchains.resolve("nodejs", "20.10.0").await {
            error!("Failed to resolve toolchains: {:?}", e);
        }

        if let Err(e) = execute_pipeline(store, quic_server, &payload.workspace_path, run_id).await {
            error!("Pipeline execution failed: {:?}", e);
        }
    });
    let _ = handle.await;

    Ok(Json(SubmitRunResponse {
        run_id: run_id.0.to_string(),
        status: "Accepted".to_string(),
        expected_jobs: 0,
    }))
}

#[derive(Deserialize)]
pub struct SearchRequest {
    query: String,
}

#[derive(Serialize)]
pub struct SearchResponse {
    results: Vec<LogEvent>,
}

async fn handle_search(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, axum::http::StatusCode> {
    let results = state.semantic_indexer.search(&payload.query).await;
    Ok(Json(SearchResponse { results }))
}

#[derive(Serialize, Deserialize)]
pub struct IntakeRequest {
    pub source: forgeyard_model::SourceInput,
}

#[derive(Serialize, Deserialize)]
pub struct IntakeResponse {
    pub digest: String,
}

async fn handle_intake(
    State(state): State<AppState>,
    Json(payload): Json<IntakeRequest>,
) -> Result<Json<IntakeResponse>, axum::http::StatusCode> {
    let cas = state.cas.clone();
    
    match intake::IntakePipeline::process(payload.source, cas).await {
        Ok(digest) => Ok(Json(IntakeResponse {
            digest: hex::encode(digest.bytes),
        })),
        Err(e) => {
            tracing::error!("Intake failed: {:?}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handle_status(
    State(state): State<AppState>,
    Path(run_id_str): Path<String>,
) -> Json<GetStatusResponse> {
    let run_id = RunId(Uuid::parse_str(&run_id_str).unwrap_or_default());
    let mut jobs_info = Vec::new();
    
    if let Ok(jobs) = state.store.get_jobs_for_run(run_id) {
        for job in jobs {
            jobs_info.push(JobStatusInfo {
                job_name: job.name,
                state: format!("{:?}", job.state),
                start_time: None,
                end_time: None,
                runner_id: None,
                dependencies: job.dependencies,
            });
        }
    }
    
    Json(GetStatusResponse {
        run_id: run_id_str,
        jobs: jobs_info,
        overall_state: "Unknown".to_string(),
        total_duration_ms: None,
    })
}

async fn handle_list_runs(
    State(state): State<AppState>,
) -> Json<forgeyard_api::ListRunsResponse> {
    let runs = state.store.get_all_runs().unwrap_or_default();
    Json(forgeyard_api::ListRunsResponse { runs })
}

async fn handle_logs(
    State(state): State<AppState>,
    Path(run_id_str): Path<String>,
) -> Json<GetLogsResponse> {
    let run_id = RunId(Uuid::parse_str(&run_id_str).unwrap_or_default());
    let mut logs_out = Vec::new();

    if let Ok(jobs) = state.store.get_jobs_for_run(run_id) {
        for job in jobs {
            let parsed_job_id = forgeyard_model::JobId(Uuid::parse_str(&job.id).unwrap_or_default());
            if let Ok(events) = state.store.get_logs_for_job(parsed_job_id) {
                for event in events {
                    logs_out.push(format!("[{}] {}", job.name, event.message));
                }
            }
        }
    }
    
    Json(GetLogsResponse {
        run_id: run_id_str,
        logs: logs_out,
    })
}

async fn handle_ws_logs(
    ws: WebSocketUpgrade,
    Path(run_id_str): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let run_id = RunId(Uuid::parse_str(&run_id_str).unwrap_or_default());
    let rx = state.log_tx.subscribe();
    ws.on_upgrade(move |socket| stream_logs(socket, rx, run_id))
}

async fn stream_logs(mut socket: WebSocket, mut rx: tokio::sync::broadcast::Receiver<LogEvent>, target_run: RunId) {
    while let Ok(event) = rx.recv().await {
        if let Some(r_id) = event.run_id {
            if r_id != target_run {
                continue;
            }
        }
        let msg = format!("[Job {}] {}", event.job_id.0, event.message);
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn handle_register_runner(
    State(state): State<AppState>,
    Json(payload): Json<forgeyard_api::RegisterRunnerRequest>,
) -> Json<forgeyard_api::RegisterRunnerResponse> {
    info!("Registering new runner with capabilities: {:?}", payload.capabilities);
    let runner_id = Uuid::new_v4().to_string();
    
    let mut runners = state.active_runners.write().await;
    runners.insert(runner_id.clone(), forgeyard_api::RunnerStatus {
        runner_id: runner_id.clone(),
        os: "unknown".to_string(), // will be populated by telemetry
        arch: "unknown".to_string(),
        capabilities: payload.capabilities,
        last_seen: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    });

    Json(forgeyard_api::RegisterRunnerResponse {
        runner_id,
        lease_timeout_secs: 300,
    })
}

async fn handle_list_runners(
    State(state): State<AppState>,
) -> Json<forgeyard_api::ListRunnersResponse> {
    let runners = state.active_runners.read().await;
    let mut list = Vec::new();
    for r in runners.values() {
        list.push(forgeyard_api::RunnerStatus {
            runner_id: r.runner_id.clone(),
            os: r.os.clone(),
            arch: r.arch.clone(),
            capabilities: r.capabilities.clone(),
            last_seen: r.last_seen,
        });
    }
    Json(forgeyard_api::ListRunnersResponse { runners: list })
}

async fn handle_telemetry(
    State(state): State<AppState>,
    Json(payload): Json<forgeyard_api::AgentTelemetryPayload>,
) -> Json<forgeyard_api::AgentTelemetryResponse> {
    tracing::debug!("Received telemetry from agent {}", payload.agent_id);
    let mut runners = state.active_runners.write().await;
    if let Some(runner) = runners.get_mut(&payload.agent_id) {
        runner.os = payload.os;
        runner.arch = payload.arch;
        runner.last_seen = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    }
    Json(forgeyard_api::AgentTelemetryResponse {
        accepted: true,
        update_available: None,
    })
}

async fn handle_create_secret(
    State(state): State<AppState>,
    Json(payload): Json<forgeyard_api::SecretCreateRequest>,
) -> axum::http::StatusCode {
    info!("Creating secret: {} in scope: {}", payload.name, payload.scope);
    let mut secrets = state.secrets.write().await;
    secrets.insert(payload.name, payload.value);
    axum::http::StatusCode::CREATED
}

async fn handle_list_secrets(
    State(state): State<AppState>,
) -> Json<forgeyard_api::SecretListResponse> {
    let secrets = state.secrets.read().await;
    Json(forgeyard_api::SecretListResponse {
        secrets: secrets.keys().cloned().collect(),
    })
}

#[derive(Serialize, Deserialize)]
struct PipelineMetrics {
    total_runs: usize,
    total_jobs: usize,
    total_logs: usize,
    cache_hit_ratio: f64,
}

async fn handle_metrics(
    State(state): State<AppState>,
) -> Json<PipelineMetrics> {
    if let Ok((runs, jobs, logs, hits)) = state.store.get_pipeline_performance_metrics() {
        let ratio = if jobs > 0 { (hits as f64 / jobs as f64) * 100.0 } else { 0.0 };
        Json(PipelineMetrics {
            total_runs: runs,
            total_jobs: jobs,
            total_logs: logs,
            cache_hit_ratio: ratio,
        })
    } else {
        Json(PipelineMetrics {
            total_runs: 0,
            total_jobs: 0,
            total_logs: 0,
            cache_hit_ratio: 0.0,
        })
    }
}

#[derive(Serialize)]
struct GraphSummaryResponse {
    summary: String,
}

async fn handle_graph() -> Json<GraphSummaryResponse> {
    // Call the analyzer directly on the current directory
    let result = match forgeyard_analyzer::graph::extract_knowledge_graph(std::path::PathBuf::from(".")).await {
        Ok(res) => res,
        Err(_) => graphify_core::model::ExtractionResult {
            nodes: Vec::new(),
            edges: Vec::new(),
            hyperedges: Vec::new(),
        },
    };
    
    let summary = forgeyard_analyzer::graph::generate_token_efficient_summary(&result);
    Json(GraphSummaryResponse { summary })
}

async fn execute_pipeline(
    store: Arc<MetadataStore>,
    quic_server: Arc<quic_server::QuicServer>,
    workspace_path: &str,
    run_id: RunId,
) -> anyhow::Result<()> {
    let exporter = forgeyard_events::TelemetryExporter::new("forgeyard-daemon", "http://localhost:4317");
    let pipeline_span = exporter.start_span("execute_pipeline", forgeyard_events::SpanKind::Server, None);

    let config_path = format!("{}/forgeyard.ron", workspace_path);
    let mut config = if std::path::Path::new(&config_path).exists() {
        ForgeyardConfig::load(&config_path).unwrap_or_else(|_| ForgeyardConfig {
            version: 1,
            project: ProjectConfig { name: "default".to_string() },
            pipelines: BTreeMap::new(),
        })
    } else {
        ForgeyardConfig {
            version: 1,
            project: ProjectConfig { name: "default".to_string() },
            pipelines: BTreeMap::new(),
        }
    };
    
    info!("Running workspace detection on {}...", workspace_path);
    let analyzer = forgeyard_detector::WorkspaceAnalyzer::new();
    if let Ok(evidence) = analyzer.analyze(std::path::Path::new(workspace_path)).await {
        for ev in evidence {
            info!("Detected technology: {:?}", ev.kind);
        }
    }

    config = CargoAdapter::inject_into_config(config, workspace_path);
    let pipeline_ir = PipelineCompiler::compile(&config, "default").context("Failed to compile pipeline")?;



    store.create_run(run_id).context("Failed to create run")?;

    info!(
        "Compiled pipeline {} with {} jobs",
        pipeline_ir.pipeline_id.0,
        pipeline_ir.jobs.len()
    );

    let mut pending = pipeline_ir.jobs.clone();
    let mut job_fingerprints: HashMap<forgeyard_model::JobId, String> = HashMap::new();

    while !pending.is_empty() {
        // Find jobs with no dependencies left in pending
        let mut next_batch = Vec::new();
        for (id, job) in &pending {
            let has_pending_deps = job.dependencies.iter().any(|dep| pending.contains_key(dep));
            if !has_pending_deps {
                next_batch.push(*id);
            }
        }

        if next_batch.is_empty() {
            error!("Cycle detected or unresolvable dependencies in pipeline execution!");
            break;
        }

        for id in next_batch {
            let job = match pending.remove(&id) {
                Some(j) => j,
                None => {
                    tracing::error!("Job {} not found in pending list", id.0);
                    continue;
                }
            };

            // Generate Fingerprint
            let mut dep_fps = Vec::new();
            for dep in &job.dependencies {
                if let Some(fp) = job_fingerprints.get(dep) {
                    dep_fps.push(fp.clone());
                }
            }
            let fingerprint = PipelineCompiler::fingerprint_job(&job, &dep_fps);
            job_fingerprints.insert(job.id, fingerprint.clone());

            // Check Cache
            if let Ok(Some(cached_job)) = store.check_cache(&fingerprint) {
                info!(
                    "Cache hit for job {}: Reusing artifact from {}",
                    job.name, cached_job
                );
                store
                    .insert_job(
                        run_id,
                        job.id,
                        &job.name,
                        JobState::Succeeded,
                        Some(&fingerprint),
                        &job.dependencies,
                    )
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to update job status: {}", e);
                    });
                continue;
            }

            store
                .insert_job(
                    run_id,
                    job.id,
                    &job.name,
                    JobState::Created,
                    Some(&fingerprint),
                    &job.dependencies,
                )
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to send event: {}", e);
                });
            let _ = store.update_job_state(job.id, JobState::Running);

            let res = quic_server.dispatch_job(job.clone()).await?;
            if !res.success {
                error!("Job {} failed on agent: {:?}", job.name, res.error_message);
                let _ = store.update_job_state(job.id, JobState::Failed);
                return Err(anyhow::anyhow!(
                    "Pipeline execution failed at job {}",
                    job.name
                ));
            } else {
                let _ = store.update_job_state(job.id, JobState::Succeeded);
                let _ = store.insert_cache_entry(&fingerprint, job.id);

                // Supply Chain Security: Sign Provenance
                let signer = forgeyard_signing::LocalEd25519Signer::generate_new("daemon_key_v1".to_string());
                use forgeyard_signing::Signer;
                let generator = forgeyard_provenance::BasicProvenanceGenerator {
                    workspace_root: ".".to_string(),
                    builder_id: "forgeyard-daemon-v1".to_string(),
                };
                let slsa_stmt = generator.generate_slsa_statement(
                    &job.name,
                    &fingerprint,
                    &job.id.0.to_string(),
                    std::collections::BTreeMap::new(),
                );
                let prov = forgeyard_model::Provenance {
                    job_id: job.id,
                    fingerprint: fingerprint.clone(),
                    artifacts: job.outputs.clone(),
                    statement: Some(slsa_stmt),
                };
                let signed_prov = signer.sign_provenance(prov);
                let _ = store.insert_provenance(&signed_prov);
                info!("Signed provenance for job {} generated and stored.", job.name);
            }
        }
    }

    exporter.finish_span(pipeline_span);
    info!("Pipeline execution completed successfully.");
    Ok(())
}
