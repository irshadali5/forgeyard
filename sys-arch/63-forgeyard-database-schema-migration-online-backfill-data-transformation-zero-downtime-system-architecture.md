# 63 — Forgeyard Database Schema Migration, Online Backfill, Data Transformation & Zero-Downtime Change Orchestration System Architecture

**Document type:** Core Database Schema Migration, Online Backfill, Data Transformation, Expand-Contract, Cutover, Rollback & Zero-Downtime Change Orchestration System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** database schema evolution, migration plans, migration generations, DDL orchestration, online backfills, chunked data transformation, expand-contract rollout, dual-read/write transitions, compatibility windows, migration verification, cutover, rollback limits, tenant-by-tenant migration, drift detection, resumability, throttling, migration locks, and recovery  
**Architecture style:** Explicit migration state, compatibility-first evolution, resumable data work, bounded batches, idempotent checkpoints, exact schema generations, separate schema/data phases, guarded cutovers, fail-safe recovery, and no opaque `migrate up` hidden in deployment scripts  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Storage/Metadata, Deployment, Infrastructure-as-Code, Compatibility Governance, Test Data, Concurrency/Fencing, Progressive Delivery, Reliability/SLOs, Multi-Tenancy, Audit, Incident Management, and Data Lifecycle. This subsystem provides the first-class state machine required for safe online schema and data evolution.

---

## 1. Purpose

Database changes are among the highest-risk effects in a production system.

They can involve:

```text
DDL
indexes
constraints
column/type changes
table rewrites
backfills
data normalization
key migration
encryption changes
tenant moves
denormalization
partitioning
```

Naive CI/CD often treats migration as:

```text
deploy app
  ↓
run `migrate up`
  ↓
hope it finishes
```

That is insufficient for:

```text
large datasets
rolling deployments
multi-version clients
multi-region systems
high availability
tenant isolation
long-running backfills
rollback
```

The central rule is:

> **A database migration is a first-class durable workflow with explicit phases, checkpoints, compatibility windows, authority, verification, and recovery semantics.**

A second rule is:

> **Schema migration and data migration are related but distinct. DDL completion does not imply data transformation completion.**

A third rule is:

> **Rollback is never assumed. Every migration declares whether rollback is safe, partial, manual, or impossible.**

---

## 2. Architectural Position

```text
                   Proposed Schema/Data Change
                              │
                              ▼
                       Migration Plan
                              │
                              ▼
                     Compatibility Check
                              │
                              ▼
                          Expand Phase
                              │
                              ▼
                      Application Rollout
                              │
                              ▼
                         Backfill Phase
                              │
                              ▼
                        Verify / Cutover
                              │
                              ▼
                         Contract Phase
```

---

## 3. Goals

The subsystem MUST:

1. define migration identity;
2. define schema generation identity;
3. define migration plan identity;
4. support expand-contract;
5. support online DDL;
6. support resumable backfills;
7. support chunking;
8. support throttling;
9. support checkpoints;
10. support dual-read/write transitions;
11. support cutover;
12. support verification;
13. support tenant-by-tenant migration;
14. support multi-region migration authority;
15. support zero/low-downtime migration;
16. support rollback classification;
17. support manual recovery;
18. support migration locks/fencing;
19. support schema drift detection;
20. support migration test environments;
21. support dry-run/planning;
22. support observability;
23. support Dioxus UI/API/CLI;
24. support audit;
25. support incident integration;
26. support DR;
27. support air-gap;
28. support large datasets;
29. support policy gates;
30. prevent opaque migration side effects hidden in deployments.

---

## 4. Non-Goals

This subsystem does not replace:

```text
database engines
ORM migration tools
SQL parsers
application compatibility testing
backup systems
deployment orchestration
```

It governs them.

---

## 5. Workspace Structure

```text
crates/migration/
├── forgeyard-migration/
├── forgeyard-migration-model/
├── forgeyard-migration-plan/
├── forgeyard-migration-schema/
├── forgeyard-migration-backfill/
├── forgeyard-migration-cutover/
├── forgeyard-migration-verify/
├── forgeyard-migration-drift/
├── forgeyard-migration-reconcile/
├── forgeyard-migration-health/
└── forgeyard-migration-testkit/
```

Adapters:

```text
crates/migration-adapters/
├── forgeyard-migration-postgres/
├── forgeyard-migration-stoolap/
├── forgeyard-migration-sqlite/
└── forgeyard-migration-custom/
```

---

## 6. MigrationId

```rust
pub struct MigrationId(Ulid);
```

One durable migration workflow.

---

## 7. MigrationPlanId

```rust
pub struct MigrationPlanId(Digest);
```

Immutable identity of exact migration intent.

---

## 8. SchemaGenerationId

```rust
pub struct SchemaGenerationId(Digest);
```

Represents exact logical schema generation.

---

## 9. DatabaseTarget

```rust
pub struct DatabaseTarget {
    pub database: DatabaseId,
    pub tenant: Option<TenantId>,
    pub region: Option<RegionId>,
}
```

---

## 10. MigrationPlan

```rust
pub struct MigrationPlan {
    pub id: MigrationPlanId,
    pub target: DatabaseTarget,
    pub from: SchemaGenerationId,
    pub to: SchemaGenerationId,
    pub phases: Vec<MigrationPhaseSpec>,
    pub compatibility: CompatibilityReportId,
}
```

---

## 11. Migration Phase

```rust
pub enum MigrationPhaseSpec {
    Expand(SchemaChangeSet),
    DeployCompatibleApplication(ReleaseId),
    Backfill(BackfillSpec),
    Verify(VerificationSpec),
    Cutover(CutoverSpec),
    Contract(SchemaChangeSet),
}
```

---

## 12. Expand-Contract Baseline

Preferred online sequence:

```text
old schema
   ↓
expand with backward-compatible changes
   ↓
old + new app versions coexist
   ↓
backfill
   ↓
switch reads/writes
   ↓
verify
   ↓
contract old fields
```

---

## 13. MigrationState

```rust
pub enum MigrationState {
    Planned,
    AwaitingApproval,
    Expanding,
    WaitingForApplication,
    Backfilling,
    Verifying,
    AwaitingCutover,
    CuttingOver,
    Contracting,
    Succeeded,
    Paused,
    Failed,
    Cancelled,
    Unknown,
}
```

---

## 14. Unknown

First-class for ambiguous external/database effects.

Never treated as success.

---

## 15. Schema Change Classification

```rust
pub enum SchemaChangeRisk {
    Additive,
    OnlineCompatible,
    RequiresRewrite,
    Destructive,
    Irreversible,
    Unknown,
}
```

---

## 16. Additive Change

Examples:

```text
new nullable column
new table
new non-blocking index where supported
```

---

## 17. Potentially Blocking Change

Examples:

```text
large table rewrite
type conversion
constraint validation
exclusive lock requirement
```

---

## 18. Database-Capability Awareness

Migration planner must know engine/version capabilities.

---

## 19. No Universal "Online DDL"

Critical.

---

## 20. MigrationEngineCapabilities

```rust
pub struct MigrationEngineCapabilities {
    pub concurrent_index: bool,
    pub online_constraint_validation: bool,
    pub transactional_ddl: bool,
    pub lock_timeout_control: bool,
}
```

---

## 21. Migration Source

Can come from:

```text
SQL file
ORM-generated migration
Forgeyard-native typed migration spec
custom engine adapter
```

---

## 22. Migration Artifact

Exact source digest retained.

---

## 23. Schema Introspection

Before planning, capture actual schema.

---

## 24. SchemaSnapshot

```rust
pub struct SchemaSnapshot {
    pub generation: SchemaGenerationId,
    pub observed_at: Timestamp,
    pub engine_version: DatabaseEngineVersion,
}
```

---

## 25. Desired vs Observed

Critical distinction.

---

## 26. Schema Drift

```rust
pub enum SchemaDrift {
    InSync,
    UnexpectedObject,
    MissingObject,
    ModifiedObject,
    Unknown,
}
```

---

## 27. Migration Preconditions

Before start:

```text
current schema generation matches expected
no incompatible active migration
backup/recovery requirements satisfied
compatibility report valid
required application versions available
```

---

## 28. Schema Generation Precondition

No blind migration from unknown schema.

---

## 29. Migration Lock

Part 60.

Scope:

```text
DatabaseId / tenant shard / schema domain
```

---

## 30. Fencing

Migration commands include current fencing token.

---

## 31. Lock Does Not Replace Schema Generation Check

Critical.

---

## 32. DDL Execution

Each step has:

```text
expected schema
statement digest
timeout
lock budget
retry safety
```

---

## 33. StatementId

```rust
pub struct MigrationStatementId(Digest);
```

---

## 34. DDL Retry Safety

Database-engine specific.

---

## 35. Generic Blind Retry

Forbidden.

---

## 36. Transactional DDL

Use where engine supports and safe.

---

## 37. Non-Transactional DDL

Needs explicit checkpoint/inspection.

---

## 38. Lock Timeout

Bounded.

---

## 39. Statement Timeout

Bounded.

---

## 40. Long Exclusive Lock

Policy can block.

---

## 41. Online Index Build

Prefer concurrent/online form where engine supports.

---

## 42. Constraint Introduction

Often:

```text
add constraint not validated
  ↓
backfill/fix data
  ↓
validate
```

---

## 43. Column Rename

Safer pattern:

```text
add new column
dual-write
backfill
switch reads
stop old writes
drop old later
```

---

## 44. Type Change

May require shadow column/table.

---

## 45. Primary Key Migration

High risk.

Requires explicit application compatibility phase.

---

## 46. Encryption/Key Migration

Can involve dual-read/write with key versioning.

---

## 47. BackfillId

```rust
pub struct BackfillId(Ulid);
```

---

## 48. BackfillSpec

```rust
pub struct BackfillSpec {
    pub source: BackfillSource,
    pub target: BackfillTarget,
    pub chunking: ChunkingStrategy,
    pub throttle: BackfillThrottle,
    pub verification: BackfillVerification,
}
```

---

## 49. Chunking Strategy

```rust
pub enum ChunkingStrategy {
    PrimaryKeyRange,
    TimestampRange,
    HashBuckets,
    Partition,
    Custom(ChunkingStrategyId),
}
```

---

## 50. ChunkId

```rust
pub struct BackfillChunkId(Digest);
```

---

## 51. Chunk State

```rust
pub enum BackfillChunkState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Retryable,
    Skipped,
}
```

---

## 52. Resumability

Backfill progress persisted per chunk.

---

## 53. Idempotency

Chunk transformation should be idempotent where possible.

---

## 54. Checkpoint

```rust
pub struct BackfillCheckpoint {
    pub backfill: BackfillId,
    pub completed: u64,
    pub last_position: Option<BackfillCursor>,
}
```

---

## 55. No "start from row 1 again"

Critical for large datasets.

---

## 56. Backfill Throttle

```rust
pub struct BackfillThrottle {
    pub rows_per_second: Option<u64>,
    pub max_concurrency: u16,
    pub max_db_cpu_percent: Option<Percentage>,
}
```

---

## 57. Adaptive Throttling

Can react to:

```text
DB CPU
replication lag
lock wait
query latency
error rate
```

---

## 58. Reliability Guard

Part 50.

---

## 59. SLO Burn

Can pause/reduce backfill.

---

## 60. Backfill Is Not User Traffic

Must yield to production workload.

---

## 61. Backfill Priority

Lower than critical serving traffic by default.

---

## 62. Backfill Pause

Durable.

---

## 63. Resume

From checkpoint.

---

## 64. Backfill Verification

Options:

```text
row counts
checksums
sample verification
full predicate verification
application-level invariant
```

---

## 65. Verification Confidence

```rust
pub enum BackfillVerificationConfidence {
    Full,
    Strong,
    Sampled,
    Weak,
    Unknown,
}
```

---

## 66. Critical Data Migration

May require Full/Strong.

---

## 67. Checksum

Use canonical representation.

---

## 68. Dual Write

Application writes old + new representations during transition.

---

## 69. DualWriteState

```rust
pub enum DualWriteState {
    Disabled,
    Shadow,
    Enabled,
    NewPrimary,
    OldDisabled,
}
```

---

## 70. Shadow

Write new path but old remains authoritative.

---

## 71. Read Shadowing

Compare new read path with old.

---

## 72. ReadMode

```rust
pub enum ReadMode {
    OldOnly,
    OldPrimaryCompareNew,
    NewPrimaryFallbackOld,
    NewOnly,
}
```

---

## 73. Data Divergence

First-class finding.

---

## 74. DivergenceFindingId

```rust
pub struct DivergenceFindingId(Ulid);
```

---

## 75. Cutover

Switch authority from old representation to new.

---

## 76. CutoverId

```rust
pub struct CutoverId(Ulid);
```

---

## 77. Cutover Preconditions

```text
backfill complete
verification passed
application compatibility valid
replication healthy
incident/change-freeze policy permits
rollback class understood
```

---

## 78. Cutover Is High-Risk

Often manual approval required.

---

## 79. Cutover State

```rust
pub enum CutoverState {
    Planned,
    AwaitingApproval,
    Executing,
    Verified,
    Failed,
    Unknown,
}
```

---

## 80. Unknown Cutover

Inspect actual application/database routing before retry.

---

## 81. No Blind Flip Retry

Critical.

---

## 82. Contract Phase

Remove old schema/columns only after compatibility window closes.

---

## 83. Contract Delay

Often days/weeks/releases.

---

## 84. Deprecation Window

Part 57.

---

## 85. No Immediate Drop After Cutover

Critical baseline.

---

## 86. RollbackClassification

```rust
pub enum MigrationRollbackClass {
    FullyReversible,
    ReversibleBeforeCutover,
    ReversibleWithDataLossRisk,
    RollForwardOnly,
    ManualRecovery,
    Irreversible,
    Unknown,
}
```

---

## 87. Rollback Plan

```rust
pub struct MigrationRollbackPlan {
    pub class: MigrationRollbackClass,
    pub steps: Vec<RollbackStep>,
}
```

---

## 88. Rollback Is Tested

Where feasible.

---

## 89. Previous Schema

Reapplying old migration blindly is not rollback.

---

## 90. Data Written After Cutover

May make old app incompatible.

---

## 91. Roll Forward

Often safest.

---

## 92. Deployment Integration

Application rollout must respect schema compatibility.

---

## 93. Example

```text
Expand schema
  ↓
deploy app version supporting old+new
  ↓
backfill
  ↓
switch reads
```

---

## 94. No New App Before Required Schema Expand

---

## 95. No Contract Before Old App Removed

Critical.

---

## 96. Compatibility Matrix

Part 57 authoritative.

---

## 97. Mixed-Version Testing

Part 56.

---

## 98. Migration Test Plan

Test:

```text
old schema + old app
expanded schema + old app
expanded schema + new app
backfilled schema + new app
```

---

## 99. Migration Fixtures

Use realistic synthetic datasets.

---

## 100. Production Snapshot Test

Only masked/approved per Part 56.

---

## 101. Dry Run

For compatible engines:

```text
plan SQL
estimate locks
estimate rows
estimate rewrite
```

---

## 102. Cost Estimate

Advisory.

---

## 103. Lock Risk Estimate

Important.

---

## 104. Large Table Rewrite

Can be blocked automatically by policy.

---

## 105. Table Size Snapshot

Captured before apply.

---

## 106. Migration Risk

```rust
pub enum MigrationRisk {
    Low,
    Moderate,
    High,
    Critical,
    Unknown,
}
```

---

## 107. Risk Inputs

```text
DDL lock class
table size
row rewrite
backfill volume
irreversibility
production criticality
multi-region scope
```

---

## 108. Policy Gates

Can require:

```text
backup evidence
maintenance window
manual approval
compatibility report
mixed-version tests
rollback plan
capacity headroom
```

---

## 109. Backup Evidence

Part 25.

---

## 110. Backup Does Not Make Unsafe Migration Safe

Critical.

---

## 111. PITR

Required for some high-risk operations.

---

## 112. Recovery Drill

Can be required for critical migrations.

---

## 113. Tenant-by-Tenant Migration

SaaS systems may migrate cohorts.

---

## 114. TenantMigrationId

```rust
pub struct TenantMigrationId(Ulid);
```

---

## 115. Tenant Rollout

```text
pilot tenant
  ↓
small cohort
  ↓
larger cohort
  ↓
all tenants
```

---

## 116. Tenant Isolation

Failures in one tenant should not corrupt migration state of others.

---

## 117. Tenant Checkpoint

Per-tenant status.

---

## 118. Tenant Rollback

May differ if data transformation irreversible.

---

## 119. Sharded Database

Migration per shard.

---

## 120. ShardMigrationId

```rust
pub struct ShardMigrationId(Ulid);
```

---

## 121. Shard Rollout

Canary shards first.

---

## 122. Global Cutover

Only after required shard readiness.

---

## 123. Multi-Region Database

Complex.

---

## 124. Authority

Part 51 single mutation authority for migration domain.

---

## 125. Replica Compatibility

Read replicas may lag schema.

---

## 126. Migration Ordering

Primary then replicas/provider-specific behavior.

---

## 127. Multi-Region Application

Must tolerate transitional schema.

---

## 128. Cross-Region Backfill

Residency aware.

---

## 129. No Moving Restricted Data For Convenience

Critical.

---

## 130. Migration Scheduling

Can use Part 44 schedules/windows.

---

## 131. Maintenance Window

Optional.

---

## 132. Zero-Downtime

Goal, not promise.

---

## 133. DowntimeRequirement

```rust
pub enum DowntimeRequirement {
    NoneExpected,
    BriefMaintenance,
    PlannedMaintenance,
    Unknown,
}
```

---

## 134. If Engine Cannot Online-Migrate

Say so.

---

## 135. No Fake Zero-Downtime Claim

Critical.

---

## 136. Schema Version Table

Database adapter may maintain Forgeyard migration metadata.

---

## 137. But

Do not rely solely on migration table.

Observed schema still checked.

---

## 138. Drift After Manual DBA Change

Detected.

---

## 139. Drift Policy

Can:

```text
freeze migration
require adoption
reconcile
```

---

## 140. Brownfield Adoption

Existing DB schema can be imported as generation baseline.

---

## 141. Adoption Requires

```text
introspection
owner approval
no hidden pending migration
backup/recovery state
```

---

## 142. Migration Artifact Signing

Optional high assurance.

---

## 143. Migration Provenance

```rust
pub struct MigrationProvenance {
    pub plan: MigrationPlanId,
    pub source: SourceSnapshotId,
    pub release: Option<ReleaseId>,
    pub compatibility: CompatibilityReportId,
}
```

---

## 144. Audit

Audit:

```text
migration approval
cutover
rollback attempt
force resume
manual checkpoint override
contract phase
drift adoption
```

---

## 145. Routine Chunk Completion

Operational telemetry, not privileged audit spam.

---

## 146. Dioxus UI

Pages:

```text
Database Migrations
Migration Plans
Backfills
Schema Drift
Cutovers
Migration History
```

---

## 147. Migration Detail

Shows:

```text
from/to schema
phase
progress
risk
compatibility
backfill
cutover
rollback class
```

---

## 148. Backfill Progress

Shows:

```text
rows/chunks complete
throughput
ETA estimate
DB health
pause reason
```

---

## 149. ETA

Estimate only.

---

## 150. CLI

```text
forgeyard migrate plan
forgeyard migrate apply
forgeyard migrate status
forgeyard migrate pause
forgeyard migrate resume
forgeyard migrate verify
forgeyard migrate cutover
forgeyard migrate rollback
forgeyard migrate drift
forgeyard migrate doctor
```

---

## 151. API

Potential:

```text
POST /v1/migrations
GET  /v1/migrations/{id}
POST /v1/migrations/{id}/start
POST /v1/migrations/{id}/pause
POST /v1/migrations/{id}/resume
POST /v1/migrations/{id}/cutover
POST /v1/migrations/{id}/rollback
```

---

## 152. Permissions

```text
migration.read
migration.plan
migration.apply
migration.pause
migration.resume
migration.cutover
migration.rollback
migration.force
```

---

## 153. Cutover

High privilege.

---

## 154. Force

Highest privilege.

---

## 155. Concurrency

Part 60.

One migration authority per schema domain.

---

## 156. Fencing Token

Stored with migration phase execution.

---

## 157. Controller Crash

New controller reconciles DB state/checkpoint.

---

## 158. No Restart From Scratch

Critical.

---

## 159. Backfill Worker

Can horizontally scale if chunking allows.

---

## 160. Chunk Claim

Lease/idempotency.

---

## 161. Same Chunk

At most one accepted active worker.

---

## 162. Duplicate Completion

Idempotent.

---

## 163. Retry

Chunk-specific.

---

## 164. Poison Chunk

Repeated failures become blocked/manual.

---

## 165. Dead Letter

Not data loss; explicit failed chunk state.

---

## 166. Data Transformation Code

Exact version/digest.

---

## 167. TransformerId

```rust
pub struct TransformerId(Digest);
```

---

## 168. Transformer Upgrade Mid-Migration

Creates new migration plan/version unless policy explicitly supports staged transformer semantics.

---

## 169. No Silent Logic Change Mid-Backfill

Critical.

---

## 170. Backfill Consistency

Need strategy for concurrent writes.

---

## 171. Options

```text
dual-write
change-data-capture
version predicate
reconciliation pass
```

---

## 172. CDC

Optional adapter.

---

## 173. CDC Cursor

Checkpointed.

---

## 174. No Lost Tail Writes

Critical.

---

## 175. Verification Pass

After initial backfill, compare/reconcile recent writes.

---

## 176. Cutover Fence

Can briefly block specific writes if needed.

---

## 177. Write Quiescence

Explicit if required.

---

## 178. Quiescence Window

Bounded.

---

## 179. No Hidden Full-Maintenance Mode

Critical.

---

## 180. Reliability Metrics

```text
migration_duration
backfill_rate
chunk_failure_rate
DB CPU
replication lag
lock wait
cutover latency
```

---

## 181. SLO Guard

Can pause if production health degrades.

---

## 182. Incident Integration

Part 61.

Migration causing incident:

```text
pause
link IncidentId
freeze further cutover
```

---

## 183. Incident Resolution

Does not auto-resume migration.

---

## 184. Resume Freshness

Re-check:

```text
schema state
compatibility
backfill
health
policy
```

---

## 185. Progressive Delivery

Part 62.

Migration phases can gate application promotion.

---

## 186. Example

```text
schema expand complete
  ↓
allow app canary
```

---

## 187. Contract Phase

Only after all old app versions removed.

---

## 188. Client Offline Compatibility

Important for local-first systems.

---

## 189. Old Clients

May reconnect later.

---

## 190. Server Schema Change

Must preserve protocol/domain compatibility or reject unsupported client version explicitly.

---

## 191. Local Embedded DB Migration

Same principles, smaller scope.

---

## 192. Device Migration

Can happen on app startup/update.

---

## 193. LocalMigrationPlanId

```rust
pub struct LocalMigrationPlanId(Digest);
```

---

## 194. Local DB Backup

Before irreversible local migration where feasible.

---

## 195. Crash Recovery

Migration resumes from local checkpoint.

---

## 196. No Half-Upgraded Local Schema Without Detection

Critical.

---

## 197. Offline Upgrade

Supported with bundled migration plan/tool.

---

## 198. Schema Compatibility Across Sync

For Aequora-like systems:

```text
client schema version
server schema version
sync protocol version
```

must be compatible.

---

## 199. Data Contract

Separate from physical schema where appropriate.

---

## 200. Data Lifecycle

Part 46.

Old columns/data may need retention before deletion.

---

## 201. Contract/Delete

Respect legal holds and retention.

---

## 202. PII Migration

Privacy policy.

---

## 203. Data Residency

Backfill must stay allowed region/site.

---

## 204. Security

Threats:

```text
destructive migration abuse
SQL injection in migration tooling
privilege escalation
unbounded backfill
data corruption
schema drift
stale cutover controller
```

---

## 205. Migration Worker Credentials

Least privilege.

---

## 206. DDL Privilege

Only migration worker/service identity.

---

## 207. Application Runtime

Should not need schema-admin privileges.

---

## 208. SQL File

Untrusted project input until policy/review.

---

## 209. Migration Parser

Bounded/sandboxed where appropriate.

---

## 210. No `psql` Shell Escape

Critical.

---

## 211. Raw SQL

May still be supported, but executed in controlled DB adapter.

---

## 212. Database Connection

Private network grant Part 59 + SecretRef.

---

## 213. No Broad Production Credential In Build Job

Critical.

---

## 214. Migration Worker

Dedicated trusted executor class.

---

## 215. Secret Rotation

Migration credentials short-lived.

---

## 216. Observability

Metrics:

```text
migrations_active
migration_phase_duration_seconds
backfill_chunks_total
backfill_chunks_failed_total
backfill_rows_processed_total
migration_paused_total
migration_cutover_unknown_total
schema_drift_findings_total
```

---

## 217. Tracing

```text
migration.plan
migration.expand
migration.backfill
migration.verify
migration.cutover
migration.contract
migration.reconcile
```

---

## 218. Health

```rust
pub enum MigrationSubsystemHealth {
    Healthy,
    BackfillDegraded,
    DriftDetected,
    CutoverBlocked,
    Unhealthy,
}
```

---

## 219. Doctor

```text
forgeyard migrate doctor
```

Checks:

```text
stale migration lock
schema drift
stuck backfill
repeated poison chunks
unknown cutover
missing rollback plan
old app still active before contract
```

---

## 220. Search

Part 31 indexes migration metadata/history.

---

## 221. Cost

Part 45.

Backfill consumes:

```text
DB compute
IO
replication bandwidth
storage
```

---

## 222. Cost Guard

Can throttle optional migration speed.

---

## 223. Cost Cannot Skip Verification

Critical.

---

## 224. Federation

Migration authority bound to site/region.

---

## 225. Site Failover During Migration

New authority must reconcile DB state before continuation.

---

## 226. Old Site

Fenced by AuthorityEpoch.

---

## 227. Air-Gap

Migration works locally with bundled tools.

---

## 228. DR

Restore scenario:

```text
restore DB
  ↓
detect schema generation
  ↓
load migration metadata
  ↓
invalidate stale leases
  ↓
reconcile
```

---

## 229. Never Resume From Stale Pre-Restore Checkpoint Blindly

Critical.

---

## 230. Backup Restore

Schema and migration metadata may have different restore times.

Reconcile.

---

## 231. Migration Bundle

```rust
pub struct MigrationBundle {
    pub plan: MigrationPlanId,
    pub statements: CasObjectRef,
    pub transformer: Option<TransformerId>,
    pub verification: CasObjectRef,
}
```

---

## 232. Signed Bundle

Optional high assurance.

---

## 233. Testkit

```text
forgeyard-migration-testkit/src/
├── lib.rs
├── schema.rs
├── plan.rs
├── ddl.rs
├── backfill.rs
├── cutover.rs
├── rollback.rs
├── drift.rs
└── assertions.rs
```

---

## 234. Core Tests

### Schema
- expected generation required;
- drift blocks unsafe start;
- additive change classified correctly.

### Expand-Contract
- old app works after expand;
- new app works against transitional schema;
- contract blocked while old app exists.

### Backfill
- chunk checkpoints durable;
- retries idempotent;
- poison chunk visible;
- throttling obeys DB health.

### Dual-Write
- divergence detected;
- no lost tail writes.

### Cutover
- prerequisites enforced;
- timeout becomes Unknown;
- stale controller fenced.

### Rollback
- irreversible migration never gets fake rollback;
- rollback target compatibility checked.

### Tenant/Shards
- pilot cohort;
- one tenant failure isolated;
- shard rollout respects readiness.

### DR/Federation
- restored state reconciles;
- authority failover does not duplicate cutover.

---

## 235. Chaos Tests

Inject:

```text
DB failover during DDL
worker crash during backfill
network partition
replica lag spike
cutover API timeout
controller restart
storage pressure
```

Expected behavior:

```text
checkpoint survives
no duplicate destructive action
migration pauses/reconciles
health remains explicit
```

---

## 236. Scale Tests

Test:

```text
billions of rows via chunk simulation
thousands of tenant migrations
hundreds of shards
long-running multi-day backfills
```

---

## 237. Implementation Phases

### Phase 1 — Migration/Schema Model
Canonical identities/state.

### Phase 2 — PostgreSQL Adapter
Primary distributed backend.

### Phase 3 — Expand/Contract Workflow
Compatibility-safe schema evolution.

### Phase 4 — Resumable Backfill Engine
Chunking/checkpoints/throttling.

### Phase 5 — Verification/Cutover
Safe authority transition.

### Phase 6 — Tenant/Shard Rollouts
Large SaaS systems.

### Phase 7 — Drift Detection
Brownfield/manual DBA safety.

### Phase 8 — Embedded/Local DB Migration
Standalone/local-first.

### Phase 9 — Federation/DR
Multi-region recovery.

### Phase 10 — UI/CLI/Doctor
Operability.

### Phase 11 — Advanced CDC/Dual-Write
Complex transformations.

### Phase 12 — Chaos/Scale/Security Hardening
Production readiness.

---

## 238. Acceptance Tests

1. Migration binds exact source and target schema generations.
2. Migration plan is immutable/versioned.
3. Schema and data migration phases remain distinct.
4. Expand-contract is baseline for rolling-compatible changes.
5. Current observed schema must match expected baseline.
6. Schema drift cannot be ignored silently.
7. Migration lock and schema-generation check are both required where applicable.
8. Stale migration controller is fenced.
9. DDL retry safety is engine-specific.
10. Generic blind DDL retry is forbidden.
11. Long-running backfills are resumable.
12. Backfill chunks have durable checkpoints.
13. Chunk transformation version is immutable for a migration.
14. Backfill can throttle based on production DB health.
15. Missing verification does not permit cutover.
16. Unknown cutover state does not become success.
17. Rollback class is explicit.
18. Irreversible migration never exposes fake rollback.
19. Old application versions remain supported through transitional schema as policy requires.
20. Contract phase is blocked until old clients/apps are no longer supported.
21. Tenant/shard migration failures are isolated.
22. Multi-region authority prevents duplicate schema mutation.
23. Production-derived migration test data obeys Part 56 controls.
24. Migration worker uses least-privilege credentials.
25. Build jobs do not receive production DDL credentials.
26. Active incident/change freeze can pause migration.
27. Resume re-checks schema/health/policy freshness.
28. DR reconciles restored DB schema against migration metadata.
29. Local embedded DB migration supports crash recovery.
30. Forgeyard dogfoods the subsystem for its own Postgres/Stoolap schema evolution.

---

## 239. Production Readiness Gates

Do not call migration orchestration production-ready until:

```text
schema generation identity is stable
Postgres expand-contract path is dogfooded
backfill checkpoints survive crashes
cutover Unknown reconciliation is proven
rollback classification is enforced
schema drift detection works
tenant/shard isolation passes
migration credentials are least privilege
DR/federation reconciliation passes
chaos/scale tests pass
```

---

## 240. Architectural Invariants

1. migration is first-class durable state;
2. schema generation is explicit;
3. migration plan is immutable;
4. schema and data phases are distinct;
5. expand-contract is preferred;
6. current schema must match expected baseline;
7. drift is first-class;
8. migration ownership is fenced;
9. DDL retries are operation-specific;
10. backfills are resumable;
11. chunk progress is durable;
12. transformation logic cannot change silently mid-run;
13. verification precedes cutover;
14. Unknown is not success;
15. rollback capability is explicit/honest;
16. contract waits for compatibility window;
17. old/new apps coexist where rolling upgrade requires;
18. migration workers use scoped credentials;
19. production DDL authority is not given to normal build jobs;
20. reliability signals can throttle/pause data work;
21. incidents/change freezes can pause migration;
22. tenant/shard failures remain scoped;
23. federation authority prevents duplicate mutation;
24. DR reconciles actual schema before continuation;
25. local DB migration is crash-safe;
26. data lifecycle/residency apply to transformed data;
27. audit captures cutover/override/destructive actions;
28. no fake zero-downtime guarantee;
29. no opaque migration side effects hidden in deployment scripts;
30. Forgeyard dogfoods its own migration system.

---

## 241. Final Target Architecture

```text
                 MigrationPlanId
                       │
                       ▼
                  Schema Expand
                       │
                       ▼
              Compatible App Rollout
                       │
                       ▼
                Resumable Backfill
                       │
                       ▼
                    Verify
                       │
                       ▼
                    Cutover
                       │
                       ▼
               Compatibility Window
                       │
                       ▼
                    Contract
```

Backfill:

```text
large dataset
    ↓
deterministic chunking
    ↓
lease chunk
    ↓
transform
    ↓
verify
    ↓
checkpoint
    ↓
resume until complete
```

Cutover:

```text
backfill complete
+
verification passed
+
app compatibility valid
+
health acceptable
+
policy approval
   ↓
cutover
   ↓
observe actual state
   ↓
verified success / Unknown / recovery
```

The key guarantee is:

> **Forgeyard treats schema and data evolution as a governed distributed workflow rather than a one-shot script. Every migration has exact generations, compatibility evidence, resumable checkpoints, health-aware throttling, explicit cutover authority, and honest rollback semantics—so large online changes remain observable and recoverable even across crashes, failovers, and long-running transformations.**

---

## 242. Extended Architecture Sequence

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
```
