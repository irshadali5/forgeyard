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

        findings
    }
}

pub type BasicPolicy = SecurityPolicy;
