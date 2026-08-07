pub struct PolicyInput {
    pub job_id: String,
    pub repository: String,
    pub command: Vec<String>,
    pub run_as_root: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PolicyFindingStatus {
    Pass,
    Warning,
    Fail,
}

pub struct PolicyFinding {
    pub rule: String,
    pub status: PolicyFindingStatus,
    pub message: String,
}

pub trait Policy: Send + Sync {
    fn evaluate(&self, input: &PolicyInput) -> Vec<PolicyFinding>;
}

pub struct SecurityPolicy {
    pub allow_root: bool,
    pub forbidden_patterns: Vec<String>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_root: false,
            forbidden_patterns: vec![
                "rm -rf /".to_string(),
                ":(){ :|:& };:".to_string(),
                "dd if=/dev/zero".to_string(),
            ],
        }
    }
}

impl Policy for SecurityPolicy {
    fn evaluate(&self, input: &PolicyInput) -> Vec<PolicyFinding> {
        let mut findings = Vec::new();

        // Rule 1: Root execution check
        if input.run_as_root && !self.allow_root {
            findings.push(PolicyFinding {
                rule: "no_root_execution".to_string(),
                status: PolicyFindingStatus::Fail,
                message: format!("Job {} attempts to execute as root user", input.job_id),
            });
        } else {
            findings.push(PolicyFinding {
                rule: "no_root_execution".to_string(),
                status: PolicyFindingStatus::Pass,
                message: "User privileges compliant".to_string(),
            });
        }

        // Rule 2: Forbidden command inspection
        let full_command = input.command.join(" ");
        let mut command_safe = true;
        for pattern in &self.forbidden_patterns {
            if full_command.contains(pattern) {
                command_safe = false;
                findings.push(PolicyFinding {
                    rule: "disallowed_command".to_string(),
                    status: PolicyFindingStatus::Fail,
                    message: format!("Disallowed command pattern detected: {}", pattern),
                });
            }
        }

        if command_safe {
            findings.push(PolicyFinding {
                rule: "command_safety".to_string(),
                status: PolicyFindingStatus::Pass,
                message: "Command safety check passed".to_string(),
            });
        }

        // Rule 3: Secret exposure inspection
        if full_command.contains("printenv") || full_command.contains("env ") || full_command.contains("echo $") {
            findings.push(PolicyFinding {
                rule: "secret_exposure_guard".to_string(),
                status: PolicyFindingStatus::Warning,
                message: "Potential environment variable or secret printing detected in command string".to_string(),
            });
        } else {
            findings.push(PolicyFinding {
                rule: "secret_exposure_guard".to_string(),
                status: PolicyFindingStatus::Pass,
                message: "No suspicious environment print statements detected".to_string(),
            });
        }

        findings
    }
}

pub type BasicPolicy = SecurityPolicy;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityItem {
    pub id: String,
    pub package_name: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: SeverityLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReport {
    pub target: String,
    pub scanner_name: String,
    pub vulnerabilities: Vec<VulnerabilityItem>,
}

pub struct VulnerabilityPolicy {
    pub max_allowed_severity: SeverityLevel,
    pub fail_on_unpatched: bool,
}

impl Default for VulnerabilityPolicy {
    fn default() -> Self {
        Self {
            max_allowed_severity: SeverityLevel::Medium,
            fail_on_unpatched: true,
        }
    }
}

impl VulnerabilityPolicy {
    pub fn evaluate_report(&self, report: &VulnerabilityReport) -> Vec<PolicyFinding> {
        let mut findings = Vec::new();

        for vuln in &report.vulnerabilities {
            if vuln.severity > self.max_allowed_severity {
                findings.push(PolicyFinding {
                    rule: "vulnerability_severity_exceeded".to_string(),
                    status: PolicyFindingStatus::Fail,
                    message: format!(
                        "CVE [{}] in package {} ({:?}) exceeds maximum allowed severity threshold ({:?})",
                        vuln.id, vuln.package_name, vuln.severity, self.max_allowed_severity
                    ),
                });
            } else if vuln.severity == SeverityLevel::Medium {
                findings.push(PolicyFinding {
                    rule: "vulnerability_warning".to_string(),
                    status: PolicyFindingStatus::Warning,
                    message: format!(
                        "Medium severity vulnerability [{}] detected in package {}",
                        vuln.id, vuln.package_name
                    ),
                });
            }

            if self.fail_on_unpatched && vuln.fixed_version.is_none() && vuln.severity >= SeverityLevel::High {
                findings.push(PolicyFinding {
                    rule: "unpatched_high_vulnerability".to_string(),
                    status: PolicyFindingStatus::Fail,
                    message: format!(
                        "Unpatched vulnerability [{}] in package {} with no fixed version",
                        vuln.id, vuln.package_name
                    ),
                });
            }
        }

        if findings.is_empty() {
            findings.push(PolicyFinding {
                rule: "vulnerability_compliance".to_string(),
                status: PolicyFindingStatus::Pass,
                message: format!("Vulnerability scan for {} compliant with policy", report.target),
            });
        }

        findings
    }
}

pub struct CargoAuditScanner;

impl CargoAuditScanner {
    pub fn parse_output(target: &str, cargo_audit_json: &str) -> VulnerabilityReport {
        let mut vulnerabilities = Vec::new();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(cargo_audit_json) {
            if let Some(vulnerabilities_arr) = value.get("vulnerabilities").and_then(|v| v.get("list")).and_then(|v| v.as_array()) {
                for item in vulnerabilities_arr {
                    let id = item.get("advisory").and_then(|a| a.get("id")).and_then(|s| s.as_str()).unwrap_or("CVE-UNKNOWN").to_string();
                    let package_name = item.get("package").and_then(|p| p.get("name")).and_then(|s| s.as_str()).unwrap_or("unknown").to_string();
                    let installed_version = item.get("package").and_then(|p| p.get("version")).and_then(|s| s.as_str()).unwrap_or("0.0.0").to_string();

                    vulnerabilities.push(VulnerabilityItem {
                        id,
                        package_name,
                        installed_version,
                        fixed_version: None,
                        severity: SeverityLevel::High,
                        description: "cargo-audit advisory finding".to_string(),
                    });
                }
            }
        }

        VulnerabilityReport {
            target: target.to_string(),
            scanner_name: "cargo-audit".to_string(),
            vulnerabilities,
        }
    }
}

pub struct TrivyScanner;

impl TrivyScanner {
    pub fn parse_output(target: &str, trivy_json: &str) -> VulnerabilityReport {
        let mut vulnerabilities = Vec::new();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trivy_json) {
            if let Some(results) = value.get("Results").and_then(|v| v.as_array()) {
                for res in results {
                    if let Some(vulns) = res.get("Vulnerabilities").and_then(|v| v.as_array()) {
                        for item in vulns {
                            let id = item.get("VulnerabilityID").and_then(|s| s.as_str()).unwrap_or("CVE-UNKNOWN").to_string();
                            let package_name = item.get("PkgName").and_then(|s| s.as_str()).unwrap_or("unknown").to_string();
                            let installed_version = item.get("InstalledVersion").and_then(|s| s.as_str()).unwrap_or("0.0.0").to_string();
                            let fixed_version = item.get("FixedVersion").and_then(|s| s.as_str()).map(|s| s.to_string());
                            let sev_str = item.get("Severity").and_then(|s| s.as_str()).unwrap_or("MEDIUM");

                            let severity = match sev_str.to_uppercase().as_str() {
                                "CRITICAL" => SeverityLevel::Critical,
                                "HIGH" => SeverityLevel::High,
                                "LOW" => SeverityLevel::Low,
                                _ => SeverityLevel::Medium,
                            };

                            vulnerabilities.push(VulnerabilityItem {
                                id,
                                package_name,
                                installed_version,
                                fixed_version,
                                severity,
                                description: "Trivy container scan finding".to_string(),
                            });
                        }
                    }
                }
            }
        }

        VulnerabilityReport {
            target: target.to_string(),
            scanner_name: "trivy".to_string(),
            vulnerabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_policy_thresholds() {
        let policy = VulnerabilityPolicy::default();
        let report = VulnerabilityReport {
            target: "app:latest".to_string(),
            scanner_name: "trivy".to_string(),
            vulnerabilities: vec![
                VulnerabilityItem {
                    id: "CVE-2026-1001".to_string(),
                    package_name: "openssl".to_string(),
                    installed_version: "1.1.1".to_string(),
                    fixed_version: Some("1.1.1w".to_string()),
                    severity: SeverityLevel::Critical,
                    description: "Buffer overflow".to_string(),
                },
                VulnerabilityItem {
                    id: "CVE-2026-1002".to_string(),
                    package_name: "zlib".to_string(),
                    installed_version: "1.2.11".to_string(),
                    fixed_version: None,
                    severity: SeverityLevel::Medium,
                    description: "Minor leak".to_string(),
                },
            ],
        };

        let findings = policy.evaluate_report(&report);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].status, PolicyFindingStatus::Fail);
        assert_eq!(findings[1].status, PolicyFindingStatus::Warning);
    }

    #[test]
    fn test_trivy_scanner_parsing() {
        let raw = r#"{
            "Results": [{
                "Vulnerabilities": [{
                    "VulnerabilityID": "CVE-2026-9999",
                    "PkgName": "curl",
                    "InstalledVersion": "7.68.0",
                    "FixedVersion": "7.68.0-1ubuntu2.1",
                    "Severity": "CRITICAL"
                }]
            }]
        }"#;
        let report = TrivyScanner::parse_output("ubuntu:latest", raw);
        assert_eq!(report.vulnerabilities.len(), 1);
        assert_eq!(report.vulnerabilities[0].id, "CVE-2026-9999");
        assert_eq!(report.vulnerabilities[0].severity, SeverityLevel::Critical);
    }
}
