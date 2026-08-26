# 02 — Forgeyard Storage & Metadata System Architecture

**Document type:** Core Infrastructure System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Metadata persistence, transactional state, database abstraction, standalone Stoolap storage, distributed PostgreSQL/Neon storage, schema evolution, consistency, backup/restore, tenancy, and persistence reliability  
**Architecture style:** Store-interface driven, metadata/CAS separation, standalone-first with distributed upgrade path  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on `01-forgeyard-core-domain-foundation.md`. It assumes the previously defined VCS-neutral source model, Change Proposal model, hermetic/reproducible build architecture, and workspace structure without redefining them.

---

# 1. Purpose

Forgeyard needs a persistence architecture that works in two very different environments:

```text
MODE 1
Standalone / offline / local developer machine

MODE 2+
Distributed / team / enterprise / HA
```

The storage layer must support both without forcing:

```text
Postgres everywhere
```

and without allowing:

```text
database-specific logic everywhere
```

The central rule is:

> **Forgeyard domain services depend on typed persistence capabilities, not on Stoolap, SQL, PostgreSQL, Neon, connection pools, or database-row structures.**

The second central rule is:

> **Metadata belongs in the metadata store. Large immutable bytes belong in CAS.**

---

# 2. Architectural Position

```text
                       Domain Services
                            │
                            ▼
                     Persistence Ports
                            │
                 ┌──────────┴──────────┐
                 ▼                     ▼
          Stoolap Adapter       PostgreSQL Adapter
                 │                     │
                 ▼                     ▼
          Local DB File         PostgreSQL / Neon
```

Bulk objects remain separate:

```text
metadata store
    │
    ├── artifact metadata
    ├── source metadata
    ├── job/run state
    ├── policy/audit metadata
    └── CAS object references
                 │
                 ▼
                CAS
```

---

# 3. Goals

The subsystem MUST:

1. provide one stable persistence abstraction;
2. support Stoolap for standalone mode;
3. support PostgreSQL/Neon for distributed mode;
4. keep database implementation details out of domain crates;
5. provide explicit transactions;
6. provide idempotent write patterns;
7. support optimistic concurrency;
8. support row/version conflict detection;
9. support append-only event/audit data;
10. support leases;
11. support reconciliation;
12. support schema migrations;
13. support expand-contract upgrades;
14. support backup/restore;
15. support point-in-time recovery in distributed deployments where backend supports it;
16. support tenancy boundaries;
17. support indexes for high-volume workflows;
18. separate metadata from CAS;
19. provide conformance tests;
20. support degraded/recovery behavior.

---

# 4. Non-Goals

The metadata store is not:

```text
artifact blob storage
source tarball storage
build-cache object storage
container registry
large log-object store
```

Those belong in CAS/object storage.

The metadata store may keep:

```text
small inline metadata
indexes
digests
references
state
audit records
```

---

# 5. Workspace Structure

```text
crates/store/
├── forgeyard-store/
├── forgeyard-store-model/
├── forgeyard-store-transaction/
├── forgeyard-store-stoolap/
├── forgeyard-store-postgres/
├── forgeyard-store-migration/
├── forgeyard-store-backup/
├── forgeyard-store-health/
└── forgeyard-store-testkit/
```

Optional later:

```text
├── forgeyard-store-readmodel/
├── forgeyard-store-archive/
└── forgeyard-store-metrics/
```

---

# 6. `forgeyard-store`

The primary capability crate.

It exposes:

```text
traits
commands
queries
transaction boundaries
typed persistence errors
```

It MUST NOT depend on:

```text
sqlx
tokio-postgres
Stoolap implementation internals
Neon SDK
```

---

# 7. `forgeyard-store-model`

Common persistence-neutral models.

Examples:

```rust
pub struct EntityVersion(u64);

pub struct PageRequest {
    pub limit: PageSize,
    pub cursor: Option<PageCursor>,
}

pub struct Page<T> {
    pub items: Vec<T>,
    pub next: Option<PageCursor>,
}
```

---

# 8. Store Trait Strategy

Do NOT create one enormous trait with hundreds of methods if it can be avoided.

Prefer subsystem-oriented capability traits:

```rust
pub trait ProjectStore { ... }
pub trait RunStore { ... }
pub trait JobStore { ... }
pub trait RunnerStore { ... }
pub trait SourceStore { ... }
pub trait VcsMetadataStore { ... }
pub trait ChangeStore { ... }
pub trait ArtifactMetadataStore { ... }
pub trait ReleaseStore { ... }
pub trait DeploymentStore { ... }
pub trait PolicyStore { ... }
pub trait AuditStore { ... }
```

Then aggregate:

```rust
pub trait ForgeyardStore:
    ProjectStore
    + RunStore
    + JobStore
    + RunnerStore
    + SourceStore
    + ChangeStore
    + ArtifactMetadataStore
    + ReleaseStore
    + DeploymentStore
    + PolicyStore
    + AuditStore
    + Send
    + Sync
{}
```

---

# 9. Why Capability Traits

Benefits:

```text
smaller interfaces
easier tests
clear ownership
less accidental coupling
incremental implementation
easier future extraction
```

---

# 10. Transaction Model

Transactions are explicit capabilities.

```rust
#[async_trait]
pub trait TransactionManager {
    type Tx: ForgeyardTransaction;

    async fn begin(&self) -> Result<Self::Tx, StoreError>;
}
```

---

# 11. Transaction Scope

Transactions should be used for:

```text
state transition + event append
lease acquire + job state update
proposal revision append + stale-evidence updates
release promotion state + audit record
```

Do not wrap network/CAS uploads inside long SQL transactions.

---

# 12. Transaction Boundary Rule

Correct:

```text
prepare immutable CAS object
    ↓
start transaction
    ↓
write metadata reference
    ↓
commit
```

Not:

```text
start transaction
    ↓
upload 5 GB artifact
    ↓
commit
```

---

# 13. Metadata / CAS Atomicity

Because SQL and CAS are separate systems, true cross-system ACID is not assumed.

Use:

```text
prepare
commit metadata
reconcile orphan
```

or:

```text
upload CAS
verify
write metadata reference transactionally
```

Then GC unreferenced CAS objects after grace period.

---

# 14. Entity Versioning

Use optimistic concurrency.

```rust
pub struct Versioned<T> {
    pub value: T,
    pub version: EntityVersion,
}
```

Updates require:

```text
expected_version
```

---

# 15. Compare-and-Set Update

Conceptually:

```text
UPDATE entity
SET ...
WHERE id = ?
AND version = expected
```

If no row changed:

```text
StoreConflict
```

---

# 16. Why Optimistic Concurrency

It protects against:

```text
lost updates
stale API writes
duplicate workers
concurrent reviews
queue races
provider sync races
```

---

# 17. Idempotency

Write operations that can be retried should accept:

```rust
pub struct IdempotencyKey(...);
```

Store mapping:

```text
scope + idempotency key -> result identity
```

---

# 18. Idempotent Command Example

```text
CreateRun(
  request_id = X
)
```

Retrying X returns same run rather than creating a duplicate.

---

# 19. Idempotency Scope

Include enough context:

```text
tenant
operation
resource
```

to prevent cross-operation collision.

---

# 20. Database Error Model

```rust
pub enum StoreError {
    NotFound,
    Conflict,
    ConstraintViolation,
    TransactionAborted,
    SerializationFailure,
    Unavailable,
    Timeout,
    Corruption,
    MigrationRequired,
    Unsupported,
    Internal,
}
```

Database-specific error codes are translated inside adapters.

---

# 21. Retry Mapping

Examples:

```text
serialization failure -> retry/backoff
deadlock -> retry/backoff
unique violation -> depends on operation
corruption -> never retry blindly
timeout -> retry/reconcile depending context
```

---

# 22. Standalone Mode

Standalone composition:

```text
forgeyard
  ↓
forgeyard-store-stoolap
  ↓
local DB file
```

No Postgres requirement.

---

# 23. Why Stoolap in Mode 1

Desired properties:

```text
embedded
zero external daemon
local filesystem
simple install
transaction support
Rust integration
low operational burden
```

---

# 24. Stoolap Adapter

```text
crates/store/forgeyard-store-stoolap/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── db.rs
    ├── schema.rs
    ├── transaction.rs
    ├── migration.rs
    ├── project.rs
    ├── run.rs
    ├── job.rs
    ├── runner.rs
    ├── source.rs
    ├── vcs.rs
    ├── change.rs
    ├── artifact.rs
    ├── release.rs
    ├── deployment.rs
    ├── policy.rs
    ├── audit.rs
    ├── health.rs
    └── error.rs
```

---

# 25. Stoolap Database Location

Default local path should be under Forgeyard data directory:

```text
~/.local/share/forgeyard/
```

or platform-equivalent application data directory.

Example:

```text
forgeyard.db
```

Do not place production state in current working directory by default.

---

# 26. Standalone Locking

Only one standalone daemon/process should own mutable local DB state unless Stoolap safely supports required multi-process access.

Use:

```text
process lock
or
DB-supported exclusive coordination
```

to avoid split-brain local writers.

---

# 27. Local Backup

Standalone backup command:

```text
forgeyard storage backup
```

should produce:

```text
metadata snapshot
CAS manifest
configuration manifest
version info
```

Optional:

```text
include CAS bytes
```

for full portable backup.

---

# 28. Distributed Mode

Distributed composition:

```text
forgeyard-daemon
   ↓
forgeyard-store-postgres
   ↓
PostgreSQL / Neon
```

---

# 29. PostgreSQL Authority

In distributed Forgeyard:

```text
Postgres/Neon
```

is authoritative for shared metadata.

Raft does not replace it.

---

# 30. PostgreSQL Adapter Tree

```text
crates/store/forgeyard-store-postgres/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── pool.rs
    ├── connection.rs
    ├── transaction.rs
    ├── statement.rs
    ├── retry.rs
    ├── row/
    │   ├── project.rs
    │   ├── run.rs
    │   ├── job.rs
    │   ├── runner.rs
    │   ├── source.rs
    │   ├── vcs.rs
    │   ├── change.rs
    │   ├── artifact.rs
    │   ├── release.rs
    │   ├── deployment.rs
    │   ├── policy.rs
    │   └── audit.rs
    ├── query/
    │   ├── project.rs
    │   ├── run.rs
    │   ├── job.rs
    │   ├── runner.rs
    │   ├── source.rs
    │   ├── vcs.rs
    │   ├── change.rs
    │   ├── artifact.rs
    │   ├── release.rs
    │   ├── deployment.rs
    │   ├── policy.rs
    │   └── audit.rs
    ├── health.rs
    └── error.rs
```

---

# 31. Row Types

DB row structs remain adapter-local.

Bad:

```rust
pub struct Run {
    pub created_at: sqlx::types::chrono::DateTime<Utc>,
}
```

inside domain.

Correct:

```text
PostgresRunRow
   ↓ TryFrom
Run
```

---

# 32. SQL Placement

All SQL stays inside:

```text
forgeyard-store-postgres
```

or migration files.

No SQL strings in:

```text
scheduler
change service
release service
VCS service
```

---

# 33. Query Ownership

Each domain store capability owns its persistence queries.

Example:

```text
RunStore
  ↔
query/run.rs
```

---

# 34. Connection Pool

Pool configuration:

```text
min connections
max connections
acquire timeout
idle timeout
max lifetime
```

Must be observable.

---

# 35. Pool Size

Do not set huge default pools.

Distributed capacity should consider:

```text
daemon replicas × pool max
```

against database connection limits.

---

# 36. Neon Considerations

Neon-compatible behavior should treat PostgreSQL protocol/semantics as source of truth.

Forgeyard should avoid depending on Neon-only semantics in the domain layer.

Neon integration can optimize:

```text
connection setup
branch/testing workflows
serverless pooling considerations
```

through deployment/configuration rather than core domain coupling.

---

# 37. Connection Resilience

Transient connection loss:

```text
retry with bounded backoff
```

but state-changing operations must remain idempotent or transactional.

---

# 38. Transaction Isolation

Default recommendations depend on operation.

General CRUD:

```text
READ COMMITTED
```

Critical compare/update workflows:

```text
explicit row version/CAS
```

Certain queue/lease operations may require:

```text
SELECT ... FOR UPDATE
```

or equivalent safe transactional primitives.

---

# 39. Serializable Transactions

Use only where necessary.

Do not make the entire system SERIALIZABLE by default.

For workflows requiring serializable behavior:

```text
retry serialization failures
```

explicitly.

---

# 40. Lease Storage

Lease record:

```rust
pub struct LeaseRecord {
    pub lease_id: LeaseId,
    pub owner: RunnerId,
    pub resource: LeaseResource,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
    pub epoch: LeaseEpoch,
}
```

---

# 41. Lease Acquisition

Transactionally:

```text
verify resource eligible
create lease
move resource state
append event
commit
```

---

# 42. Lease Expiry

DB expiry is not magic.

Use reconcilers:

```text
find expired leases
   ↓
verify current state
   ↓
mark lost/requeue
```

---

# 43. Job Persistence

Core job row metadata:

```text
job_id
run_id
state
attempt
priority
created_at
updated_at
version
```

Large logs are not stored inline.

---

# 44. Run Persistence

Run metadata:

```text
run_id
project_id
pipeline_id
source_snapshot
status
created_by
timestamps
version
```

---

# 45. Source Metadata Persistence

Store:

```text
repository ID
native revision ID
SourceSnapshotId
provenance ID
nested-source relationships
signature evidence references
```

Source bytes remain CAS.

---

# 46. Change Proposal Persistence

Store:

```text
proposal
proposal revisions
reviews
approvals
comments
checks
policy decisions
integration candidates
queue state
provider binding
```

Bulk diff/rendered artifacts may live in CAS.

---

# 47. Artifact Metadata Persistence

Store:

```text
artifact ID
digest
CAS object ID
type
size
producer
retention
created_at
```

Not artifact bytes.

---

# 48. Release Persistence

Store:

```text
release ID
candidate artifact digests
approval state
signing evidence
promotion state
timestamps
```

---

# 49. Deployment Persistence

Store:

```text
deployment ID
release/artifact
environment
state
health evidence
rollback pointer
```

---

# 50. Audit Persistence

Audit is append-only.

```text
audit_event_id
actor
action
resource
timestamp
payload digest/reference
```

Do not allow ordinary update/delete semantics.

---

# 51. Event Persistence

Durable domain events may use:

```text
event_log table
```

or subsystem-specific append tables.

Avoid building a giant generalized event-sourcing framework unless needed.

---

# 52. Event Outbox

For reliable external/internal publication:

```text
domain transaction
  ↓
state update
+ outbox row
  ↓
commit
  ↓
publisher sends event
  ↓
mark delivered
```

---

# 53. Why Outbox

Prevents:

```text
DB commit succeeded
but event publish failed
```

from losing important state notifications.

---

# 54. Outbox Semantics

At-least-once.

Consumers must deduplicate by:

```text
EventId
```

---

# 55. Inbox / Dedup

For inbound durable commands/events:

```text
processed_message
```

table can record:

```text
message ID
consumer
processed_at
result
```

---

# 56. Multi-Tenancy

Every shared metadata entity should be tenant-scoped where applicable.

Examples:

```text
tenant_id
organization_id
project_id
```

---

# 57. Tenant Boundary Rule

Queries must not rely solely on caller filtering.

Storage API should include tenant context.

Bad:

```rust
get_project(project_id)
```

Better:

```rust
get_project(tenant_id, project_id)
```

or a scoped store/session.

---

# 58. Scoped Store

Possible abstraction:

```rust
pub struct TenantStore<'a> {
    tenant: TenantId,
    inner: &'a dyn ForgeyardStore,
}
```

---

# 59. Row-Level Security

Postgres RLS may be considered later.

But domain correctness must not rely exclusively on RLS.

Use both:

```text
typed tenant scoping
+
DB constraints
```

---

# 60. Foreign Keys

Use foreign keys where they preserve real invariants.

Examples:

```text
job -> run
run -> project
proposal revision -> proposal
artifact -> project
```

---

# 61. Cascades

Avoid broad destructive `ON DELETE CASCADE` for audit/history-critical data.

Deletion should usually be controlled by service policy.

---

# 62. Soft Delete

Use only where domain semantics require retention.

Avoid automatic `deleted_at` on everything.

---

# 63. Immutable Records

Examples:

```text
audit events
proposal revisions
source provenance
release attestations
```

should be append-only/immutable.

---

# 64. Mutable Records

Examples:

```text
runner heartbeat
queue position
current job state
```

may update in place with versioning.

---

# 65. Append + Current Projection Pattern

For important state:

```text
append history/event
+
update current row
```

within same transaction.

---

# 66. Index Strategy

Indexes should support real query patterns.

Examples:

```text
jobs by state/priority
jobs by run
runs by project/created_at
runners by availability
proposals by repository/status
queue entries by target/state
audit by tenant/time
outbox by delivery state
```

---

# 67. Avoid Over-Indexing

Every index costs:

```text
write amplification
storage
vacuum/maintenance
```

Add based on measured query needs.

---

# 68. Pagination

Use cursor/keyset pagination for large tables.

Avoid deep:

```text
OFFSET 100000
```

queries.

---

# 69. Cursor Model

```rust
pub struct PageCursor(BoundedString);
```

Opaque to clients.

---

# 70. Ordering

Pagination order must be stable.

Example:

```text
created_at DESC, id DESC
```

---

# 71. Search

Do not turn primary metadata DB into universal full-text/search engine initially.

Start with indexed structured filters.

Add dedicated search capability only when needed.

---

# 72. Logs

High-volume logs should not live as giant SQL rows.

Store:

```text
log stream metadata
sequence ranges
CAS/object references
```

---

# 73. Log Metadata

Possible:

```text
job_id
stream_id
first_seq
last_seq
object_ref
created_at
```

---

# 74. Log Tail Index

Small recent/tail buffers may be held in memory or optimized store, but durable full logs remain separate.

---

# 75. Metrics

Metrics are observability data, not core transactional metadata.

Do not store time-series metrics in primary Forgeyard metadata DB by default.

---

# 76. Health

Storage health check:

```text
connection
read
write optional
migration status
latency
pool saturation
replica lag where relevant
```

---

# 77. Readiness

Daemon should not report ready if:

```text
required migrations pending
DB inaccessible
required schema incompatible
```

---

# 78. Schema Migration Architecture

Workspace:

```text
crates/store/forgeyard-store-migration/
migrations/
```

---

# 79. Migration Version

Every migration has stable ID:

```text
0001
0002
...
```

Never rewrite applied migrations.

---

# 80. Migration Metadata

Track:

```text
migration ID
checksum
applied_at
Forgeyard version
```

---

# 81. Migration Checksum

If an applied migration file changes unexpectedly:

```text
fail
```

Do not silently accept history rewrite.

---

# 82. Expand-Contract

Distributed rolling upgrade strategy:

```text
1. expand schema
2. deploy code using both/new fields
3. backfill
4. verify
5. switch reads
6. contract old schema later
```

---

# 83. Breaking Migration Rule

Never require all daemons/agents to stop simultaneously unless explicitly performing maintenance.

---

# 84. Migration Lock

Only one migration runner should apply schema changes.

Use:

```text
database advisory lock
or
coordination lock
```

---

# 85. Automatic Migration

Production default:

```text
safe expand-only migrations may auto-run
```

Potential destructive/expensive migrations:

```text
explicit operator command
```

---

# 86. Backfill

Large backfills should be:

```text
chunked
resumable
observable
idempotent
```

Not one huge transaction.

---

# 87. Backfill State

Store progress:

```text
migration ID
cursor
processed rows
errors
updated_at
```

---

# 88. Schema Ownership

Each subsystem owns its tables logically.

Example:

```text
change_*
vcs_*
run_*
artifact_*
```

Avoid one global miscellaneous table design.

---

# 89. Database Namespace

Optionally use Postgres schemas:

```text
forgeyard_core
forgeyard_change
forgeyard_vcs
```

but this can increase migration complexity.

Initial recommendation:

```text
single DB schema + disciplined prefixes
```

unless operational need justifies multiple schemas.

---

# 90. Naming Convention

Tables:

```text
snake_case plural
```

Columns:

```text
snake_case
```

Primary keys:

```text
<entity>_id
```

Version:

```text
entity_version
```

---

# 91. ID Storage

ULID/entity IDs can be stored as:

```text
UUID-compatible binary
byte representation
text
```

Choose one canonical DB representation.

Recommendation:

```text
native UUID where compatible with chosen ID semantics
or fixed binary
```

Avoid arbitrary variable strings if not needed.

---

# 92. Digest Storage

Store:

```text
algorithm
digest bytes
```

or canonical fixed binary columns.

Do not repeatedly store long textual prefixes when unnecessary.

---

# 93. Timestamps

Use database timestamp types carefully.

Domain converts to/from:

```text
forgeyard-time::Timestamp
```

---

# 94. Server-Side Now

Avoid relying on DB `NOW()` as sole domain time source when cross-system consistency matters.

Use explicit timestamps from service `Clock`, unless DB timestamp is intentionally authoritative for specific internal purpose.

---

# 95. Transaction Timestamp

A transaction can use one consistent service timestamp:

```text
occurred_at
```

for state + event.

---

# 96. Sequence Numbers

Use explicit sequence values for:

```text
logs
event streams
proposal revision number
job attempts
```

where ordering matters.

---

# 97. Job Attempts

Separate:

```text
job
job_attempt
```

A job may have multiple attempts.

---

# 98. Attempt Table

Store:

```text
attempt_id
job_id
attempt_number
runner_id
lease_id
state
start/end
failure classification
```

---

# 99. Runner Heartbeats

High-frequency writes can overload DB.

Architecture options:

```text
coalesced heartbeat writes
in-memory recent state + periodic durable update
separate lightweight heartbeat table
```

Do not write every tiny heartbeat update into a huge row with many indexed columns.

---

# 100. Runner Liveness

Separate:

```text
reported_at
lease heartbeat
health status
```

from immutable runner identity/capabilities.

---

# 101. Capability Storage

Runner capability blob should be:

```text
versioned
canonical
digest-addressed if large
```

with queryable important fields extracted/indexed.

---

# 102. Scheduler Query Path

Avoid scheduler scanning entire runner/job tables.

Maintain efficient queries/indexes for:

```text
eligible jobs
available runners
leases
resource availability
```

---

# 103. Queue Query Path

Integration queue tables need:

```text
target
state
priority
sequence
created_at
```

indexes.

---

# 104. Advisory Locks

Postgres advisory locks can be used for:

```text
migration lock
rare singleton operation
```

but do not make them the universal distributed lock architecture.

Forgeyard coordination layer handles broader leadership/exclusive semantics.

---

# 105. Postgres `SKIP LOCKED`

May be useful for certain work-claim patterns.

But scheduler semantics should remain explicit and tested.

Do not let a SQL queue become the scheduler architecture.

---

# 106. Database as Coordination

Use DB transaction semantics where natural.

Do not use DB as replacement for:

```text
agent protocol
runner leases
Raft leadership
CAS transfer
```

---

# 107. Neon Branching

Neon branching may be useful for:

```text
test environments
migration tests
ephemeral integration validation
```

But it remains an optional deployment optimization, not core requirement.

---

# 108. Read Replicas

Read replicas may serve:

```text
analytics/UI historical reads
audit browsing
```

Never serve consistency-sensitive write-after-read decisions without explicit consistency model.

---

# 109. Replica Lag

If replica used:

```text
lag must be observable
```

and critical paths must use primary/consistent reads.

---

# 110. Read Consistency Classes

Possible API:

```rust
pub enum ReadConsistency {
    Strong,
    Eventual,
}
```

Use sparingly.

Most domain stores can choose internally based on operation.

---

# 111. Strong Reads

Required for:

```text
lease validation
mergeability submit
approval current revision
release promotion
permission changes
```

---

# 112. Eventual Reads

May be acceptable for:

```text
dashboard counters
historical lists
analytics
```

---

# 113. Backup Architecture

Distributed backup includes:

```text
database backup/PITR
CAS backup/replication
configuration backup
secret-provider backup according to provider
```

---

# 114. Metadata Backup

Postgres:

```text
managed backup
WAL/PITR
logical dump for portability
```

depending deployment.

Neon can provide managed PostgreSQL recovery capabilities, but Forgeyard should still define its own restore/runbook expectations.

---

# 115. Stoolap Backup

Provide application-consistent backup operation.

Do not recommend copying live DB file unless backend guarantees safety.

---

# 116. CAS Backup

CAS backup is separate.

Metadata backup alone may leave artifact references without bytes.

---

# 117. Backup Manifest

Forgeyard backup manifest:

```rust
pub struct BackupManifest {
    pub forgeyard_version: String,
    pub schema_version: StoreSchemaVersion,
    pub created_at: Timestamp,
    pub metadata_backup: BackupObjectRef,
    pub cas_manifest: Option<CasBackupManifestRef>,
}
```

---

# 118. Restore

Restore stages:

```text
validate backup
restore metadata
validate schema
restore/attach CAS
run integrity verification
reconcile
resume service
```

---

# 119. Restore Into Newer Version

Preferred:

```text
restore into matching schema/version
then migrate forward
```

not arbitrary direct restore into incompatible latest schema.

---

# 120. Restore Verification

Verify:

```text
foreign keys
critical counts
CAS references
release artifacts
source snapshots
audit chain
```

---

# 121. Disaster Recovery

After outage:

```text
DB restored
  ↓
CAS checked
  ↓
leases expired/reconciled
  ↓
running jobs marked lost/retried
  ↓
provider state reconciled
  ↓
queues revalidated
```

---

# 122. Lease Recovery

Never assume a restored "Running" job is still running.

Reconcile via:

```text
agent heartbeat
lease expiry
attempt identity
```

---

# 123. Change Queue Recovery

Restored integration candidate must revalidate:

```text
target revision still expected
```

before submission.

---

# 124. Release Recovery

Release promotion state must verify:

```text
signed artifact exists
provenance exists
target registry/store state
```

before retry.

---

# 125. Data Integrity Checker

Tool/service:

```text
forgeyard storage verify
```

checks:

```text
dangling foreign keys
invalid states
CAS refs missing
duplicate immutable mappings
revision->snapshot mutation
release artifact mismatch
```

---

# 126. Database Constraints

Use DB constraints for invariants that are easy and stable:

```text
NOT NULL
UNIQUE
FOREIGN KEY
CHECK
```

Domain validates too.

Defense in depth.

---

# 127. Unique Constraints

Examples:

```text
repository + native revision -> snapshot mapping
proposal + revision_number
job + attempt_number
tenant + idempotency scope/key
```

---

# 128. Immutable Mapping Constraint

Critical:

```text
same VCS revision cannot map to different SourceSnapshotId
```

unless backend semantics explicitly say identity is mutable, which should be treated separately.

---

# 129. State Constraints

DB may use CHECK constraints for simple legal values if representation allows.

But transition legality belongs in domain.

---

# 130. Store Conformance Testkit

```text
crates/store/forgeyard-store-testkit/
```

Every backend runs same test suite.

---

# 131. Conformance Areas

```text
CRUD
transactions
optimistic concurrency
idempotency
pagination
tenant isolation
leases
append-only records
migration compatibility
error mapping
```

---

# 132. Example Conformance Test

```rust
async fn optimistic_update_rejects_stale_version<S: RunStore>(store: &S) {
    ...
}
```

Run against:

```text
Stoolap
Postgres
```

---

# 133. Ephemeral Postgres Tests

Integration tests may use:

```text
temporary local Postgres
containerized Postgres
test database
```

Forgeyard should not require Neon for correctness testing.

---

# 134. Neon Compatibility Tests

Separate optional test suite validates:

```text
connection behavior
pooling
migrations
long-running transactions assumptions
```

against Neon.

---

# 135. Stoolap Compatibility Tests

Run on every PR because standalone mode is first-class.

---

# 136. Migration Tests

For every migration:

```text
old schema fixture
  ↓
apply migration
  ↓
read/write using new code
```

---

# 137. N/N-1 Tests

Distributed rolling upgrade tests:

```text
N-1 daemon schema compatibility
N daemon
shared DB
```

where supported.

---

# 138. Performance Benchmarks

Measure:

```text
job enqueue
lease acquisition
heartbeat write
proposal read
queue read
audit append
pagination
```

---

# 139. Load Tests

Simulate:

```text
100k jobs
10k runners
large audit history
large proposal histories
many concurrent queue writes
```

Targets can evolve, but architecture must be tested under realistic scale.

---

# 140. Query Instrumentation

Every DB query should record:

```text
latency
operation name
rows
errors
timeout
```

Do not log SQL with secrets.

---

# 141. Slow Query Detection

Metrics:

```text
p50/p95/p99
```

for named operations.

---

# 142. Query Naming

Every query path gets stable logical name:

```text
run.get
job.claim
change.current
queue.next
audit.append
```

---

# 143. Connection Metrics

Expose:

```text
pool size
in-use
idle
waiters
acquire latency
errors
```

---

# 144. Storage Health Status

```rust
pub enum StorageHealth {
    Healthy,
    Degraded,
    Unavailable,
    MigrationRequired,
}
```

---

# 145. Degraded Mode

Some reads may continue if DB is degraded.

But do not allow unsafe state-changing operations without authoritative metadata storage.

---

# 146. Read-Only Emergency Mode

Optional operator mode:

```text
metadata reads
artifact downloads
audit browsing
```

while writes disabled.

---

# 147. Database Credentials

Use `SecretRef`.

Do not store:

```text
password in config file
```

---

# 148. TLS

Postgres production connections should support:

```text
TLS verification
custom CA
client certificate where applicable
```

---

# 149. Credential Rotation

Pool must reconnect safely after secret rotation.

Avoid requiring daemon restart where possible.

---

# 150. Database URL Logging

Always redact credentials.

---

# 151. SQL Injection

Use parameterized queries only.

Dynamic identifiers require strict whitelist/typed construction.

---

# 152. Migration Security

Migration files are trusted code.

Require:

```text
review
digest verification
release provenance
```

---

# 153. Tenant Query Safety

Architecture checker/testing should discourage adapter methods lacking tenant scoping for tenant-owned resources.

---

# 154. Audit for Admin Storage Operations

Record:

```text
migration
backup
restore
repair
manual data correction
```

---

# 155. Manual Data Repair

Provide controlled command:

```text
forgeyard storage repair
```

Never recommend direct ad-hoc SQL as normal operational workflow.

---

# 156. Repair Plan

```text
detect
generate plan
operator approve
apply transactionally
audit
verify
```

---

# 157. Data Export

Provide structured export for:

```text
project metadata
audit
release provenance
```

without exporting secret values.

---

# 158. Data Portability

Store abstraction should make future migration possible.

But do not target lowest-common-denominator DB semantics so aggressively that Postgres capabilities cannot be used inside adapter safely.

---

# 159. Backend-Specific Optimizations

Allowed inside adapter:

```text
Postgres indexes
JSONB for non-authoritative extension metadata
generated columns
partial indexes
advisory locks
```

if core semantics remain stable.

---

# 160. JSONB Rule

Use JSONB only for:

```text
provider payload fragments
extension metadata
rare unstructured evidence
```

Do not put core typed domain state into opaque JSON blobs.

---

# 161. Binary Metadata

Postcard blobs may be stored for:

```text
versioned opaque evidence
```

but critical searchable state should use structured columns.

---

# 162. RON in Database

Do not use RON as primary DB row format.

RON is for human configuration.

---

# 163. CAS Reference Type

```rust
pub struct CasObjectRef {
    pub digest: Digest,
    pub media_type: Option<MediaType>,
    pub size: ByteSize,
}
```

---

# 164. Artifact Record Example

```rust
pub struct ArtifactMetadata {
    pub id: ArtifactId,
    pub project: ProjectId,
    pub object: CasObjectRef,
    pub producer: JobId,
    pub created_at: Timestamp,
    pub retention: RetentionPolicy,
}
```

---

# 165. Source Snapshot Record Example

```rust
pub struct SourceSnapshotRecord {
    pub id: SourceSnapshotId,
    pub root_tree: TreeObjectId,
    pub provenance: SourceProvenanceId,
    pub created_at: Timestamp,
}
```

---

# 166. Change Proposal Record Example

```rust
pub struct ChangeProposalRecord {
    pub id: ChangeProposalId,
    pub repository: RepositoryId,
    pub lifecycle: ProposalLifecycle,
    pub current_revision: ProposalRevisionId,
    pub version: EntityVersion,
}
```

---

# 167. Store Command vs Row Model

Public store traits should accept domain-oriented command/record types.

Adapter-specific rows remain private.

---

# 168. Transactional State + Audit

Pattern:

```text
validate transition
  ↓
BEGIN
  ↓
update state with expected version
  ↓
append audit/event
  ↓
COMMIT
```

---

# 169. Transactional State + Outbox

Pattern:

```text
BEGIN
  ↓
state
audit
outbox
  ↓
COMMIT
```

Then async delivery.

---

# 170. Exactly-Once Warning

Do not claim exactly-once distributed effects.

Database can provide exactly-once row mutation in one transaction.

Cross-system delivery remains:

```text
at-least-once + idempotency + reconciliation
```

---

# 171. Store Service Boundary

A service should depend on:

```rust
Arc<dyn RunStore>
```

or aggregated application storage interface.

Not:

```rust
PgPool
```

---

# 172. Composition Wiring

Standalone:

```text
StoolapStore
  ↓
Arc<dyn ForgeyardStore>
```

Distributed:

```text
PostgresStore
  ↓
Arc<dyn ForgeyardStore>
```

Domain services unchanged.

---

# 173. Avoid Generic Repository Pattern Abuse

Do not create:

```rust
Repository<T>
```

for everything.

Persistence interfaces should express domain operations.

Example:

```rust
acquire_job_lease(...)
```

can be better than generic:

```rust
update(job)
```

because it encodes concurrency semantics.

---

# 174. Domain-Specific Store Operations

Good:

```text
append_proposal_revision
acquire_job_lease
record_job_attempt
mark_check_stale
claim_outbox_batch
```

These make atomicity explicit.

---

# 175. Batch Operations

Store APIs should support efficient batch writes for:

```text
events
artifact refs
source tree metadata
test results
```

where appropriate.

---

# 176. Large `IN` Queries

Use batch/temp-table strategies if large sets become common.

Do not generate thousands of scalar queries.

---

# 177. N+1 Query Prevention

Service tests/metrics should detect common N+1 patterns.

---

# 178. Prepared Statements

Use driver-supported prepared statement/cache behavior safely.

---

# 179. Statement Timeout

Critical DB calls should have bounded timeout.

Different operation classes may have different limits.

---

# 180. Long-Running Reports

Run historical/report queries separately from critical scheduler transaction paths.

---

# 181. Operational Separation

At scale consider:

```text
primary operational Postgres
read replica/analytics export
```

not immediately another database.

---

# 182. Audit Export

SIEM export should consume event/audit stream asynchronously.

Do not make audit transaction wait for SIEM network.

---

# 183. GC Metadata

CAS GC roots live in metadata:

```text
artifact refs
release refs
source refs
pinned refs
retention refs
```

---

# 184. GC Race Protection

GC must not delete object being attached concurrently.

Use:

```text
grace periods
mark epochs
metadata snapshot
```

---

# 185. Retention Policy

Metadata includes:

```text
retain until
pin
legal hold
release permanent
```

---

# 186. Legal Hold

Legal/compliance hold overrides normal deletion/GC.

---

# 187. Project Deletion

Deletion workflow:

```text
mark deletion requested
verify holds
disable new writes
delete/expire owned metadata
schedule CAS GC
audit
```

Avoid instant destructive cascade.

---

# 188. User Deletion

Identity/privacy-related deletion may require anonymization while preserving audit integrity.

Handled by identity/compliance layer with store support.

---

# 189. Store Schema Documentation

`docs/architecture/storage.md` should document logical tables/entities.

SQL migration files remain implementation truth for exact DB schema.

---

# 190. ER Diagram

Generate from schema/tooling where useful.

Do not manually maintain giant diagrams that drift.

---

# 191. Schema Generator

Optional:

```text
tools/forgeyard-schema-gen/
```

can produce:

```text
ER diagram
table docs
migration status
```

---

# 192. Storage CLI

```text
forgeyard storage status
forgeyard storage doctor
forgeyard storage migrate
forgeyard storage verify
forgeyard storage backup
forgeyard storage restore
forgeyard storage repair
forgeyard storage stats
```

---

# 193. `storage status`

Shows:

```text
backend
schema version
migration state
health
pool
database size
critical table counts
```

---

# 194. `storage doctor`

Checks:

```text
connectivity
permissions
schema
migration lock
latency
required extensions
timezone assumptions
CAS reference integrity sample
```

---

# 195. `storage migrate`

Supports:

```text
plan
apply
status
```

Dangerous operations require explicit operator confirmation in CLI.

---

# 196. `storage verify`

Modes:

```text
quick
full
```

Quick:

```text
schema
constraints
sample CAS refs
```

Full:

```text
all CAS refs
immutable mappings
audit integrity
```

---

# 197. `storage stats`

Displays:

```text
table sizes
row counts
growth
outbox backlog
old leases
orphan references
```

---

# 198. Admin API

Do not expose destructive storage admin operations to ordinary public REST API by default.

Prefer:

```text
local CLI
restricted admin API
```

---

# 199. Monitoring Alerts

Alert on:

```text
DB unavailable
pool exhaustion
migration mismatch
outbox backlog
replica lag
backup stale
PITR disabled where required
corruption signal
```

---

# 200. Backup SLO

Enterprise deployments should define:

```text
RPO
RTO
```

instead of saying merely "backups enabled."

---

# 201. Example RPO/RTO

Example target:

```text
RPO: 5 minutes
RTO: 30 minutes
```

Actual values are deployment policy, not hardcoded architecture.

---

# 202. Mode 1 Recovery

Standalone priorities:

```text
simple backup
portable restore
minimal operator knowledge
```

---

# 203. Mode 2 Recovery

Distributed priorities:

```text
PITR
tested restore
CAS consistency
reconciliation
```

---

# 204. Restore Drill

Production readiness requires periodic restore test.

Backup without tested restore is insufficient.

---

# 205. Migration Rollback

Schema rollback is not always safe.

Prefer forward-fix.

For app binary rollback:

```text
expand-contract schema
```

keeps N-1 compatibility.

---

# 206. Destructive Schema Changes

Only after:

```text
old code removed
backfill verified
retention window elapsed
```

---

# 207. Database Version Compatibility

Daemon startup checks:

```text
min supported schema
max supported schema
```

Refuse unsafe mismatch.

---

# 208. Agent Independence

Agents should not connect directly to Forgeyard metadata DB.

They communicate with daemon over protocol.

This preserves trust/control boundaries.

---

# 209. CLI DB Access

Normal CLI should use API.

Direct DB admin access only through explicit maintenance commands/tools.

---

# 210. UI DB Access

Never.

Dioxus UI only calls Forgeyard APIs.

---

# 211. Provider Adapter DB Access

Provider adapters use store/service interfaces.

No direct table access outside store adapter.

---

# 212. VCS Adapter DB Access

Same rule.

VCS adapters can emit metadata/provenance to VCS service, which uses store interfaces.

---

# 213. Ecosystem Adapter DB Access

None.

Ecosystem adapters produce plans/evidence.

Orchestrating services persist through store.

---

# 214. Scheduler DB Access

Scheduler may use domain-specific scheduler/store capability interfaces.

Do not hand it raw pool.

---

# 215. Scheduler Hot Path

Hot path should minimize round trips.

Example:

```text
get eligible batch
get runner candidates
transactionally lease
```

---

# 216. Queue Hot Path

Integration queue should support:

```text
claim next entries
lease queue operation
verify target state
```

with minimal DB contention.

---

# 217. Lock Ordering

Define consistent row/resource lock ordering for workflows touching multiple entities.

This reduces deadlocks.

---

# 218. Deadlock Handling

If DB deadlock occurs:

```text
translate
retry bounded
```

do not panic.

---

# 219. Transaction Retry Helper

Crate:

```text
forgeyard-store-transaction
```

can provide:

```rust
retry_transaction(policy, || async { ... })
```

only for operations proven idempotent at transaction boundary.

---

# 220. Retry Safety

Never auto-retry transaction if it contains irreversible external side effect.

---

# 221. Side-Effect Staging

Correct:

```text
DB reserves operation
commit
external effect
record outcome
reconcile
```

for:

```text
publish
sign
deploy
```

---

# 222. Saga-Like Workflows

Long workflows:

```text
release
deployment
integration
```

use persisted state machines + compensating/reconciliation actions, not giant DB transaction.

---

# 223. Storage Data Classes

Classify:

```text
transactional metadata
immutable provenance
audit
ephemeral coordination
high-volume stream metadata
```

This guides schema/index/retention.

---

# 224. Ephemeral Coordination Data

Examples:

```text
leases
heartbeats
temporary claims
```

may have aggressive cleanup.

---

# 225. Historical Metadata

Examples:

```text
releases
proposals
audit
source provenance
```

long retention.

---

# 226. Cleanup Jobs

Periodic:

```text
expired idempotency entries
old ephemeral leases
delivered outbox
temporary rows
```

with retention windows.

---

# 227. Outbox Retention

Delivered outbox may be compacted after audit-safe period.

Do not let table grow forever.

---

# 228. Vacuum / Maintenance

Postgres operational docs should monitor:

```text
autovacuum
bloat
long transactions
dead tuples
```

Forgeyard app should avoid creating pathologically long transactions.

---

# 229. Long Transaction Alert

Trace any transaction above threshold.

---

# 230. Isolation in Tests

Each integration test should use:

```text
fresh database/schema/transaction
```

to avoid cross-test state.

---

# 231. Deterministic Fixtures

Store testkit uses deterministic IDs/time.

---

# 232. Fault Injection

Test adapter behavior under:

```text
connection loss
timeout
serialization failure
deadlock
constraint violation
disk full simulation where possible
```

---

# 233. Corruption Simulation

For local DB, test:

```text
invalid file
partial backup
unsupported schema
```

and ensure errors are clear.

---

# 234. Store API Documentation

Every method documents:

```text
atomicity
consistency
idempotency
expected conflicts
retry behavior
```

---

# 235. Example Store API

```rust
#[async_trait]
pub trait JobStore {
    async fn insert_job(
        &self,
        job: NewJobRecord,
    ) -> Result<Versioned<JobRecord>, StoreError>;

    async fn acquire_lease(
        &self,
        request: AcquireJobLease,
    ) -> Result<JobLeaseResult, StoreError>;

    async fn complete_attempt(
        &self,
        request: CompleteJobAttempt,
    ) -> Result<JobRecord, StoreError>;
}
```

---

# 236. Why Domain Operations in Store

`acquire_lease()` needs one atomic DB operation.

A generic CRUD abstraction would encourage unsafe multi-step service logic.

---

# 237. Store Interface Granularity

Do not put business policy in store.

Store handles:

```text
atomic persistence semantics
constraints
query efficiency
```

Service handles:

```text
business decision
policy
authorization
```

---

# 238. Authorization Boundary

Store does not decide whether user is allowed to act.

Authz service decides.

Store enforces tenant/resource constraints.

---

# 239. Audit Boundary

Security-sensitive service writes audit event in same transaction where possible.

---

# 240. Immutable Audit Payload

If audit payload is large:

```text
CAS object
+
digest/reference in DB
```

---

# 241. CAS Missing Object

Metadata can temporarily reference missing CAS only in controlled transitional state.

Normal committed artifact should require CAS verification before metadata becomes visible.

---

# 242. Artifact Visibility States

```rust
pub enum ArtifactVisibilityState {
    Pending,
    Available,
    Quarantined,
    Deleted,
}
```

---

# 243. Upload Workflow

```text
create upload intent
  ↓
upload CAS
  ↓
verify digest
  ↓
transaction: artifact metadata = Available
```

---

# 244. Orphan CAS Cleanup

Failed upload metadata leaves CAS object unreferenced.

GC handles after grace.

---

# 245. Missing CAS Reconciliation

If metadata says Available but object absent:

```text
mark degraded/quarantined
alert
attempt replica recovery
```

---

# 246. Source Snapshot Visibility

A source snapshot should be registered only after its tree/blob closure is complete/verified.

---

# 247. Immutable Provenance Mapping

Store should reject:

```text
same SourceSnapshotId with different canonical tree
```

by construction/digest verification.

---

# 248. Check Evidence Storage

Check metadata in DB:

```text
state
kind
required
run ID
evidence refs
```

Large report in CAS.

---

# 249. Policy Decision Storage

Store:

```text
policy digest
decision
violations
exception refs
evaluation timestamp
```

---

# 250. Data Encryption

At rest encryption generally delegated to:

```text
filesystem/disk
managed Postgres
object store
```

Forgeyard must not invent custom transparent DB crypto casually.

Secrets use dedicated secret system.

---

# 251. Sensitive Metadata

Some metadata may still be sensitive:

```text
private repo URLs
audit data
user identifiers
```

Protect through:

```text
DB access control
TLS
RBAC
backup encryption
```

---

# 252. Data Classification

Optional model:

```rust
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}
```

Can drive retention/export policy later.

---

# 253. Export Policy

Admin exports respect classification.

---

# 254. SQL Logging

Never log bind values by default in production.

---

# 255. PII Minimization

Store only user/profile fields needed for operation.

Identity provider remains source for broader profile data when possible.

---

# 256. Audit Immutability

Administrative repair of audit data should be extremely restricted and separately recorded.

---

# 257. Schema Tests

Automated tests verify:

```text
foreign keys present
unique constraints present
expected indexes present
migration checksums
```

---

# 258. Store Architecture Checker

Potential static check ensures:

```text
no sqlx dependency outside postgres adapter
no Stoolap dependency outside Stoolap adapter
```

---

# 259. Cargo Dependency Rule

`sqlx` / Postgres crate allowed only in:

```text
forgeyard-store-postgres
migration/admin helpers
```

not domain.

---

# 260. Stoolap Dependency Rule

Stoolap crate allowed only in:

```text
forgeyard-store-stoolap
```

and its test helpers.

---

# 261. DB Serialization Types

Adapters convert:

```text
DB integer -> typed enum
DB bytes -> typed ID/digest
DB timestamp -> forgeyard Timestamp
```

Conversion is fallible.

---

# 262. Invalid DB Value

If DB contains invalid enum/state:

```text
StoreError::Corruption
```

not default fallback.

---

# 263. Store Startup

Startup order:

```text
load config
resolve secret
connect
check DB version
check migrations
run allowed migrations
verify health
start services
```

---

# 264. Standalone Startup

```text
ensure data dir
acquire process lock
open Stoolap
migrate
verify
start
```

---

# 265. Distributed Startup

```text
connect pool
acquire migration lock if needed
schema check
migrate
health/readiness
```

---

# 266. Migration Ownership in HA

One daemon migrates.

Others wait/read migration state.

---

# 267. Zero-Downtime Upgrade Example

```text
v1 schema
  ↓ expand migration
v1 + v2 code compatible
  ↓ deploy v2
backfill
  ↓
later contract migration
```

---

# 268. Store Version Compatibility

```rust
pub struct StoreCompatibility {
    pub min_schema: StoreSchemaVersion,
    pub max_schema: StoreSchemaVersion,
}
```

---

# 269. Store Health API

```rust
pub trait StoreHealth {
    async fn health(&self) -> StorageHealthReport;
}
```

---

# 270. Health Report

```rust
pub struct StorageHealthReport {
    pub status: StorageHealth,
    pub schema: StoreSchemaVersion,
    pub latency: Duration,
    pub details: Vec<HealthDetail>,
}
```

---

# 271. Testkit Structure

```text
forgeyard-store-testkit/src/
├── lib.rs
├── project.rs
├── run.rs
├── job.rs
├── runner.rs
├── source.rs
├── change.rs
├── artifact.rs
├── release.rs
├── transaction.rs
├── idempotency.rs
├── tenant.rs
└── migration.rs
```

---

# 272. Benchmark Structure

```text
benches/
├── job_claim.rs
├── run_create.rs
├── proposal_read.rs
├── audit_append.rs
└── outbox_claim.rs
```

---

# 273. Migration Directory

```text
migrations/postgres/
├── 0001_core.sql
├── 0002_runs_jobs.sql
├── 0003_sources_vcs.sql
├── 0004_changes.sql
├── 0005_artifacts.sql
├── 0006_release_deploy.sql
├── 0007_audit_outbox.sql
└── ...
```

Stoolap migration representation should map equivalent domain schema semantics.

---

# 274. Logical Schema Groups

```text
core
project
run/job
runner
source/vcs
change proposal
artifact/cache refs
release/deployment
policy
audit/events
coordination metadata
```

---

# 275. Store Module Ownership

Each subsystem defines:

```text
domain models
store trait
```

but exact implementation sits under store adapters.

Alternative layout may keep trait near domain crate.

Recommended rule:

```text
domain-facing persistence interface near domain
adapter implementation under store backend
```

Avoid circular dependencies.

---

# 276. Preferred Interface Placement

Example:

```text
forgeyard-run-store-api
```

or:

```text
forgeyard-run/src/store.rs
```

depending complexity.

The workspace structure can evolve, but adapter independence must remain.

---

# 277. No ORM Domain Leakage

ORM/driver annotations stay adapter-local.

---

# 278. Query Builder Choice

Use a mature Rust Postgres client/query approach.

But architecture is independent of exact library.

Do not make query library part of domain model.

---

# 279. SQL Compile-Time Checking

If using a library with compile-time query checking, integrate it into CI carefully.

Do not require production DB connectivity just to compile release unless offline query metadata is supported.

---

# 280. Prepared Query Metadata

If used, keep generated metadata versioned/updated by tooling.

---

# 281. Store Doctor Categories

```text
REQUIRED
OPTIONAL
PERFORMANCE
SECURITY
```

Example:

```text
REQUIRED: schema compatible
PERFORMANCE: missing recommended index
SECURITY: TLS disabled in production
```

---

# 282. Local Dev DB Reset

Command:

```text
forgeyard dev storage reset
```

only for development profiles.

Never expose destructive reset in production without strong safety.

---

# 283. Fixture Loading

Test/development only:

```text
forgeyard dev storage seed
```

---

# 284. Schema Documentation Generation

Generate:

```text
docs/reference/database-schema.md
```

from migrations/schema tooling where possible.

---

# 285. Capacity Planning

Important metadata dimensions:

```text
runs/day
jobs/run
log refs
artifacts/run
proposal activity
audit events/day
runner count
```

---

# 286. Data Growth

Large-growth areas:

```text
job attempts
events
audit
outbox
log metadata
```

need retention/partition strategy.

---

# 287. Partitioning

Do not introduce Postgres partitioning on day one.

Add for:

```text
audit/event scale
very large job history
```

when measurements justify it.

---

# 288. Archival

Old historical metadata can be archived/exported later.

Core transactional correctness should not depend on archive tier.

---

# 289. Read Model

If UI queries become expensive:

```text
materialized read models
```

may be added.

Do not prematurely duplicate all domain data.

---

# 290. Read Model Consistency

Read models are derived/eventual.

Critical decisions always use authoritative tables/services.

---

# 291. Cache Above Store

In-memory/cache layer may cache:

```text
project config
policy bundle
toolchain metadata
```

but authoritative mutations still go to store.

---

# 292. Cache Invalidation

Use:

```text
version
event
TTL
```

not indefinite stale caching.

---

# 293. Database Is Not General Cache

Do not store:

```text
compiler cache blobs
dependency archives
container layers
```

inside Postgres.

---

# 294. PostgreSQL Large Objects

Avoid unless a very specific need appears.

CAS is the object store abstraction.

---

# 295. Binary Columns

Small hashes/IDs/evidence are appropriate.

Large artifacts are not.

---

# 296. Storage Architecture Security Invariants

1. agents never connect directly to DB;
2. UI never connects directly to DB;
3. domain crates never import DB drivers;
4. DB credentials are secret refs;
5. tenant scope is explicit;
6. parameterized queries only;
7. migrations are integrity checked;
8. backups are encrypted/protected operationally;
9. restore is tested;
10. audit writes are append-oriented.

---

# 297. Store Reliability Invariants

1. cross-system effects are not claimed exactly-once;
2. idempotency keys prevent duplicate command effects;
3. optimistic concurrency prevents lost updates;
4. leases expire and reconcile;
5. outbox prevents lost event publication;
6. DB corruption is distinct from not-found;
7. state + event are committed atomically where needed;
8. long external work is outside DB transaction.

---

# 298. Standalone Invariants

1. no external DB setup required;
2. one local authoritative Stoolap DB;
3. local backup is supported;
4. schema migrations are automatic when safe;
5. distributed-only features are not required for correctness.

---

# 299. Distributed Invariants

1. PostgreSQL/Neon is shared metadata authority;
2. CAS stores bulk immutable bytes;
3. Raft does not replace Postgres;
4. daemon replicas use compatible schema;
5. migrations support rolling upgrades;
6. database restore is followed by reconciliation.

---

# 300. Implementation Phase 1 — Store API

Implement:

```text
StoreError
EntityVersion
pagination
transaction API
ProjectStore
RunStore
JobStore
RunnerStore
```

---

# 301. Phase 2 — Stoolap

Implement:

```text
local schema
transactions
core stores
migration runner
health
backup
```

Exit:

```text
standalone Forgeyard core workflows persist locally
```

---

# 302. Phase 3 — PostgreSQL

Implement:

```text
pool
transactions
core schema
run/job/runner stores
source/artifact refs
```

---

# 303. Phase 4 — Conformance Testkit

Run same behavioral tests against:

```text
Stoolap
Postgres
```

---

# 304. Phase 5 — Idempotency / Outbox

Implement:

```text
idempotency table/service
outbox
inbox/dedup where needed
publisher reconciliation
```

---

# 305. Phase 6 — VCS / Change Persistence

Integrate already-designed:

```text
VCS metadata/provenance
Change Proposal/review/check/queue
```

without changing their domain contracts.

---

# 306. Phase 7 — Release / Deployment / Audit

Add:

```text
release
deployment
audit
policy decisions
```

---

# 307. Phase 8 — Migration Hardening

Implement:

```text
checksums
expand-contract
backfill framework
migration lock
N/N-1 tests
```

---

# 308. Phase 9 — Backup / Restore

Implement:

```text
metadata backup
restore
manifest
integrity verification
reconciliation
```

---

# 309. Phase 10 — HA / Performance

Implement/validate:

```text
pool tuning
failover behavior
read replicas if needed
large-scale indexes
queue hot path
scheduler hot path
```

---

# 310. Acceptance Tests

1. Same domain service works with Stoolap and Postgres.
2. Domain crates do not import DB driver.
3. Stale entity version update fails.
4. Same idempotency key does not duplicate run creation.
5. Job lease acquisition is atomic.
6. Expired lease reconciles correctly.
7. State transition + event commit atomically.
8. Outbox retry does not duplicate consumer effect.
9. Tenant A cannot read Tenant B resource.
10. CAS object bytes are not stored in metadata tables.
11. Artifact metadata points to verified CAS object.
12. Failed metadata transaction does not expose artifact.
13. Orphan CAS object becomes GC-eligible.
14. Postgres migration checksum mismatch fails startup.
15. Stoolap migration produces compatible domain behavior.
16. N-1 compatible daemon can coexist during expand phase.
17. Backup restore returns same critical metadata.
18. Restore reconciliation marks stale running jobs correctly.
19. Integration queue candidate is revalidated after restore.
20. Store health detects schema mismatch.
21. Database credential never appears in logs.
22. Audit event cannot be casually updated.
23. Revision→snapshot mutation is rejected.
24. High-volume log bytes remain outside SQL.
25. Scheduler can claim work without table-wide scans.

---

# 311. Production Readiness Gates

Do not call storage production-ready until:

```text
Stoolap adapter passes conformance
Postgres adapter passes conformance
transaction semantics documented
optimistic concurrency works
idempotency works
outbox/reconciliation works
tenant isolation tested
migrations are checksum-verified
expand-contract tested
backup/restore tested
CAS reference integrity checks exist
health/metrics exist
```

---

# 312. Final Architecture

```text
                         Domain Services
                               │
                               ▼
                   Domain Persistence Interfaces
                               │
                 ┌─────────────┴─────────────┐
                 ▼                           ▼
          StoolapStore                 PostgresStore
                 │                           │
                 ▼                           ▼
         local metadata DB          PostgreSQL / Neon
                 │                           │
                 └─────────────┬─────────────┘
                               ▼
                        Metadata State
                  ┌────────────┼────────────┐
                  ▼            ▼            ▼
              Runs/Jobs      VCS/Change   Release/Audit
                  │            │            │
                  └────────────┼────────────┘
                               ▼
                           CAS References
                               │
                               ▼
                               CAS
```

---

# 313. Final Architectural Position

The persistence rule is:

```text
business operation
   ↓
service validates policy/invariants
   ↓
typed store operation
   ↓
transaction
   ↓
metadata mutation
+ durable event/outbox
   ↓
commit
```

For large immutable data:

```text
produce bytes
  ↓
write/verify CAS
  ↓
transactionally register metadata reference
  ↓
reconcile/GC orphan objects
```

Standalone:

```text
ForgeyardStore
  ↓
Stoolap
```

Distributed:

```text
ForgeyardStore
  ↓
PostgreSQL / Neon
```

The domain does not change.

This is the key guarantee:

> **Forgeyard can move from a zero-setup local binary to a shared enterprise deployment without rewriting business logic around a different database, while still using database-specific optimizations safely inside adapters.**

---

# 314. Recommended New-Repository Sequence

After `01-forgeyard-core-domain-foundation.md`, implement this subsystem before:

```text
pipeline execution
scheduler
runner
Change Proposal persistence
release state
distributed HA
```

because all of them depend on stable transactional state semantics.

Recommended implementation order now becomes:

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
14 Artifact / Packaging
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

The previously completed language, hermetic, VCS-neutral, and Change Proposal documents plug into these foundations rather than being rewritten.
