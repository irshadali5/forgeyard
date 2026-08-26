# 54 — Forgeyard Merge Queue, Speculative Integration, Batch Validation & Protected Target Submission System Architecture

**Document type:** Core Merge Queue, Integration Queue, Speculative Validation, Batch Merge, Protected Target Submission & Integration Correctness System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** merge queues, serial integration, speculative execution, merge trains, batch validation, exact integration candidates, protected-target submission, target-base refresh, stale-candidate invalidation, queue priority/fairness, admission, cancellation, flaky/inconclusive checks, SCM submit ambiguity, reconciliation, queue health, and protected branch integration governance  
**Architecture style:** Exact candidate identity, immutable base/source/result snapshots, queue-state machine, optimistic throughput with conservative invalidation, policy-bound evidence, submit-after-verify, reconciliation after ambiguous provider effects, and no merge based on stale validation  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds directly on Change Proposal, SCM Provider Integrations, Pipeline IR, Run/Job State Machine, Policy/Authz, Triggers, Test Intelligence, Static Analysis, Cache Correctness, Monorepo Intelligence, Events/Reconciliation, Reliability, and Multi-Tenancy. The Change Proposal architecture defined baseline integration candidates and serial submission; this subsystem deepens queueing, speculative validation, batching, train invalidation, throughput, fairness, and protected-target correctness.

---

# 1. Purpose

Large repositories and busy protected branches often encounter a fundamental race:

```text
PR A validated against target T0
PR B validated against target T0
PR A merges
target becomes T1
PR B's previous validation may no longer represent what will actually merge
```

Naive systems often solve this by:

```text
merging anyway
re-running everything manually
serializing all work slowly
trusting provider mergeability without exact candidate validation
```

Forgeyard needs a first-class integration subsystem that can answer:

```text
which proposal is next?
what exact target base was it validated against?
what exact merged result did Forgeyard test?
what happens if target changes?
can multiple proposals be speculatively validated?
can compatible proposals be batched?
how does a failed proposal affect the train behind it?
what if the SCM provider times out during merge?
how are flaky or inconclusive checks handled?
how do priorities avoid starvation?
```

The central rule is:

> **A protected-target submission is valid only for the exact integration candidate that was evaluated: exact proposal revision + exact target base + exact resulting source snapshot + exact policy/evidence context.**

A second rule is:

> **Speculation may reduce latency, but speculation never weakens invalidation. If an earlier queue item changes the effective target base, every dependent speculative candidate whose semantics are no longer exact becomes stale and must be recomputed or proven equivalent.**

A third rule is:

> **The merge queue controls ordering and submission, but Policy remains the authority for whether evidence is sufficient, and SCM remains the external side-effect boundary for the final target mutation.**

---

# 2. Architectural Position

```text
                 Change Proposals
                       │
                       ▼
                 Queue Admission
                       │
                       ▼
                  Queue Ordering
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
          Serial    Speculative  Batch
             │         │         │
             └─────────┼─────────┘
                       ▼
              Integration Candidate
                       │
                       ▼
                 Validate Evidence
                       │
                       ▼
               Mergeability Decision
                       │
                       ▼
                 Protected Submit
                       │
                       ▼
                  Verify Result
                       │
                       ▼
                   Reconcile
```

---

# 3. Goals

The subsystem MUST:

1. define merge queue identity;
2. define queue item identity;
3. define exact integration candidate identity;
4. support serial queues;
5. support speculative queues;
6. support merge-train semantics;
7. support batch integration;
8. support target-base invalidation;
9. support proposal revision invalidation;
10. support policy/evidence invalidation;
11. support queue priority;
12. support fairness;
13. support admission policies;
14. support concurrency limits;
15. support queue cancellation;
16. support queue pause/freeze;
17. support flaky/inconclusive checks;
18. support protected target locks;
19. support SCM provider submit;
20. support ambiguous submit reconciliation;
21. support exact post-submit verification;
22. support merge strategy policy;
23. support retry after transient infrastructure failures;
24. support tenant isolation;
25. support audit;
26. support UI/API/CLI;
27. support HA;
28. support disaster recovery;
29. support scale;
30. preserve Change Proposal/Policy authority.

---

# 4. Non-Goals

This subsystem does not:

```text
replace ChangeProposal
replace SCM provider APIs
replace policy
replace test/analysis evidence systems
replace Pipeline IR
allow merge based only on branch names
```

---

# 5. Workspace Structure

```text
crates/integration/
├── forgeyard-integration/
├── forgeyard-integration-model/
├── forgeyard-integration-queue/
├── forgeyard-integration-candidate/
├── forgeyard-integration-train/
├── forgeyard-integration-batch/
├── forgeyard-integration-admission/
├── forgeyard-integration-submit/
├── forgeyard-integration-reconcile/
├── forgeyard-integration-health/
└── forgeyard-integration-testkit/
```

SCM provider-specific mutation remains under existing SCM adapters.

---

# 6. MergeQueueId

```rust
pub struct MergeQueueId(Ulid);
```

A queue is typically bound to a protected integration target.

Examples:

```text
repository/main
repository/release/1.x
monorepo/trunk
```

---

# 7. Queue Target

```rust
pub struct QueueTarget {
    pub repository: RepositoryId,
    pub target: TargetRef,
}
```

`TargetRef` is human/provider context.

Execution correctness always resolves exact target revision/snapshot.

---

# 8. QueueItemId

```rust
pub struct QueueItemId(Ulid);
```

---

# 9. Queue Item

```rust
pub struct MergeQueueItem {
    pub id: QueueItemId,
    pub queue: MergeQueueId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub admitted_at: Timestamp,
    pub priority: QueuePriority,
    pub state: MergeQueueItemState,
}
```

---

# 10. Queue Item State

```rust
pub enum MergeQueueItemState {
    Waiting,
    Preparing,
    Speculating,
    Validating,
    Ready,
    Submitting,
    Submitted,
    Completed,
    Stale,
    Blocked,
    Failed,
    Cancelled,
}
```

---

# 11. Proposal Revision Pinning

Queue item admission binds an exact proposal revision.

If the proposal receives new commits:

```text
old queue item -> Stale/Cancelled
new proposal revision -> new queue item or item revision
```

Never silently mutate the old candidate.

---

# 12. IntegrationCandidateId

```rust
pub struct IntegrationCandidateId(Digest);
```

Content-derived from exact integration semantics.

---

# 13. Integration Candidate

```rust
pub struct IntegrationCandidate {
    pub id: IntegrationCandidateId,
    pub proposal_revision: ProposalRevisionId,
    pub target_revision: RevisionId,
    pub target_snapshot: SourceSnapshotId,
    pub result_snapshot: SourceSnapshotId,
    pub strategy: IntegrationStrategy,
    pub policy: PolicyDigest,
}
```

---

# 14. Candidate Identity Inputs

At minimum:

```text
proposal revision
target exact revision/snapshot
merge strategy
resulting exact source snapshot
relevant integration semantics
policy digest
```

---

# 15. Why ResultSnapshot Matters

Provider-native merge IDs are not enough.

Forgeyard validates the canonical resulting source tree.

Correctness is:

```text
proposal source
+
target base
+
integration strategy
  ↓
exact result SourceSnapshotId
```

---

# 16. Integration Strategy

```rust
pub enum IntegrationStrategy {
    FastForward,
    MergeCommit,
    Rebase,
    Squash,
    ProviderNative,
}
```

---

# 17. Strategy Policy

Protected target can restrict allowed strategies.

---

# 18. ProviderNative

Allowed only when Forgeyard can materialize/verify the exact resulting snapshot before or immediately after submission.

---

# 19. Mergeability Decision

```rust
pub struct MergeabilityDecision {
    pub candidate: IntegrationCandidateId,
    pub result: MergeabilityResult,
    pub evidence: Vec<EvidenceRef>,
    pub policy: PolicyDigest,
}
```

---

# 20. MergeabilityResult

```rust
pub enum MergeabilityResult {
    Mergeable,
    Blocked,
    Incomplete,
    Stale,
    Unknown,
}
```

---

# 21. Incomplete

Required checks/evidence not complete.

---

# 22. Unknown

Provider ambiguity or missing exactness.

---

# 23. Unknown Never Means Mergeable

Critical.

---

# 24. Queue Admission

Admission evaluates:

```text
proposal state
source trust
target protection
required review state
basic policy
queue eligibility
```

Admission does not mean merge readiness.

---

# 25. Admission Result

```rust
pub enum QueueAdmissionResult {
    Admitted,
    Rejected,
    Deferred,
}
```

---

# 26. Deferred

Example:

```text
draft proposal
missing ownership review
dependency proposal not ready
```

---

# 27. Queue Ordering

Baseline ordering dimensions:

```text
priority class
admission time
dependency constraints
manual emergency override
```

---

# 28. QueuePriority

```rust
pub enum QueuePriority {
    Normal,
    High,
    ReleaseCritical,
    Emergency,
}
```

---

# 29. Priority Does Not Bypass Policy

Critical.

---

# 30. Fairness

Prevent starvation.

---

# 31. Fairness Policy

Could include:

```text
FIFO within class
aging
tenant/project weighted fairness
```

---

# 32. Queue Aging

Long-waiting normal item may increase effective scheduling priority.

---

# 33. Security/Emergency Hotfix

Can jump ordering with audited authorization.

Still must satisfy required evidence unless explicit break-glass policy.

---

# 34. Serial Queue

Simplest correctness model:

```text
item A candidate against T0
  ↓ validate
  ↓ submit
target -> T1
  ↓
item B candidate against T1
  ↓ validate
```

---

# 35. Serial Baseline

Recommended first implementation.

---

# 36. Speculative Queue

To improve throughput:

```text
T0
├── candidate A => T1*
├── candidate A+B => T2*
├── candidate A+B+C => T3*
```

where `*` denotes speculative integration snapshots.

---

# 37. Merge Train

Each later item is validated on top of earlier speculative items.

Example:

```text
base T0

Train slot 1:
T0 + A => S1

Train slot 2:
S1 + B => S2

Train slot 3:
S2 + C => S3
```

---

# 38. TrainSlotId

```rust
pub struct TrainSlotId(Ulid);
```

---

# 39. Train Slot

```rust
pub struct MergeTrainSlot {
    pub id: TrainSlotId,
    pub item: QueueItemId,
    pub parent_candidate: Option<IntegrationCandidateId>,
    pub candidate: IntegrationCandidateId,
    pub state: TrainSlotState,
}
```

---

# 40. Parent Dependency

Slot B depends on candidate A.

If A becomes invalid:

```text
B/C/... candidates likely stale
```

---

# 41. Train Invalidations

Triggers:

```text
earlier item failed
earlier item cancelled
earlier proposal updated
target changed externally
policy changed materially
integration strategy changed
candidate result changed
```

---

# 42. Conservative Invalidation

Recompute dependent slots unless exact equivalence can be proven.

---

# 43. Exact Equivalence Optimization

If an earlier event does not change result snapshot or policy-relevant semantics, reuse may be possible.

Proof must be explicit.

---

# 44. No Heuristic Reuse

Critical.

---

# 45. Speculation Depth

Configurable.

```rust
pub struct SpeculationDepth(u16);
```

---

# 46. Bounded Depth

Prevents excessive wasted compute.

---

# 47. Speculative Work

Normal Forgeyard Runs/Jobs.

---

# 48. Cache

May accelerate speculative candidates safely under Part 38.

---

# 49. Speculation Failure

A failed speculative candidate does not mutate target.

---

# 50. Failed Early Slot

Later dependent slots invalidated.

---

# 51. Independent Reordering

Possible only by queue policy.

If B fails but C could pass without B:

```text
remove B
recompute C against new parent/base
```

Never reuse prior C result automatically.

---

# 52. Batch Integration

Batch multiple proposals into one candidate:

```text
T0 + A + B + C => BatchCandidate
```

---

# 53. IntegrationBatchId

```rust
pub struct IntegrationBatchId(Ulid);
```

---

# 54. Batch Candidate

```rust
pub struct IntegrationBatch {
    pub id: IntegrationBatchId,
    pub items: Vec<QueueItemId>,
    pub candidate: IntegrationCandidateId,
}
```

---

# 55. Batch Goals

Reduce:

```text
validation cost
queue latency
target update rate
```

---

# 56. Batch Risk

If batch fails, need isolate culprit.

---

# 57. Batch Failure Strategy

```rust
pub enum BatchFailureStrategy {
    Split,
    SerialFallback,
    RemoveKnownBad,
    Manual,
}
```

---

# 58. Binary Split

Possible:

```text
A+B+C+D fails
  ↓
A+B test
C+D test
```

but use bounded experimentation.

---

# 59. Batch Eligibility

Only when proposals are compatible according to policy.

---

# 60. Batch Constraints

Examples:

```text
same target
no ordering dependency conflict
compatible risk class
batch size limit
release policy
```

---

# 61. High-Risk Changes

May be excluded from batching.

---

# 62. Batch Size

Bounded.

---

# 63. Batch Validation Subject

Exact batch integration result snapshot.

---

# 64. Batch Approval

Policy can require per-proposal reviews + batch candidate evidence.

---

# 65. Individual Review

Batching does not combine human approvals into one magical approval.

---

# 66. Target Head Refresh

Before submit:

```text
read authoritative current target
```

---

# 67. Submit Precondition

```text
current target revision == candidate.target_revision
```

for serial candidate.

For train/batch, equivalent exact expected-base rule applies.

---

# 68. Target Moved

Candidate becomes stale.

---

# 69. External Target Mutation

If someone merges outside Forgeyard:

```text
detect target change
  ↓
invalidate queue train
  ↓
rebase/recompute candidates
```

---

# 70. Protected Target

Recommended provider configuration prevents bypass.

---

# 71. Bypass Detection

If external mutation occurs, audit/alert.

---

# 72. Target Lock

Forgeyard may use narrow coordination/SCM expected-head precondition.

---

# 73. Do Not Depend on Long-Lived Global Mutex Alone

Critical.

Use exact expected-base compare-and-submit semantics.

---

# 74. SubmissionRequestId

```rust
pub struct SubmissionRequestId(Ulid);
```

---

# 75. Submission Intent

Persist before SCM mutation.

---

# 76. Submission State

```rust
pub enum SubmissionState {
    Prepared,
    Submitting,
    Submitted,
    Verified,
    Failed,
    Unknown,
    Reconciled,
}
```

---

# 77. SCM Submit

External effect.

At-least-once/ambiguous.

---

# 78. Unknown Submit

If provider times out:

```text
DO NOT simply retry merge
```

---

# 79. Correct Unknown Recovery

```text
submission timeout
  ↓
state = Unknown
  ↓
inspect target ref
  ↓
materialize resulting revision
  ↓
compare resulting SourceSnapshotId
  ↓
if expected candidate -> success
if unchanged -> retry may be safe
if unexpected -> conflict/manual reconciliation
```

---

# 80. Post-Submit Verification

Mandatory.

---

# 81. Verify

After provider reports success:

```text
fetch target revision
  ↓
materialize SourceSnapshot
  ↓
assert result snapshot == candidate.result_snapshot
```

---

# 82. Provider Success Is Not Sufficient

Critical.

---

# 83. Mismatch

Security/correctness incident.

---

# 84. Submission Evidence

```rust
pub struct SubmissionEvidence {
    pub candidate: IntegrationCandidateId,
    pub expected_result: SourceSnapshotId,
    pub actual_revision: RevisionId,
    pub actual_snapshot: SourceSnapshotId,
}
```

---

# 85. Queue Evidence Freshness

Candidate must have required evidence tied exactly to candidate subject.

---

# 86. Examples

```text
tests
static analysis
coverage
security scan
reproducibility
integration tests
policy facts
```

---

# 87. No PR-Head-Only Evidence for Integrated Candidate

Critical where target interaction matters.

---

# 88. Two Evidence Layers

May evaluate:

```text
proposal-head evidence
+
integration-candidate evidence
```

---

# 89. Policy Chooses Required Layers

---

# 90. Fast Checks

Proposal-head.

---

# 91. Merge-Critical Checks

Integrated candidate.

---

# 92. Monorepo Incremental Integration

Part 34 determines affected work for exact candidate/base.

---

# 93. Target Change

Can alter affected set.

Therefore re-plan if base changes.

---

# 94. Integration Plan Freshness

```rust
pub enum IntegrationPlanFreshness {
    Current,
    TargetChanged,
    ProposalChanged,
    PolicyChanged,
    GraphChanged,
    EvidenceStale,
    Unknown,
}
```

---

# 95. Freshness Required Before Submit

---

# 96. Policy Change

Not every policy/config change invalidates everything.

Use explicit relevant digest.

---

# 97. Relevant Policy Digest

Candidate records exact policy used for mergeability.

---

# 98. Security Critical Policy Change

Can invalidate queued candidates.

---

# 99. Feature Flags

If they alter integration semantics, relevant digest changes.

---

# 100. Test Flakes

Part 32.

---

# 101. Flaky Check Handling

Queue policy can allow:

```text
bounded retry
manual override
quarantine-aware evaluation
```

---

# 102. Original Failure Remains Evidence

Retry pass does not erase it.

---

# 103. Inconclusive Evidence

Never silently green.

---

# 104. QueueItemBlockedReason

```rust
pub enum QueueItemBlockedReason {
    RequiredCheckFailed,
    EvidenceIncomplete,
    ReviewMissing,
    Conflict,
    TargetChanged,
    ProposalUpdated,
    PolicyChanged,
    DependencyBlocked,
    ManualHold,
}
```

---

# 105. Infrastructure Failure

Retry candidate validation according to normal retry policy.

---

# 106. Product/Test Failure

Queue item blocked/failed.

---

# 107. Queue Retry

Creates new Run/Attempt evidence; candidate identity may remain same if exact inputs unchanged.

---

# 108. Queue Item Dependencies

Proposal stacks/dependent changes.

---

# 109. Dependency Graph

```rust
pub struct QueueItemDependency {
    pub before: QueueItemId,
    pub after: QueueItemId,
}
```

---

# 110. Stacked Proposals

Can enter train in dependency order.

---

# 111. Child Proposal

Validated against parent integration result if configured.

---

# 112. Parent Changes

Child candidate invalidates.

---

# 113. Cycles

Rejected.

---

# 114. Queue Pause

```rust
pub enum MergeQueueOperationalState {
    Active,
    Paused,
    DrainOnly,
    Frozen,
}
```

---

# 115. Paused

No new submission; validation may continue depending policy.

---

# 116. DrainOnly

No new admissions; process existing queue.

---

# 117. Frozen

No target mutation.

---

# 118. Incident Use

Part 40 kill switch can freeze submission.

---

# 119. Reliability Integration

Part 50 error-budget governance can require:

```text
smaller batches
serial mode
extra approval
queue freeze for risky changes
```

---

# 120. Reliability Does Not Bypass Security

Existing invariant.

---

# 121. Queue Capacity

Limit speculative workload.

---

# 122. Per-Queue Limits

```text
max waiting items
max concurrent candidates
max speculation depth
max batch size
```

---

# 123. Admission Backpressure

If queue overloaded:

```text
defer admission
```

not silently drop.

---

# 124. Queue Priority vs Tenant Fairness

For shared hosted service, avoid one tenant consuming all speculative capacity.

---

# 125. Scheduler Fairness

Execution resource fairness remains Part 06/27.

---

# 126. Queue Ordering Fairness

Separate logical merge fairness.

---

# 127. Emergency Priority

Requires permission/audit.

---

# 128. QueueItemPriorityChange

Audited when manual.

---

# 129. Change Proposal Removal

If closed:

```text
cancel queue item
invalidate dependents
```

---

# 130. Proposal Drafted Again

Block/cancel according config.

---

# 131. Approval Revoked

Candidate becomes blocked/stale depending policy.

---

# 132. New Review Needed

If proposal revision changed.

---

# 133. Approval Binding

Existing Change Proposal invariant:

approval binds exact proposal revision/candidate/policy as configured.

---

# 134. Merge Queue Does Not Reuse Approval Across Changed Revision Silently

Critical.

---

# 135. Target Conflict

Integration engine attempts selected strategy.

If conflict:

```text
candidate creation fails
item Blocked(Conflict)
```

---

# 136. Auto-Rebase

Can generate new candidate only.

Does not mutate proposal branch unless explicit workflow.

---

# 137. Rebase-on-Submit

Provider behavior must be modeled exactly.

---

# 138. Squash Message

Deterministic/provider-specific metadata.

---

# 139. Commit Metadata

Author/committer attribution preserved according policy.

---

# 140. Signed Commit Requirement

If protected target requires signed integration commit, signing worker handles exact candidate metadata.

---

# 141. Signing Worker

Cannot compile.

Existing invariant.

---

# 142. Integration Commit Signing

Separate from artifact signing.

---

# 143. Integration Commit Identity

Provider/VCS specific.

---

# 144. VCS Neutral Core

Core candidate remains source-snapshot based.

---

# 145. Git Adapter

Can create merge/rebase/squash candidate.

---

# 146. Mercurial Adapter

Equivalent via changesets where supported.

---

# 147. Provider API vs Local VCS Construction

Two options:

```text
local deterministic integration construction
provider-native preview
```

Prefer local canonical materialization when possible.

---

# 148. Provider Native Mergeability

Advisory until exact candidate verified.

---

# 149. Change Proposal Checks

SCM status/check run can reflect queue state.

Examples:

```text
Queued
Validating
Ready
Stale
Blocked
Submitted
```

---

# 150. SCM Required Check

Can be a Forgeyard merge-queue check.

---

# 151. Avoid Deadlock

Provider branch protection must not require impossible self-referential status.

---

# 152. Integration Result Publication

Queue can publish candidate check results to proposal.

---

# 153. Check Subject

Exact candidate context/digest included.

---

# 154. Queue Webhooks

SCM provider events trigger reconciliation.

---

# 155. Poll/Reconcile

Do not rely only on webhook delivery.

---

# 156. Queue Reconciler

Checks:

```text
target head
proposal revision
review state
policy digest
candidate status
submission ambiguity
SCM target result
```

---

# 157. Reconciliation Is Correctness Mechanism

Events accelerate.

---

# 158. HA

Multiple queue workers can operate safely.

---

# 159. Queue Work Claim

DB lease/claim.

---

# 160. Submission Serialization

Per target authority domain.

---

# 161. Coordination Epoch

Can use narrow Part 22 coordination for target-submit ownership.

---

# 162. Postgres Business State Remains Authority

---

# 163. QueueTargetEpoch

```rust
pub struct QueueTargetEpoch(u64);
```

Optional fencing for submit controller leadership.

---

# 164. Stale Submit Controller

Cannot submit after leadership/epoch change.

---

# 165. But SCM Expected-Head Check Is Still Required

Critical.

---

# 166. DR

Queue items/candidates/submission intents backed up.

---

# 167. After Restore

Before resuming submit:

```text
reconcile current SCM target
re-check proposal state
invalidate stale candidates
```

---

# 168. Never Blindly Resume Pending Submit

Critical.

---

# 169. Federation

Part 51 target authority must belong to one site.

---

# 170. No Cross-Site Concurrent Submit

---

# 171. Disconnected Site

Cannot submit to protected global target without delegated target authority and connectivity/provider access.

Baseline: forbidden.

---

# 172. Air-Gap

Can validate candidate offline, but final merge authority remains explicit.

---

# 173. Security

Threats:

```text
stale candidate merge
SCM bypass
provider spoofed success
malicious priority escalation
approval reuse
candidate/evidence mismatch
TOCTOU target movement
```

---

# 174. Core Security Controls

```text
exact candidate digest
expected target head
post-submit verification
authz on queue controls
audit priority/bypass
policy/evidence binding
```

---

# 175. Break-Glass Merge

Possible only via explicit high-risk policy.

---

# 176. Break-Glass Requirements

At minimum:

```text
authorized actor
reason
target
proposal revision
known missing evidence
audit
notification
```

---

# 177. Break-Glass Never Becomes Normal Queue Path

---

# 178. Bypass Provider Merge

If organization permits direct push, Forgeyard still enforces exact expected-head and post-push snapshot verification.

---

# 179. Queue Metrics

```text
merge_queue_depth
merge_queue_wait_seconds
merge_queue_candidate_build_seconds
merge_queue_stale_total
merge_queue_submit_total
merge_queue_submit_unknown_total
merge_queue_batch_size
merge_queue_speculation_waste_total
```

---

# 180. Speculation Waste

Work invalidated before useful submit.

Useful for tuning depth.

---

# 181. Labels

Low cardinality:

```text
queue
result
mode
```

Queue count bounded.

---

# 182. Reliability SLOs

Possible:

```text
queue wait p95
candidate validation latency
submission success
stale-rate
```

---

# 183. Cost

Part 45 can attribute speculative/batch validation cost.

---

# 184. Speculation Cost

Advisory tuning metric.

---

# 185. Do Not Disable Required Validation Merely Due Cost

Critical.

---

# 186. Search/Analytics

Part 31 indexes queue history.

---

# 187. Analytics

Examples:

```text
average queue time
failure causes
speculation reuse
batch efficiency
target bypass rate
```

---

# 188. No Developer Ranking

Critical.

---

# 189. Catalog Integration

Component ownership can route blocked/failing queue notifications.

---

# 190. Notification

Examples:

```text
proposal blocked in queue
target changed externally
queue frozen
submission unknown
batch split
```

---

# 191. Audit

Audit:

```text
manual priority change
queue freeze/unfreeze
manual removal
break-glass merge
submission reconciliation override
batch policy change
```

---

# 192. Routine queue state transitions

Operational events.

---

# 193. Dioxus UI

Pages/panels:

```text
Merge Queues
Queue Detail
Merge Train
Candidate Detail
Submission History
Blocked Items
```

---

# 194. Queue Detail

Shows:

```text
position
proposal
revision
candidate
target base
state
checks
wait time
priority
```

---

# 195. Train UI

Visual:

```text
T0
 └─ A → S1
      └─ B → S2
           └─ C → S3
```

---

# 196. Invalidation UI

Explain why candidate became stale.

---

# 197. Candidate Detail

Shows:

```text
proposal revision
base revision
result snapshot
strategy
policy digest
evidence
```

---

# 198. Submission Detail

Shows expected vs actual target result.

---

# 199. CLI

```text
forgeyard queue list
forgeyard queue show
forgeyard queue add
forgeyard queue remove
forgeyard queue prioritize
forgeyard queue pause
forgeyard queue resume
forgeyard queue freeze
forgeyard queue explain <item>
forgeyard queue candidate show
forgeyard queue doctor
```

---

# 200. API

Potential:

```text
GET  /v1/merge-queues
GET  /v1/merge-queues/{id}
POST /v1/merge-queues/{id}/items
DELETE /v1/merge-queues/{id}/items/{item}
POST /v1/merge-queues/{id}/pause
POST /v1/merge-queues/{id}/freeze
GET  /v1/integration-candidates/{id}
```

---

# 201. Permissions

```text
merge_queue.read
merge_queue.enqueue
merge_queue.remove
merge_queue.priority.manage
merge_queue.pause
merge_queue.freeze
merge_queue.breakglass
```

---

# 202. Enqueue

Can be automatic after reviews or manual depending policy.

---

# 203. Queue Mode

```rust
pub enum MergeQueueMode {
    Serial,
    Speculative,
    Batch,
    Hybrid,
}
```

---

# 204. Hybrid

Example:

```text
speculate normal proposals
serial high-risk proposals
batch low-risk docs/tooling proposals
```

---

# 205. Queue Policy

```rust
pub struct MergeQueuePolicy {
    pub mode: MergeQueueMode,
    pub speculation_depth: u16,
    pub max_batch_size: u16,
    pub fairness: QueueFairnessPolicy,
    pub retry: QueueRetryPolicy,
}
```

---

# 206. Risk-Aware Mode

Policy may choose by proposal classification.

---

# 207. Security-Critical Change

Serial/exclusive validation.

---

# 208. Documentation-Only Change

May permit batch if policy/evidence allows.

---

# 209. Queue Retry Policy

Distinguish:

```text
infrastructure failure
provider failure
test flake
product failure
```

---

# 210. Retry Budget

Bounded.

---

# 211. Infrastructure Retry

Automatic within limits.

---

# 212. Product Failure

No automatic endless retry.

---

# 213. Flaky Failure

Part 32 determines allowed retry/quarantine semantics.

---

# 214. Queue Item Requeue

Creates fresh candidate if target changed.

---

# 215. Requeue History

Preserved.

---

# 216. QueueAttemptId

```rust
pub struct QueueAttemptId(Ulid);
```

---

# 217. Queue Attempt

One candidate-validation cycle.

---

# 218. Queue Item History

Append-only.

---

# 219. Stale Candidate Retention

Keep metadata/evidence long enough for debugging/analytics.

---

# 220. Data Lifecycle

Part 46.

---

# 221. Candidate CAS Roots

Only while needed; result snapshot may already be rooted through run/release/history policy.

---

# 222. Candidate Logs

Normal run retention.

---

# 223. Target Protection Configuration

Provider branch protection desired state can be reconciled.

---

# 224. Example

Require:

```text
Forgeyard queue status
no direct push except service identity
```

---

# 225. Provider Drift

If protection removed:

```text
alert
possibly freeze queue
```

---

# 226. Queue Integrity Doctor

```text
forgeyard queue doctor
```

Checks:

```text
target protection
stale candidates
unknown submissions
queue item/proposal mismatch
missing candidate evidence
leadership/epoch
SCM connectivity
```

---

# 227. Health State

```rust
pub enum MergeQueueHealth {
    Healthy,
    Degraded,
    Frozen,
    SubmissionUncertain,
    Unhealthy,
}
```

---

# 228. Unknown Submission

High-priority health issue.

---

# 229. Target Divergence

If target actual snapshot does not match expected after submit:

```text
freeze queue
open security/correctness incident
```

---

# 230. No Further Submission Until Reconciled

Critical.

---

# 231. Merge Candidate Construction

Prefer deterministic VCS adapter.

Inputs:

```text
base snapshot
proposal snapshot
strategy
commit metadata rules
```

---

# 232. Candidate Construction Sandbox

No untrusted build execution required.

---

# 233. Submodule/Subrepo

Exact child snapshots.

---

# 234. LFS/Large Files

Resolved according source snapshot semantics.

---

# 235. Merge Conflict

Deterministically represented.

---

# 236. Candidate SourceSnapshotId

Canonical tree hash.

---

# 237. Candidate vs Native VCS Commit

Native commit ID can vary due metadata.

Forgeyard correctness can still be tree/source-snapshot based, while provider-native revision retained.

---

# 238. Cases Where Commit Metadata Matters

If policy cares about exact commit/signature identity, include in candidate semantics.

---

# 239. Merge Commit Reproducibility

Timestamp/committer may affect VCS ID.

Need controlled metadata or post-submit equivalence rule.

---

# 240. Source Snapshot Truth

File tree equality remains strongest cross-VCS baseline.

---

# 241. Merge Queue + Reproducibility

Candidate build/test can feed release path later.

---

# 242. But merge validation artifact does not automatically become ReleaseTrusted.

Policy decides.

---

# 243. Build-Once Opportunity

If exact post-merge snapshot equals validated candidate and evidence remains fresh, downstream main build may reuse trusted cache/evidence where policy allows.

---

# 244. No Automatic Evidence Promotion

Critical.

---

# 245. Queue Admission From Trigger

Part 44 events:

```text
review approved
required checks complete
manual enqueue
```

---

# 246. Auto-Enqueue

Optional.

---

# 247. Auto-Enqueue Does Not Auto-Merge Until queue policy/evidence satisfied.

---

# 248. Queue Admission Cancellation

Proposal update can automatically invalidate current admission.

---

# 249. Race Safety

Every transition uses expected version/current state.

---

# 250. Optimistic Concurrency

DB compare-and-set/version.

---

# 251. QueueItemVersion

```rust
pub struct QueueItemVersion(u64);
```

---

# 252. No Lost Updates

---

# 253. Batch Split Algorithm

Must be deterministic and bounded.

---

# 254. Default

Simple halves preserving order.

---

# 255. Example

```text
[A,B,C,D] fails
  ↓
[A,B] and [C,D]
```

---

# 256. If [A,B] fails

split again.

---

# 257. Cost/Run Limit

Stop and fall back serial after threshold.

---

# 258. No Exponential Explosion

Critical.

---

# 259. Speculative Cancellation

When candidate stale, request cancellation of validation Run.

---

# 260. Cancellation Is Best-Effort

If already completing, result retained but marked stale subject.

---

# 261. Stale Result

Never satisfies new candidate.

---

# 262. Evidence Subject Matching

Exact `IntegrationCandidateId`.

---

# 263. Target Merge Freeze Window

Release branches may have scheduled freeze.

---

# 264. Queue During Freeze

Can:

```text
continue validation
hold Ready items
```

depending policy.

---

# 265. Resume

Revalidate target/freshness first.

---

# 266. Maintenance

Queue freeze auditable.

---

# 267. Multi-Target Proposals

If proposal targets multiple branches, separate queue item/candidate per target.

---

# 268. Backport Queue

Separate target.

---

# 269. Cherry-Pick Automation

Can construct explicit candidate, but not hidden mutation.

---

# 270. Release Branch Queue

May have stricter policy.

---

# 271. Queue Dependencies Across Targets

Avoid baseline complexity.

---

# 272. External CI Checks

SCM checks from external systems can be policy evidence if trusted and exact enough.

---

# 273. Trust Class

External check provider trust explicit.

---

# 274. Stale External Check

Cannot satisfy current candidate.

---

# 275. Queue State Event

```rust
pub enum MergeQueueEvent {
    ItemAdmitted,
    CandidateCreated,
    CandidateInvalidated,
    ValidationStarted,
    ValidationCompleted,
    ItemReady,
    SubmitStarted,
    SubmitUnknown,
    SubmitVerified,
    ItemCompleted,
}
```

---

# 276. Events

At-least-once.

---

# 277. Persisted Queue State Authority

Existing invariant.

---

# 278. Reconciler

Can reconstruct progress from persisted state + SCM observed state.

---

# 279. Failure Recovery

Worker crash during candidate validation:

```text
Run state recovers normally
queue reconciler resumes
```

---

# 280. Crash During Submit

Submission state `Unknown` until SCM inspected.

---

# 281. Crash After Provider Success Before DB Update

Reconcile target and bind verified result.

---

# 282. Crash Before Provider Call

Submission intent remains Prepared; safe to resume.

---

# 283. Submission Idempotency

Provider-native idempotency token if available.

Still inspect target.

---

# 284. Multiple Repositories

One queue per target repository/ref.

---

# 285. Monorepo

High-value for speculative queue due expensive validation.

---

# 286. Partial Validation

Part 34 affected work can reduce candidate cost.

---

# 287. Shadow Full Validation

Continue to verify impact analysis quality.

---

# 288. Integration Candidate Search

Search by:

```text
proposal
target
result snapshot
state
```

---

# 289. Diagnostics

Part 48 can diagnose candidate failure.

---

# 290. Bisect

Not generally within queue, but failing integrated candidate can use diagnosis tooling.

---

# 291. Test Flake Intelligence

Queue can surface:

```text
failed deterministic
suspected flaky
infrastructure
```

---

# 292. Developer Experience

Local command:

```text
forgeyard queue simulate
```

can build candidate locally without enqueue/submit.

---

# 293. Simulate

Never mutates SCM.

---

# 294. Simulation Result

Exact candidate snapshot and plan.

---

# 295. Rebase Preview

Developer can see whether current proposal would conflict.

---

# 296. Candidate Reproduction

```text
forgeyard reproduce --candidate <id>
```

---

# 297. Audit Trail

For final merge, trace:

```text
proposal revision
reviews
queue item
candidate
evidence
policy
submission
actual target revision
```

---

# 298. Supply Chain

This lineage can later feed release provenance.

---

# 299. Compliance

Protected branch submission evidence exportable.

---

# 300. No Hidden Queue Decisions

Explainability:

```text
why position?
why stale?
why blocked?
why batch?
why serial?
```

---

# 301. `forgeyard queue explain`

Returns deterministic rationale.

---

# 302. Testkit

```text
forgeyard-integration-testkit/src/
├── lib.rs
├── queue.rs
├── candidate.rs
├── train.rs
├── batch.rs
├── submit.rs
├── reconcile.rs
└── assertions.rs
```

---

# 303. Unit Tests

Candidate identity determinism.

---

# 304. Proposal Update Test

Old candidate stale.

---

# 305. Target Move Test

Candidate stale.

---

# 306. Policy Change Test

Relevant candidate invalidates.

---

# 307. Serial Queue Test

Correct base chaining.

---

# 308. Speculative Train Test

B depends on A candidate.

---

# 309. Earlier Failure Test

Dependent candidates invalidate.

---

# 310. Batch Success Test

Exact result verified.

---

# 311. Batch Failure Split Test

Bounded split.

---

# 312. No Explosion Test

Fallback serial after configured threshold.

---

# 313. Unknown Submit Test

Inspect before retry.

---

# 314. Provider Success Mismatch Test

Queue freezes.

---

# 315. Approval Reuse Test

Changed proposal cannot reuse stale approval improperly.

---

# 316. Evidence Subject Test

PR-head-only result cannot satisfy candidate when integrated evidence required.

---

# 317. Flaky Check Test

Policy rules applied.

---

# 318. Infrastructure Retry Test

Candidate remains exact.

---

# 319. Priority Test

Emergency priority cannot bypass policy.

---

# 320. Fairness Test

Normal items do not starve.

---

# 321. External Bypass Test

Target mutation invalidates train.

---

# 322. HA Submit Controller Test

Only current controller/epoch submits.

---

# 323. SCM Expected-Head Test

Stale controller cannot merge against changed base.

---

# 324. DR Test

Restored queue reconciles before submission.

---

# 325. Federation Test

Only authority site submits target.

---

# 326. Tenant Isolation Test

No cross-tenant queue data leakage.

---

# 327. Security Test

Unauthorized priority/freeze/breakglass denied.

---

# 328. Fuzzing

Fuzz:

```text
queue transition commands
provider submit responses
candidate metadata
batch split inputs
```

---

# 329. Property Tests

A Verified submission must imply:

```text
actual target snapshot == candidate result snapshot
```

---

# 330. Scale Test

Thousands of queued proposals.

---

# 331. Monorepo Load Test

Large speculative plans and affected-work selection.

---

# 332. Chaos Tests

```text
SCM outage
webhook duplication
worker crash
DB restart
target external mutation
provider timeout after merge
```

---

# 333. Implementation Phase 1 — Serial Merge Queue

Correctness first.

---

# 334. Phase 2 — Exact Candidate Construction

SourceSnapshot-based.

---

# 335. Phase 3 — Protected Submit/Reconciliation

SCM ambiguity safety.

---

# 336. Phase 4 — Queue UI/CLI/Explain

Operability.

---

# 337. Phase 5 — Speculative Train

Throughput.

---

# 338. Phase 6 — Flake/Retry Intelligence

Quality.

---

# 339. Phase 7 — Batch Integration

High-scale optimization.

---

# 340. Phase 8 — Priority/Fairness

Enterprise.

---

# 341. Phase 9 — Federation/HA

Multi-region correctness.

---

# 342. Phase 10 — Cost/Reliability Tuning

Optimization.

---

# 343. Phase 11 — Compliance/Audit Export

Governance.

---

# 344. Phase 12 — Scale/Chaos/Fuzz Hardening

Production readiness.

---

# 345. Acceptance Tests

1. Queue item binds exact ProposalRevisionId.
2. Integration candidate binds exact target revision/snapshot.
3. Integration candidate binds exact result SourceSnapshotId.
4. Protected submission requires current candidate freshness.
5. Target movement invalidates stale candidates.
6. Proposal updates invalidate stale candidates.
7. Relevant policy changes invalidate stale candidates.
8. Unknown/incomplete evidence cannot be treated as mergeable.
9. Serial queue validates each item against actual current predecessor result.
10. Speculative train records explicit parent-candidate dependency.
11. Failure/cancellation of an earlier train item invalidates dependent slots.
12. Speculative result is never reused heuristically after base change.
13. Batch candidate is exact and immutable.
14. Batch failure recovery is bounded.
15. Queue priority never bypasses policy/review/security.
16. Fairness prevents indefinite starvation.
17. Protected submit uses expected-head/precondition semantics.
18. Provider submit timeout becomes Unknown, not immediate retry.
19. Unknown submit inspects SCM target before retry.
20. Provider success is followed by exact post-submit snapshot verification.
21. Actual target mismatch freezes protected queue and raises incident.
22. PR-head evidence cannot substitute integrated-candidate evidence when policy requires latter.
23. Flaky retries preserve original failure evidence.
24. Infrastructure failures can retry without changing candidate identity when exact inputs remain.
25. External target mutation invalidates train.
26. HA controllers cannot submit concurrently for same target.
27. DR does not blindly resume pending submission.
28. Federation allows one authority site for target mutation.
29. Break-glass is explicit/high-privilege/audited.
30. Candidate/queue lineage is exportable for audit/provenance.
31. Cache/affected-work optimizations never alter semantic candidate correctness.
32. Standalone/distributed share queue semantics.
33. Queue events are at-least-once and reconciled.
34. Large queues remain bounded/observable.
35. Forgeyard dogfoods the merge queue on its own protected development branch.

---

# 346. Production Readiness Gates

Do not call merge queue production-ready until:

```text
serial queue correctness passes
candidate exact-source identity is stable
target movement invalidation is reliable
expected-head protected submit works
Unknown submit reconciliation is proven
post-submit snapshot verification is mandatory
stale evidence cannot pass
HA/DR submit-controller tests pass
provider bypass/drift detection works
chaos tests pass
```

Speculative trains and batching are optimizations layered on top of the correct serial model.

---

# 347. Architectural Invariants

1. queue ordering never substitutes for policy;
2. queue item binds exact proposal revision;
3. candidate binds exact target base;
4. candidate binds exact result snapshot;
5. stale candidate cannot submit;
6. target movement invalidates dependent candidates;
7. proposal changes invalidate candidate;
8. relevant policy/evidence changes invalidate candidate;
9. speculation never weakens exactness;
10. dependent speculative slots invalidate conservatively;
11. batch identity is exact;
12. batch failure handling is bounded;
13. priority cannot bypass required controls;
14. fairness prevents starvation;
15. expected-head check is mandatory for protected submit;
16. SCM mutation is ambiguous external effect;
17. Unknown submit is inspected before retry;
18. provider success is not final until target snapshot verified;
19. result mismatch freezes queue;
20. integrated evidence is exact-subject bound;
21. flaky retries preserve failed observations;
22. queue reconciliation is correctness mechanism;
23. external target bypass is detected;
24. HA uses fenced submit ownership plus SCM precondition;
25. DR reconciles before resuming;
26. federation permits one target authority site;
27. break-glass is explicit/audited;
28. optimization cannot weaken correctness;
29. queue lineage is auditable;
30. Forgeyard dogfoods its own merge queue.

---

# 348. Final Target Architecture

```text
                    ChangeProposal
                          │
                          ▼
                     Queue Item
                          │
                          ▼
              Exact IntegrationCandidate
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
      Serial          Speculative         Batch
        │                 │                 │
        └─────────────────┼─────────────────┘
                          ▼
                   Candidate Evidence
                          │
                          ▼
                    Policy Decision
                          │
                          ▼
                Expected-Head Submit
                          │
                          ▼
               Post-Submit Verification
                          │
                          ▼
                       Complete
```

Serial correctness:

```text
T0 + A -> S1
verify S1
submit A
target == S1

S1 + B -> S2
verify S2
submit B
target == S2
```

Speculative train:

```text
T0
 └─ A -> S1
      └─ B -> S2
           └─ C -> S3
```

If A changes:

```text
S1 stale
  ↓
S2 stale
  ↓
S3 stale
  ↓
recompute
```

Submit ambiguity:

```text
SCM merge timeout
     ↓
SubmissionState::Unknown
     ↓
inspect current target
     ↓
materialize SourceSnapshot
     ↓
compare with candidate result
     ↓
verified success / safe retry / conflict
```

The key guarantee is:

> **Forgeyard can make protected-target integration fast without making it approximate. Serial, speculative, train, and batch modes all reduce to the same invariant: only the exact candidate that was validated against the exact target base and exact policy/evidence state may be submitted, and the target is verified afterward to ensure the SCM provider produced exactly what Forgeyard approved.**

---

# 349. Extended Architecture Sequence

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
27 Multi-Tenancy / Quotas / Resource Governance
28 Audit / Compliance / Evidence Retention / Security Governance
29 Notifications / Alerting / Human Workflow
30 Entitlements / Licensing / Subscription / Commercial Access Control
31 Search / Indexing / Query / Operational Analytics
32 Test Results / Quality Gates / Coverage / Flaky-Test Intelligence
33 Benchmarking / Performance Regression / Load-Test / Capacity Intelligence
34 Monorepo Intelligence / Dependency Graph / Affected-Change / Incremental Execution
35 Developer Experience / Local Dev Environment / CLI Workflows / Reproducible Workstation
36 Dependency / Package Registry / Artifact Mirror / Software-Source Governance
37 Static Analysis / Code Quality / Security Scanning / Findings Management
38 Cache / Build Acceleration / Remote Cache / Cache Correctness
39 Configuration / Feature Flags / Runtime Settings / Dynamic Configuration Governance
40 Security Architecture / Threat Model / Hardening / Incident Response
41 Release Distribution / Update Delivery / Installer / Channel / Client Update
42 Workflow Templates / Reusable Pipelines / Organization Standards / Golden Paths
43 Runner Fleet Autoscaling / Capacity Provisioning / Infrastructure Providers
44 Pipeline Triggers / Schedules / Manual Dispatch / Event-Driven Execution
45 Cost Accounting / FinOps / Chargeback / Showback / Resource Economics
46 Data Lifecycle / Retention / Archival / Deletion / Legal Hold / Privacy Governance
47 CI/CD Migration / Import / Compatibility / Legacy-System Interoperability
48 Failure Diagnosis / Debugging / Reproduction / Bisect / Root-Cause Intelligence
49 Service Catalog / Component Ownership / Environment Inventory / Developer Portal
50 Reliability Engineering / SLO / Error Budget / Availability / Resilience Governance
51 Multi-Region Federation / Edge Sites / Disconnected Operation / Cross-Site Replication
52 Artifact Registry / Package Repository / OCI Distribution / Internal Software Distribution
53 Infrastructure-as-Code / Environment Provisioning / Preview Environments / Drift Reconciliation
54 Merge Queue / Speculative Integration / Batch Validation / Protected Target Submission
```
