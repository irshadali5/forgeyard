# 60 — Forgeyard Workflow Concurrency, Distributed Locks, Idempotency Keys, Reservations & Exclusive Resource Coordination System Architecture

**Document type:** Core Workflow Concurrency, Distributed Locking, Idempotency, Reservation, Exclusive Resource Coordination, Lease/Fencing & Contention Management System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** concurrency groups, distributed locks, environment/release exclusivity, idempotency keys, reservations, ownership leases, fencing tokens, leader/controller failover, deadlock avoidance, stale-owner rejection, queue serialization, resource contention, conflict detection, and exclusive external-effect governance  
**Architecture style:** Explicit ownership, versioned leases, fencing, idempotent command semantics, expected-version checks, short-lived reservations, deterministic conflict scopes, no business truth hidden in locks, and reconciliation after ambiguous external effects  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Run/Job State Machine, Scheduler, Events/Reconciliation, HA/Raft, Release, Deployment, Triggers, Infrastructure-as-Code, Merge Queue, Federation, Runner Fleet, Device Lab, API Idempotency, and Security. This subsystem makes cross-cutting concurrency semantics explicit instead of scattering ad-hoc mutexes across services.

---

# 1. Purpose

Forgeyard performs many operations that must not overlap arbitrarily:

```text
two deployments to the same environment
two infrastructure applies against the same state
two release publishers mutating the same channel
two merge controllers submitting the same target
two jobs leasing the same device
two migrations applying against the same database
two scheduled runs inside a single-concurrency group
two workers processing the same external side-effect intent
```

Without a unified architecture, systems drift toward:

```text
process-local mutexes
database boolean flags
"running = true"
long-lived locks without fencing
blind retries
duplicate provider effects
ad-hoc Redis locks
```

Those patterns are fragile across:

```text
process crashes
network partitions
leader failover
database failover
provider timeouts
multi-region operation
```

The central rule is:

> **A lock coordinates temporary ownership; it never becomes the canonical truth of the protected business object.**

A second rule is:

> **Every exclusive mutable operation that can outlive its coordinator uses a fencing epoch/token or equivalent expected-version check so stale owners cannot continue after failover.**

A third rule is:

> **Idempotency controls duplicate intent; reconciliation controls ambiguous external effects. Neither one removes the need for the other.**

---

# 2. Architectural Position

```text
                    Command / Intent
                          │
                          ▼
                    Idempotency Check
                          │
                          ▼
                  Conflict Scope Resolve
                          │
                          ▼
                   Lease / Reservation
                          │
                          ▼
                 Canonical State Transition
                          │
                          ▼
                   External Side Effect
                          │
                          ▼
                      Observe
                          │
                          ▼
                    Reconcile / Commit
```

---

# 3. Goals

The subsystem MUST:

1. define concurrency-scope identity;
2. define exclusive-resource identity;
3. define reservation identity;
4. define lease identity;
5. define fencing tokens;
6. define idempotency-key semantics;
7. support workflow concurrency groups;
8. support environment exclusivity;
9. support release exclusivity;
10. support infrastructure-state exclusivity;
11. support merge-target exclusivity;
12. support device/resource reservations;
13. support bounded lease expiry;
14. support stale-owner rejection;
15. support controller failover;
16. support retry after coordinator crash;
17. support deadlock prevention;
18. support fairness;
19. support cancellation/supersession;
20. support provider ambiguity;
21. support HA;
22. support multi-region authority;
23. support audit;
24. support UI/API/CLI;
25. support observability;
26. support load/backpressure;
27. support transactional acquisition where required;
28. remain independent of a specific lock backend;
29. never replace business-state versioning;
30. never rely on process-local locking for distributed correctness.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Scheduler
replace Postgres transactions
replace Raft
replace Deployment state
replace Release state
replace provider-side resource locks
replace normal optimistic concurrency
```

It standardizes when and how those mechanisms coordinate.

---

# 5. Workspace Structure

```text
crates/concurrency/
├── forgeyard-concurrency/
├── forgeyard-concurrency-model/
├── forgeyard-concurrency-idempotency/
├── forgeyard-concurrency-lease/
├── forgeyard-concurrency-reservation/
├── forgeyard-concurrency-fencing/
├── forgeyard-concurrency-group/
├── forgeyard-concurrency-conflict/
├── forgeyard-concurrency-reconcile/
├── forgeyard-concurrency-health/
└── forgeyard-concurrency-testkit/
```

Adapters:

```text
crates/concurrency-adapters/
├── forgeyard-concurrency-postgres/
├── forgeyard-concurrency-stoolap/
└── forgeyard-concurrency-custom/
```

Postgres is the Mode 2 baseline.

Stoolap supports Mode 1 local semantics.

---

# 6. ConcurrencyScopeId

```rust
pub struct ConcurrencyScopeId(Digest);
```

Canonical identity for a scope within which operations conflict.

Examples:

```text
environment:prod
release-channel:stable
infra-env:payments-prod
merge-target:repo/main
device:pixel-8-01
migration:tenant-db-42
```

---

# 7. Scope Must Be Typed

Avoid:

```text
"prod"
"main"
"foo-lock"
```

Prefer:

```rust
pub enum ConcurrencyScope {
    DeploymentEnvironment(EnvironmentId),
    InfrastructureEnvironment(InfrastructureEnvironmentId),
    MergeTarget(MergeQueueId),
    ReleaseChannel(ReleaseChannelId),
    Device(DeviceId),
    RunnerFleet(RunnerFleetId),
    WorkflowGroup(WorkflowConcurrencyGroupId),
    DatabaseMigration(DatabaseId),
    Custom(CustomConcurrencyScopeId),
}
```

---

# 8. Scope Resolution

Concurrency scope is computed before acquisition.

---

# 9. No Hidden Dynamic Lock Name

Critical.

---

# 10. Concurrency Group

Pipeline-level semantics.

```rust
pub struct WorkflowConcurrencyGroupId(Digest);
```

---

# 11. WorkflowConcurrencySpec

```rust
pub struct WorkflowConcurrencySpec {
    pub group: WorkflowConcurrencyGroupId,
    pub mode: WorkflowConcurrencyMode,
}
```

---

# 12. WorkflowConcurrencyMode

```rust
pub enum WorkflowConcurrencyMode {
    AllowParallel,
    Queue,
    CancelPrevious,
    RejectNew,
}
```

---

# 13. AllowParallel

No exclusivity.

---

# 14. Queue

Serialize by group.

---

# 15. CancelPrevious

New intent supersedes older active intent where policy allows.

---

# 16. RejectNew

Useful for high-risk exclusive operations.

---

# 17. Release/Deploy

`CancelPrevious` may be forbidden.

---

# 18. Concurrency Group Subject

Can derive from:

```text
project
branch
environment
component
custom typed key
```

---

# 19. Dynamic Expression

Must compile deterministically during planning.

---

# 20. No Secret Value in Group Key

Critical.

---

# 21. LeaseId

```rust
pub struct LeaseId(Ulid);
```

---

# 22. Lease

```rust
pub struct Lease {
    pub id: LeaseId,
    pub scope: ConcurrencyScopeId,
    pub holder: LeaseHolder,
    pub epoch: FencingToken,
    pub acquired_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 23. Lease Holder

```rust
pub enum LeaseHolder {
    Run(RunId),
    JobAttempt(JobAttemptId),
    Controller(ControllerId),
    Deployment(DeploymentId),
    Release(ReleaseId),
    InfrastructureApply(InfrastructureApplyId),
    QueueSubmission(SubmissionRequestId),
}
```

---

# 24. FencingToken

```rust
pub struct FencingToken(u64);
```

Monotonically increasing per scope.

---

# 25. Why Fencing

Scenario:

```text
Controller A acquires lease token 7
A stalls
lease expires
Controller B acquires token 8
A wakes up
```

Without fencing, A may still mutate external/internal state.

With fencing:

```text
token 7 < current token 8
=> reject stale mutation
```

---

# 26. Fencing Enforcement

At every protected mutable boundary that can validate token.

---

# 27. Internal DB Write

Include expected fencing token/version.

---

# 28. External Provider

Use provider-side expected version/idempotency/precondition if available.

---

# 29. Provider Without Fencing Primitive

Reconcile carefully and keep authority narrow.

---

# 30. Lock Alone Is Not Enough

Critical.

---

# 31. Lease Duration

Bounded.

---

# 32. Renewal

Explicit heartbeat.

---

# 33. Lease Renewal

```rust
pub struct LeaseRenewal {
    pub lease: LeaseId,
    pub expected_epoch: FencingToken,
    pub extend_to: Timestamp,
}
```

---

# 34. Stale Renewal

Rejected.

---

# 35. Lease Expiry

Does not automatically mean underlying operation never happened.

---

# 36. Reconciliation Required

Critical.

---

# 37. ReservationId

```rust
pub struct ReservationId(Ulid);
```

---

# 38. Reservation vs Lease

Reservation means:

```text
resource/time/capacity is promised
```

Lease means:

```text
holder currently owns operational right
```

---

# 39. Example

Device:

```text
reservation created for upcoming job
  ↓
job starts
  ↓
lease activated
```

---

# 40. Reservation Model

```rust
pub struct Reservation {
    pub id: ReservationId,
    pub resource: ReservableResource,
    pub owner: ReservationOwner,
    pub starts_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 41. ReservableResource

```rust
pub enum ReservableResource {
    Device(DeviceId),
    RunnerCapacity(CapacityClassId),
    Environment(EnvironmentId),
    TestAccount(TestAccountId),
    Custom(ResourceReservationId),
}
```

---

# 42. Reservations Are Not Execution Authority

Scheduler/resource manager validates.

---

# 43. IdempotencyKey

```rust
pub struct IdempotencyKey(BoundedString);
```

---

# 44. Idempotency Domain

Key meaning is scoped.

```rust
pub struct IdempotencyScope {
    pub tenant: TenantId,
    pub operation: OperationKind,
    pub subject: IdempotencySubject,
}
```

---

# 45. Idempotency Record

```rust
pub struct IdempotencyRecord {
    pub scope: IdempotencyScope,
    pub key: IdempotencyKey,
    pub request_digest: Digest,
    pub outcome: IdempotencyOutcome,
}
```

---

# 46. Request Digest

Same key + different request = conflict.

---

# 47. Critical Rule

Never silently reuse an idempotency key for semantically different request.

---

# 48. IdempotencyOutcome

```rust
pub enum IdempotencyOutcome {
    InProgress,
    Succeeded(ResponseRef),
    FailedTerminal(ErrorRef),
    Unknown,
}
```

---

# 49. InProgress

Duplicate caller observes same intent.

---

# 50. Succeeded

Return stored semantic outcome.

---

# 51. FailedTerminal

Depends on operation policy whether same key may be retried.

---

# 52. Unknown

External side effect uncertain.

---

# 53. Unknown Must Not Be Converted to New Intent Automatically

Critical.

---

# 54. Idempotency Lifetime

Operation-specific.

---

# 55. API Write

Could retain hours/days.

---

# 56. Release Publish

Longer.

---

# 57. Webhook Semantic Dedupe

Part 44.

---

# 58. TriggerIdempotency

Separate from HTTP request idempotency but same architecture primitives can serve.

---

# 59. External Effect Intent

Persist before provider call.

---

# 60. ExternalEffectIntentId

```rust
pub struct ExternalEffectIntentId(Ulid);
```

---

# 61. External Effect State

```rust
pub enum ExternalEffectState {
    Planned,
    Started,
    Succeeded,
    Failed,
    Unknown,
    Reconciled,
}
```

---

# 62. Idempotency + External Provider

Ideal:

```text
Forgeyard idempotency key
+
provider idempotency token
+
expected provider version
```

---

# 63. Provider Example

Cloud create operation.

---

# 64. Unknown Provider Timeout

Inspect first.

---

# 65. Blind Retry

Forbidden for non-idempotent operations.

---

# 66. Canonical State Versioning

Every business aggregate uses its own:

```text
version
epoch
expected state
```

---

# 67. Lease Does Not Replace Aggregate Version

Critical.

---

# 68. Example Deployment

```text
Deployment version 19
+
environment lease token 88
```

Both may be required.

---

# 69. Why

Lease protects concurrent actor ownership.

Aggregate version protects stale business-state transition.

---

# 70. Double Protection

Intentional.

---

# 71. Database Transaction

Acquisition and canonical state transition may need same transaction.

---

# 72. Postgres Baseline

Use:

```text
row version
unique constraints
transactional insert/update
SELECT ... FOR UPDATE where appropriate
advisory locks only as optimization/narrow primitive
```

---

# 73. Advisory Locks

Never sole durable truth.

---

# 74. Session Lock Failure

Connection loss releases lock.

Therefore fencing/DB state still required.

---

# 75. Lease Table

Durable.

---

# 76. Sample Fields

```text
scope_id
holder_kind
holder_id
epoch
expires_at
version
```

---

# 77. Acquire Lease

Transaction:

```text
read current scope
verify expired/unowned
increment epoch
store holder
commit
```

---

# 78. Unique Scope Constraint

One active durable owner.

---

# 79. Clock Semantics

Use DB/server authoritative time.

---

# 80. Avoid Client Clock For Expiry

Critical.

---

# 81. Monotonic Time

Process-local for durations.

Persistent expiry based on trusted server timestamp.

---

# 82. Clock Skew

Must not allow two valid owners.

---

# 83. Lease Grace

Only for operational cleanup, not authority extension.

---

# 84. Lock Contention

First-class outcome.

---

# 85. AcquireResult

```rust
pub enum AcquireResult {
    Acquired(Lease),
    Busy(CurrentLeaseSummary),
    Rejected(ConflictReason),
}
```

---

# 86. Queueing

Caller may enqueue rather than spin.

---

# 87. No Busy-Wait Polling

Critical.

---

# 88. Waiter

```rust
pub struct LeaseWaiterId(Ulid);
```

---

# 89. Wait Queue

Durable optional.

---

# 90. FIFO

Baseline fairness.

---

# 91. Priority

Explicit policy.

---

# 92. Starvation

Prevent via aging.

---

# 93. Exclusive Resource Class

Some resources support capacity >1.

---

# 94. Semaphore

```rust
pub struct CapacitySemaphore {
    pub scope: ConcurrencyScopeId,
    pub capacity: u32,
}
```

---

# 95. Permit

```rust
pub struct SemaphorePermit {
    pub lease: LeaseId,
    pub units: u32,
}
```

---

# 96. Examples

```text
5 concurrent preview environments
2 signing slots
10 macOS simulator slots
```

---

# 97. Scheduler Resource Capacity

Scheduler still owns execution resources.

Use concurrency semaphore only for logical shared constraints.

---

# 98. Do Not Duplicate Scheduler Capacity Model

Critical.

---

# 99. Multi-Resource Acquisition

Dangerous.

Example:

```text
environment lock
+
device
+
test account
```

---

# 100. Deadlock Prevention

Prefer:

```text
single composite scope
or
global deterministic acquisition order
```

---

# 101. Composite Scope

```rust
pub struct CompositeConcurrencyScopeId(Digest);
```

---

# 102. Deterministic Order

Sort by canonical scope ID.

---

# 103. No Arbitrary Nested Locking

Critical.

---

# 104. Try-Acquire-All

Transactional where same DB domain.

---

# 105. Partial Acquisition

Release all on failure.

---

# 106. Cross-System Resource

May not support atomic acquire.

---

# 107. Saga-Style Reservation

Use:

```text
reserve A
reserve B
if B fails -> release A
```

with expiry.

---

# 108. Deadlock Detection

Secondary safety.

---

# 109. Lock Ordering

Primary safety.

---

# 110. Lease Transfer

Usually avoid.

---

# 111. Better

Release + acquire new owner with new fencing token.

---

# 112. Ownership Handoff

If required:

```rust
pub struct LeaseHandoff {
    pub old: LeaseId,
    pub new_holder: LeaseHolder,
}
```

---

# 113. Handoff

Atomic transaction + epoch increment.

---

# 114. Stale Old Holder

Fenced.

---

# 115. Cancellation

Cancelling business operation does not blindly release lease before external effects settle.

---

# 116. Correct Sequence

```text
cancel requested
  ↓
stop new side effects
  ↓
reconcile in-flight effect
  ↓
transition business state
  ↓
release lease
```

---

# 117. Otherwise

New owner may collide with old external operation.

---

# 118. Supersession

Part 44 trigger concurrency.

---

# 119. CancelPrevious

Safe only for operations declared supersedable.

---

# 120. Supersedable

Examples:

```text
preview build
branch CI
```

---

# 121. Not Supersedable

Examples:

```text
production deploy
release publish
DB migration
```

baseline.

---

# 122. ConcurrencyPolicy

```rust
pub struct ConcurrencyPolicy {
    pub supersedable: bool,
    pub max_parallel: Option<u32>,
    pub fairness: FairnessPolicy,
}
```

---

# 123. Environment Deployment Lock

Scope:

```text
EnvironmentId
```

---

# 124. Deployment Strategy

Some strategies need sub-scope.

Example:

```text
blue/green
```

Even then control authority stays one deployment operation.

---

# 125. No Two Independent Deploy Controllers For Same Environment

Critical.

---

# 126. Release Publish Lock

Scope:

```text
release target/channel/package coordinate
```

---

# 127. Package Version

Immutable publication usually no lock after exact uniqueness constraint.

---

# 128. Mutable Alias

Needs exclusive update/precondition.

---

# 129. Infrastructure Apply Lock

Part 53.

---

# 130. Database Migration Lock

High risk.

---

# 131. Migration Lease

Includes expected schema generation.

---

# 132. Merge Target Lock

Part 54.

---

# 133. Expected-Head

Still mandatory.

---

# 134. Queue Controller Lease

Coordinates submit controller.

---

# 135. Target Movement

SCM precondition catches.

---

# 136. Device Lease

Part 20.

---

# 137. Device Reset

Lease retained through reset until resource known clean.

---

# 138. Test Account Lease

Part 56.

---

# 139. Secret Lease

Secrets themselves not locked normally.

Dynamic credential issuance may have provider quotas/semaphores.

---

# 140. Signing Slot

Signing system may use semaphore for HSM capacity.

---

# 141. Signing Authority

Still Part 13/12.

---

# 142. No Lock-Based Signing Authorization

Critical.

---

# 143. Runner Fleet Operation Lock

Image rollout/drain operations can have fleet-management scope.

---

# 144. Autoscaler

Does not need global fleet mutex for normal scaling.

Use desired/observed reconciliation.

---

# 145. Avoid Over-Locking

Critical.

---

# 146. Lock Only True Exclusivity

Do not serialize work merely because implementation is easier.

---

# 147. Concurrency Classification

```rust
pub enum ConcurrencyRequirement {
    None,
    Optimistic,
    Exclusive,
    Capacity(u32),
}
```

---

# 148. Optimistic

Aggregate versioning only.

---

# 149. Exclusive

Lease + fencing.

---

# 150. Capacity

Semaphore/reservation.

---

# 151. Selection

Architectural review.

---

# 152. Lock Granularity

Too broad => low throughput.

Too narrow => race risk.

---

# 153. Scope Explainability

`forgeyard concurrency explain`.

---

# 154. Must Show

```text
scope
holder
epoch
expiry
reason
waiting operations
```

---

# 155. Lock Name Privacy

Avoid sensitive secrets in scope text.

---

# 156. Hashed/TYPED IDs

Preferred.

---

# 157. Lease Renewal Failure

Holder enters:

```text
OwnershipUncertain
```

---

# 158. OwnershipUncertain

No new protected side effects.

---

# 159. Then Reconcile

---

# 160. Network Partition

Holder cannot assume lease remains valid.

Critical.

---

# 161. Fencing Token

External/internal writes protect against stale continuation.

---

# 162. Split Brain

Two processes may think they own.

Only highest accepted epoch may mutate.

---

# 163. Federation

Part 51 authority epoch sits above local concurrency.

---

# 164. Hierarchy

```text
Federation AuthorityEpoch
+
Local Concurrency FencingToken
+
Aggregate Version
```

---

# 165. Each protects different race.

---

# 166. No Global WAN Lock Service Baseline

Critical.

---

# 167. Site Authority

Only authority site acquires local mutation lease for that domain.

---

# 168. Site Failover

New authority epoch invalidates prior site writes.

---

# 169. Local Locks

Do not cross authority site boundary.

---

# 170. Raft

Part 22 only narrow coordination.

---

# 171. Lock State

Postgres business metadata baseline.

---

# 172. Raft Lock Service

Not baseline.

---

# 173. Why

Business recovery/reconciliation needs durable metadata, not opaque consensus lock state.

---

# 174. Leader Election

Can use coordination epoch.

---

# 175. Controller Lease

DB-backed or coordination-backed depending subsystem.

---

# 176. Leader != Business Owner

Critical.

---

# 177. Leader can schedule/reconcile.

Business lease still explicit.

---

# 178. Idempotent Command Model

```rust
pub struct CommandEnvelope<C> {
    pub command_id: CommandId,
    pub idempotency: Option<IdempotencyKey>,
    pub expected_version: Option<u64>,
    pub payload: C,
}
```

---

# 179. CommandId

Globally unique.

---

# 180. Same CommandId

Never executed twice semantically.

---

# 181. Command Result

Stored or derivable.

---

# 182. At-Least-Once Delivery

Safe via command/idempotency.

---

# 183. Exactly-Once Claim

Forbidden.

---

# 184. Outbox/Inbox

Part 10.

---

# 185. Inbox Dedup

Message ID.

---

# 186. Business Idempotency

May be stronger than message dedup.

---

# 187. Example

Two different messages ask:

```text
publish ReleaseId X to channel stable
```

Semantic idempotency can dedupe same intent.

---

# 188. SemanticOperationId

```rust
pub struct SemanticOperationId(Digest);
```

---

# 189. Derived from immutable subject + target.

---

# 190. Use Carefully

Different requested behavior must not collapse accidentally.

---

# 191. Retry Policy

Classify:

```text
safe immediate retry
safe after backoff
inspect before retry
never retry automatically
```

---

# 192. RetrySafety

```rust
pub enum RetrySafety {
    Idempotent,
    ProviderIdempotent,
    InspectBeforeRetry,
    ManualOnly,
}
```

---

# 193. Every External Adapter

Declares retry safety.

---

# 194. No Generic Retry Middleware For Mutations

Critical.

---

# 195. Read Operations

Usually safe.

---

# 196. Write Operations

Operation-specific.

---

# 197. Payment-like Provider

Not core Forgeyard, but same principle.

---

# 198. Lock Timeout

Waiting operation may time out.

---

# 199. Wait Timeout

Does not steal lease.

---

# 200. Force Release

High risk.

---

# 201. ForceReleaseRequest

```rust
pub struct ForceReleaseRequest {
    pub scope: ConcurrencyScopeId,
    pub expected_epoch: FencingToken,
    pub reason: BoundedString,
}
```

---

# 202. Force Release

Only after reconciliation/manual proof.

---

# 203. New Epoch

Must increment.

---

# 204. Never reuse old epoch.

---

# 205. Lease Tombstone

Historical ownership record retained.

---

# 206. Audit

Useful for incident investigation.

---

# 207. Reservation Overbooking

Forbidden unless resource policy explicitly supports it.

---

# 208. Reservation Conflict

First-class.

---

# 209. Capacity Reservation

Scheduler/Part 43 integration.

---

# 210. Reserved Capacity

Does not guarantee successful runner provision if provider fails.

---

# 211. Reservation State

```rust
pub enum ReservationState {
    Pending,
    Active,
    Consumed,
    Released,
    Expired,
    Failed,
}
```

---

# 212. Expired Reservation

Cannot be consumed.

---

# 213. Reservation Renewal

Policy-specific.

---

# 214. UI

Dioxus pages/panels:

```text
Concurrency
Active Leases
Reservations
Blocked Operations
Idempotency
Contention
```

---

# 215. Normal User UX

Mostly contextual:

```text
Waiting: production environment is busy
Holder: deployment #123
Since: ...
```

---

# 216. Admin View

Epoch/token/details.

---

# 217. Force Release UI

High-risk with warnings and reconciliation evidence.

---

# 218. CLI

```text
forgeyard concurrency status
forgeyard concurrency leases
forgeyard concurrency reservations
forgeyard concurrency explain <scope>
forgeyard concurrency force-release
forgeyard concurrency doctor
```

---

# 219. API

Potential:

```text
GET  /v1/concurrency/leases
GET  /v1/concurrency/scopes/{id}
GET  /v1/concurrency/reservations
POST /v1/concurrency/scopes/{id}/force-release
```

---

# 220. Permissions

```text
concurrency.read
concurrency.force_release
concurrency.reservation.manage
concurrency.admin
```

---

# 221. Force Release

Highest privilege.

---

# 222. Audit Events

```text
lease force release
manual priority change
reservation override
idempotency override
deadlock/manual recovery
```

---

# 223. Routine acquire/release

Operational event.

---

# 224. Observability Metrics

```text
concurrency_lease_acquire_total
concurrency_lease_contention_total
concurrency_lease_expired_total
concurrency_stale_fence_rejected_total
concurrency_wait_seconds
concurrency_force_release_total
idempotency_conflict_total
idempotency_unknown_total
```

---

# 225. Labels

Low-cardinality:

```text
scope_kind
result
holder_kind
```

---

# 226. No individual scope IDs as high-cardinality metrics.

---

# 227. Tracing

```text
concurrency.acquire
concurrency.renew
concurrency.release
concurrency.wait
concurrency.fence
idempotency.check
idempotency.record
reservation.acquire
```

---

# 228. Health

Checks:

```text
stale expired leases
renewal lag
wait queue backlog
fencing monotonicity
idempotency store health
```

---

# 229. Doctor

```text
forgeyard concurrency doctor
```

Checks:

```text
expired but unreconciled leases
duplicate active owners
epoch regression
orphan reservations
long contention
unknown idempotency outcomes
```

---

# 230. Duplicate Active Owner

Critical health failure.

---

# 231. Epoch Regression

Critical corruption.

---

# 232. Database Constraints

Prevent where possible.

---

# 233. Lease Reconciler

Periodic.

---

# 234. Reconciler Duties

```text
expire dead leases
verify holder state
trigger external-effect reconciliation
clean orphan waiters
```

---

# 235. Expire != release business effect

Critical.

---

# 236. Example

Deployment lease expired but provider rollout still ongoing.

Reconciler inspects deployment/provider before new deployment proceeds.

---

# 237. Protected Scope Recovery

May enter:

```text
BlockedForReconciliation
```

---

# 238. ScopeOperationalState

```rust
pub enum ScopeOperationalState {
    Free,
    Owned,
    Waiting,
    BlockedForReconciliation,
    Quarantined,
}
```

---

# 239. Quarantined

Severe invariant violation.

---

# 240. No New Ownership

Until resolved.

---

# 241. Security

Threats:

```text
lock stealing
fencing bypass
force-release abuse
idempotency collision
scope spoofing
deadlock DoS
lease exhaustion
```

---

# 242. Scope Authorization

User/job cannot choose arbitrary protected scope.

---

# 243. Scope Derived By Trusted Planner

Critical.

---

# 244. User-Provided Concurrency Group

Namespaced to project/tenant.

---

# 245. Cannot target system internal scope.

---

# 246. Fencing Token

Opaque to untrusted job unless needed.

---

# 247. Provider Token

Not secret, but controlled.

---

# 248. Force Release

Requires reason/audit.

---

# 249. Rate Limit Acquisitions

Prevent lock spam.

---

# 250. Tenant Isolation

Scopes tenant-qualified.

---

# 251. Cross-Tenant Global Resource

Only system-defined.

---

# 252. Global HSM

Capacity managed by trusted service, not tenant-generated key.

---

# 253. Reservation Abuse

Quota Part 27.

---

# 254. Cost

Long-held reservations may consume cost.

Part 45 tracks.

---

# 255. Reliability

Possible SLOs:

```text
lease acquisition latency
stale-fence rejection
reconciliation time
```

---

# 256. High Contention

Operational signal, not necessarily failure.

---

# 257. AI

Part 55 may recommend reducing contention.

Cannot force-release or change exclusivity autonomously.

---

# 258. Search

Part 31 can index historical contention summaries.

---

# 259. Data Lifecycle

Lease history/audit retention Part 46.

---

# 260. Active Lease Data

Operational.

---

# 261. Historical Ownership

Useful for incident/compliance.

---

# 262. Standalone Mode

Process-local optimization permitted only underneath durable local Stoolap state.

---

# 263. Even Mode 1

Avoid correctness based solely on Rust mutex if operations persist/crash.

---

# 264. Distributed Mode

Postgres transactional leases.

---

# 265. Multi-Process Local

Still correct.

---

# 266. Federation

Site authority first.

---

# 267. Air-Gap

Local concurrency operates normally.

---

# 268. Reconnect

No merging of conflicting global ownership if site lacked authority.

---

# 269. Compatibility

Lease/idempotency schema protocol versioned.

---

# 270. Rolling Upgrade

Old/new controllers must agree on fencing semantics.

---

# 271. No Downgrade That Resets Epoch

Critical.

---

# 272. Backup/Restore

Restored DB may contain stale leases.

---

# 273. Restore Rule

All pre-restore active leases considered invalid/stale.

---

# 274. RecoveryEpoch

Can advance globally/per scope.

---

# 275. After Restore

Reconcile external providers before reopening protected scopes.

---

# 276. DR Safety

Critical.

---

# 277. Testkit

```text
forgeyard-concurrency-testkit/src/
├── lib.rs
├── lease.rs
├── fencing.rs
├── idempotency.rs
├── reservation.rs
├── deadlock.rs
├── failover.rs
└── assertions.rs
```

---

# 278. Unit Tests

Scope identity deterministic.

---

# 279. Lease Acquire Test

Only one holder.

---

# 280. Lease Expiry Test

New epoch increments.

---

# 281. Stale Holder Test

Old token rejected.

---

# 282. Renewal Race Test

Only current epoch renews.

---

# 283. Idempotency Same Request Test

Same semantic result.

---

# 284. Idempotency Different Request Test

Conflict.

---

# 285. Unknown Outcome Test

No blind duplicate effect.

---

# 286. DB Connection Loss Test

Session loss does not break durable ownership model.

---

# 287. Controller Crash Test

New controller reconciles.

---

# 288. Provider Timeout Test

Lease not blindly released before observation.

---

# 289. Cancel Test

Lease retained through side-effect reconciliation.

---

# 290. Force Release Test

Requires exact epoch + audit.

---

# 291. Deadlock Order Test

Canonical order prevents cycle.

---

# 292. Multi-Resource Failure Test

Partial reservations compensated.

---

# 293. Semaphore Test

Capacity never exceeded.

---

# 294. Fairness Test

Waiter starvation prevented.

---

# 295. Tenant Isolation Test

Cannot acquire another tenant scope.

---

# 296. Federation Test

Old site authority cannot acquire accepted local mutation scope after failover.

---

# 297. DR Restore Test

Old leases invalidated/reconciled.

---

# 298. Rolling Upgrade Test

Epoch semantics remain compatible.

---

# 299. Fuzzing

Fuzz:

```text
state transitions
idempotency key requests
lease command ordering
multi-resource acquisition
```

---

# 300. Property Tests

At most one accepted owner per exclusive scope/epoch.

---

# 301. Concurrency Stress Test

Many competing controllers.

---

# 302. Chaos Tests

```text
DB failover
network partition
controller pause/resume
process crash after provider call
clock skew
```

---

# 303. Implementation Phase 1 — Idempotency + Scope Model

Core.

---

# 304. Phase 2 — Postgres Lease/Fencing

Distributed correctness.

---

# 305. Phase 3 — Workflow Concurrency Groups

User-facing CI semantics.

---

# 306. Phase 4 — Deployment/Infrastructure/Merge Integration

Protected mutations.

---

# 307. Phase 5 — Reservations/Semaphores

Devices/capacity/test accounts.

---

# 308. Phase 6 — Deadlock/Composite Acquisition

Complex workflows.

---

# 309. Phase 7 — UI/CLI/Doctor

Operability.

---

# 310. Phase 8 — Federation/DR

Authority recovery.

---

# 311. Phase 9 — Fairness/Backpressure

Scale.

---

# 312. Phase 10 — Advanced Provider Idempotency

Adapters.

---

# 313. Phase 11 — Security Hardening

Force-release/scope abuse.

---

# 314. Phase 12 — Chaos/Scale/Fuzz

Production readiness.

---

# 315. Acceptance Tests

1. Locks/leases never become canonical business truth.
2. Exclusive scopes have typed canonical identity.
3. One accepted lease owner exists per exclusive scope.
4. Every new owner receives a higher fencing token.
5. Stale holders cannot mutate protected state.
6. Lease expiry does not imply external effect absence.
7. Ambiguous external effects reconcile before new ownership proceeds.
8. Aggregate versioning remains separate from lease ownership.
9. Idempotency key is scoped by operation/subject/tenant.
10. Same key with different request digest is rejected.
11. Unknown idempotency outcome does not create a new operation automatically.
12. Provider mutations declare retry safety.
13. Generic blind retry middleware is not used for non-idempotent writes.
14. Workflow concurrency groups support queue/cancel/reject explicitly.
15. Non-supersedable protected operations cannot be cancelled by ordinary `CancelPrevious`.
16. Multi-resource acquisition uses composite scope/order/compensation.
17. Deadlock avoidance is designed, not reactive only.
18. Reservations do not exceed resource capacity.
19. Device/test-account reservations expire and reconcile.
20. Force release requires exact scope/epoch/privilege/audit.
21. Controller failover does not allow stale owner continuation.
22. Process-local mutexes are never distributed correctness primitives.
23. Postgres/Stoolap implementations preserve same semantic model.
24. Federation authority fences site-local ownership.
25. Restore invalidates stale pre-restore leases.
26. Protected scopes reconcile before reopening after DR.
27. High contention is observable.
28. Tenant-generated scope names cannot collide with system scopes.
29. Lease history is retained according lifecycle/audit policy.
30. Forgeyard dogfoods the subsystem for deployment, infra apply, merge submission, device leases, release aliases, and workflow concurrency.

---

# 316. Production Readiness Gates

Do not call concurrency architecture production-ready until:

```text
lease/fencing monotonicity is proven
stale-owner mutation tests pass
idempotency request-digest conflicts are enforced
provider Unknown reconciliation works
deployment/infra/merge integrations are dogfooded
deadlock prevention is tested
force-release safety is audited
DR invalidates stale leases
federation authority integration passes
chaos/concurrency stress tests pass
```

---

# 317. Architectural Invariants

1. locks coordinate; they do not define business truth;
2. every exclusive scope is typed;
3. every ownership transfer increments fencing token;
4. stale owners cannot mutate;
5. lease expiry is not proof external effect stopped;
6. reconciliation follows ambiguity;
7. business aggregate version remains separate;
8. idempotency scope includes operation/subject/tenant;
9. same key/different request is conflict;
10. Unknown outcome is first-class;
11. generic mutation retry is forbidden;
12. concurrency groups are explicit;
13. supersession is policy-bound;
14. non-supersedable operations stay protected;
15. multi-resource acquisition avoids arbitrary nesting;
16. deadlock prevention is deterministic;
17. reservations differ from leases;
18. semaphore capacity is enforced;
19. force release is high-risk/audited;
20. process mutexes are not distributed correctness;
21. Postgres/Stoolap share semantics;
22. site authority precedes local concurrency;
23. restore invalidates stale ownership;
24. protected scopes reconcile after DR;
25. tenant scopes are isolated;
26. coordination epochs/leader election do not replace business leases;
27. observability exposes contention/unknown states;
28. lifecycle retains needed ownership history;
29. HA controllers recover through reconciliation;
30. Forgeyard dogfoods its own concurrency primitives.

---

# 318. Final Target Architecture

```text
                       User/System Intent
                              │
                              ▼
                       Idempotency Scope
                              │
                              ▼
                     Concurrency ScopeId
                              │
                              ▼
                       Lease + Fence
                              │
                              ▼
                  Canonical State Transition
                              │
                              ▼
                    External Side Effect
                              │
                              ▼
                         Observation
                              │
                              ▼
                      Reconciliation
                              │
                              ▼
                        Release Lease
```

Protected mutation:

```text
AggregateVersion
+
Concurrency FencingToken
+
AuthorityEpoch (if federated)
  ↓
accepted mutation
```

Duplicate request:

```text
IdempotencyKey
+
request digest
  ↓
existing outcome?
  ├─ yes, same request → return existing semantic result
  ├─ same key, different request → conflict
  └─ unknown effect → inspect/reconcile
```

The key guarantee is:

> **Forgeyard can serialize only the operations that truly need exclusivity, without hiding correctness inside ephemeral locks. Durable leases, monotonically increasing fencing tokens, scoped idempotency keys, aggregate version checks, and reconciliation work together so crashes, retries, failovers, and partitions cannot silently create two accepted owners or duplicate protected effects.**

---

# 319. Extended Architecture Sequence

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
```
