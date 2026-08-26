# 25 — Forgeyard Operations, Backup, Upgrade & Disaster Recovery System Architecture

**Document type:** Core Operations, Backup, Upgrade, Restore & Disaster Recovery System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** operational runbooks, backup policy, PostgreSQL/Neon recovery, Stoolap/local-mode backup, CAS durability/replication, restore verification, RPO/RTO, rolling upgrades, schema migrations, maintenance mode, corruption handling, secrets/trust recovery, HA coordination recovery, air-gapped restore, capacity management, incident recovery, and disaster drills  
**Architecture style:** Separate recovery domains for metadata, CAS, coordination, and secrets; verified backups; restore-first design; expand-contract upgrades; explicit RPO/RTO; immutable artifact preservation; and rehearsed disaster recovery rather than backup-only optimism  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on Storage/Metadata, CAS, HA/Coordination/Raft, Secrets/Trust, Events/Reconciliation, Observability/Doctor, API, Release, Deployment, and all stateful subsystems. It defines how Forgeyard is safely operated, upgraded, backed up, and recovered in production.

---

# 1. Purpose

A production CI/CD platform is not complete merely because it works when healthy.

Forgeyard must remain operable when:

```text
database fails
CAS object is corrupted
daemon version is broken
cluster loses quorum
credential is compromised
region is lost
operator makes a mistake
migration fails
disk fills
backup is incomplete
provider is unavailable
```

The central rule is:

> **A backup is not considered useful until Forgeyard has proven it can restore from it.**

A second rule is:

> **Metadata, immutable artifact bytes, coordination state, and secrets/trust material are separate recovery domains with different backup and restore semantics.**

A third rule is:

> **Production upgrades use expand-contract compatibility and rolling deployment whenever possible; destructive schema changes never assume all nodes upgrade simultaneously.**

---

# 2. Recovery Domains

Forgeyard recovery is divided into:

```text
1. Business metadata
2. CAS/object data
3. Coordination/Raft state
4. Secrets/trust material
5. Configuration
6. External provider state
```

---

# 3. Business Metadata

Authoritative distributed-mode metadata:

```text
PostgreSQL / Neon
```

Includes:

```text
Projects
Runs
Jobs
Attempts
Leases
Releases
Deployments
Policies
Identity metadata
Events
Outbox
Audit references
```

---

# 4. CAS Domain

Immutable bytes:

```text
source snapshots
artifacts
packages
SBOMs
provenance
diagnostic artifacts
release manifests
```

---

# 5. Coordination Domain

Raft/coordination:

```text
membership
leader/role epochs
maintenance state
exclusive coordination operations
```

Small and reconstructible by design.

---

# 6. Secret/Trust Domain

Includes:

```text
secret provider configuration
KEK/DEK hierarchy
internal CA
signing key references
trust roots
certificate state
```

---

# 7. Configuration Domain

Examples:

```text
forgeyard.ron
policy bundles
deployment config
plugin manifests
cluster config
```

---

# 8. External Provider State

Examples:

```text
GitHub checks
OCI tags
app-store release states
Kubernetes actual state
cloud deployment state
```

External state must be reconciled, not assumed restored from Forgeyard backup.

---

# 9. Goals

The subsystem MUST:

1. define backup policy;
2. define retention;
3. define RPO;
4. define RTO;
5. support PostgreSQL PITR;
6. support Neon recovery;
7. support local Stoolap backup;
8. support CAS replication;
9. support CAS backup;
10. support restore verification;
11. support corruption detection;
12. support backup encryption;
13. support secret/trust recovery;
14. support cluster recovery;
15. support rolling upgrade;
16. support expand-contract migrations;
17. support downgrade constraints;
18. support maintenance mode;
19. support capacity planning;
20. support disk-pressure handling;
21. support region loss;
22. support air-gap restore;
23. support incident runbooks;
24. support operator mistake recovery;
25. support disaster drills;
26. support automated restore tests;
27. support auditability;
28. support standalone mode;
29. support distributed mode;
30. avoid single-backup dependency.

---

# 10. Workspace Structure

```text
crates/operations/
├── forgeyard-operations/
├── forgeyard-operations-model/
├── forgeyard-backup/
├── forgeyard-restore/
├── forgeyard-upgrade/
├── forgeyard-migration/
├── forgeyard-dr/
├── forgeyard-maintenance/
├── forgeyard-capacity/
├── forgeyard-corruption/
├── forgeyard-runbook/
├── forgeyard-recovery-testkit/
└── forgeyard-operations-health/
```

Operational tooling:

```text
apps/forgeyard-migration/
tools/forgeyard-backup/
tools/forgeyard-restore/
tools/forgeyard-drill/
```

---

# 11. Backup Policy

```rust
pub struct BackupPolicy {
    pub metadata: MetadataBackupPolicy,
    pub cas: CasBackupPolicy,
    pub secrets: SecretBackupPolicy,
    pub retention: BackupRetentionPolicy,
}
```

---

# 12. Backup Set

```rust
pub struct BackupSetId(Ulid);
```

---

# 13. Backup Manifest

```rust
pub struct BackupManifest {
    pub id: BackupSetId,
    pub created_at: Timestamp,
    pub forgeyard_version: ForgeyardVersion,
    pub metadata: MetadataBackupRef,
    pub cas: CasBackupRef,
    pub coordination: CoordinationBackupRef,
    pub secrets: SecretBackupRef,
    pub config: ConfigBackupRef,
}
```

---

# 14. Backup Manifest Integrity

Signed or MAC/integrity protected.

---

# 15. Backup Encryption

Mandatory for sensitive backup sets.

---

# 16. Encryption Key Separation

Backup encryption key must not be stored only inside same backup.

---

# 17. Backup Storage

Use independent failure domain where practical.

---

# 18. RPO

Recovery Point Objective.

Example classes:

```text
Metadata: 5 minutes
CAS: near-zero for replicated immutable objects
Secrets: after every rotation/change
```

---

# 19. RTO

Recovery Time Objective.

Example:

```text
Control plane: < 1 hour
Critical release data: < 2 hours
Full historical artifact restoration: longer
```

Actual targets organization-specific.

---

# 20. Recovery Tiers

```rust
pub enum RecoveryTier {
    Tier0Critical,
    Tier1Operational,
    Tier2Historical,
}
```

---

# 21. Tier 0

Examples:

```text
current metadata
current releases
current deployment state
trust/secrets
```

---

# 22. Tier 1

Recent runs/artifacts.

---

# 23. Tier 2

Old historical build data.

---

# 24. PostgreSQL Backup

Use:

```text
continuous WAL/PITR
scheduled snapshots/base backups
```

---

# 25. Neon

Leverage provider-supported branch/PITR/restore features where available.

Forgeyard still validates restore.

---

# 26. PITR

Restore to timestamp before:

```text
operator error
bad migration
data corruption
```

---

# 27. Metadata Backup Consistency

Backup must reflect transactionally valid database state.

---

# 28. No App-Level Table Dump as Only Backup

Logical dumps useful, but not sufficient as sole production DR strategy.

---

# 29. Logical Export

Good for:

```text
migration
debug
small standalone environments
```

---

# 30. Physical/PITR Recovery

Primary distributed production strategy.

---

# 31. Backup Metadata

Record:

```text
LSN/timestamp
DB schema version
Forgeyard version
migration version
```

---

# 32. Restore Compatibility

Restore tool validates binary/schema compatibility.

---

# 33. Stoolap Standalone Backup

Single-machine mode must support consistent local snapshot.

---

# 34. Local Backup Sequence

```text
pause writes / snapshot-safe transaction
  ↓
copy/export embedded DB
  ↓
capture local CAS manifest
  ↓
config/secrets refs
```

---

# 35. Zero-External-Server Requirement

Standalone backup should work without cloud dependency.

---

# 36. Local Destination

Examples:

```text
external drive
NAS
user-selected directory
```

---

# 37. Local Backup Encryption

Optional but strongly recommended.

---

# 38. Local Automatic Backup

Configurable periodic backup.

---

# 39. Local Restore

One command/workflow.

---

# 40. CAS Backup Philosophy

CAS is immutable.

This simplifies replication.

---

# 41. CAS Durability Classes

```rust
pub enum CasDurabilityClass {
    EphemeralCache,
    DurableArtifact,
    ReleaseCritical,
    ComplianceRetained,
}
```

---

# 42. Ephemeral Cache

Can be regenerated.

Low backup priority.

---

# 43. Durable Artifact

Should survive normal storage failure.

---

# 44. Release Critical

Multiple replicas/backup.

---

# 45. Compliance Retained

Retention policy/legal requirements.

---

# 46. CAS Replication

Replicate by object digest.

---

# 47. Replication Target

Separate storage/site.

---

# 48. CAS Manifest Backup

Metadata lists required object digests.

---

# 49. Incremental CAS Backup

Only new immutable objects copied.

---

# 50. CAS Verification

Destination recomputes/verifies digest.

---

# 51. Silent Corruption

Detected by periodic scrubbing.

---

# 52. CAS Scrub

```rust
pub struct CasScrubPolicy {
    pub interval: Duration,
    pub sample_rate: Ratio,
}
```

---

# 53. Full Scrub

Periodic for release-critical sets.

---

# 54. Corrupt Object

Mark:

```text
Quarantined
```

---

# 55. Repair

Fetch healthy replica/backup and verify digest.

---

# 56. Missing Object

Use replica/backup.

---

# 57. Irrecoverable Missing Object

Metadata remains but artifact marked unavailable.

---

# 58. Do Not Forge Replacement

Never substitute different bytes under same digest.

---

# 59. CAS GC and Backup

GC must not delete object still required by:

```text
backup retention
release
rollback
legal retention
```

---

# 60. Backup Root

Backup manifest can temporarily pin objects.

---

# 61. Release Root

Released artifacts strongly retained.

---

# 62. Restore CAS First or Metadata First

Depends incident.

Recommended restoration workflow handles both references safely.

---

# 63. Metadata Ref to Missing CAS

Allowed temporarily during restore, but system enters degraded/read-only for affected operations.

---

# 64. Restore Verification

After restore:

```text
metadata integrity
CAS existence sampling/full critical check
schema version
trust integrity
coordination consistency
```

---

# 65. Secret Backup

Do not merely dump plaintext secrets.

---

# 66. External Secret Provider

Backup:

```text
references/config/policies
```

provider itself handles secret durability.

---

# 67. Local Encrypted Secret Store

Backup encrypted ciphertext + wrapped keys.

---

# 68. KEK Recovery

Separate procedure.

---

# 69. HSM/KMS Keys

Backup/recovery follows provider key policy.

---

# 70. Signing Keys

Prefer non-exportable.

Recovery means:

```text
restore provider/KMS access
```

not restore private key file.

---

# 71. CA Recovery

Critical.

---

# 72. Root CA

Offline backup in secure independent storage.

---

# 73. Intermediate CA

Recoverable/rotatable.

---

# 74. CA Loss

Can require re-enrollment/reissue.

---

# 75. Trust Epoch

Recovery may advance TrustEpoch after compromise.

---

# 76. Compromised Key Incident

Different from simple loss.

---

# 77. Key Loss

Restore/reissue.

---

# 78. Key Compromise

Revoke/rotate/re-evaluate historical trust.

---

# 79. Secret Backup Access

Strongly restricted/audited.

---

# 80. Coordination Backup

Raft state is small.

---

# 81. Normal HA

Surviving quorum is primary recovery.

---

# 82. Coordination Snapshot

Backup latest signed/trusted snapshot metadata.

---

# 83. Total Coordination Loss

Explicit DR procedure.

---

# 84. Rebuild Coordination

Because business truth remains PostgreSQL/CAS.

---

# 85. Recovery Steps

```text
verify DB/CAS
initialize/recover ClusterId
reconstruct membership
bump coordination epochs
clear/reconcile stale exclusive operations
start reconciliation
```

---

# 86. Never Reuse Stale Epoch

Recovery always advances.

---

# 87. Stale Daemon

Cannot rejoin with old authority.

---

# 88. Config Backup

Version-control/config-management recommended.

---

# 89. Runtime Config Snapshot

Backup effective configuration digest.

---

# 90. Policy Backup

Policy bundles immutable/versioned.

---

# 91. Plugin Backup

Installed plugin package digests + config.

---

# 92. External Provider Config

Binding metadata/backups.

Credentials remain secret-provider-backed.

---

# 93. Upgrade Strategy

Default:

```text
expand
roll out compatible binaries
contract later
```

---

# 94. Expand Migration

Add new columns/tables/indexes compatible with old nodes.

---

# 95. Binary Rollout

Old + new versions coexist.

---

# 96. Contract Migration

Remove old schema only after all old binaries gone.

---

# 97. Migration Version

```rust
pub struct SchemaVersion(u64);
```

---

# 98. Migration Record

```rust
pub struct MigrationRecord {
    pub version: SchemaVersion,
    pub state: MigrationState,
    pub applied_at: Option<Timestamp>,
}
```

---

# 99. Migration State

```text
Pending
Applying
Applied
Failed
RolledBack
```

---

# 100. Migration Lock

Only one schema migration coordinator.

---

# 101. Migration Leader

Coordination/DB lock.

---

# 102. Migration Idempotency

Required.

---

# 103. Migration Precheck

Verify:

```text
DB version
free space
backup freshness
binary compatibility
```

---

# 104. Backup Before Risky Migration

Policy requirement.

---

# 105. Online Migration

Prefer.

---

# 106. Long Backfill

Separate resumable background task.

---

# 107. No Massive Transaction

Backfill in batches.

---

# 108. Backfill Checkpoint

Persist progress.

---

# 109. Backfill Reconcile

Restart-safe.

---

# 110. Index Creation

Use online/concurrent mechanisms where available.

---

# 111. Migration Timeout

Do not hold global lock indefinitely.

---

# 112. Failed Migration

Enter safe mode/readiness false if schema ambiguous.

---

# 113. Rollback Migration

Only if explicitly safe.

---

# 114. Forward Fix

Preferred for many production migrations.

---

# 115. Binary Rollback

Only while schema remains backward-compatible.

---

# 116. Downgrade Gate

Forgeyard binary checks minimum/maximum schema compatibility.

---

# 117. No Unsafe Downgrade

Refuse startup if schema too new.

---

# 118. Version Compatibility Matrix

```text
binary version
API version
protocol version
DB schema range
Raft state schema
plugin API
```

---

# 119. Upgrade Planner

```rust
pub struct UpgradePlan {
    pub from: ForgeyardVersion,
    pub to: ForgeyardVersion,
    pub migrations: Vec<SchemaMigrationId>,
    pub compatibility: UpgradeCompatibility,
}
```

---

# 120. Upgrade Compatibility

```rust
pub enum UpgradeCompatibility {
    Rolling,
    MaintenanceWindow,
    OfflineRequired,
}
```

---

# 121. Rolling Upgrade

Preferred.

---

# 122. Maintenance Window

Only for incompatible transitions.

---

# 123. Offline Required

Exceptional.

---

# 124. Upgrade Sequence

Recommended distributed:

```text
1. verify backup
2. run preflight doctor
3. apply expand migrations
4. upgrade follower/API nodes
5. validate
6. transfer leadership
7. upgrade old leader
8. verify agents/protocol compatibility
9. activate new feature version
10. contract later
```

---

# 125. Agent Upgrade

Agents can be upgraded independently within protocol compatibility matrix.

---

# 126. Runner Drain

Before upgrade:

```text
drain
finish/cancel according to policy
upgrade
re-enroll/reconnect
```

---

# 127. Signing Worker Upgrade

High-assurance separate rollout.

---

# 128. Device Agent Upgrade

Drain device sessions first.

---

# 129. UI Upgrade

API compatibility preserves use during rolling control-plane upgrade.

---

# 130. Plugin Upgrade

Part 24 semantics.

---

# 131. Backup Upgrade Compatibility

Restore old backup into supported recovery binary/toolchain if necessary.

---

# 132. Recovery Binary

Keep documented compatibility for recent major versions.

---

# 133. Backup Manifest Version

Explicit.

---

# 134. Backup Format Evolution

Reader supports previous versions.

---

# 135. Restore Tool Version

May be newer than backup.

---

# 136. Restore Validation

Never import blindly.

---

# 137. Maintenance Mode

Global:

```text
Normal
ReadOnly
NoScheduling
NoReleasePromotion
NoDeployment
FullMaintenance
```

---

# 138. Maintenance Use

Examples:

```text
schema migration
storage repair
DR testing
trust recovery
```

---

# 139. Maintenance Entry

Audited.

---

# 140. Maintenance Exit

Only after health/doctor passes.

---

# 141. Read-Only Recovery Mode

Useful when metadata readable but writes unsafe.

---

# 142. User Experience

UI/API displays explicit impact.

---

# 143. Disk Pressure

Critical operational condition.

---

# 144. Disk Watermarks

```rust
pub struct DiskWatermarks {
    pub warn: Percent,
    pub critical: Percent,
    pub emergency: Percent,
}
```

---

# 145. Warn

Alert.

---

# 146. Critical

Throttle nonessential writes/cache.

---

# 147. Emergency

Stop new work requiring local storage.

---

# 148. Never Delete Critical Metadata Randomly

Controlled GC only.

---

# 149. Telemetry First to Drop

Noncritical telemetry yields before CAS/business data.

---

# 150. CAS Cache Eviction

Ephemeral cache first.

---

# 151. Release Artifacts Protected

Do not evict release-critical roots.

---

# 152. Capacity Planning

Track:

```text
DB growth
CAS growth
job concurrency
runner capacity
event backlog
backup window
restore bandwidth
```

---

# 153. Capacity Forecast

Operational advisory.

---

# 154. Storage Growth Model

CAS mostly append + GC.

---

# 155. Backup Window

Measure actual duration.

---

# 156. Restore Window

Measure actual duration.

---

# 157. Backup Success Metric

Not enough.

---

# 158. Restore Success Metric

Essential.

---

# 159. Automated Restore Test

Periodically restore into isolated environment.

---

# 160. Restore Drill

Validate:

```text
DB
CAS
config
trust refs
startup
sample run history
release verification
```

---

# 161. DR Drill Frequency

Configurable.

Recommended periodic.

---

# 162. Disaster Scenarios

At minimum rehearse:

```text
DB accidental delete
DB region loss
CAS bucket loss
single CAS object corruption
all daemon nodes lost
Raft quorum loss
secret provider unavailable
signing key revoked
bad software upgrade
bad migration
```

---

# 163. Scenario: Bad DB Migration

Sequence:

```text
stop/limit writes
assess compatibility
restore PITR or forward-fix
verify schema
reconcile
```

---

# 164. Scenario: Operator Deletes Release

If metadata soft-delete/audit retained:

restore/recover row state.

If hard deletion occurred:

PITR.

---

# 165. Scenario: CAS Object Corruption

```text
detect digest mismatch
quarantine
fetch replica
verify
restore
```

---

# 166. Scenario: CAS Region Loss

Fail over to replicated store.

---

# 167. Scenario: DB Region Loss

Promote/restore database, update endpoints, restart/reconnect control plane.

---

# 168. Scenario: Total Control Plane Loss

New daemon fleet connects to restored/healthy DB+CAS, reconstructs coordination, reconciles.

---

# 169. Scenario: Secret Provider Loss

Operations requiring secrets blocked.

Non-secret reads/jobs may continue.

---

# 170. Scenario: Signing Key Compromise

```text
disable signing
revoke trust
rotate key
evaluate previously signed artifacts
update TrustEpoch
```

---

# 171. Scenario: Broken Binary Upgrade

If schema backward-compatible:

```text
roll back binary
```

---

# 172. Broken Upgrade With Forward Schema

Use compatible previous binary only if supported.

Otherwise forward fix.

---

# 173. Canary Control-Plane Upgrade

Upgrade one follower first.

---

# 174. Health Gate

Require:

```text
API
DB
scheduler
events
agent reconnect
```

before continuing.

---

# 175. Backup Retention

Example tiers:

```text
hourly: 24-48h
daily: 30d
weekly: 12w
monthly: 12m
```

Organization-specific.

---

# 176. Immutable Backup

Prefer object-lock/WORM for critical backup.

---

# 177. Ransomware Resilience

At least one backup credential/storage independent from production.

---

# 178. Offline Backup

Optional high-assurance tier.

---

# 179. Backup Credential

Separate SecretRef/role.

---

# 180. Backup Writer

Cannot delete prior backups ideally.

---

# 181. Backup Delete Permission

Separate role.

---

# 182. Backup Integrity

Digest manifests.

---

# 183. Backup Authenticity

Signature/MAC.

---

# 184. Backup Catalog

```rust
pub struct BackupCatalogEntry {
    pub id: BackupSetId,
    pub created_at: Timestamp,
    pub status: BackupStatus,
    pub restore_verified: bool,
}
```

---

# 185. Backup Status

```text
Creating
Complete
Failed
Corrupt
Expired
```

---

# 186. Restore Verification State

Separate.

---

# 187. Backup Event Model

```text
BackupStarted
BackupCompleted
BackupFailed
RestoreTestStarted
RestoreTestPassed
RestoreTestFailed
```

---

# 188. Upgrade Events

```text
UpgradePlanned
MigrationStarted
MigrationApplied
NodeUpgradeStarted
NodeUpgradeCompleted
UpgradeCompleted
```

---

# 189. DR Events

```text
DisasterDeclared
RecoveryStarted
RecoveryCheckpoint
RecoveryCompleted
```

---

# 190. Recovery Audit

Strong audit.

---

# 191. Runbooks

Version-controlled operational procedures.

---

# 192. Runbook Structure

Each includes:

```text
symptoms
impact
preconditions
diagnostics
safe actions
rollback
verification
escalation
```

---

# 193. Machine-Assisted Runbooks

Doctor can suggest exact runbook.

---

# 194. RunbookId

```rust
pub struct RunbookId(BoundedString);
```

---

# 195. No Automatic Destructive Runbook

Operator approval required.

---

# 196. Operations CLI

```text
forgeyard backup create
forgeyard backup list
forgeyard backup verify
forgeyard restore plan
forgeyard restore execute
forgeyard upgrade plan
forgeyard upgrade preflight
forgeyard maintenance enter
forgeyard maintenance exit
forgeyard dr status
forgeyard dr drill
forgeyard storage scrub
```

---

# 197. `backup create`

Creates coordinated backup set/manifest.

---

# 198. `backup verify`

Checks integrity without restore.

---

# 199. `restore plan`

Explains:

```text
target
RPO point
schema compatibility
required CAS
secret/trust requirements
```

---

# 200. `restore execute`

High-risk, audited.

---

# 201. Restore Target

Never overwrite active production accidentally.

---

# 202. Restore Environment

Explicit target cluster/environment.

---

# 203. Production Restore

Requires maintenance/break-glass as policy.

---

# 204. Upgrade Preflight

Checks:

```text
backup freshness
DB schema
disk
cluster health
protocol compatibility
plugin compatibility
```

---

# 205. Plugin Compatibility

Block upgrade if required plugin incompatible.

---

# 206. Agent Compatibility

Warn if too-old agents.

---

# 207. CAS Format Compatibility

CAS object format should be backward-compatible/versioned.

---

# 208. API Compatibility

Part 18.

---

# 209. Raft Compatibility

Part 22.

---

# 210. Supply Chain Verification

Forgeyard upgrade binaries/packages themselves signed/verified.

---

# 211. Self-Upgrade Trust

Never install unsigned/unverified control-plane binary in production.

---

# 212. Upgrade Artifact

Exact ReleaseId/package digest.

---

# 213. No Mutable `latest`

Resolve exact upgrade release.

---

# 214. Rollout Artifact

Same exact bytes across cluster nodes.

---

# 215. Node-Specific Package

OS/arch-specific exact release package.

---

# 216. Bootstrap Recovery

Fresh server can install known Forgeyard release then attach restored data.

---

# 217. Air-Gapped Backup

Can export complete recovery bundle.

---

# 218. Air-Gap Recovery Bundle

Includes:

```text
backup manifest
metadata backup
required CAS objects
config
public trust material
encrypted secret-store backup if applicable
restore instructions/version metadata
```

---

# 219. Air-Gap Integrity

Signed manifest.

---

# 220. No Private Key in Plaintext Bundle

Critical.

---

# 221. Restore Ordering

Suggested:

```text
1. provision clean hosts
2. restore trust/secret provider access
3. restore PostgreSQL
4. restore/attach CAS
5. restore config
6. initialize coordination
7. start Forgeyard in maintenance/read-only
8. verify
9. reconcile
10. exit maintenance
```

---

# 222. Reconciliation After Restore

Required.

---

# 223. Why

External/provider state may have advanced since backup.

---

# 224. SCM Reconcile

Fetch provider current state.

---

# 225. Release Reconcile

Inspect publication destinations.

---

# 226. Deployment Reconcile

Inspect actual provider state.

---

# 227. Runner Reconcile

Agents reconnect/new sessions.

---

# 228. Lease Recovery

Expired/stale leases fenced.

---

# 229. Timers

Rebuild due timers from DB.

---

# 230. Event Outbox

Republish pending outbox safely.

---

# 231. Inbox

Prevents duplicate side effects.

---

# 232. Backup of Outbox

Yes as normal DB state.

---

# 233. Restore Duplicate Risk

At-least-once semantics handle.

---

# 234. External Unknown Effects

Inspect before retry.

---

# 235. Backup Consistency Across DB/CAS

Perfect global snapshot may not be practical.

---

# 236. Design Advantage

CAS immutable + metadata refs allow asynchronous backup.

---

# 237. Required Rule

DB-referenced release-critical CAS objects must be recoverable by retention/replication.

---

# 238. Backup Watermark

Backup manifest records:

```text
DB point
CAS replication watermark
```

---

# 239. CAS New Object Race

If DB backup references newly created object not yet backup-replicated:

backup not Complete until object durable.

---

# 240. Backup Coordinator

Tracks required CAS closure for critical state.

---

# 241. Historical Noncritical CAS

Can follow eventual backup policy.

---

# 242. Backup Completeness Class

```rust
pub enum BackupCompleteness {
    MetadataOnly,
    CriticalClosure,
    FullConfigured,
}
```

---

# 243. Production DR Backup

Require at least `CriticalClosure`.

---

# 244. Release Critical Closure

Includes:

```text
released package bytes
signatures
SBOM/provenance
release manifests
rollback releases
```

---

# 245. Current Deployment Closure

Include current + previous healthy artifacts.

---

# 246. Metadata Integrity Check

Foreign keys/constraints.

---

# 247. Domain Invariant Check

Run specialized validators after restore.

---

# 248. Examples

```text
terminal Job has terminal Attempt semantics
Released release references existing package metadata
active lease epoch valid
```

---

# 249. Restore Doctor

```text
forgeyard doctor --restore
```

---

# 250. Restore Doctor Checks

```text
schema
CAS closure
trust
coordination
events
external provider connectivity
```

---

# 251. Restore Certification

Only after doctor passes can production writes resume.

---

# 252. Backup Health

Show:

```text
last successful backup
last verified restore
RPO age
backup lag
```

---

# 253. Alerting

Alert on:

```text
backup failed
restore test failed
RPO exceeded
CAS replica lag
PITR unavailable
backup encryption key unavailable
```

---

# 254. Metrics

```text
backup_duration
backup_bytes
backup_failures
backup_rpo_seconds
restore_test_duration
restore_test_failures
cas_replication_lag
migration_duration
upgrade_failures
```

---

# 255. No Backup ID Metric Label

Use logs/traces.

---

# 256. Tracing

```text
backup.create
backup.verify
restore.plan
restore.execute
upgrade.preflight
migration.apply
dr.drill
```

---

# 257. UI

Admin operations pages:

```text
Backups
Restore
Upgrades
Migrations
DR
Capacity
Maintenance
Storage Health
```

---

# 258. Backup Page

Shows:

```text
created
RPO point
size
completeness
encrypted
restore verified
```

---

# 259. Restore Page

Dangerous action.

Show exact backup ID/timestamp/target.

---

# 260. Upgrade Page

Shows:

```text
current version
target version
compatibility
migration steps
plugin compatibility
agent compatibility
```

---

# 261. DR Dashboard

Shows:

```text
RPO
RTO target
last drill
replication state
backup health
```

---

# 262. Maintenance Banner

Global.

---

# 263. Capacity Dashboard

Shows trends.

---

# 264. Corruption Dashboard

CAS scrub findings.

---

# 265. Authorization Permissions

```text
backup.read
backup.create
backup.verify
restore.plan
restore.execute
upgrade.plan
upgrade.execute
maintenance.manage
dr.manage
```

---

# 266. Restore Execute

Highest-risk.

---

# 267. Step-Up Authentication

Recommended.

---

# 268. Break Glass

May be needed if IdP unavailable.

---

# 269. Offline Recovery Identity

Secure emergency recovery principal/material.

---

# 270. Emergency Access Storage

Independent/offline protected.

---

# 271. Emergency Credentials

Short procedure; rotate after use.

---

# 272. Recovery Without IdP

Must be possible in catastrophic outage.

---

# 273. But Not Hidden Backdoor

Explicit break-glass identity with audit.

---

# 274. Backup Access Separation

Operators who run backups may not necessarily restore.

---

# 275. Separation of Duties

Useful high-assurance environment.

---

# 276. Standalone Recovery

Simpler:

```text
backup directory
  ↓
verify
  ↓
restore Stoolap + CAS + config
  ↓
start app
```

---

# 277. Standalone Upgrade

Before upgrade:

```text
automatic backup
verify
install new binary
run migration
```

---

# 278. Local Rollback

If schema compatible.

---

# 279. Local Safe Mode

If migration fails:

```text
read-only/restore flow
```

---

# 280. Distributed Upgrade

HA rolling.

---

# 281. Region DR

Recommended baseline:

```text
database DR
CAS replica
config/secrets availability
fresh daemon fleet
```

---

# 282. Active-Active

Not baseline.

---

# 283. Active-Passive DR

Simpler/safer initially.

---

# 284. DR Region Activation

Manual/policy-controlled.

---

# 285. DNS/LB Failover

External infra.

Runbook includes.

---

# 286. Duplicate Active Region Risk

Use coordination/DB authority to prevent two active writers.

---

# 287. Fencing Old Region

Before DR activation if possible.

---

# 288. Split-Region Recovery

Strong operator procedure.

---

# 289. Database Authority

Only one writable authoritative primary unless architecture specifically supports multi-primary.

---

# 290. CAS Can Be Multi-Replica

Immutable.

---

# 291. DR Validation

Check exact release artifacts.

---

# 292. Self-Hosting

Part 26 will build Forgeyard's own release/bootstrap pipeline on top.

---

# 293. Operations Testkit

```text
forgeyard-recovery-testkit/src/
├── lib.rs
├── backup.rs
├── restore.rs
├── migration.rs
├── upgrade.rs
├── corruption.rs
├── dr.rs
└── assertions.rs
```

---

# 294. Backup Unit Tests

Manifest, retention, encryption metadata.

---

# 295. Restore Integration Test

Restore real test DB/CAS.

---

# 296. PITR Test

Write data, corrupt/delete, restore before event.

---

# 297. CAS Corruption Test

Flip bytes -> detect -> repair from replica.

---

# 298. Missing CAS Test

Metadata remains, repair marks restored.

---

# 299. Secret Restore Test

Encrypted secret-store backup recoverable.

---

# 300. Lost KEK Test

Recovery fails safely.

---

# 301. Raft Loss Test

Rebuild coordination from restored DB/CAS.

---

# 302. Migration Crash Test

Process dies mid-backfill -> resumes.

---

# 303. Rolling Upgrade Test

N/N-1 mixed cluster.

---

# 304. Downgrade Test

Refuse unsafe binary/schema combination.

---

# 305. Backup During Load Test

No unacceptable application pause.

---

# 306. Restore Scale Test

Measure RTO.

---

# 307. Region Loss Simulation

Fresh control plane from backups/replicas.

---

# 308. External Reconcile Test

Provider state advanced after backup; restore does not blindly replay side effect.

---

# 309. Restore Duplicate Outbox Test

Inbox/idempotency prevents duplicate harmful action.

---

# 310. Disk Full Test

System enters safe degraded mode.

---

# 311. Telemetry Disk Test

Telemetry yields before critical data.

---

# 312. Backup Storage Credential Compromise Test

Rotate without production DB outage.

---

# 313. Ransomware Scenario

Production delete credential cannot delete immutable/offline backup.

---

# 314. Air-Gap Restore Test

Complete recovery without internet.

---

# 315. Fuzzing

Fuzz:

```text
backup manifest parser
restore plan parser
migration metadata
```

---

# 316. Failure Injection

```text
backup upload interruption
DB restore interruption
CAS replica unavailable
secret provider outage
migration lock loss
node crash
```

---

# 317. Implementation Phase 1 — Backup Model/Catalog

Implement manifests, policies, health.

---

# 318. Phase 2 — PostgreSQL/Neon Backup Integration

PITR/snapshot metadata + restore tooling.

---

# 319. Phase 3 — Standalone Backup

Stoolap + local CAS.

---

# 320. Phase 4 — CAS Replication/Scrub

Critical durability.

---

# 321. Phase 5 — Restore Verification

Doctor/invariant checks.

---

# 322. Phase 6 — Upgrade Planner/Migrations

Expand-contract.

---

# 323. Phase 7 — HA Rolling Upgrade

Coordination integration.

---

# 324. Phase 8 — Secret/Trust Recovery

Key/CA procedures.

---

# 325. Phase 9 — DR Automation

Region/control-plane reconstruction.

---

# 326. Phase 10 — Automated Restore Drills

Periodic.

---

# 327. Phase 11 — Air-Gap Recovery

High-assurance.

---

# 328. Phase 12 — Chaos/Scale Hardening

Production validation.

---

# 329. Acceptance Tests

1. Business metadata and CAS are backed up separately.
2. Coordination state is not treated as business-data backup.
3. Secret/trust material has separate protected recovery.
4. PostgreSQL production mode has PITR capability.
5. Neon recovery path is documented/tested when used.
6. Standalone Stoolap backup is consistent.
7. CAS backup is incremental by immutable digest.
8. CAS destination verifies digest.
9. Corrupt CAS object is quarantined.
10. Corrupt object can be repaired from healthy replica.
11. Different bytes are never substituted under same digest.
12. Released artifacts are protected retention roots.
13. Backup manifest is integrity protected.
14. Backup encryption key is separated from backup storage.
15. Restore tool validates Forgeyard/schema compatibility.
16. A backup is not marked restore-verified until isolated restore passes.
17. Backup RPO age is observable/alertable.
18. Restore RTO is measured through drills.
19. Production schema changes use expand-contract where possible.
20. Old/new daemon versions can coexist during rolling upgrade.
21. Unsafe downgrade is refused.
22. Long backfills are resumable.
23. Risky migration requires recent verified backup by policy.
24. Disk pressure degrades safely rather than corrupting data.
25. Telemetry is sacrificed before critical metadata/artifacts.
26. Total Raft loss can recover without losing PostgreSQL/CAS business truth.
27. Restore advances/fences stale coordination epochs.
28. External provider state is reconciled after restore.
29. Restored outbox does not cause blind duplicate side effects.
30. Signing key compromise uses revoke/rotate rather than simple restore.
31. Catastrophic IdP outage has explicit audited break-glass recovery.
32. Air-gapped recovery works from signed bundle.
33. Region-loss drill can restore control-plane service.
34. Standalone/distributed both have tested recovery flows.
35. Forgeyard's own production deployment runs regular restore drills.

---

# 330. Production Readiness Gates

Do not call operations/DR production-ready until:

```text
metadata backup/PITR configured
CAS critical replication configured
restore verification automated
RPO/RTO targets defined
rolling upgrade path tested
migration rollback/forward-fix policy defined
secret/trust recovery documented
Raft total-loss recovery exercised
air-gap or independent backup tier exists where required
disaster drills are scheduled and passing
```

---

# 331. Architectural Invariants

1. backup without tested restore is insufficient;
2. metadata/CAS/coordination/secrets are separate recovery domains;
3. PostgreSQL/Neon is recovered as business-data authority;
4. CAS recovery is digest-verified;
5. coordination can be reconstructed/fenced from authoritative state;
6. secrets are never backed up as casual plaintext;
7. backup encryption keys are separately protected;
8. released artifacts are durable retention roots;
9. CAS corruption never rewrites digest identity;
10. PITR is preferred for distributed metadata recovery;
11. standalone backup remains server-free;
12. upgrade uses exact signed Forgeyard release artifacts;
13. rolling upgrade is preferred;
14. expand-contract is default schema strategy;
15. long migrations/backfills are resumable;
16. unsafe downgrade is refused;
17. maintenance mode is explicit;
18. disk pressure has staged safe behavior;
19. restore always runs domain invariant checks;
20. restored external side effects are reconciled, not replayed blindly;
21. restore advances/fences stale coordination authority;
22. break-glass recovery is explicit and audited;
23. DR region activation prevents duplicate writers;
24. immutable backup/offline copy protects from ransomware/operator deletion;
25. restore drills measure real RTO;
26. backup health exposes real RPO lag;
27. air-gapped recovery is possible for high-assurance deployments;
28. standalone/distributed share recovery principles;
29. operational runbooks are versioned/tested;
30. Forgeyard dogfoods its own upgrade and disaster-recovery system.

---

# 332. Final Target Architecture

```text
                    Forgeyard State
                         │
        ┌────────────────┼─────────────────┐
        ▼                ▼                 ▼
   PostgreSQL/Neon      CAS          Secrets/Trust
        │                │                 │
        ▼                ▼                 ▼
   PITR/Snapshot      Replica/Backup     Secure Backup/
        │                │              Provider Recovery
        └────────────────┼─────────────────┘
                         ▼
                    Backup Manifest
                         │
                         ▼
                   Restore Verification
                         │
                         ▼
                 Isolated Restore Drill
                         │
                         ▼
                      DR Ready
```

Upgrade path:

```text
verified backup
  ↓
preflight
  ↓
expand migration
  ↓
rolling binary upgrade
  ↓
health validation
  ↓
feature activation
  ↓
contract migration later
```

Disaster recovery:

```text
declare incident
  ↓
restore/activate metadata authority
  ↓
restore/attach CAS
  ↓
restore secret/trust access
  ↓
rebuild/fence coordination
  ↓
start maintenance/read-only
  ↓
verify
  ↓
reconcile external state
  ↓
resume production
```

The key guarantee is:

> **Forgeyard is designed so that a catastrophic control-plane loss does not imply loss of business truth or release artifacts. Metadata can be restored through PostgreSQL/Neon recovery, immutable bytes can be recovered by digest from CAS replicas/backups, coordination can be rebuilt and fenced, and production only resumes after a verified restore and reconciliation pass.**

---

# 333. New-Repository Sequence

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
