pub struct LogStream {
    pub job_id: String,
}

pub trait LogReader: Send + Sync {
    fn read_logs(&self, job_id: &str) -> Vec<String>;
}

pub trait LogWriter: Send + Sync {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String>;
}

pub struct FileLogSystem {
    pub log_dir: std::path::PathBuf,
}

impl LogReader for FileLogSystem {
    fn read_logs(&self, job_id: &str) -> Vec<String> {
        let path = self.log_dir.join(format!("{}.log", job_id));
        if let Ok(file) = std::fs::File::open(path) {
            use std::io::BufRead;
            let reader = std::io::BufReader::new(file);
            reader.lines().filter_map(Result::ok).collect()
        } else {
            Vec::new()
        }
    }
}

impl LogWriter for FileLogSystem {
    fn write_log(&self, job_id: &str, line: &str) -> Result<(), String> {
        use std::io::Write;
        let path = self.log_dir.join(format!("{}.log", job_id));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
            
        writeln!(file, "{}", line).map_err(|e| e.to_string())
    }
}
