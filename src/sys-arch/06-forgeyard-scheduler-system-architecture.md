# 06 — Forgeyard Scheduler System Architecture

**Document type:** Core Execution Orchestration System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Job eligibility, placement, runner selection, capability matching, resource accounting, locality scoring, fairness, queueing, priorities, lease issuance, backpressure, preemption policy, scheduling recovery, and distributed-safe orchestration  
**Architecture style:** Two-phase scheduling — hard eligibility filtering followed by soft scoring — with persisted lease authority and reconciliation  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on `05-forgeyard-run-job-state-machine.md`, consuming `Eligible` jobs and creating authoritative attempts/leases. It also integrates with the CAS data plane, pipeline capability planning, platform/device capability models, and storage layer.

---

# 1. Purpose

Forgeyard needs a scheduler that can place work correctly across:

```text
local runner
distributed Linux runners
Windows runners
macOS runners
Android device pools
Apple device pools
GPU runners
high-memory runners
special toolchain runners
trusted signing workers
confidential execution workers
```

The scheduler must decide:

```text
where should this eligible job run?
```

without becoming the authority for:

```text
job state
attempt state
lease truth
```

The central rule is:

> **The scheduler decides placement; the Run/Job state machine and store decide authority.**

A second rule:

> **Scheduling is always hard filtering first, soft scoring second.**

A third rule:

> **A runner is eligible only if every required capability can be satisfied. Locality, load, queue time, cache warmth, and cost influence ranking only after correctness constraints are met.**

---

# 2. Architectural Position

```text
                  Eligible Jobs
                       │
                       ▼
               Scheduler Ingress
                       │
                       ▼
              Hard Requirement Match
                       │
                       ▼
              Eligible Runner Set
                       │
                       ▼
                  Soft Scoring
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
      Locality       Headroom       Fairness
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                  PlacementDecision
                       │
                       ▼
          Attempt + Lease Transaction
                       │
                       ▼
                      Runner
```

---

# 3. Goals

The scheduler MUST:

1. consume only jobs in `Eligible`;
2. never mutate job state directly outside store/state APIs;
3. match all required capabilities;
4. support heterogeneous runner pools;
5. support OS/architecture constraints;
6. support toolchain/SDK constraints;
7. support resource constraints;
8. support GPU/device/special hardware;
9. support trust/security requirements;
10. support data locality;
11. support cache locality;
12. support queue fairness;
13. support tenant/project fairness;
14. support job priorities;
15. support backpressure;
16. support retry-aware placement;
17. support anti-affinity/reliability heuristics;
18. support runner drain;
19. support capacity reservations;
20. support preemption where explicitly allowed;
21. support standalone mode;
22. support distributed mode;
23. support HA-safe multiple daemon instances;
24. support deterministic explainability;
25. expose scheduler metrics;
26. tolerate runner churn;
27. tolerate stale capability reports;
28. avoid scheduling correctness based on predictions alone;
29. integrate with device leases;
30. integrate with CAS locality without making CAS locality mandatory.

---

# 4. Non-Goals

The scheduler does not:

```text
execute jobs
validate pipeline syntax
resolve secrets
perform sandbox setup
store artifact bytes
decide business authorization
```

It uses upstream validated data.

---

# 5. Workspace Structure

```text
crates/scheduler/
├── forgeyard-scheduler/
├── forgeyard-scheduler-model/
├── forgeyard-scheduler-eligibility/
├── forgeyard-scheduler-placement/
├── forgeyard-scheduler-score/
├── forgeyard-scheduler-resource/
├── forgeyard-scheduler-fairness/
├── forgeyard-scheduler-priority/
├── forgeyard-scheduler-queue/
├── forgeyard-scheduler-backpressure/
├── forgeyard-scheduler-preemption/
├── forgeyard-scheduler-locality/
├── forgeyard-scheduler-retry/
├── forgeyard-scheduler-leasing/
├── forgeyard-scheduler-reconcile/
├── forgeyard-scheduler-metrics/
└── forgeyard-scheduler-testkit/
```

---

# 6. Scheduler Input

The scheduler should consume an immutable scheduling view:

```rust
pub struct SchedulingJob {
    pub job_id: JobId,
    pub run_id: RunId,
    pub spec: JobSpecId,
    pub requirements: CapabilityRequirementSet,
    pub resources: ResourceRequest,
    pub priority: JobPriority,
    pub locality: LocalityHints,
    pub retry_context: RetryContext,
}
```

---

# 7. Runner Scheduling View

```rust
pub struct SchedulingRunner {
    pub runner_id: RunnerId,
    pub session_id: AgentSessionId,
    pub state: RunnerSchedulingState,
    pub capabilities: RunnerCapabilities,
    pub resources: RunnerResources,
    pub locality: RunnerLocality,
    pub trust: RunnerTrust,
    pub recent_health: RunnerHealthSummary,
}
```

---

# 8. Runner Scheduling States

```rust
pub enum RunnerSchedulingState {
    Online,
    Draining,
    Saturated,
    Unhealthy,
    Offline,
}
```

Only:

```text
Online
```

is normally eligible for new work.

---

# 9. Hard Eligibility

Hard eligibility answers:

```text
can this runner execute this job correctly?
```

This is a boolean decision.

---

# 10. Hard Requirement Categories

```text
OS
architecture
CPU features
memory
disk
GPU
toolchain
SDK
runtime
sandbox
device
trust level
network mode
signing capability
confidential execution
```

---

# 11. Capability Match

```rust
pub trait CapabilityMatcher {
    fn matches(
        &self,
        requirement: &CapabilityRequirement,
        runner: &RunnerCapabilities,
    ) -> CapabilityMatch;
}
```

---

# 12. Capability Match Result

```rust
pub enum CapabilityMatch {
    Satisfied,
    Unsatisfied(CapabilityMismatch),
    Unknown(CapabilityUnknown),
}
```

`Unknown` must not be treated as satisfied for hard requirements.

---

# 13. Versioned Capabilities

Example:

```text
Xcode >= 18.0
Android NDK r28
Rust toolchain digest X
Windows SDK version Y
GPU compute capability >= Z
```

Never reduce to:

```text
"mac"
"android"
"gpu"
```

---

# 14. Platform Matching

Examples:

```text
linux/x86_64
windows/x86_64
macos/aarch64
android/device
wasm/runtime
```

---

# 15. Architecture Matching

Use typed architecture:

```text
x86_64
aarch64
riscv64
wasm32
```

No freeform strings in core model.

---

# 16. CPU Feature Requirement

Example:

```text
AVX2
AES-NI
NEON
```

Only required if build/test semantics genuinely depend on it.

---

# 17. Resource Eligibility

Runner must have enough allocatable resources.

```text
requested CPU <= available CPU
requested memory <= available memory
requested disk <= available disk
GPU request satisfiable
```

---

# 18. Reservation vs Physical Capacity

Runner reports:

```text
physical capacity
reserved capacity
allocatable capacity
```

Scheduler uses allocatable.

---

# 19. Resource Model

```rust
pub struct RunnerResources {
    pub total: ResourceVector,
    pub reserved: ResourceVector,
    pub allocated: ResourceVector,
    pub allocatable: ResourceVector,
}
```

---

# 20. Resource Vector

```rust
pub struct ResourceVector {
    pub cpu: CpuUnits,
    pub memory: ByteSize,
    pub disk: ByteSize,
    pub gpu: Vec<GpuResource>,
}
```

---

# 21. CPU Units

Prefer fractional logical CPU units if needed.

Avoid raw floats.

---

# 22. Memory Headroom

Do not schedule exactly to physical limit by default.

Reserve:

```text
OS
agent
CAS
sandbox overhead
```

---

# 23. Disk Headroom

Input/output temp space must be considered.

---

# 24. Dynamic Resource Governor

Scheduler may reserve conservative capacity based on:

```text
historical peak
job declarations
platform overhead
```

Optimization only if bounded by declared hard limits.

---

# 25. Soft Scoring

After hard filtering:

```text
which eligible runner is best?
```

---

# 26. Score Model

```rust
pub struct PlacementScore {
    pub locality: Score,
    pub cache: Score,
    pub headroom: Score,
    pub queue: Score,
    pub reliability: Score,
    pub fairness: Score,
    pub cost: Score,
    pub affinity: Score,
}
```

---

# 27. Weighted Score

```text
total =
  W_locality * locality
+ W_cache * cache
+ W_headroom * headroom
+ W_queue * queue
+ W_reliability * reliability
+ W_fairness * fairness
+ W_cost * cost
+ W_affinity * affinity
```

Weights are policy/configurable.

---

# 28. Hard/Soft Separation

A high locality score can NEVER override:

```text
missing Xcode
not enough memory
wrong OS
wrong trust class
```

---

# 29. Locality

Locality sources:

```text
source snapshot present
toolchain present
dependency closure present
artifact inputs present
site-local CAS
region-local CAS
device proximity
```

---

# 30. Locality Hint

```rust
pub struct LocalityHints {
    pub required_objects: Vec<CasObjectId>,
    pub preferred_region: Option<RegionId>,
    pub site: Option<SiteId>,
}
```

---

# 31. CAS Locality

Scheduler may ask data plane:

```text
which objects are already present where?
```

---

# 32. Locality Accuracy

Locality data can be stale.

Therefore:

```text
locality affects score, not correctness
```

Runner still fetches/verifies missing objects.

---

# 33. Cache Warmth

Runner can report:

```text
toolchain IDs
known dependency closures
action-cache hints
```

Use bounded summaries.

---

# 34. Avoid Huge Capability Reports

Do not report millions of CAS object IDs in heartbeat.

Use:

```text
site cache presence
toolchain set
Bloom hints
recent closure IDs
```

---

# 35. Headroom Score

Prefer runner with enough remaining capacity after placement.

Avoid fragmentation.

---

# 36. Resource Fragmentation

Example:

```text
one 64 GB runner
many 8 GB jobs
```

Do not consume scarce high-memory runner unnecessarily if standard runner fits.

---

# 37. Scarcity-Aware Scoring

Capabilities can have scarcity weight:

```text
rare GPU
rare Xcode version
signing hardware
```

Prefer preserving rare capacity for jobs needing it.

---

# 38. Runner Reliability Score

Based on recent:

```text
lost attempts
sandbox failures
input fetch failures
disconnects
```

---

# 39. Reliability Guardrail

Do not permanently starve temporarily degraded runners.

Decay historical penalties over time.

---

# 40. Retry-Aware Placement

If previous attempt failed due to runner-specific infrastructure:

```text
prefer a different runner
```

---

# 41. Anti-Affinity

```rust
pub struct RetryAntiAffinity {
    pub avoid_runners: BTreeSet<RunnerId>,
    pub avoid_sites: BTreeSet<SiteId>,
}
```

---

# 42. Same-Runner Retry

Allowed if failure class suggests workload issue or no alternative exists.

---

# 43. Failure-Domain Awareness

Possible domains:

```text
runner
host
rack/site
region
cloud provider
```

---

# 44. Retry Across Failure Domains

For repeated infra failures:

```text
move to another host/site/region
```

if policy/cost allows.

---

# 45. Fairness

Scheduler must prevent one tenant/project from monopolizing shared capacity.

---

# 46. Fairness Dimensions

```text
tenant
organization
project
queue
priority class
```

---

# 47. Fairness Algorithm

Initial recommendation:

```text
weighted fair queueing + aging
```

Simpler than overly complex dominant-resource fairness initially.

---

# 48. Weighted Fair Queueing

Each tenant/project gets configured weight.

---

# 49. Aging

Long-waiting jobs gain scheduling preference.

Prevents starvation.

---

# 50. Priority

```rust
pub enum JobPriority {
    Low,
    Normal,
    High,
    ReleaseCritical,
    Emergency,
}
```

---

# 51. Priority Policy

Users cannot arbitrarily elevate themselves to Emergency without permission.

---

# 52. Priority vs Fairness

Priority should influence within allowed policy but not allow permanent starvation.

---

# 53. Queue Model

Eligible jobs enter logical scheduler queues.

Potential keys:

```text
tenant
project
capability class
priority
```

---

# 54. Queue Sharding

At scale, scheduler can shard by:

```text
platform
region
capability pool
```

while preserving global fairness policy.

---

# 55. Initial Scheduler

Start with centralized logical scheduler service.

Avoid premature distributed peer-to-peer scheduling.

---

# 56. Scheduler Loop

```text
load eligible batch
  ↓
load candidate runners
  ↓
hard filter
  ↓
score
  ↓
attempt lease transaction
  ↓
repeat
```

---

# 57. Batch Scheduling

Prefer small batches to reduce DB round trips.

---

# 58. Greedy Initial Placement

Initial implementation can use:

```text
best-score greedy placement
```

with resource updates after each choice.

---

# 59. Global Optimization

Do not start with NP-hard global bin packing.

Add smarter batching if real measurements justify.

---

# 60. Placement Decision

```rust
pub struct PlacementDecision {
    pub job: JobId,
    pub runner: RunnerId,
    pub score: PlacementScore,
    pub reasons: Vec<PlacementReason>,
}
```

---

# 61. Placement Explainability

Store/debug:

```text
matched capabilities
top candidate scores
why winner selected
why others rejected
```

---

# 62. Rejection Reasons

```rust
pub enum RunnerRejectionReason {
    Offline,
    Draining,
    MissingCapability(CapabilityId),
    InsufficientCpu,
    InsufficientMemory,
    InsufficientDisk,
    TrustMismatch,
    DeviceUnavailable,
    ResourceReserved,
}
```

---

# 63. Scheduler Explain CLI

```text
forgeyard scheduler explain-job <job>
```

Shows:

```text
requirements
eligible runner count
rejected runners/reasons
score winner
queue age
```

---

# 64. No Runner Available

Job remains:

```text
Eligible
```

unless queue timeout expires.

---

# 65. Unschedulable Detection

Distinguish:

```text
temporarily no capacity
```

from:

```text
no registered runner can ever satisfy requirement
```

---

# 66. Unschedulable State

Do not add JobState `Unschedulable` initially.

Keep Job Eligible + scheduler diagnostic.

---

# 67. Persistent Unschedulable Diagnostic

Store/report:

```text
missing capability X
```

for UI/doctor.

---

# 68. Queue Timeout

If configured, after deadline:

```text
Job -> Failed/TimedOut
```

through state service with reason:

```text
QueueTimeout
```

---

# 69. Backpressure

Scheduler must slow intake when:

```text
DB overloaded
CAS degraded
runner fleet saturated
event backlog high
```

---

# 70. Backpressure Classes

```rust
pub enum BackpressureSignal {
    StoragePressure,
    CasPressure,
    RunnerSaturation,
    QueueDepth,
    ControlPlanePressure,
}
```

---

# 71. Backpressure Action

Examples:

```text
reduce batch size
pause low priority scheduling
delay cache warming
reject new noncritical runs at API layer
```

---

# 72. Scheduler Should Not Drop Jobs

Backpressure changes admission/pace, not job correctness.

---

# 73. Resource Accounting

When lease transaction commits:

```text
runner allocated resources += job request
```

or equivalent reservation record.

---

# 74. Resource Reservation Record

```rust
pub struct ResourceReservation {
    pub lease: LeaseId,
    pub runner: RunnerId,
    pub resources: ResourceVector,
}
```

---

# 75. Atomicity

Lease + reservation must commit atomically.

---

# 76. Release Reservation

When lease ends:

```text
reservation released
```

idempotently.

---

# 77. Lost Lease Reservation

Reconciler releases after authoritative expiry/loss.

---

# 78. Double Reservation Protection

Unique lease ID.

---

# 79. Runner Capacity Reconciliation

Compare:

```text
persisted active reservations
vs
runner-reported active attempts
```

repair discrepancies.

---

# 80. Overcommit

Optional for CPU.

Default:

```text
no memory overcommit
```

unless explicit policy.

---

# 81. CPU Overcommit

May be supported for lightweight jobs.

Must be bounded and observable.

---

# 82. GPU Scheduling

GPU requirements include:

```text
vendor
model/class
VRAM
compute capability
count
exclusive/shared
```

---

# 83. GPU Sharing

Only if executor/platform supports safe partitioning.

---

# 84. Device Scheduling

Physical device jobs need:

```text
runner
+
device lease
```

---

# 85. Device Eligibility

Examples:

```text
Android API level
device model
ABI
iOS version
physical/simulator
```

---

# 86. Device Lease Coordination

Scheduler composes:

```text
runner placement
device pool allocation
```

through device subsystem.

---

# 87. Two-Resource Allocation

Potential race:

```text
runner available
device unavailable
```

Use reservation ordering or transactional coordinator.

---

# 88. Recommended Device Flow

```text
select compatible runner/device pool
  ↓
reserve device
  ↓
lease job to hosting runner
```

If second step fails:

```text
release device reservation
```

---

# 89. Signing Workers

Jobs requiring signing capability must only target restricted signing workers.

---

# 90. Trust Class

```rust
pub enum RunnerTrust {
    UntrustedGeneral,
    TrustedInternal,
    SigningRestricted,
    Confidential,
}
```

---

# 91. Trust Is Hard Requirement

A normal runner cannot score its way into signing job.

---

# 92. Network Capability

Some jobs require:

```text
internet denied
restricted egress
internal network
```

Scheduler must choose runner/executor supporting requested network policy.

---

# 93. Sandbox Capability

Example:

```text
Linux namespace sandbox
Windows job object sandbox
VM isolation
confidential VM
```

---

# 94. Platform-Specific Toolchains

Real macOS runner requirement remains hard.

Linux cross-compilation does not satisfy macOS/Xcode production requirement where Apple tooling is needed.

---

# 95. Runner Capabilities

```rust
pub struct RunnerCapabilities {
    pub platform: PlatformCapability,
    pub toolchains: ToolchainCapabilitySet,
    pub sandbox: SandboxCapabilitySet,
    pub devices: DeviceCapabilitySummary,
    pub security: SecurityCapabilitySet,
}
```

---

# 96. Capability Version

Runner capability report includes schema version/digest.

---

# 97. Capability Refresh

Runner sends on:

```text
registration
toolchain change
device change
significant resource change
```

not necessarily every heartbeat.

---

# 98. Capability Staleness

If report older than threshold:

```text
runner may become unschedulable
```

for sensitive jobs.

---

# 99. Runner Heartbeat

Scheduler consumes liveness from runner subsystem.

---

# 100. Draining

Draining runner:

```text
no new leases
existing jobs continue
```

---

# 101. Immediate Drain

Admin may request:

```text
cancel active
```

through run/job service.

---

# 102. Maintenance Windows

Runner pool can be marked unavailable ahead of maintenance.

---

# 103. Runner Pools

Logical grouping:

```rust
pub struct RunnerPoolId(Ulid);
```

---

# 104. Pool Uses

```text
platform
team ownership
trust
region
cost
autoscaling
```

---

# 105. Pool Requirement

Job may explicitly target allowed pool set only through policy-approved config.

---

# 106. Default Pool Selection

Capability-based, not hardcoded labels.

---

# 107. Labels

Runner labels can be additional hints/constraints.

Avoid using labels to replace typed core capabilities.

---

# 108. Taints/Tolerations

Optional Kubernetes-like concept for specialized runners.

Could model as policy constraints later.

---

# 109. Initial Recommendation

Use typed capabilities + optional labels first.

---

# 110. Affinity

Soft:

```text
same site as source
same runner as warm toolchain
```

---

# 111. Anti-Affinity

Soft or hard depending use:

```text
avoid previous failed host
spread test shards
```

---

# 112. Test Shard Spreading

Can spread shards across failure domains to reduce correlated loss.

---

# 113. Cost-Aware Scheduling

Optional score component:

```text
spot/cheap runner
expensive macOS
GPU cost
region egress
```

---

# 114. Cost Cannot Violate Requirements

Hard constraints remain first.

---

# 115. Latency-Aware Scheduling

Interactive jobs may prefer low queue/locality.

---

# 116. Batch Jobs

Nightly/background can favor cheaper capacity.

---

# 117. Scheduling Classes

```rust
pub enum SchedulingClass {
    Interactive,
    Normal,
    Batch,
    Release,
    Emergency,
}
```

Derived from policy/job priority.

---

# 118. Admission Control

Before becoming Eligible, project/org may enforce concurrency quotas.

Could live in scheduler/admission layer.

---

# 119. Concurrency Quotas

Examples:

```text
max 20 running jobs/project
max 100 running jobs/tenant
max 2 release-critical jobs
```

---

# 120. Admission vs Placement

Admission decides:

```text
may this job enter active scheduling?
```

Placement decides:

```text
which runner?
```

---

# 121. Recommended Initial Design

Keep job `Eligible` but scheduler fairness/quota controls placement.

Add explicit admission queues later if needed.

---

# 122. Preemption

Preemption means stopping lower-priority work to free resources.

Dangerous.

---

# 123. Default

```text
disabled
```

for general jobs.

---

# 124. Allowed Preemption

Potential for:

```text
emergency
release-critical
```

only when policy permits.

---

# 125. Preemptibility

Job declares/plan sets:

```rust
pub enum PreemptionPolicy {
    Never,
    Restartable,
    Checkpointable,
}
```

---

# 126. Restartable

Preempted job becomes:

```text
Lost/RetryWaiting
```

with infra/preemption reason.

---

# 127. Checkpointable

Future advanced capability.

Not initial requirement.

---

# 128. Preemption Cost

Score considers:

```text
elapsed runtime
artifact progress
retry cost
priority gap
```

---

# 129. Never Preempt

Examples:

```text
signing
production deploy
non-idempotent external step
```

---

# 130. Scheduler Persistence

Scheduler should persist only necessary coordination state:

```text
leases
reservations
queue metadata if needed
```

Core queue can often be derived from Job table.

---

# 131. No Hidden In-Memory Queue Authority

In-memory heap can optimize.

Persisted Job state remains truth.

---

# 132. Scheduler Cache

In-memory:

```text
eligible jobs
runner capabilities
resource snapshots
locality hints
```

rebuildable after restart.

---

# 133. Scheduler Restart

On startup:

```text
reload active runners
reload active reservations
reload eligible jobs
reconcile
```

---

# 134. Distributed Scheduler

Multiple daemon replicas may run scheduler loops.

Correctness from atomic lease transaction prevents double assignment.

---

# 135. Leader vs Leaderless Scheduling

Initial distributed design can allow:

```text
multiple scheduler workers
```

competing safely on store.

---

# 136. Why Not Require Raft Initially

Postgres transactional claim/lease is sufficient for job placement correctness.

Raft may later coordinate:

```text
scheduler epoch
global exclusive operations
```

but not required for basic scheduling.

---

# 137. Scheduler Epoch

Optional future:

```rust
pub struct SchedulerEpoch(u64);
```

Can invalidate old scheduler authority during failover.

---

# 138. Lease Authority Still Primary

Runner validates lease, not scheduler process identity.

---

# 139. Store API

Scheduler-specific atomic interface:

```rust
#[async_trait]
pub trait SchedulerStore {
    async fn list_eligible_jobs(
        &self,
        query: EligibleJobQuery,
    ) -> Result<Vec<SchedulingJob>, StoreError>;

    async fn try_lease_job(
        &self,
        command: TryLeaseJob,
    ) -> Result<LeaseAttemptResult, StoreError>;
}
```

---

# 140. `try_lease_job`

Transaction validates:

```text
job still Eligible
no termination intent
runner/session still schedulable
capacity reservation valid
```

then:

```text
create attempt
create lease
reserve resources
Job -> Leased
```

---

# 141. Lease Race

If another scheduler leased first:

```text
LeaseAttemptResult::Conflict
```

scheduler moves on.

---

# 142. Runner Capacity Race

Two schedulers targeting same runner must not over-reserve.

Use transactional resource reservation/version.

---

# 143. Runner Resource Version

```rust
pub struct RunnerResourceVersion(u64);
```

or derive from active reservation rows transactionally.

---

# 144. Placement Decision Is Advisory Until Commit

The actual authority exists only after `try_lease_job` commits.

---

# 145. Score Recalculation

If commit conflict, recalculate next candidate.

---

# 146. Scheduler Loop Pseudocode

```text
while capacity:
    jobs = eligible_batch()

    for job in fair_order(jobs):
        candidates = runners()
        eligible = hard_filter(job, candidates)

        if eligible empty:
            record diagnostic
            continue

        ranked = score(job, eligible)

        for runner in ranked:
            if try_lease(job, runner) succeeds:
                dispatch lease
                break
```

---

# 147. Dispatch Failure After Lease Commit

Lease exists but runner didn't receive message.

Reconciliation/expiry handles.

Can retry dispatch while lease valid.

---

# 148. At-Least-Once Lease Delivery

Runner may receive duplicate lease message.

Agent deduplicates by `LeaseId`.

---

# 149. Lease Acceptance Timeout

If no acknowledgment quickly:

```text
revoke/expire
```

according to policy.

---

# 150. Lease Revocation

Before runner starts:

```text
Leased -> Eligible
```

can be safe.

After running:

```text
use cancellation/loss semantics
```

---

# 151. Resource Release on Rejection

Atomic with lease rejection state transition.

---

# 152. Local Standalone Scheduler

Same algorithms with:

```text
one local runner
```

Hard matching still matters.

---

# 153. Standalone Unschedulable

If project requests:

```text
macOS
```

on Linux standalone:

show:

```text
no eligible runner
```

rather than silently changing target.

---

# 154. Local Auto-Runner

Standalone bootstrap registers local runner capabilities automatically.

---

# 155. Distributed Agent Registration

Runner subsystem supplies capability snapshot.

---

# 156. Remote Runners

Scheduler does not assume shared filesystem.

Inputs always through CAS/source materialization.

---

# 157. Site Awareness

```rust
pub struct SiteId(Ulid);
```

Examples:

```text
home-lab
office
region-a
```

---

# 158. Region Awareness

```rust
pub struct RegionId(BoundedString);
```

---

# 159. Network Cost

Site/region matrix can inform score.

---

# 160. Egress Avoidance

Prefer same region as large input/output durable CAS where sensible.

---

# 161. Data Size Weight

Locality benefit should consider total input bytes.

---

# 162. Input Closure Summary

Planner/data plane can provide:

```text
required bytes
known cached bytes
```

---

# 163. Locality Score Example

```text
100% local -> high score
80% local -> medium-high
0% local -> neutral
```

Not negative enough to violate fairness indefinitely.

---

# 164. Toolchain Warmth

Toolchain present can be significant because installation may be expensive.

---

# 165. Cache Hit Prediction

Predictive model can be used as a score hint.

Never correctness authority.

---

# 166. Scheduler Scoring Version

```rust
pub struct SchedulerPolicyVersion(Digest);
```

Record for explainability.

---

# 167. Deterministic Scoring

Given same snapshot of inputs/config:

```text
same score ordering
```

except explicit random tie-break if configured.

---

# 168. Tie Breaking

Recommended:

```text
stable hash(job_id, runner_id)
```

for deterministic spread.

---

# 169. Avoid Runner Hotspot

Stable tie-break plus load/headroom prevents same runner always winning.

---

# 170. Fairness Queue Data

Potential:

```rust
pub struct FairnessState {
    pub tenant_usage: ResourceUsage,
    pub project_usage: ResourceUsage,
}
```

---

# 171. CPU-Time vs Concurrent Slots

Initial fairness can use:

```text
running job count
weighted resource units
```

---

# 172. Dominant Resource Fairness

Future option if heterogeneous resources create fairness issues.

---

# 173. Scheduler Quotas

```rust
pub struct SchedulerQuota {
    pub max_running_jobs: Option<u32>,
    pub max_cpu: Option<CpuUnits>,
    pub max_memory: Option<ByteSize>,
}
```

---

# 174. Quota Scope

```text
tenant
project
runner pool
```

---

# 175. Quota Reconciliation

Derived from active reservations; repair if mismatch.

---

# 176. Burst

Policy may allow temporary burst above baseline.

---

# 177. Emergency Priority

Must be audited.

---

# 178. Starvation Guard

Aging ensures lower-priority jobs eventually receive opportunity unless hard capacity never exists.

---

# 179. Dedicated Runner Pools

Project may reserve runners.

Scheduler honors ownership constraints.

---

# 180. Shared Pool

Default distributed pool.

---

# 181. Reserved Capacity

Some runner capacity reserved for:

```text
release jobs
interactive jobs
```

---

# 182. Capacity Class

```rust
pub enum CapacityClass {
    Shared,
    Reserved(ReservationGroupId),
}
```

---

# 183. Spillover

Policy can allow reserved pool idle capacity to serve shared jobs.

---

# 184. Spot/Preemptible Runners

Mark runner capacity as:

```text
preemptible infrastructure
```

Scheduler prefers for restartable batch jobs.

---

# 185. Spot Avoidance

Do not place signing/release-critical long job on volatile runner unless policy allows.

---

# 186. Reliability Class

```rust
pub enum RunnerReliabilityClass {
    Stable,
    Standard,
    Preemptible,
}
```

---

# 187. Scheduler Policy Input

```rust
pub struct SchedulingPolicy {
    pub score_weights: ScoreWeights,
    pub quotas: QuotaPolicy,
    pub preemption: PreemptionConfig,
    pub locality: LocalityPolicy,
}
```

---

# 188. Config vs Policy

Admin config sets operational defaults.

Policy engine can impose security/business constraints.

---

# 189. Policy Example

```text
production deploy:
  only trusted pool
  preemption forbidden
  region = primary
```

---

# 190. Scheduler Does Not Evaluate User Authorization

Authz already happened upstream.

It applies scheduling policy to already-authorized job.

---

# 191. Capability Requirement Provenance

For explainability, each requirement records source:

```rust
pub enum CapabilityRequirementOrigin {
    Pipeline,
    Ecosystem,
    Platform,
    Policy,
    Device,
    Security,
}
```

---

# 192. Explain Missing Capability

Example:

```text
requires Xcode 18
origin: Swift iOS build plan
```

---

# 193. Scheduler Diagnostics

```rust
pub struct SchedulingDiagnostic {
    pub job: JobId,
    pub code: SchedulingDiagnosticCode,
    pub details: Vec<SchedulingDetail>,
}
```

---

# 194. Diagnostic Codes

Examples:

```text
NO_RUNNERS
NO_MATCHING_PLATFORM
INSUFFICIENT_MEMORY
MISSING_TOOLCHAIN
NO_DEVICE
TRUST_MISMATCH
QUOTA_BLOCKED
```

---

# 195. Diagnostic Persistence

Store latest summary, not every scheduling loop iteration.

---

# 196. Diagnostic Debounce

Avoid event spam for persistent unschedulable job.

---

# 197. Scheduler Doctor

```text
forgeyard scheduler doctor
```

Checks:

```text
runner pools
missing capability classes
quota config
stuck eligible jobs
lease backlog
resource accounting
```

---

# 198. CLI

```text
forgeyard scheduler status
forgeyard scheduler explain-job
forgeyard scheduler queues
forgeyard scheduler runners
forgeyard scheduler capacity
forgeyard scheduler rebalance
```

---

# 199. `scheduler status`

Shows:

```text
eligible jobs
running jobs
queue age
active leases
runner capacity
backpressure state
```

---

# 200. `scheduler queues`

Grouped by:

```text
priority
tenant
platform
pool
```

---

# 201. `scheduler capacity`

Shows:

```text
CPU
memory
disk
GPU
device
platform pools
```

---

# 202. `rebalance`

Should not move running jobs by default.

May only reconsider queued/eligible jobs.

---

# 203. Dioxus UI

Scheduler page:

```text
Overview
Queues
Runner Pools
Capacity
Unschedulable
Leases
Fairness
```

---

# 204. Queue Visualization

Show:

```text
waiting count
oldest wait
priority mix
```

---

# 205. Runner Pool View

Show:

```text
online/draining/offline
capacity
allocatable
active jobs
toolchains
trust
region/site
```

---

# 206. Unschedulable View

Human explanation:

```text
7 jobs require macOS/Xcode 18
0 compatible runners online
```

---

# 207. API

Potential admin/read APIs:

```text
GET /v1/scheduler/status
GET /v1/scheduler/queues
GET /v1/scheduler/capacity
GET /v1/jobs/{id}/scheduling
```

---

# 208. Internal Scheduler Control

Not broadly public:

```text
drain runner
set pool state
```

through runner/admin APIs.

---

# 209. Metrics

```text
scheduler_cycle_latency
scheduler_jobs_considered
scheduler_placements
scheduler_placement_conflicts
scheduler_unschedulable
scheduler_queue_wait
scheduler_fairness_delay
scheduler_backpressure
scheduler_preemptions
scheduler_lease_dispatch_failures
```

---

# 210. Resource Metrics

```text
runner_cpu_allocated
runner_memory_allocated
runner_disk_allocated
gpu_allocated
device_allocated
```

---

# 211. Locality Metrics

```text
placement_locality_score
cas_local_hit_after_placement
toolchain_warm_hit
```

---

# 212. Reliability Metrics

```text
runner_lost_attempt_rate
runner_rejection_rate
```

---

# 213. Tracing

Spans:

```text
scheduler.cycle
scheduler.eligibility
scheduler.score
scheduler.place
scheduler.lease
scheduler.dispatch
scheduler.reconcile
```

---

# 214. High-Cardinality Caution

Do not put RunnerId/JobId into metrics labels.

Use tracing/logs for IDs.

---

# 215. Alerting

Alert on:

```text
eligible queue growing
oldest wait above SLO
lease conflict spike
resource accounting mismatch
no runners for critical capability
scheduler loop stalled
```

---

# 216. Scheduler Reconciler

```text
forgeyard-scheduler-reconcile
```

Checks:

```text
active reservations without active lease
lease without reservation
runner over-allocation
eligible job never reconsidered
draining runner receiving new lease
```

---

# 217. Reservation Repair

Recompute from authoritative active leases if needed.

---

# 218. Queue Recovery

Eligible jobs are reloaded after restart.

No lost in-memory queue.

---

# 219. Scheduler Crash During Placement

If lease transaction not committed:

```text
nothing authoritative happened
```

If committed but dispatch not sent:

```text
re-dispatch/reconcile
```

---

# 220. DB Failure

Scheduler pauses new placements.

Existing jobs continue under valid leases.

---

# 221. CAS Degradation

If durable CAS unavailable for required job inputs:

scheduler may pause placement depending data-plane health/policy.

---

# 222. Runner Registration Race

New runner becomes eligible after registration transaction + health state.

---

# 223. Runner Disconnect

No new jobs.

Existing leases handled by run/lease expiration logic.

---

# 224. Scheduler and Run State Boundary

Scheduler may request:

```text
TryLeaseJob
```

It may not directly set:

```text
JobState::Leased
```

outside state/store API.

---

# 225. Scheduler and Runner Boundary

Scheduler dispatches lease payload.

Runner acceptance/rejection goes through run/job state service.

---

# 226. Scheduler and Device Boundary

Device subsystem provides reservable device capability.

Scheduler orchestrates combined placement.

---

# 227. Scheduler and Autoscaling

Future autoscaler observes:

```text
queue demand
capability shortages
```

and scales runner pools.

---

# 228. Autoscaler Is Separate

Suggested future crates:

```text
crates/autoscale/
├── forgeyard-autoscale/
├── forgeyard-autoscale-demand/
├── forgeyard-autoscale-kubernetes/
└── forgeyard-autoscale-cloud/
```

---

# 229. Scheduler Demand Signal

```rust
pub struct CapacityDemand {
    pub capability_class: CapabilityClass,
    pub queued_jobs: u32,
    pub requested_resources: ResourceVector,
}
```

---

# 230. KEDA/Kubernetes

Optional adapter consumes demand.

Scheduler itself remains infrastructure-neutral.

---

# 231. Scale-to-Zero

For expensive pools, autoscaler may scale to zero.

Scheduler reports jobs waiting for that capability.

---

# 232. Startup Latency

Queue timeout must account for autoscaling cold-start policy if configured.

---

# 233. Scheduler API Trait

```rust
#[async_trait]
pub trait Scheduler {
    async fn schedule_once(
        &self,
        context: SchedulingContext,
    ) -> Result<SchedulingCycleResult, SchedulerError>;
}
```

---

# 234. Eligibility Trait

```rust
pub trait EligibilityEngine {
    fn evaluate(
        &self,
        job: &SchedulingJob,
        runner: &SchedulingRunner,
    ) -> EligibilityResult;
}
```

---

# 235. Scoring Trait

```rust
pub trait PlacementScorer {
    fn score(
        &self,
        job: &SchedulingJob,
        runner: &SchedulingRunner,
    ) -> PlacementScore;
}
```

---

# 236. Pluggable Scoring

Internal strategy can evolve behind stable trait.

Do not expose arbitrary third-party scheduler code initially.

---

# 237. Deterministic Test Scorer

Testkit can use fixed scorer.

---

# 238. Scheduler Error Model

```rust
pub enum SchedulerError {
    StoreUnavailable,
    InvalidJobRequirements,
    ResourceAccountingCorrupt,
    PlacementConflict,
    PolicyError,
    Internal,
}
```

---

# 239. Placement Conflict Is Normal

Not exceptional/fatal in multi-scheduler environment.

---

# 240. Retry Classification

```text
store unavailable -> backoff
placement conflict -> immediate next candidate
invalid requirements -> no retry until plan fixed
```

---

# 241. Testkit

```text
forgeyard-scheduler-testkit/src/
├── lib.rs
├── job.rs
├── runner.rs
├── capabilities.rs
├── resources.rs
├── scorer.rs
├── fairness.rs
└── assertions.rs
```

---

# 242. Unit Tests

Test:

```text
hard capability matching
resource fit
score ordering
priority
fairness
aging
retry anti-affinity
drain behavior
```

---

# 243. Property Tests

Properties:

```text
ineligible runner never selected
selected runner always satisfies hard requirements
resource reservation never exceeds capacity
aging prevents starvation under bounded assumptions
```

---

# 244. Concurrency Tests

Simulate:

```text
two schedulers
same job
same runner
many jobs
```

prove one lease per job and no over-reservation.

---

# 245. Failure Injection

Inject:

```text
DB timeout
runner disconnect
dispatch failure
stale capability report
CAS locality mismatch
```

---

# 246. Large Fleet Simulation

Simulate:

```text
10k runners
100k eligible jobs
```

for algorithmic profiling.

---

# 247. Scheduler Complexity

Initial hard filtering should avoid:

```text
jobs × every runner
```

at very large scale.

---

# 248. Capability Index

Maintain runner indexes by:

```text
OS
architecture
pool
trust
toolchain class
device class
```

---

# 249. Candidate Narrowing

Use indexes to produce candidate subset before detailed matching.

---

# 250. Exact Match Still Required

Index is optimization.

Matcher verifies.

---

# 251. Resource Index

Keep available-capacity buckets or sorted structures in memory.

Rebuildable.

---

# 252. Scheduler Snapshot

Each cycle uses coherent-enough snapshot of:

```text
runner state
reservations
eligible jobs
```

Final lease transaction resolves races.

---

# 253. Stale In-Memory View

Safe because commit revalidates.

---

# 254. Queue Ordering

Suggested base key:

```text
effective_priority
aging
created_at
job_id
```

---

# 255. Effective Priority

```text
base priority
+
aging
+
policy adjustments
```

bounded.

---

# 256. Starvation Bound

Document/test expected fairness characteristics.

---

# 257. Large Job Avoidance

Small jobs should not indefinitely starve large resource jobs.

Reserve/aging can help.

---

# 258. Gang Scheduling

Jobs needing multiple runners simultaneously are not initially required.

---

# 259. Future Distributed Tests

If MPI/multi-node workloads added, separate gang scheduling subsystem.

---

# 260. Service Affinity

If one job depends on another runtime service on same host, explicit co-location requirement.

Avoid implicit shared-host assumptions.

---

# 261. Build Shards

Independent shards scheduled independently.

---

# 262. Matrix Jobs

Ordinary jobs after pipeline expansion.

Scheduler sees only capabilities/resources.

---

# 263. Reproducibility Jobs

May require:

```text
different runner/site
```

to strengthen independent reproduction.

This is a hard/soft anti-affinity from reproducibility policy.

---

# 264. Multi-Party Reproduction

Require distinct trust/failure domains if configured.

---

# 265. Security-Sensitive Jobs

Examples:

```text
signing
secret-heavy integration
production deployment
```

must run only in allowed trust pools.

---

# 266. Untrusted Proposal Jobs

Should prefer/general-purpose untrusted pools and never privileged workers.

---

# 267. Runner Pool Isolation

Pool policy rejects incompatible trust class.

---

# 268. Resource Estimate Feedback

Actual runtime metrics can improve future requested recommendations.

But scheduler never silently lowers declared limits below policy.

---

# 269. Auto-Tuning

Optional:

```text
suggest memory request
suggest CPU
```

not mutate build semantics automatically at first.

---

# 270. Queue Prediction

Estimate wait time.

Informational only.

---

# 271. ETA

Use historical stats but label as estimate.

---

# 272. Scheduler History

Store summarized placement history if useful for analytics.

Do not store every score for every candidate indefinitely.

---

# 273. Placement Audit

For security-critical jobs, record:

```text
selected runner
trust
policy
capability digest
```

---

# 274. Release Audit

Release/signing placement should be auditable.

---

# 275. Runner Capability Digest

Attempt record can reference exact capability snapshot digest used for placement.

---

# 276. Runner Software Version

Hard compatibility requirement between daemon/agent.

Transport/version layer handles.

---

# 277. Mixed Agent Versions

Scheduler excludes incompatible agents.

---

# 278. Runner Health

Runner subsystem reports:

```text
Healthy
Degraded
Unhealthy
```

---

# 279. Degraded Runner

May receive only low-risk jobs or none depending policy.

---

# 280. Temperature/Hardware Health

Optional specialized capability/health inputs.

---

# 281. Disk Pressure

Runner enters:

```text
Saturated
```

or reduces allocatable disk.

---

# 282. Memory Pressure

Same.

---

# 283. Local CAS Pressure

Can reduce locality benefit or block large jobs if no staging capacity.

---

# 284. Scheduler Backoff

If no placement possible:

do not spin aggressively.

Use event-driven wakeups + bounded polling/reconcile fallback.

---

# 285. Wakeup Sources

```text
job becomes Eligible
runner registers
runner frees resources
retry deadline expires
device becomes available
```

---

# 286. Event-Driven Scheduling

Preferred for responsiveness.

---

# 287. Polling Fallback

Periodic scan ensures missed events do not stall jobs.

---

# 288. Scheduling Trigger Coalescing

Many events can collapse into one cycle.

---

# 289. Scheduler Work Queue

Internal in-memory wake queue.

Not durable authority.

---

# 290. Lease Dispatch Queue

Could use outbox/event mechanism.

---

# 291. Dispatch Outbox

Transaction:

```text
lease commit
+
dispatch event
```

then transport sends.

Improves reliability.

---

# 292. Duplicate Dispatch

Runner deduplicates by LeaseId.

---

# 293. Lease Acknowledgement

Runner sends:

```text
accept/reject
```

---

# 294. No Ack

Lease expiry/revoke.

---

# 295. Lease TTL

Depends on:

```text
dispatch latency
network conditions
job type
```

Separate acceptance TTL and execution-renewal TTL can be useful.

---

# 296. Lease Phases

Potential:

```text
OfferLease
ActiveLease
```

Initial implementation can use one lease + ack deadline.

---

# 297. Offer Timeout

If not accepted quickly:

```text
return Job Eligible
```

---

# 298. Active Renewal

After accept, periodic heartbeat renews.

---

# 299. Scheduler Does Not Own Lease Renewal

Lease/run service owns authoritative renewal.

Scheduler may observe.

---

# 300. Scheduler Rebalance

Do not revoke healthy running leases just because a better runner appears.

---

# 301. Queue Reordering

Eligible jobs can be reprioritized by policy/admin.

---

# 302. Priority Change Audit

Required.

---

# 303. Scheduler Security Invariants

1. only authenticated registered runners considered;
2. trust requirement is hard;
3. device/signing capabilities cannot be spoofed without registration validation;
4. scheduler never bypasses policy to improve throughput;
5. tenant quota enforced;
6. runner labels cannot grant privileged trust.

---

# 304. Capability Attestation

High-trust runners may require stronger enrollment/cert identity.

Scheduler consumes trust result, not attestation implementation.

---

# 305. Agent Self-Reported Capability Trust

General capabilities can be self-reported + probed.

Sensitive capabilities should be provisioned/verified.

---

# 306. Toolchain Verification

Runner reports immutable toolchain identity.

If mismatch at runtime, attempt fails preparation.

---

# 307. Scheduler Correctness Invariants

1. only `Eligible` jobs are placed;
2. selected runner satisfies all hard requirements;
3. resource reservation never exceeds allowed capacity;
4. one authoritative lease per current attempt;
5. placement is not authoritative until transaction commit;
6. soft score never overrides hard rejection;
7. locality is advisory;
8. predictive models are advisory;
9. in-memory state is rebuildable;
10. multi-daemon races resolve through store.

---

# 308. Standalone Invariants

1. same eligibility/score logic;
2. one local runner can still be rejected;
3. no silent target substitution;
4. local mode may optimize transport but not semantics.

---

# 309. Distributed Invariants

1. agents never write scheduler DB directly;
2. scheduler can restart without losing jobs;
3. lease dispatch is at-least-once safe;
4. runner churn handled through liveness/lease expiry;
5. no Raft required for basic placement correctness.

---

# 310. Implementation Phase 1 — Models

Implement:

```text
SchedulingJob
SchedulingRunner
ResourceVector
PlacementScore
RunnerRejectionReason
```

---

# 311. Phase 2 — Hard Matcher

Implement:

```text
platform
architecture
resources
toolchain
trust
sandbox
```

---

# 312. Phase 3 — Simple Scorer

Implement:

```text
headroom
load
locality
stable tie-break
```

---

# 313. Phase 4 — Lease Integration

Implement `try_lease_job` with atomic attempt/resource reservation.

---

# 314. Phase 5 — Runner Pools / Drain

Implement pool grouping and drain states.

---

# 315. Phase 6 — Fairness / Priority

Add weighted queues + aging.

---

# 316. Phase 7 — Retry-Aware Placement

Add anti-affinity/failure-domain behavior.

---

# 317. Phase 8 — Device/GPU

Add specialized resource scheduling.

---

# 318. Phase 9 — Backpressure

Integrate control-plane/CAS/runner pressure signals.

---

# 319. Phase 10 — Reconciliation / HA

Multi-scheduler contention tests, rebuildable caches, reservation repair.

---

# 320. Phase 11 — Advanced Optimization

Add:

```text
scarcity
cost
predictive cache
autoscaling demand
```

only after correctness stable.

---

# 321. Acceptance Tests

1. Ineligible runner is never selected.
2. Missing toolchain rejects runner.
3. Wrong OS rejects runner.
4. Insufficient memory rejects runner.
5. Trust mismatch rejects runner.
6. Eligible runner can be scored.
7. Highest score wins absent fairness constraints.
8. Hard requirement always dominates soft score.
9. Resource reservation is atomic with lease.
10. Two schedulers cannot over-reserve same runner.
11. Two schedulers cannot lease same job twice.
12. Draining runner receives no new job.
13. Offline runner receives no new job.
14. Retry after runner loss prefers different runner when possible.
15. Locality improves score but missing local object never breaks execution.
16. Stale locality hint only causes slower fetch, not incorrect placement.
17. Fairness prevents one tenant from monopolizing bounded shared capacity.
18. Aging prevents indefinite starvation.
19. Priority elevation requires policy/permission.
20. Queue timeout is surfaced explicitly.
21. Unschedulable job remains Eligible with diagnostic.
22. Standalone Linux refuses macOS-only job.
23. Real Apple capability required for Apple-specific job.
24. Device job acquires compatible device+runner.
25. Signing job cannot run on general runner.
26. Scheduler restart rebuilds queue from persisted state.
27. Lease committed but dispatch lost is safely retried.
28. Duplicate lease dispatch is safe.
29. DB outage pauses new placements without corrupting active leases.
30. Reconciler repairs leaked resource reservation.
31. Predictive cache score cannot bypass hard requirements.
32. Same scheduler logic works with Stoolap and Postgres.
33. Scheduler explain output identifies rejection reasons.
34. High queue load does not require full runner scan per job after indexing.
35. Forgeyard self-hosting jobs can be placed through this scheduler.

---

# 322. Production Readiness Gates

Do not call scheduler production-ready until:

```text
hard matcher correct
resource reservation atomic
multi-scheduler race tests pass
runner drain works
fairness/priority behavior tested
lease dispatch recovery works
retry-aware placement works
reconciliation works
unschedulable diagnostics exist
metrics/doctor exist
```

Advanced preemption/autoscaling/predictive scoring may mature later.

---

# 323. Architectural Invariants

1. Scheduler consumes only Eligible jobs.
2. Scheduler never directly assigns Job state.
3. Attempt+Lease creation is authoritative store operation.
4. Hard filtering precedes scoring.
5. Every required capability must match.
6. Soft scoring never relaxes correctness.
7. Resource capacity is reserved transactionally.
8. Placement decision is advisory until commit.
9. In-memory scheduler state is rebuildable.
10. Locality/cache data is advisory.
11. Predictive models are advisory.
12. Runner trust is hard requirement.
13. Draining runners receive no new work.
14. Retry placement may avoid prior failure domains.
15. Fairness and priority are explicit policy.
16. Preemption is disabled by default.
17. Signing/device/confidential jobs use restricted capability paths.
18. Standalone and distributed modes share scheduler semantics.
19. Multiple schedulers may contend safely.
20. Scheduler does not require Raft for basic correctness.
21. Agent never directly edits scheduler metadata.
22. Resource leaks are reconciled.
23. Dispatch is at-least-once safe.
24. Queue persistence derives from Job state, not hidden heap authority.
25. Forgeyard itself should dogfood the scheduler.

---

# 324. Final Target Architecture

```text
                  Eligible Job Set
                         │
                         ▼
                  Fair Queue Order
                         │
                         ▼
               Candidate Runner Index
                         │
                         ▼
                 Hard Eligibility
               ┌─────────┼─────────┐
               ▼         ▼         ▼
            Platform   Resources   Trust
               │         │         │
               └─────────┼─────────┘
                         ▼
                   Eligible Set
                         │
                         ▼
                    Soft Score
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Locality       Headroom       Fairness
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                  Best Placement
                         │
                         ▼
                  try_lease_job()
                         │
        ┌────────────────┴────────────────┐
        ▼                                 ▼
     Conflict                         Commit
        │                                 │
        └── retry candidate               ▼
                                  Attempt + Lease
                                           │
                                           ▼
                                         Runner
```

---

# 325. Final Architectural Position

Scheduling correctness:

```text
Eligible Job
+
Hard Requirements
+
Current Runner Capabilities
+
Current Allocatable Resources
        ↓
Eligible Runner Set
```

Optimization:

```text
Eligible Runner Set
+
Locality
+
Cache warmth
+
Headroom
+
Reliability
+
Fairness
+
Cost
        ↓
Ranked Candidates
```

Authority:

```text
Ranked Candidate
  ↓
atomic attempt + lease + reservation transaction
  ↓
only then placement becomes real
```

The key guarantee is:

> **Forgeyard can optimize aggressively for locality, speed, fairness, cache warmth, and cost without ever letting those optimizations weaken the hard correctness, trust, platform, or resource requirements of the job.**

---

# 326. New-Repository Sequence

The sequence is now:

```text
01 Core Domain & Foundation
02 Storage & Metadata
03 CAS & Artifact Data Plane
04 Pipeline IR / Parsing / Planning
05 Run / Job State Machine
06 Scheduler
07 Runner / Agent
08 Sandbox & Executor
09 Transport / QUIC / Protocol
10 Events / Reconciliation
11 Policy / Authorization / Identity
12 Secrets & Trust
13 Supply Chain / SBOM / Provenance / Signing
14 Packaging
15 Release
16 Deployment
17 Observability / Health / Doctor
18 API / Axum
19 Dioxus UI
20 Device Lab
21 SCM Provider Integrations
22 HA / Coordination / Raft
23 RBE Interop
24 Plugin / Extension Architecture
25 Operations / Backup / Upgrade / DR
26 Self-Hosting / Bootstrap / Release of Forgeyard
```
