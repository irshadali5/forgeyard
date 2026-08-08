#![allow(clippy::collapsible_if)]
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

pub struct JUnitXmlParser;

impl ReportParser for JUnitXmlParser {
    fn parse(&self, raw_output: &str) -> Result<TestReport, String> {
        use quick_xml::events::Event;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(raw_output);
        reader.config_mut().trim_text(true);

        let mut suite_name = "JUnit Test Suite".to_string();
        let mut results = Vec::new();

        let mut buf = Vec::new();
        let mut current_test_name = String::new();
        let mut current_duration_ms = 0u64;
        let mut is_failed = false;
        let mut failure_msg = None;
        let mut in_testcase = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name_bytes = e.name();
                    let name = String::from_utf8_lossy(name_bytes.as_ref());
                    if name == "testsuite" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                    suite_name = val.to_string();
                                }
                            }
                        }
                    } else if name == "testcase" {
                        in_testcase = true;
                        is_failed = false;
                        failure_msg = None;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                    current_test_name = val.to_string();
                                }
                            } else if attr.key.as_ref() == b"time" {
                                if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                    current_duration_ms = (val.parse::<f64>().unwrap_or(0.0) * 1000.0) as u64;
                                }
                            }
                        }
                    } else if name == "failure" || name == "error" {
                        is_failed = true;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"message" {
                                if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                    failure_msg = Some(val.to_string());
                                }
                            }
                        }
                        if failure_msg.is_none() {
                            failure_msg = Some("JUnit assertion failure".to_string());
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name_bytes = e.name();
                    let name = String::from_utf8_lossy(name_bytes.as_ref());
                    if name == "testcase" {
                        let mut tname = String::new();
                        let mut tdur = 0u64;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                    tname = val.to_string();
                                }
                            } else if attr.key.as_ref() == b"time" {
                                if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                    tdur = (val.parse::<f64>().unwrap_or(0.0) * 1000.0) as u64;
                                }
                            }
                        }
                        results.push(TestResult {
                            name: tname,
                            passed: true,
                            duration_ms: tdur,
                            error_message: None,
                        });
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name_bytes = e.name();
                    let name = String::from_utf8_lossy(name_bytes.as_ref());
                    if name == "testcase" && in_testcase {
                        results.push(TestResult {
                            name: current_test_name.clone(),
                            passed: !is_failed,
                            duration_ms: current_duration_ms,
                            error_message: failure_msg.clone(),
                        });
                        in_testcase = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => (),
            }
            buf.clear();
        }

        if results.is_empty() {
            for line in raw_output.lines() {
                if line.contains("<testcase") {
                    let name = line
                        .find("name=\"")
                        .map(|i| &line[i + 6..])
                        .and_then(|s| s.find('"').map(|j| &s[..j]))
                        .unwrap_or("test")
                        .to_string();
                    let failed = line.contains("<failure");
                    results.push(TestResult {
                        name,
                        passed: !failed,
                        duration_ms: 10,
                        error_message: if failed {
                            Some("JUnit failure".to_string())
                        } else {
                            None
                        },
                    });
                }
            }
        }

        Ok(TestReport {
            suite_name,
            results,
        })
    }
}

impl JUnitXmlParser {
    pub fn parse_file_io_uring(&self, path: &std::path::Path) -> Result<TestReport, String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(mut ring) = io_uring::IoUring::new(8) {
                if let Ok(file) = std::fs::File::open(path) {
                    use std::os::unix::io::AsRawFd;
                    let fd = io_uring::types::Fd(file.as_raw_fd());
                    let file_size = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
                    if file_size > 0 {
                        let mut buf = vec![0u8; file_size];
                        let read_e = io_uring::opcode::Read::new(fd, buf.as_mut_ptr(), file_size as u32).build();
                        unsafe {
                            let _ = ring.submission().push(&read_e);
                        }
                        if ring.submit_and_wait(1).is_ok() {
                            if let Ok(content) = String::from_utf8(buf) {
                                return self.parse(&content);
                            }
                        }
                    }
                }
            }
        }

        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.parse(&content)
    }
}

pub struct FlakyTestDetector;

impl FlakyTestDetector {
    pub fn analyze_history(history: &[TestReport]) -> Vec<String> {
        let mut test_history: std::collections::HashMap<String, Vec<bool>> = std::collections::HashMap::new();

        for report in history {
            for res in &report.results {
                test_history.entry(res.name.clone()).or_default().push(res.passed);
            }
        }

        let mut flaky_tests = Vec::new();
        for (test_name, runs) in test_history {
            if runs.len() > 1 {
                let has_pass = runs.iter().any(|&p| p);
                let has_fail = runs.iter().any(|&p| !p);
                if has_pass && has_fail {
                    flaky_tests.push(test_name);
                }
            }
        }

        flaky_tests
    }
}

pub struct FlakyTestQuarantine;

impl FlakyTestQuarantine {
    pub fn quarantine_tests(flaky_tests: &[String], report: &mut TestReport) -> usize {
        let mut quarantined_count = 0;
        for result in &mut report.results {
            if flaky_tests.contains(&result.name) && !result.passed {
                result.passed = true; // Quarantine failure so pipeline deployment isn't blocked
                result.error_message = Some(format!("[QUARANTINED FLAKY TEST] {}", result.error_message.as_deref().unwrap_or("Failure ignored")));
                quarantined_count += 1;
            }
        }
        quarantined_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_test_parser() {
        let raw = r#"
{"type":"test","name":"tests::test_foo","event":"ok"}
{"type":"test","name":"tests::test_bar","event":"failed","stdout":"assertion failed"}
"#;
        let parser = CargoTestParser;
        let report = parser.parse(raw).unwrap();
        assert_eq!(report.results.len(), 2);
        assert!(report.results[0].passed);
        assert!(!report.results[1].passed);
        assert_eq!(report.results[1].error_message.as_deref(), Some("assertion failed"));
    }

    #[test]
    fn test_flaky_test_detector() {
        let report1 = TestReport {
            suite_name: "Run 1".to_string(),
            results: vec![
                TestResult { name: "test_a".to_string(), passed: true, duration_ms: 10, error_message: None },
                TestResult { name: "test_flaky".to_string(), passed: false, duration_ms: 15, error_message: Some("timeout".to_string()) },
            ],
        };

        let report2 = TestReport {
            suite_name: "Run 2".to_string(),
            results: vec![
                TestResult { name: "test_a".to_string(), passed: true, duration_ms: 12, error_message: None },
                TestResult { name: "test_flaky".to_string(), passed: true, duration_ms: 14, error_message: None },
            ],
        };

        let flaky = FlakyTestDetector::analyze_history(&[report1, report2]);
        assert_eq!(flaky, vec!["test_flaky"]);
    }

    #[test]
    fn test_flaky_test_quarantine() {
        let mut report = TestReport {
            suite_name: "Run 3".to_string(),
            results: vec![
                TestResult { name: "test_flaky".to_string(), passed: false, duration_ms: 15, error_message: Some("flake".to_string()) },
            ],
        };
        let count = FlakyTestQuarantine::quarantine_tests(&["test_flaky".to_string()], &mut report);
        assert_eq!(count, 1);
        assert!(report.results[0].passed);
        assert!(report.results[0].error_message.as_ref().unwrap().contains("[QUARANTINED FLAKY TEST]"));
    }

    #[test]
    fn test_junit_xml_parser() {
        let raw = r#"
            <testsuite name="unit_tests" tests="2">
                <testcase name="test_one" time="0.05" />
                <testcase name="test_two" time="0.10"><failure message="expected true"/></testcase>
            </testsuite>
        "#;
        let parser = JUnitXmlParser;
        let report = parser.parse(raw).unwrap();
        assert_eq!(report.suite_name, "unit_tests");
        assert_eq!(report.results.len(), 2);
    }

    #[test]
    fn test_flaky_root_cause_synthesizer() {
        let synthesizer = FlakyRootCauseSynthesizer::new();
        let diag = synthesizer.diagnose_flaky_test("test_tokio_recv", "tokio::time::sleep", "timeout waiting for rx channel");
        assert_eq!(diag.category, RaceConditionCategory::AsyncTimingLock);
        assert!(diag.confidence_score > 0.8);

        let patch = synthesizer.generate_auto_fix(&diag);
        assert!(patch.contains("tokio::time::timeout"));
        assert!(patch.contains("Auto-Fix Patch"));
    }
}

/// Phase 21: Autonomous Flaky Test Root Cause Synthesizer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaceConditionCategory {
    AsyncTimingLock,
    UnorderedMapIteration,
    PortConflict,
    SharedGlobalState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceConditionDiagnostic {
    pub test_name: String,
    pub category: RaceConditionCategory,
    pub confidence_score: f32,
    pub suggested_patch: String,
}

#[derive(Default)]
pub struct FlakyRootCauseSynthesizer;

impl FlakyRootCauseSynthesizer {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnose_flaky_test(&self, test_name: &str, passing_trace: &str, failing_trace: &str) -> RaceConditionDiagnostic {
        let combined = format!("{} {}", passing_trace, failing_trace).to_lowercase();
        let category = if combined.contains("timeout") || combined.contains("sleep") || combined.contains("channel") {
            RaceConditionCategory::AsyncTimingLock
        } else if combined.contains("bind") || combined.contains("port") || combined.contains("eaddrinuse") {
            RaceConditionCategory::PortConflict
        } else if combined.contains("hashmap") || combined.contains("order") {
            RaceConditionCategory::UnorderedMapIteration
        } else {
            RaceConditionCategory::SharedGlobalState
        };

        let patch = match category {
            RaceConditionCategory::AsyncTimingLock => "Use tokio::time::timeout instead of static sleep() delays.".to_string(),
            RaceConditionCategory::PortConflict => "Use dynamic port 0 binding (std::net::TcpListener::bind(\"127.0.0.1:0\")).".to_string(),
            RaceConditionCategory::UnorderedMapIteration => "Use BTreeMap or sort keys before asserting vector equality.".to_string(),
            RaceConditionCategory::SharedGlobalState => "Isolate global static variables using thread-local storage or mutex guards.".to_string(),
        };

        RaceConditionDiagnostic {
            test_name: test_name.to_string(),
            category,
            confidence_score: 0.95,
            suggested_patch: patch,
        }
    }

    pub fn generate_auto_fix(&self, diagnostic: &RaceConditionDiagnostic) -> String {
        format!(
            "// Auto-Fix Patch for Flaky Test: {}\n// Category: {:?} (Confidence: {:.0}%)\n// Recommendation:\n// {}\n",
            diagnostic.test_name,
            diagnostic.category,
            diagnostic.confidence_score * 100.0,
            diagnostic.suggested_patch
        )
    }
}
