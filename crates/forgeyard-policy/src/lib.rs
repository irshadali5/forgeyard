pub struct PolicyInput {
    pub job_id: String,
    pub repository: String,
    // Provide whatever data is needed to evaluate
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

pub struct BasicPolicy;

impl Policy for BasicPolicy {
    fn evaluate(&self, _input: &PolicyInput) -> Vec<PolicyFinding> {
        vec![PolicyFinding {
            rule: "default_allow".to_string(),
            status: PolicyFindingStatus::Pass,
            message: "Default policy passed.".to_string(),
        }]
    }
}
