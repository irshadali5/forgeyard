# 28 — Forgeyard Audit, Compliance, Evidence Retention & Security Governance System Architecture

**Document type:** Core Audit, Compliance, Security Governance & Evidence Retention System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** immutable audit records, security event provenance, compliance evidence, retention/legal hold, audit export, SIEM integration, break-glass review, privileged-action tracking, policy/configuration history, access reviews, compliance controls, security investigations, and tamper-evident audit chains  
**Architecture style:** Append-only, tamper-evident, tenant-scoped, actor/resource/action explicit, non-repudiation-oriented where practical, independent from ordinary telemetry, retention-governed, exportable, and policy-integrated  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Policy/Authz/Identity, Secrets/Trust, Events/Reconciliation, Supply Chain, Release, Deployment, Observability, API, SCM, Plugins, HA, Operations/DR, and Multi-Tenancy. It formalizes the audit/compliance concepts referenced throughout those documents into one authoritative subsystem.

---

# 1. Purpose

Forgeyard performs high-impact actions:

```text
run code
access secrets
approve changes
sign packages
promote releases
deploy production
modify policies
enroll runners
rotate trust roots
install plugins
restore backups
change quotas
use break-glass privileges
```

A production platform must be able to answer:

```text
who did what?
when?
to which resource?
under which tenant/project?
using which identity/session?
under which policy version?
was the action approved?
was break-glass used?
what changed?
what evidence proves it?
was the record later altered?
```

The central rule is:

> **Audit is an append-only security record of meaningful actions and decisions. It is not ordinary application logging, tracing, or the domain event bus.**

A second rule is:

> **Audit records are designed for investigation, accountability, compliance, and historical verification; therefore they must survive telemetry loss, process restarts, and routine log rotation.**

A third rule is:

> **Every privileged or security-sensitive action records explicit actor, action, resource, scope, decision context, and outcome.**

---

# 2. Architectural Position

```text
                    User / Service / System
                             │
                             ▼
                      Protected Action
                             │
                  ┌──────────┼──────────┐
                  ▼          ▼          ▼
                Authn      Authz      Policy
                  │          │          │
                  └──────────┼──────────┘
                             ▼
                        Domain Action
                             │
                             ▼
                       Audit Record
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         Append Store    Hash Chain      Retention
              │              │              │
              └──────────────┼──────────────┘
                             ▼
                   Search / Export / SIEM
```

---

# 3. Goals

The subsystem MUST:

1. define audit identities;
2. record actors;
3. record actions;
4. record resources;
5. record tenant/project scope;
6. record authn context;
7. record authz/policy decision references;
8. record result/outcome;
9. be append-only;
10. detect tampering;
11. support retention;
12. support legal hold;
13. support tenant isolation;
14. support privileged-action review;
15. support break-glass review;
16. support policy/config change history;
17. support secret-access metadata without values;
18. support signing/release/deployment audit;
19. support plugin/admin audit;
20. support backup/restore audit;
21. support export;
22. support SIEM integration;
23. support compliance evidence;
24. support access reviews;
25. support investigation timelines;
26. support immutable snapshots;
27. support offline verification;
28. support HA/distributed ordering semantics;
29. support standalone mode;
30. avoid secret/PII leakage.

---

# 4. Non-Goals

Audit does not:

```text
replace domain state
replace observability logs
replace traces
replace policy engine
replace SIEM
replace legal/compliance professionals
```

---

# 5. Workspace Structure

```text
crates/audit/
├── forgeyard-audit/
├── forgeyard-audit-model/
├── forgeyard-audit-writer/
├── forgeyard-audit-store-api/
├── forgeyard-audit-query/
├── forgeyard-audit-chain/
├── forgeyard-audit-retention/
├── forgeyard-audit-hold/
├── forgeyard-audit-export/
├── forgeyard-audit-siem/
├── forgeyard-audit-review/
├── forgeyard-audit-compliance/
├── forgeyard-audit-health/
└── forgeyard-audit-testkit/
```

Use module-first boundaries; split crates only where security/runtime/dependency separation is useful.

---

# 6. AuditRecordId

```rust
pub struct AuditRecordId(Ulid);
```

Stable record identity.

---

# 7. Audit Record

```rust
pub struct AuditRecord {
    pub id: AuditRecordId,
    pub recorded_at: Timestamp,
    pub tenant: Option<TenantId>,
    pub actor: AuditActor,
    pub action: AuditAction,
    pub resource: AuditResource,
    pub context: AuditContext,
    pub outcome: AuditOutcome,
    pub integrity: AuditIntegrity,
}
```

---

# 8. Audit Actor

```rust
pub enum AuditActor {
    Principal(PrincipalId),
    Service(ServicePrincipalId),
    Runner(RunnerId),
    Workload(WorkloadIdentityId),
    System(SystemActor),
}
```

---

# 9. System Actor

Examples:

```text
Reconciler
Scheduler
MigrationCoordinator
BackupWorker
ReleaseCoordinator
```

---

# 10. No Display Name Authority

Email/usernames/display names may be stored as snapshot metadata but actor identity uses stable IDs.

---

# 11. Audit Action

Use stable machine-readable action identifiers.

Examples:

```text
run.create
run.cancel
secret.use
secret.rotate
policy.activate
release.approve
release.promote
deployment.apply
deployment.rollback
runner.enroll
runner.trust.change
plugin.install
cluster.member.remove
backup.restore
quota.override
break_glass.activate
```

---

# 12. AuditActionId

```rust
pub struct AuditActionId(BoundedString);
```

---

# 13. Resource

```rust
pub struct AuditResource {
    pub kind: AuditResourceKind,
    pub id: Option<AuditResourceId>,
    pub parent_scope: Option<ResourceScope>,
}
```

---

# 14. Resource Kinds

```text
Tenant
Organization
Project
Run
Job
Secret
Policy
Release
Deployment
Runner
Plugin
Cluster
Backup
Quota
SCM Binding
Device
Artifact
```

---

# 15. Audit Context

```rust
pub struct AuditContext {
    pub request_id: Option<RequestId>,
    pub trace_id: Option<TraceId>,
    pub session: Option<SessionId>,
    pub authn_assurance: Option<AuthnAssurance>,
    pub policy_digest: Option<PolicyDigest>,
    pub source_ip: Option<RedactedIp>,
    pub user_agent: Option<BoundedString>,
    pub reason: Option<BoundedString>,
}
```

---

# 16. Privacy

Context is minimized.

Do not capture full request body by default.

---

# 17. Outcome

```rust
pub enum AuditOutcome {
    Succeeded,
    Denied,
    Failed(AuditFailureClass),
    Cancelled,
    Unknown,
}
```

---

# 18. Denied Actions

Security-significant denials may be audited.

---

# 19. Failed Actions

Record whether failure occurred before/after external side effect if known.

---

# 20. Unknown Outcome

Important for remote effects.

---

# 21. Audit Timing

For high-risk writes:

```text
intent/decision
+
final outcome
```

may be separate linked records.

---

# 22. Audit Correlation

```rust
pub struct AuditCorrelationId(Ulid);
```

Groups multi-step action.

---

# 23. Example

Production deployment:

```text
deployment.requested
deployment.approved
deployment.apply_started
deployment.apply_succeeded
```

---

# 24. Append-Only

Audit rows are not updated in place except strictly separate archival metadata if required.

---

# 25. Corrections

If human-readable metadata needs correction:

```text
append corrective record
```

never rewrite history.

---

# 26. Deletion

Audit deletion follows explicit retention/legal policy only.

No normal CRUD delete.

---

# 27. Audit Store

Dedicated logical store interface.

---

# 28. PostgreSQL Baseline

Append-only audit table/partitioning in PostgreSQL.

---

# 29. Stronger Retention

Optional export/replication to immutable object/WORM storage.

---

# 30. Audit Partitioning

By time and optionally tenant.

---

# 31. Audit Indexes

Common:

```text
tenant
actor
action
resource
recorded_at
correlation
```

---

# 32. No Mutable "Current Audit State"

Audit is historical facts.

---

# 33. Integrity Chain

Each audit segment can use chained digest.

---

# 34. Audit Chain Entry

```rust
pub struct AuditIntegrity {
    pub previous_digest: Option<Digest>,
    pub record_digest: Digest,
    pub segment: AuditSegmentId,
}
```

---

# 35. Canonical Encoding

Hash canonical stable serialization.

---

# 36. Postcard/RON

Internal canonical record encoding may use Postcard with explicit schema.

---

# 37. Public Export

JSON/JSONL/CEF-like/SIEM formats as adapters.

---

# 38. Segment

Avoid one single global chain bottleneck.

---

# 39. Segment Strategy

Possible:

```text
tenant + day
```

or

```text
partition shard + time
```

---

# 40. No Global Total Order Requirement

Distributed system does not need one global sequential audit number.

---

# 41. Ordering

Use:

```text
recorded_at
entity sequence where available
correlation IDs
segment chain order
```

---

# 42. Tamper Evidence

Changing/removing record breaks digest chain/export seal.

---

# 43. Segment Seal

At close:

```rust
pub struct AuditSegmentSeal {
    pub segment: AuditSegmentId,
    pub final_digest: Digest,
    pub record_count: u64,
    pub signed_at: Timestamp,
    pub signature: Option<SignatureRef>,
}
```

---

# 44. Signing

Optional/required per high-assurance profile.

---

# 45. Signing Key

Separate audit-seal signing key purpose.

---

# 46. Key Rotation

Historical public keys retained.

---

# 47. WORM Export

Closed segments can be copied to immutable object storage.

---

# 48. Audit Store Failure

Critical privileged action handling must be explicit.

---

# 49. Fail-Closed Profile

High-assurance protected writes may fail if durable audit cannot be recorded.

---

# 50. Fail-Open Profile

Low-risk actions may proceed with degraded audit buffering.

---

# 51. Recommended

For:

```text
release promote
production deploy
secret reveal
trust root change
break-glass
backup restore
cluster recovery
```

require durable audit availability.

---

# 52. Audit Durability Class

```rust
pub enum AuditDurability {
    BestEffort,
    Durable,
    ComplianceCritical,
}
```

---

# 53. Durable Audit Write

Can participate in same DB transaction as business state when audit is database-local.

---

# 54. External WORM Export

Asynchronous after durable DB append.

---

# 55. Transactional Rule

For sensitive state change:

```text
business mutation
+
audit record
+
outbox
```

same transaction where practical.

---

# 56. External Effect Audit

Persist pre-call intent, then append outcome after call.

---

# 57. Secret Audit

Record:

```text
secret ID/ref
version
purpose
actor/workload
result
```

---

# 58. Never Record Secret Value

Critical.

---

# 59. Secret Reveal Audit

If plaintext reveal feature exists:

```text
actor
secret metadata
reason
step-up auth
duration
```

not value.

---

# 60. Signing Audit

Record:

```text
key ref
subject digest
evidence bundle
requesting principal/service
signing worker
result
```

---

# 61. Release Audit

Record approvals/promotions/yanks/channel moves.

---

# 62. Deployment Audit

Record plan identity, environment, approvals, rollback.

---

# 63. Policy Audit

Record:

```text
old PolicyDigest
new PolicyDigest
actor
scope
validation result
```

---

# 64. Policy Diff

Can store safe summary/CAS ref.

---

# 65. Authz Denial Audit

Sample/high-risk only to avoid enormous volume.

---

# 66. Authentication Audit

Examples:

```text
login success/failure
MFA enrollment
token issue
token revoke
session revoke
```

---

# 67. API Token Audit

Creation/revocation/use for sensitive actions.

---

# 68. Runner Audit

```text
enroll
trust class change
drain
retire
certificate revoke
```

---

# 69. Device Audit

```text
quarantine
manual reset
manual reservation
trust/pool change
```

---

# 70. Plugin Audit

```text
install
enable
permission grant
trust promotion
update
quarantine
uninstall
```

---

# 71. SCM Audit

```text
repository binding
provider installation
merge/integration submit
webhook secret rotation
```

---

# 72. Cluster Audit

```text
member add/remove
leadership transfer
maintenance
recovery
```

---

# 73. Backup/Restore Audit

Restore is always compliance-critical.

---

# 74. Quota Audit

Record overrides and tenant suspension.

---

# 75. Break-Glass

Dedicated model.

---

# 76. BreakGlassSessionId

```rust
pub struct BreakGlassSessionId(Ulid);
```

---

# 77. Break-Glass Record

```rust
pub struct BreakGlassRecord {
    pub id: BreakGlassSessionId,
    pub actor: PrincipalId,
    pub reason: BoundedString,
    pub scope: AuthorizationScope,
    pub activated_at: Timestamp,
    pub expires_at: Timestamp,
}
```

---

# 78. Activation

Requires:

```text
strong auth
specific permission
reason
expiry
```

---

# 79. Every Break-Glass Action

Audit record links to BreakGlassSessionId.

---

# 80. Review

Break-glass sessions appear in mandatory post-event review queue.

---

# 81. Review State

```rust
pub enum SecurityReviewState {
    Pending,
    Reviewed,
    Escalated,
    Closed,
}
```

---

# 82. Privileged Action Review

Can apply beyond break-glass.

---

# 83. Review Queue Examples

```text
trust root rotation
production secret reveal
restore execution
manual release override
quota override
```

---

# 84. Four-Eyes Review

Optional compliance requirement.

---

# 85. Review Does Not Undo Event

It records assessment/closure.

---

# 86. Compliance Control

```rust
pub struct ComplianceControl {
    pub id: ComplianceControlId,
    pub framework: ComplianceFramework,
    pub evidence_requirements: Vec<ComplianceEvidenceRequirement>,
}
```

---

# 87. Frameworks

Architecture can map to external frameworks such as:

```text
SOC 2
ISO 27001
PCI-style controls where relevant
internal enterprise policy
```

Forgeyard should not claim certification merely from technical support.

---

# 88. Compliance Mapping

Maps controls to evidence sources.

---

# 89. Example

Access control review:

```text
identity bindings
roles/permissions
access-review records
```

---

# 90. Evidence Record

```rust
pub struct ComplianceEvidenceRecord {
    pub control: ComplianceControlId,
    pub subject: ComplianceSubject,
    pub collected_at: Timestamp,
    pub source: EvidenceSource,
    pub artifact: Option<CasObjectRef>,
}
```

---

# 91. Evidence Sources

```text
AuditQuery
PolicySnapshot
ConfigurationSnapshot
ReleaseEvidence
BackupVerification
AccessReview
DoctorReport
```

---

# 92. Compliance Snapshot

Point-in-time export.

---

# 93. Snapshot Contents

Can include:

```text
effective policies
privileged users
active tokens
break-glass sessions
backup verification
release signing configuration
runner trust
plugin inventory
```

---

# 94. No Secret Values

Ever.

---

# 95. Access Review

Periodic review of privileged access.

---

# 96. AccessReviewId

```rust
pub struct AccessReviewId(Ulid);
```

---

# 97. Access Review Scope

```text
tenant
organization
project
system
production environment
```

---

# 98. Review Subjects

```text
human principals
service accounts
API tokens
runner trust
plugin grants
```

---

# 99. Review Decisions

```text
Retain
Revoke
Modify
Escalate
```

---

# 100. Review Evidence

Immutable.

---

# 101. Review Schedule

Periodic automation may create review tasks.

---

# 102. Expiring Privilege

Prefer automatically expiring grants where possible.

---

# 103. Security Investigation

Audit query should support timelines.

---

# 104. InvestigationId

```rust
pub struct InvestigationId(Ulid);
```

---

# 105. Investigation Bundle

Collects references, not copied mutable data where possible.

---

# 106. Investigation Timeline

Can correlate:

```text
login
policy change
secret use
run
release
deployment
break-glass
```

---

# 107. Incident Integration

Link to Part 17 IncidentId.

---

# 108. Evidence Preservation

Legal hold/investigation hold can pin relevant audit/CAS records.

---

# 109. LegalHoldId

```rust
pub struct LegalHoldId(Ulid);
```

---

# 110. Legal Hold

Prevents retention deletion for scoped records/evidence.

---

# 111. Hold Scope

```text
tenant
resource
actor
date range
investigation
```

---

# 112. Hold Creation

High privilege, audited.

---

# 113. Hold Release

High privilege, audited.

---

# 114. Retention Policy

```rust
pub struct AuditRetentionPolicy {
    pub scope: ResourceScope,
    pub minimum: Duration,
    pub maximum: Option<Duration>,
    pub archive: ArchivePolicy,
}
```

---

# 115. Retention Classes

```text
Operational
Security
Compliance
LegalHold
```

---

# 116. Audit Retention vs Telemetry

Independent.

---

# 117. Audit Retention vs Domain History

Independent but linked.

---

# 118. Purge

Only records past retention and not under hold.

---

# 119. Purge Record

Deletion itself audited at aggregate/segment level.

---

# 120. Tamper Seal Before Purge

Closed segment verification preserved.

---

# 121. Export

Supported formats:

```text
JSONL
CSV for selected reports
CEF-like
OTLP log bridge
SIEM-specific adapters
```

---

# 122. Canonical Export

JSONL with stable schema.

---

# 123. Export Manifest

```rust
pub struct AuditExportManifest {
    pub export_id: AuditExportId,
    pub query_digest: Digest,
    pub generated_at: Timestamp,
    pub record_count: u64,
    pub content_digest: Digest,
}
```

---

# 124. Export Signature

Optional/required high-assurance.

---

# 125. Offline Verification

Export manifest + segment seals can verify tamper evidence.

---

# 126. SIEM Integration

Push/stream adapter.

---

# 127. SIEM Failure

Does not lose canonical Forgeyard audit.

---

# 128. Canonical Store

Forgeyard durable audit remains authority.

---

# 129. SIEM Is Replica/consumer

Critical.

---

# 130. Delivery

At-least-once.

---

# 131. SIEM Cursor

Persist per sink.

---

# 132. Dedup

AuditRecordId.

---

# 133. Sink Health

Monitor lag.

---

# 134. Multiple SIEM Sinks

Supported.

---

# 135. Sink Credential

SecretRef.

---

# 136. Sink Filtering

Tenant/action/severity.

---

# 137. Compliance Critical Sink

Can require delivery SLA, but not replace source.

---

# 138. Audit Severity

```rust
pub enum AuditSeverity {
    Informational,
    SecurityRelevant,
    Privileged,
    Critical,
}
```

---

# 139. Examples

Critical:

```text
root trust change
restore production
cluster recovery
break-glass
production secret reveal
```

---

# 140. Security Alert

Critical audit records can trigger notification/incident rules.

---

# 141. But Alert Is Separate

Audit fact remains immutable regardless notification delivery.

---

# 142. Audit Query API

```rust
pub struct AuditQuery {
    pub tenant: Option<TenantId>,
    pub actor: Option<AuditActorFilter>,
    pub action: Option<AuditActionId>,
    pub resource: Option<AuditResourceFilter>,
    pub from: Timestamp,
    pub to: Timestamp,
    pub cursor: Option<AuditCursor>,
}
```

---

# 143. Pagination

Cursor/keyset.

---

# 144. Authorization

Audit read is sensitive.

---

# 145. Permissions

```text
audit.read
audit.export
audit.review
audit.hold
audit.admin
compliance.read
compliance.manage
```

---

# 146. Tenant Admin

Can read tenant audit per policy.

---

# 147. System Audit

System-level records restricted.

---

# 148. Sensitive Fields

Some audit records may need field-level redaction.

---

# 149. Redacted Audit View

Never mutate source record.

Projection hides fields.

---

# 150. Support Personnel

May receive limited projection.

---

# 151. Data Residency

Audit retention/storage can be tenant-region aware later.

---

# 152. Multi-Tenant Isolation

Every tenant audit query scoped.

---

# 153. System Actor Cross-Tenant Event

May emit one system record plus per-tenant references if needed.

---

# 154. Avoid Cross-Tenant Payload

Do not embed lists of unrelated tenant data in one tenant-visible record.

---

# 155. Audit DTO

Separate from storage domain.

---

# 156. Public API

Potential:

```text
GET  /v1/audit/events
GET  /v1/audit/events/{id}
POST /v1/audit/exports
GET  /v1/audit/exports/{id}
POST /v1/audit/holds
POST /v1/security/access-reviews
GET  /v1/compliance/evidence
```

---

# 157. Admin UI

Pages:

```text
Audit
Security Reviews
Break Glass
Access Reviews
Compliance
Legal Holds
Exports
SIEM
```

---

# 158. Audit Timeline UI

Filters:

```text
actor
action
resource
scope
date
severity
outcome
```

---

# 159. Record Detail

Shows:

```text
actor
action
resource
authn context
policy digest
reason
outcome
integrity status
correlation
```

---

# 160. Integrity Badge

```text
Verified
Unverified
Broken
```

---

# 161. Broken Chain

Critical alert.

---

# 162. Break-Glass Dashboard

Shows active/recent sessions.

---

# 163. Review Queue

Action-required.

---

# 164. Access Review UI

Reviewer sees effective access and last use.

---

# 165. Last-Used Facts

Derived from audit, not authz authority.

---

# 166. Compliance Dashboard

Shows evidence completeness, not certification claim.

---

# 167. Language

Use:

```text
control evidence available
```

not:

```text
SOC2 certified
```

unless externally certified.

---

# 168. Audit Writer API

```rust
#[async_trait]
pub trait AuditWriter {
    async fn append(
        &self,
        record: NewAuditRecord,
    ) -> Result<AuditRecordId, AuditWriteError>;
}
```

---

# 169. High-Risk Transaction Helper

```rust
pub trait AuditedTransaction {
    async fn execute_with_audit(...);
}
```

conceptually.

---

# 170. Avoid Forgetting Audit

Sensitive service methods can require audit context/capability.

---

# 171. Typed Audit Requirement

```rust
pub struct AuditRequiredContext {
    pub actor: AuditActor,
    pub action: AuditActionId,
    pub resource: AuditResource,
}
```

---

# 172. Service Signature Example

```rust
release.promote(
    authz: Authorized<ReleasePromote>,
    audit: AuditRequiredContext,
    request: PromoteReleaseRequest,
)
```

---

# 173. Compiler Help

Use type-system to make high-risk methods difficult to call without audit context.

---

# 174. No Generic String Audit Everywhere

Known action IDs/types where possible.

---

# 175. Audit Schema Version

```rust
pub struct AuditSchemaVersion(u16);
```

---

# 176. Backward Compatibility

Historical decoders retained.

---

# 177. Migration

Append store schema evolves expand-contract.

---

# 178. Segment Hash Schema

Versioned canonicalization.

---

# 179. Audit Chain Verification

```rust
pub trait AuditVerifier {
    fn verify_segment(
        &self,
        records: &[AuditRecord],
        seal: &AuditSegmentSeal,
    ) -> AuditVerificationResult;
}
```

---

# 180. Verification Modes

```text
Quick
Full
Offline
```

---

# 181. Quick

Check seal/metadata.

---

# 182. Full

Recompute all record digests.

---

# 183. Offline

Using exported segment + public verification material.

---

# 184. Periodic Scrub

Audit integrity verifier runs periodically.

---

# 185. Failure

Broken chain:

```text
mark segment compromised
alert
preserve evidence
investigate
```

---

# 186. Never Auto-Heal Audit History

Critical.

---

# 187. Backup

Audit store included in metadata backup.

---

# 188. WORM Replica

Independent retention.

---

# 189. Restore

Audit chain verified after restore.

---

# 190. PITR

May restore audit to earlier point.

---

# 191. External WORM

Can preserve later immutable segments independently.

---

# 192. DR

Audit reconstruction must not invent missing records.

---

# 193. Missing Audit Range

Explicit gap.

---

# 194. Gap Record

Post-recovery system can append:

```text
AUDIT_GAP_DECLARED
```

with known range/reason.

---

# 195. Never Backfill Fiction

Critical.

---

# 196. Time

Use server timestamp.

---

# 197. Clock Skew

Record monotonic/entity sequence/correlation where possible.

---

# 198. Legal Ordering

Audit timestamps are evidence, not guaranteed globally linearizable time.

---

# 199. Entity Sequence

For some resources, include entity version.

---

# 200. Request Source

IP/user-agent optional and privacy-sensitive.

---

# 201. IP Retention

Policy-controlled.

---

# 202. Authentication Assurance

Record:

```text
password/local
OIDC
MFA
step-up
```

without credential details.

---

# 203. Token Identity

Token ID/fingerprint, not token value.

---

# 204. Workload Identity

Record exact workload identity.

---

# 205. Policy Decision

Record PolicyDigest and relevant decision ID.

---

# 206. Full Policy Input

Not always stored due privacy/size.

---

# 207. Policy Explanation

Store safe reason codes.

---

# 208. Denial Reason

Stable code.

---

# 209. Change History

Configuration changes:

```text
before digest
after digest
```

---

# 210. Full Config

May exist as separate CAS snapshot with permissions.

---

# 211. Secret-Containing Config

Only sanitized snapshot.

---

# 212. Resource Deletion

Audit before/after identifiers.

---

# 213. Tenant Closure

Audit entire lifecycle.

---

# 214. Export Deletion

Audit.

---

# 215. Legal Hold Change

Audit.

---

# 216. Audit Admin Changes

Meta-audit.

---

# 217. Who Can Disable Audit?

No normal user.

---

# 218. Maintenance

Audit remains enabled during maintenance.

---

# 219. Safe Mode

Audit at least local/durable emergency path.

---

# 220. Catastrophic DB Outage

If durable audit unavailable, protected critical writes should remain blocked.

---

# 221. Emergency Recovery

Break-glass recovery actions can write to secure recovery journal if DB unavailable.

---

# 222. Recovery Journal

Small append-only local signed file.

---

# 223. Import After DB Recovery

Recovery journal imported as clearly marked recovery-origin audit records.

---

# 224. RecoveryJournalId

```rust
pub struct RecoveryJournalId(Ulid);
```

---

# 225. Use Only Catastrophic Paths

Not normal audit path.

---

# 226. Journal Integrity

Chained/signature protected.

---

# 227. No Secret Values

Same.

---

# 228. Access Reviews

Can be automated:

```text
create review
collect current grants
collect last-used facts
assign reviewer
record decisions
apply revocations through normal authz/admin services
```

---

# 229. Review Action

Audit itself.

---

# 230. Compliance Evidence Automation

Scheduled jobs can collect:

```text
backup restore status
MFA coverage
privileged-role list
plugin inventory
release signing state
```

---

# 231. Automation Does Not Certify

Produces evidence only.

---

# 232. Evidence Freshness

Each control has freshness window.

---

# 233. Missing Evidence

Explicit `Incomplete`.

---

# 234. Stale Evidence

Explicit `Stale`.

---

# 235. Compliance Status

```rust
pub enum ComplianceEvidenceStatus {
    Complete,
    Incomplete,
    Stale,
    Failed,
}
```

---

# 236. No False Pass

Critical.

---

# 237. Evidence Bundle

Can export point-in-time compliance package.

---

# 238. Bundle Contents

```text
control map
audit query extracts
policy digests
access review records
backup verification evidence
release/signing evidence
```

---

# 239. Bundle Integrity

Signed manifest.

---

# 240. External Auditor

Can receive least-privilege exported bundle rather than production access.

---

# 241. SIEM Adapter

```rust
pub trait AuditSink {
    async fn deliver(
        &self,
        batch: AuditBatch,
    ) -> Result<AuditSinkResult, AuditSinkError>;
}
```

---

# 242. Sink Delivery State

```text
Pending
Delivered
Failed
Unknown
```

---

# 243. Sink Reconciliation

Retry idempotently by AuditRecordId/export cursor.

---

# 244. Unknown

Inspect/ack semantics depending sink.

---

# 245. Metrics

```text
audit_records_appended_total
audit_write_failures_total
audit_export_lag_seconds
audit_chain_verification_failures
audit_siem_lag_seconds
audit_break_glass_active
audit_pending_reviews
compliance_evidence_stale
```

---

# 246. Labels

Low cardinality:

```text
action_class
severity
outcome
sink_type
```

---

# 247. No PrincipalId/TenantId Metric Labels

Use audit query store.

---

# 248. Tracing

```text
audit.append
audit.segment.seal
audit.verify
audit.export
audit.siem.deliver
audit.review
```

---

# 249. Health

Checks:

```text
append store
segment sealer
WORM exporter
SIEM lag
integrity verification
```

---

# 250. Doctor

```text
forgeyard audit doctor
```

---

# 251. Doctor Checks

```text
append capability
recent chain verification
retention policy
hold consistency
export sink connectivity
recovery journal state
```

---

# 252. Security Governance Doctor

Can check:

```text
active break-glass sessions
unreviewed privileged actions
stale access reviews
```

---

# 253. CLI

```text
forgeyard audit search
forgeyard audit show
forgeyard audit verify
forgeyard audit export
forgeyard audit hold create
forgeyard audit hold release
forgeyard security review
forgeyard compliance evidence
forgeyard compliance export
```

---

# 254. Search

Cursor based.

---

# 255. Verify

Full/offline options.

---

# 256. Export

Permission/audit.

---

# 257. Hold

High privilege.

---

# 258. Audit Query Limits

Bounded date range/page size by default.

---

# 259. Large Export

Async operation.

---

# 260. Export Artifact

Stored in protected CAS/object store.

---

# 261. Export Retention

Explicit.

---

# 262. Export Encryption

For sensitive exports.

---

# 263. Export Download

Short-lived authorized URL.

---

# 264. Data Minimization

Audit only what is needed.

---

# 265. Avoid Full Payload Capture

Especially:

```text
source files
logs
request bodies
secret material
```

---

# 266. Personal Data

Keep minimal and retention-aware.

---

# 267. Right-to-Delete Conflicts

Legal/compliance retention may override ordinary account deletion depending policy/law; architecture supports holds/retention, not legal interpretation.

---

# 268. Tenant Audit Export

Scoped.

---

# 269. Cross-Tenant Operator Investigation

Requires system-level security permission.

---

# 270. Support Staff

Do not automatically receive audit access.

---

# 271. Audit Severity Mapping

Central registry.

---

# 272. High-Risk Actions

Must be statically enumerated.

---

# 273. Compiler/Architecture Check

Can enforce known sensitive service methods call audit wrapper.

---

# 274. Architecture Lint

`forgeyard-architecture-check` can inspect service metadata/macros.

---

# 275. Audited Macro

Optional:

```rust
#[audited(action = "release.promote", durability = "compliance")]
```

But explicit types may be clearer.

---

# 276. No Macro Magic as Sole Guarantee

Tests/architecture checks required.

---

# 277. Audit Testkit

```text
forgeyard-audit-testkit/src/
├── lib.rs
├── record.rs
├── writer.rs
├── chain.rs
├── retention.rs
├── hold.rs
├── export.rs
├── review.rs
└── assertions.rs
```

---

# 278. Unit Tests

Canonical encoding/digest chain.

---

# 279. Tamper Test

Modify one historical record -> verification fails.

---

# 280. Deletion Test

Remove record -> chain fails.

---

# 281. Reorder Test

Reorder records -> chain fails.

---

# 282. Segment Seal Test

Valid.

---

# 283. Key Rotation Test

Old segments still verify.

---

# 284. Secret Leakage Test

Test secret never appears in audit.

---

# 285. Token Leakage Test

Bearer/API token never appears.

---

# 286. Privileged Action Test

Release promotion writes audit transactionally.

---

# 287. Denied Action Test

High-risk denial captured.

---

# 288. Break-Glass Test

Every action links session.

---

# 289. Review Test

Break-glass session cannot silently disappear from review queue.

---

# 290. Legal Hold Test

Held segment not purged.

---

# 291. Retention Test

Expired non-held records purge only through policy.

---

# 292. Export Integrity Test

Export digest/signature verifies.

---

# 293. Tenant Isolation Test

Tenant A cannot query Tenant B audit.

---

# 294. RLS/Store Scope Test

Defense in depth.

---

# 295. SIEM Failure Test

Canonical audit preserved despite sink outage.

---

# 296. SIEM Replay Test

No duplicate semantic confusion.

---

# 297. DR Restore Test

Audit chain verifies after restore.

---

# 298. Recovery Journal Test

Catastrophic recovery actions imported with origin preserved.

---

# 299. Audit Gap Test

Missing range declared, never fabricated.

---

# 300. Compliance Evidence Test

Stale control evidence does not show Complete.

---

# 301. Access Review Test

Revocation performed through normal authz service and audited.

---

# 302. Scale Test

High event volume with partitioned storage.

---

# 303. Query Scale

Large date-range exports without exhausting memory.

---

# 304. Fuzzing

Fuzz audit decoders/export parsers.

---

# 305. Failure Injection

```text
DB outage
WORM export failure
SIEM outage
segment sealer crash
signing key unavailable
```

---

# 306. Implementation Phase 1 — Audit Model/Writer

Core append-only records.

---

# 307. Phase 2 — Transactional Integration

High-risk service actions.

---

# 308. Phase 3 — Query/UI/API

Search/detail.

---

# 309. Phase 4 — Integrity Chains/Segment Seals

Tamper evidence.

---

# 310. Phase 5 — Retention/Legal Hold

Governance.

---

# 311. Phase 6 — SIEM Export

External integration.

---

# 312. Phase 7 — Break-Glass/Privileged Review

Security governance.

---

# 313. Phase 8 — Access Reviews

Periodic privilege governance.

---

# 314. Phase 9 — Compliance Evidence

Control mapping/export.

---

# 315. Phase 10 — WORM/Offline Verification

High assurance.

---

# 316. Phase 11 — Recovery Journal

Catastrophic operations.

---

# 317. Phase 12 — Scale/Chaos/Security Hardening

Production readiness.

---

# 318. Acceptance Tests

1. Audit is separate from telemetry/logging.
2. Audit is separate from domain event transport.
3. Every privileged action records stable actor/action/resource.
4. Tenant/project scope is explicit.
5. Display names/emails are not actor authority.
6. Secret values never enter audit records.
7. API/bearer tokens never enter audit records.
8. PolicyDigest is recorded for protected actions where available.
9. High-risk business mutation and audit append are transactional where practical.
10. External-effect intents/outcomes are both auditable.
11. Audit records are append-only.
12. Historical corrections append new records rather than rewrite old.
13. Digest-chain tampering is detectable.
14. Closed segments can be sealed/signed.
15. Historical seals remain verifiable after signing-key rotation.
16. WORM export is independent from canonical query store.
17. SIEM outage never destroys canonical audit.
18. Tenant A cannot query Tenant B audit.
19. Break-glass use is fully linked and reviewable.
20. Production secret reveal is compliance-critical and audited.
21. Release promotion/deployment/restore/cluster recovery are compliance-critical.
22. Legal hold prevents retention purge.
23. Retention deletion cannot remove held records.
24. Export bundles include integrity manifest.
25. Offline verification works with public verification material.
26. Missing audit records after disaster are declared as gaps, never fabricated.
27. Access reviews record decisions and resulting revocations.
28. Compliance evidence distinguishes Complete/Incomplete/Stale/Failed.
29. Forgeyard never claims certification from technical evidence alone.
30. Audit store failure can fail closed for configured critical actions.
31. Audit integrity is rechecked after restore.
32. Recovery-journal actions remain distinguishable from normal audit.
33. Plugin/RBE/SCM/device alternate paths audit high-risk actions consistently.
34. Standalone/distributed share audit semantics.
35. Forgeyard's own production release/deployment/admin actions use this audit subsystem.

---

# 319. Production Readiness Gates

Do not call enterprise audit/compliance production-ready until:

```text
append-only audit model is stable
high-risk service coverage is complete
secret/token leakage tests pass
tenant isolation passes
transactional audit writes work
integrity chain verification works
retention/legal hold works
break-glass review works
SIEM export is resilient
restore integrity verification passes
compliance evidence reports stale/incomplete honestly
```

---

# 320. Architectural Invariants

1. audit is not telemetry;
2. audit is not domain event transport;
3. audit records are append-only;
4. actor identity uses stable IDs;
5. action/resource/scope are explicit;
6. secret values never enter audit;
7. tokens/private keys never enter audit;
8. high-risk actions require durable audit;
9. critical business mutation and audit share transaction where practical;
10. external effects record intent and outcome;
11. audit integrity tampering is detectable;
12. no single global ordering is required;
13. segment chains are bounded and verifiable;
14. retention never overrides legal hold;
15. SIEM is a replica/consumer, not source of truth;
16. export manifests are integrity protected;
17. tenant audit queries are isolated;
18. break-glass is explicit, expiring, reasoned, and reviewable;
19. privileged access reviews are first-class;
20. compliance evidence never implies certification automatically;
21. stale evidence is not reported as complete;
22. historical gaps are declared, not fabricated;
23. recovery actions remain auditable even during catastrophic outages;
24. audit survives routine telemetry/log rotation;
25. policy/config changes record before/after digests;
26. plugin/SCM/RBE/device paths cannot bypass audit requirements;
27. audit storage and WORM replication have independent health;
28. standalone/distributed share audit semantics;
29. architecture checks help prevent missing audit on sensitive services;
30. Forgeyard dogfoods its audit/security-governance layer.

---

# 321. Final Target Architecture

```text
                Principal / Service / System
                           │
                           ▼
                    Protected Action
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
           Authn         Authz         Policy
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                      Domain Mutation
                           │
                           ▼
                      Audit Append
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
        Append Store    Hash Chain      WORM
             │             │             │
             └─────────────┼─────────────┘
                           ▼
               Query / Review / Export
                           │
                ┌──────────┼──────────┐
                ▼          ▼          ▼
              SIEM     Compliance  Investigation
```

---

# 322. Final Architectural Position

Privileged action:

```text
authenticated actor
+
authorized permission
+
policy decision
+
resource identity
  ↓
domain action
  ↓
append-only audit record
  ↓
tamper-evident segment
```

Break-glass:

```text
strong auth
+
reason
+
scope
+
expiry
  ↓
BreakGlassSessionId
  ↓
all actions linked
  ↓
mandatory review
```

Compliance:

```text
control requirement
+
audit/policy/backup/release/access-review evidence
  ↓
Complete / Incomplete / Stale / Failed
  ↓
signed export bundle
```

The key guarantee is:

> **Forgeyard can reconstruct and verify the security history of privileged actions without relying on ephemeral logs. Audit records are durable, append-only, tenant-scoped, tamper-evident, retention-governed, and independently exportable, while compliance tooling reports evidence honestly rather than pretending technical controls alone equal certification.**

---

# 323. Extended Architecture Sequence

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
```
