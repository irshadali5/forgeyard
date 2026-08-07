use forgeyard_model::{JobId, JobState, RunId};
use stoolap::Database;
use std::path::Path;

use std::sync::Mutex;

pub struct MetadataStore {
    conn: Mutex<Database>,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(String),
    #[error("Mutex lock poisoned: {0}")]
    Lock(String),
    #[error("Serialization error: {0}")]
    Serialize(String),
}

impl MetadataStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let raw_str = db_path.as_ref().to_str().unwrap_or("forgeyard_metadata.db");
        let dsn = if raw_str == ":memory:" || raw_str == "memory" {
            "memory://".to_string()
        } else if !raw_str.contains("://") {
            format!("file://{}", raw_str)
        } else {
            raw_str.to_string()
        };
        let conn = Database::open(&dsn).map_err(|e| StorageError::Db(e.to_string()))?;
        let store = Self { conn: Mutex::new(conn) };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        
        let queries = [
            "CREATE TABLE IF NOT EXISTS runs (
                pk_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS jobs (
                pk_id INTEGER PRIMARY KEY AUTOINCREMENT,
                id TEXT NOT NULL,
                run_id TEXT NOT NULL,
                name TEXT NOT NULL,
                state TEXT NOT NULL,
                fingerprint TEXT,
                dependencies TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS cache_entries (
                pk_id INTEGER PRIMARY KEY AUTOINCREMENT,
                fingerprint TEXT NOT NULL,
                job_id TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS logs (
                pk_id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                stream TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                message TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS provenance_records (
                pk_id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                payload TEXT NOT NULL,
                signature TEXT NOT NULL,
                key_id TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS vector_embeddings (
                pk_id INTEGER PRIMARY KEY AUTOINCREMENT,
                vec_id TEXT NOT NULL,
                entity_label TEXT NOT NULL,
                embedding_vector TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        ];

        for query in queries {
            conn.execute(query, ()).map_err(|e| StorageError::Db(e.to_string()))?;
        }

        Ok(())
    }

    pub fn create_run(&self, run_id: RunId) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO runs (id) VALUES ($1)",
            (run_id.0.to_string(),),
        ).map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn insert_job(
        &self,
        run_id: RunId,
        job_id: JobId,
        name: &str,
        state: JobState,
        fingerprint: Option<&str>,
        dependencies: &[JobId],
    ) -> Result<(), StorageError> {
        let state_str = format!("{:?}", state);
        let fingerprint_str = fingerprint.map(|s| s.to_string()).unwrap_or_default();
        let deps_json = serde_json::to_string(dependencies).unwrap_or_else(|_| "[]".to_string());
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO jobs (id, run_id, name, state, fingerprint, dependencies) VALUES ($1, $2, $3, $4, $5, $6)",
            (
                job_id.0.to_string(),
                run_id.0.to_string(),
                name.to_string(),
                state_str,
                fingerprint_str,
                deps_json,
            ),
        ).map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn insert_cache_entry(&self, fingerprint: &str, job_id: JobId) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO cache_entries (fingerprint, job_id) VALUES ($1, $2)",
            (fingerprint.to_string(), job_id.0.to_string()),
        ).map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn check_cache(&self, fingerprint: &str) -> Result<Option<String>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let rows = conn
            .query("SELECT job_id FROM cache_entries WHERE fingerprint = $1", (fingerprint.to_string(),))
            .map_err(|e| StorageError::Db(e.to_string()))?;
        
        for row in rows {
            if let Ok(r) = row {
                if let Ok(val) = r.get::<String>(0) {
                    return Ok(Some(val));
                }
            }
        }
        Ok(None)
    }

    pub fn update_job_state(&self, job_id: JobId, state: JobState) -> Result<(), StorageError> {
        let state_str = serde_json::to_string(&state).unwrap_or_else(|_| "\"Unknown\"".to_string());
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "UPDATE jobs SET state = $1 WHERE id = $2",
            (state_str, job_id.0.to_string()),
        ).map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn store_log_batch(&self, batch: &[forgeyard_model::LogEvent]) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        for log in batch {
            let stream_str = match log.stream {
                forgeyard_model::LogStream::Stdout => "stdout",
                forgeyard_model::LogStream::Stderr => "stderr",
                forgeyard_model::LogStream::System => "system",
            };
            conn.execute(
                "INSERT OR REPLACE INTO logs (job_id, sequence, stream, timestamp, message) VALUES ($1, $2, $3, $4, $5)",
                (
                    log.job_id.0.to_string(),
                    log.sequence as i64,
                    stream_str.to_string(),
                    log.timestamp.clone(),
                    log.message.clone()
                )
            ).map_err(|e| StorageError::Db(e.to_string()))?;
        }
        Ok(())
    }

    pub fn insert_provenance(&self, signed_prov: &forgeyard_model::SignedProvenance) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let payload_str = serde_json::to_string(&signed_prov.provenance).map_err(|e| StorageError::Serialize(e.to_string()))?;
        conn.execute(
            "INSERT INTO provenance_records (job_id, fingerprint, payload, signature, key_id) VALUES ($1, $2, $3, $4, $5)",
            (
                signed_prov.provenance.job_id.0.to_string(),
                signed_prov.provenance.fingerprint.clone(),
                payload_str,
                signed_prov.signature.clone(),
                signed_prov.key_id.clone()
            ),
        ).map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

}

pub struct JobStatus {
    pub id: String,
    pub run_id: String,
    pub name: String,
    pub state: JobState,
    pub fingerprint: Option<String>,
    pub dependencies: Vec<String>,
}

impl MetadataStore {
    pub fn get_jobs_for_run(&self, run_id: RunId) -> Result<Vec<JobStatus>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let run_id_str = run_id.0.to_string();
        
        let rows = conn.query("SELECT id, name, state, fingerprint, dependencies FROM jobs WHERE run_id = $1", (run_id_str.clone(),))
            .map_err(|e| StorageError::Db(e.to_string()))?;
        
        let mut jobs = Vec::new();
        for row in rows {
            if let Ok(r) = row {
                let id: String = r.get(0).map_err(|e| StorageError::Db(e.to_string()))?;
                let name: String = r.get(1).map_err(|e| StorageError::Db(e.to_string()))?;
                let state_str: String = r.get(2).map_err(|e| StorageError::Db(e.to_string()))?;
                let fingerprint: String = r.get(3).unwrap_or_default();
                let deps_json: String = r.get(4).unwrap_or_else(|_| "[]".to_string());
                let dependencies: Vec<String> = serde_json::from_str(&deps_json).unwrap_or_default();
                
                let state = serde_json::from_str(&state_str).unwrap_or(JobState::Created);
                            
                jobs.push(JobStatus { 
                    id, 
                    run_id: run_id_str.clone(),
                    name, 
                    state,
                    fingerprint: if fingerprint.is_empty() { None } else { Some(fingerprint) },
                    dependencies
                });
            }
        }
        Ok(jobs)
    }

    pub fn get_logs_for_job(&self, job_id: JobId) -> Result<Vec<forgeyard_model::LogEvent>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let job_id_str = job_id.0.to_string();
        
        let rows = conn.query("SELECT sequence, stream, timestamp, message FROM logs WHERE job_id = $1 ORDER BY sequence ASC", (job_id_str,))
            .map_err(|e| StorageError::Db(e.to_string()))?;
            
        let mut events = Vec::new();
        for row in rows {
            if let Ok(r) = row {
                let sequence_i64 = r.get::<i64>(0).map_err(|e| StorageError::Db(e.to_string()))?;
                let sequence = sequence_i64 as u64;
                let stream_str = r.get::<String>(1).map_err(|e| StorageError::Db(e.to_string()))?;
                let timestamp = r.get::<String>(2).map_err(|e| StorageError::Db(e.to_string()))?;
                let message = r.get::<String>(3).map_err(|e| StorageError::Db(e.to_string()))?;
                
                let stream = match stream_str.as_str() {
                    "stdout" => forgeyard_model::LogStream::Stdout,
                    "stderr" => forgeyard_model::LogStream::Stderr,
                    _ => forgeyard_model::LogStream::System,
                };
                
                events.push(forgeyard_model::LogEvent {
                    run_id: None,
                    job_id,
                    sequence,
                    stream,
                    timestamp,
                    message,
                });
            }
        }
        Ok(events)
    }

    pub fn get_all_runs(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let rows = conn.query("SELECT id FROM runs ORDER BY created_at DESC", ())
            .map_err(|e| StorageError::Db(e.to_string()))?;
            
        let mut runs = Vec::new();
        for row in rows {
            if let Ok(r) = row {
                let id = r.get::<String>(0).map_err(|e| StorageError::Db(e.to_string()))?;
                runs.push(id);
            }
        }
        Ok(runs)
    }

    pub fn get_pipeline_performance_metrics(&self) -> Result<(usize, usize, usize, usize), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        
        let total_runs = conn.query("SELECT COUNT(*) FROM runs", ())
            .map_err(|e| StorageError::Db(e.to_string()))?
            .into_iter()
            .next()
            .and_then(|r| r.ok())
            .and_then(|r| r.get::<i64>(0).ok())
            .unwrap_or(0) as usize;

        let total_jobs = conn.query("SELECT COUNT(*) FROM jobs", ())
            .map_err(|e| StorageError::Db(e.to_string()))?
            .into_iter()
            .next()
            .and_then(|r| r.ok())
            .and_then(|r| r.get::<i64>(0).ok())
            .unwrap_or(0) as usize;

        let total_logs = conn.query("SELECT COUNT(*) FROM logs", ())
            .map_err(|e| StorageError::Db(e.to_string()))?
            .into_iter()
            .next()
            .and_then(|r| r.ok())
            .and_then(|r| r.get::<i64>(0).ok())
            .unwrap_or(0) as usize;

        let cache_hits = conn.query("SELECT COUNT(*) FROM cache_entries", ())
            .map_err(|e| StorageError::Db(e.to_string()))?
            .into_iter()
            .next()
            .and_then(|r| r.ok())
            .and_then(|r| r.get::<i64>(0).ok())
            .unwrap_or(0) as usize;

        Ok((total_runs, total_jobs, total_logs, cache_hits))
    }

    pub fn store_vector_embedding(&self, id: &str, label: &str, vector: &[f32]) -> Result<(), StorageError> {
        let vec_str = serde_json::to_string(vector).map_err(|e| StorageError::Serialize(e.to_string()))?;
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO vector_embeddings (vec_id, entity_label, embedding_vector) VALUES ($1, $2, $3)",
            (id.to_string(), label.to_string(), vec_str),
        ).map_err(|e| StorageError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn search_similar_vectors(&self, query_vector: &[f32], limit: usize) -> Result<Vec<(String, String, f32)>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Lock(e.to_string()))?;
        let rows = conn.query("SELECT vec_id, entity_label, embedding_vector FROM vector_embeddings", ())
            .map_err(|e| StorageError::Db(e.to_string()))?;

        let mut scored_results = Vec::new();

        for row in rows {
            if let Ok(r) = row {
                let id = r.get::<String>(0).map_err(|e| StorageError::Db(e.to_string()))?;
                let label = r.get::<String>(1).map_err(|e| StorageError::Db(e.to_string()))?;
                let vec_str = r.get::<String>(2).map_err(|e| StorageError::Db(e.to_string()))?;
                let vector: Vec<f32> = serde_json::from_str(&vec_str).unwrap_or_default();

                if vector.len() == query_vector.len() && !vector.is_empty() {
                    let dot_product: f32 = query_vector.iter().zip(vector.iter()).map(|(a, b)| a * b).sum();
                    let norm_q: f32 = query_vector.iter().map(|a| a * a).sum::<f32>().sqrt();
                    let norm_v: f32 = vector.iter().map(|b| b * b).sum::<f32>().sqrt();

                    if norm_q > 0.0 && norm_v > 0.0 {
                        let similarity = dot_product / (norm_q * norm_v);
                        scored_results.push((id, label, similarity));
                    }
                }
            }
        }

        scored_results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        scored_results.truncate(limit);

        Ok(scored_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_embedding_similarity_search() {
        let store = MetadataStore::new(":memory:").expect("Failed to init memory temp store");
        store.store_vector_embedding("vec-1", "Build Failed", &[1.0, 0.0, 0.0]).unwrap();
        store.store_vector_embedding("vec-2", "Compilation Error", &[0.9, 0.1, 0.0]).unwrap();
        store.store_vector_embedding("vec-3", "Unit Test Succeeded", &[0.0, 1.0, 0.0]).unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let results = store.search_similar_vectors(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "vec-1");
        assert!(results[0].2 > 0.99);
        assert_eq!(results[1].0, "vec-2");
    }
}
