use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub suite_name: String,
    pub results: Vec<TestResult>,
}

pub trait ReportParser: Send + Sync {
    fn parse(&self, raw_output: &str) -> Result<TestReport, String>;
}

pub struct CargoTestParser;

impl ReportParser for CargoTestParser {
    fn parse(&self, raw_output: &str) -> Result<TestReport, String> {
        let mut results = Vec::new();

        for line in raw_output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                if value.get("type").and_then(|v| v.as_str()) == Some("test") {
                    let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    let event = value.get("event").and_then(|v| v.as_str()).unwrap_or("");
                    
                    if event == "ok" || event == "failed" {
                        let passed = event == "ok";
                        let error_message = if !passed {
                            value.get("stdout").and_then(|v| v.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        };

                        results.push(TestResult {
                            name,
                            passed,
                            duration_ms: 0, // Cargo doesn't directly give duration here without unstable flags
                            error_message,
                        });
                    }
                }
            }
        }

        Ok(TestReport {
            suite_name: "Cargo Tests".to_string(),
            results,
        })
    }
}
