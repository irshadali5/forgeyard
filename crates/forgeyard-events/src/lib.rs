use async_trait::async_trait;

pub struct JournalEvent {
    pub id: String,
    pub timestamp_ms: u64,
    pub payload: Vec<u8>,
}

#[async_trait]
pub trait EventJournal: Send + Sync {
    async fn append(&self, event: &JournalEvent) -> Result<(), String>;
    async fn replay(&self, since_id: Option<String>) -> Result<Vec<JournalEvent>, String>;
}

pub struct StoolapEventJournal {
    db: tokio::sync::Mutex<stoolap::Database>,
}

impl StoolapEventJournal {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let path_str = path.as_ref().to_str().ok_or("Invalid path")?;
        let conn = stoolap::Database::open(path_str).map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS journal (
                id TEXT PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL,
                payload TEXT NOT NULL
            )",
            (),
        ).map_err(|e| e.to_string())?;
        
        Ok(Self {
            db: tokio::sync::Mutex::new(conn),
        })
    }
}

#[async_trait]
impl EventJournal for StoolapEventJournal {
    async fn append(&self, event: &JournalEvent) -> Result<(), String> {
        let conn = self.db.lock().await;
        conn.execute(
            "INSERT INTO journal (id, timestamp_ms, payload) VALUES ($1, $2, $3)",
            (event.id.clone(), event.timestamp_ms as i64, hex::encode(&event.payload)),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn replay(&self, since_id: Option<String>) -> Result<Vec<JournalEvent>, String> {
        let conn = self.db.lock().await;
        let iter = conn.query("SELECT id, timestamp_ms, payload FROM journal ORDER BY timestamp_ms ASC", ()).map_err(|e| e.to_string())?;

        let mut events = Vec::new();
        let mut skip = since_id.is_some();
        for row in iter.flatten() {
            let id = row.get::<String>(0).map_err(|e| e.to_string())?;
            let timestamp_ms = row.get::<i64>(1).map_err(|e| e.to_string())? as u64;
            let payload_hex = row.get::<String>(2).map_err(|e| e.to_string())?;
            let payload = hex::decode(payload_hex).map_err(|e| e.to_string())?;
            let event = JournalEvent { id, timestamp_ms, payload };

            if skip {
                if Some(&event.id) == since_id.as_ref() {
                    skip = false;
                }
                continue;
            }
            events.push(event);
        }
        Ok(events)
    }
}

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time_ms: u64,
    pub end_time_ms: Option<u64>,
    pub attributes: BTreeMap<String, String>,
    pub status: String,
}

pub struct TelemetryExporter {
    pub service_name: String,
    pub endpoint: String,
    spans: std::sync::Mutex<Vec<OtelSpan>>,
}

impl TelemetryExporter {
    pub fn new(service_name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            endpoint: endpoint.into(),
            spans: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn start_span(&self, name: impl Into<String>, kind: SpanKind, parent: Option<&OtelSpan>) -> OtelSpan {
        let trace_id = parent
            .map(|p| p.trace_id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        let start_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        OtelSpan {
            trace_id,
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: parent.map(|p| p.span_id.clone()),
            name: name.into(),
            kind,
            start_time_ms,
            end_time_ms: None,
            attributes: BTreeMap::new(),
            status: "OK".to_string(),
        }
    }

    pub fn finish_span(&self, mut span: OtelSpan) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        span.end_time_ms = Some(now);

        if let Ok(mut lock) = self.spans.lock() {
            lock.push(span);
        }
    }

    pub fn drain_spans(&self) -> Vec<OtelSpan> {
        if let Ok(mut lock) = self.spans.lock() {
            std::mem::take(&mut *lock)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_span_lifecycle() {
        let exporter = TelemetryExporter::new("forgeyard-daemon", "http://localhost:4317");
        
        let parent_span = exporter.start_span("execute_pipeline", SpanKind::Server, None);
        let mut child_span = exporter.start_span("schedule_job", SpanKind::Internal, Some(&parent_span));
        
        child_span.attributes.insert("job_id".to_string(), "job-100".to_string());
        exporter.finish_span(child_span);
        exporter.finish_span(parent_span);

        let drained = exporter.drain_spans();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].name, "schedule_job");
        assert_eq!(drained[1].name, "execute_pipeline");
        assert_eq!(drained[0].trace_id, drained[1].trace_id);
    }
}
