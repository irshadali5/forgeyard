use anyhow::Result;
use forgeyard_model::JobIr;
use forgeyard_protocol::{AgentMessage, DaemonMessage, JobLeaseResponse, JobResult};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use quinn::{Endpoint, ServerConfig};
use rcgen::generate_simple_self_signed;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info, debug};
use futures::{StreamExt, SinkExt};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use bytes::BytesMut;

pub struct QuicServer {
    job_tx: mpsc::Sender<(JobIr, oneshot::Sender<JobResult>)>,
}

impl QuicServer {
    pub async fn start(port: u16, token: String, cas: Arc<forgeyard_cas::CasEngine>, store: Arc<forgeyard_storage::MetadataStore>, broker: Arc<forgeyard_secrets::SecretBroker>) -> Result<Self> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let priv_key = cert.serialize_private_key_der();
        
        let server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                vec![rustls::Certificate(cert_der)],
                rustls::PrivateKey(priv_key)
            )
            .unwrap();

        let server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
        let bind_addr = format!("0.0.0.0:{}", port).parse()?;
        let endpoint = Endpoint::server(server_config, bind_addr)?;
        info!("Listening on QUIC {}", endpoint.local_addr()?);

        // Start mDNS
        let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
        let service_type = "_forgeyard._udp.local.";
        let instance_name = "forgeyard_daemon";
        let host_name = "forgeyard.local.";
        let port = 4433;
        let properties = [("version", "0.1.0")];

        let service_info = ServiceInfo::new(
            service_type,
            instance_name,
            host_name,
            "127.0.0.1",
            port,
            &properties[..],
        ).map_err(|e| anyhow::anyhow!("mdns error: {}", e))?;

        mdns.register(service_info).map_err(|e| anyhow::anyhow!("mdns register error: {}", e))?;
        info!("Registered mDNS service {}", service_type);

        let (job_tx, job_rx) = mpsc::channel::<(JobIr, oneshot::Sender<JobResult>)>(100);
        let job_rx = Arc::new(Mutex::new(job_rx));

        tokio::spawn(async move {
            info!("QUIC server listening on 0.0.0.0:4433");

            while let Some(conn) = endpoint.accept().await {
                let connection = match conn.await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Connection failed: {}", e);
                        continue;
                    }
                };

                info!("Agent connected from {}", connection.remote_address());

                let (send, recv) = match connection.accept_bi().await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Stream failed: {}", e);
                        continue;
                    }
                };

                let job_rx_clone = job_rx.clone();
                let token_clone = token.clone();
                let cas_clone = cas.clone();
                let store_clone = store.clone();
                let broker_clone = broker.clone();
                
                tokio::spawn(async move {
                    let mut framed_read = FramedRead::new(recv, LengthDelimitedCodec::new());
                    let mut framed_write = FramedWrite::new(send, LengthDelimitedCodec::new());
                    
                    let mut authenticated = false;
                    let mut pending_job_tx: Option<oneshot::Sender<JobResult>> = None;

                    while let Some(msg_res) = framed_read.next().await {
                        match msg_res {
                            Ok(bytes) => {
                                match postcard::from_bytes::<AgentMessage>(&bytes) {
                                    Ok(msg) => {
                                        match msg {
                                            AgentMessage::Register(info) => {
                                                if info.token == token_clone {
                                                    authenticated = true;
                                                    info!("Agent authenticated: {:?}", info.capabilities);
                                                } else {
                                                    error!("Agent authentication failed");
                                                    return;
                                                }
                                            }
                                            AgentMessage::RequestLease(_req) => {
                                                if !authenticated { return; }
                                                let mut rx = job_rx_clone.lock().await;
                                                if let Ok((job, result_tx)) = rx.try_recv() {
                                                    pending_job_tx = Some(result_tx);
                                                    let resolved_secrets = broker_clone.resolve_job_secrets(&job.secrets).await.unwrap_or_default();

                                                    let resp = DaemonMessage::LeaseResponse(
                                                        JobLeaseResponse {
                                                            job: Some(job.clone()),
                                                            resolved_secrets,
                                                        },
                                                    );
                                                    if let Ok(out) = postcard::to_allocvec(&resp) {
                                                        if let Err(e) = framed_write.send(bytes::Bytes::from(out)).await {
                                                            error!("Failed to send lease: {}", e);
                                                        }
                                                    }
                                                } else {
                                                    let resp = DaemonMessage::LeaseResponse(
                                                        JobLeaseResponse { job: None, resolved_secrets: std::collections::HashMap::new() },
                                                    );
                                                    if let Ok(out) = postcard::to_allocvec(&resp) {
                                                        let _ = framed_write.send(bytes::Bytes::from(out)).await;
                                                    }
                                                }
                                            }
                                            AgentMessage::Heartbeat(_hb) => {
                                                if !authenticated { return; }
                                                if let Ok(out) = postcard::to_allocvec(&DaemonMessage::AcknowledgeHeartbeat) {
                                                    let _ = framed_write.send(bytes::Bytes::from(out)).await;
                                                }
                                            }
                                            AgentMessage::ReportResult(res) => {
                                                if !authenticated { return; }
                                                if let Some(tx) = pending_job_tx.take() {
                                                    let _ = tx.send(res);
                                                }
                                            }
                                            AgentMessage::PullArtifact { hash } => {
                                                if !authenticated { return; }

                                                let mut data_opt = None;
                                                let mut decoded_hash = [0u8; 32];
                                                if let Ok(bytes) = hex::decode(&hash) {
                                                    if bytes.len() == 32 {
                                                        decoded_hash.copy_from_slice(&bytes);
                                                        let digest = forgeyard_model::Digest { bytes: decoded_hash };
                                                        if let Ok(Some(blob)) = cas_clone.read_blob(&digest).await {
                                                            use base64::{Engine as _, engine::general_purpose::STANDARD};
                                                            data_opt = Some(STANDARD.encode(&blob));
                                                        }
                                                    }
                                                }

                                                let resp = DaemonMessage::ArtifactData { hash, data: data_opt };
                                                if let Ok(out) = postcard::to_allocvec(&resp) {
                                                    let _ = framed_write.send(bytes::Bytes::from(out)).await;
                                                }
                                            }
                                            AgentMessage::PushArtifact { hash: _, data } => {
                                                if !authenticated { return; }
                                                use base64::{Engine as _, engine::general_purpose::STANDARD};
                                                if let Ok(decoded_data) = STANDARD.decode(&data) {
                                                    let _ = cas_clone.write_blob(&decoded_data).await;
                                                }
                                            }
                                            AgentMessage::LogBatch(batch) => {
                                                if !authenticated { return; }
                                                if let Err(e) = store_clone.store_log_batch(&batch) {
                                                    error!("Failed to store log batch: {}", e);
                                                } else {
                                                    if let Ok(out) = postcard::to_allocvec(&DaemonMessage::AcknowledgeLogBatch) {
                                                        let _ = framed_write.send(bytes::Bytes::from(out)).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to deserialize AgentMessage: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Framed read error: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
        });

        Ok(Self { job_tx })
    }

    pub async fn dispatch_job(&self, job: JobIr) -> Result<JobResult> {
        let (tx, rx) = oneshot::channel();
        self.job_tx.send((job, tx)).await?;
        let res = rx.await?;
        Ok(res)
    }
}
