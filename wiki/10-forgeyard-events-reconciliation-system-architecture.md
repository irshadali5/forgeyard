# 10 — Forgeyard Events, Event Delivery & Reconciliation System Architecture

**Document type:** Core Reliability & Coordination System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Durable domain events, transactional outbox, inbox/deduplication, event publication, subscriptions, replay/backfill, ordering, idempotent consumers, timers, reconciliation loops, missed-event repair, dead-letter handling, retention, HA-safe processing, and recovery semantics  
**Architecture style:** Persisted state is authority; events are durable facts/notifications; delivery is at-least-once; reconciliation guarantees eventual correctness  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on `02-forgeyard-storage-metadata.md`, `05-forgeyard-run-job-state-machine.md`, `06-forgeyard-scheduler-system-architecture.md`, `07-forgeyard-runner-agent-system-architecture.md`, and `09-forgeyard-transport-quic-internal-protocol.md`. It provides the reliability glue across all Forgeyard subsystems.

---

# 1. Purpose

Forgeyard is distributed and failure-prone by nature.

Events can be:

```text
delayed
duplicated
lost in transit
processed twice
processed after restart
observed out of order across streams
```

Processes can crash:

```text
after DB commit
before publish
after side effect
before ACK
```

Therefore Forgeyard must never depend on:

```text
"this event will be delivered exactly once"
```

The central rule is:

> **Persisted authoritative state is the source of truth. Events accelerate propagation; reconciliation repairs anything events miss.**

A second rule is:

> **Forgeyard uses at-least-once event delivery plus idempotent consumers, transactional outbox, persisted deadlines, and reconciliation.**

A third rule is:

> **Events represent facts that happened. Commands represent requested intent. Do not mix them.**

---

# 2. Architectural Position

```text
                 Domain Transaction
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       State         Event        Outbox
          │            │            │
          └────────────┼────────────┘
                       ▼
                     COMMIT
                       │
                       ▼
                 Event Publisher
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       Scheduler     UI/Event      Reconciler
       Consumer      Stream        Triggers
          │            │            │
          └────────────┼────────────┘
                       ▼
                 Idempotent Effects
```

Correctness fallback:

```text
Persisted State
      │
      ▼
Reconciler
      │
      ▼
repair missed/stuck state
```

---

# 3. Goals

The subsystem MUST:

1. define stable EventId;
2. define versioned event envelopes;
3. distinguish commands from events;
4. support transactional outbox;
5. support at-least-once publication;
6. support idempotent consumers;
7. support inbox/deduplication where needed;
8. support event replay;
9. support UI live-event streaming;
10. support event backfill;
11. support per-entity ordering where required;
12. avoid global ordering assumptions;
13. support durable timers;
14. support reconciliation loops;
15. recover from daemon crashes;
16. recover from missed event delivery;
17. support provider/webhook reconciliation;
18. support scheduler/run/job reconciliation;
19. support CAS/replication reconciliation;
20. support lease expiry reconciliation;
21. support queue/retry deadline reconciliation;
22. support dead-letter/error tracking;
23. support bounded event retention;
24. support event schema versioning;
25. support HA consumers;
26. expose lag/backlog metrics;
27. support test replay;
28. support audit integration;
29. avoid requiring Kafka/NATS for correctness;
30. allow external brokers later as adapters.

---

# 4. Non-Goals

The event system is not:

```text
primary metadata database
full event-sourced replacement for Forgeyard state
message broker product
workflow engine replacement
```

Forgeyard does not need to event-source every domain object.

---

# 5. Workspace Structure

```text
crates/events/
├── forgeyard-event/
├── forgeyard-event-model/
├── forgeyard-event-store/
├── forgeyard-event-outbox/
├── forgeyard-event-inbox/
├── forgeyard-event-publisher/
├── forgeyard-event-subscriber/
├── forgeyard-event-stream/
├── forgeyard-event-replay/
├── forgeyard-event-retention/
├── forgeyard-event-deadletter/
├── forgeyard-event-timer/
├── forgeyard-event-health/
├── forgeyard-event-metrics/
└── forgeyard-event-testkit/
```

Reconciliation:

```text
crates/reconciliation/
├── forgeyard-reconcile/
├── forgeyard-reconcile-model/
├── forgeyard-reconcile-run/
├── forgeyard-reconcile-job/
├── forgeyard-reconcile-lease/
├── forgeyard-reconcile-runner/
├── forgeyard-reconcile-scheduler/
├── forgeyard-reconcile-cas/
├── forgeyard-reconcile-artifact/
├── forgeyard-reconcile-change/
├── forgeyard-reconcile-provider/
├── forgeyard-reconcile-release/
├── forgeyard-reconcile-deployment/
├── forgeyard-reconcile-timer/
├── forgeyard-reconcile-health/
└── forgeyard-reconcile-testkit/
```

---

# 6. Event Identity

```rust
pub struct EventId(Ulid);
```

Globally unique.

---

# 7. Event Envelope

```rust
pub struct EventEnvelope<E> {
    pub schema: EventSchemaVersion,
    pub event_id: EventId,
    pub occurred_at: Timestamp,
    pub actor: ActorRef,
    pub correlation_id: Option<CorrelationId>,
    pub causation_id: Option<EventId>,
    pub event: E,
}
```

---

# 8. CausationId

Useful:

```text
JobSucceeded
  causes
DependentJobEligible
```

or:

```text
ProviderWebhookImported
  causes
ProposalRevisionCreated
```

---

# 9. CorrelationId vs CausationId

`CorrelationId`:

```text
same larger operation/trace
```

`CausationId`:

```text
which event directly caused this event
```

---

# 10. Event Schema Version

```rust
pub struct EventSchemaVersion(u16);
```

Independent from:

```text
wire protocol version
DB schema version
Pipeline IR version
```

---

# 11. Domain Event Rule

Events are past-tense facts.

Good:

```text
JobSucceeded
LeaseExpired
RunnerRegistered
ArtifactAvailable
```

Bad:

```text
RunJobNow
DeleteArtifact
```

Those are commands.

---

# 12. Command Rule

Commands express intent.

```rust
pub struct CancelRun { ... }
pub struct GrantLease { ... }
```

Services validate and may emit events.

---

# 13. Event Categories

```rust
pub enum EventCategory {
    Run,
    Job,
    Runner,
    Scheduler,
    Source,
    Vcs,
    Change,
    Artifact,
    Cas,
    Release,
    Deployment,
    Security,
    System,
}
```

Mostly useful for stream/filtering, not business logic.

---

# 14. Durable vs Ephemeral Events

Durable:

```text
state transitions
lease grant/loss
proposal revision
release promotion
security events
```

Ephemeral:

```text
heartbeat
live progress sample
temporary UI hint
```

Do not persist every high-frequency event.

---

# 15. Significant Event Principle

Persist only events that matter for:

```text
recovery
audit
history
downstream state
```

---

# 16. Transactional Outbox

Core pattern:

```text
BEGIN
  update authoritative state
  append domain event
  append outbox record
COMMIT
```

---

# 17. Why Outbox

Without outbox:

```text
DB commit succeeds
publisher crashes
event lost forever
```

With outbox:

publisher can retry.

---

# 18. Outbox Record

```rust
pub struct OutboxRecord {
    pub event_id: EventId,
    pub topic: EventTopic,
    pub payload_ref: EventPayloadRef,
    pub created_at: Timestamp,
    pub attempts: u32,
    pub next_attempt_at: Timestamp,
}
```

---

# 19. Payload Storage

Small event:

```text
inline binary
```

Large evidence:

```text
CAS ref
```

Do not store huge payloads in outbox.

---

# 20. Outbox State

```rust
pub enum OutboxState {
    Pending,
    Publishing,
    Delivered,
    Failed,
}
```

---

# 21. Publishing Claim

Multiple publisher workers can claim batches safely.

---

# 22. Claim Lease

Use transactional claim/lease.

---

# 23. Delivery Semantics

```text
at least once
```

A Delivered mark can fail after actual send, so consumer may see duplicate.

---

# 24. Consumer Rule

Every durable consumer must be idempotent.

---

# 25. Inbox

For consumers where duplicate side effects matter:

```rust
pub struct InboxRecord {
    pub consumer: ConsumerId,
    pub event: EventId,
    pub processed_at: Timestamp,
}
```

---

# 26. Inbox Transaction

```text
BEGIN
  check/insert inbox
  apply effect
COMMIT
```

---

# 27. When Inbox Is Needed

Use for:

```text
DB side effects
external side effects coordination
critical provider sync
```

Not necessarily for simple recomputation.

---

# 28. Consumer Idempotency Without Inbox

Some effects naturally idempotent:

```text
recompute run aggregate
refresh cache
```

---

# 29. Event Topics

Examples:

```text
run.state
job.state
runner.state
artifact.state
change.state
release.state
security.audit
```

---

# 30. Topic Is Routing Metadata

Do not infer domain truth solely from topic string.

Typed payload remains authority.

---

# 31. Event Store

Forgeyard may keep durable event history table.

This is not full event sourcing.

---

# 32. Event Store Record

```rust
pub struct StoredEvent {
    pub envelope: EventEnvelopeRef,
    pub entity: Option<EntityRef>,
    pub sequence: Option<EntityEventSeq>,
}
```

---

# 33. Per-Entity Sequence

For entities needing ordered history:

```rust
pub struct EntityEventSeq(u64);
```

Examples:

```text
Run
Job
ChangeProposal
```

---

# 34. No Global Ordering

Do not assume:

```text
Event A before Event B globally
```

unless same ordered entity/stream.

---

# 35. Cross-Entity Ordering

Use:

```text
causation
transaction
state reads
```

not timestamps.

---

# 36. Timestamp Ordering

Diagnostic only.

Clock skew exists.

---

# 37. Replay

```text
forgeyard-event-replay
```

Allows:

```text
UI backfill
consumer rebuild
derived projection rebuild
debug/testing
```

---

# 38. Replay Safety

Consumer must know whether replay should:

```text
rebuild projection
```

or:

```text
avoid external side effects
```

---

# 39. Replay Mode

```rust
pub enum ReplayMode {
    Projection,
    Simulation,
    LiveRedelivery,
}
```

---

# 40. External Side Effects

Never blindly replay:

```text
send email
deploy production
sign artifact
```

without idempotency/protection.

---

# 41. Event Stream API

UI/event consumers:

```text
subscribe from cursor
```

---

# 42. Event Cursor

```rust
pub struct EventCursor(BoundedString);
```

Opaque.

---

# 43. Backfill

Client reconnect:

```text
last cursor
  ↓
fetch missing events
  ↓
resume live stream
```

---

# 44. Public Live Events

Can use:

```text
WebSocket
SSE
```

through API layer.

---

# 45. Internal Events

Can remain in-process/store-backed initially.

No need for broker.

---

# 46. Broker Independence

Core event APIs should permit future adapter:

```text
NATS
Kafka
Redis Streams
```

without making them required.

---

# 47. Initial Recommendation

Use:

```text
Postgres/Stoolap outbox
+
in-process publisher/subscriber
+
WebSocket/SSE for UI
```

---

# 48. Why No Kafka Initially

Forgeyard can achieve correctness with:

```text
DB + outbox + reconciliation
```

and avoid operational complexity.

---

# 49. Event Publisher

```rust
#[async_trait]
pub trait EventPublisher {
    async fn publish_batch(
        &self,
        events: Vec<PublishableEvent>,
    ) -> Result<PublishBatchResult, EventPublishError>;
}
```

---

# 50. Subscriber

```rust
#[async_trait]
pub trait EventConsumer<E> {
    async fn handle(
        &self,
        event: EventEnvelope<E>,
    ) -> Result<(), EventConsumerError>;
}
```

---

# 51. Consumer Identity

```rust
pub struct ConsumerId(BoundedString);
```

Stable for inbox/dedup.

---

# 52. Consumer Version

Useful if semantics change.

```rust
pub struct ConsumerVersion(u16);
```

---

# 53. Event Delivery Failure

Retry with backoff.

---

# 54. Backoff

```text
exponential
bounded
jitter
```

---

# 55. Poison Event

If deterministic consumer failure persists:

```text
dead-letter
```

after configured attempts.

---

# 56. Dead Letter

```rust
pub struct DeadLetterRecord {
    pub event: EventId,
    pub consumer: ConsumerId,
    pub error_code: ErrorCode,
    pub attempts: u32,
    pub last_error_at: Timestamp,
}
```

---

# 57. Dead Letter Does Not Delete Event

Original event remains.

---

# 58. Dead Letter Operator Workflow

```text
inspect
fix consumer/data
replay
mark resolved
```

---

# 59. Dead Letter UI

Admin-only.

---

# 60. Reconciliation

Reconciliation inspects current persisted state and restores invariants.

---

# 61. Reconciler Contract

```rust
#[async_trait]
pub trait Reconciler {
    async fn reconcile(
        &self,
        scope: ReconcileScope,
    ) -> Result<ReconcileReport, ReconcileError>;
}
```

---

# 62. Reconcile Result

```rust
pub struct ReconcileReport {
    pub checked: u64,
    pub repaired: u64,
    pub deferred: u64,
    pub errors: Vec<ReconcileIssue>,
}
```

---

# 63. Reconcile Principle

Reconciliation should be:

```text
idempotent
bounded
observable
resumable
```

---

# 64. Run Reconciler

Checks:

```text
all jobs terminal but run active
run succeeded incorrectly
run cancellation not propagated
run timeout passed
```

---

# 65. Job Reconciler

Checks:

```text
dependencies satisfied but Pending
RetryWaiting expired
terminal attempt but job active
Succeeded without result refs
```

---

# 66. Lease Reconciler

Checks:

```text
expired lease
lease without active attempt
attempt without current lease
```

---

# 67. Runner Reconciler

Checks:

```text
runner heartbeat stale
session mismatch
resource reservations vs active attempts
```

---

# 68. Scheduler Reconciler

Checks:

```text
reservation leak
draining runner got new lease
eligible job stuck
```

---

# 69. CAS Reconciler

Checks:

```text
metadata says Available but object missing
replica count below policy
corrupt replica
```

---

# 70. Artifact Reconciler

Checks:

```text
Pending upload expired
Available artifact missing object
retention mismatch
```

---

# 71. Change Reconciler

Checks:

```text
check run completed but aggregate stale
approval invalidation not applied
candidate target drift
queue entry stale
```

---

# 72. Provider Reconciler

Checks external SCM/provider divergence:

```text
missing status
missed webhook
proposal state drift
```

---

# 73. Release Reconciler

Checks:

```text
signed object exists
durability satisfied
promotion side effect state
```

---

# 74. Deployment Reconciler

Checks:

```text
desired deployment
actual provider state
health
rollback requirements
```

---

# 75. Timer Reconciler

Checks persisted deadlines:

```text
lease expiry
retry_not_before
run timeout
queue timeout
approval expiry
```

---

# 76. Durable Timers

Persist:

```text
deadline timestamp
timer kind
entity
```

Do not rely solely on Tokio sleeps.

---

# 77. Timer Record

```rust
pub struct DurableTimer {
    pub id: TimerId,
    pub kind: TimerKind,
    pub entity: EntityRef,
    pub due_at: Timestamp,
    pub state: TimerState,
}
```

---

# 78. Timer States

```rust
pub enum TimerState {
    Pending,
    Claimed,
    Fired,
    Cancelled,
}
```

---

# 79. Timer Worker

Claims due timers transactionally.

---

# 80. Timer Event

Firing produces command/event into domain service.

---

# 81. Duplicate Timer Fire

Must be safe.

---

# 82. In-Memory Timer Wheel

Optimization.

---

# 83. Startup Timer Recovery

Query due timers immediately.

---

# 84. Periodic Reconciliation

Even with event-driven handlers.

---

# 85. Frequency Classes

```rust
pub enum ReconcileCadence {
    Fast,
    Normal,
    Slow,
    OnDemand,
}
```

---

# 86. Fast Reconcile

Examples:

```text
lease expiry
retry timers
```

---

# 87. Normal

```text
run aggregates
runner reservations
```

---

# 88. Slow

```text
CAS integrity
provider drift
historical cleanup
```

---

# 89. Event-Triggered Reconciliation

An event can enqueue targeted reconcile.

---

# 90. Periodic Fallback

Ensures missed event doesn't stall.

---

# 91. Reconcile Work Queue

Can be persisted if high-value.

Initial:

```text
event-triggered in-memory + periodic DB scan
```

is acceptable where scan is bounded/indexed.

---

# 92. Reconcile Cursor

For large scans:

```rust
pub struct ReconcileCursor(...);
```

Persist progress.

---

# 93. Chunked Reconciliation

Process bounded batches.

---

# 94. No Giant Full Scan

At enterprise scale, indexes by:

```text
active state
due timestamp
degraded status
```

---

# 95. Reconcile Locking

Multiple workers may reconcile different entities.

Use:

```text
claim leases
optimistic version
```

---

# 96. Same Entity Race

Only one repair commit wins.

Others re-read/no-op.

---

# 97. HA

All event/outbox/reconcile workers are restartable.

No single in-memory authority.

---

# 98. Outbox HA

Multiple daemon replicas claim different pending records.

---

# 99. Publisher Crash

Claim expires/retries.

---

# 100. Consumer Crash

Inbox transaction prevents duplicate state effect.

---

# 101. Reconciler Crash

Cursor/claim resumes.

---

# 102. Event Ordering Per Run

Optional per-Run sequence.

---

# 103. UI Ordering

UI can use:

```text
run event sequence
```

for timeline.

---

# 104. Eventual UI

Current state query is authority if event stream has gap.

---

# 105. UI Reconnect

```text
GET current state
+
backfill events
+
resume live
```

---

# 106. Notification Integration

Notification subsystem consumes durable events.

---

# 107. Email Duplicate Defense

Notification consumer uses inbox/idempotency.

---

# 108. Webhook Outbound

Same.

---

# 109. Provider Status Update

Status publication is idempotent by:

```text
proposal/check identity + snapshot
```

---

# 110. External Side Effect Pattern

For external APIs:

```text
persist desired operation
  ↓
worker calls external API
  ↓
record outcome
  ↓
reconcile actual state
```

---

# 111. Do Not Couple DB Tx to External API

No long transaction around network call.

---

# 112. Ambiguous External Result

Example:

```text
HTTP timeout after submit
```

Reconciler queries external system before retry.

---

# 113. Integration Submit Example

Change Proposal integration:

```text
submit candidate
timeout
  ↓
query target revision
  ↓
materialize/compare
  ↓
decide success/retry
```

---

# 114. Event Payload Size

Bounded.

---

# 115. Event Payload Ref

```rust
pub enum EventPayloadRef {
    Inline(BoundedBytes),
    Cas(CasObjectRef),
}
```

---

# 116. Sensitive Events

Event payloads may be sensitive.

Avoid embedding secrets.

---

# 117. Secret Events

Events reference:

```text
SecretRef
```

not secret value.

---

# 118. Audit Events

Audit system may consume domain events but audit records have separate retention/security semantics.

---

# 119. Event vs Audit

Not every event is audit-worthy.

Not every audit record needs event broadcast.

---

# 120. Security Events

Examples:

```text
RunnerCredentialRevoked
SandboxViolationDetected
PolicyExceptionGranted
```

longer retention.

---

# 121. Event Retention

Different classes:

```text
operational
historical
security
audit
```

---

# 122. Operational Retention

Could be shorter.

---

# 123. Historical Run Events

Keep enough for run timeline/history policy.

---

# 124. Release/Change Events

Longer retention.

---

# 125. Compaction

Old event history may be compacted after:

```text
current state durable
audit retained
retention policy
```

---

# 126. Never Compact Needed Audit

Audit retention wins.

---

# 127. Snapshotting Projections

If a projection uses event replay, periodic snapshot can reduce replay cost.

Not needed for core entity state because current rows already exist.

---

# 128. Event Indexes

Index:

```text
event_id
entity
occurred_at
topic
sequence
```

based on queries.

---

# 129. Outbox Index

```text
state
next_attempt_at
```

---

# 130. Timer Index

```text
state
due_at
```

---

# 131. Dead Letter Index

```text
consumer
resolved
last_error_at
```

---

# 132. Event Store Backend

Uses same `ForgeyardStore` backend:

```text
Stoolap
Postgres
```

---

# 133. Cross-Mode Invariant

Same event/reconcile semantics in standalone and distributed mode.

---

# 134. Standalone Events

Publisher can dispatch in-process after outbox commit.

---

# 135. Distributed Events

Multiple daemon replicas can consume.

---

# 136. No Broker Requirement

Important for Mode 1 simplicity.

---

# 137. Optional Broker Adapter

Later:

```text
forgeyard-event-nats
forgeyard-event-kafka
```

for scale/integration.

---

# 138. Broker as Transport

Authoritative event record/outbox semantics remain defined by Forgeyard.

---

# 139. Duplicate Broker Delivery

Still expected.

---

# 140. Consumer Registry

Application bootstrap registers consumers.

No global mutable registry.

---

# 141. Consumer Dependency Direction

Consumer belongs near owning subsystem.

Example:

```text
run subsystem consumes JobTerminal
scheduler consumes JobEligible
notification consumes RunCompleted
```

---

# 142. Avoid Event Spaghetti

Do not replace direct service call with event just to decouple everything.

---

# 143. When to Use Direct Call

Use direct call when:

```text
same transaction/operation
strong immediate result needed
```

---

# 144. When to Use Event

Use event when:

```text
fact already committed
multiple downstream consumers
eventual propagation acceptable
```

---

# 145. Example Direct + Event

Job completion service:

```text
validate + commit Job success
  ↓
emit JobSucceeded
```

Dependency resolver consumes event asynchronously.

---

# 146. Fast Path

Service may update direct dependents synchronously if cheap.

Event/reconcile remains backup.

---

# 147. Do Not Double-Apply

Fast path and event consumer must both be idempotent.

---

# 148. Event Version Upgrade

Consumer supports old retained event versions or migration adapter.

---

# 149. Event Upcaster

```rust
pub trait EventUpcaster {
    fn upcast(
        &self,
        old: StoredEvent,
    ) -> Result<CurrentEvent, EventUpgradeError>;
}
```

---

# 150. Avoid Rewriting Historical Events

Prefer decode/upcast.

---

# 151. Breaking Event Schema

Create new version.

---

# 152. Event Schema Tests

Golden vectors for durable released event formats.

---

# 153. Event Replay Tests

Old fixture events must still decode/upcast.

---

# 154. Outbox Payload Schema

References event schema explicitly.

---

# 155. Publisher Transport

Initial internal publisher can call consumer bus directly.

---

# 156. UI Stream Publisher

Reads durable event stream and pushes through WS/SSE.

---

# 157. Backpressure

Consumers have bounded queues.

---

# 158. Slow Consumer

Options:

```text
fall behind cursor
replay from store
```

Do not block core state commit.

---

# 159. UI Slow Client

Disconnect/reconnect with cursor.

---

# 160. Notification Slow Consumer

Outbox/inbox backlog grows; alert.

---

# 161. Consumer Lag

Metric:

```text
latest_event_time - last_processed_time
```

or sequence lag.

---

# 162. Event Publisher Metrics

```text
outbox_pending
outbox_oldest_age
events_published
event_publish_failures
event_delivery_retries
```

---

# 163. Consumer Metrics

```text
consumer_lag
consumer_errors
consumer_retries
consumer_deadletters
```

---

# 164. Reconcile Metrics

```text
reconcile_checked
reconcile_repaired
reconcile_errors
reconcile_duration
reconcile_backlog
```

---

# 165. Timer Metrics

```text
timers_due
timer_fire_delay
timer_errors
```

---

# 166. Alerting

Alert on:

```text
outbox backlog growing
oldest event too old
dead letter count
reconcile backlog
timer lag
consumer stuck
```

---

# 167. Tracing

Spans:

```text
event.persist
outbox.claim
event.publish
consumer.handle
event.replay
reconcile.run
timer.fire
```

---

# 168. Trace Context

Propagate CorrelationId/trace context into event envelope where useful.

---

# 169. Event Logging

Log event type/ID.

Do not dump sensitive payload.

---

# 170. Reconcile Logging

Log:

```text
entity
issue
repair action
```

bounded.

---

# 171. Health

Event subsystem health:

```text
outbox writable
publisher running
backlog age
deadletter status
```

---

# 172. Reconciliation Health

```text
last successful cycle
oldest unreconciled entity
error rate
```

---

# 173. Doctor

```text
forgeyard events doctor
forgeyard reconcile doctor
```

---

# 174. Event CLI

```text
forgeyard events list
forgeyard events show
forgeyard events replay
forgeyard events deadletters
forgeyard events retry
```

---

# 175. Reconcile CLI

```text
forgeyard reconcile status
forgeyard reconcile run
forgeyard reconcile entity
forgeyard reconcile dry-run
```

---

# 176. Dry Run

Shows proposed repairs without commit.

---

# 177. Admin Safety

Manual replay/reconcile can be dangerous.

Require permissions.

---

# 178. Event Replay Authorization

Security/audit events restricted.

---

# 179. Dead Letter Retry

After operator fixes cause:

```text
retry same EventId
```

---

# 180. Duplicate Effect Protection

Inbox keeps safe.

---

# 181. Reconcile Issue Model

```rust
pub struct ReconcileIssue {
    pub code: ReconcileIssueCode,
    pub entity: EntityRef,
    pub severity: ReconcileSeverity,
    pub repair: Option<RepairAction>,
}
```

---

# 182. Reconcile Severity

```rust
pub enum ReconcileSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
```

---

# 183. Automatic Repair

Safe cases:

```text
expired lease
retry timer elapsed
stale aggregate
orphan reservation
```

---

# 184. Manual Repair

Unsafe/ambiguous cases:

```text
missing release artifact with no replica
conflicting external provider state
corrupt provenance
```

---

# 185. Repair Action

```rust
pub enum RepairAction {
    TransitionEntity,
    ReleaseReservation,
    Requeue,
    MarkDegraded,
    RequestReplication,
    RequireOperator,
}
```

---

# 186. Reconciliation Must Not Fabricate Data

If CAS object lost:

```text
mark degraded
```

not invent bytes.

---

# 187. Reconcile and Audit

Automatic repairs affecting security/business state can produce audit event.

---

# 188. Reconcile Idempotency

Running same reconciler repeatedly converges.

---

# 189. Fixed Point

Ideal:

```text
reconcile(state) -> no changes
```

after invariants restored.

---

# 190. Event-Driven Dependency Propagation

`JobSucceeded` triggers dependent evaluation.

---

# 191. Missed Event

Periodic job reconciler still finds Pending dependency now satisfied.

---

# 192. Event-Driven Scheduler Wake

`JobEligible` wakes scheduler.

---

# 193. Missed Wake

Scheduler periodic eligible scan finds job.

---

# 194. Event-Driven Runner Cleanup

`LeaseExpired` can notify agent if connected.

---

# 195. Missed Notify

Runner lease deadline stops work locally.

---

# 196. Event-Driven CAS Replication

`ArtifactAvailable`/object durable requirement triggers replication.

---

# 197. Missed Replication Event

CAS reconciler scans durability gaps.

---

# 198. Event-Driven Provider Sync

Proposal/check changes publish status.

---

# 199. Missed Provider Update

Provider reconciler compares desired/current state.

---

# 200. Core Reliability Pattern

Every important asynchronous workflow should have:

```text
event fast path
+
reconcile slow path
```

---

# 201. Side Effect State Machine

For external effect:

```rust
pub enum ExternalEffectState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Unknown,
}
```

---

# 202. Unknown

Used when network result ambiguous.

---

# 203. Unknown Reconciliation

Query external destination.

---

# 204. Example Deployment

```text
request sent
connection reset
state = Unknown
  ↓
query deployment provider
  ↓
Succeeded / retry
```

---

# 205. No Blind External Retry

Could duplicate destructive action.

---

# 206. Idempotency Key for External API

Use if provider supports.

---

# 207. Event Projection

UI/read model can maintain:

```text
run counters
project recent status
```

---

# 208. Projection Rebuild

Replay durable events or recompute from state.

---

# 209. Prefer State Query for Critical UI

Projection is convenience.

---

# 210. Event Privacy

Tenant events must be tenant-scoped.

---

# 211. Public Event Authorization

WS/SSE filters after authz.

---

# 212. No Cross-Tenant Cursor Leakage

Cursor must not enable access to unauthorized events.

---

# 213. Event Payload Redaction

Public stream DTO differs from internal event if needed.

---

# 214. Internal vs Public Event

Separate models.

---

# 215. Audit vs UI Event

Security details can remain internal/audit only.

---

# 216. Event Size Limits

Enforce before DB/write/publish.

---

# 217. Event Fan-Out

One event may have many consumers.

Do not synchronously block state transaction on all consumers.

---

# 218. Consumer Isolation

One failing consumer does not prevent others.

---

# 219. Consumer Parallelism

Per consumer configurable.

---

# 220. Per-Entity Serialization

For consumers requiring ordered same-entity handling:

```text
partition by entity ID
```

or transactional version check.

---

# 221. Global Parallelism

Different entities process concurrently.

---

# 222. Consumer Reordering

If event N+1 arrives before N:

consumer should use entity version/sequence.

---

# 223. Stale Event

If current state already beyond event:

```text
no-op/recompute
```

---

# 224. Projection Sequence Check

Store last entity sequence.

---

# 225. Event Gap

Projection can request replay/rebuild.

---

# 226. Event Store Compaction and Gaps

Cursor semantics need stable behavior.

---

# 227. Cursor Expiry

If requested cursor older than retention:

```text
client must full-refresh current state
```

---

# 228. UI Response

Return:

```text
CursorExpired
```

then reload.

---

# 229. Provider Webhooks

Inbound webhooks are external events but enter through:

```text
verify
dedupe
normalize
persist
```

---

# 230. Webhook DeliveryId

Stored for dedupe.

---

# 231. Webhook Outbox?

Provider inbound pipeline can produce Forgeyard domain events transactionally after normalization.

---

# 232. SCM Provider Missed Webhook

Periodic reconcile catches.

---

# 233. File/Source Events

VCS exact revision/snapshot identity in event.

Never use only branch name.

---

# 234. Change Events

Bind ProposalRevisionId/SourceSnapshotId.

---

# 235. Run Events

Bind RunId/PlanId where relevant.

---

# 236. Artifact Events

Bind ArtifactId/CasObjectRef.

---

# 237. Release Events

Bind immutable release artifact digest.

---

# 238. Security Event Immutability

Long-term retained, append-only.

---

# 239. Store Schema

Logical tables:

```text
events
outbox
inbox
dead_letters
timers
reconcile_progress
```

---

# 240. Outbox Table

Suggested columns:

```text
event_id
topic
payload
state
attempts
next_attempt_at
claimed_by
claim_expires_at
created_at
delivered_at
```

---

# 241. Inbox Table

```text
consumer_id
event_id
processed_at
result_digest optional
```

Unique:

```text
(consumer_id, event_id)
```

---

# 242. Timer Table

```text
timer_id
kind
entity_type
entity_id
due_at
state
version
```

---

# 243. Dead Letter Table

```text
consumer_id
event_id
attempts
last_error_code
last_error_at
resolved_at
```

---

# 244. Reconcile Progress

```text
reconciler_id
cursor
last_started_at
last_completed_at
last_error
```

---

# 245. Stoolap Implementation

Same semantic tables/traits.

---

# 246. Postgres Implementation

Supports:

```text
claim batches
SKIP LOCKED where useful
indexes
```

adapter-local.

---

# 247. Migration

Event schemas/database schema evolve independently.

---

# 248. Retention Cleanup

Delivered outbox entries can be deleted after safe window.

---

# 249. Inbox Cleanup

Keep at least as long as event redelivery risk/idempotency horizon.

---

# 250. Dead Letter Retention

Long enough for operations/debug.

---

# 251. Timer Cleanup

Remove old fired timers after history policy.

---

# 252. Event History Retention

Per event class.

---

# 253. Consumer Deployment Upgrade

New version starts supporting old event schemas before old consumer removed.

---

# 254. Rolling Upgrade

N/N-1 event compatibility where event retained across deployment.

---

# 255. Event Producer Upgrade

Use expand/compat pattern:

```text
new optional event variant
consumers ready
then producer emits
```

---

# 256. Event Contracts

Document in:

```text
protocols/internal/events.md
```

or `docs/architecture/events.md`.

---

# 257. Testkit

```text
forgeyard-event-testkit/src/
├── lib.rs
├── event.rs
├── outbox.rs
├── inbox.rs
├── consumer.rs
├── replay.rs
├── timer.rs
└── assertions.rs
```

---

# 258. Reconcile Testkit

```text
forgeyard-reconcile-testkit/src/
├── lib.rs
├── fixture.rs
├── fake_clock.rs
├── run.rs
├── job.rs
├── lease.rs
├── cas.rs
└── assertions.rs
```

---

# 259. Unit Tests

Test:

```text
event envelope
outbox retry
inbox dedup
timer due logic
reconcile decision
```

---

# 260. Concurrency Tests

Multiple publishers claim same outbox.

Only one active claim, duplicate delivery still safe.

---

# 261. Consumer Crash Test

Crash after effect before ACK/mark.

Retry does not duplicate effect.

---

# 262. Publisher Crash Test

Crash after publish before Delivered.

Duplicate publish safe.

---

# 263. Timer Crash Test

Crash after timer command before marking fired.

Duplicate fire safe.

---

# 264. Reconciler Crash Test

Resume cursor.

---

# 265. Missed Event Test

Disable consumer, mutate state, re-enable only reconciler.

State converges.

---

# 266. Dependency Event Loss Test

Drop `JobSucceeded`.

Dependent eventually Eligible through reconciler.

---

# 267. Scheduler Wake Loss Test

Drop `JobEligible`.

Periodic scheduler scan still places.

---

# 268. CAS Replication Event Loss Test

Drop replication trigger.

CAS reconciler repairs durability.

---

# 269. Provider Webhook Loss Test

Provider reconciler imports missed state.

---

# 270. Replay Test

Projection rebuilt from retained events.

---

# 271. Cursor Expiry Test

Client receives full-refresh requirement.

---

# 272. Dead Letter Test

Poison event reaches DLQ after bounded retries.

---

# 273. Fuzzing

Fuzz:

```text
event decoding
event upcasting
cursor parsing
deadletter payload metadata
timer models
```

---

# 274. Model-Based Reliability Tests

Random sequence:

```text
commit
publish/drop/duplicate
consumer crash
reconcile
```

assert final state equals invariant model.

---

# 275. Failure Injection

```text
DB unavailable
publisher unavailable
consumer panic
clock jump
process restart
```

---

# 276. Clock Jump

Deadlines use Timestamp + injected clock.

Monotonic in-process helpers must not replace durable due times.

---

# 277. Performance Tests

Measure:

```text
outbox publish throughput
consumer latency
reconcile batch throughput
event replay
timer claims
```

---

# 278. Large Event History

Test pagination/backfill.

---

# 279. Backlog Recovery

Simulate 1M pending events.

Publisher recovers without overwhelming DB/downstream.

---

# 280. Adaptive Batch

Publisher can adjust batch size.

---

# 281. Backpressure

If consumer downstream unavailable:

```text
retry/backoff
```

Do not block state mutation.

---

# 282. Notification Storm

Coalescing can happen in notification layer.

Core event history remains factual.

---

# 283. Reconciliation Storm

Jitter schedules across replicas.

---

# 284. Leader Requirement

Most reconcilers can be multi-worker/claim-based.

Some global operations may later use coordination/leader.

---

# 285. Raft Relationship

Raft is not event delivery mechanism.

---

# 286. Event Broker Relationship

Broker is optional transport adapter.

---

# 287. Reliability Invariants

1. state commit never depends on successful event consumer;
2. outbox stored in same transaction as state;
3. delivery at least once;
4. consumers idempotent;
5. reconciliation repairs missed propagation;
6. timers persisted;
7. no critical deadline depends only on memory;
8. external ambiguous effects reconciled before retry;
9. UI events are not authority;
10. no global ordering assumption.

---

# 288. Security Invariants

1. event tenant scope enforced;
2. secret values excluded;
3. public stream redacted;
4. replay requires authz;
5. security events retained appropriately;
6. deadletter payloads protected;
7. manual replay audited.

---

# 289. Implementation Phase 1 — Event Model

Implement:

```text
EventId
EventEnvelope
EventSchemaVersion
event store API
```

---

# 290. Phase 2 — Transactional Outbox

Integrate with Run/Job state transitions.

---

# 291. Phase 3 — Publisher / Consumer Bus

In-process/store-backed.

---

# 292. Phase 4 — Inbox / Dedup

Add for critical consumers.

---

# 293. Phase 5 — Run/Job Reconciliation

First essential reconcilers.

---

# 294. Phase 6 — Timers

Lease expiry, retry waiting, run timeout.

---

# 295. Phase 7 — Scheduler/Runner Reconcile

Reservations/session/liveness.

---

# 296. Phase 8 — CAS/Artifact Reconcile

Durability/object availability.

---

# 297. Phase 9 — UI Stream / Replay

WS/SSE bridge with cursor/backfill.

---

# 298. Phase 10 — Provider/Change/Release Reconcile

External drift and high-level workflows.

---

# 299. Phase 11 — Dead Letter / Operations

Operator workflows.

---

# 300. Phase 12 — Hardening

HA, massive backlog, fuzzing, compatibility.

---

# 301. Acceptance Tests

1. State + outbox event commit atomically.
2. Publisher crash after send can duplicate but not lose effect.
3. Consumer duplicate delivery is idempotent.
4. Inbox uniqueness prevents duplicate DB side effect.
5. JobSucceeded event can wake dependency resolver.
6. Dropped JobSucceeded event is repaired by reconciler.
7. Dropped JobEligible wake is repaired by scheduler scan.
8. Lease expiry timer survives daemon restart.
9. RetryWaiting deadline survives daemon restart.
10. Run timeout survives daemon restart.
11. Heartbeats are not persisted as full event history.
12. Event replay does not blindly repeat external side effects.
13. External ambiguous submit enters Unknown/reconcile flow.
14. UI can reconnect from cursor.
15. Cursor older than retention requires full refresh.
16. Dead-letter event can be inspected and retried.
17. Multiple publisher workers safely share outbox.
18. Multiple reconcilers safely share work.
19. Reconciler is idempotent.
20. Reconcile reaches fixed point after repair.
21. CAS missing replica repaired after lost event.
22. Artifact pending upload expires through timer/reconcile.
23. Provider missed webhook repaired by provider reconcile.
24. Change aggregate stale state repaired.
25. Release durability state repaired.
26. Event schemas remain version-readable.
27. N/N-1 deployment handles retained events.
28. Public event stream cannot cross tenant boundary.
29. Secret values never appear in events.
30. Same event/reconcile behavior works on Stoolap/Postgres.
31. Forgeyard restart can recover without in-memory event state.
32. No Kafka/NATS is required for Mode 1/2 correctness.
33. Optional broker duplicate delivery remains safe.
34. Event backlog metrics/alerts work.
35. Forgeyard self-hosting uses same events/reconciliation.

---

# 302. Production Readiness Gates

Do not call events/reconciliation production-ready until:

```text
transactional outbox works
idempotent consumers proven
Run/Job reconciler works
durable timers work
restart recovery tested
missed-event tests pass
dead-letter workflow exists
event schema compatibility tested
UI replay/backfill works
metrics/alerts exist
```

Provider/CAS/release reconcilers can mature incrementally but must exist before those respective subsystems are production-critical.

---

# 303. Architectural Invariants

1. persisted state is authority;
2. events represent facts;
3. commands represent intent;
4. state + outbox commit atomically where required;
5. event delivery is at least once;
6. no exactly-once transport claim;
7. consumers are idempotent;
8. inbox used where duplicate side effects matter;
9. no global ordering assumption;
10. entity sequence used where ordering matters;
11. timestamps are not ordering authority;
12. replay modes distinguish projections from live side effects;
13. durable timers survive restart;
14. in-memory timers are optimizations only;
15. every important async workflow has reconcile fallback;
16. reconciliation is idempotent;
17. reconciliation never fabricates missing data;
18. external ambiguous outcomes are queried before retry;
19. event consumers cannot block state commit;
20. slow UI clients use replay/cursor;
21. event payloads are bounded;
22. secret values do not enter events;
23. dead letters preserve original event;
24. manual replay is privileged/audited;
25. broker infrastructure is optional;
26. Stoolap/Postgres share semantics;
27. HA workers coordinate through claims/versioning;
28. old event versions remain readable/upcastable per retention;
29. event history can be compacted only under retention rules;
30. Forgeyard dogfoods its own event/reconcile system.

---

# 304. Final Target Architecture

```text
                     Domain Service
                          │
                          ▼
                    BEGIN TRANSACTION
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          State         Event        Outbox
             │            │            │
             └────────────┼────────────┘
                          ▼
                        COMMIT
                          │
                          ▼
                  Outbox Publisher
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
        Consumers       UI Stream    Triggers
             │            │            │
             └────────────┼────────────┘
                          ▼
                    Fast Propagation

                    Persisted State
                          │
                          ▼
                    Reconciler Loop
                          │
                          ▼
                  Missed/Drift Repair
```

---

# 305. Final Architectural Position

Fast path:

```text
state transition
  ↓
event/outbox
  ↓
consumer
  ↓
downstream update
```

Failure path:

```text
event dropped/delayed
consumer crashed
publisher restarted
  ↓
authoritative state remains correct
  ↓
reconciler scans
  ↓
missing downstream effect repaired
```

Timer path:

```text
persisted due_at
  ↓
timer worker/reconciler
  ↓
idempotent domain command
```

External side effect:

```text
desired state
  ↓
external call
  ↓
Succeeded / Failed / Unknown
  ↓
reconcile actual remote state
```

The key guarantee is:

> **Forgeyard does not try to make distributed messaging magically exactly-once. It makes every important state transition durable, every delivery retry-safe, and every asynchronous workflow recoverable by reconciliation until the system converges on the correct authoritative state.**

---

# 306. New-Repository Sequence

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
