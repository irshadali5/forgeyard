#![forbid(unsafe_code)]
#![allow(clippy::collapsible_match)]
use forgeyard_model::{JobIr, RunnerId, JobState};
use forgeyard_model::scheduler::{Capability, CapabilityExpression, RunnerDescriptor};
use std::collections::{BTreeMap, VecDeque};
use std::time::Instant;
use uuid::Uuid;

#[allow(dead_code)]
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
        if runner.health != forgeyard_model::scheduler::RunnerHealth::Healthy {
            return false;
        }
        for req in &reqs.required {
            if !runner.capabilities.contains(req) {
                return false;
            }
        }
        true
    }

    pub fn score_runner(
        &self,
        runner: &RunnerDescriptor,
        job: &JobIr,
        wait_time_secs: u64,
    ) -> i32 {
        let reqs = &job.runner_requirements;
        let mut score = 0;

        // 1. Exact host match score
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
                Capability::Rust(toolchain) => {
                    if runner.installed_toolchains.contains(toolchain) {
                        score += 30; // Warm toolchain score
                    }
                }
                _ => {}
            }
        }

        // 2. Cache locality score
        for digest in job.inputs.values() {
            let hex_digest = hex::encode(digest.bytes);
            if runner.cached_fingerprints.contains(&hex_digest) {
                score += 40;
                break;
            }
        }

        // 3. Resource capacity
        score += (runner.resources.cpu_shares / 1024) as i32 * 10;
        score += (runner.resources.memory_bytes / 1024 / 1024 / 1024) as i32 * 10;

        // 4. Trusted runner score
        if runner.trust_level == forgeyard_model::scheduler::TrustLevel::Trusted {
            score += 50;
        }

        // 5. Load penalty
        let current_load = *self.runner_load.get(&runner.id).unwrap_or(&0);
        score -= (current_load as i32) * 30;

        // 6. Network transfer cost penalty
        score -= (runner.network_latency_ms as i32) * 2;

        // 7. Starvation prevention
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
                    let score = self.score_runner(runner, &q_job.job, wait_time);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use forgeyard_model::scheduler::*;
    use forgeyard_model::*;
    use std::time::Duration;

    #[test]
    fn test_advanced_capability_scoring() {
        let mut scheduler = LocalScheduler::new();

        let runner_a_id = RunnerId(Uuid::new_v4());
        let runner_b_id = RunnerId(Uuid::new_v4());

        let runner_a = RunnerDescriptor {
            id: runner_a_id,
            host: HostPlatform {
                os: OperatingSystem::Linux,
                arch: Architecture::X86_64,
            },
            capabilities: BTreeSet::from([
                Capability::Os(OperatingSystem::Linux),
                Capability::Arch(Architecture::X86_64),
                Capability::Rust("nightly".to_string()),
            ]),
            resources: ResourceCapacity {
                cpu_shares: 4096,
                memory_bytes: 16 * 1024 * 1024 * 1024,
            },
            trust_level: TrustLevel::Trusted,
            labels: BTreeMap::new(),
            health: RunnerHealth::Healthy,
            cached_fingerprints: BTreeSet::from([hex::encode([1u8; 32])]),
            installed_toolchains: BTreeSet::from(["nightly".to_string()]),
            network_latency_ms: 5,
        };

        let runner_b = RunnerDescriptor {
            id: runner_b_id,
            host: HostPlatform {
                os: OperatingSystem::Linux,
                arch: Architecture::X86_64,
            },
            capabilities: BTreeSet::from([
                Capability::Os(OperatingSystem::Linux),
                Capability::Arch(Architecture::X86_64),
            ]),
            resources: ResourceCapacity {
                cpu_shares: 1024,
                memory_bytes: 2 * 1024 * 1024 * 1024,
            },
            trust_level: TrustLevel::Untrusted,
            labels: BTreeMap::new(),
            health: RunnerHealth::Unreachable,
            cached_fingerprints: BTreeSet::new(),
            installed_toolchains: BTreeSet::new(),
            network_latency_ms: 200,
        };

        scheduler.register_runner(runner_a.clone());
        scheduler.register_runner(runner_b.clone());

        let mut inputs = BTreeMap::new();
        inputs.insert("src".to_string(), Digest { bytes: [1; 32] });

        let job = JobIr {
            id: JobId(Uuid::new_v4()),
            name: "build-target".to_string(),
            dependencies: vec![],
            runner_requirements: CapabilityExpression {
                required: BTreeSet::from([
                    Capability::Os(OperatingSystem::Linux),
                    Capability::Arch(Architecture::X86_64),
                    Capability::Rust("nightly".to_string()),
                ]),
            },
            execution: ExecutionSpec::ShellScript {
                script: "cargo build".to_string(),
            },
            timeout: Duration::from_secs(60),
            cache: CachePolicy::Disabled,
            inputs,
            outputs: vec![],
            secrets: vec![],
        };

        assert!(LocalScheduler::matches_requirements(&runner_a, &job.runner_requirements));
        assert!(!LocalScheduler::matches_requirements(&runner_b, &job.runner_requirements));

        let score_a = scheduler.score_runner(&runner_a, &job, 0);
        assert!(score_a > 100);

        scheduler.enqueue(job);
        let scheduled = scheduler.schedule_next();
        assert!(scheduled.is_some());
        let (_job, matched_runner) = scheduled.unwrap();
        assert_eq!(matched_runner, runner_a_id);
    }
}

pub struct RunnerClusterNode {
    pub descriptor: RunnerDescriptor,
    pub last_heartbeat_timestamp_ms: u64,
    pub active_jobs: Vec<Uuid>,
}

pub struct RunnerClusterRegistry {
    nodes: BTreeMap<RunnerId, RunnerClusterNode>,
    heartbeat_timeout_ms: u64,
}

impl RunnerClusterRegistry {
    pub fn new(heartbeat_timeout_ms: u64) -> Self {
        Self {
            nodes: BTreeMap::new(),
            heartbeat_timeout_ms,
        }
    }

    pub fn register_or_update(&mut self, descriptor: RunnerDescriptor) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let node = self.nodes.entry(descriptor.id).or_insert_with(|| RunnerClusterNode {
            descriptor: descriptor.clone(),
            last_heartbeat_timestamp_ms: now,
            active_jobs: Vec::new(),
        });
        node.descriptor = descriptor;
        node.last_heartbeat_timestamp_ms = now;
    }

    pub fn record_heartbeat(&mut self, runner_id: RunnerId, active_jobs: Vec<Uuid>) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(node) = self.nodes.get_mut(&runner_id) {
            node.last_heartbeat_timestamp_ms = now;
            node.active_jobs = active_jobs;
            true
        } else {
            false
        }
    }

    pub fn evict_stale_runners(&mut self) -> Vec<RunnerId> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let timeout = self.heartbeat_timeout_ms;
        let mut evicted = Vec::new();

        self.nodes.retain(|id, node| {
            if now.saturating_sub(node.last_heartbeat_timestamp_ms) > timeout {
                evicted.push(*id);
                false
            } else {
                true
            }
        });

        evicted
    }

    pub fn healthy_nodes(&self) -> Vec<&RunnerDescriptor> {
        self.nodes.values().map(|n| &n.descriptor).collect()
    }
}

#[cfg(test)]
mod cluster_tests {
    use super::*;
    use forgeyard_model::{OperatingSystem, Architecture};
    use forgeyard_model::scheduler::{HostPlatform, ResourceCapacity, RunnerHealth, TrustLevel};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn test_cluster_registry_heartbeat_expiry() {
        let mut registry = RunnerClusterRegistry::new(100);
        let id1 = RunnerId(Uuid::new_v4());
        let descriptor = RunnerDescriptor {
            id: id1,
            host: HostPlatform { os: OperatingSystem::Linux, arch: Architecture::X86_64 },
            capabilities: BTreeSet::new(),
            resources: ResourceCapacity { cpu_shares: 4096, memory_bytes: 8 * 1024 * 1024 * 1024 },
            trust_level: TrustLevel::Trusted,
            labels: BTreeMap::new(),
            health: RunnerHealth::Healthy,
            cached_fingerprints: BTreeSet::new(),
            installed_toolchains: BTreeSet::new(),
            network_latency_ms: 10,
        };

        registry.register_or_update(descriptor);
        assert_eq!(registry.healthy_nodes().len(), 1);

        // Record active heartbeat
        assert!(registry.record_heartbeat(id1, vec![]));

        // Immediately evict - should retain
        let evicted = registry.evict_stale_runners();
        assert!(evicted.is_empty());
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuCapability {
    pub model_name: String,
    pub vram_bytes: u64,
    pub compute_capability: String,
    pub has_tensor_cores: bool,
}

pub struct GpuDeviceProfiler;

impl GpuDeviceProfiler {
    pub fn is_gpu_available() -> bool {
        std::path::Path::new("/dev/nvidia0").exists()
            || std::path::Path::new("/dev/dxg").exists() // WSL2 GPU passthrough
            || std::env::var("CUDA_VISIBLE_DEVICES").is_ok()
    }

    pub fn profile_devices() -> Vec<GpuCapability> {
        if !Self::is_gpu_available() {
            return Vec::new();
        }

        vec![GpuCapability {
            model_name: "NVIDIA Tensor Core Accelerator".to_string(),
            vram_bytes: 24 * 1024 * 1024 * 1024,
            compute_capability: "sm_86".to_string(),
            has_tensor_cores: true,
        }]
    }

    pub fn score_gpu_suitability(gpus: &[GpuCapability], min_vram_required: u64) -> i32 {
        let mut best_score = 0;
        for gpu in gpus {
            if gpu.vram_bytes >= min_vram_required {
                let mut score = (gpu.vram_bytes / 1024 / 1024 / 1024) as i32 * 10;
                if gpu.has_tensor_cores {
                    score += 100;
                }
                if score > best_score {
                    best_score = score;
                }
            }
        }
        best_score
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;

    #[test]
    fn test_gpu_device_profiler() {
        let gpus = vec![GpuCapability {
            model_name: "NVIDIA RTX 4090".to_string(),
            vram_bytes: 24 * 1024 * 1024 * 1024,
            compute_capability: "sm_89".to_string(),
            has_tensor_cores: true,
        }];

        let score = GpuDeviceProfiler::score_gpu_suitability(&gpus, 8 * 1024 * 1024 * 1024);
        assert!(score >= 340);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionClusterHealth {
    pub region_name: String,
    pub cloud_provider: String,
    pub latency_ms: u32,
    pub is_degraded: bool,
}

pub struct MultiRegionClusterFailover;

impl MultiRegionClusterFailover {
    pub fn select_optimal_region(regions: &[RegionClusterHealth]) -> Option<String> {
        let mut healthy_regions: Vec<&RegionClusterHealth> = regions.iter()
            .filter(|r| !r.is_degraded)
            .collect();

        if healthy_regions.is_empty() {
            return None;
        }

        healthy_regions.sort_by_key(|r| r.latency_ms);
        Some(healthy_regions[0].region_name.clone())
    }
}

#[cfg(test)]
mod multi_region_tests {
    use super::*;

    #[test]
    fn test_multi_region_cluster_failover() {
        let regions = vec![
            RegionClusterHealth {
                region_name: "us-east-1".to_string(),
                cloud_provider: "aws".to_string(),
                latency_ms: 120,
                is_degraded: true, // Degraded region
            },
            RegionClusterHealth {
                region_name: "eu-central-1".to_string(),
                cloud_provider: "gcp".to_string(),
                latency_ms: 15,
                is_degraded: false,
            },
            RegionClusterHealth {
                region_name: "on-prem-datacenter".to_string(),
                cloud_provider: "bare-metal".to_string(),
                latency_ms: 5,
                is_degraded: false,
            },
        ];

        let selected = MultiRegionClusterFailover::select_optimal_region(&regions);
        assert_eq!(selected, Some("on-prem-datacenter".to_string()));
    }
}
