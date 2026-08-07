use forgeyard_model::{JobIr, RunnerId, JobState};
use forgeyard_model::scheduler::{Capability, CapabilityExpression, RunnerDescriptor};
use std::collections::{BTreeMap, VecDeque};
use std::time::Instant;
use uuid::Uuid;

struct QueuedJob {
    pub job: JobIr,
    pub enqueued_at: Instant,
    pub retries: u32,
}

pub struct LocalScheduler {
    ready_queue: VecDeque<QueuedJob>,
    pub running_jobs: BTreeMap<Uuid, JobIr>,
    pub job_states: BTreeMap<Uuid, JobState>,
    pub runners: BTreeMap<RunnerId, RunnerDescriptor>,
    pub runner_load: BTreeMap<RunnerId, usize>,
}

impl Default for LocalScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            running_jobs: BTreeMap::new(),
            job_states: BTreeMap::new(),
            runners: BTreeMap::new(),
            runner_load: BTreeMap::new(),
        }
    }

    pub fn register_runner(&mut self, runner: RunnerDescriptor) {
        self.runner_load.entry(runner.id).or_insert(0);
        self.runners.insert(runner.id, runner);
    }

    pub fn remove_runner(&mut self, runner_id: RunnerId) {
        self.runners.remove(&runner_id);
        self.runner_load.remove(&runner_id);
    }

    pub fn enqueue(&mut self, job: JobIr) {
        self.job_states.insert(job.id.0, JobState::Ready);
        self.ready_queue.push_back(QueuedJob {
            job,
            enqueued_at: Instant::now(),
            retries: 0,
        });
    }

    pub fn update_job_state(&mut self, job_id: Uuid, state: JobState, runner_id: Option<RunnerId>) {
        if state == JobState::Succeeded || state == JobState::Failed || state == JobState::Cancelled {
            self.running_jobs.remove(&job_id);
            if let Some(r_id) = runner_id {
                if let Some(load) = self.runner_load.get_mut(&r_id) {
                    *load = load.saturating_sub(1);
                }
            }
        }
        self.job_states.insert(job_id, state);
    }

    pub fn matches_requirements(runner: &RunnerDescriptor, reqs: &CapabilityExpression) -> bool {
        for req in &reqs.required {
            if !runner.capabilities.contains(req) {
                return false;
            }
        }
        true
    }

    pub fn score_runner(&self, runner: &RunnerDescriptor, reqs: &CapabilityExpression, wait_time_secs: u64) -> i32 {
        let mut score = 0;
        
        // Exact host match score
        for req in &reqs.required {
            match req {
                Capability::Os(os) => {
                    if runner.host.os == *os {
                        score += 50;
                    }
                }
                Capability::Arch(arch) => {
                    if runner.host.arch == *arch {
                        score += 50;
                    }
                }
                _ => {}
            }
        }
        
        // Resource capacity (more capacity = better)
        score += (runner.resources.cpu_shares / 1024) as i32 * 10;
        score += (runner.resources.memory_bytes / 1024 / 1024 / 1024) as i32 * 10;
        
        // Load penalty (prefer less loaded runners)
        let current_load = *self.runner_load.get(&runner.id).unwrap_or(&0);
        score -= (current_load as i32) * 30;

        // Starvation prevention: slightly boost based on wait time so older jobs find a runner
        score += (wait_time_secs / 10) as i32;

        score
    }

    pub fn schedule_next(&mut self) -> Option<(JobIr, RunnerId)> {
        let mut best_match: Option<(usize, RunnerId, i32)> = None;

        for (idx, q_job) in self.ready_queue.iter().enumerate() {
            let wait_time = q_job.enqueued_at.elapsed().as_secs();
            let mut best_score = -10000;
            let mut best_runner = None;

            for (runner_id, runner) in &self.runners {
                if Self::matches_requirements(runner, &q_job.job.runner_requirements) {
                    let score = self.score_runner(runner, &q_job.job.runner_requirements, wait_time);
                    if score > best_score {
                        best_score = score;
                        best_runner = Some(*runner_id);
                    }
                }
            }

            if let Some(runner_id) = best_runner {
                best_match = Some((idx, runner_id, best_score));
                // Stop searching the queue if we found an excellent match for the oldest job
                if best_score > 50 {
                    break;
                }
            }
        }

        if let Some((idx, runner_id, _score)) = best_match {
            if let Some(q_job) = self.ready_queue.remove(idx) {
                let job = q_job.job;
                self.job_states.insert(job.id.0, JobState::Leased);
                self.running_jobs.insert(job.id.0, job.clone());
                
                if let Some(load) = self.runner_load.get_mut(&runner_id) {
                    *load += 1;
                }
                
                Some((job, runner_id))
            } else {
                None
            }
        } else {
            None
        }
    }
}
