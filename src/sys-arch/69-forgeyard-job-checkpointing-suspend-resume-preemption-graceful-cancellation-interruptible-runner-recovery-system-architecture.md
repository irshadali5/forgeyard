# 69 — Forgeyard Job Checkpointing, Suspend/Resume, Preemption, Graceful Cancellation & Spot/Interruptible Runner Recovery System Architecture

**Document type:** Core Job Checkpointing, Suspend/Resume, Preemption, Graceful Cancellation, Runner-Loss Recovery & Interruptible Execution System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** resumable jobs, checkpoint artifacts, execution epochs, suspend/resume, spot/preemptible runners, runner loss, graceful cancellation, signal delivery, cancellation grace periods, stage-level restartability, partial-output handling, external-effect boundaries, checkpoint validation, resume placement, cache/checkpoint separation, recovery policy, and interrupted long-running workloads  
**Architecture style:** Explicit execution epochs, immutable checkpoints, deterministic resume contracts, cancellation as a state machine, resumability declared by the workload, external-effect fencing, safe restart from known points, no partial-process resurrection, and no assumption that runner loss is equivalent to job failure  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Run/Job State Machine, Scheduler, Runner/Agent, Sandbox/Executor, CAS, Events/Reconciliation, Concurrency/Fencing, Runner Fleet Autoscaling, Cost/FinOps, Failure Diagnosis, Reliability, Historical Replay, Test Environments, and Progressive Delivery. This subsystem makes interruption and recovery semantics first-class for long-running and expensive workloads.

---

## 1. Purpose

Forgeyard jobs may be interrupted for many reasons:

```text
spot/preemptible VM termination
runner host maintenance
autoscaler scale-down
runner crash
network partition
manual cancellation
superseding commit
resource pressure
fleet rebalancing
security quarantine
region failover
```

A simplistic CI system treats every interruption as:

```text
job failed
  ↓
restart from beginning
```

That is wasteful or unsafe for:

```text
large builds
ML/AI compilation
large test suites
benchmarks
long data-generation tasks
device tests
multi-hour packaging
large integration environments
expensive GPU workloads
```

The opposite mistake is worse:

```text
process stopped somewhere
  ↓
assume it can continue later
```

That can corrupt state.

The central rule is:

> **Forgeyard resumes only from explicit, immutable, verified checkpoints created under a declared resumability contract. A partially executed process is never treated as a valid checkpoint merely because its filesystem still exists.**

A second rule is:

> **Cancellation, suspension, preemption, runner loss, and retry are different states with different semantics.**

A third rule is:

> **A checkpoint may preserve computation state, but it never proves external side effects are safe to repeat. External effects remain governed by idempotency, fencing, and reconciliation.**

---

## 2. Architectural Position

```text
                       JobAttempt
                           │
                           ▼
                      ExecutionEpoch
                           │
                 ┌─────────┼─────────┐
                 ▼         ▼         ▼
              Running   Checkpoint  Interrupted
                            │
                            ▼
                     CheckpointArtifact
                            │
                            ▼
                        New Runner
                            │
                            ▼
                     Resume Validation
                            │
                            ▼
                       New Epoch
```

Cancellation:

```text
CancelRequested
      ↓
GracefulStop
      ↓
checkpoint if allowed
      ↓
terminate
      ↓
Cancelled / Suspended / Retryable
```

---

## 3. Goals

The subsystem MUST:

1. define checkpoint identity;
2. define resumability contract identity;
3. define execution epoch;
4. support graceful cancellation;
5. support forced cancellation;
6. support suspend/resume;
7. support spot/preemptible runners;
8. support runner-loss recovery;
9. support checkpoint creation;
10. support checkpoint validation;
11. support checkpoint retention;
12. support checkpoint encryption where required;
13. support cross-runner resume;
14. support same-platform resume requirements;
15. support hardware-bound checkpoints;
16. support non-resumable jobs;
17. support partial-stage restart;
18. support workflow supersession;
19. support external-effect-safe boundaries;
20. support checkpoint-aware scheduling;
21. support cost-aware recovery;
22. support retry/restart policies;
23. support UI/API/CLI;
24. support audit;
25. support observability;
26. support HA;
27. support federation;
28. support DR;
29. preserve deterministic job semantics;
30. never resume from unverified partial state.

---

## 4. Non-Goals

This subsystem does not:

```text
checkpoint arbitrary OS processes magically
replace application-specific checkpoint support
replace distributed database transactions
make every job resumable
replace CAS
replace normal retry logic
replace idempotency/reconciliation
```

---

## 5. Workspace Structure

```text
crates/checkpoint/
├── forgeyard-checkpoint/
├── forgeyard-checkpoint-model/
├── forgeyard-checkpoint-contract/
├── forgeyard-checkpoint-create/
├── forgeyard-checkpoint-restore/
├── forgeyard-checkpoint-validate/
├── forgeyard-checkpoint-cancel/
├── forgeyard-checkpoint-preempt/
├── forgeyard-checkpoint-reconcile/
├── forgeyard-checkpoint-health/
└── forgeyard-checkpoint-testkit/
```

Adapters:

```text
crates/checkpoint-adapters/
├── forgeyard-checkpoint-process/
├── forgeyard-checkpoint-container/
├── forgeyard-checkpoint-vm/
├── forgeyard-checkpoint-build-tool/
├── forgeyard-checkpoint-test-framework/
├── forgeyard-checkpoint-device/
└── forgeyard-checkpoint-custom/
```

---

## 6. CheckpointId

```rust
pub struct CheckpointId(Digest);
```

Immutable identity of a valid checkpoint artifact plus resume metadata.

---

## 7. CheckpointManifestId

```rust
pub struct CheckpointManifestId(Digest);
```

---

## 8. ExecutionEpoch

```rust
pub struct ExecutionEpoch(u64);
```

Every resumed execution starts a new epoch.

---

## 9. Why Epochs

Scenario:

```text
runner A executes epoch 4
A becomes unreachable
runner B resumes checkpoint -> epoch 5
A returns
```

Epoch 4 must not publish outputs/effects as current.

---

## 10. Fencing

Job attempt output/effect commit includes expected current execution epoch.

---

## 11. Stale Epoch

Rejected.

---

## 12. ResumabilityContractId

```rust
pub struct ResumabilityContractId(Digest);
```

---

## 13. Resumability Contract

```rust
pub struct ResumabilityContract {
    pub id: ResumabilityContractId,
    pub mode: ResumabilityMode,
    pub checkpoint_policy: CheckpointPolicy,
    pub compatibility: ResumeCompatibility,
}
```

---

## 14. ResumabilityMode

```rust
pub enum ResumabilityMode {
    None,
    RestartStage,
    ApplicationCheckpoint,
    FrameworkCheckpoint,
    FilesystemSnapshot,
    VmSnapshot,
    Custom(ResumabilityAdapterId),
}
```

---

## 15. None

Restart from beginning/stage.

---

## 16. RestartStage

Pipeline stage outputs are committed; interrupted current stage restarts.

---

## 17. ApplicationCheckpoint

Application/tool explicitly writes checkpoint.

---

## 18. FrameworkCheckpoint

Test/build framework exposes resume cursor/shard state.

---

## 19. FilesystemSnapshot

Only valid if workload semantics declare filesystem snapshot sufficient.

---

## 20. VM Snapshot

Rare/high-cost and trust-sensitive.

---

## 21. Critical Rule

Filesystem/VM state is not automatically semantically resumable.

---

## 22. CheckpointPolicy

```rust
pub struct CheckpointPolicy {
    pub interval: Option<Duration>,
    pub on_preemption_notice: bool,
    pub on_cancel: bool,
    pub max_checkpoints: u16,
}
```

---

## 23. Interval

Best-effort.

---

## 24. Checkpoint Too Frequent

Cost/IO overhead.

---

## 25. Checkpoint Too Rare

Lost work.

---

## 26. Adaptive Interval

Can depend on:

```text
job cost
runtime
checkpoint size
preemption risk
```

---

## 27. Checkpoint Manifest

```rust
pub struct CheckpointManifest {
    pub job_attempt: JobAttemptId,
    pub epoch: ExecutionEpoch,
    pub source: SourceSnapshotId,
    pub pipeline_job: JobIrId,
    pub toolchains: Vec<ToolchainDescriptorId>,
    pub runner_baseline: RunnerBaselineId,
    pub payload: CasObjectRef,
    pub resume_contract: ResumabilityContractId,
}
```

---

## 28. Exact Inputs

Checkpoint binds execution inputs.

---

## 29. Source Change

Invalidates checkpoint.

---

## 30. Toolchain Change

Invalidates unless contract explicitly supports.

---

## 31. Job Spec Change

Invalidates.

---

## 32. Environment Change

Compatibility-dependent.

---

## 33. ResumeCompatibility

```rust
pub struct ResumeCompatibility {
    pub architecture: ResumeRequirement<Architecture>,
    pub operating_system: ResumeRequirement<OperatingSystem>,
    pub runner_baseline: ResumeBaselineRequirement,
    pub hardware: HardwareResumeRequirement,
}
```

---

## 34. ResumeRequirement

```rust
pub enum ResumeRequirement<T> {
    Exact(T),
    Compatible(CompatibilityClassId),
    Any,
}
```

---

## 35. HardwareResumeRequirement

```rust
pub enum HardwareResumeRequirement {
    Any,
    CpuFeatures(CpuFeatureSet),
    GpuExact(GpuIdentity),
    GpuCompatible(GpuCompatibilityClassId),
    DeviceExact(DeviceId),
}
```

---

## 36. GPU Checkpoint

May require exact GPU architecture/driver/runtime compatibility.

---

## 37. Device Test

May require exact physical device.

---

## 38. Cross-Runner Resume

Allowed only if compatibility passes.

---

## 39. Checkpoint Payload

Stored in CAS.

---

## 40. Checkpoint Metadata

Stored in metadata DB.

---

## 41. Checkpoint vs Cache

Different semantics.

Cache:

```text
reusable derived output by key
```

Checkpoint:

```text
partial progress state for a specific execution contract
```

---

## 42. Never Treat Checkpoint As Generic Cache

Critical.

---

## 43. Checkpoint Scope

```rust
pub enum CheckpointScope {
    JobAttempt,
    JobDefinition,
    Run,
}
```

---

## 44. Baseline

`JobAttempt`.

---

## 45. Cross-Run Reuse

Usually forbidden.

---

## 46. Restartable Work

Should use cache/declared outputs instead where possible.

---

## 47. Checkpoint Creation

Sequence:

```text
request checkpoint
  ↓
quiesce workload if required
  ↓
flush state
  ↓
write payload
  ↓
hash
  ↓
write manifest
  ↓
validate
  ↓
commit CheckpointId
```

---

## 48. No Validity Before Commit

Critical.

---

## 49. Partial Upload

Garbage collectible.

---

## 50. CheckpointCommit

```rust
pub struct CheckpointCommit {
    pub id: CheckpointId,
    pub manifest: CheckpointManifestId,
    pub created_at: Timestamp,
}
```

---

## 51. Quiescence

Application-specific.

---

## 52. Crash-Consistent Snapshot

May be insufficient.

---

## 53. Application-Consistent Snapshot

Stronger.

---

## 54. CheckpointConsistency

```rust
pub enum CheckpointConsistency {
    ApplicationConsistent,
    FrameworkConsistent,
    CrashConsistent,
    BestEffort,
}
```

---

## 55. Resume Policy

Can require minimum consistency.

---

## 56. Checkpoint Validation

```rust
pub enum CheckpointValidationResult {
    Valid,
    Incompatible,
    Corrupt,
    Incomplete,
    Revoked,
    Unknown,
}
```

---

## 57. Unknown Is Not Valid

Critical.

---

## 58. Validation Checks

```text
manifest digest
payload digest
input identities
resume contract
runner compatibility
security policy
retention
```

---

## 59. Checkpoint Revocation

Can happen after:

```text
security incident
runner compromise
corrupt CAS
toolchain revocation
```

---

## 60. Compromised Runner

Checkpoint created on compromised runner may be untrusted.

---

## 61. Provenance

Checkpoint records runner baseline and attestation.

---

## 62. High-Trust Resume

Can require verified runner at checkpoint creation.

---

## 63. Cancellation Model

```rust
pub enum CancellationState {
    None,
    Requested,
    GracefulStopping,
    ForceStopping,
    Cancelled,
    CancellationFailed,
}
```

---

## 64. CancelRequestedAt

Recorded.

---

## 65. Cancellation Reason

```rust
pub enum CancellationReason {
    User,
    Superseded,
    Timeout,
    Budget,
    Incident,
    Security,
    Shutdown,
    Preemption,
    Policy,
}
```

---

## 66. User Cancel

Intentional.

---

## 67. Superseded

Newer run replaces older where Part 60 allows.

---

## 68. Timeout

Execution deadline.

---

## 69. Budget

Optional resource policy.

---

## 70. Security

May require immediate force stop.

---

## 71. Graceful Stop

Executor sends appropriate signal/protocol.

---

## 72. POSIX

Usually:

```text
SIGTERM
grace
SIGKILL
```

---

## 73. Windows

Use platform-appropriate control/job-object termination.

---

## 74. Container

Use runtime stop semantics.

---

## 75. VM

Guest agent/hypervisor operations.

---

## 76. No Platform Pretending

Critical.

---

## 77. CancellationGracePeriod

```rust
pub struct CancellationGracePeriod(Duration);
```

---

## 78. Job Override

Can request bounded grace.

---

## 79. System Maximum

Policy-controlled.

---

## 80. Security Cancel

May ignore normal grace.

---

## 81. Checkpoint On Cancel

Only if:

```text
contract allows
reason allows
time remains
```

---

## 82. User "Cancel"

Usually means stop, not suspend.

---

## 83. Suspend

Different command.

---

## 84. SuspensionState

```rust
pub enum SuspensionState {
    None,
    Requested,
    Checkpointing,
    Suspended,
    Resuming,
    Failed,
}
```

---

## 85. Suspend Contract

Requires resumability.

---

## 86. Non-Resumable Job

Suspend rejected or becomes cancel/retry according explicit option.

---

## 87. ResumeId

```rust
pub struct ResumeId(Ulid);
```

---

## 88. Resume Sequence

```text
select CheckpointId
  ↓
validate
  ↓
resolve compatible runner
  ↓
allocate
  ↓
materialize checkpoint
  ↓
start new ExecutionEpoch
  ↓
fence prior epoch
  ↓
run
```

---

## 89. Fencing Before Resume

Critical.

---

## 90. Resume Placement

Scheduler hard filters:

```text
platform
toolchain support
hardware
trust
checkpoint locality
```

---

## 91. Checkpoint Locality

Soft score.

---

## 92. Correctness Before Locality

Existing scheduler rule.

---

## 93. Spot/Preemptible Runner

Part 43 capacity class can mark:

```rust
pub enum InterruptionClass {
    Stable,
    Preemptible,
    Spot,
    BestEffort,
}
```

---

## 94. Job Eligibility

Job declares tolerance.

---

## 95. InterruptionTolerance

```rust
pub enum InterruptionTolerance {
    None,
    Restartable,
    Checkpointable,
}
```

---

## 96. Non-Checkpointable Long Job

Can still use spot if restart cost acceptable and policy permits.

---

## 97. Protected Release Job

May require stable capacity.

---

## 98. Preemption Notice

Cloud/provider may provide warning.

---

## 99. PreemptionNotice

```rust
pub struct PreemptionNotice {
    pub runner: RunnerId,
    pub deadline: Option<Timestamp>,
}
```

---

## 100. On Notice

Runner/agent can:

```text
stop new jobs
checkpoint eligible active jobs
upload logs
drain
```

---

## 101. Deadline Unreliable

Never assume full grace will be available.

---

## 102. Checkpoint Budget

Prioritize highest-value resumable work.

---

## 103. Preemption Priority

```rust
pub enum CheckpointPriority {
    Critical,
    High,
    Normal,
    Low,
}
```

---

## 104. Priority Inputs

```text
elapsed work
cost
checkpoint time
restart cost
job priority
```

---

## 105. Runner Loss

No notice.

---

## 106. RunnerLost

Part 07/10 event.

---

## 107. On Runner Loss

Job attempt becomes:

```text
ExecutionUnknown
```

until lease/heartbeat expiration and effect reconciliation.

---

## 108. Last Checkpoint

May be used after fencing old epoch.

---

## 109. No Immediate Resume While Old Epoch May Still Commit

Critical.

---

## 110. Lease/Fence

Part 60.

---

## 111. Execution Lease Expiry

Then epoch increments.

---

## 112. External Side Effects

Most important restriction.

---

## 113. Example Dangerous Work

```text
publish package
deploy
send webhook
mutate external DB
```

---

## 114. Checkpoint Cannot Make Non-Idempotent Effect Repeat Safe

Critical.

---

## 115. Effect Boundary

```rust
pub struct EffectBoundary {
    pub before_checkpoint_allowed: bool,
    pub after_effect_resume: EffectResumePolicy,
}
```

---

## 116. EffectResumePolicy

```rust
pub enum EffectResumePolicy {
    Safe,
    ReconcileBeforeResume,
    RestartFromPriorSafePoint,
    NonResumable,
}
```

---

## 117. Reconcile Before Resume

Default for ambiguous protected effect.

---

## 118. Stage Design

Recommended:

```text
pure compute
  ↓
checkpoint
  ↓
protected effect
  ↓
commit outcome
```

---

## 119. Avoid Checkpoint In Middle Of Irreversible Effect

Critical.

---

## 120. Job Step Resume Model

```rust
pub enum StepResumeMode {
    Restart,
    ResumeCheckpoint,
    AlreadyCommitted,
    Reconcile,
}
```

---

## 121. Step Completion

Immutable step result.

---

## 122. AlreadyCommitted

Skip only if exact commit evidence exists.

---

## 123. No "looks done"

Critical.

---

## 124. Partial Outputs

Uncommitted outputs never become canonical artifacts.

---

## 125. Output Commit

Only after job/step success and fencing checks.

---

## 126. Interrupted Output

CAS may contain blobs, but metadata marks uncommitted.

---

## 127. GC

Cleans unrooted partial outputs.

---

## 128. Logs

Sequence logs survive interruption.

---

## 129. Resume Log Continuity

New execution epoch in same JobAttempt can continue logical job log with epoch markers.

---

## 130. Log Sequence

Globally monotonic logical sequence.

---

## 131. Epoch Marker

Visible.

---

## 132. Test Suite Checkpointing

Possible by test shard/case completion.

---

## 133. TestResumeCursor

```rust
pub struct TestResumeCursor {
    pub completed_tests: DigestSet,
}
```

---

## 134. Test Isolation

Only safe if test ordering/shared state semantics support.

---

## 135. Stateful Tests

May need restart entire suite.

---

## 136. Build System Checkpointing

Prefer native build cache/CAS over process memory checkpoint.

---

## 137. Compilation

If compiler/build system already supports incremental outputs, model them as cache/artifacts where safe.

---

## 138. Checkpoint Only What Cannot Be Better Expressed As Cache

Critical.

---

## 139. ML/Long Computation

Application checkpoint ideal.

---

## 140. Device Test

Checkpoint may represent completed test cases, not device process memory.

---

## 141. Device Reset

On resume validate device state.

---

## 142. Benchmark

Resume often invalid because thermal/cache state changed.

---

## 143. BenchmarkResumability

Usually `None` or restart stage.

---

## 144. Reproducibility

Checkpoint must not introduce hidden state.

---

## 145. Resume Inputs

Included in provenance.

---

## 146. Job Provenance

Records:

```text
initial epoch
checkpoint IDs
resume epochs
runner IDs
```

---

## 147. Release Provenance

Can include resumed execution chain.

---

## 148. Security

Resume chain does not weaken trust.

---

## 149. Checkpoint Encryption

For sensitive job state.

---

## 150. CheckpointSensitivity

```rust
pub enum CheckpointSensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
}
```

---

## 151. Restricted

Encrypt at rest + scoped access.

---

## 152. Secrets In Memory

VM/process snapshot may capture secrets.

---

## 153. Therefore

Memory/VM checkpoint is high-risk.

---

## 154. Default

Avoid whole-memory checkpoints for secret-bearing jobs.

---

## 155. Secret Redaction Impossible Generally

Critical.

---

## 156. VM Snapshot Policy

Can forbid when secrets resolved.

---

## 157. Application Checkpoint

Preferred because it can exclude secrets.

---

## 158. Checkpoint Access

```text
checkpoint.read
checkpoint.create
checkpoint.resume
checkpoint.forensic
```

---

## 159. Tenant Isolation

Checkpoint bound to tenant/project/job.

---

## 160. Cross-Tenant Reuse

Forbidden.

---

## 161. Checkpoint Export

Restricted.

---

## 162. Replay

Part 65 can use retained checkpoints as historical evidence, but not assume resumability forever.

---

## 163. Retention

Part 46.

---

## 164. CheckpointRetentionPolicy

```rust
pub struct CheckpointRetentionPolicy {
    pub keep_latest: u16,
    pub max_age: Duration,
    pub keep_on_failure: bool,
}
```

---

## 165. Successful Job

Checkpoint can often be deleted.

---

## 166. Failed/Interrupted

Keep for recovery window.

---

## 167. Release Job

May retain checkpoint manifest/provenance, not payload.

---

## 168. Payload GC

After no longer resumable.

---

## 169. Cost

Part 45.

Checkpoint cost includes:

```text
storage
upload bandwidth
serialization time
resume time
```

---

## 170. Cost Model

Can decide whether checkpointing is worthwhile.

---

## 171. But

User/policy may require checkpoint regardless cost.

---

## 172. Spot Economics

Checkpoint-aware scheduler can favor cheaper interruptible capacity for tolerant jobs.

---

## 173. ExpectedCost

```text
spot cost
+
expected interruption loss
+
checkpoint overhead
```

---

## 174. No Cost-Only Scheduling

Trust/deadline requirements first.

---

## 175. Job Deadline

Checkpoint/resume may violate deadline.

---

## 176. Scheduler Can Prefer Stable Runner Near Deadline

---

## 177. Retry Policy

```rust
pub enum InterruptedRetryPolicy {
    Never,
    Restart,
    ResumeLatestCheckpoint,
    ResumeOrRestart,
}
```

---

## 178. ResumeOrRestart

If checkpoint invalid, restart only if job semantics allow.

---

## 179. Max Interruptions

Bounded.

---

## 180. InterruptionBudget

```rust
pub struct InterruptionBudget {
    pub max_interruptions: u16,
}
```

---

## 181. Avoid Endless Spot Thrashing

Critical.

---

## 182. Escalation To Stable Capacity

After repeated interruptions.

---

## 183. Checkpoint Corruption

Fallback according retry policy.

---

## 184. Corrupt Checkpoint

Never partially restore.

---

## 185. Checkpoint Chain

Can be incremental.

---

## 186. ParentCheckpointId

```rust
pub struct IncrementalCheckpoint {
    pub parent: Option<CheckpointId>,
    pub delta: CasObjectRef,
}
```

---

## 187. Chain Depth

Bounded.

---

## 188. Periodic Full Checkpoint

Avoid fragile long chains.

---

## 189. Parent Missing

Child invalid.

---

## 190. Dedup

CAS can deduplicate blobs.

---

## 191. Compression

Implementation detail.

---

## 192. Cross-Version Resume

Application/framework-defined.

---

## 193. Job code version changes

Normally invalidates.

---

## 194. Migration Of Checkpoint Format

Only if explicit compatibility bridge.

---

## 195. CheckpointFormatVersion

```rust
pub struct CheckpointFormatVersion(u16);
```

---

## 196. Part 57

Compatibility governance applies.

---

## 197. Old Format

Read/convert only if proven safe.

---

## 198. No Silent Conversion

Critical.

---

## 199. Workflow Suspend

Optional higher-level feature.

---

## 200. RunSuspensionId

```rust
pub struct RunSuspensionId(Ulid);
```

---

## 201. Workflow Suspend

Only if active jobs can:

```text
complete
checkpoint
or
cancel safely
```

---

## 202. Pending Jobs

Stay queued/suspended.

---

## 203. New Triggers

Separate runs.

---

## 204. Run Resume

Re-evaluates policy/resources.

---

## 205. No Resume Under Old Revoked Policy/Secrets

Critical.

---

## 206. Current Security Floor

Always applies.

---

## 207. Secret Refresh On Resume

Resolve fresh SecretRefs.

---

## 208. Do Not Restore Expired Credential From Checkpoint

Critical.

---

## 209. Application Checkpoint Must Not Contain Credentials

Preferred.

---

## 210. Network Connections

Not resumable directly.

---

## 211. Reconnect

Application/framework re-establishes.

---

## 212. Socket State Snapshot

Not baseline.

---

## 213. External Session

Must be reauthenticated.

---

## 214. Database Transaction

Open transaction cannot be assumed resumable.

---

## 215. Transaction Boundary

Checkpoint only after commit/rollback.

---

## 216. Distributed Lock

Not persisted as resume authority.

---

## 217. Reacquire lease with new fencing token.

---

## 218. File Locks

Re-created.

---

## 219. Local Temp Files

Only restored if checkpoint contract includes them.

---

## 220. Sandbox Reconstruction

New sandbox.

---

## 221. Never reuse old runner sandbox blindly.

---

## 222. Workspace Cleanup

Old runner workspace destroyed/reconciled when reachable.

---

## 223. Security Incident

Checkpoint from quarantined runner can be revoked.

---

## 224. Incident

Part 61 can suspend/cancel jobs.

---

## 225. Incident Cancellation

Reason recorded.

---

## 226. Incident Resume

Requires explicit operator/policy after incident state changes.

---

## 227. Progressive Delivery

Long rollout analysis jobs may checkpoint computation but protected traffic shifts remain external effects.

---

## 228. Migration

Part 63 backfill checkpoints are domain-specific and remain migration authority.

---

## 229. Do Not Replace Migration Checkpoint With Generic Job Checkpoint

Critical.

---

## 230. Generic Checkpoint

Can preserve worker computation around domain checkpoint, but domain state remains canonical.

---

## 231. Remote Workspace

Part 64 suspend/resume is workspace-specific; may reuse storage/VM primitives but different authority.

---

## 232. Dioxus UI

Pages/panels:

```text
Interrupted Jobs
Checkpoints
Suspended Runs
Preemption
Recovery
```

---

## 233. Job Detail

Shows:

```text
resumability mode
current epoch
latest checkpoint
interruptions
resume history
```

---

## 234. Cancellation UI

Options:

```text
Cancel
Cancel after checkpoint
Force cancel
Suspend
```

only when supported.

---

## 235. Force Cancel Warning

Clearly destructive.

---

## 236. Resume UI

Shows compatibility and checkpoint validation.

---

## 237. CLI

```text
forgeyard job checkpoint <job>
forgeyard job suspend <job>
forgeyard job resume <job>
forgeyard job cancel <job>
forgeyard job cancel --force
forgeyard checkpoint list
forgeyard checkpoint inspect
forgeyard checkpoint doctor
```

---

## 238. API

Potential:

```text
POST /v1/jobs/{id}/checkpoint
POST /v1/jobs/{id}/suspend
POST /v1/jobs/{id}/resume
POST /v1/jobs/{id}/cancel
GET  /v1/jobs/{id}/checkpoints
```

---

## 239. Permissions

```text
job.cancel
job.force_cancel
job.suspend
job.resume
checkpoint.read
checkpoint.forensic
```

---

## 240. Force Cancel

Elevated permission for protected jobs.

---

## 241. Audit

Audit:

```text
force cancellation
security cancellation
forensic checkpoint access
manual resume override
checkpoint revocation
```

---

## 242. Routine Preemption

Operational event.

---

## 243. Events

```rust
pub enum CheckpointEvent {
    Requested,
    Started,
    Created,
    ValidationFailed,
    ResumeStarted,
    ResumeCompleted,
    Revoked,
    Deleted,
}
```

---

## 244. Job Events

```text
PreemptionNoticeReceived
CancellationRequested
SuspensionRequested
RunnerLost
ResumeScheduled
```

---

## 245. At-Least-Once

Handlers idempotent.

---

## 246. Observability Metrics

```text
job_interruptions_total
job_resume_total
job_resume_failures_total
checkpoint_created_total
checkpoint_bytes
checkpoint_create_seconds
checkpoint_restore_seconds
job_force_cancel_total
```

---

## 247. Labels

Low cardinality:

```text
reason
resumability_mode
result
interruption_class
```

---

## 248. Tracing

```text
checkpoint.create
checkpoint.validate
checkpoint.restore
job.cancel
job.suspend
job.resume
preemption.handle
```

---

## 249. Health

```rust
pub enum CheckpointSubsystemHealth {
    Healthy,
    CasDegraded,
    ValidationDegraded,
    ResumeDegraded,
    Unhealthy,
}
```

---

## 250. Doctor

```text
forgeyard checkpoint doctor
```

Checks:

```text
orphan checkpoints
invalid chains
stale suspended jobs
resume loops
revoked checkpoint referenced
checkpoint retention overflow
```

---

## 251. Resume Loop Detection

Important.

---

## 252. Example

Repeated:

```text
resume
  ↓
immediate crash
  ↓
resume same checkpoint
```

---

## 253. ResumeAttemptCount

Bounded.

---

## 254. Bad Checkpoint

Quarantine after threshold.

---

## 255. Failure Diagnosis

Part 48.

Can compare:

```text
failure before interruption
failure after resume
```

---

## 256. Reproducibility

Resume path is part of execution evidence.

---

## 257. Historical Replay

Part 65 can reproduce interruption sequence if artifacts retained.

---

## 258. Federation

Checkpoint data residency applies.

---

## 259. Resume Site

Must satisfy:

```text
residency
artifact availability
hardware
trust
```

---

## 260. Cross-Region Resume

Allowed only if data policy permits.

---

## 261. Site Partition

Local checkpoint may remain local.

---

## 262. Global Scheduler

Does not migrate restricted checkpoint blindly.

---

## 263. Air-Gap

Checkpoint/resume works with local CAS.

---

## 264. DR

Checkpoint metadata/data included only if durability promise requires.

---

## 265. After Metadata Restore

Checkpoint validity recalculated.

---

## 266. Stale Execution Epochs

Invalidated.

---

## 267. No Resume From Checkpoint Whose Parent Run State Was Rolled Back Incorrectly

Critical.

---

## 268. Backup Timing

Checkpoint payload without metadata may be orphaned.

---

## 269. Metadata without payload

Checkpoint unavailable.

---

## 270. Consistency

Doctor/reconciler handles.

---

## 271. Reconciler

Checks:

```text
runner lost
lease expired
latest valid checkpoint
resume eligibility
orphan payload
suspended timeout
```

---

## 272. HA

Multiple recovery controllers safe.

---

## 273. Resume Claim

Part 60 lease.

---

## 274. Only one accepted current epoch.

---

## 275. Scheduler Integration

Runner selection includes resume compatibility.

---

## 276. Fleet Autoscaling

Can provision stable capacity after repeated spot interruption.

---

## 277. Resource Backpressure

Checkpointing many jobs simultaneously can overload CAS.

---

## 278. Checkpoint Admission Control

```rust
pub struct CheckpointAdmissionPolicy {
    pub max_concurrent_uploads: u32,
    pub max_bandwidth: Option<Bandwidth>,
}
```

---

## 279. Preemption Storm

Prioritize.

---

## 280. Runner Shutdown

Drain window can trigger checkpoint.

---

## 281. Maintenance

Part 58/43 fleet recycle.

---

## 282. Graceful Fleet Drain

```text
stop assignment
  ↓
checkpoint/resume eligible jobs
  ↓
wait bounded jobs
  ↓
terminate
```

---

## 283. No Unlimited Drain

Critical.

---

## 284. Deadline

Fleet maintenance may force after max drain.

---

## 285. Testkit

```text
forgeyard-checkpoint-testkit/src/
├── lib.rs
├── contract.rs
├── create.rs
├── validate.rs
├── resume.rs
├── cancellation.rs
├── preemption.rs
├── fencing.rs
└── assertions.rs
```

---

## 286. Core Tests

### Checkpoint
- partial upload never valid;
- payload/manifest digest verified;
- source/toolchain/job changes invalidate.

### Epoch
- old runner completion after resume rejected;
- only current epoch publishes outputs.

### Cancellation
- graceful then force sequence;
- security cancellation bypasses ordinary grace where policy says.

### Suspend
- non-resumable job cannot silently suspend;
- valid checkpoint required.

### Resume
- hardware/platform mismatch rejected;
- new runner starts new epoch;
- expired secrets re-resolved, not restored.

### External Effects
- ambiguous publish/deploy effect reconciled before resume;
- checkpoint does not cause duplicate effect.

### Spot
- preemption notice creates checkpoint when time permits;
- repeated interruptions escalate capacity.

### Security
- compromised checkpoint revoked;
- VM snapshot with secret-bearing job blocked by policy.

---

## 287. Chaos Tests

Inject:

```text
runner disappears during checkpoint
CAS upload interruption
controller crash during resume
old runner returns after fencing
spot preemption storm
region partition
```

Expected:

```text
no duplicate current epoch
no partial checkpoint accepted
safe retry/reconcile
```

---

## 288. Scale Tests

Test:

```text
thousands of simultaneous preemptions
large checkpoint payloads
GPU checkpoint fleet
long-running jobs over many hours
large checkpoint chain
```

---

## 289. Implementation Phases

### Phase 1 — Execution Epoch & Cancellation
Core correctness.

### Phase 2 — Stage Restart
Simple resumability.

### Phase 3 — Application Checkpoint Contract
General checkpoint API.

### Phase 4 — CAS Checkpoint Storage
Manifest/validation.

### Phase 5 — Resume Scheduler Integration
Cross-runner recovery.

### Phase 6 — Spot/Preemptible Capacity
Cost optimization.

### Phase 7 — Framework/Test Checkpointing
Higher-level progress.

### Phase 8 — Sensitive/Encrypted Checkpoints
Security.

### Phase 9 — Fleet Drain/Maintenance
Operational integration.

### Phase 10 — Federation/DR
Distributed recovery.

### Phase 11 — UI/CLI/Doctor
Operability.

### Phase 12 — Chaos/Scale/Security Hardening
Production readiness.

---

## 290. Acceptance Tests

1. Every resume starts a new ExecutionEpoch.
2. Old epochs are fenced from output/effect commit.
3. A checkpoint is valid only after atomic manifest/payload commit.
4. Partial uploads are never resumable checkpoints.
5. Source/job/toolchain mismatches invalidate checkpoint.
6. Filesystem snapshots are not assumed semantically resumable.
7. VM/process snapshots are high-risk and policy-controlled.
8. Cancellation and suspension are distinct.
9. User cancel does not silently become suspend.
10. Non-resumable jobs cannot pretend to suspend.
11. Graceful cancellation has bounded platform-aware semantics.
12. Force cancellation is explicit.
13. Security cancellation can override normal grace.
14. Spot/preemptible eligibility is declared by job policy.
15. Preemption notice is best-effort, not guaranteed.
16. Runner loss does not immediately imply safe duplicate execution.
17. Resume waits for fencing/reconciliation of prior epoch.
18. Checkpoint never makes external side effects automatically repeatable.
19. Protected effects use idempotency/fencing/reconciliation.
20. Uncommitted partial outputs never become canonical artifacts.
21. Checkpoint and cache are separate concepts.
22. Secrets are not restored from expired checkpoint credentials.
23. Secret-bearing whole-memory snapshots are restricted.
24. Repeated resume failure quarantines bad checkpoint/escalates.
25. Scheduler enforces resume hardware/platform/trust compatibility.
26. Spot thrashing is bounded by interruption budget.
27. Fleet drain can checkpoint/resume eligible jobs.
28. Federation/residency constrains checkpoint movement.
29. DR recalculates checkpoint validity and epochs.
30. Forgeyard dogfoods checkpoint/preemption on its own long-running CI workloads.

---

## 291. Production Readiness Gates

Do not call checkpoint/preemption architecture production-ready until:

```text
execution epoch fencing is proven
partial checkpoint rejection is proven
cross-runner resume validation works
external-effect resume safety is tested
spot interruption recovery works
secret-bearing checkpoint policy is enforced
resume-loop detection works
fleet-drain integration works
federation/DR tests pass
chaos/scale tests pass
```

---

## 292. Architectural Invariants

1. resume requires explicit valid checkpoint;
2. partial process state is not checkpoint;
3. every resume increments execution epoch;
4. stale epochs cannot commit;
5. checkpoint binds exact execution inputs;
6. checkpoint and cache are different;
7. cancellation and suspension are different;
8. non-resumable jobs stay non-resumable;
9. platform stop semantics are explicit;
10. checkpoint creation is committed atomically;
11. Unknown validation is not Valid;
12. external effects require independent safety semantics;
13. partial outputs remain uncommitted;
14. secrets are not blindly restored;
15. whole-memory snapshots are restricted;
16. runner loss requires fencing/reconciliation;
17. spot eligibility is declared;
18. repeated interruption is bounded;
19. resume compatibility is scheduler hard filter;
20. correctness beats checkpoint locality;
21. bad checkpoints are quarantined;
22. checkpoint retention is explicit;
23. resume chain appears in provenance;
24. federation/residency apply;
25. DR invalidates stale execution epochs;
26. maintenance drain is bounded;
27. app/domain checkpoints remain domain authority;
28. HA recovery controllers are idempotent/fenced;
29. no fake arbitrary-process checkpoint guarantee;
30. Forgeyard dogfoods its own checkpoint system.

---

## 293. Final Target Architecture

```text
                    JobAttempt
                        │
                        ▼
                  ExecutionEpoch 1
                        │
                  ┌─────┴─────┐
                  ▼           ▼
               Running     Checkpoint
                              │
                              ▼
                        CheckpointId
                              │
                     interruption/loss
                              │
                              ▼
                        fence epoch 1
                              │
                              ▼
                        validate checkpoint
                              │
                              ▼
                       ExecutionEpoch 2
```

Cancellation:

```text
CancelRequested
      ↓
graceful signal
      ↓
optional checkpoint
      ↓
bounded grace
      ↓
force termination if required
```

External effect boundary:

```text
pure compute
    ↓
checkpoint
    ↓
protected external effect
    ↓
commit/reconcile effect
    ↓
next resumable point
```

The key guarantee is:

> **Forgeyard can safely exploit spot capacity, survive runner loss, suspend expensive work, and reclaim fleet resources without pretending arbitrary processes are magically resumable. Resume is allowed only from explicit verified checkpoints under a declared contract, with execution epochs fencing stale workers and external effects remaining independently idempotent and reconcilable.**

---

## 294. Extended Architecture Sequence

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
12 Secrets / Trust
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
26 Self-Hosting / Bootstrap / Release Forgeyard
27 Multi-Tenancy / Quotas / Resource Governance
28 Audit / Compliance / Security Governance
29 Notifications / Alerting / Human Workflow
30 Entitlements / Licensing / Subscription
31 Search / Indexing / Operational Analytics
32 Test Results / Quality / Coverage / Flaky Intelligence
33 Benchmark / Performance / Load / Capacity
34 Monorepo / Dependency Graph / Affected Work
35 Developer Experience / Local Dev
36 Dependency / Registry / Mirror Governance
37 Static Analysis / Security Findings
38 Cache / Remote Cache / Correctness
39 Configuration / Feature Flags / Runtime
40 Security / Threat Model / Incident Response
41 Release Distribution / Update Delivery
42 Workflow Templates / Golden Paths
43 Runner Fleet Autoscaling / Provisioning
44 Pipeline Triggers / Schedules / Dispatch
45 Cost Accounting / FinOps
46 Data Lifecycle / Retention / Privacy
47 CI/CD Migration / Compatibility
48 Failure Diagnosis / Reproduction / Bisect
49 Service Catalog / Ownership / Developer Portal
50 Reliability / SLO / Error Budget / Resilience
51 Multi-Region Federation / Edge / Disconnected
52 Artifact Registry / OCI / Package Distribution
53 Infrastructure-as-Code / Preview Environments / Drift
54 Merge Queue / Speculative Integration / Batch Validation
55 AI-Assisted CI Optimization / Engineering Copilot
56 Test Data / Fixtures / Ephemeral Databases / Service Virtualization
57 API / ABI / Schema / Protocol Compatibility / Contract Evolution
58 Runner Image Factory / Golden Images / Patch Management / Baseline Attestation
59 Network Connectivity / Private Resource Access / Egress / Tunneling / Zero-Trust Connectivity
60 Workflow Concurrency / Distributed Locks / Idempotency / Reservations / Exclusive Coordination
61 Incident Management / On-Call / Escalation / Response Coordination / Postmortem
62 Environment Promotion / Progressive Delivery / Feature Rollout / Canary Analysis / Automated Rollback
63 Database Schema Migration / Online Backfill / Data Transformation / Zero-Downtime Change Orchestration
64 Remote Development Environments / Cloud Workspaces / Developer Workspace Orchestration
65 Build Graph Replay / Historical Reproducibility / Time-Travel CI / Evidence Reconstruction
66 Change Risk Assessment / Preflight Simulation / Policy Preview / What-If Analysis
67 Artifact Promotion Policy / Release Train / Environment Channel / Lifecycle Governance
68 Configuration Drift Detection / Desired-State Convergence / Runtime Reconciliation / Environment Consistency
69 Job Checkpointing / Suspend-Resume / Preemption / Graceful Cancellation / Interruptible Runner Recovery
```
