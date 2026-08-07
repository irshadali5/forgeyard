use anyhow::Result;
use forgeyard_model::JobIr;
use forgeyard_protocol::{AgentMessage, DaemonMessage, JobLeaseResponse, JobResult};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use quinn::Endpoint;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{error, info};
use futures::{StreamExt, SinkExt};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct QuicServer {
    job_tx: mpsc::Sender<(JobIr, oneshot::Sender<JobResult>)>,
}

impl QuicServer {
    pub async fn start(port: u16, token: String, cas: Arc<forgeyard_cas::CasEngine>, store: Arc<forgeyard_storage::MetadataStore>, broker: Arc<forgeyard_secrets::SecretBroker>, log_tx: tokio::sync::broadcast::Sender<forgeyard_model::LogEvent>) -> Result<Self> {
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
                let log_tx_clone = log_tx.clone();
                let connection_clone = connection.clone();
                
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

                                                let mut decoded_hash = [0u8; 32];
                                                if let Ok(bytes) = hex::decode(&hash) {
                                                    if bytes.len() == 32 {
                                                        decoded_hash.copy_from_slice(&bytes);
                                                        let digest = forgeyard_model::Digest { bytes: decoded_hash };
                                                        
                                                        if let Ok(Some(file)) = cas_clone.read_blob_stream(&digest).await {
                                                            let resp = DaemonMessage::ArtifactStreamReady { hash: hash.clone(), exists: true };
                                                            if let Ok(out) = postcard::to_allocvec(&resp) {
                                                                let _ = framed_write.send(bytes::Bytes::from(out)).await;
                                                            }
                                                            
                                                            if let Ok(mut uni_stream) = connection_clone.open_uni().await {
                                                                // write hash length and hash so receiver knows what this is
                                                                let _ = uni_stream.write_u32(hash.len() as u32).await;
                                                                let _ = uni_stream.write_all(hash.as_bytes()).await;
                                                                
                                                                let mut file = file;
                                                                let _ = tokio::io::copy(&mut file, &mut uni_stream).await;
                                                                let _ = uni_stream.finish();
                                                            }
                                                        } else {
                                                            let resp = DaemonMessage::ArtifactStreamReady { hash: hash.clone(), exists: false };
                                                            if let Ok(out) = postcard::to_allocvec(&resp) {
                                                                let _ = framed_write.send(bytes::Bytes::from(out)).await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            AgentMessage::PushArtifact { hash } => {
                                                if !authenticated { return; }
                                                let cas = cas_clone.clone();
                                                let conn = connection_clone.clone();
                                                let hash_clone = hash.clone();
                                                
                                                tokio::spawn(async move {
                                                    if let Ok(mut uni_stream) = conn.accept_uni().await {
                                                        // Expect the hash prefixed
                                                        if let Ok(len) = uni_stream.read_u32().await {
                                                            let mut hash_buf = vec![0u8; len as usize];
                                                            if uni_stream.read_exact(&mut hash_buf).await.is_ok() {
                                                                if let Ok(received_hash) = String::from_utf8(hash_buf) {
                                                                    if received_hash == hash_clone {
                                                                        let mut decoded = [0u8; 32];
                                                                        if hex::decode_to_slice(&received_hash, &mut decoded).is_ok() {
                                                                            let digest = forgeyard_model::Digest { bytes: decoded };
                                                                            let _ = cas.write_blob_stream(&digest, uni_stream).await;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                            }
                                            AgentMessage::LogBatch(batch) => {
                                                if !authenticated { return; }
                                                if let Err(e) = store_clone.store_log_batch(&batch) {
                                                    error!("Failed to store log batch: {}", e);
                                                } else {
                                                    for event in &batch {
                                                        let _ = log_tx_clone.send(event.clone());
                                                    }
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
