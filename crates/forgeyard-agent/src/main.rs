use anyhow::Result;
use clap::Parser;
use forgeyard_protocol::{
    AgentMessage, DaemonMessage, Heartbeat, JobLeaseRequest, JobResult, RunnerCapabilities,
    RunnerInfo,
};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use quinn::{ClientConfig, Endpoint};
use rustls::client::ServerCertVerified;
use rustls::client::ServerCertVerifier;
use rustls::{Certificate, Error as RustlsError, ServerName};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use tracing::{info, debug};
use uuid::Uuid;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::{StreamExt, SinkExt};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use bytes::Bytes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    server: Option<String>,
    #[arg(short, long, default_value = "default_token")]
    token: String,
}

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }
}

async fn discover_daemon() -> Result<SocketAddr> {
    info!("Discovering forgeyard daemon via mDNS...");
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let service_type = "_forgeyard._udp.local.";
    let receiver = mdns.browse(service_type).expect("Failed to browse");

    loop {
        if let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                info!("Discovered daemon: {}", info.get_fullname());
                for addr in info.get_addresses() {
                    let ip: std::net::IpAddr = addr.to_string().parse().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
                    return Ok(SocketAddr::new(ip, info.get_port()));
                }
            }
        }
    }
}

// Telemetry Module
mod telemetry {
    use std::fs;
    pub struct SysMetrics {
        pub total_mem: u64,
        pub free_mem: u64,
        pub cpu_usage: f32,
        pub load_avg: [f32; 3],
    }

    pub fn collect_metrics() -> SysMetrics {
        let mut total_mem = 0;
        let mut free_mem = 0;
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        total_mem = parts[1].parse::<u64>().unwrap_or(0) * 1024;
                    }
                } else if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        free_mem = parts[1].parse::<u64>().unwrap_or(0) * 1024;
                    }
                }
            }
        }

        let mut load_avg = [0.0; 3];
        if let Ok(loadinfo) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = loadinfo.split_whitespace().collect();
            if parts.len() >= 3 {
                load_avg[0] = parts[0].parse().unwrap_or(0.0);
                load_avg[1] = parts[1].parse().unwrap_or(0.0);
                load_avg[2] = parts[2].parse().unwrap_or(0.0);
            }
        }

        SysMetrics {
            total_mem,
            free_mem,
            cpu_usage: load_avg[0], // approximate mapping for now
            load_avg,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let runner_id = Uuid::new_v4();
    info!("Starting Forgeyard Agent {}...", runner_id);

    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();
    let client_config = ClientConfig::new(Arc::new(crypto));

    let mut endpoint = Endpoint::client("[::]:0".parse().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()))?;
    endpoint.set_default_client_config(client_config);

    let settings = forgeyard_config::AgentSettings::load().unwrap_or_else(|_| forgeyard_config::AgentSettings {
        daemon_url: None,
        token: "default_token".to_string(),
        max_concurrent_jobs: 4,
    });
    
    let token = if args.token != "default_token" {
        args.token.clone()
    } else {
        settings.token.clone()
    };

    let server_addr: SocketAddr = match args.server.or(settings.daemon_url) {
        Some(s) => s.parse()?,
        None => discover_daemon().await?,
    };

    info!("Connecting to Forgeyard Daemon at {}", server_addr);

    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    info!("Connected to QUIC daemon at 127.0.0.1:4433");

    let cas = Arc::new(forgeyard_cas::CasEngine::new(".agent_cas").await?);
    
    let (send, recv) = connection.open_bi().await?;
    
    let connection_clone = connection.clone();
    let cas_clone = cas.clone();
    tokio::spawn(async move {
        while let Ok(mut stream) = connection_clone.accept_uni().await {
            let cas = cas_clone.clone();
            tokio::spawn(async move {
                if let Ok(len) = stream.read_u32().await {
                    let mut hash_buf = vec![0u8; len as usize];
                    if stream.read_exact(&mut hash_buf).await.is_ok() {
                        if let Ok(hash_str) = String::from_utf8(hash_buf) {
                            let mut decoded = [0u8; 32];
                            if hex::decode_to_slice(&hash_str, &mut decoded).is_ok() {
                                let digest = forgeyard_model::Digest { bytes: decoded };
                                let _ = cas.write_blob_stream(&digest, stream).await;
                                info!("Agent successfully received streamed artifact: {}", hash_str);
                            }
                        }
                    }
                }
            });
        }
    });

    let mut framed_write = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut framed_read = FramedRead::new(recv, LengthDelimitedCodec::new());

    // Register
    let info = RunnerInfo {
        runner_id,
        token: token.clone(),
        capabilities: RunnerCapabilities {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            labels: vec![],
        },
    };

    if let Ok(msg_bytes) = postcard::to_allocvec(&AgentMessage::Register(info)) {
        framed_write.send(Bytes::from(msg_bytes)).await?;
    }

    let cas = Arc::new(forgeyard_cas::CasEngine::new(".agent_cas").await?);
    let runner = forgeyard_runner::LocalRunner::new(cas.clone());
    
    let telemetry_agent_id = runner_id.to_string();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        loop {
            let metrics = telemetry::collect_metrics();
            debug!("Collected system metrics - Free Mem: {} bytes, Load Avg: {:?}", metrics.free_mem, metrics.load_avg);
            let payload = forgeyard_api::AgentTelemetryPayload {
                agent_id: telemetry_agent_id.clone(),
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                total_memory: metrics.total_mem,
                available_memory: metrics.free_mem,
                cpu_usage_percent: metrics.cpu_usage,
                load_average: metrics.load_avg,
                active_jobs: 0,
                version: "0.1.0".to_string(),
            };
            
            // Send telemetry via HTTP since REST is supported
            let _ = client.post("http://127.0.0.1:8080/api/v1/runners/telemetry")
                .json(&payload)
                .send()
                .await;
                
            sleep(Duration::from_secs(10)).await;
        }
    });

    loop {
        // Request Lease
        if let Ok(msg_bytes) = postcard::to_allocvec(&AgentMessage::RequestLease(JobLeaseRequest { runner_id })) {
            framed_write.send(Bytes::from(msg_bytes)).await?;
        }

        // Wait for response
        if let Some(msg_res) = framed_read.next().await {
            if let Ok(bytes) = msg_res {
                if let Ok(DaemonMessage::LeaseResponse(resp)) = postcard::from_bytes::<DaemonMessage>(&bytes) {
                    if let Some(job) = resp.job {
                        info!("Received lease for job: {}", job.name);
                        
                        // 1. Pull Missing Inputs from Daemon
                        for (path, digest) in &job.inputs {
                            if cas.read_blob(digest).await.unwrap_or(None).is_none() {
                                let hash_str = hex::encode(digest.bytes);
                                info!("Pulling missing input {} (hash {}) from Daemon", path, hash_str);
                                let req = AgentMessage::PullArtifact { hash: hash_str.clone() };
                                if let Ok(req_bytes) = postcard::to_allocvec(&req) {
                                    framed_write.send(Bytes::from(req_bytes)).await?;
                                }
                                
                                // Wait for ArtifactStreamReady
                                if let Some(art_res) = framed_read.next().await {
                                    if let Ok(art_bytes) = art_res {
                                        if let Ok(DaemonMessage::ArtifactStreamReady { hash, exists }) = postcard::from_bytes::<DaemonMessage>(&art_bytes) {
                                            if hash == hash_str && exists {
                                                info!("Daemon is streaming {}", hash_str);
                                                // We don't read the stream here, it's handled by the async task below
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 2. Execute Job with Log Streaming
                        let (log_tx, mut log_rx) = tokio::sync::mpsc::channel(1024);
                        let runner_fut = runner.run_job(&job, Some(log_tx), resp.resolved_secrets.clone());
                        tokio::pin!(runner_fut);
                        #[allow(unused_assignments)]
                        let mut success = false;
                        let mut batch = Vec::new();
                        loop {
                            tokio::select! {
                                res = &mut runner_fut => {
                                    success = res.is_ok();
                                    break;
                                },
                                Some(log) = log_rx.recv() => {
                                    batch.push(log);
                                    if batch.len() >= 50 {
                                        let msg = AgentMessage::LogBatch(std::mem::take(&mut batch));
                                        if let Ok(s) = postcard::to_allocvec(&msg) {
                                            let _ = framed_write.send(Bytes::from(s)).await;
                                        }
                                    }
                                }
                            }
                        }
                        // flush remaining
                        while let Ok(log) = log_rx.try_recv() {
                            batch.push(log);
                        }
                        if !batch.is_empty() {
                            let msg = AgentMessage::LogBatch(batch);
                            if let Ok(s) = postcard::to_allocvec(&msg) {
                                let _ = framed_write.send(Bytes::from(s)).await;
                            }
                        }

                        // 3. Push Outputs to Daemon
                        if success {
                            for path in &job.outputs {
                                // For the prototype, read directly from the workspace
                                if let Ok(data) = tokio::fs::read(path).await {
                                    use sha2::{Digest, Sha256};
                                    let mut hasher = Sha256::new();
                                    hasher.update(&data);
                                    let hash_bytes = hasher.finalize();
                                    let hash_str = hex::encode(hash_bytes);
                                    
                                    let push_req = AgentMessage::PushArtifact {
                                        hash: hash_str.clone(),
                                    };
                                    if let Ok(s) = postcard::to_allocvec(&push_req) {
                                        let _ = framed_write.send(Bytes::from(s)).await;
                                        
                                        if let Ok(mut uni_stream) = connection.open_uni().await {
                                            let _ = uni_stream.write_u32(hash_str.len() as u32).await;
                                            let _ = uni_stream.write_all(hash_str.as_bytes()).await;
                                            let mut file = tokio::fs::File::open(path).await.unwrap();
                                            let _ = tokio::io::copy(&mut file, &mut uni_stream).await;
                                            let _ = uni_stream.finish();
                                        }
                                        
                                        info!("Pushed artifact {} to daemon CAS", path);
                                    }
                                    if let Ok(s) = postcard::to_allocvec(&push_req) {
                                        let _ = framed_write.send(Bytes::from(s)).await;
                                        info!("Pushed artifact {} to daemon CAS", path);
                                    }
                                }
                            }
                        }

                        let result_msg = AgentMessage::ReportResult(JobResult {
                            runner_id,
                            job_id: job.id,
                            success,
                            error_message: if success {
                                None
                            } else {
                                Some("Execution failed".to_string())
                            },
                        });

                        if let Ok(s) = postcard::to_allocvec(&result_msg) {
                            let _ = framed_write.send(Bytes::from(s)).await;
                        }
                    }
                }
            }
        }

        // Heartbeat
        let hb = AgentMessage::Heartbeat(Heartbeat {
            runner_id,
            active_jobs: vec![],
        });
        if let Ok(s) = postcard::to_allocvec(&hb) {
            let _ = framed_write.send(Bytes::from(s)).await;
        }

        sleep(Duration::from_secs(2)).await;
    }
}
