# 05 — Forgeyard Run / Job State Machine System Architecture

**Document type:** Core Execution-State System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Pipeline run lifecycle, job lifecycle, attempts, leases, retries, cancellation, timeout, dependency propagation, aggregate status, durable transitions, stale completion rejection, idempotency, recovery, reconciliation, and persistence semantics  
**Architecture style:** Explicit persisted state machines with typed transitions, immutable attempts, lease-bound execution, event/audit coupling, and at-least-once-safe reconciliation  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on `01-forgeyard-core-domain-foundation.md`, `02-forgeyard-storage-metadata.md`, `03-forgeyard-cas-artifact-data-plane.md`, and `04-forgeyard-pipeline-ir-parsing-planning.md`. It consumes `ExecutablePlan` and produces durable run/job execution state for the scheduler and runner subsystems.

---

# 1. Purpose

Forgeyard needs a precise execution-state model for:

```text
pipeline runs
jobs
attempts
leases
retries
cancellation
timeouts
failures
dependency propagation
recovery
```

A CI/CD system becomes unreliable very quickly if state is represented as:

```text
status: String
```

with ad-hoc updates.

The central rule is:

> **Every persisted execution state transition is explicit, validated, versioned, auditable, and safe under retries, duplicate messages, process crashes, stale workers, and network partitions.**

A second rule:

> **A Job is the logical unit of planned work; a JobAttempt is one concrete execution attempt; a Lease authorizes exactly one attempt on one runner for a bounded time.**

A third rule:

> **A worker result is accepted only if it matches the currently authoritative attempt + lease.**

---

# 2. Architectural Position

```text
                  ExecutablePlan
                       │
                       ▼
                     Run
                       │
                       ▼
                   Job Graph
                       │
                       ▼
                  Job States
                       │
                Scheduler/Lease
                       │
                       ▼
                  JobAttempt
                       │
                       ▼
                    Runner
                       │
                       ▼
                  Completion
                       │
                       ▼
             validated state transition
```

---

# 3. Goals

The subsystem MUST:

1. define immutable `RunId`;
2. define immutable `JobId`;
3. define immutable `JobAttemptId`;
4. define `LeaseId`;
5. model run lifecycle explicitly;
6. model job lifecycle explicitly;
7. model attempt lifecycle explicitly;
8. validate all transitions;
9. persist transition reasons;
10. support retries;
11. distinguish infrastructure failure from workload failure;
12. support cancellation;
13. support timeout;
14. support stale lease rejection;
15. support stale completion rejection;
16. support dependency propagation;
17. support fail-fast;
18. support continue-on-error;
19. support skipped/pruned jobs;
20. support manual gates;
21. support matrix aggregates;
22. support idempotent commands;
23. support durable events;
24. support recovery/reconciliation;
25. work with Stoolap and PostgreSQL/Neon;
26. remain scheduler/runner implementation-neutral.

---

# 4. Non-Goals

This subsystem does not:

```text
choose runners
execute processes
sandbox workloads
stream bytes
resolve secrets
compile pipeline config
```

Those are neighboring systems.

It owns execution **state truth**.

---

# 5. Workspace Structure

```text
crates/run/
├── forgeyard-run/
├── forgeyard-run-model/
├── forgeyard-run-state/
├── forgeyard-run-service/
├── forgeyard-run-store-api/
├── forgeyard-job/
├── forgeyard-job-model/
├── forgeyard-job-state/
├── forgeyard-job-attempt/
├── forgeyard-job-dependency/
├── forgeyard-job-result/
├── forgeyard-job-retry/
├── forgeyard-job-timeout/
├── forgeyard-job-cancel/
├── forgeyard-run-aggregate/
├── forgeyard-run-events/
├── forgeyard-run-reconcile/
└── forgeyard-run-testkit/
```

Related lease crates:

```text
crates/lease/
├── forgeyard-lease/
├── forgeyard-job-lease/
└── forgeyard-lease-testkit/
```

---

# 6. `Run`

A Run is one execution instance of one immutable planned pipeline.

```rust
pub struct Run {
    pub id: RunId,
    pub project: ProjectId,
    pub pipeline: PipelineDefinitionId,
    pub plan: PipelinePlanId,
    pub source: SourceSnapshotId,
    pub state: RunState,
    pub created_by: ActorRef,
    pub created_at: Timestamp,
    pub version: EntityVersion,
}
```

---

# 7. Run Identity

`RunId` is an entity ID.

It does not derive from the pipeline plan digest because:

```text
same plan may run multiple times
```

---

# 8. Run State

Recommended:

```rust
pub enum RunState {
    Created,
    Planning,
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Superseded,
}
```

---

# 9. Why `Planning`

If run creation and plan materialization are separate durable stages:

```text
Created
  ↓
Planning
  ↓
Queued
```

If planning is always completed before Run creation, `Planning` may be omitted.

Recommended new Forgeyard:

```text
create Run only after ExecutablePlan exists
```

Then state can start at:

```text
Queued
```

This architecture supports either, but simpler implementation should prefer:

```text
Queued
```

as first persisted execution state.

---

# 10. Recommended Final Run State Set

```rust
pub enum RunState {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Superseded,
}
```

---

# 11. Run Terminal States

```text
Succeeded
Failed
Cancelled
TimedOut
Superseded
```

Terminal states cannot transition back to active.

---

# 12. Job

A Job is one logical planned node in the executable DAG.

```rust
pub struct Job {
    pub id: JobId,
    pub run: RunId,
    pub node: JobNodeId,
    pub state: JobState,
    pub attempt_count: u16,
    pub current_attempt: Option<JobAttemptId>,
    pub required: bool,
    pub version: EntityVersion,
}
```

---

# 13. Job State

Canonical state set:

```rust
pub enum JobState {
    Pending,
    Eligible,
    Leased,
    Preparing,
    Running,
    UploadingOutputs,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
    Skipped,
}
```

---

# 14. Meaning of `Pending`

Job exists but prerequisites are not yet satisfied.

Examples:

```text
upstream jobs still running
manual gate not satisfied
condition not yet evaluable
```

---

# 15. Meaning of `Eligible`

All required dependencies and gates are satisfied.

Scheduler may now place it.

---

# 16. Meaning of `Leased`

Scheduler has granted one runner authority to execute one specific attempt.

---

# 17. Meaning of `Preparing`

Runner accepted the lease and is:

```text
fetching inputs
materializing source
resolving late secrets
creating sandbox
preparing toolchain
```

---

# 18. Meaning of `Running`

User workload/process has begun.

---

# 19. Meaning of `UploadingOutputs`

Process finished and runner is:

```text
capturing outputs
hashing
uploading CAS
publishing logs/reports
```

Job is not yet successful.

---

# 20. Meaning of `Succeeded`

Required output/evidence commit succeeded.

---

# 21. Meaning of `Failed`

Attempt ended in non-retryable or exhausted failure and job will not automatically retry.

---

# 22. Meaning of `Cancelled`

Cancellation intent was honored.

---

# 23. Meaning of `TimedOut`

Job exceeded configured timeout.

---

# 24. Meaning of `Lost`

Authoritative attempt disappeared or its lease expired without a valid final result.

`Lost` is usually a transition point for retry logic rather than necessarily final business failure.

---

# 25. Meaning of `Skipped`

Job will not execute due to:

```text
condition false
dependency failure + policy
plan pruning
manual exclusion
supersession
```

---

# 26. Job Terminal States

Potential terminal logical states:

```text
Succeeded
Failed
Cancelled
TimedOut
Skipped
```

`Lost` can be terminal only if retry policy is exhausted.

Recommended:

```text
Lost -> Eligible
```

for retry if allowed;

or:

```text
Lost -> Failed
```

if no retries remain.

---

# 27. State vs Attempt

Do not model retries by bouncing same attempt back to `Running`.

Each retry creates a new:

```text
JobAttempt
```

---

# 28. JobAttempt

```rust
pub struct JobAttempt {
    pub id: JobAttemptId,
    pub job: JobId,
    pub number: AttemptNumber,
    pub state: AttemptState,
    pub runner: Option<RunnerId>,
    pub lease: Option<LeaseId>,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub result: Option<JobAttemptResult>,
}
```

---

# 29. Attempt Number

```rust
pub struct AttemptNumber(NonZeroU16);
```

Monotonically increasing per Job.

---

# 30. Attempt State

```rust
pub enum AttemptState {
    Created,
    Leased,
    Preparing,
    Running,
    UploadingOutputs,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
    Rejected,
}
```

---

# 31. Why Attempt State Exists

Job state is logical outcome.

Attempt state is physical execution history.

Example:

```text
Attempt 1 -> Lost
Attempt 2 -> Failed transient
Attempt 3 -> Succeeded

Job -> Succeeded
```

---

# 32. Attempt Immutability

Once terminal, attempt record is immutable except append-only evidence references if explicitly allowed.

Recommended:

```text
terminal attempt = immutable
```

---

# 33. Lease

```rust
pub struct JobLease {
    pub id: LeaseId,
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub runner: RunnerId,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
    pub epoch: LeaseEpoch,
}
```

---

# 34. Lease Authority

The valid current lease is the authorization token for state-changing worker messages.

---

# 35. Lease Epoch

```rust
pub struct LeaseEpoch(u64);
```

Useful if a Job can be re-leased after expiry.

New lease gets higher epoch.

---

# 36. Lease Invariant

At most one authoritative active lease for one job attempt.

---

# 37. Lease Acquisition Flow

```text
Job Eligible
   ↓
scheduler selects runner
   ↓
transaction:
    create Attempt
    create Lease
    Job -> Leased
   ↓
commit
```

---

# 38. Lease Acceptance

Runner receives lease.

If runner accepts:

```text
Attempt -> Preparing
Job -> Preparing
```

---

# 39. Lease Rejection

Runner may reject before execution due to:

```text
capability changed
disk pressure
shutdown/drain
local preparation issue
```

Attempt:

```text
Rejected
```

Job may return to:

```text
Eligible
```

without consuming retry budget depending classification.

---

# 40. Lease Expiry

If lease expires before valid completion:

```text
Attempt -> Lost
Job -> Lost
```

then retry policy decides.

---

# 41. Heartbeats

Lease may be renewed via:

```text
runner heartbeat
attempt heartbeat
```

depending scheduler design.

---

# 42. Lease Renewal

Only current lease ID/epoch can renew.

---

# 43. Stale Heartbeat

Old lease heartbeat:

```text
ignored/rejected
```

must not revive job.

---

# 44. Stale Completion

Critical rule:

```text
Attempt A lease expired
Job re-leased as Attempt B
Attempt A later reports success
```

Forgeyard MUST reject Attempt A completion as authoritative.

---

# 45. Stale Completion Result

Record optional audit:

```text
LateAttemptResultRejected
```

but do not mutate Job.

---

# 46. Output Objects from Stale Attempt

CAS objects may already exist.

They remain:

```text
unreferenced
```

or diagnostic-only if explicitly preserved.

GC handles them.

---

# 47. Job Transition API

```rust
pub fn transition_job(
    current: JobState,
    event: JobEvent,
) -> Result<JobState, JobTransitionError>
```

---

# 48. Job Events

```rust
pub enum JobEvent {
    DependenciesSatisfied,
    LeaseGranted,
    PreparationStarted,
    WorkStarted,
    WorkFinished,
    OutputUploadStarted,
    AttemptSucceeded,
    AttemptFailed(FailureClass),
    CancelRequested,
    Cancelled,
    TimeoutReached,
    LeaseExpired,
    Skip(SkipReason),
}
```

---

# 49. Run Transition API

```rust
pub fn transition_run(
    current: RunState,
    event: RunEvent,
) -> Result<RunState, RunTransitionError>
```

---

# 50. Run Events

```rust
pub enum RunEvent {
    FirstJobStarted,
    AllRequiredJobsSucceeded,
    RequiredJobFailed,
    CancelRequested,
    AllActiveJobsCancelled,
    RunTimeoutReached,
    Superseded,
}
```

---

# 51. State Transition Table

Recommended logical transitions:

```text
Pending
  -> Eligible
  -> Skipped
  -> Cancelled

Eligible
  -> Leased
  -> Skipped
  -> Cancelled

Leased
  -> Preparing
  -> Eligible    (lease rejected/revoked before execution)
  -> Lost
  -> Cancelled

Preparing
  -> Running
  -> Failed
  -> TimedOut
  -> Cancelled
  -> Lost

Running
  -> UploadingOutputs
  -> Failed
  -> TimedOut
  -> Cancelled
  -> Lost

UploadingOutputs
  -> Succeeded
  -> Failed
  -> TimedOut
  -> Cancelled
  -> Lost

Lost
  -> Eligible    (retry)
  -> Failed      (retry exhausted)

Failed
  terminal

Succeeded
  terminal

Cancelled
  terminal

TimedOut
  terminal or retryable depending policy via new attempt path

Skipped
  terminal
```

---

# 52. Retry Modeling

Do not transition:

```text
Failed -> Running
```

Instead:

```text
Attempt 1 Failed
Job decides retry
Job -> Eligible
Attempt 2 created later
```

---

# 53. Retry Policy

From pipeline plan:

```rust
pub struct JobRetryPolicy {
    pub max_attempts: NonZeroU16,
    pub retry_on: RetryPredicate,
    pub backoff: BackoffPolicy,
}
```

---

# 54. Failure Classification

```rust
pub enum FailureClass {
    UserCommand,
    TestFailure,
    CompileFailure,
    Infrastructure,
    RunnerLost,
    InputFetch,
    OutputUpload,
    SandboxSetup,
    ToolchainUnavailable,
    Timeout,
    Cancelled,
    Policy,
    Internal,
}
```

---

# 55. Retry Defaults

Usually retry:

```text
Infrastructure
RunnerLost
transient InputFetch
transient OutputUpload
```

Usually do not retry automatically:

```text
CompileFailure
TestFailure
Policy
```

unless user explicitly requests.

---

# 56. Retry Budget

Attempt count vs configured maximum.

---

# 57. Retry Backoff

Persist next eligible time:

```rust
pub struct RetrySchedule {
    pub eligible_at: Timestamp,
}
```

Job may remain:

```text
PendingRetry
```

or reuse `Pending`.

---

# 58. Should `PendingRetry` Be a State?

Option A:

```text
JobState::Pending
+ retry_not_before
```

Option B:

```text
JobState::RetryWaiting
```

Recommended for clarity:

```rust
pub enum JobState {
    Pending,
    RetryWaiting,
    Eligible,
    ...
}
```

---

# 59. Final Recommended Job State

```rust
pub enum JobState {
    Pending,
    RetryWaiting,
    Eligible,
    Leased,
    Preparing,
    Running,
    UploadingOutputs,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
    Skipped,
}
```

---

# 60. RetryWaiting

Meaning:

```text
attempt ended retryably
backoff has not elapsed
```

Transition:

```text
RetryWaiting -> Eligible
```

when time reached and run still active.

---

# 61. Timeout Types

Differentiate:

```text
queue timeout
job execution timeout
step timeout
run timeout
lease timeout
```

---

# 62. Queue Timeout

Job eligible but no runner became available within allowed time.

Could classify:

```text
Infrastructure/Capacity
```

---

# 63. Execution Timeout

Starts at configured semantic point.

Recommended:

```text
when Attempt enters Running
```

unless pipeline says timeout includes preparation.

---

# 64. Preparation Timeout

Separate safety limit for:

```text
input fetch
toolchain materialization
sandbox creation
```

---

# 65. Output Upload Timeout

Separate operational limit.

---

# 66. Run Timeout

Whole run deadline.

If reached:

```text
Run -> Cancelling/TimedOut
active jobs cancellation requested
```

---

# 67. Timeout Authority

Daemon/control plane determines authoritative timeout based on persisted timestamps/deadlines.

Runner may enforce local timeout too as defense in depth.

---

# 68. Cancellation

Cancellation is an intent plus eventual terminal state.

Run:

```text
Running -> Cancelling -> Cancelled
```

---

# 69. Job Cancellation Request

Active Job receives cancellation intent.

Runner attempts graceful termination then hard kill according to sandbox policy.

---

# 70. Cancel vs Fail

User cancellation must not be reported as workload failure.

---

# 71. Cancel Race with Success

If runner completed successfully before authoritative cancel transition:

```text
transaction ordering determines result
```

Define rule explicitly.

Recommended:

- if success transaction commits before cancellation takes authority, Job succeeds;
- if cancellation intent already persisted before final commit, completion may be rejected or marked cancelled according to policy.

---

# 72. Cancellation Epoch

Optional:

```rust
pub struct CancellationEpoch(u64);
```

Can simplify races in highly distributed environments.

Initial implementation may rely on entity version + persisted cancel request timestamp.

---

# 73. Run Cancellation Propagation

When Run cancellation requested:

```text
Pending/Eligible/RetryWaiting -> Cancelled
Leased/Preparing/Running/UploadingOutputs -> cancellation requested
```

---

# 74. Dependency Graph

Jobs originate from `ExecutablePlan` DAG.

Runtime tracks dependency satisfaction.

---

# 75. Dependency State

```rust
pub struct JobDependencyState {
    pub job: JobId,
    pub prerequisite: JobId,
    pub required: bool,
    pub satisfied: bool,
}
```

Could be derived rather than stored individually.

---

# 76. Dependency Satisfaction

Required dependency:

```text
Succeeded
```

usually satisfies.

Optional/allowed-failure semantics can differ.

---

# 77. Failure Propagation

If required upstream fails:

downstream policy decides:

```text
Skipped
Cancelled
still eligible if condition allows failure
```

---

# 78. Dependency Failure Reason

```rust
pub enum SkipReason {
    ConditionFalse,
    UpstreamFailed(JobId),
    UpstreamCancelled(JobId),
    Policy,
    Superseded,
    Manual,
    PlanPruned,
}
```

---

# 79. Continue-on-Error

A failed job marked:

```text
FailureImpact::Allowed
```

does not necessarily fail Run or block downstream.

---

# 80. Required vs Informational Jobs

Run aggregate only considers required jobs for success.

---

# 81. Aggregate Run Status

Run status is derived from job states + run-level intent.

Do not allow arbitrary direct assignment from UI.

---

# 82. Run Succeeded

Condition:

```text
all required terminal jobs satisfy success policy
```

and no active jobs remain.

---

# 83. Run Failed

At least one required job reaches final blocking failure and no retry remains.

---

# 84. Run Cancelled

Cancellation requested and all required active jobs reached cancelled/terminal resolution.

---

# 85. Run TimedOut

Run-level deadline reached and terminalization completes.

---

# 86. Matrix Aggregate

Matrix children are ordinary jobs with shared origin metadata.

UI may compute group summary:

```text
3/4 passed
1 failed
```

Do not create special execution semantics unless needed.

---

# 87. Fail-Fast

Matrix/pipeline fail-fast may issue cancellation requests to sibling jobs.

---

# 88. Fail-Fast Is Policy

It does not mutate already completed outcomes.

---

# 89. Manual Gates

Gate jobs/nodes may exist as:

```text
Pending
```

until approval evidence arrives.

Alternative:

separate gate entity.

Recommended:

```text
Gate is a planned dependency node
```

with explicit gate state rather than pretending it's a runner job.

---

# 90. Gate State

```rust
pub enum GateState {
    Pending,
    Satisfied,
    Rejected,
    Cancelled,
}
```

---

# 91. Gate Integration

Run dependency resolver treats satisfied gate like successful prerequisite.

---

# 92. Job Result

```rust
pub struct JobResult {
    pub outcome: JobOutcome,
    pub outputs: Vec<ArtifactId>,
    pub logs: Option<LogStreamId>,
    pub reports: Vec<ArtifactId>,
    pub finished_at: Timestamp,
}
```

---

# 93. Job Outcome

```rust
pub enum JobOutcome {
    Success,
    Failure(FailureClass),
    Cancelled,
    TimedOut,
    Lost,
}
```

---

# 94. Attempt Result

Contains lower-level execution details:

```rust
pub struct JobAttemptResult {
    pub exit: Option<ProcessExit>,
    pub failure: Option<FailureClass>,
    pub outputs: Vec<CasObjectRef>,
    pub diagnostics: Vec<ArtifactId>,
}
```

---

# 95. Process Exit

```rust
pub struct ProcessExit {
    pub code: Option<i32>,
    pub signal: Option<SignalInfo>,
}
```

Platform-neutral representation.

---

# 96. Failure Reason Detail

Typed detail can include:

```text
command index
step ID
toolchain
sandbox
CAS backend
runner
```

but user-safe rendering separate.

---

# 97. State Reason

Every nontrivial transition may store:

```rust
pub struct StateReason {
    pub code: StateReasonCode,
    pub message: Option<BoundedString>,
}
```

---

# 98. Transition Record

```rust
pub struct JobStateTransition {
    pub job: JobId,
    pub from: JobState,
    pub to: JobState,
    pub reason: StateReason,
    pub at: Timestamp,
    pub actor: ActorRef,
    pub version: EntityVersion,
}
```

---

# 99. Persist Current + History

Recommended:

```text
current job row
+
append transition/event history
```

same transaction.

---

# 100. Event Coupling

State change transaction:

```text
validate
  ↓
update row/version
  ↓
append JobStateChanged event
  ↓
append audit if needed
  ↓
commit
```

---

# 101. Run Events

Examples:

```rust
pub enum RunDomainEvent {
    RunStarted,
    RunCancelling,
    RunSucceeded,
    RunFailed,
    RunCancelled,
    RunTimedOut,
    RunSuperseded,
}
```

---

# 102. Job Events

Examples:

```rust
pub enum JobDomainEvent {
    JobEligible,
    JobLeased,
    JobPreparing,
    JobStarted,
    JobUploadingOutputs,
    JobSucceeded,
    JobFailed,
    JobCancelled,
    JobTimedOut,
    JobLost,
    JobSkipped,
    JobRetryScheduled,
}
```

---

# 103. Event Idempotency

Duplicate same transition command must not append duplicate semantic transition.

Use:

```text
entity version
command id/idempotency key
lease identity
```

---

# 104. Command API

Examples:

```rust
pub struct MarkJobEligible { ... }
pub struct GrantJobLease { ... }
pub struct StartPreparation { ... }
pub struct StartJobExecution { ... }
pub struct CompleteJobAttempt { ... }
pub struct RequestRunCancellation { ... }
```

---

# 105. Service

```rust
#[async_trait]
pub trait RunService {
    async fn create_run(...);
    async fn cancel_run(...);
    async fn reconcile_run(...);
}
```

---

# 106. Job Service

```rust
#[async_trait]
pub trait JobService {
    async fn mark_eligible(...);
    async fn grant_lease(...);
    async fn accept_lease(...);
    async fn start(...);
    async fn complete(...);
    async fn fail(...);
}
```

---

# 107. Store API

Domain-specific atomic methods.

```rust
#[async_trait]
pub trait JobStore {
    async fn transition(
        &self,
        command: PersistJobTransition,
    ) -> Result<Versioned<JobRecord>, StoreError>;

    async fn create_attempt_with_lease(
        &self,
        command: CreateAttemptLease,
    ) -> Result<AttemptLeaseRecord, StoreError>;
}
```

---

# 108. No Raw CRUD for Critical State

Avoid service logic:

```text
read job
set status
write job
```

without version check.

---

# 109. Entity Version

Every mutable Run/Job row has:

```text
EntityVersion
```

---

# 110. Optimistic Concurrency

Transition requires:

```text
expected_version
```

---

# 111. Conflict Handling

If stale:

```text
reload
re-evaluate command
```

Never blindly overwrite.

---

# 112. Scheduler Boundary

Scheduler can request:

```text
lease eligible job
```

but state subsystem validates atomic authority.

---

# 113. Runner Boundary

Runner sends:

```text
LeaseId
AttemptId
JobId
state/progress/result
```

for every authoritative message.

---

# 114. Runner Cannot Set Job State Arbitrarily

Runner says:

```text
"attempt started"
```

service decides legal transition.

---

# 115. Heartbeat Model

```rust
pub struct AttemptHeartbeat {
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub runner: RunnerId,
    pub observed_at: Timestamp,
}
```

---

# 116. Heartbeat Persistence

Do not append full durable event for every heartbeat.

Store coalesced liveness metadata.

---

# 117. Progress

Optional:

```rust
pub struct JobProgress {
    pub phase: ProgressPhase,
    pub fraction: Option<BoundedRatio>,
}
```

Informational only.

---

# 118. Progress Is Not State Authority

A runner may report 100% before final output commit.

Job remains nonterminal until completion transaction.

---

# 119. Output Commit

Success requires:

```text
expected output refs
CAS verification
valid lease
current attempt
```

then transactional Job success.

---

# 120. UploadingOutputs State Benefit

Allows distinction between:

```text
command succeeded
but artifact upload failed
```

and actual successful job.

---

# 121. Output Upload Failure

Classify:

```text
Infrastructure / OutputUpload
```

Potential retry strategy:

```text
retry upload without rerun
```

if runner still retains verified local outputs.

---

# 122. Resume Output Commit

Could allow attempt-specific output finalization retry.

Do not create new attempt if no workload rerun needed.

---

# 123. Attempt Finalization Token

Optional:

```text
lease + attempt identity
```

authorizes output finalize within bounded grace.

---

# 124. Lease Expiry During Upload

Need explicit rule.

Recommended:

- once process completed and Attempt entered `UploadingOutputs`, lease can be extended by active heartbeat;
- if lease expires, attempt becomes Lost and late finalize rejected unless server issued dedicated finalize grace.

Simpler initial implementation:

```text
lease remains valid through output upload
```

---

# 125. Step-Level State

Do not persist every step as a full state machine initially unless product needs live step views/retries.

Recommended:

```text
Job is persistence authority
Step events/log spans are subordinate
```

---

# 126. Step Record

Optional:

```rust
pub struct StepExecution {
    pub step: StepNodeId,
    pub state: StepState,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
}
```

---

# 127. Step State

```text
Pending
Running
Succeeded
Failed
Skipped
Cancelled
```

---

# 128. Step Retry

Usually job-level.

Per-step retry can be supported later explicitly.

---

# 129. Run Supersession

Use case:

```text
new Change Proposal revision arrives
old CI run becomes obsolete
```

---

# 130. Supersede Policy

Can:

```text
cancel old run
mark old run Superseded
```

---

# 131. Superseded vs Cancelled

`Superseded` communicates:

```text
not user cancellation
newer equivalent work replaced this run
```

---

# 132. Supersede Active Jobs

Cancellation requested.

Run final state becomes:

```text
Superseded
```

after active work drains/stops.

---

# 133. Run Priority

State subsystem stores priority metadata.

Scheduler interprets.

---

# 134. Priority Change

Audited/versioned.

Does not change Run state.

---

# 135. Run Labels

Non-authoritative metadata.

---

# 136. Created vs Started Time

Run:

```text
created_at
started_at
finished_at
```

Job/attempt same.

---

# 137. Started Definition

Run `started_at` when first actual job/gate execution begins.

---

# 138. Finished Definition

Terminal state commit time.

---

# 139. Queue Duration

Derived:

```text
job leased_at - eligible_at
```

---

# 140. Execution Duration

Derived:

```text
attempt finished - started
```

---

# 141. Preparation Duration

Derived separately.

---

# 142. Accurate Performance Metrics

Persist phase timestamps enough for:

```text
queue
prepare
run
upload
```

analysis.

---

# 143. Job Phase Timestamps

Recommended fields:

```text
eligible_at
leased_at
preparing_at
started_at
uploading_at
finished_at
```

May live on attempt/history rather than current Job row.

---

# 144. Run Aggregate Service

```text
forgeyard-run-aggregate
```

Computes:

```text
run status
job counts
matrix group summaries
critical path progress
```

---

# 145. Aggregate Caching

Materialize current counters if UI scale requires.

Derived truth remains job states.

---

# 146. Run Progress

Possible:

```text
completed required jobs / total required jobs
```

Do not overclaim percentage for heterogeneous job durations.

---

# 147. Critical Path

Optional later analytics.

Not state authority.

---

# 148. Retry Semantics and Run State

Run remains:

```text
Running
```

while job is `RetryWaiting`.

---

# 149. No Active Job But Future Retry

Run is still Running/Queued-like.

Recommended:

```text
Running
```

once any execution started.

---

# 150. All Jobs Waiting for Retry

Run remains active.

---

# 151. All Jobs Pending Dependencies

Initial run:

```text
Queued
```

until first Job moves to Preparing/Running.

---

# 152. Eligible Jobs but No Runner

Run remains:

```text
Queued
```

if nothing started yet.

---

# 153. Run -> Running

When first job enters:

```text
Preparing
```

or `Running`.

Recommended:

```text
first attempt PreparationStarted
```

to reflect real resource use.

---

# 154. Run Completion Evaluator

Called after any terminal job/gate transition.

---

# 155. Completion Algorithm

Conceptually:

```text
if run cancelling and no active jobs:
    terminal cancellation state

else if required final failure exists:
    Failed

else if every required path terminal-success/skipped-allowed:
    Succeeded

else:
    remain active
```

---

# 156. Required Skip

Whether `Skipped` counts as acceptable depends on why.

ConditionFalse may be acceptable.

UpstreamFailed may contribute to Run failure already through upstream.

---

# 157. Skip Semantics

Define:

```rust
pub struct SkipOutcome {
    pub reason: SkipReason,
    pub satisfies_run: bool,
}
```

---

# 158. Dependency Evaluator

After Job state changes:

```text
find dependents
  ↓
evaluate dependency/condition
  ↓
Pending -> Eligible or Skipped
```

---

# 159. Avoid Recursive Transaction Explosion

Do not update entire downstream DAG recursively in one giant transaction.

Use:

```text
event/reconciliation
batched propagation
```

with idempotent transitions.

---

# 160. Immediate vs Reconciled Propagation

Fast path:

```text
update direct dependents
```

Reconciler ensures eventual correctness.

---

# 161. Reconciliation

```text
forgeyard-run-reconcile
```

Scans for impossible/stuck conditions.

---

# 162. Reconcile Cases

```text
eligible dependency but job still Pending
expired lease
attempt terminal but job nonterminal
run with all jobs terminal but run active
cancelled run with pending jobs still Pending
retry_waiting deadline elapsed
```

---

# 163. Reconciliation Principle

Reliability must not depend on every event handler firing exactly once.

---

# 164. Reconcile Job

Given current authoritative rows:

```text
derive expected legal state/action
```

and repair idempotently.

---

# 165. Reconcile Run

Recompute aggregate terminal conditions.

---

# 166. Startup Recovery

Daemon startup:

```text
scan/reconcile active runs
expired leases
stale attempts
retry_waiting deadlines
```

---

# 167. Standalone Crash Recovery

Same logic on Stoolap.

---

# 168. Distributed Failover

New daemon replica can continue using persisted state.

No in-memory-only authority.

---

# 169. Scheduler Crash

Leases remain persisted.

Replacement scheduler/daemon can inspect expiry.

---

# 170. Runner Crash

Lease eventually expires.

Attempt becomes Lost.

---

# 171. Network Partition

Runner may continue work while daemon unreachable.

Outcome handling depends on lease validity.

---

# 172. Long Partition

If lease expires and job re-leased, original runner's late result is stale.

---

# 173. Split Brain Defense

Only metadata authority can create valid lease/epoch.

---

# 174. Clock Skew

Lease expiry authority should use control-plane persisted/server time.

Runner local clock is advisory.

---

# 175. Monotonic Runner Timer

Runner can use local monotonic deadline for safe self-stop.

---

# 176. Retry After Lost

New attempt gets new:

```text
AttemptId
LeaseId
epoch
```

---

# 177. Attempt History

Never delete prior attempts for active/audit history under normal retention.

---

# 178. Attempt Failure Evidence

Link:

```text
logs
diagnostics
runner metadata
```

---

# 179. Runner Metadata Snapshot

Attempt records capability snapshot/reference at lease time if useful for debugging.

---

# 180. Toolchain/Plan Binding

Attempt executes immutable `PlannedJob`.

Record:

```text
plan/job spec digest
```

---

# 181. JobSpec Digest

```rust
pub struct JobSpecId(Digest);
```

Ensures runner executed intended spec.

---

# 182. Runner Ack

Runner acknowledges:

```text
JobSpecId
LeaseId
AttemptId
```

---

# 183. Spec Mismatch

Reject/abort if runner cannot interpret exact protocol/spec version.

---

# 184. Protocol Compatibility

Attempt state messages versioned by transport protocol.

---

# 185. Internal Command Idempotency

Every worker state message has:

```text
MessageId
AttemptId
LeaseId
```

---

# 186. Duplicate Start

Second `WorkStarted` for same attempt:

```text
idempotent no-op
```

if state already Running.

---

# 187. Out-of-Order Message

Example:

```text
UploadingOutputs arrives before WorkStarted
```

Reject unless protocol/reconciler can prove missing event but legal state.

Recommended:

```text
strict reject + runner resync
```

---

# 188. Runner Resync

Upon reconnect runner reports:

```text
active attempts
lease IDs
local phase
```

daemon responds:

```text
continue
cancel
stale
```

---

# 189. Reconnect Contract

Important for long builds.

---

# 190. Daemon Response

```rust
pub enum AttemptAuthority {
    Continue { lease_expires_at: Timestamp },
    Cancel,
    Stale,
}
```

---

# 191. Runner Drain

Draining runner stops accepting new leases.

Active attempts continue or cancel according to operator policy.

---

# 192. Runner Shutdown

Graceful:

```text
drain
finish/cancel active
disconnect
```

---

# 193. Forced Runner Loss

Attempts -> Lost on expiry/reconciliation.

---

# 194. Capacity Failure Before Lease

No attempt created if scheduler cannot place.

Job stays Eligible.

---

# 195. Preparation Failure

If runner cannot prepare due to local transient condition:

classify:

```text
SandboxSetup
ToolchainUnavailable
InputFetch
```

retry policy decides.

---

# 196. User Code Failure

Do not blame runner.

Failure class:

```text
UserCommand/TestFailure/CompileFailure
```

---

# 197. Infrastructure Retry Accounting

Policy may distinguish:

```text
workload attempts
infrastructure reschedules
```

---

# 198. Retry Budget Types

Possible:

```rust
pub struct RetryBudget {
    pub max_workload_attempts: u16,
    pub max_infrastructure_reschedules: u16,
}
```

More precise than one count.

---

# 199. Recommended Initial Simplicity

Use:

```text
max_attempts
+
failure predicate
```

first.

Add separate infra budget later if needed.

---

# 200. Timeout Retry

Configurable.

A timeout may be deterministic workload behavior or infrastructure problem.

---

# 201. Cancel Retry

Never auto-retry cancellation.

---

# 202. Policy Failure

Never auto-retry until policy/input changes.

---

# 203. Run Retry

"Retry run" should create:

```text
new RunId
```

unless product explicitly offers rerun failed jobs inside same run.

Recommended:

```text
new Run
```

for clean provenance.

---

# 204. Retry Failed Jobs Within Run

Could be offered as:

```text
manual new attempt
```

but complicates run immutability/history.

Recommended initial system:

```text
automatic retries inside run
manual rerun = new Run
```

---

# 205. Re-run Failed Only

New Run can use same Plan and mark unaffected jobs as reused/skipped based on explicit provenance if feature added later.

---

# 206. Reused Job Result

If implemented:

```rust
pub enum JobExecutionSource {
    Executed,
    ReusedFrom(JobId),
    CacheHit,
}
```

---

# 207. Action Cache Hit

Does a cache hit mean Job goes straight to Succeeded?

Potential flow:

```text
Eligible
  ↓
cache resolution
  ↓
Succeeded
```

without runner execution.

---

# 208. Cache Resolution State

To preserve clarity, cache check can happen during scheduling/planning.

No extra state required initially.

Transition:

```text
Eligible -> Succeeded
```

with reason:

```text
ActionCacheHit
```

if all required outputs verified.

---

# 209. Cache Hit Attempt

No JobAttempt needed if no runner executed.

Record:

```text
execution source = CacheHit
```

---

# 210. Cache Miss

Proceed to lease.

---

# 211. Cache Corrupt/Missing

Treat as miss + invalidate cache mapping.

---

# 212. Gate Satisfaction

Similarly no runner attempt.

Job/gate-specific state subsystem should distinguish non-execution node.

---

# 213. Node Kinds

Runtime node:

```rust
pub enum RuntimeNodeKind {
    Job,
    Gate,
}
```

Potential future:

```text
deployment gate
approval
```

---

# 214. Simpler Model

Keep `Job` only for executable nodes.

Use separate Gate records.

Recommended.

---

# 215. Run Graph Runtime

```rust
pub struct RunGraph {
    pub jobs: BTreeMap<JobNodeId, JobId>,
    pub gates: BTreeMap<GateNodeId, GateId>,
}
```

---

# 216. Gate Service

Separate from runner scheduler.

---

# 217. Job Dependency Types

Runtime dependency target may be:

```text
Job
Gate
```

---

# 218. Run Graph Identity

Derived from `PipelinePlanId`.

---

# 219. Run Creation

Transaction:

```text
create Run
create Job rows
create Gate rows
create dependency mapping
mark initial eligible/skipped nodes
append RunCreated
```

Could be large.

---

# 220. Large DAG Creation

Batch inserts.

Avoid one SQL statement per job.

---

# 221. Run Creation Idempotency

Use:

```text
RunRequestId / IdempotencyKey
```

---

# 222. Initial Eligibility

Jobs with:

```text
no dependencies
condition true
```

become Eligible.

Plan-time false jobs can be created Skipped or omitted from runtime graph.

---

# 223. Omitted vs Skipped

For audit/UI, recommended:

```text
create Skipped job
```

if part of canonical plan but pruned by runtime context.

Plan-time permanently pruned nodes may be omitted with plan diagnostics.

---

# 224. Job Record Fields

Suggested:

```rust
pub struct JobRecord {
    pub id: JobId,
    pub run: RunId,
    pub node: JobNodeId,
    pub spec: JobSpecId,
    pub state: JobState,
    pub current_attempt: Option<JobAttemptId>,
    pub attempt_count: u16,
    pub retry_not_before: Option<Timestamp>,
    pub required: bool,
    pub failure_impact: FailureImpact,
    pub version: EntityVersion,
}
```

---

# 225. Run Record Fields

```rust
pub struct RunRecord {
    pub id: RunId,
    pub project: ProjectId,
    pub plan: PipelinePlanId,
    pub source: SourceSnapshotId,
    pub state: RunState,
    pub cancel_requested_at: Option<Timestamp>,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub version: EntityVersion,
}
```

---

# 226. Attempt Record Fields

```rust
pub struct JobAttemptRecord {
    pub id: JobAttemptId,
    pub job: JobId,
    pub number: AttemptNumber,
    pub runner: Option<RunnerId>,
    pub lease: Option<LeaseId>,
    pub state: AttemptState,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub failure: Option<FailureClass>,
}
```

---

# 227. Lease Record Fields

```rust
pub struct JobLeaseRecord {
    pub id: LeaseId,
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub runner: RunnerId,
    pub epoch: LeaseEpoch,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 228. Current Lease Pointer

Job may store current lease/attempt reference for fast validation.

---

# 229. Unique Constraints

Examples:

```text
(job_id, attempt_number) unique
active lease per job unique
attempt_id unique
```

---

# 230. Terminal Immutability Constraint

Service/domain ensures no terminal transition out.

DB may reinforce where practical.

---

# 231. Run State Transition Table

```text
Queued -> Running
Queued -> Cancelling
Queued -> Cancelled
Queued -> Superseded

Running -> Cancelling
Running -> Succeeded
Running -> Failed
Running -> TimedOut
Running -> Superseded

Cancelling -> Cancelled
Cancelling -> TimedOut
Cancelling -> Superseded

terminal -> no transitions
```

---

# 232. `TimedOut` Path

Run-level timeout can move:

```text
Running -> Cancelling
```

with terminal intent `TimedOut`, then final:

```text
TimedOut
```

---

# 233. Terminal Intent

Useful type:

```rust
pub enum RunTerminationIntent {
    Cancelled,
    TimedOut,
    Superseded,
}
```

Stored while draining.

---

# 234. Why Terminal Intent

`Cancelling` alone loses why active work is stopping.

---

# 235. Job Cancellation Intent

Similarly:

```rust
pub enum JobTerminationIntent {
    Cancelled,
    TimedOut,
    Superseded,
    FailFast,
}
```

---

# 236. Active Cancellation State

Could avoid adding `Cancelling` to Job state by storing intent separately.

Recommended:

```text
Job remains current phase
+
termination_intent
```

until runner confirms.

This avoids many duplicated states:

```text
RunningCancelling
PreparingCancelling
```

---

# 237. Job Model With Intent

```rust
pub struct JobRecord {
    pub state: JobState,
    pub termination_intent: Option<JobTerminationIntent>,
    ...
}
```

---

# 238. Command Acceptance

Once termination intent exists:

```text
new WorkStarted
```

may be rejected depending current phase.

---

# 239. Already Finished Race

If terminal before intent transaction:

terminal outcome wins.

---

# 240. Step Logs After Cancel

May arrive late; accepted as diagnostics only if attempt known.

---

# 241. Final Result After Cancel

If authoritative cancel intent already persisted, successful workload exit may still final as Cancelled unless output commit had already succeeded.

Explicit transaction ordering decides.

---

# 242. State Machine Determinism

Given:

```text
current state
event
current intent
attempt/lease authority
policy
```

next state is deterministic.

---

# 243. No Hidden State

Do not derive legal transition from mutable external facts without including them in command context.

---

# 244. Policy Version

Retry/failure-impact policy from Plan is immutable for Run.

Do not let mid-run config edits change job retry semantics.

---

# 245. Run Policy Snapshot

Store Plan/Policy IDs.

---

# 246. Scheduler Receives Eligible Jobs

State subsystem exposes efficient query:

```text
eligible now
retry_not_before <= now
no termination intent
```

---

# 247. Scheduler Claim

Atomic lease grant removes from competing scheduler claim.

---

# 248. Multiple Daemon Replicas

Optimistic concurrency/transaction uniqueness prevents duplicate authority.

---

# 249. DB `SKIP LOCKED`

May optimize eligible claim, but semantics live in store API.

---

# 250. Run Events Outbox

Transition transaction may append event/outbox.

---

# 251. UI Updates

WebSocket/SSE consumes run/job events.

UI does not poll authoritative DB directly.

---

# 252. Event Loss

UI can reconnect and backfill from event/log sequence or query current state.

---

# 253. Notification Hooks

Failures/completion events feed notification subsystem.

---

# 254. Change Proposal Checks

CheckRun maps to Run/Job state.

Proposal check status should use exact Run/Plan/Snapshot.

---

# 255. Required Check Completion

When Run terminal, Change Proposal check aggregator reevaluates.

---

# 256. Release Pipeline

Same Run/Job state machinery.

No separate release execution engine.

---

# 257. Deployment Jobs

Deployment workflow may use same job mechanics plus deployment state subsystem.

---

# 258. Device Jobs

Job capability requires device lease, but Job state remains same.

---

# 259. Nested Lease

A Job can hold:

```text
runner lease
device lease
```

separately.

---

# 260. Resource Reservation

Scheduler may reserve resources as part of lease.

State subsystem records lease identity, not resource accounting logic.

---

# 261. JobAttempt Sandbox Identity

Record:

```text
SandboxId
```

optional for audit/debug.

---

# 262. Attempt Worker Session

Record runner session/agent instance identity to detect restarts.

---

# 263. Runner Restart

Same RunnerId but new AgentSessionId.

Old attempts may be lost unless restored/reported.

---

# 264. Agent Session

```rust
pub struct AgentSessionId(Ulid);
```

---

# 265. Lease Binding

Lease can bind:

```text
RunnerId + AgentSessionId
```

to prevent restarted agent from accidentally inheriting old attempt.

---

# 266. Recommended Lease Record

```rust
pub struct JobLease {
    pub id: LeaseId,
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub runner: RunnerId,
    pub agent_session: AgentSessionId,
    pub epoch: LeaseEpoch,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 267. Reconnect Same Session

Can continue if lease valid.

---

# 268. New Session

Must explicitly recover/reclaim only through daemon protocol.

Default:

```text
old lease considered suspect
```

---

# 269. Durable Local Runner State

Agent may persist active attempt metadata locally for crash recovery.

Control plane remains authority.

---

# 270. Attempt Recovery

If agent restarts quickly:

```text
report persisted attempt
daemon validates active lease/session recovery policy
```

Could continue if supported.

Initial implementation may mark lost for simplicity.

---

# 271. Production Evolution

Start:

```text
agent restart -> attempt lost
```

Later add safe process/sandbox reattachment if platform supports.

---

# 272. Job Timeouts During Daemon Downtime

Runner self-enforces configured deadline.

On reconnect reports timeout result.

Control plane reconciles.

---

# 273. Job Deadline

Send absolute + duration semantics safely.

Runner uses local monotonic timer initialized on lease/start.

---

# 274. Cancellation Delivery Failure

If daemon cannot reach runner:

```text
termination intent persisted
lease not renewed
runner eventually self-stops/lease expires
```

---

# 275. Hard Safety Lease

Runner should not continue indefinitely without lease renewal in distributed mode unless job policy explicitly allows disconnected execution.

---

# 276. Disconnected Execution Mode

Possible for remote/offline edge jobs:

```rust
pub enum LeaseConnectivityPolicy {
    Continuous,
    Grace(Duration),
    OfflineAuthorized(Duration),
}
```

---

# 277. Default

Distributed:

```text
Continuous/short grace
```

Air-gap/local standalone:

```text
different authority model
```

---

# 278. Standalone Lease

Still useful for internal consistency, though daemon/runner same process.

---

# 279. Standalone Optimization

Can use local in-process scheduling while persisting same states.

---

# 280. Cross-Mode Invariant

Same Job state machine in standalone and distributed modes.

---

# 281. Audit Detail

Security-sensitive transitions:

```text
manual cancellation
retry override
priority override
forced completion/repair
```

must be audited.

---

# 282. Admin Repair

Do not expose:

```text
set job status = success
```

casually.

Provide controlled repair actions.

---

# 283. Repair Command

```text
forgeyard run repair
```

generates plan:

```text
detect inconsistency
proposed transition
reason
audit
```

---

# 284. Forced Terminalization

Rare:

```text
operator marks irrecoverable lost run failed
```

requires privileged command/audit.

---

# 285. State Corruption

Invalid DB state combination:

```text
Job Running but no Attempt
```

reconciler reports `InvariantViolation`.

---

# 286. Invariant Examples

1. `Leased` requires current lease.
2. `Preparing` requires current attempt/lease.
3. `Running` requires current attempt/lease.
4. `UploadingOutputs` requires current attempt.
5. `Succeeded` requires result/output commit.
6. terminal Job has no active lease.
7. Attempt number unique/monotonic.
8. active lease attempt equals Job current attempt.

---

# 287. Run Invariants

1. terminal Run has no active required jobs.
2. Succeeded Run has no blocking failed required job.
3. Cancelled Run had cancellation/supersession intent or equivalent reason.
4. Plan/source IDs immutable for lifetime.

---

# 288. Attempt Invariants

1. terminal attempt cannot change terminal outcome.
2. attempt belongs to one job.
3. lease belongs to one attempt.
4. result belongs to same attempt/spec.

---

# 289. Completion Command

```rust
pub struct CompleteAttempt {
    pub run: RunId,
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub runner: RunnerId,
    pub spec: JobSpecId,
    pub result: JobAttemptResult,
    pub message_id: MessageId,
}
```

---

# 290. Completion Validation

Check:

```text
job current attempt matches
lease current/valid
runner matches
spec matches
job phase legal
termination intent
output refs valid
message id not already processed
```

---

# 291. Completion Transaction

```text
BEGIN
validate versions/lease
terminalize attempt
terminalize or retry job
release lease
append event
update dependents marker/outbox
COMMIT
```

---

# 292. Dependency Propagation Outside Transaction

Can append event:

```text
JobTerminal
```

then dependency evaluator processes.

Reconciler catches missed event.

---

# 293. Retry Transaction

If retryable:

```text
Attempt -> Failed/Lost
Job -> RetryWaiting
retry_not_before = ...
lease removed
```

---

# 294. Retry Eligibility Event

Timer/reconciler:

```text
RetryWaiting + now >= retry_not_before
  -> Eligible
```

---

# 295. Durable Timers

Do not rely only on in-memory timers.

Persist deadline/timestamp.

---

# 296. Timer Wheel Optimization

In-memory timing wheel can wake jobs efficiently.

Persistence/reconciler remains correctness fallback.

---

# 297. Run Deadline Persistence

```text
run_deadline
```

stored.

---

# 298. Job Deadline Persistence

Can store derived/current attempt deadline.

---

# 299. Lease Expiry Index

DB index on:

```text
active leases expires_at
```

---

# 300. Retry Waiting Index

Index:

```text
state = RetryWaiting, retry_not_before
```

---

# 301. Active Run Index

Index:

```text
RunState active
```

for recovery/reconcile.

---

# 302. Event Sequence

Per run:

```rust
pub struct RunEventSeq(u64);
```

Optional useful for UI backfill/order.

---

# 303. Global Event ID

Still use `EventId`.

---

# 304. Per-Run Ordering

Transactionally increment run event sequence if needed.

---

# 305. Event Storage Cost

Do not append heartbeat/progress every second to durable event history.

---

# 306. Significant Events Only

Persist:

```text
state transitions
attempt creation
lease grant/loss
retry scheduled
cancellation intent
terminal outcomes
```

---

# 307. Progress Sampling

Transient stream or coalesced storage.

---

# 308. Logs

Separate log data plane.

Job state only stores LogStreamId.

---

# 309. Step Events

May be streamed/UI-history but not full critical state machine.

---

# 310. CLI

```text
forgeyard run list
forgeyard run show
forgeyard run cancel
forgeyard run retry
forgeyard run events
forgeyard run graph
forgeyard run explain

forgeyard job show
forgeyard job attempts
forgeyard job logs
forgeyard job cancel
forgeyard job explain
```

---

# 311. `run show`

Displays:

```text
RunId
PlanId
SourceSnapshotId
state
created/started/finished
job aggregate
active retries
```

---

# 312. `job show`

Displays:

```text
state
current attempt
lease
runner
resources
capabilities
attempt history
failure
outputs
```

---

# 313. `job attempts`

Timeline:

```text
#1 runner A Lost
#2 runner B Failed Infrastructure
#3 runner C Succeeded
```

---

# 314. Explain Job State

```text
why Pending
why Skipped
why RetryWaiting
why Failed
```

---

# 315. Dioxus UI

Run page:

```text
Overview
Graph
Jobs
Logs
Artifacts
Attempts
Timeline
Diagnostics
```

---

# 316. Job Row

Show:

```text
state
attempt
runner
queue time
duration
retry
```

---

# 317. Retry Visualization

Clearly distinguish:

```text
attempt failed
job retrying
```

from final failure.

---

# 318. Lost Visualization

Show infrastructure/liveness issue, not workload red by default.

---

# 319. Run Graph

Runtime states overlay Pipeline DAG.

---

# 320. Timeline

State transitions in sequence.

---

# 321. API

Potential:

```text
GET  /v1/runs/{id}
GET  /v1/runs/{id}/jobs
GET  /v1/runs/{id}/events
POST /v1/runs/{id}/cancel

GET  /v1/jobs/{id}
GET  /v1/jobs/{id}/attempts
POST /v1/jobs/{id}/cancel
```

---

# 322. Worker Internal API

Not public REST.

QUIC/Postcard messages:

```text
lease
accept
heartbeat
phase change
completion
failure
```

---

# 323. Security

Threats:

```text
stale worker success injection
duplicate completion
fake runner result
lease replay
cross-tenant job access
unauthorized cancel
operator status forgery
```

---

# 324. Lease Replay Defense

Lease ID + attempt + runner/session + expiry.

---

# 325. Fake Completion Defense

Authenticated runner channel + lease validation.

---

# 326. Cross-Tenant Defense

Run/Job access scoped by project/tenant.

---

# 327. Cancel Authorization

API authz checks:

```text
run.cancel
```

---

# 328. Worker Cannot Cancel Arbitrarily

Runner can report local failure/shutdown; control plane decides state.

---

# 329. Manual Success Override

Forbidden by default.

---

# 330. Observability Metrics

```text
runs_created
runs_succeeded
runs_failed
run_duration
jobs_eligible
jobs_leased
job_queue_duration
job_prepare_duration
job_execution_duration
job_upload_duration
job_retries
job_lost
lease_expired
stale_completion_rejected
```

---

# 331. Metrics Dimensions

Safe low-cardinality:

```text
project class
job kind
failure class
platform
```

Avoid JobId labels.

---

# 332. Tracing

Spans:

```text
run.create
job.eligible
job.lease
attempt.prepare
attempt.run
attempt.upload
attempt.complete
job.retry
run.complete
run.reconcile
```

---

# 333. Alerting

Alert on:

```text
high lost-attempt rate
lease expiry spike
stale completion spike
reconcile backlog
runs stuck active
```

---

# 334. Reconcile Metrics

```text
reconcile_repairs
reconcile_invariant_failures
```

---

# 335. Store Integration

Uses `JobStore`, `RunStore`, `LeaseStore`, `AttemptStore`.

---

# 336. Metadata/CAS Boundary

Run/Job metadata store references:

```text
artifacts
logs
reports
```

CAS stores bytes.

---

# 337. No CAS in State Transaction

Do not upload bytes inside SQL transaction.

---

# 338. Output Validation Before Completion Transaction

CAS refs verified before final state commit.

---

# 339. Storage Failure During Completion

If DB unavailable after upload:

```text
runner retries completion
```

idempotently while lease valid/grace.

---

# 340. Completion Retry

Same `MessageId` returns prior result if committed.

---

# 341. Unknown Commit Result

Runner may not know if DB commit succeeded due to network loss.

Retry same command safely.

---

# 342. At-Least-Once Worker Protocol

Assume all worker messages may repeat.

---

# 343. State Machine Testkit

```text
forgeyard-run-testkit/src/
├── lib.rs
├── run_builder.rs
├── job_builder.rs
├── attempt_builder.rs
├── lease_builder.rs
├── transitions.rs
├── retry.rs
├── timeout.rs
├── cancellation.rs
└── reconciliation.rs
```

---

# 344. Unit Tests

Every legal transition.

Every illegal transition.

---

# 345. Transition Table Test

Generate all:

```text
state × event
```

and assert expected result/rejection.

---

# 346. Property Tests

Properties:

```text
terminal states never leave terminal
attempt numbers monotonically increase
only current lease can complete
retry never reuses AttemptId
```

---

# 347. Fuzzing

Fuzz random event sequences.

Assert:

```text
no invariant broken
```

---

# 348. Model-Based Testing

Maintain reference pure state machine and compare store/service implementation.

---

# 349. Concurrency Tests

Simulate:

```text
two schedulers lease same job
cancel vs success
timeout vs success
two duplicate completions
```

---

# 350. Failure Injection

```text
daemon crash after lease commit
runner crash during upload
DB outage during completion
event publish failure
```

---

# 351. Restart Test

Persist active run, kill daemon, restart, reconcile to correct state.

---

# 352. Standalone Crash Test

Same with Stoolap.

---

# 353. Distributed Test

Multiple daemon replicas contend for same eligible jobs.

Exactly one authoritative lease.

---

# 354. Late Completion Test

Attempt 1 lost, Attempt 2 succeeds, Attempt 1 late success rejected.

---

# 355. Cancellation Test

Cancel at every active phase.

---

# 356. Timeout Test

Timeout at:

```text
Preparing
Running
UploadingOutputs
```

---

# 357. Retry Test

Retryable infra failure schedules new attempt.

---

# 358. Non-Retry Test

Compile failure finalizes Job Failed when policy says no retry.

---

# 359. Dependency Test

Upstream failure skips required downstream.

---

# 360. Continue-on-Error Test

Allowed failure does not fail Run.

---

# 361. Fail-Fast Test

Failure cancels siblings according to policy.

---

# 362. Cache Hit Test

Eligible -> Succeeded without Attempt.

---

# 363. Event Idempotency Test

Duplicate completion emits one semantic terminal transition.

---

# 364. Production Readiness Gates

Do not call this subsystem production-ready until:

```text
state tables frozen/versioned
all legal transitions tested
illegal transitions rejected
attempt/lease authority correct
stale completion rejection proven
retry semantics deterministic
cancellation races tested
timeout races tested
dependency propagation reconciled
crash recovery tested
duplicate message idempotency tested
metrics/audit available
```

---

# 365. Implementation Phase 1 — Pure Models

Implement:

```text
RunState
JobState
AttemptState
FailureClass
SkipReason
TerminationIntent
transition functions
```

No DB yet.

---

# 366. Phase 2 — Store APIs

Implement:

```text
RunStore
JobStore
AttemptStore
LeaseStore
versioned transition persistence
```

---

# 367. Phase 3 — Run Creation

Create Run + Jobs + dependency mapping from `ExecutablePlan`.

---

# 368. Phase 4 — Eligibility / Dependency Engine

Implement:

```text
initial eligibility
downstream propagation
skip behavior
```

---

# 369. Phase 5 — Attempts / Leases

Implement atomic attempt+lease creation and validation.

---

# 370. Phase 6 — Runner Phase Updates

Implement:

```text
Preparing
Running
UploadingOutputs
Completion
```

---

# 371. Phase 7 — Retry / Timeout / Cancellation

Implement explicit policies and durable deadlines.

---

# 372. Phase 8 — Reconciliation

Implement active-run and lease recovery.

---

# 373. Phase 9 — Events / UI/API

Expose timeline/state updates.

---

# 374. Phase 10 — Hardening

Concurrency, fuzzing, failure injection, multi-daemon tests.

---

# 375. Acceptance Tests

1. Run creation is idempotent.
2. Job starts Pending/Eligible correctly from DAG.
3. Only Eligible job can receive a lease.
4. Lease creation and Attempt creation are atomic.
5. Two schedulers cannot create two authoritative leases.
6. Runner cannot start without current lease.
7. Preparing -> Running transition is explicit.
8. Running -> UploadingOutputs is explicit.
9. Job cannot become Succeeded before verified output commit.
10. Retry creates new AttemptId.
11. Retry never reuses old LeaseId.
12. Lost attempt can retry.
13. Exhausted retry finalizes Job Failed.
14. Compile failure is not retried by default.
15. Infrastructure failure can retry.
16. Cancelled Job is not reported Failed.
17. Run cancellation propagates.
18. Run timeout terminates active work.
19. Stale heartbeat cannot renew new lease.
20. Stale completion cannot overwrite current attempt.
21. Duplicate completion is idempotent.
22. DB outage after CAS upload can be retried safely.
23. Upstream required failure blocks/skips dependent.
24. Continue-on-error failure does not fail Run.
25. Fail-fast cancels siblings only when configured.
26. Cache hit can complete without attempt.
27. Daemon restart reconciles active run.
28. Runner crash leads to Lost then retry/failure.
29. Run terminal state never reopens.
30. Terminal attempt never changes outcome.
31. Same state machine works on Stoolap and Postgres.
32. UI/API cannot directly assign state.
33. Audit records manual cancellation/override.
34. State transition history can reconstruct execution timeline.
35. Forgeyard self-hosting pipeline can use this exact model.

---

# 376. Architectural Invariants

1. Run, Job, Attempt, and Lease are distinct identities.
2. Job is logical work; Attempt is one execution.
3. Retry always creates a new Attempt.
4. Lease binds Job + Attempt + Runner + session/epoch.
5. Only current authoritative lease can mutate active attempt.
6. Late/stale completion never changes current Job.
7. Terminal states do not reopen.
8. Success requires output/evidence commit.
9. CAS upload alone does not mean Job success.
10. Job state transitions are validated, not assigned.
11. State transition + event are persisted atomically where required.
12. Duplicate messages are idempotent.
13. Worker protocol is at-least-once safe.
14. Cancellation is distinct from failure.
15. Timeout is distinct from cancellation/failure.
16. Lost is distinct from workload failure.
17. Retry policy is immutable for Run/Plan.
18. Run aggregate is derived from Job outcomes + run intent.
19. Dependency propagation is reconcilable.
20. In-memory timers are optimizations; deadlines are persisted.
21. Heartbeats are not durable event spam.
22. Agents never become state authority.
23. UI never becomes state authority.
24. Scheduler requests leases; state/store layer authorizes them.
25. Same model runs in standalone and distributed modes.
26. DB restore/restart requires reconciliation before assuming active work.
27. Manual state repair is privileged/audited.
28. Attempts preserve execution history.
29. Plan/source/spec identities are immutable for a Run.
30. Forgeyard itself should dogfood this state machine.

---

# 377. Final Target Architecture

```text
                    ExecutablePlan
                         │
                         ▼
                        Run
                         │
                         ▼
                     Job Graph
                         │
                  dependencies/gates
                         │
                         ▼
                 Pending / Eligible
                         │
                         ▼
                     Scheduler
                         │
                         ▼
                Attempt + Lease
                         │
                         ▼
                     Runner
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      Preparing        Running      UploadingOutputs
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                    Attempt Result
                         │
                 validate authority
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         Success       Retry        Final Failure
            │            │            │
            ▼            ▼            ▼
        JobSucceeded  Eligible      JobFailed
            │
            ▼
      dependency propagation
            │
            ▼
        Run aggregation
```

---

# 378. Final Architectural Position

Run creation:

```text
ExecutablePlan
  ↓
Run
+ Job records
+ dependency graph
```

Execution authority:

```text
Eligible Job
  ↓
Attempt
+ Lease
  ↓
Runner
```

Completion authority:

```text
JobId
+ AttemptId
+ LeaseId
+ Runner/Session
+ JobSpecId
+ verified outputs
  ↓
validated transition
```

Retry:

```text
Attempt N failed/lost
  ↓
Job RetryWaiting
  ↓
backoff elapsed
  ↓
Eligible
  ↓
Attempt N+1
```

Recovery:

```text
persisted state
+ leases
+ deadlines
+ attempt records
  ↓
reconciler
  ↓
correct active state
```

The key guarantee is:

> **Forgeyard never trusts a worker merely because it says a job succeeded. It accepts a result only when the exact current attempt, lease, runner authority, plan/spec identity, and required outputs all still match the persisted execution state.**

---

# 379. New-Repository Sequence

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
