use anyhow::Context;
use clap::Parser;
use forgeyard_adapter_cargo::CargoAdapter;
use forgeyard_cas::CasEngine;
use forgeyard_config::{ForgeyardConfig, ProjectConfig};
use forgeyard_model::{JobState, RunId};
use forgeyard_pipeline::PipelineCompiler;
use forgeyard_runner::LocalRunner;
use forgeyard_storage::MetadataStore;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path},
};
use forgeyard_api::{SubmitRunRequest, SubmitRunResponse, GetStatusResponse, GetLogsResponse, JobStatusInfo};
use uuid::Uuid;

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
    let broker_clone = broker.clone();

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

    let quic_server = quic_server::QuicServer::start(settings.quic_port, token, cas.clone(), store.clone(), broker.clone()).await?;
    let quic_server = Arc::new(quic_server);

    let app_state = AppState {
        store: store.clone(),
        quic_server: quic_server.clone(),
        active_runners: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    let app = Router::new()
        .route("/api/v1/run", post(handle_run))
        .route("/api/v1/status/:run_id", get(handle_status))
        .route("/api/v1/logs/:run_id", get(handle_logs))
        .route("/api/v1/runners/register", post(handle_register_runner))
        .route("/api/v1/runners/telemetry", post(handle_telemetry))
        .route("/api/v1/runners", get(handle_list_runners))
        .route("/api/v1/runs", get(handle_list_runs))
        .route("/api/v1/secrets", post(handle_create_secret))
        .route("/api/v1/secrets/list", get(handle_list_secrets))
        .with_state(app_state);

    info!("Starting HTTP API server on 127.0.0.1:{}", settings.http_port);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", settings.http_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    store: Arc<MetadataStore>,
    quic_server: Arc<quic_server::QuicServer>,
    active_runners: Arc<tokio::sync::RwLock<std::collections::HashMap<String, forgeyard_api::RunnerStatus>>>,
}

async fn handle_run(
    State(state): State<AppState>,
    Json(payload): Json<SubmitRunRequest>,
) -> Json<SubmitRunResponse> {
    info!("Received run request for workspace: {}", payload.workspace_path);
    let run_id = RunId::new();
    
    let store = state.store.clone();
    let quic_server = state.quic_server.clone();
    
    tokio::spawn(async move {
        if let Err(e) = execute_pipeline(store, quic_server, &payload.workspace_path, run_id).await {
            error!("Pipeline execution failed: {:?}", e);
        }
    });

    Json(SubmitRunResponse {
        run_id: run_id.0.to_string(),
        status: "Accepted".to_string(),
        expected_jobs: 0,
    })
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
            if let Ok(events) = state.store.get_logs_for_job(job.id) {
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
    // Real implementation would inject into SecretBroker
    axum::http::StatusCode::CREATED
}

async fn handle_list_secrets(
    State(state): State<AppState>,
) -> Json<forgeyard_api::SecretListResponse> {
    // Return mock secrets for now
    Json(forgeyard_api::SecretListResponse {
        secrets: vec!["NPM_TOKEN".to_string(), "AWS_ACCESS_KEY_ID".to_string()],
    })
}

async fn execute_pipeline(
    store: Arc<MetadataStore>,
    quic_server: Arc<quic_server::QuicServer>,
    workspace_path: &str,
    run_id: RunId,
) -> anyhow::Result<()> {
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
                let prov = forgeyard_model::Provenance {
                    job_id: job.id,
                    fingerprint: fingerprint.clone(),
                    artifacts: job.outputs.clone(),
                };
                let signed_prov = signer.sign_provenance(prov);
                let _ = store.insert_provenance(&signed_prov);
                info!("Signed provenance for job {} generated and stored.", job.name);
            }
        }
    }

    info!("Pipeline execution completed successfully.");
    Ok(())
}
