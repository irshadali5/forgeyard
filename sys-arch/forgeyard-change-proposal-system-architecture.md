# Forgeyard Change Proposal, Review & Integration System Architecture

**Document type:** Core Forgeyard System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** VCS-neutral change proposal, review, approval, policy, checks, mergeability, integration queue, submit/merge, and external forge synchronization  
**Implementation direction:** Pure Rust Forgeyard core, Dioxus UI, Axum APIs, Postcard internal protocol, RON policy/config, VCS-neutral source/snapshot model, PostgreSQL/Neon distributed metadata, Stoolap standalone metadata  
**Status:** Target production architecture  
**Position in Forgeyard:** This subsystem sits directly above the VCS-neutral source layer and directly below/alongside Pipeline IR, policy, scheduler, checks, release, and deployment systems.

---

# 1. Purpose

Forgeyard needs a first-class change-review system because source changes are the control point connecting:

```text
source
review
CI
policy
security
reproducibility
integration
release
```

A conventional Pull Request implementation is too Git/provider-specific.

Forgeyard therefore uses the neutral domain concept:

```text
ChangeProposal
```

which can represent:

```text
GitHub Pull Request
GitLab Merge Request
Forgejo/Gitea Pull Request
Mercurial review proposal
Jujutsu logical change proposal
Pijul/Darcs change proposal
Forgeyard-native proposal
```

The system must not assume:

```text
branch
commit
merge
```

are universal concepts.

Instead, it builds around:

```text
proposal source state
proposal target state
canonical SourceSnapshotId
review evidence
check evidence
policy evidence
integration candidate
submit result
```

---

# 2. Central Architectural Rule

> **A Forgeyard approval, check result, mergeability decision, and integration decision is always bound to an exact immutable source snapshot or integration snapshot—not merely to a mutable branch/ref/proposal number.**

Therefore:

```text
proposal update
   ↓
new SourceSnapshotId
   ↓
previous evidence evaluated
   ↓
retain / invalidate / partially invalidate
   ↓
new checks/review if required
```

---

# 3. Core Design Goals

Forgeyard Change Proposal MUST:

1. remain VCS-neutral;
2. support external and Forgeyard-native proposals;
3. bind review/check evidence to immutable snapshots;
4. support base/head revisions and snapshots;
5. support logical change identities;
6. support patch-centric VCS backends;
7. support draft/open/closed lifecycle;
8. separate lifecycle/review/check/policy/integration states;
9. support inline review comments;
10. support threaded discussions;
11. support suggestions;
12. support approvals;
13. support changes-requested reviews;
14. support CODEOWNERS-like ownership policy;
15. support path/domain scoped approvals;
16. support required checks;
17. support custom checks;
18. support policy gates;
19. support security findings;
20. support dependency/SBOM diffs;
21. support performance/size/coverage diffs;
22. support mergeability evaluation;
23. support integration candidates;
24. support merge/rebase/squash/backend-native submission;
25. support merge/submit queue;
26. support speculative integration;
27. support queue batching;
28. support stale-base handling;
29. support conflict detection;
30. support exact tested-integration submission;
31. support external provider synchronization;
32. support webhook idempotency;
33. support concurrency/race protection;
34. support auditability;
35. support enterprise approval policy;
36. support multi-tenancy;
37. support notifications/events;
38. support CLI/API/UI;
39. work in standalone mode;
40. work in distributed mode.

---

# 4. Non-Goals

Forgeyard Change Proposal does not initially need to replace GitHub/GitLab/Forgejo review UIs. It can first import and synchronize those systems, then progressively enable Forgeyard-native review.

The system also does not implement universal VCS merge semantics itself. Integration semantics belong to the VCS backend.

---

# 5. Architectural Position

```text
                VCS / Source Layer
                       │
                       ▼
                Change Proposal
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
        Review       Checks       Policy
          │            │            │
          └────────────┼────────────┘
                       ▼
                  Mergeability
                       │
                       ▼
              Integration Candidate
                       │
                       ▼
                Integration Queue
                       │
                       ▼
                 Submit / Merge
                       │
                       ▼
                Resulting Revision
                       │
                       ▼
                 Release Pipeline
```

---

# 6. Suggested Workspace

```text
crates/
├── change/
│   ├── forgeyard-change/
│   ├── forgeyard-change-model/
│   ├── forgeyard-change-store/
│   ├── forgeyard-change-service/
│   ├── forgeyard-change-review/
│   ├── forgeyard-change-comment/
│   ├── forgeyard-change-approval/
│   ├── forgeyard-change-ownership/
│   ├── forgeyard-change-check/
│   ├── forgeyard-change-policy/
│   ├── forgeyard-change-mergeability/
│   ├── forgeyard-change-integration/
│   ├── forgeyard-change-queue/
│   ├── forgeyard-change-provider/
│   ├── forgeyard-change-events/
│   ├── forgeyard-change-notification/
│   └── forgeyard-change-audit/
```

Physical crate count can later be consolidated.

---

# 7. Dependency Direction

```text
forgeyard-core
      ↑
forgeyard-vcs-model
      ↑
forgeyard-change-model
      ↑
change services
      ↑
provider adapters / API / UI
```

Core must never know GitHub PR numbers, GitLab IIDs, or Git merge refs as primary domain identity.

---

# 8. ChangeProposalId

```rust
pub struct ChangeProposalId(Ulid);
```

Do not encode repository + provider PR number into the internal ID. Provider IDs are aliases.

---

# 9. Change Proposal Model

```rust
pub struct ChangeProposal {
    pub id: ChangeProposalId,
    pub repository: RepositoryId,
    pub source: ProposalSource,
    pub target: ProposalTarget,
    pub title: ProposalTitle,
    pub description: ProposalDescription,
    pub author: PrincipalId,
    pub lifecycle: ProposalLifecycle,
    pub current_revision: ProposalRevisionId,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub provider_binding: Option<ProviderProposalBinding>,
}
```

---

# 10. Proposal Revision

A proposal itself evolves. Each meaningful update creates an immutable revision:

```rust
pub struct ProposalRevision {
    pub id: ProposalRevisionId,
    pub proposal: ChangeProposalId,
    pub source_revision: ResolvedRevision,
    pub source_snapshot: SourceSnapshotId,
    pub target_revision: ResolvedRevision,
    pub target_snapshot: SourceSnapshotId,
    pub change_set: ChangeSetId,
    pub created_at: Timestamp,
}
```

This preserves:

```text
Proposal #42
  rev1 -> Snapshot A
  rev2 -> Snapshot B
  rev3 -> Snapshot C
```

Approvals and checks can bind to exact proposal revisions and snapshots.

---

# 11. Proposal Source and Target

```rust
pub struct ProposalSource {
    pub selector: RevisionSpec,
    pub resolved: Option<ResolvedRevision>,
}

pub struct ProposalTarget {
    pub selector: RevisionSpec,
    pub resolved: Option<ResolvedRevision>,
}
```

These can represent Git branches, Mercurial bookmarks, Jujutsu ChangeIds, Pijul channels/changes, exact revisions, or provider-supplied source states.

---

# 12. Lifecycle and Composite Status

```rust
pub enum ProposalLifecycle {
    Draft,
    Open,
    Closed,
    Integrated,
    Superseded,
    Abandoned,
}
```

Lifecycle is separate from review/check/policy/integration state.

```rust
pub struct ChangeProposalStatus {
    pub lifecycle: ProposalLifecycle,
    pub review: ReviewStatus,
    pub checks: CheckAggregateStatus,
    pub policy: PolicyStatus,
    pub mergeability: MergeabilityStatus,
    pub integration: IntegrationStatus,
}
```

This avoids giant impossible state enums.

---

# 13. Review State

```rust
pub enum ReviewStatus {
    NotRequired,
    AwaitingReview,
    ChangesRequested,
    Approved,
    PartiallyApproved,
    Stale,
}
```

---

# 14. Checks State

```rust
pub enum CheckAggregateStatus {
    NotConfigured,
    Pending,
    Running,
    Passed,
    Failed,
    Cancelled,
    Stale,
}
```

---

# 15. Policy State

```rust
pub enum PolicyStatus {
    NotEvaluated,
    Passing,
    Failing,
    RequiresException,
    ExceptionGranted,
}
```

---

# 16. Mergeability State

```rust
pub enum MergeabilityStatus {
    Unknown,
    Evaluating,
    Mergeable,
    Conflicted,
    StaleBase,
    PolicyBlocked,
    ReviewBlocked,
    ChecksBlocked,
    BackendBlocked,
}
```

---

# 17. Integration State

```rust
pub enum IntegrationStatus {
    NotRequested,
    CandidatePending,
    CandidateReady,
    Queued,
    Testing,
    ReadyToSubmit,
    Submitting,
    Submitted,
    Failed,
    Superseded,
}
```

---

# 18. Review Model

```rust
pub struct Review {
    pub id: ReviewId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub reviewer: PrincipalId,
    pub verdict: ReviewVerdict,
    pub body: Option<String>,
    pub scope: ReviewScope,
    pub created_at: Timestamp,
}

pub enum ReviewVerdict {
    Comment,
    Approve,
    RequestChanges,
}
```

---

# 19. Review Scope

```rust
pub enum ReviewScope {
    WholeProposal,
    Paths(PathScope),
    Domains(DomainScope),
    Policy(PolicyScope),
}
```

This supports path-aware and domain-aware approvals.

---

# 20. Approval Binding

```rust
pub struct Approval {
    pub id: ApprovalId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub snapshot: SourceSnapshotId,
    pub reviewer: PrincipalId,
    pub scope: ApprovalScope,
    pub created_at: Timestamp,
}
```

Approval authority is snapshot-bound.

---

# 21. Approval Invalidation

```rust
pub enum ApprovalInvalidationPolicy {
    AlwaysInvalidate,
    InvalidateIfScopedPathsChanged,
    PreserveIfOnlyNonOwnedPathsChanged,
    PreserveManual,
}
```

Example:

```text
reviewer approved crates/security/**
new update changes only docs/**
```

Policy may preserve security approval.

---

# 22. Discussion Threads

```rust
pub struct DiscussionThread {
    pub id: DiscussionThreadId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub anchor: DiscussionAnchor,
    pub state: ThreadState,
}
```

```rust
pub enum DiscussionAnchor {
    Proposal,
    File { path: CanonicalRepoPath },
    Line {
        path: CanonicalRepoPath,
        side: DiffSide,
        line: u32,
    },
    Symbol {
        path: CanonicalRepoPath,
        symbol: SymbolAnchor,
    },
}
```

---

# 23. Thread State

```rust
pub enum ThreadState {
    Open,
    Resolved,
    Outdated,
}
```

Line numbers are unstable. Persist path, old/new blob IDs, line context hash, optional symbol anchor, and proposal revision so UI can attempt safe relocation.

If an anchor cannot map to the latest revision, mark it `Outdated`; never silently move it to unrelated code.

---

# 24. Comments and Suggestions

```rust
pub struct ReviewComment {
    pub id: CommentId,
    pub thread: DiscussionThreadId,
    pub author: PrincipalId,
    pub body: String,
    pub created_at: Timestamp,
    pub edited_at: Option<Timestamp>,
}
```

```rust
pub struct CodeSuggestion {
    pub thread: DiscussionThreadId,
    pub base_snapshot: SourceSnapshotId,
    pub path: CanonicalRepoPath,
    pub range: SourceRange,
    pub replacement: Vec<u8>,
}
```

Applying a suggestion creates a new source state, ProposalRevision, and SourceSnapshotId.

---

# 25. Ownership System

Forgeyard needs a provider-neutral CODEOWNERS-like model.

```rust
pub struct OwnershipRule {
    pub pattern: PathPattern,
    pub owners: Vec<OwnerRef>,
    pub requirements: OwnershipRequirement,
}
```

```rust
pub enum OwnerRef {
    User(PrincipalId),
    Team(TeamId),
    Role(RoleId),
    Dynamic(DynamicOwnerRule),
}
```

```rust
pub struct OwnershipRequirement {
    pub approvals_required: u16,
    pub require_distinct_owners: bool,
    pub dismissal_policy: ApprovalInvalidationPolicy,
}
```

---

# 26. Ownership Sources

Forgeyard can import:

```text
CODEOWNERS
GitLab CODEOWNERS
Forgeyard RON ownership rules
organization ownership directory
```

into one internal model.

Ownership resolution:

```text
changed paths
   ↓
ownership matcher
   ↓
required owner groups
   ↓
approval requirements
```

---

# 27. Domain Ownership

Beyond path ownership, Forgeyard can define domains:

```text
security
billing
authentication
release
platform/apple
platform/windows
schema
migration
```

Domain ownership may require independent approval even when file ownership overlaps.

---

# 28. CheckRun

```rust
pub struct CheckRun {
    pub id: CheckRunId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub snapshot: SourceSnapshotId,
    pub kind: CheckKind,
    pub state: CheckState,
    pub run: Option<RunId>,
    pub evidence: Vec<EvidenceRef>,
    pub required: bool,
    pub started_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
}
```

---

# 29. Check Kinds

```rust
pub enum CheckKind {
    Build,
    UnitTest,
    IntegrationTest,
    EndToEndTest,
    Lint,
    Format,
    StaticAnalysis,
    Security,
    DependencyPolicy,
    LicensePolicy,
    Coverage,
    Benchmark,
    BinarySize,
    Reproducibility,
    Packaging,
    IntegrationCandidate,
    Custom(CheckKindId),
}
```

---

# 30. Check States

```rust
pub enum CheckState {
    Queued,
    Running,
    Passed,
    Failed,
    Cancelled,
    TimedOut,
    Skipped,
    Stale,
}
```

---

# 31. Check Evidence

```rust
pub enum EvidenceRef {
    Artifact(StoreObjectId),
    TestReport(TestReportId),
    SecurityReport(SecurityReportId),
    Sbom(SbomId),
    Provenance(ProvenanceId),
    Benchmark(BenchmarkReportId),
    Coverage(CoverageReportId),
    Reproduction(ReproductionEvidenceId),
}
```

---

# 32. Required Checks

```rust
pub struct RequiredCheckRule {
    pub condition: ProposalCondition,
    pub checks: Vec<CheckRequirement>,
}
```

Examples:

```text
all proposals -> build + tests
security/** changed -> security scan
release/** changed -> reproducibility
native/** changed -> ABI/linkage checks
```

---

# 33. Conditional Check Planning

```text
changed paths
+
ecosystem graph
+
policy
    ↓
required check plan
```

Reuse previous evidence only when relevant SourceSnapshotId, derivation/check identity, policy version, and evidence scope all match.

---

# 34. Stale Check Protection

When source changes, old checks become stale unless the scoped derivation is provably unchanged and policy explicitly allows evidence reuse.

A check completion must include proposal revision, snapshot, and attempt identity. A stale result can never mark the latest proposal green.

---

# 35. Policy Engine Integration

Change Proposal invokes Forgeyard policy for:

```text
review requirements
ownership
required checks
dependency policies
security policies
license policies
integration strategy
queue requirements
release constraints
exceptions
```

```rust
pub struct ChangePolicyInput {
    pub proposal: ChangeProposalId,
    pub revision: ProposalRevisionId,
    pub source_snapshot: SourceSnapshotId,
    pub target_snapshot: SourceSnapshotId,
    pub changes: ChangeSetId,
    pub author: PrincipalId,
    pub labels: BTreeSet<Label>,
    pub repository: RepositoryId,
}
```

---

# 36. Policy Decision

```rust
pub struct ChangePolicyDecision {
    pub allowed: bool,
    pub requirements: Vec<PolicyRequirement>,
    pub violations: Vec<PolicyViolation>,
    pub decision_digest: Digest,
}
```

Every decision records policy bundle/version digest for audit.

---

# 37. Policy Exceptions

```rust
pub struct PolicyException {
    pub id: PolicyExceptionId,
    pub violation: PolicyViolationId,
    pub granted_by: PrincipalId,
    pub reason: String,
    pub expires_at: Option<Timestamp>,
}
```

Do not mutate a failing policy into a passing policy. Preserve the violation plus explicit exception evidence.

---

# 38. Semantic Change Evidence

The proposal view can aggregate ecosystem-specific evidence:

```text
dependency diff
SBOM diff
vulnerability diff
license diff
coverage diff
benchmark diff
binary/package-size diff
API/ABI diff
schema/migration diff
protocol compatibility diff
```

These are evidence sources, not review authority by themselves.

---

# 39. Dependency Diff

Examples:

```text
+ crate/package
- package
version upgrade
source change
new build script
new proc macro
new native dependency
new lifecycle script
new annotation processor
```

---

# 40. SBOM / Vulnerability / License Diff

Prefer deltas:

```text
new components
removed components
new findings
resolved findings
new licenses
removed licenses
policy-impacting changes
```

rather than only totals.

---

# 41. Performance and Size Diff

Benchmark evidence includes runner class and confidence/noise metadata.

Binary/package size can compare:

```text
binary size
package size
section size
asset size
```

by target.

---

# 42. Mergeability Engine

Mergeability is derived, never manually assigned.

```rust
pub struct MergeabilityDecision {
    pub status: MergeabilityStatus,
    pub reasons: Vec<MergeBlockReason>,
    pub evaluated_revision: ProposalRevisionId,
    pub target_revision: RevisionKey,
    pub policy_digest: Digest,
}
```

---

# 43. Merge Block Reasons

```rust
pub enum MergeBlockReason {
    Draft,
    ReviewRequired,
    ChangesRequested,
    ApprovalMissing,
    RequiredCheckPending,
    RequiredCheckFailed,
    PolicyViolation,
    Conflict,
    BaseOutdated,
    QueueRequired,
    ProviderRestriction,
    BackendRestriction,
}
```

---

# 44. Integration Strategy

```rust
pub enum IntegrationStrategy {
    FastForward,
    Merge,
    Rebase,
    Squash,
    BackendNative(BackendIntegrationKind),
}
```

The VCS backend owns execution semantics.

---

# 45. VCS Integration Backend

```rust
#[async_trait]
pub trait VcsIntegrationBackend {
    async fn prepare_candidate(
        &self,
        request: &IntegrationRequest,
    ) -> Result<IntegrationCandidate, IntegrationError>;

    async fn submit_candidate(
        &self,
        candidate: &IntegrationCandidate,
        expected_target: &ResolvedRevision,
    ) -> Result<IntegrationResult, IntegrationError>;
}
```

---

# 46. Integration Candidate

```rust
pub struct IntegrationCandidate {
    pub id: IntegrationCandidateId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub base_revision: ResolvedRevision,
    pub source_revision: ResolvedRevision,
    pub strategy: IntegrationStrategy,
    pub result_snapshot: SourceSnapshotId,
    pub native_candidate: NativeIntegrationCandidate,
    pub created_at: Timestamp,
}
```

Candidates are immutable. Changed source/base produces a new candidate.

---

# 47. Candidate CI

The critical safety pattern is:

```text
proposal source passes CI
        ↓
create integration candidate
        ↓
candidate snapshot passes CI
        ↓
submit same candidate state
```

This prevents a proposal from passing against an old base and then entering a changed target untested.

---

# 48. Integration Snapshot Identity

Reuse `SourceSnapshotId` for candidate content. The semantic meaning is carried by candidate metadata, not by inventing a second tree digest scheme.

---

# 49. Submit Precondition

Before submit:

```text
current target revision == candidate expected base revision
```

unless queue/backend policy explicitly reprepares the candidate.

Conceptually submission behaves like:

```text
CAS(target_ref, expected_base, candidate_result)
```

where the VCS can provide equivalent semantics.

---

# 50. Base Drift

If target moves:

```text
candidate -> Superseded
```

then Forgeyard reprepares and retests according to queue policy.

---

# 51. Integration Queue

```rust
pub struct IntegrationQueue {
    pub id: IntegrationQueueId,
    pub repository: RepositoryId,
    pub target: RevisionSpec,
    pub policy: QueuePolicy,
}
```

```rust
pub struct QueueEntry {
    pub id: QueueEntryId,
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub priority: QueuePriority,
    pub enqueued_at: Timestamp,
    pub state: QueueEntryState,
}
```

---

# 52. Queue State

```rust
pub enum QueueEntryState {
    Waiting,
    Preparing,
    Testing,
    Ready,
    Submitting,
    Submitted,
    Failed,
    Superseded,
    Removed,
}
```

---

# 53. Queue Policy

```rust
pub struct QueuePolicy {
    pub mode: QueueMode,
    pub batching: BatchPolicy,
    pub stale_base: StaleBasePolicy,
    pub failure: QueueFailurePolicy,
    pub max_parallel_candidates: u16,
}
```

```rust
pub enum QueueMode {
    Serial,
    Speculative,
    Batched,
}
```

---

# 54. Serial Queue

```text
main
 ↓
A candidate/test/submit
 ↓
main+A
 ↓
B candidate/test/submit
```

This is the safest initial production mode.

---

# 55. Speculative Queue

Can prepare predicted chains:

```text
A on main
B on predicted main+A
C on predicted main+A+B
```

If A fails, dependent predictions are invalidated.

---

# 56. Batched Queue

Combine A+B+C into one candidate for throughput. If the batch fails, adaptive bisect policy can identify the failing proposal(s).

```rust
pub enum BatchPolicy {
    Disabled,
    MaxSize(u16),
    Adaptive {
        max_size: u16,
        failure_bisect: bool,
    },
}
```

---

# 57. Queue Priority and Fairness

```rust
pub enum QueuePriority {
    Normal,
    Elevated,
    ReleaseCritical,
    Emergency,
}
```

Priority changes require permission and audit.

Fairness can combine FIFO, priority aging, repository fairness, and tenant fairness.

---

# 58. Queue Failure Policy

```rust
pub enum QueueFailurePolicy {
    RemoveFailed,
    PauseQueue,
    RetryTransient,
    BisectBatch,
}
```

Distinguish deterministic failures from infrastructure/transient failures before retrying.

---

# 59. Queue Leases

```rust
pub struct IntegrationLease {
    pub lease_id: LeaseId,
    pub entry: QueueEntryId,
    pub worker: WorkerId,
    pub expires_at: Timestamp,
}
```

Leases prevent duplicate submission work.

---

# 60. Idempotency

State-changing commands accept idempotency keys:

```text
create proposal
submit review
enqueue
prepare candidate
submit candidate
provider sync
```

---

# 61. External Provider Mode

```text
External PR/MR
    ↓
provider adapter
    ↓
Forgeyard ChangeProposal
    ↓
checks/review evidence
    ↓
status/comment sync
```

External provider may remain the source of review UX initially.

---

# 62. Forgeyard-Native Mode

```text
VCS repository
      ↓
Forgeyard ChangeProposal
      ↓
Forgeyard reviews/checks/integration
      ↓
VCS submit
```

---

# 63. Hybrid Mode

Possible configurations:

```text
external proposal + Forgeyard-owned policy/queue
Forgeyard-native proposal + external issue tracker
provider-owned review + Forgeyard submit authority
```

---

# 64. Provider Binding

```rust
pub struct ProviderProposalBinding {
    pub provider: ProviderId,
    pub external_repository: ExternalRepositoryId,
    pub external_proposal: ExternalProposalId,
    pub sync_mode: ProviderSyncMode,
}

pub enum ProviderSyncMode {
    ImportOnly,
    Bidirectional,
    StatusOnly,
    ForgeyardAuthoritative,
}
```

---

# 65. Provider Adapter

```rust
#[async_trait]
pub trait ChangeProviderAdapter {
    async fn import_proposal(
        &self,
        event: ProviderProposalEvent,
    ) -> Result<ProviderProposalSnapshot>;

    async fn publish_check(
        &self,
        update: ProviderCheckUpdate,
    ) -> Result<()>;

    async fn publish_comment(
        &self,
        comment: ProviderCommentUpdate,
    ) -> Result<()>;

    async fn publish_review(
        &self,
        review: ProviderReviewUpdate,
    ) -> Result<()>;
}
```

Provider is separate from VCS: GitHub/GitLab/Forgejo are providers; Git is the VCS.

---

# 66. Provider Webhook Handling

```text
receive
 ↓
verify signature
 ↓
dedupe delivery ID
 ↓
normalize event
 ↓
resolve exact source state
 ↓
append proposal revision
 ↓
schedule reevaluation
```

---

# 67. Change Events

```rust
pub enum ChangeEvent {
    ProposalCreated,
    ProposalUpdated,
    ProposalClosed,
    ProposalReopened,
    ReviewSubmitted,
    ThreadCreated,
    ThreadResolved,
    CheckUpdated,
    PolicyEvaluated,
    CandidateCreated,
    QueueEntered,
    QueueRemoved,
    IntegrationSubmitted,
}
```

---

# 68. Event Envelope

```rust
pub struct EventEnvelope<T> {
    pub schema_version: u16,
    pub event_id: EventId,
    pub occurred_at: Timestamp,
    pub actor: ActorRef,
    pub payload: T,
}
```

Use append-only audit/event history even when current state is relational.

---

# 69. Internal Protocol and Formats

Use:

```text
Postcard -> internal daemon/service messages
RON      -> human policy/configuration
JSON     -> public/provider APIs where required
```

---

# 70. Storage Model

Distributed mode:

```text
PostgreSQL / Neon
```

Standalone mode:

```text
Stoolap
```

Both behind the same store trait.

---

# 71. ChangeStore

```rust
#[async_trait]
pub trait ChangeStore {
    async fn create_proposal(...);
    async fn append_revision(...);
    async fn put_review(...);
    async fn put_comment(...);
    async fn put_check(...);
    async fn put_policy_decision(...);
    async fn put_candidate(...);
    async fn update_queue_entry(...);
}
```

---

# 72. Suggested Tables

```text
change_proposals
change_proposal_revisions
change_reviews
change_approvals
change_threads
change_comments
change_suggestions
change_ownership_requirements
change_checks
change_policy_decisions
change_policy_exceptions
integration_candidates
integration_queues
integration_queue_entries
provider_proposal_bindings
provider_sync_state
change_events
change_audit
```

Bulk artifacts remain in CAS.

---

# 73. Concurrency Control

Use optimistic entity versioning for mutable metadata:

```rust
pub struct EntityVersion(u64);
```

Updates require expected version to prevent lost writes.

Proposal revisions themselves are append-only.

---

# 74. Review Race Protection

Review submission includes expected proposal revision. If source changed during review, record the review against the old revision and mark it stale according to policy. Never silently attach it to the latest revision.

---

# 75. Check Race Protection

Check completion includes proposal revision, source snapshot, and run attempt. Stale results cannot alter the latest proposal status.

---

# 76. Integration Race Protection

Candidate submit includes expected target revision and result snapshot. Stale target blocks submission.

---

# 77. Notifications

Events may trigger:

```text
review requested
changes requested
approval obtained
check failed
queue entered
queue blocked
proposal integrated
```

Notification channels are adapters.

Deduplicate by event ID + recipient + notification kind.

---

# 78. Review Assignment

Reviewer assignment sources:

```text
manual request
ownership rules
domain rules
load balancing
round-robin
expertise mapping
```

Eligibility checks permissions, self-approval policy, team membership, conflict policy, and required role.

---

# 79. Separation of Duties

Enterprise policy may require:

```text
author != approver
approver != submitter
security approver distinct from code owner
```

```rust
pub struct ApprovalQuorum {
    pub minimum: u16,
    pub distinct_teams: Option<u16>,
    pub required_roles: Vec<RoleId>,
}
```

---

# 80. Labels and Risk

```rust
pub struct Label {
    pub key: LabelKey,
    pub value: Option<LabelValue>,
}

pub enum ChangeRisk {
    Low,
    Medium,
    High,
    Critical,
}
```

Risk can derive from paths, dependency changes, sensitive modules, migration/schema changes, binary changes, and release files.

---

# 81. High-Risk Policy Example

```text
risk:critical
    requires:
      2 owners
      security approval
      full integration tests
      reproducibility
      integration queue
```

---

# 82. Breaking Change Evidence

Ecosystem adapters can contribute:

```text
public API diff
schema diff
protocol diff
ABI diff
```

as check evidence.

---

# 83. Migration / Protocol Review

If migration files change, require migration validation and rollback/compatibility evidence.

If Postcard/API schema changes, require rolling-upgrade/protocol compatibility checks.

---

# 84. Security-Sensitive Paths

Examples:

```text
auth/**
crypto/**
policy/**
secrets/**
sandbox/**
signing/**
```

can require stronger approvals and checks.

---

# 85. Secret Detection

Secret scan can produce blocking check evidence before integration.

---

# 86. Generated File Awareness

If generated files changed, ecosystem adapters can verify their generator inputs also changed or regenerate-and-compare.

---

# 87. Docs-Only Optimization

Policy may reduce checks only after VCS-neutral snapshot diff proves the proposal is truly docs-only.

---

# 88. Reproducibility Evidence

High-assurance proposal can attach:

```text
primary artifact digest
independent rebuild digest
comparison state
```

Release-critical integration candidates may require their own reproduction evidence.

---

# 89. Artifact Previews

Proposal can expose preview artifacts:

```text
web preview
desktop package
APK
docs
benchmark report
SBOM
```

with retention policy.

Preview deployments use ephemeral environments and no production secrets by default.

---

# 90. Integration Candidate Provenance

```rust
pub struct IntegrationProvenance {
    pub proposal_revision: ProposalRevisionId,
    pub base_revision: RevisionKey,
    pub source_revision: RevisionKey,
    pub strategy: IntegrationStrategy,
    pub result_snapshot: SourceSnapshotId,
    pub backend_version: ToolIdentity,
}
```

---

# 91. Integration Result

```rust
pub struct IntegrationResult {
    pub resulting_revision: ResolvedRevision,
    pub resulting_snapshot: SourceSnapshotId,
    pub submitted_at: Timestamp,
    pub actor: PrincipalId,
}
```

After submit:

```text
materialize resulting revision
  ↓
canonicalize
  ↓
must equal candidate SourceSnapshotId
```

If not, raise `IntegrationContentViolation`.

---

# 92. Backend Hook Effects

If VCS/provider hooks alter resulting source state, Forgeyard detects snapshot mismatch. It must never claim that the tested candidate was integrated if final source differs.

Commit metadata may differ while tree content stays equal; validate commit/signature metadata separately if policy requires it.

---

# 93. Protected Targets

Target protection can require:

```text
queue only
minimum approvals
signed integration
no direct writes
specific submitters
```

---

# 94. Direct Write Detection

If protected target changes outside Forgeyard queue:

```text
audit event
policy alert
optional target freeze
```

according to repository policy and provider/VCS integration.

---

# 95. Emergency Bypass

Break-glass integration may exist with:

```text
special permission
mandatory reason
strong auth/MFA where available
immutable audit
post-submit checks
```

Never silently bypass requirements.

---

# 96. Proposal Dependencies and Stacks

```rust
pub struct ProposalDependency {
    pub prerequisite: ChangeProposalId,
    pub kind: ProposalDependencyKind,
}

pub enum ProposalDependencyKind {
    MustIntegrateFirst,
    StackParent,
    Related,
}
```

Useful for Jujutsu stacks, Git stacked branches, Mercurial bookmark chains, and logical review dependencies.

---

# 97. Stack Update

When parent proposal changes, child source may change. Append a new ProposalRevision and invalidate/reuse evidence according to snapshot impact.

---

# 98. Supersession and Duplicate Detection

One proposal may supersede another without destroying history.

Optional duplicate heuristics may compare same target + same SourceSnapshotId or highly similar ChangeSet, but must never auto-close solely on heuristic confidence.

---

# 99. Search

Index:

```text
title
description
author
labels
paths
reviewers
status
check failures
revision IDs
SourceSnapshotId
```

---

# 100. REST API

Potential endpoints:

```text
POST /v1/change-proposals
GET  /v1/change-proposals/{id}
PATCH /v1/change-proposals/{id}

GET  /v1/change-proposals/{id}/revisions
GET  /v1/change-proposals/{id}/diff

POST /v1/change-proposals/{id}/reviews
POST /v1/change-proposals/{id}/threads
POST /v1/change-proposals/{id}/comments

GET  /v1/change-proposals/{id}/checks
GET  /v1/change-proposals/{id}/policy
GET  /v1/change-proposals/{id}/mergeability

POST /v1/change-proposals/{id}/integrate
POST /v1/change-proposals/{id}/queue
DELETE /v1/change-proposals/{id}/queue
```

---

# 101. CLI

```text
forgeyard change create
forgeyard change show
forgeyard change list
forgeyard change diff
forgeyard change update
forgeyard change close

forgeyard change review
forgeyard change approve
forgeyard change request-changes
forgeyard change comment
forgeyard change resolve-thread

forgeyard change checks
forgeyard change policy
forgeyard change explain-block

forgeyard change candidate
forgeyard change queue
forgeyard change unqueue
forgeyard change integrate
```

---

# 102. `forgeyard change show`

Display:

```text
proposal
author
target
source
current snapshot
changed paths
review state
check state
policy state
mergeability
queue state
```

---

# 103. `forgeyard change explain-block`

Explain exact blocking reasons such as:

```text
missing security owner approval
integration test pending
base revision stale
unresolved review thread
```

---

# 104. Dioxus UI

Primary screen:

```text
Header
├── title
├── author
├── lifecycle
├── source -> target
├── risk
└── mergeability

Tabs
├── Overview
├── Changes
├── Checks
├── Review
├── Dependencies
├── Security
├── Performance
├── Artifacts
├── Integration
└── Activity
```

---

# 105. Overview Panel

Show:

```text
files changed
packages/crates affected
dependency changes
required reviewers
approval count
required checks
policy violations
integration status
```

---

# 106. Changes View

Support:

```text
unified diff
split diff
file tree
symbol navigation
whitespace toggle
generated-file marker
binary-file summary
```

---

# 107. Review UI

Support inline comments, threads, resolve/reopen, suggestions, review summary, approve, and request changes.

---

# 108. Checks UI

Each check displays state, duration, runner, logs, artifacts, evidence, retry history, and stale/current marker.

---

# 109. Integration UI

Display strategy, candidate snapshot, base revision, candidate checks, queue position, and expected submit target.

---

# 110. Activity Timeline

Append-only timeline:

```text
proposal opened
source updated
review submitted
approval invalidated
check failed/passed
policy exception
queue entry
candidate created
submitted
```

---

# 111. Mobile UI

Dioxus mobile view prioritizes status, required action, review comments, approvals, check failures, and queue state. Detailed diff uses file-by-file navigation.

---

# 112. Permissions

Suggested permissions:

```text
change.create
change.edit
change.close
change.review
change.approve
change.manage_owners
change.override_policy
change.enqueue
change.submit
change.admin
```

Roles are convenience bundles; permissions are authoritative.

---

# 113. Multi-Tenancy

Every proposal/check/review belongs to tenant, organization, repository. Cross-tenant access is denied.

---

# 114. Comment Audit

Edits/deletes preserve audit history. User-facing deletion may show a tombstone while immutable audit records remain.

---

# 115. Rate Limits

Protect against comment spam, review spam, candidate churn, provider sync loops, and queue abuse.

---

# 116. Provider Sync Loop Prevention

Outbound provider updates carry correlation metadata where possible. Inbound echoes are deduplicated.

---

# 117. Observability

Metrics:

```text
proposal_open_duration
review_time
time_to_first_review
check_latency
mergeability_evaluation_latency
queue_wait_time
candidate_failure_rate
integration_success_rate
approval_invalidation_count
provider_sync_failure
```

---

# 118. Tracing

OTLP spans:

```text
change.create
change.resolve_revision
change.compute_diff
change.policy
change.check.plan
change.review
change.candidate.prepare
change.queue.evaluate
change.submit
```

---

# 119. Audit

Immutable audit records proposal creation/update, reviews/approvals, approval invalidation, policy exceptions, queue priority changes, candidate generation, submit/bypass, and provider sync.

---

# 120. Failure Model

```rust
pub enum ChangeFailure {
    ProposalNotFound,
    ProposalClosed,
    ProposalRevisionStale,
    PermissionDenied,
    ReviewNotAllowed,
    ApprovalStale,
    CheckFailed,
    PolicyBlocked,
    Conflict,
    BaseStale,
    CandidateSuperseded,
    QueueBlocked,
    IntegrationFailed,
    ProviderSyncFailed,
    VcsBackendFailed,
    IntegrityViolation,
}
```

---

# 121. Resilience and Reconciliation

Use reconcilers for provider sync, stuck checks, stale candidates, queue entries, lost runner attempts, and incomplete submit verification.

Do not depend on exactly-once messaging.

Use:

```text
at-least-once events
+
idempotent transitions
+
reconciliation
```

---

# 122. Provider Reconciler

Periodically compare external proposal state vs Forgeyard binding state and repair missed webhook state safely.

---

# 123. Check Reconciler

If CheckRun says Running but underlying Run is terminal, repair state.

---

# 124. Queue Reconciler

If lease expires, return entry to evaluable state unless submission may have partially succeeded.

---

# 125. Submit Reconciler

After ambiguous submit:

```text
query target
materialize result
compare candidate
```

before retrying. This prevents duplicate integration.

---

# 126. Conflict Model

```rust
pub struct IntegrationConflict {
    pub paths: Vec<CanonicalRepoPath>,
    pub native_evidence: NativeConflictEvidence,
}
```

Conflict resolution creates a new proposal/source revision; candidates are immutable.

---

# 127. External Review Import

When importing external reviews:

```text
map reviewer
map verdict
bind external review to provider head revision
resolve corresponding SourceSnapshotId
```

If exact source mapping is not reliable, mark evidence weak/stale rather than overclaiming validity.

---

# 128. Provider Approval Semantics

External providers differ in dismissal rules, team reviews, codeowners, and review state. Forgeyard preserves external evidence but evaluates its own policy independently.

---

# 129. Check Trust

```rust
pub enum CheckTrust {
    ForgeyardNative,
    TrustedExternal,
    Informational,
    Untrusted,
}
```

External check statuses may be evidence without becoming automatically trusted merge authority.

---

# 130. Mergeability Cache

Cache key:

```text
proposal revision
target revision
policy digest
review evidence digest
check evidence digest
```

Any relevant change invalidates decision.

---

# 131. Deterministic Decision

Given identical proposal revision, target revision, policy version, evidence, and explicit time inputs, mergeability should be deterministic.

---

# 132. Time-Dependent Policy

If policy depends on approval age, release window, or freeze state, include the relevant time/window identifier in the recorded decision.

---

# 133. Approval Expiry

Approvals may have explicit expiry timestamps. Do not implement hidden age-based invalidation without recording policy and expiry.

---

# 134. Freeze Windows

Policy can block integration during release/maintenance/incident freezes while allowing an explicit exception path.

---

# 135. Proposal Templates

Repository may define structured templates:

```text
bugfix
feature
security
breaking change
migration
```

---

# 136. Structured Proposal Metadata

```rust
pub struct ProposalMetadata {
    pub issue_refs: Vec<ExternalIssueRef>,
    pub risk: Option<ChangeRisk>,
    pub rollout: Option<RolloutPlanRef>,
    pub migration: Option<MigrationPlanRef>,
}
```

Issue tracker integration remains optional.

---

# 137. Release Notes

Policy may require a release-note fragment for selected changes. Release-note aggregation should consume structured proposal metadata rather than only commit-message heuristics.

---

# 138. AI Assistance Boundary

AI may optionally summarize diffs, identify likely reviewers, explain failed checks, and summarize discussion. AI is never equivalent to required human approval and never bypasses policy.

```rust
pub enum ReviewerKind {
    Human,
    AutomatedTool,
    AiAssistant,
}
```

---

# 139. Security Invariant

Proposal source is untrusted executable code even when author has repository access. CI runs in the normal untrusted workload sandbox.

---

# 140. Secret Access

Proposal jobs receive no production secrets by default. Use scoped test/preview credentials only when explicitly allowed.

Fork/external proposals additionally receive no trusted runners, signing keys, release credentials, or deployment credentials.

---

# 141. Proposal Trust

```rust
pub enum ProposalTrust {
    InternalTrustedAuthor,
    InternalUntrustedCode,
    ExternalFork,
    AutomatedDependencyUpdate,
}
```

Author trust does not make code trusted by default.

---

# 142. Bot Proposals and Auto-Integration

Machine principals may create proposals. Policy can require human review.

Auto-integration is allowed only under explicit conditions, for example:

```text
low-risk dependency patch
all checks green
no new licenses
no new build scripts
no new native deps
ownership policy permits
```

Still use integration candidate + queue + target precondition.

---

# 143. Repository Protection Policy Example

```ron
(
    target: Ref("main"),

    direct_write: Denied,

    approvals: (
        minimum: 2,
    ),

    ownership: Required,

    checks: [
        Build,
        UnitTest,
        IntegrationTest,
        Security,
    ],

    queue: Required,

    integration: (
        strategy: Squash,
    ),
)
```

---

# 144. Security Path Policy Example

```ron
(
    when: PathsMatch(["crates/security/**", "crates/auth/**"]),

    require: (
        owners: ["team:security"],
        approvals: 2,
        checks: [Security, Reproducibility],
    ),
)
```

---

# 145. Native Toolchain Policy Example

```ron
(
    when: PathsMatch(["crates/native/**", "crates/assembly/**"]),

    require: (
        checks: [
            Build,
            NativeAbi,
            RuntimeLinkage,
            Reproducibility,
        ],
    ),
)
```

---

# 146. Docs-Only Policy Example

```ron
(
    when: OnlyPaths(["docs/**"]),

    require: (
        checks: [Docs],
        approvals: 1,
    ),
)
```

Only apply after safe VCS-neutral snapshot diff.

---

# 147. Standalone Mode

```text
local VCS repo
  ↓
Forgeyard local ChangeProposal
  ↓
Stoolap metadata
  ↓
local runner
  ↓
local review/check/integration
```

No central server required.

---

# 148. Distributed Mode

```text
central daemon
  ↓
Postgres/Neon metadata
  ↓
distributed runners
  ↓
shared review/check state
  ↓
integration queue
```

---

# 149. Enterprise HA Mode

Adds OIDC/SAML/SCIM, RBAC, HA daemon, queue leadership, audit/SIEM, protected signing, multi-region CAS, and provider federation.

Queue coordination may use Forgeyard's coordination layer, but proposal business data remains in Postgres/Neon rather than Raft.

---

# 150. Compatibility and Migrations

Use versioned envelopes for internal messages, N/N-1 rolling-upgrade compatibility where practical, and expand-contract database migrations so active reviews/queues survive deployment.

---

# 151. Retention

Proposal/review metadata is normally retained long-term. Large logs/artifacts follow artifact retention policy. Audit/compliance evidence may have longer retention.

---

# 152. Backup / Restore

Backup proposal/review/check metadata, provider bindings, queue state, and audit. CAS backup/replication handles artifacts separately.

After restore, reconcile provider state, queue state, and candidate target revisions before any submit.

---

# 153. Performance Strategy

Avoid recomputing everything on every UI request. Materialize current aggregate review/check/policy/mergeability state while preserving immutable evidence/event records.

Reuse snapshot diff cache across ownership, checks, UI, and change-impact analysis.

---

# 154. Large Proposal Handling

Use pagination, lazy per-file diff, binary summaries, streaming, and server-side diff chunks. Repositories may define max diff-size policy with exception mechanism.

---

# 155. Core Service Trait

```rust
#[async_trait]
pub trait ChangeProposalService {
    async fn create(&self, cmd: CreateProposal) -> Result<ChangeProposal>;
    async fn update_source(&self, cmd: UpdateProposalSource) -> Result<ProposalRevision>;
    async fn review(&self, cmd: SubmitReview) -> Result<Review>;
    async fn evaluate(&self, id: ChangeProposalId) -> Result<ChangeProposalStatus>;
    async fn prepare_candidate(&self, cmd: PrepareCandidate) -> Result<IntegrationCandidate>;
    async fn enqueue(&self, cmd: EnqueueProposal) -> Result<QueueEntry>;
}
```

---

# 156. Policy Evaluator Trait

```rust
#[async_trait]
pub trait ChangePolicyEvaluator {
    async fn evaluate(
        &self,
        input: &ChangePolicyInput,
    ) -> Result<ChangePolicyDecision>;
}
```

---

# 157. Boundary Rules

Review services do not call SQL directly; they use store interfaces.

Provider-specific tokens/status semantics stay outside core domain crates.

Dioxus UI is a client, never mergeability authority.

Daemon/service owns policy evaluation, review validation, queue transitions, candidate creation, and submit authority.

---

# 158. Trust Boundary

```text
Browser/UI
  ↓
API
  ↓
Change service
  ↓
Policy / VCS / Scheduler
  ↓
Runner
```

Proposal code never executes inside the daemon process.

---

# 159. Security Threat Model

Threats include:

```text
stale approval replay
stale check replay
provider webhook spoofing
queue race
double submit
privilege escalation
review tampering
secret leakage
fork pipeline attack
candidate substitution
```

---

# 160. Security Protections

Approval lookup requires matching proposal revision + snapshot + scope + policy context.

Check pass never attaches by proposal ID alone.

Candidate digest/result snapshot binds queue/check/submit.

Double-submit protection uses lease + idempotency key + expected target + post-submit verification.

Provider webhook signatures are verified before normalization.

Audit events are append-only with integrity/retention policy.

---

# 161. Property Tests

Required properties:

```text
approval never applies to unrelated snapshot
stale passed check never marks latest proposal green
candidate result equals tested snapshot
submit fails if target precondition violated
queue transition graph remains valid
```

---

# 162. Fuzzing

Fuzz provider event normalization, diff anchors, RON policy parsing, state transition commands, and suggestion patch parsing.

---

# 163. Integration Tests

Test:

```text
Git + external provider
Git + Forgeyard-native review
Mercurial-native proposal
Jujutsu logical change
patch-centric VCS adapters
```

as those backends mature.

---

# 164. End-to-End Test

```text
create proposal
update proposal
run checks
request changes
update source
invalidate approval
approve
create candidate
candidate test
queue
target drift
reprepare
submit
verify resulting snapshot
```

---

# 165. Failure Injection

Inject runner loss, database failover, provider timeout, VCS timeout, queue-worker crash, and ambiguous submit responses. Reconciliation must recover without double-submit or stale green state.

---

# 166. Self-Hosting Forgeyard

Forgeyard should eventually use its own Change Proposal system:

```text
Forgeyard source change
  ↓
Forgeyard ChangeProposal
  ↓
Forgeyard Rust CI
  ↓
review + policy
  ↓
Forgeyard IntegrationCandidate
  ↓
Forgeyard queue
  ↓
submit exact tested candidate
```

---

# 167. Bootstrap Transition

```text
Phase A:
external PR + Forgeyard checks

Phase B:
external PR + Forgeyard policy/queue

Phase C:
Forgeyard-native review optional

Phase D:
Forgeyard-native review/integration for Forgeyard itself
```

---

# 168. Implementation Phase 1 — Core Model

Implement ChangeProposalId, ProposalRevision, composite status, review/comment/approval types.

---

# 169. Phase 2 — Snapshot/Diff Integration

Implement base/head resolution, SourceSnapshotId binding, snapshot diff, and append-only proposal revisions.

---

# 170. Phase 3 — Checks

Implement CheckRun, Pipeline IR linkage, required checks, stale-evidence rules.

---

# 171. Phase 4 — Review

Implement threads, comments, review verdict, approval binding, invalidation.

---

# 172. Phase 5 — Ownership

Implement path rules, team/role ownership, CODEOWNERS import, required approvals.

---

# 173. Phase 6 — Policy

Implement policy decisions, risk classification, exceptions, and conditional requirements.

---

# 174. Phase 7 — Integration Candidate

Implement VCS integration backend trait, candidate snapshot, candidate CI, post-build evidence.

---

# 175. Phase 8 — Serial Queue

Implement serial queue, leases, compare-and-submit target precondition, retry/reconciliation.

---

# 176. Phase 9 — External Provider

Implement first provider adapter for proposal import, webhooks, status/check publishing, and comments while keeping provider types outside core.

---

# 177. Phase 10 — Advanced Queue

Implement speculative queue, batching, adaptive bisect, priorities.

---

# 178. Phase 11 — Native Review UI

Implement full Dioxus diff, threads, approvals, checks, candidate, and queue UI.

---

# 179. Phase 12 — Enterprise Hardening

Implement OIDC/RBAC, separation of duties, audit/SIEM, HA queue, break-glass, and multi-tenant protections.

---

# 180. Acceptance Tests

1. Approval binds to exact snapshot.
2. New source revision invalidates approval according to policy.
3. Unchanged approval scope may remain valid when scoped policy permits.
4. Stale check cannot mark new proposal revision green.
5. Required failed check blocks mergeability.
6. Missing owner approval blocks mergeability.
7. Policy exception preserves violation + grant separately.
8. Integration candidate has immutable result snapshot.
9. Candidate CI runs against integration result, not only proposal head.
10. Target movement supersedes stale candidate.
11. Queue never submits against unexpected target revision.
12. Post-submit snapshot equals candidate snapshot.
13. Provider duplicate webhook is idempotent.
14. Provider review maps to exact source snapshot or is marked weak/stale.
15. Fork proposal receives no privileged secrets by default.
16. Queue worker crash does not double-submit.
17. Ambiguous submit is reconciled before retry.
18. CODEOWNERS import maps to Forgeyard ownership model.
19. Jujutsu change update creates new ProposalRevision without losing ChangeId.
20. Patch-centric VCS can provide proposal source without fake Git commit semantics.
21. Integration queue works in standalone mode.
22. Same core model works with Postgres distributed mode.
23. Forgeyard can process a proposal for its own codebase.
24. Audit reconstructs all review/queue/submit decisions.

---

# 181. Production Readiness Gates

Do not call Change Proposal production-ready until:

```text
snapshot-bound approvals are correct
stale check isolation is correct
ownership matching is deterministic
policy versioning works
candidate/result snapshot identity is stable
target compare-and-submit is safe
serial integration queue is reliable
post-submit verification works
reconciliation handles crashes
provider webhook verification/idempotency works
permissions/RBAC are enforced
audit is complete
```

Speculative/batched queue can remain optional after initial production readiness.

---

# 182. Architectural Invariants

1. Change Proposal is VCS-neutral.
2. Proposal number/provider ID is not internal identity.
3. Proposal updates append immutable ProposalRevision records.
4. Approval binds to ProposalRevision + SourceSnapshotId.
5. Check result binds to ProposalRevision + SourceSnapshotId.
6. Lifecycle/review/check/policy/integration states are separate.
7. CODEOWNERS-like rules normalize into Forgeyard ownership.
8. Policy decision records policy digest.
9. Mergeability is derived.
10. Mutable branch/ref names are never enough for approval/check authority.
11. Integration semantics belong to VCS backend.
12. Integration candidate is immutable.
13. Candidate CI validates result snapshot.
14. Queue submit uses expected-target precondition.
15. Target drift invalidates/supersedes candidate.
16. Post-submit result is snapshot-verified.
17. Provider and VCS are separate abstractions.
18. External provider approval does not automatically equal Forgeyard approval.
19. Stale evidence never silently applies to latest source.
20. Required/optional checks remain distinct.
21. Policy exceptions do not erase policy violations.
22. Break-glass bypass is explicit/audited.
23. Proposal code runs as untrusted workload.
24. Fork proposals do not receive privileged secrets by default.
25. Queue transitions are lease/idempotency protected.
26. Reconciliation handles at-least-once events.
27. UI is never policy authority.
28. Postgres/Stoolap are hidden behind store interface.
29. CAS stores evidence/artifacts; SQL stores metadata/state.
30. Forgeyard should eventually dogfood this subsystem on Forgeyard itself.

---

# 183. Final Target Architecture

```text
                 VCS-Neutral Source Layer
                         │
                         ▼
                   ChangeProposal
                         │
                         ▼
                  ProposalRevision
                  ├── source snapshot
                  ├── target snapshot
                  └── change set
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
        Review          Checks         Policy
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                    Mergeability
                         │
                         ▼
              IntegrationCandidate
                  ├── exact base
                  ├── exact source
                  ├── strategy
                  └── result snapshot
                         │
                         ▼
                   Candidate CI
                         │
                         ▼
                 Integration Queue
                         │
                         ▼
            compare-and-submit target
                         │
                         ▼
                 Resulting Revision
                         │
                         ▼
             Snapshot Verification
                         │
                         ▼
               Integrated Proposal
```

---

# 184. Final Architectural Position

Proposal identity:

```text
Repository
+
Source selection
+
Target selection
+
Author/metadata
=
ChangeProposal
```

Immutable proposal state:

```text
resolved source revision
+
source SourceSnapshotId
+
resolved target revision
+
target SourceSnapshotId
+
ChangeSet
=
ProposalRevision
```

Mergeability:

```text
ProposalRevision
+
Review evidence
+
Approval evidence
+
Check evidence
+
Policy decision
+
Current target revision
=
MergeabilityDecision
```

Integration:

```text
ProposalRevision
+
Exact target revision
+
VCS-native integration strategy
=
IntegrationCandidate
+
Result SourceSnapshotId
```

Submission:

```text
Candidate passed CI
+
Target still equals expected base
+
Queue/policy permits submit
        ↓
submit candidate
        ↓
resulting VCS revision
        ↓
canonical snapshot
        ↓
must equal tested candidate snapshot
```

> **Forgeyard does not merely merge code that passed CI. Forgeyard integrates the exact source state that passed the required review, policy, and candidate checks.**

This turns Change Proposal into a trustworthy control plane between source control and production delivery rather than a thin wrapper around provider-specific Pull Requests.

---

# Appendix A — Example Change Proposal Policy

```ron
(
    target: Ref("main"),

    lifecycle: (
        draft_merge: Denied,
    ),

    review: (
        approvals: 2,
        dismiss_on_source_change: Scoped,
        unresolved_threads: Denied,
    ),

    ownership: Required,

    checks: (
        required: [
            Build,
            UnitTest,
            IntegrationTest,
            Security,
        ],
    ),

    policy: (
        exceptions: MaintainersOnly,
    ),

    integration: (
        strategy: Squash,
        queue: Required,
        candidate_ci: Required,
    ),
)
```

---

# Appendix B — Example Approval Record

```ron
(
    proposal: "cp:01...",
    revision: "cpr:01...",
    snapshot: "blake3:...",

    reviewer: "principal:alice",

    scope: Paths([
        "crates/security/**",
    ]),

    verdict: Approve,
)
```

---

# Appendix C — Example Integration Candidate

```ron
(
    proposal: "cp:01...",
    proposal_revision: "cpr:01...",

    base_revision: "vcsrev:...",
    source_revision: "vcsrev:...",

    strategy: Squash,

    result_snapshot: "blake3:...",

    state: CandidateReady,
)
```

---

# Appendix D — Example Queue Policy

```ron
(
    queue: (
        mode: Speculative,

        batching: Adaptive(
            max_size: 4,
            failure_bisect: true,
        ),

        stale_base: Reprepare,

        failure: RetryTransient,

        max_parallel_candidates: 4,
    ),
)
```

---

# Appendix E — External Provider Mapping

| Forgeyard | GitHub | GitLab | Forgejo/Gitea | VCS-native |
|---|---|---|---|---|
| ChangeProposal | Pull Request | Merge Request | Pull Request | native proposal/change |
| ProposalRevision | PR head state | MR source state | PR head state | resolved source state |
| Review | Review | Approval/review | Review | Forgeyard review |
| CheckRun | Check/status | Pipeline/status | Check/status | Forgeyard check |
| IntegrationCandidate | merge candidate | merge result candidate | merge candidate | backend candidate |
| IntegrationQueue | merge queue | merge train analogue | queue if available | Forgeyard queue |
| Integrated | merged | merged | merged | backend submit result |

Forgeyard mapping never makes provider terminology part of the core domain.

---

# Appendix F — Self-Hosting Target

Eventually Forgeyard's own development process should be:

```text
Git source repository
      ↓
Forgeyard ChangeProposal
      ↓
Forgeyard Rust CI
      ↓
Review + policy
      ↓
Forgeyard IntegrationCandidate
      ↓
candidate reproduction
      ↓
Forgeyard IntegrationQueue
      ↓
submit exact tested candidate
      ↓
Forgeyard release pipeline
```

This becomes a continuous proof that Forgeyard's VCS-neutral source, Rust ecosystem, CI, review, queue, provenance, and release architectures work together as one production system.
