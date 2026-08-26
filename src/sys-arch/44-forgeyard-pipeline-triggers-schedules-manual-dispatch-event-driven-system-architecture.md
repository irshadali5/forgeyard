# 44 — Forgeyard Pipeline Triggers, Schedules, Manual Dispatch & Event-Driven Execution System Architecture

**Document type:** Core Triggering, Scheduling, Manual Dispatch, Event Ingestion & Run Initiation System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** SCM push/change triggers, webhook-driven execution, manual dispatch, API dispatch, scheduled/cron pipelines, delayed runs, recurring workflows, system triggers, debounce/coalescing, source resolution, trigger deduplication, concurrency groups, supersession, missed-run recovery, durable timers, replay/backfill, and trigger governance  
**Architecture style:** Event-driven but state-authoritative, durable trigger intent, exact source resolution, idempotent run creation, policy-aware, timezone-correct, reconciliation-backed, and strictly subordinate to canonical planning/run semantics  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on SCM Provider Integrations, VCS-neutral source snapshots, Change Proposal, Pipeline IR, Run/Job State Machine, Events/Reconciliation, API/Axum, Policy/Authz, Configuration, Notifications, HA, and Developer Experience. This subsystem centralizes how work begins without creating an alternate execution authority.

---

# 1. Purpose

Forgeyard already knows how to:

```text
plan pipelines
schedule jobs
execute work
reconcile failures
```

but production CI/CD also requires a complete answer to:

```text
what starts a pipeline?
how does a push trigger a run?
how does a merge request update cancel obsolete work?
how do scheduled nightly jobs work?
how do timezone and DST affect cron?
how are duplicate webhooks prevented from creating duplicate runs?
how do manual runs bind exact inputs/source?
what happens when Forgeyard is down during a schedule?
how are missed recurring runs handled?
```

The central rule is:

> **A trigger creates durable intent to evaluate or start work; it does not directly create arbitrary JobAttempt or bypass the normal Pipeline IR → Run → Job state machine.**

A second rule is:

> **Every source-driven trigger resolves mutable provider/VCS references to exact immutable source identity before protected execution begins.**

A third rule is:

> **Trigger delivery is at-least-once. Duplicate events, retries, reconnects, and service restarts must not create unintended duplicate semantic runs.**

---

# 2. Architectural Position

```text
                    Trigger Sources
      ┌──────────────┼──────────────┐
      ▼              ▼              ▼
     SCM           Manual         Schedule
      │              │              │
      └──────────────┼──────────────┘
                     ▼
                Trigger Ingest
                     │
                     ▼
              Verify / Deduplicate
                     │
                     ▼
                Trigger Intent
                     │
                     ▼
                Source Resolve
                     │
                     ▼
               Policy / Planning
                     │
                     ▼
                     Run
```

---

# 3. Goals

The subsystem MUST:

1. define trigger identity;
2. define trigger source;
3. support SCM push;
4. support Change Proposal updates;
5. support tag/ref events;
6. support manual dispatch;
7. support API dispatch;
8. support schedules;
9. support cron-like recurrence;
10. support delayed one-shot execution;
11. support timezone semantics;
12. handle DST correctly;
13. support debounce;
14. support coalescing;
15. support concurrency groups;
16. support supersession;
17. support cancellation policies;
18. support idempotent run initiation;
19. support exact source resolution;
20. support missed-run handling;
21. support backfill;
22. support replay;
23. support durable timers;
24. support HA;
25. support trigger auditing;
26. support permissions/policy;
27. support notifications;
28. support CLI/UI;
29. support standalone/distributed modes;
30. remain event/reconciliation safe.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Pipeline IR
replace Run/Job state machines
replace scheduler
replace SCM provider adapters
replace policy
create arbitrary jobs directly
```

---

# 5. Workspace Structure

```text
crates/trigger/
├── forgeyard-trigger/
├── forgeyard-trigger-model/
├── forgeyard-trigger-ingest/
├── forgeyard-trigger-dedup/
├── forgeyard-trigger-resolve/
├── forgeyard-trigger-dispatch/
├── forgeyard-trigger-schedule/
├── forgeyard-trigger-concurrency/
├── forgeyard-trigger-supersede/
├── forgeyard-trigger-reconcile/
├── forgeyard-trigger-health/
└── forgeyard-trigger-testkit/
```

Time/schedule helpers may reuse core time abstractions rather than creating a second time crate.

---

# 6. TriggerDefinitionId

```rust
pub struct TriggerDefinitionId(Digest);
```

Content-derived logical trigger configuration.

---

# 7. TriggerOccurrenceId

```rust
pub struct TriggerOccurrenceId(Ulid);
```

One concrete observed firing.

---

# 8. TriggerIntentId

```rust
pub struct TriggerIntentId(Digest);
```

Content-derived semantic intent after normalization.

---

# 9. Why Three IDs

```text
definition
  = configured rule

occurrence
  = observed event/timer firing

intent
  = normalized semantic run request
```

---

# 10. Trigger Kind

```rust
pub enum TriggerKind {
    ScmPush,
    ChangeProposal,
    Tag,
    Manual,
    Api,
    Schedule,
    Delayed,
    System,
    ProviderEvent,
    Custom(TriggerKindId),
}
```

---

# 11. Trigger Definition

```rust
pub struct TriggerDefinition {
    pub id: TriggerDefinitionId,
    pub kind: TriggerKind,
    pub scope: TriggerScope,
    pub pipeline: PipelineSelector,
    pub filters: TriggerFilterSet,
    pub concurrency: Option<TriggerConcurrencyPolicy>,
}
```

---

# 12. Trigger Scope

```text
project
repository
branch/ref pattern
change proposal
environment
```

---

# 13. SCM Push Trigger

Typical flow:

```text
provider webhook
  ↓
verify/dedup
  ↓
normalize push event
  ↓
resolve exact VCS revision
  ↓
materialize SourceSnapshotId
  ↓
evaluate trigger filters
  ↓
create TriggerIntent
  ↓
plan/run
```

---

# 14. Mutable Ref

Branch/ref name is context only.

---

# 15. Exact Source

Protected run binds exact:

```text
RevisionId
SourceSnapshotId
```

---

# 16. Push Filter

Examples:

```text
branch
tag
path
author/source trust
repository
```

---

# 17. Path Filters

Use canonical source diff where applicable.

---

# 18. Path Filter Safety

Do not make path filter stronger than known diff/ownership semantics.

---

# 19. Unknown Diff

Conservative match if correctness-sensitive.

---

# 20. Change Proposal Trigger

Events:

```text
opened
updated
reopened
ready-for-review
label/state change
```

---

# 21. Exact Proposal Revision

Run binds exact `ProposalRevisionId`.

---

# 22. Target Movement

If target branch changes, integration candidate/check may need re-plan.

---

# 23. Tag Trigger

Resolve tag to exact revision before run.

---

# 24. Annotated/Lightweight Tag

Provider/VCS adapter normalizes.

---

# 25. Deleted Tag

Does not invalidate historical run.

---

# 26. Manual Trigger

```rust
pub struct ManualDispatchRequest {
    pub project: ProjectId,
    pub pipeline: PipelineSelector,
    pub source: ManualSourceSelector,
    pub inputs: ManualInputSet,
    pub actor: PrincipalId,
}
```

---

# 27. Manual Source Selector

May initially be:

```text
branch
tag
revision
current source snapshot
```

---

# 28. Resolution

Mutable selector resolves before run creation.

---

# 29. Manual Inputs

Typed according to pipeline/template input schema.

---

# 30. No Arbitrary Env Injection

Critical.

---

# 31. Secret Inputs

SecretRef only.

---

# 32. API Dispatch

Same semantic model as manual dispatch.

---

# 33. API Auth

Normal authn/authz.

---

# 34. Idempotency-Key

Required/recommended for retryable dispatch.

---

# 35. Same Idempotency Key + Different Payload

Conflict.

---

# 36. Scheduled Trigger

```rust
pub struct ScheduleDefinition {
    pub id: TriggerDefinitionId,
    pub schedule: ScheduleSpec,
    pub timezone: TimeZoneId,
    pub missed_run_policy: MissedRunPolicy,
}
```

---

# 37. ScheduleSpec

```rust
pub enum ScheduleSpec {
    Cron(CronExpression),
    Interval(IntervalSchedule),
    OneShot(Timestamp),
    Calendar(CalendarSchedule),
}
```

---

# 38. Cron

Must use explicit timezone.

---

# 39. Default Timezone

Project/system default only if explicitly configured.

---

# 40. No Silent UTC Assumption in User-Facing Schedule

Critical.

---

# 41. DST

Schedules in civil time can encounter:

```text
non-existent local times
duplicated local times
```

---

# 42. DST Policy

```rust
pub enum DstResolutionPolicy {
    SkipNonexistent,
    ShiftForward,
    RunOnceOnDuplicate,
    RunBothOnDuplicate,
}
```

---

# 43. Must Be Explicit

Do not guess silently.

---

# 44. Interval Schedule

Based on duration/anchor.

---

# 45. Calendar Schedule

Examples:

```text
first business day
specific weekdays
monthly date
```

if implemented.

---

# 46. Business Calendars

Not baseline unless explicitly configured.

---

# 47. Scheduled Source

Schedule needs source-selection rule.

---

# 48. Source Rule

Examples:

```text
default branch head at fire time
exact pinned revision
release channel head
```

---

# 49. Default Branch Head

Resolved exact when occurrence fires.

---

# 50. Historical Repeatability

Occurrence stores resolved exact source.

---

# 51. Delayed Trigger

One-shot future execution.

---

# 52. Durable Timer

Stored in metadata DB.

---

# 53. In-Memory Timer

Optimization only.

---

# 54. Timer Identity

```rust
pub struct TriggerTimerId(Ulid);
```

---

# 55. Timer State

```rust
pub enum TriggerTimerState {
    Scheduled,
    Due,
    Claimed,
    Fired,
    Cancelled,
    Expired,
}
```

---

# 56. HA Timer Claim

DB atomic claim/lease.

---

# 57. No Raft Requirement

Ordinary schedule correctness can use Postgres + idempotency.

---

# 58. At-Least-Once Timer Firing

Expected.

---

# 59. Semantic Dedup

Occurrence schedule slot included in intent key.

---

# 60. ScheduledOccurrenceKey

Example:

```text
TriggerDefinitionId
+
scheduled civil/UTC occurrence
```

---

# 61. Duplicate Timer Worker

Same intent deduped.

---

# 62. Missed Run Policy

```rust
pub enum MissedRunPolicy {
    Skip,
    RunLatest,
    RunAll { max_backfill: u32 },
    ManualReview,
}
```

---

# 63. Example

Forgeyard down for 8 hours.

Nightly hourly schedule:

```text
Skip
RunLatest
RunAll capped
```

---

# 64. Never Infinite Backfill

Critical.

---

# 65. Backfill Bound

Count/time horizon.

---

# 66. Missed Occurrence

Persist enough to inspect.

---

# 67. Schedule Reconciliation

On startup:

```text
find due timers
compute missed occurrences
apply policy
create intents idempotently
```

---

# 68. Manual Backfill

```text
forgeyard trigger backfill
```

permission-gated.

---

# 69. Backfill Uses Exact Occurrence Times

---

# 70. Trigger Filter

```rust
pub enum TriggerFilter {
    Ref(RefPattern),
    Path(PathPattern),
    EventAction(EventActionFilter),
    SourceTrust(SourceTrustClass),
    Label(LabelFilter),
    Custom(TriggerFilterId),
}
```

---

# 71. Filter Semantics

All deterministic from normalized event/context.

---

# 72. Provider-Specific Filter

Normalized at adapter edge.

---

# 73. No Provider SDK Types in Core Trigger

Critical.

---

# 74. Debounce

Useful for bursty pushes.

---

# 75. DebouncePolicy

```rust
pub struct DebouncePolicy {
    pub window: Duration,
    pub mode: DebounceMode,
}
```

---

# 76. Debounce Mode

```rust
pub enum DebounceMode {
    Latest,
    First,
    Aggregate,
}
```

---

# 77. Latest

Within window, run newest semantic source.

---

# 78. First

Ignore later matching events during window.

---

# 79. Aggregate

Create one intent carrying a bounded event set.

---

# 80. Default for CI Push

Often Latest.

---

# 81. Debounce Is Optimization

Must not discard protected semantic events if policy requires every occurrence.

---

# 82. Coalescing

Multiple events that resolve to same exact source/pipeline/input can share one semantic run.

---

# 83. Coalescing Key

```rust
pub struct TriggerCoalesceKey(Digest);
```

---

# 84. Inputs

```text
project
pipeline plan selector
exact source
manual/scheduled inputs
trigger semantics
```

---

# 85. Different Security Context

Do not coalesce blindly.

---

# 86. Actor-Sensitive Manual Run

Normally separate.

---

# 87. Concurrency Group

```rust
pub struct TriggerConcurrencyGroupId(Digest);
```

---

# 88. Use Cases

```text
one deployment per environment
one branch CI run at a time
one nightly maintenance workflow
```

---

# 89. Concurrency Policy

```rust
pub enum TriggerConcurrencyMode {
    AllowParallel,
    Queue,
    CancelPrevious,
    SupersedePrevious,
}
```

---

# 90. CancelPrevious

Requests cancellation of previous Run.

---

# 91. SupersedePrevious

Marks older run obsolete and may request cancellation.

---

# 92. Do Not Mutate Old Run Into New Run

Critical.

---

# 93. Superseded Run

Historical identity preserved.

---

# 94. Supersession Reason

Stores newer TriggerIntentId/RunId.

---

# 95. Safe Cancellation

Uses normal Run cancellation semantics.

---

# 96. Running Release/Deploy

May not be cancellable merely because source changed.

---

# 97. Policy

Can forbid supersession for protected workflows.

---

# 98. Example

PR validation:

```text
SupersedePrevious
```

Stable release:

```text
Queue / explicit control
```

---

# 99. Concurrency Key

Can derive from:

```text
project
branch
environment
pipeline
custom typed key
```

---

# 100. No Arbitrary User String Without Scope

Prevent collisions/leakage.

---

# 101. Trigger Intent

```rust
pub struct TriggerIntent {
    pub id: TriggerIntentId,
    pub definition: TriggerDefinitionId,
    pub occurrence: TriggerOccurrenceId,
    pub project: ProjectId,
    pub pipeline: PipelineSelector,
    pub source: ResolvedTriggerSource,
    pub inputs: TriggerInputSet,
    pub actor: TriggerActor,
}
```

---

# 102. Trigger Actor

```rust
pub enum TriggerActor {
    Human(PrincipalId),
    ScmProvider(ScmInstallationId),
    Schedule(SystemActorId),
    Service(ServiceAccountId),
    System(SystemActorId),
}
```

---

# 103. Provider Trigger Does Not Pretend to Be Human

Critical.

---

# 104. Manual Actor

Human/service principal.

---

# 105. Schedule Actor

System schedule identity.

---

# 106. Trigger Authorization

Two questions:

```text
may actor create/modify trigger definition?
may this occurrence initiate this pipeline?
```

---

# 107. Schedule Definition Creation

Admin/project permission.

---

# 108. Schedule Firing

Runs under configured system/workload identity, not creator's indefinitely cached session.

---

# 109. Scheduled Principal

```rust
pub struct ScheduledExecutionPrincipal {
    pub service_principal: PrincipalId,
    pub scope: ResourceScope,
}
```

---

# 110. Do Not Reuse Human Session Token

Critical.

---

# 111. Manual High-Risk Pipeline

May require step-up.

---

# 112. Protected Environment Manual Trigger

Policy.

---

# 113. Trigger Policy Inputs

```text
actor
source trust
event kind
pipeline
environment
time/window
```

---

# 114. Source Trust

Fork event may restrict secrets/signing.

---

# 115. Trigger Cannot Grant Secrets

Critical.

---

# 116. Secret Eligibility

Decided later by policy/workload identity.

---

# 117. Trigger State

```rust
pub enum TriggerOccurrenceState {
    Received,
    Verified,
    Deduplicated,
    FilteredOut,
    ResolvingSource,
    Ready,
    Dispatched,
    Rejected,
    Failed,
}
```

---

# 118. FilteredOut

Normal outcome, not error.

---

# 119. Source Resolution Failure

Retry/reconcile if transient.

---

# 120. Deleted/Unknown Ref

Terminal or ignored according to event semantics.

---

# 121. Trigger Dedup

External delivery key where available.

---

# 122. Provider Delivery ID

Part 21.

---

# 123. Semantic Dedup

Needed beyond provider delivery ID.

---

# 124. TriggerDedupKey

```rust
pub struct TriggerDedupKey(Digest);
```

---

# 125. Examples

Push:

```text
provider/repository/event/revision/action
```

Schedule:

```text
definition/occurrence timestamp
```

Manual:

```text
Idempotency-Key
```

---

# 126. Dedup Store

Metadata DB.

---

# 127. TTL

Keep long enough for expected replay horizon.

---

# 128. Durable Intent

Persist before Run creation.

---

# 129. Correct Flow

```text
normalize
  ↓
persist TriggerIntent
  ↓
plan/authorize
  ↓
idempotently create Run
```

---

# 130. Crash After Intent Before Run

Reconciler creates Run.

---

# 131. Crash After Run Before Ack

Same intent finds existing Run.

---

# 132. IntentToRun Mapping

Unique constraint.

---

# 133. No Exactly-Once Claim

At-least-once + idempotency.

---

# 134. Trigger-to-Run Binding

```rust
pub struct TriggerRunBinding {
    pub intent: TriggerIntentId,
    pub run: RunId,
}
```

---

# 135. One Intent

Normally one semantic Run.

---

# 136. Retry of Failed Run

New Run or JobAttempt depending user action/policy, not reuse same semantic state blindly.

---

# 137. Replay Trigger

Need explicit semantics.

---

# 138. Replay Mode

```rust
pub enum TriggerReplayMode {
    ReevaluateOnly,
    CreateNewRun,
}
```

---

# 139. ReevaluateOnly

Re-run filter/diagnostics without side effects.

---

# 140. CreateNewRun

Explicit permission and new intent/replay lineage.

---

# 141. Never Blindly Replay Historic Webhooks Into Live Side Effects

Existing Part 10 principle.

---

# 142. Replay Record

Stores original occurrence.

---

# 143. Schedule Pause

```rust
pub enum TriggerDefinitionState {
    Active,
    Paused,
    Disabled,
    Archived,
}
```

---

# 144. Pause

No new firing.

---

# 145. Existing queued/running runs

Unaffected unless separately cancelled.

---

# 146. Disable

Stronger admin state.

---

# 147. Archived

Historical/read-only.

---

# 148. Trigger Versioning

Changing trigger config creates new definition revision or immutable config version.

---

# 149. TriggerDefinitionVersion

```rust
pub struct TriggerDefinitionVersion(u64);
```

---

# 150. Occurrence Binds Version

Historical reproducibility.

---

# 151. Schedule Edit

Does not retroactively change past occurrences.

---

# 152. Timezone Edit

New definition version.

---

# 153. Trigger Config Source

May be repository pipeline config or admin metadata depending type.

---

# 154. Repository Trigger

Untrusted project config cannot create privileged schedule identity.

---

# 155. System-Managed Trigger

Admin plane.

---

# 156. Trigger Ownership

```text
project
organization
system
```

---

# 157. SCM Webhook Registration

Provider adapter may create/update webhook.

---

# 158. Desired/Observed Registration

Reconciled.

---

# 159. Webhook Registration Drift

Detected.

---

# 160. Provider Webhook Missing

Recreate if authorized.

---

# 161. Duplicate Webhook

Reconcile.

---

# 162. Provider Rate Limits

Part 21.

---

# 163. Trigger Health

Measures:

```text
webhook delivery lag
schedule lag
source resolution failures
dedup backlog
dispatch backlog
```

---

# 164. Trigger Lag

```rust
pub struct TriggerLag(Duration);
```

---

# 165. Schedule Lag

Actual dispatch time - scheduled occurrence.

---

# 166. SLO

Can define.

---

# 167. Trigger Priority

Trigger ingestion should be lightweight.

---

# 168. Heavy Work

Async after persistence.

---

# 169. Backpressure

Bound queue.

---

# 170. Storm

Thousands of push events.

---

# 171. Storm Controls

```text
dedup
debounce
coalesce
rate limit
bounded queue
```

---

# 172. Do Not Drop Protected Events Silently

Critical.

---

# 173. Overflow State

Explicit degraded health.

---

# 174. Manual Dispatch Rate Limit

Per principal/project.

---

# 175. Scheduled Burst

Spread/jitter optional for non-exact schedules.

---

# 176. Exact Schedule

Do not add hidden jitter.

---

# 177. Jitter

Only explicit setting.

---

# 178. Staggering

Useful across many projects.

---

# 179. Scheduled Concurrency

If previous occurrence still running:

```text
AllowParallel
Skip
Queue
CancelPrevious
```

---

# 180. ScheduleOverlapPolicy

```rust
pub enum ScheduleOverlapPolicy {
    Allow,
    SkipNew,
    Queue,
    CancelPrevious,
}
```

---

# 181. SkipNew

Record skipped occurrence.

---

# 182. No Silent Missing History

Critical.

---

# 183. Input Schema

Pipeline/template declares dispatch inputs.

---

# 184. Trigger Input Validation

Before intent ready.

---

# 185. Default Inputs

Canonicalized.

---

# 186. SecretRef Validation

Reference only.

---

# 187. Input Digest

```rust
pub struct TriggerInputDigest(Digest);
```

---

# 188. Run Plan Identity

Includes normalized inputs as appropriate.

---

# 189. Environment Selector

Manual/scheduled trigger may select environment, but policy validates.

---

# 190. No Free-Form Production Environment Name Bypass

Critical.

---

# 191. Deployment Trigger

Could be triggered by release event.

---

# 192. System Trigger

Examples:

```text
ReleasePublished
ArtifactPromoted
PolicyChanged
DependencySecurityUpdate
```

---

# 193. Internal Domain Event Trigger

Use carefully.

---

# 194. Event Fast Path + Reconciliation

Same architecture.

---

# 195. Avoid Trigger Loops

A run emits event that retriggers itself indefinitely.

---

# 196. Trigger Causation Chain

Store correlation/causation IDs.

---

# 197. Max Trigger Chain Depth

Bound.

---

# 198. Cycle Detection

For configured internal trigger graph where possible.

---

# 199. Trigger Loop Protection

```text
causation chain
depth bound
same-intent dedup
```

---

# 200. Trigger Causation

```rust
pub struct TriggerCausation {
    pub correlation: CorrelationId,
    pub cause_event: Option<EventId>,
    pub parent_intent: Option<TriggerIntentId>,
}
```

---

# 201. Release-on-Build Loops

Prevent via explicit event type/phase.

---

# 202. Environment Promotion Trigger

May create deployment request, still normal policy.

---

# 203. Calendar/Blackout Windows

Policy may deny trigger execution during freeze.

---

# 204. Trigger Fires During Freeze

Intent can be:

```text
Rejected
Queued
AwaitingPolicy
```

depending policy.

---

# 205. AwaitingPolicy

If durable deferred approval/workflow supported.

---

# 206. Baseline

Prefer explicit Rejected/Queued.

---

# 207. Maintenance Window

Can be part of schedule/policy.

---

# 208. UI

Pages:

```text
Triggers
Schedules
Recent Trigger Activity
Concurrency Groups
Missed Runs
Webhook Health
```

---

# 209. Trigger Detail

Shows:

```text
kind
filters
source selector
last occurrence
last run
dedup/debounce
state
```

---

# 210. Schedule UI

Shows timezone and next occurrences.

---

# 211. Preview Next Occurrences

Important for DST.

---

# 212. `forgeyard trigger schedule preview`

Shows next N UTC + local times.

---

# 213. Manual Run UI

Typed form from pipeline input schema.

---

# 214. High-Risk Inputs

Reauth/confirmation.

---

# 215. Recent Activity

Shows:

```text
received
filtered
deduplicated
dispatched
failed
```

---

# 216. Explain

Why a webhook did/did not start run.

---

# 217. `forgeyard trigger explain <occurrence>`

High-value operator tool.

---

# 218. CLI

```text
forgeyard trigger list
forgeyard trigger show
forgeyard trigger enable
forgeyard trigger pause
forgeyard trigger dispatch
forgeyard trigger backfill
forgeyard trigger explain
forgeyard schedule preview
```

---

# 219. API

Potential:

```text
GET  /v1/triggers
POST /v1/triggers
PATCH /v1/triggers/{id}
POST /v1/triggers/{id}/dispatch
POST /v1/triggers/{id}/pause
POST /v1/triggers/{id}/backfill
GET  /v1/trigger-occurrences
```

---

# 220. Permissions

```text
trigger.read
trigger.manage
trigger.dispatch
trigger.backfill
trigger.schedule.manage
trigger.system.manage
```

---

# 221. Manual Dispatch

Requires pipeline/run permission too.

---

# 222. Trigger Manage != Pipeline Execute

Both checks where appropriate.

---

# 223. Audit

Audit:

```text
trigger create/update/delete
schedule pause/resume
manual protected dispatch
backfill
system trigger changes
```

---

# 224. Routine SCM occurrences

Domain/event records, not privileged audit each time unless security-relevant.

---

# 225. Notification

Examples:

```text
schedule repeatedly failing
trigger backlog
webhook disconnected
missed runs require review
```

---

# 226. Search

Part 31 indexes trigger metadata/activity where useful.

---

# 227. Analytics

Examples:

```text
trigger-to-run latency
dedup rate
debounce rate
missed schedule count
manual dispatch volume
```

---

# 228. No User Productivity Surveillance

Aggregate operational use.

---

# 229. Observability Metrics

```text
trigger_occurrences_total
trigger_deduplicated_total
trigger_filtered_total
trigger_dispatch_total
trigger_dispatch_failures_total
trigger_schedule_lag_seconds
trigger_missed_occurrences_total
trigger_backlog
```

---

# 230. Labels

Low-cardinality:

```text
kind
result
provider_kind
```

---

# 231. No branch/project IDs in metrics.

---

# 232. Tracing

```text
trigger.ingest
trigger.verify
trigger.dedup
trigger.resolve_source
trigger.filter
trigger.dispatch
trigger.schedule
trigger.reconcile
```

---

# 233. Health

Checks:

```text
timer scanner
dispatch backlog
SCM webhook registration
source resolver
dedup store
```

---

# 234. Doctor

```text
forgeyard trigger doctor
```

---

# 235. Doctor Checks

```text
invalid cron/timezone
no eligible source
webhook registration drift
stuck timers
backlog
permission/policy mismatch
```

---

# 236. Standalone Mode

Schedules stored in Stoolap/local metadata.

---

# 237. Local Timer Service

Durable local timer scanning.

---

# 238. Distributed Mode

Postgres authoritative.

---

# 239. HA

Multiple trigger workers can claim due items safely.

---

# 240. No Single In-Memory Scheduler Authority

Critical.

---

# 241. Leader

Not required for ordinary timers if DB claim semantics correct.

---

# 242. Optional Coordinator

Can reduce duplicate work, but correctness remains idempotency.

---

# 243. Clock

Server time authority.

---

# 244. Clock Skew

Use DB/server trusted time for due calculations in distributed mode.

---

# 245. Client Clock

Never authority for scheduled run.

---

# 246. Timezone Database

Versioned system dependency.

---

# 247. TZ Database Changes

Future occurrence interpretation may change.

---

# 248. Schedule Metadata

Record timezone + schedule definition; occurrence records exact UTC instant.

---

# 249. Historical Occurrence

Never recomputed from newer timezone rules.

---

# 250. Schedule Edit Race

Occurrence claims exact definition version.

---

# 251. Trigger Disable Race

If already claimed, policy/state check before dispatch.

---

# 252. Trigger Delete

Prefer archive/deactivate with history.

---

# 253. Event Retention

Occurrence history configurable.

---

# 254. Trigger Intent

Longer retention for audit/debug if tied to runs.

---

# 255. DR

Trigger definitions/schedules/timers backed up.

---

# 256. On Restore

Recalculate due/missed occurrences from authoritative definitions + last-fired state.

---

# 257. Do Not Replay External Webhooks Blindly

Critical.

---

# 258. Scheduled Triggers

Can safely compute missed windows.

---

# 259. Manual/API

No automatic replay unless durable intent already existed.

---

# 260. SCM

Reconcile provider state/current refs rather than blindly replay historical effects.

---

# 261. Trigger Migration

Schema versioned.

---

# 262. Schedule Semantics Version

```rust
pub struct ScheduleSemanticsVersion(u16);
```

---

# 263. Why

Cron/parser/DST behavior must not silently change.

---

# 264. Upgrade Compatibility

Old definitions migrate explicitly.

---

# 265. Testkit

```text
forgeyard-trigger-testkit/src/
├── lib.rs
├── scm.rs
├── manual.rs
├── schedule.rs
├── dedup.rs
├── debounce.rs
├── concurrency.rs
├── timer.rs
└── assertions.rs
```

---

# 266. Unit Tests

Trigger intent identity.

---

# 267. Duplicate Webhook Test

One semantic run.

---

# 268. Provider Redelivery Test

Dedup.

---

# 269. Push Source Resolution Test

Exact revision/snapshot.

---

# 270. Deleted Ref Test

No stale branch-head guessing.

---

# 271. Manual Idempotency Test

Same key/same payload returns existing semantic dispatch.

---

# 272. Manual Conflict Test

Same key/different payload conflict.

---

# 273. Typed Input Test

Invalid input rejected.

---

# 274. Secret Input Test

Plaintext secret disallowed.

---

# 275. Cron Timezone Test

Expected occurrence.

---

# 276. DST Gap Test

Explicit policy.

---

# 277. DST Duplicate Test

Explicit once/both behavior.

---

# 278. Restart Missed Schedule Test

MissedRunPolicy applied.

---

# 279. Backfill Bound Test

Never infinite.

---

# 280. Timer Duplicate Claim Test

Intent dedup.

---

# 281. Debounce Latest Test

Newest revision selected.

---

# 282. Concurrency Supersede Test

Old Run remains historical.

---

# 283. Protected Workflow Test

Supersession forbidden by policy.

---

# 284. Trigger Loop Test

Causation depth stops recursion.

---

# 285. Fork Trust Test

Trigger does not unlock secrets/signing.

---

# 286. Schedule Identity Test

Uses service/system principal, not creator session.

---

# 287. Trigger Disable Race Test

State rechecked.

---

# 288. HA Test

Two workers do not create duplicate semantic runs.

---

# 289. DB Restart Test

Pending intent reconciles to Run.

---

# 290. Timezone DB Upgrade Test

Historical occurrence remains exact.

---

# 291. DR Test

Schedules restore and missed policy applies.

---

# 292. Fuzzing

Fuzz:

```text
cron parser
filter parser
provider event normalizer
manual input decoder
```

---

# 293. Property Tests

Same semantic trigger input -> same TriggerIntentId.

---

# 294. Storm Load Test

Large webhook burst with bounded memory/queue.

---

# 295. Schedule Scale Test

Millions of scheduled definitions if needed.

---

# 296. Implementation Phase 1 — Trigger Model/Manual Dispatch

Core intent/run binding.

---

# 297. Phase 2 — SCM Push/Proposal Triggers

Provider integration.

---

# 298. Phase 3 — Durable Schedules

Cron/one-shot/timezone.

---

# 299. Phase 4 — Dedup/Reconciliation

Crash-safe.

---

# 300. Phase 5 — Debounce/Coalescing

Efficiency.

---

# 301. Phase 6 — Concurrency/Supersession

PR workflow quality.

---

# 302. Phase 7 — Missed Runs/Backfill

Operational resilience.

---

# 303. Phase 8 — System/Event Triggers

Release/deploy automation.

---

# 304. Phase 9 — UI/CLI/Explain

Operability.

---

# 305. Phase 10 — HA/Scale

Large installations.

---

# 306. Phase 11 — Security/Loop Hardening

Defense.

---

# 307. Phase 12 — Fuzz/DR/Compatibility

Production readiness.

---

# 308. Acceptance Tests

1. Triggers create durable intent, not arbitrary JobAttempts.
2. All triggered execution flows through normal Pipeline IR and Run state machine.
3. Mutable SCM refs resolve to exact revision/source snapshot before protected execution.
4. Proposal triggers bind exact ProposalRevisionId.
5. Duplicate webhook deliveries do not create duplicate semantic runs.
6. Manual/API dispatch supports idempotency.
7. Same idempotency key with different payload conflicts.
8. Manual inputs are typed.
9. Secret values cannot be injected as plaintext trigger inputs.
10. Schedule definitions use explicit timezone semantics.
11. DST gaps/duplicates follow explicit configured behavior.
12. Scheduled occurrences persist exact UTC instants.
13. In-memory timers are optimization only.
14. Distributed timer firing is at-least-once and idempotent.
15. Service restart applies explicit missed-run policy.
16. Backfill is bounded.
17. Debounce/coalescing never bypasses policy-required semantic occurrences.
18. Supersession preserves historical Run identity.
19. Cancellation uses normal Run cancellation state machine.
20. Protected release/deploy workflows can forbid supersession.
21. Scheduled runs execute under service/system identity, not stale creator sessions.
22. Triggers never grant secret/signing privileges.
23. Fork/source trust restrictions still apply.
24. Trigger loops are bounded through causation/depth/dedup controls.
25. Provider-specific types remain outside core trigger model.
26. Trigger disable/pause state is checked before dispatch.
27. HA workers cannot create duplicate semantic runs.
28. Crash after intent/before Run reconciles correctly.
29. Crash after Run/before acknowledgement returns existing binding.
30. Trigger storm handling is bounded and observable.
31. Restore does not blindly replay old external webhooks.
32. Scheduled missed occurrences are recomputed safely after DR.
33. Standalone/distributed share trigger semantics.
34. Trigger explain shows why an occurrence did or did not run.
35. Forgeyard dogfoods SCM/manual/scheduled triggering on its own repository.

---

# 309. Production Readiness Gates

Do not call trigger architecture production-ready until:

```text
intent/run idempotency is proven
SCM source resolution is exact
manual dispatch idempotency works
cron/timezone/DST behavior is tested
durable timers survive restart
missed-run/backfill policy is bounded
debounce/supersession semantics are transparent
fork/source-trust restrictions pass
HA duplicate-worker tests pass
DR and storm tests pass
```

---

# 310. Architectural Invariants

1. trigger is initiation intent, not execution authority;
2. every run still flows through canonical planning/state machines;
3. mutable source refs resolve exactly before run;
4. trigger delivery is at-least-once;
5. semantic run creation is idempotent;
6. provider delivery ID alone is not enough for all dedup;
7. manual dispatch uses idempotency semantics;
8. scheduled timers are durable;
9. in-memory timers are optimization only;
10. timezone/DST behavior is explicit;
11. missed runs follow explicit bounded policy;
12. backfill is never infinite;
13. debounce/coalescing cannot weaken protected semantics;
14. supersession never rewrites historical runs;
15. cancellation uses existing Run semantics;
16. scheduled execution uses service/system identity;
17. triggers do not grant secrets/capabilities;
18. source trust still governs privileged access;
19. internal event triggers retain causation chains;
20. trigger loops are bounded;
21. disabled/paused state is authoritative;
22. schedule edits create new definition version;
23. historical occurrences bind exact definition/time/source;
24. HA correctness uses idempotency/reconciliation;
25. ordinary schedules do not require Raft;
26. trigger storm handling is bounded;
27. provider webhook registration is reconciled;
28. DR does not blindly replay external effects;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its own trigger subsystem.

---

# 311. Final Target Architecture

```text
                   Trigger Source
                         │
                         ▼
                    Ingest/Verify
                         │
                         ▼
                 Dedup / Filter
                         │
                         ▼
                  TriggerIntent
                         │
                         ▼
                   Source Resolve
                         │
                         ▼
                    Policy/Plan
                         │
                         ▼
                       Run
                         │
                         ▼
                  Normal Scheduler
```

Schedule path:

```text
ScheduleDefinition
        ↓
durable occurrence
        ↓
timer claim
        ↓
TriggerIntent
        ↓
exact source resolve
        ↓
Run
```

Crash safety:

```text
persist intent
  ↓
create Run idempotently
  ↓
bind intent → Run
  ↓
reconcile until complete
```

The key guarantee is:

> **Forgeyard can react to pushes, change proposals, manual requests, API calls, schedules, and internal events without turning trigger delivery into execution truth. Every occurrence becomes a durable, deduplicated, policy-checked intent that resolves immutable source identity and then enters the same canonical Pipeline IR and Run/Job machinery as every other execution.**

---

# 312. Extended Architecture Sequence

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
```
