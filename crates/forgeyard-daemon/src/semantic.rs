use forgeyard_model::LogEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct SemanticIndexer {
    // In a real Stoolap deployment, this would be a vector database client.
    // For MVP, we index logs in memory.
    logs: Arc<RwLock<HashMap<String, Vec<LogEvent>>>>,
}

impl SemanticIndexer {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn index_log(&self, job_id: &str, event: LogEvent) {
        let mut map = self.logs.write().await;
        map.entry(job_id.to_string()).or_default().push(event);
    }

    pub async fn search(&self, query: &str) -> Vec<LogEvent> {
        let map = self.logs.read().await;
        let query_lower = query.to_lowercase();
        
        let mut results = Vec::new();
        for events in map.values() {
            for event in events {
                if event.message.to_lowercase().contains(&query_lower) {
                    results.push(event.clone());
                }
            }
        }
        
        // For MVP, limit to top 20 matches
        results.into_iter().take(20).collect()
    }
}
