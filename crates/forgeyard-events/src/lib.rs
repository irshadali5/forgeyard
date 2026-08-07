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
        for e in iter {
            if let Ok(row) = e {
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
        }
        Ok(events)
    }
}
