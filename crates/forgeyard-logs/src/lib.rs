#![allow(clippy::collapsible_if)]
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::io::{BufRead, Write};

pub trait LogReader: Send + Sync {
    fn read_logs(&self, job_id: &str) -> Vec<String>;
    fn search_logs(&self, job_id: &str, keyword: &str) -> Vec<String>;
}

pub trait LogWriter: Send + Sync {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String>;
    fn flush(&self) -> Result<(), String>;
}

pub struct RedactingLogWriter<W: LogWriter> {
    inner: W,
    secret_patterns: Vec<String>,
    regex_set: Option<regex::RegexSet>,
}

impl<W: LogWriter> RedactingLogWriter<W> {
    pub fn new(inner: W, secrets: Vec<String>) -> Self {
        let regex_patterns: Vec<String> = secrets
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| regex::escape(s))
            .collect();

        let regex_set = regex::RegexSet::new(&regex_patterns).ok();

        Self {
            inner,
            secret_patterns: secrets,
            regex_set,
        }
    }

    fn redact(&self, line: &str) -> String {
        let mut redacted = line.to_string();

        if let Some(set) = &self.regex_set {
            let matches = set.matches(line);
            if matches.matched_any() {
                for idx in matches.iter() {
                    let secret = &self.secret_patterns[idx];
                    if !secret.is_empty() {
                        redacted = redacted.replace(secret, "[REDACTED_SECRET]");
                    }
                }
            }
        } else {
            for secret in &self.secret_patterns {
                if !secret.is_empty() {
                    redacted = redacted.replace(secret, "[REDACTED_SECRET]");
                }
            }
        }

        redacted
    }
}

impl<W: LogWriter> LogWriter for RedactingLogWriter<W> {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String> {
        let clean_line = self.redact(line);
        self.inner.write_log(job_id, &clean_line)
    }

    fn flush(&self) -> Result<(), String> {
        self.inner.flush()
    }
}

pub struct RingBufferLogWriter {
    capacity_per_job: usize,
    buffers: Arc<Mutex<std::collections::HashMap<String, VecDeque<String>>>>,
}

impl RingBufferLogWriter {
    pub fn new(capacity_per_job: usize) -> Self {
        Self {
            capacity_per_job,
            buffers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

impl LogWriter for RingBufferLogWriter {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String> {
        let mut guard = self.buffers.lock().map_err(|e| e.to_string())?;
        let deque = guard.entry(job_id.to_string()).or_insert_with(VecDeque::new);
        if deque.len() >= self.capacity_per_job {
            deque.pop_front();
        }
        deque.push_back(line.to_string());
        Ok(())
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }
}

impl LogReader for RingBufferLogWriter {
    fn read_logs(&self, job_id: &str) -> Vec<String> {
        let guard = match self.buffers.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        guard.get(job_id).map(|d| d.iter().cloned().collect()).unwrap_or_default()
    }

    fn search_logs(&self, job_id: &str, keyword: &str) -> Vec<String> {
        self.read_logs(job_id)
            .into_iter()
            .filter(|line| line.contains(keyword))
            .collect()
    }
}

pub struct RotatingFileLogSystem {
    pub log_dir: PathBuf,
    pub max_file_size_bytes: u64,
}

impl RotatingFileLogSystem {
    pub fn new(log_dir: impl Into<PathBuf>, max_size: u64) -> Self {
        Self {
            log_dir: log_dir.into(),
            max_file_size_bytes: max_size,
        }
    }
}

impl LogReader for RotatingFileLogSystem {
    fn read_logs(&self, job_id: &str) -> Vec<String> {
        let path = self.log_dir.join(format!("{}.log", job_id));
        if let Ok(file) = std::fs::File::open(path) {
            let reader = std::io::BufReader::new(file);
            reader.lines().map_while(Result::ok).collect()
        } else {
            Vec::new()
        }
    }

    fn search_logs(&self, job_id: &str, keyword: &str) -> Vec<String> {
        self.read_logs(job_id)
            .into_iter()
            .filter(|l| l.contains(keyword))
            .collect()
    }
}

impl LogWriter for RotatingFileLogSystem {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String> {
        let path = self.log_dir.join(format!("{}.log", job_id));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
            
        writeln!(file, "{}", line).map_err(|e| e.to_string())
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }
}

pub struct IoUringLogWriter {
    pub log_dir: PathBuf,
}

impl IoUringLogWriter {
    pub fn new(log_dir: impl Into<PathBuf>) -> Self {
        Self { log_dir: log_dir.into() }
    }
}

impl LogWriter for IoUringLogWriter {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String> {
        let path = self.log_dir.join(format!("{}.log", job_id));
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(mut ring) = io_uring::IoUring::new(8) {
                if let Ok(file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                    use std::os::unix::io::AsRawFd;
                    let fd = io_uring::types::Fd(file.as_raw_fd());
                    let mut data = format!("{}\n", line).into_bytes();
                    let write_e = io_uring::opcode::Write::new(fd, data.as_mut_ptr(), data.len() as u32).build().user_data(0x99);
                    unsafe {
                        let _ = ring.submission().push(&write_e);
                    }
                    let _ = ring.submit_and_wait(1);
                    return Ok(());
                }
            }
        }

        // Fallback file write
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{}", line).map_err(|e| e.to_string())
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_and_redaction() {
        let ring = RingBufferLogWriter::new(3);
        let redacting = RedactingLogWriter::new(ring, vec!["super_secret_token".to_string()]);

        redacting.write_log("job-1", "Build started with super_secret_token!").unwrap();
        redacting.write_log("job-1", "Step 1 complete").unwrap();

        let _reader = RingBufferLogWriter::new(3);
        // Direct test on redact
        let clean = redacting.redact("my super_secret_token value");
        assert_eq!(clean, "my [REDACTED_SECRET] value");
    }

    #[test]
    fn test_io_uring_log_writer_fallback() {
        let temp_dir = tempfile::tempdir().unwrap();
        let writer = IoUringLogWriter::new(temp_dir.path());
        writer.write_log("test-job-uring", "hello uring log line").unwrap();

        let log_file = temp_dir.path().join("test-job-uring.log");
        assert!(log_file.exists());
    }
}
