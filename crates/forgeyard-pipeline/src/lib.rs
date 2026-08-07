pub mod dag;
pub mod matrix;

use std::fmt;
use std::collections::{BTreeMap, HashSet};
use forgeyard_model::{JobIr, PipelineIr, JobId, PipelineId, ExecutionSpec, NetworkPolicy, ResourceRequest, OperatingSystem};
use forgeyard_model::scheduler::Capability;
use forgeyard_config::ForgeyardConfig;
use uuid::Uuid;
use crate::dag::PipelineDag;
use crate::matrix::MatrixExpander;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PipelineError(pub String);

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pipeline Error: {}", self.0)
    }
}

impl std::error::Error for PipelineError {}

pub struct PipelineCompiler;

impl PipelineCompiler {
    pub fn compile(
        config: &ForgeyardConfig,
        target_pipeline: &str,
    ) -> Result<PipelineIr, PipelineError> {
        let pipeline_cfg = config.pipelines.get(target_pipeline)
            .ok_or_else(|| PipelineError(format!("Pipeline '{}' not found in config", target_pipeline)))?;

        let mut dag = PipelineDag::new();
        let mut known_jobs = HashSet::new();

        // Register nodes
        for (job_name, _job_cfg) in &pipeline_cfg.jobs {
            dag.add_node(job_name.clone());
            known_jobs.insert(job_name.clone());
        }

        // Register edges
        for (job_name, job_cfg) in &pipeline_cfg.jobs {
            for dep in &job_cfg.needs {
                dag.add_edge(dep.clone(), job_name.clone());
            }
        }

        // Validate DAG (detect cycles and missing deps)
        let sorted_job_names = dag.validate(&known_jobs)
            .map_err(|e| PipelineError(e.to_string()))?;

        let mut compiled_jobs = BTreeMap::new();
        let mut job_name_to_id: BTreeMap<String, Vec<JobId>> = BTreeMap::new();

        for base_name in sorted_job_names {
            let job_cfg = pipeline_cfg.jobs.get(&base_name).unwrap();
            
            // Resolve dependencies based on the matrix expansion of parents
            let mut resolved_deps = Vec::new();
            for dep_name in &job_cfg.needs {
                if let Some(ids) = job_name_to_id.get(dep_name) {
                    resolved_deps.extend(ids.iter().cloned());
                }
            }

            let matrix_contexts = MatrixExpander::expand(&job_cfg.matrix);
            let mut current_job_ids = Vec::new();

            for ctx in matrix_contexts {
                let job_id = JobId(Uuid::new_v4());
                
                // Formulate a unique name if matrix variables are present
                let mut display_name = base_name.clone();
                let mut os_req = None;
                if !ctx.variables.is_empty() {
                    let vars: Vec<String> = ctx.variables.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                    display_name = format!("{}[{}]", base_name, vars.join(", "));
                    
                    if let Some(target) = ctx.variables.get("target") {
                        if target.contains("windows-msvc") {
                            os_req = Some(OperatingSystem::Windows);
                        } else if target.contains("apple") {
                            os_req = Some(OperatingSystem::MacOS);
                        } else {
                            os_req = Some(OperatingSystem::Linux);
                        }
                    }
                }

                // Map execution spec from command
                // In a production app, we would detect container vs command here. We default to shell for config string vectors.
                let program = "bash".to_string();
                let arguments = vec!["-c".to_string(), job_cfg.command.join(" \n ")];

                let execution = ExecutionSpec::Command {
                    program,
                    arguments,
                    working_directory: camino::Utf8PathBuf::from("."),
                    environment: ctx.variables.clone(),
                    network: NetworkPolicy::AllowAll,
                    resources: ResourceRequest::default(),
                };

                let mut required_caps = std::collections::BTreeSet::new();
                if let Some(os) = os_req {
                    required_caps.insert(Capability::Os(os));
                }

                let job_ir = JobIr {
                    id: job_id,
                    name: display_name,
                    dependencies: resolved_deps.clone(),
                    runner_requirements: forgeyard_model::scheduler::CapabilityExpression {
                        required: required_caps,
                    },
                    execution,
                    timeout: Duration::from_secs(3600),
                    cache: forgeyard_model::CachePolicy::Disabled,
                    inputs: BTreeMap::new(),
                    outputs: Vec::new(),
                    secrets: Vec::new(),
                };

                compiled_jobs.insert(job_id, job_ir);
                current_job_ids.push(job_id);
            }
            
            job_name_to_id.insert(base_name, current_job_ids);
        }

        let pipeline_id = PipelineId(Uuid::new_v4());

        // Construct edges explicitly for the IR
        let mut edges = Vec::new();
        for job in compiled_jobs.values() {
            for dep in &job.dependencies {
                edges.push((*dep, job.id));
            }
        }

        Ok(PipelineIr {
            pipeline_id,
            jobs: compiled_jobs,
            edges,
        })
    }

    pub fn fingerprint_job(
        job: &JobIr,
        deps: &Vec<String>,
    ) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&job.name);
        for dep in deps {
            hasher.update(dep.as_bytes());
        }
        let hash = hasher.finalize();
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
