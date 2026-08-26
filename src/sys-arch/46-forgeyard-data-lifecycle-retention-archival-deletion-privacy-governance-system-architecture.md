# 46 — Forgeyard Data Lifecycle, Retention, Archival, Deletion, Legal Hold & Privacy Governance System Architecture

**Document type:** Core Data Lifecycle, Retention, Archival, Deletion, Legal Hold, Privacy & Data-Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** metadata retention, CAS lifecycle, logs, test/benchmark artifacts, findings, audit, release evidence, deployment evidence, cache lifecycle, backups, tenant data, personal data, legal hold, deletion, tombstones, archival, data export, privacy operations, retention policy, evidence preservation, cryptographic erasure, and restoration constraints  
**Architecture style:** Explicit data classes, policy-driven lifecycle, immutable evidence where required, deletion by governed intent, retention provenance, legal-hold precedence, tenant isolation, privacy-aware minimization, and no silent destruction of authoritative or security-critical evidence  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Storage/Metadata, CAS, Audit/Compliance, Security, Operations/Backup/DR, Multi-Tenancy, Search/Analytics, Observability, Tests, Benchmarks, Findings, Release, Deployment, Cost/FinOps, and Configuration Governance. This subsystem centralizes how Forgeyard decides what data exists, for how long, why it exists, and how it is deleted or preserved.

---

# 1. Purpose

Forgeyard can accumulate large volumes of data:

```text
runs
jobs
attempts
logs
artifacts
source snapshots
test reports
coverage
benchmarks
SBOMs
provenance
audit records
security events
release evidence
deployment history
cache objects
provider metadata
cost facts
tenant configuration
personal identifiers
```

Without a dedicated lifecycle architecture, systems tend to drift into one of two bad extremes:

```text
keep everything forever
```

or:

```text
delete aggressively without knowing what was still required
```

The central rule is:

> **Every retained object must have an explicit lifecycle class and retention reason. Every deletion must be a governed state transition, not an ad-hoc filesystem/database cleanup.**

A second rule is:

> **Legal hold, active security investigation, release reproducibility, and authoritative integrity requirements take precedence over ordinary retention expiry.**

A third rule is:

> **Privacy requests and tenant deletion must be fulfilled as far as legally and technically permissible without falsifying immutable historical evidence. Where data cannot be erased, Forgeyard minimizes, pseudonymizes, restricts, or tombstones references rather than rewriting history.**

---

# 2. Architectural Position

```text
                      Data Created
                          │
                          ▼
                   Data Classification
                          │
                          ▼
                    Retention Policy
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Retain        Archive       Expire
             │            │            │
             │            │            ▼
             │            │        Delete Intent
             │            │            │
             │            │      ┌─────┼─────┐
             │            │      ▼           ▼
             │            │   Hold Check   Delete
             │            │      │           │
             │            │      ▼           ▼
             └────────────┴── Preserve     Tombstone
```

---

# 3. Goals

The subsystem MUST:

1. classify Forgeyard data;
2. define retention classes;
3. support configurable retention;
4. support legal holds;
5. support security holds;
6. support archive transitions;
7. support metadata deletion;
8. support CAS deletion;
9. support tombstones;
10. support tenant deletion;
11. support project deletion;
12. support user/privacy deletion;
13. support data export;
14. support personal-data minimization;
15. support pseudonymization;
16. support cryptographic erasure where appropriate;
17. support backup lifecycle;
18. support restore-aware deletion;
19. support immutable audit requirements;
20. support release reproducibility requirements;
21. support cost-aware retention inputs;
22. support retention previews;
23. support dry-run deletion plans;
24. support policy exceptions;
25. support tenant isolation;
26. support audit;
27. support notifications;
28. support API/UI/CLI;
29. support standalone/distributed modes;
30. remain recoverable and explainable.

---

# 4. Non-Goals

This subsystem does not:

```text
replace external records-management systems
replace jurisdiction-specific legal counsel
rewrite immutable release history
guarantee deletion from third-party systems outside Forgeyard control
replace backup/DR architecture
```

---

# 5. Workspace Structure

```text
crates/lifecycle/
├── forgeyard-lifecycle/
├── forgeyard-lifecycle-model/
├── forgeyard-lifecycle-classify/
├── forgeyard-lifecycle-retention/
├── forgeyard-lifecycle-archive/
├── forgeyard-lifecycle-delete/
├── forgeyard-lifecycle-hold/
├── forgeyard-lifecycle-privacy/
├── forgeyard-lifecycle-export/
├── forgeyard-lifecycle-reconcile/
├── forgeyard-lifecycle-health/
└── forgeyard-lifecycle-testkit/
```

Use modules first; split only where real data/security/runtime boundaries justify.

---

# 6. DataClassId

```rust
pub struct DataClassId(BoundedString);
```

---

# 7. Core Data Classes

```rust
pub enum DataClass {
    OperationalMetadata,
    SourceSnapshot,
    BuildArtifact,
    ReleaseArtifact,
    DeploymentEvidence,
    TestEvidence,
    BenchmarkEvidence,
    StaticAnalysisEvidence,
    SecurityEvidence,
    AuditEvidence,
    LogData,
    CacheData,
    CostData,
    ConfigurationData,
    IdentityData,
    SecretMetadata,
    PersonalData,
    BackupData,
    Custom(DataClassId),
}
```

---

# 8. LifecycleClass

```rust
pub enum LifecycleClass {
    Ephemeral,
    ShortLived,
    Operational,
    LongLivedEvidence,
    ReleaseCritical,
    SecurityCritical,
    LegallyHeld,
    PermanentByPolicy,
}
```

---

# 9. Lifecycle Object

```rust
pub struct LifecycleObjectRef {
    pub tenant: TenantId,
    pub resource: ResourceRef,
    pub data_class: DataClass,
    pub lifecycle_class: LifecycleClass,
}
```

---

# 10. Classification

Prefer explicit creation-time classification.

---

# 11. Derived Classification

Can infer from resource type only when unambiguous.

---

# 12. Unknown Classification

Conservative retention.

---

# 13. RetentionPolicyId

```rust
pub struct RetentionPolicyId(Digest);
```

---

# 14. Retention Policy

```rust
pub struct RetentionPolicy {
    pub id: RetentionPolicyId,
    pub scope: ResourceScope,
    pub data_class: DataClass,
    pub duration: RetentionDuration,
    pub archive: ArchivePolicy,
    pub delete: DeletePolicy,
}
```

---

# 15. Retention Duration

```rust
pub enum RetentionDuration {
    Duration(Duration),
    UntilEvent(RetentionEvent),
    Forever,
}
```

---

# 16. Until Event

Examples:

```text
release superseded + 1 year
tenant closed + 90 days
incident closed + 7 years
```

---

# 17. Retention Event

Typed.

---

# 18. No Free-Form Retention Formula

Critical.

---

# 19. Policy Scope

Can be:

```text
system
tenant
organization
project
data class
environment
```

---

# 20. Minimum Retention

Central policy may impose floor.

---

# 21. Maximum Retention

Privacy/cost policy may impose ceiling where legally appropriate.

---

# 22. Lower Scope

Cannot shorten below mandatory floor.

---

# 23. Retention Resolution

```text
system baseline
+
compliance floor
+
tenant/project policy
+
holds
  ↓
effective retention
```

---

# 24. Holds Always Considered Last

Critical.

---

# 25. LegalHoldId

```rust
pub struct LegalHoldId(Ulid);
```

---

# 26. Hold Type

```rust
pub enum HoldType {
    Legal,
    SecurityIncident,
    Regulatory,
    InternalInvestigation,
    ManualAdministrative,
}
```

---

# 27. Hold Scope

```rust
pub enum HoldScope {
    Tenant(TenantId),
    Project(ProjectId),
    Resource(ResourceRef),
    TimeRange(TimeRange),
    Principal(PrincipalId),
    Incident(IncidentId),
}
```

---

# 28. Hold Record

```rust
pub struct DataHold {
    pub id: LegalHoldId,
    pub kind: HoldType,
    pub scope: HoldScope,
    pub reason: BoundedString,
    pub created_by: PrincipalId,
    pub created_at: Timestamp,
    pub released_at: Option<Timestamp>,
}
```

---

# 29. Hold Is Immutable History

Release creates separate state transition.

---

# 30. Hold Precedence

Deletion blocked while applicable hold active.

---

# 31. Hold Does Not Duplicate Data

It changes lifecycle eligibility.

---

# 32. Retention State

```rust
pub enum LifecycleState {
    Active,
    Archived,
    Expired,
    DeletionPlanned,
    Held,
    Deleting,
    Tombstoned,
    Deleted,
    DeletionFailed,
}
```

---

# 33. Expired

Eligible for lifecycle action.

Not yet deleted.

---

# 34. Deletion Intent

Persist before deletion.

---

# 35. DeletionIntentId

```rust
pub struct DeletionIntentId(Ulid);
```

---

# 36. Deletion Intent

```rust
pub struct DeletionIntent {
    pub id: DeletionIntentId,
    pub resource: LifecycleObjectRef,
    pub reason: DeletionReason,
    pub policy: RetentionPolicyId,
    pub created_at: Timestamp,
}
```

---

# 37. Correct Deletion Flow

```text
retention says expired
  ↓
create deletion intent
  ↓
resolve holds/dependencies
  ↓
prepare delete plan
  ↓
delete eligible representations
  ↓
verify
  ↓
tombstone metadata if required
```

---

# 38. No Delete-First-Reconcile-Later

Critical for important data.

---

# 39. Delete Plan

```rust
pub struct DeletionPlan {
    pub id: DeletionPlanId,
    pub intent: DeletionIntentId,
    pub targets: Vec<DeletionTarget>,
    pub blockers: Vec<DeletionBlocker>,
}
```

---

# 40. Deletion Blockers

Examples:

```text
legal hold
release root
incident evidence
backup hold
referenced by retained artifact
active deployment
```

---

# 41. Dry Run

```text
forgeyard lifecycle plan
```

shows what would be deleted.

---

# 42. Archive

Move cold data to cheaper storage where policy permits.

---

# 43. ArchivePolicy

```rust
pub enum ArchivePolicy {
    None,
    After(Duration),
    Immediate,
}
```

---

# 44. Archive Is Not Delete

Critical.

---

# 45. Archived Data

Still addressable/authorized.

---

# 46. CAS Archive Tier

Part 03 tiered storage.

---

# 47. Metadata Archive

May move to historical tables/object snapshots if necessary.

---

# 48. Restore Latency

Explicit.

---

# 49. ArchiveClass

```rust
pub enum ArchiveClass {
    Warm,
    Cold,
    DeepArchive,
}
```

---

# 50. Release Artifact Lifecycle

Release-critical bytes normally long-lived.

---

# 51. Release Evidence

Retain long enough for:

```text
verification
reproducibility
incident response
customer support
```

---

# 52. Yanking

Does not imply deletion.

---

# 53. Compromised Release

Evidence retained longer, not shorter.

---

# 54. Build Artifact Lifecycle

Non-release build outputs can have shorter retention.

---

# 55. Source Snapshot Lifecycle

If bound to release/evidence, retain.

---

# 56. Unreferenced PR Snapshot

Can expire sooner.

---

# 57. CAS Roots

Lifecycle integrates with Part 03 GC roots.

---

# 58. Root Types

```text
active run
release
legal hold
incident
offline bundle
debug pin
retention pin
```

---

# 59. CAS GC

Only deletes objects with no effective root/reference.

---

# 60. Shared CAS Object

Physical bytes cannot be deleted while any authorized retained reference remains.

---

# 61. Tenant Logical Delete

May remove tenant reference while bytes remain due to another tenant/reference.

---

# 62. Privacy

Physical dedup must not prevent logical erasure of access/reference.

---

# 63. Cache Lifecycle

Part 38 cache is aggressively evictable.

---

# 64. Cache Retention

Performance policy only.

---

# 65. Cache Evidence Link

Original authoritative evidence must not depend solely on cache retention.

---

# 66. Logs

Retention can vary by:

```text
ordinary run
release
security incident
```

---

# 67. Log Redaction

Before persistence when possible.

---

# 68. Log Retention

Security incident can extend via hold.

---

# 69. Audit

Part 28 may require long/WORM retention.

---

# 70. Audit Deletion

Usually exceptional.

---

# 71. Privacy Tension

Audit actor references may need pseudonymization rather than deletion.

---

# 72. Pseudonymization

Replace direct personal attributes while retaining event integrity.

---

# 73. Principal Tombstone

Example:

```text
principal deleted
  ↓
audit retains stable PrincipalId
  ↓
display name/email removed or pseudonymized
```

---

# 74. Historical Truth Preserved

Critical.

---

# 75. IdentityData Lifecycle

User profile/contact data can be deleted separately from immutable event references.

---

# 76. PersonalDataClass

```rust
pub enum PersonalDataKind {
    DisplayName,
    Email,
    IpAddress,
    UserAgent,
    ExternalIdentity,
    Other(PersonalDataKindId),
}
```

---

# 77. Data Minimization

Store only necessary personal data.

---

# 78. IP Retention

Shorter by default unless security need.

---

# 79. User Agent

Likewise.

---

# 80. Search Index

Derived personal data must be deleted/reindexed when source deleted.

---

# 81. Analytics

Derived aggregates should avoid direct personal identifiers.

---

# 82. Privacy Request

```rust
pub struct PrivacyRequest {
    pub id: PrivacyRequestId,
    pub subject: PrivacySubject,
    pub kind: PrivacyRequestKind,
    pub state: PrivacyRequestState,
}
```

---

# 83. Privacy Request Kind

```rust
pub enum PrivacyRequestKind {
    Access,
    Export,
    Correction,
    Deletion,
    Restriction,
}
```

---

# 84. Jurisdiction

External legal policy decides applicability.

---

# 85. Forgeyard Core

Implements technical workflow.

---

# 86. Privacy Request State

```rust
pub enum PrivacyRequestState {
    Received,
    IdentityVerification,
    Evaluating,
    Processing,
    PartiallyFulfilled,
    Fulfilled,
    Rejected,
}
```

---

# 87. Identity Verification

Required before export/delete.

---

# 88. Export

Collect user-associated personal data.

---

# 89. Export Does Not Include

Other tenants/users' secrets/data.

---

# 90. Deletion

Removes eligible personal data.

---

# 91. Immutable Evidence

May retain pseudonymous actor reference if required.

---

# 92. Partial Fulfillment

First-class.

---

# 93. Explanation

Must state retained categories/reason.

---

# 94. Tenant Deletion

High-risk workflow.

---

# 95. TenantDeletionId

```rust
pub struct TenantDeletionId(Ulid);
```

---

# 96. Tenant Deletion States

```rust
pub enum TenantDeletionState {
    Requested,
    CoolingOff,
    Blocked,
    Ready,
    Deleting,
    Verifying,
    Completed,
    Failed,
}
```

---

# 97. Cooling-Off

Optional safety period.

---

# 98. Tenant Delete Preconditions

```text
authorization
billing/support state if relevant
legal hold check
active incident check
export option
```

---

# 99. Tenant Delete Scope

```text
projects
runs
metadata
artifacts
config
private templates
private packages
search indexes
cache refs
cost detail where erasable
```

---

# 100. Security/Audit Evidence

May be retained under mandatory policy.

---

# 101. Tenant Delete Never Deletes Another Tenant's Shared CAS Bytes

Critical.

---

# 102. Tenant Delete Removes Logical References

Then shared bytes remain if referenced elsewhere.

---

# 103. Project Deletion

Similar but narrower.

---

# 104. Soft Delete

Default initial state.

---

# 105. Soft Delete

Removes normal visibility/access while retaining recoverability window.

---

# 106. Hard Delete

After cooling-off and blocker checks.

---

# 107. Tombstone

Minimal record that resource existed/deleted.

---

# 108. Tombstone Contents

```text
resource ID
deletion timestamp
deletion reason class
```

No sensitive content.

---

# 109. Tombstone Purpose

Prevent ID reuse/ghost resurrection.

---

# 110. ID Reuse

Forbidden.

---

# 111. Cryptographic Erasure

Useful when data encrypted by per-tenant/per-object keys.

---

# 112. Key Destruction

Can render ciphertext unrecoverable.

---

# 113. Not Universal

Only if key architecture supports.

---

# 114. Key Erasure Record

Audit key destruction event.

---

# 115. Backups Complicate Deletion

Critical.

---

# 116. Backup Retention

Backups expire according to separate policy.

---

# 117. Delete From Backup

Often impractical to surgically mutate immutable backup.

---

# 118. Correct Model

```text
production deleted now
backup remains inaccessible/retained until expiry
restored backup must reapply tombstones/deletion journal
```

---

# 119. Deletion Journal

```rust
pub struct DeletionJournalEntry {
    pub resource: ResourceRef,
    pub deleted_at: Timestamp,
    pub deletion_epoch: DeletionEpoch,
}
```

---

# 120. Restore

After restoring old backup:

```text
replay deletion journal
  ↓
re-delete logically erased data
```

---

# 121. DeletionEpoch

```rust
pub struct DeletionEpoch(u64);
```

---

# 122. Recovery Must Not Resurrect Deleted Data

Critical.

---

# 123. Backup Hold

Legal/security hold can extend backup retention.

---

# 124. Backup Key Rotation

Part 25.

---

# 125. Archived Backups

Restricted.

---

# 126. Third-Party Data

Examples:

```text
SCM comments/checks
email notifications
cloud logs
external scanners
```

---

# 127. Forgeyard Delete Request

Can delete local copies.

---

# 128. External Deletion

Best-effort via provider API if supported and policy permits.

---

# 129. ProviderLimitation

Explicit.

---

# 130. No False Claim of Full Erasure

Critical.

---

# 131. ExternalDeletionState

```rust
pub enum ExternalDeletionState {
    NotApplicable,
    Deleted,
    Requested,
    Unsupported,
    Failed,
    Unknown,
}
```

---

# 132. Retention Reason

```rust
pub enum RetentionReason {
    Operational,
    Reproducibility,
    Security,
    Legal,
    Contractual,
    UserRequested,
    SystemRequired,
}
```

---

# 133. Object May Have Multiple Reasons

Effective retention = strongest applicable.

---

# 134. Retention Pin

```rust
pub struct RetentionPin {
    pub resource: ResourceRef,
    pub reason: RetentionReason,
    pub expires_at: Option<Timestamp>,
}
```

---

# 135. Pins

Prefer expiring where possible.

---

# 136. Permanent Pin

Requires strong reason.

---

# 137. Release Reproducibility Pin

Can be tied to release support lifetime.

---

# 138. Security Incident Pin

Until incident/hold release.

---

# 139. Cost Integration

Part 45 can estimate cost impact of long retention.

---

# 140. Cost Is Advisory

Cannot delete required evidence.

---

# 141. Retention Simulation

```text
forgeyard lifecycle forecast
```

shows storage/cost impact.

---

# 142. Retention Policy Change

Can affect future eligibility.

---

# 143. Shortening Retention

Does not immediately delete without lifecycle plan/reconciliation.

---

# 144. Lengthening Retention

Stops future deletion if not already deleted.

---

# 145. Cannot Restore Already Deleted Bytes

unless backup/archive still valid.

---

# 146. Policy Change Audit

Mandatory.

---

# 147. Deletion Approval

High-risk data classes may require approval.

---

# 148. Example

Release evidence deletion.

---

# 149. Break-Glass

Cannot bypass legal hold.

---

# 150. Security Admin

Cannot unilaterally release legal hold unless authorized role/policy.

---

# 151. Hold Permissions

```text
lifecycle.hold.read
lifecycle.hold.create
lifecycle.hold.release
```

---

# 152. Delete Permissions

```text
lifecycle.delete.project
lifecycle.delete.tenant
lifecycle.delete.evidence
```

---

# 153. Evidence Delete

Highest privilege.

---

# 154. Privacy Permissions

```text
privacy.request.read
privacy.request.process
privacy.export
privacy.delete
```

---

# 155. Separation of Duties

Legal hold release may require different actor than requester.

---

# 156. API

Potential:

```text
GET  /v1/lifecycle/policies
POST /v1/lifecycle/plan
GET  /v1/lifecycle/holds
POST /v1/lifecycle/holds
POST /v1/lifecycle/holds/{id}/release
POST /v1/projects/{id}/delete
POST /v1/tenants/{id}/delete
GET  /v1/privacy/requests
POST /v1/privacy/requests
```

---

# 157. Dioxus UI

Pages:

```text
Data Lifecycle
Retention Policies
Deletion Plans
Legal Holds
Privacy Requests
Archive
```

---

# 158. Policy Detail

Shows:

```text
data class
retention
archive
delete behavior
scope
minimum floor
```

---

# 159. Deletion Preview

Shows:

```text
objects
bytes
references
blockers
holds
```

---

# 160. No One-Click Irreversible Delete Without Plan

Critical.

---

# 161. Tenant Delete UI

Requires explicit typed confirmation/reauth according to policy.

---

# 162. Privacy UI

Restricted.

---

# 163. CLI

```text
forgeyard lifecycle status
forgeyard lifecycle plan
forgeyard lifecycle run
forgeyard lifecycle hold create
forgeyard lifecycle hold release
forgeyard lifecycle explain <resource>
forgeyard privacy export
forgeyard privacy delete
```

---

# 164. `lifecycle explain`

High value.

Shows:

```text
classification
retention source
holds
pins
references
earliest deletion eligibility
```

---

# 165. Retention Engine

Periodic reconciler.

---

# 166. Lifecycle Reconciler

```text
find eligible objects
  ↓
apply effective retention
  ↓
check holds/references
  ↓
archive/delete
  ↓
verify
```

---

# 167. Bounded Batches

Avoid huge DB/IO spikes.

---

# 168. Backpressure

Pause on backend degradation.

---

# 169. Deletion Retry

Idempotent.

---

# 170. External Delete Unknown

Inspect provider if possible.

---

# 171. Local Delete Unknown

Verify object/reference absence.

---

# 172. CAS Delete Race

Reference check must account for concurrent new root.

---

# 173. Generation/lease

Use GC mark/sweep safety/grace from Part 03.

---

# 174. Grace Period

Avoid deleting just-unreferenced object immediately.

---

# 175. Mark Phase

Compute retained reachable set.

---

# 176. Sweep

Delete unreferenced after grace.

---

# 177. Legal Hold Root

Added to mark roots.

---

# 178. Retention Pin Root

Likewise.

---

# 179. Search Index Lifecycle

Derived indexes purge after source tombstone.

---

# 180. Search Reconciler

No deleted source remains discoverable.

---

# 181. Analytics Lifecycle

Aggregates may remain if non-identifying.

---

# 182. Personal Analytics

Must remove/reaggregate if identifying.

---

# 183. Cost Data

Detailed per-principal cost may be personal/internal.

Aggregate tenant cost can remain.

---

# 184. Notification Data

Messages may be retained separately.

---

# 185. Webhook Deliveries

Short retention unless incident/audit need.

---

# 186. Provider Tokens

Secrets subsystem handles destruction.

---

# 187. Secret Metadata

Refs/history may persist without secret value.

---

# 188. Config Snapshots

Retain history according to operational/security needs.

---

# 189. Plugin Data

Plugin namespace must register lifecycle hooks/classification.

---

# 190. Plugin Cannot Declare "Permanent" Without policy approval.

---

# 191. Plugin Delete Hook

Host-mediated and idempotent.

---

# 192. Plugin Failure

Does not block unrelated lifecycle work.

---

# 193. Device Lab

Device test captures/media can be sensitive.

---

# 194. Device Reset Evidence

Retain operationally.

---

# 195. Screenshots/Recordings

Shorter default retention unless attached to test evidence.

---

# 196. Source Code Privacy

Private source archives highly sensitive.

---

# 197. Release Public Source

May have public retention policy.

---

# 198. Data Sensitivity

Separate from lifecycle class.

---

# 199. DataSensitivity

```rust
pub enum DataSensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
}
```

---

# 200. Lifecycle Policy

Can use both:

```text
class
sensitivity
```

---

# 201. Encryption

At rest according to storage/security profile.

---

# 202. Archive Encryption

Must preserve.

---

# 203. Key Rotation

Archive remains decryptable.

---

# 204. Cryptographic Erasure

Only when explicitly supported.

---

# 205. WORM

Audit/security evidence may use WORM storage.

---

# 206. WORM Retention

Cannot be shortened before lock expiry.

---

# 207. UI Must Explain

Deletion blocked by WORM/hold.

---

# 208. No Pretend Delete

Critical.

---

# 209. Compliance Mapping

Retention controls can map to external control framework.

---

# 210. No Legal Claims

Forgeyard records technical evidence only.

---

# 211. Data Inventory

```rust
pub struct DataInventoryEntry {
    pub data_class: DataClass,
    pub store: StorageLocation,
    pub sensitivity: DataSensitivity,
    pub retention_policy: RetentionPolicyId,
}
```

---

# 212. Inventory

Machine-readable.

---

# 213. Purpose

Know where data lives.

---

# 214. Storage Locations

```text
Postgres/Stoolap
CAS
object storage
local disk
search index
backup
external provider
```

---

# 215. Data Flow Map

Part of architecture/security docs.

---

# 216. Privacy Data Inventory

Tag personal-data-bearing fields.

---

# 217. Schema Metadata

Can annotate:

```text
personal
sensitive
retention class
```

---

# 218. Static Check

Architecture tool can detect new persistent models lacking lifecycle classification.

---

# 219. Critical Rule

No new durable table/object type without lifecycle owner/class.

---

# 220. LifecycleOwner

```rust
pub struct LifecycleOwner(ComponentKind);
```

---

# 221. Ownership

Component responsible for classification/deletion semantics.

---

# 222. Orphan Data

No lifecycle owner = architecture violation.

---

# 223. Backup Restore

Deletion journal applied before serving user traffic when practical.

---

# 224. Restore Safety Gate

System can enter:

```text
RestoredPendingDeletionReplay
```

---

# 225. User Traffic

Protected until critical deletion/tombstone replay complete.

---

# 226. Disaster Recovery

Legal holds restored too.

---

# 227. Hold State

Part of authoritative backup.

---

# 228. Deletion Journal

Independent durable copy recommended.

---

# 229. Security Incident Restore

Incident holds replayed.

---

# 230. Retention Policy Versioning

```rust
pub struct RetentionPolicyVersion(u64);
```

---

# 231. Object Evaluation

Records policy version used.

---

# 232. Deletion Evidence

```rust
pub struct DeletionEvidence {
    pub intent: DeletionIntentId,
    pub plan: DeletionPlanId,
    pub completed_at: Timestamp,
    pub deleted_targets: Vec<DeletionTargetRef>,
    pub retained_blocked: Vec<DeletionBlocker>,
}
```

---

# 233. Deletion Evidence

Contains no deleted sensitive payload.

---

# 234. Audit

Audit:

```text
policy change
hold create/release
tenant/project delete
privacy request
evidence deletion
retention override
```

---

# 235. Routine TTL/cache cleanup

Operational event, not privileged audit each object.

---

# 236. Notification

Examples:

```text
tenant deletion scheduled
hold created
retention policy conflict
deletion blocked
privacy request due
```

---

# 237. SLA/Deadlines

Privacy/legal workflow deadlines can be tracked, but legal values are configured externally.

---

# 238. Search

Lifecycle metadata can be indexed safely.

---

# 239. Analytics

Examples:

```text
bytes by lifecycle class
archive rate
deletion backlog
held bytes
privacy request status
```

---

# 240. Cost

Part 45 can estimate:

```text
retention cost
held-data cost
archive savings
```

---

# 241. Observability Metrics

```text
lifecycle_objects_expired_total
lifecycle_delete_total
lifecycle_delete_failures_total
lifecycle_deletion_backlog
lifecycle_held_objects
lifecycle_archive_total
privacy_requests_open
```

---

# 242. Labels

Low-cardinality:

```text
data_class
result
lifecycle_state
```

---

# 243. No Tenant/User IDs in metrics.

---

# 244. Tracing

```text
lifecycle.evaluate
lifecycle.plan
lifecycle.archive
lifecycle.delete
lifecycle.hold
privacy.export
privacy.delete
```

---

# 245. Health

Checks:

```text
deletion backlog
archive backend
hold store
deletion journal
search purge lag
```

---

# 246. Doctor

```text
forgeyard lifecycle doctor
```

---

# 247. Doctor Checks

```text
objects without classification
retention conflicts
stuck deletion
hold integrity
deletion journal continuity
restore replay state
```

---

# 248. Standalone Mode

Stoolap/local CAS lifecycle.

---

# 249. Distributed Mode

Postgres/CAS/object-store reconciliation.

---

# 250. HA

Multiple lifecycle workers claim batches idempotently.

---

# 251. No Single Scheduler Required

DB leases/claims.

---

# 252. Race Safety

Object deletion rechecks references/holds at execution time.

---

# 253. Policy Snapshot

Deletion plan records exact policy/hold snapshot.

---

# 254. Stale Plan

Must revalidate before destructive action.

---

# 255. DeletionPlanFreshness

```rust
pub enum DeletionPlanFreshness {
    Current,
    StalePolicy,
    NewHold,
    NewReference,
    Unknown,
}
```

---

# 256. Stale

Re-plan.

---

# 257. No Destructive Action From Stale Plan

Critical.

---

# 258. Restore Race

Deletion replay has precedence before reactivation.

---

# 259. Testkit

```text
forgeyard-lifecycle-testkit/src/
├── lib.rs
├── classify.rs
├── retention.rs
├── hold.rs
├── archive.rs
├── delete.rs
├── privacy.rs
├── restore.rs
└── assertions.rs
```

---

# 260. Unit Tests

Retention resolution.

---

# 261. Minimum Floor Test

Project cannot shorten mandatory retention.

---

# 262. Hold Test

Expired object not deleted.

---

# 263. Hold Release Test

Becomes eligible later.

---

# 264. Shared CAS Test

One tenant delete does not remove bytes still referenced by another.

---

# 265. Tombstone Test

Deleted ID not reused.

---

# 266. Privacy Pseudonymization Test

Audit integrity remains while direct identity removed.

---

# 267. Search Purge Test

Deleted user/project not discoverable.

---

# 268. Backup Restore Test

Deleted data does not resurrect.

---

# 269. WORM Test

Deletion correctly blocked until expiry.

---

# 270. External Provider Unsupported Test

Reports Unsupported, not false success.

---

# 271. Stale Plan Test

New hold blocks delete.

---

# 272. Race Test

New release root blocks CAS deletion.

---

# 273. Tenant Delete Test

Only tenant references removed.

---

# 274. Project Delete Test

No cross-project collateral damage.

---

# 275. Privacy Export Test

No other-user data leakage.

---

# 276. Legal Hold Separation-of-Duties Test

Unauthorized release denied.

---

# 277. DR Test

Holds/journal/policies restored.

---

# 278. Plugin Data Test

Unclassified plugin data rejected/flagged.

---

# 279. Fuzzing

Fuzz lifecycle policy/parser/import formats.

---

# 280. Property Tests

Held object is never eligible for destructive delete.

---

# 281. Scale Test

Billions of CAS refs/metadata rows using batched processing.

---

# 282. Failure Injection

```text
DB restart
CAS delete timeout
archive outage
search purge failure
worker crash
```

---

# 283. Implementation Phase 1 — Data Classification/Retention

Core model.

---

# 284. Phase 2 — CAS/Metadata Lifecycle

GC integration.

---

# 285. Phase 3 — Holds/Deletion Plans

Governance.

---

# 286. Phase 4 — Tenant/Project Deletion

Multi-tenancy.

---

# 287. Phase 5 — Privacy Export/Delete

Personal data workflows.

---

# 288. Phase 6 — Backup Deletion Journal

Restore safety.

---

# 289. Phase 7 — Archive Tiers

Cost reduction.

---

# 290. Phase 8 — Search/Analytics Purge

Derived data correctness.

---

# 291. Phase 9 — WORM/High-Assurance Holds

Enterprise.

---

# 292. Phase 10 — External Provider Deletion

Best effort.

---

# 293. Phase 11 — Cost/Forecast Integration

Retention economics.

---

# 294. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 295. Acceptance Tests

1. Every durable data type has lifecycle classification.
2. Retention policies are versioned and scoped.
3. Lower scopes cannot shorten mandatory retention floors.
4. Legal/security holds override ordinary expiry.
5. Expired does not mean immediately deleted.
6. Destructive deletion requires a current validated deletion plan.
7. New hold/reference makes old deletion plan stale.
8. Release-critical evidence cannot be silently GC'd.
9. CAS physical deletion occurs only when no retained reference/root exists.
10. Tenant logical deletion does not delete another tenant's shared CAS bytes.
11. Cache eviction never removes authoritative evidence.
12. Audit/security evidence may be pseudonymized rather than rewritten.
13. Principal deletion does not falsify historical event identity.
14. Privacy exports are tenant/user scoped.
15. Privacy deletions can be partially fulfilled with explicit retained reasons.
16. Search indexes purge deleted source records.
17. Derived analytics do not retain prohibited personal identifiers.
18. Backup restore replays deletion journal before normal service.
19. Deleted data is not silently resurrected from old backup.
20. WORM/legal-hold blockers are surfaced explicitly.
21. Third-party deletion limitations are reported honestly.
22. Cryptographic erasure is used only when key architecture supports it.
23. Tenant/project deletion has cooling-off/blocker/verification states.
24. Tombstones prevent ID reuse/ghost resurrection.
25. Retention-cost optimization never overrides mandatory evidence.
26. Lifecycle workers are idempotent and crash-safe.
27. Deletion retries do not cause cross-resource damage.
28. Standalone/distributed share lifecycle semantics.
29. DR restores policies, holds, and deletion journal.
30. Plugin persistent data must declare lifecycle classification.
31. Lifecycle explain can show why data is retained.
32. Deletion evidence is auditable without retaining deleted payload.
33. Search/analytics/cache remain derived and purgeable.
34. Security incidents can extend retention without rewriting base policy.
35. Forgeyard dogfoods lifecycle governance on its own CI data.

---

# 296. Production Readiness Gates

Do not call lifecycle governance production-ready until:

```text
all persistent models are classified
retention resolution is deterministic
CAS root/GC integration is safe
legal/security hold enforcement passes
stale deletion plans are rejected
tenant/project deletion is isolated
privacy export/delete paths are verified
backup restore deletion replay works
search/index purge is reliable
DR/scale/failure-injection tests pass
```

---

# 297. Architectural Invariants

1. every durable object has lifecycle classification;
2. every deletion has an explicit reason/policy;
3. retention policy is versioned;
4. legal/security holds outrank normal expiry;
5. expiry is eligibility, not deletion;
6. destructive deletion uses a current plan;
7. stale plans cannot delete;
8. retained references prevent CAS physical deletion;
9. tenant logical deletion never harms another tenant;
10. release/security evidence is preserved according to mandatory policy;
11. cache is not authoritative retention;
12. privacy deletion does not falsify immutable history;
13. pseudonymization is preferred where immutable actor references must remain;
14. deleted IDs are not reused;
15. backup restore replays deletion journal;
16. old backups cannot silently resurrect deleted data;
17. WORM/hold limitations are surfaced honestly;
18. third-party deletion is best-effort and explicit;
19. cryptographic erasure is not claimed without key support;
20. derived indexes/analytics purge after source deletion where required;
21. persistent plugin data must register lifecycle semantics;
22. cost influences retention only within allowed policy space;
23. retention changes are audited;
24. lifecycle workers are idempotent/reconciled;
25. large deletes are batched;
26. archive and delete are distinct states;
27. standalone/distributed share semantics;
28. lifecycle state is recoverable after DR;
29. deletion evidence does not contain deleted sensitive payload;
30. Forgeyard dogfoods its own data lifecycle governance.

---

# 298. Final Target Architecture

```text
                     Data Object
                         │
                         ▼
                    Classification
                         │
                         ▼
                  Retention Policy
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Active      Archive     Expire
              │          │          │
              │          │          ▼
              │          │     Deletion Plan
              │          │          │
              │          │   ┌──────┼──────┐
              │          │   ▼             ▼
              │          │ Hold/Root     Delete
              │          │   │             │
              └──────────┴───┴──► Preserve/Tombstone
```

---

# 299. Final Architectural Position

Retention:

```text
data classification
+
retention policy
+
mandatory floors
+
holds/pins
  ↓
effective lifecycle
```

Deletion:

```text
expired object
  ↓
DeletionIntent
  ↓
current reference/hold analysis
  ↓
DeletionPlan
  ↓
revalidate
  ↓
delete/archive/tombstone
  ↓
DeletionEvidence
```

Restore:

```text
restore backup
  ↓
restore holds/policies
  ↓
replay deletion journal
  ↓
purge resurrected logically-deleted data
  ↓
reconcile
  ↓
resume service
```

The key guarantee is:

> **Forgeyard can control data growth and honor deletion/privacy requirements without sacrificing security evidence, release reproducibility, tenant isolation, or recovery correctness. Data is kept because a known policy says why it must remain, and data is deleted only after Forgeyard proves that no stronger hold, retained reference, or recovery requirement still applies.**

---

# 300. Extended Architecture Sequence

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
```
