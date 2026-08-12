# 27 — Forgeyard Multi-Tenancy, Quotas, Resource Governance & Fair-Use System Architecture

**Document type:** Core Multi-Tenant Isolation, Quota, Usage Accounting & Resource Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** tenant/org/project boundaries, quota hierarchy, scheduler fairness, compute budgets, storage/CAS limits, concurrency controls, bandwidth limits, device/RBE/plugin/provider quotas, cross-tenant cache policy, noisy-neighbor protection, usage metering, overage behavior, enforcement, observability, administration, and enterprise governance  
**Architecture style:** Strong logical isolation, deny-by-default cross-tenant access, hierarchical quotas, centralized policy with distributed enforcement, scheduler-aware fairness, bounded resource consumption, explicit over-quota states, immutable usage records, and no resource-governance bypass through alternate protocols or plugins  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Extends Core Domain, Storage, CAS, Scheduler, Runner, RBE, Device Lab, Plugins, API, Policy/Authz, Release, Deployment, Observability, HA, and Operations. This document fills the major enterprise-production gap left after the original 01–26 series.

---

# 1. Purpose

Forgeyard may serve:

```text
one developer
one team
multiple teams
multiple business units
multiple customers
enterprise tenants
hosted SaaS tenants
```

A correct multi-tenant platform must prevent one tenant from:

```text
reading another tenant's data
using another tenant's secrets
consuming all scheduler capacity
filling shared CAS/storage
monopolizing device pools
exhausting API limits
poisoning shared cache
causing unfair queue starvation
```

The central rule is:

> **Every resource-owning object in distributed Forgeyard has an explicit tenant scope, and every access or allocation is evaluated within that scope.**

A second rule is:

> **Quotas limit consumption; scheduler fairness allocates contention; policy decides exceptions. These are related but separate mechanisms.**

A third rule is:

> **Cross-tenant sharing is always explicit and policy-controlled. Shared infrastructure does not imply shared authorization or shared cache visibility.**

---

# 2. Architectural Position

```text
                       Tenant
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
        Organization   Project    Environment
             │           │           │
             └───────────┼───────────┘
                         ▼
                   Resource Scope
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Quotas         Fairness        Policy
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                   Enforcement
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      Scheduler         CAS/API       Devices/RBE
```

---

# 3. Goals

The subsystem MUST:

1. define tenant hierarchy;
2. define organization/project ownership;
3. isolate metadata;
4. isolate secrets;
5. isolate CAS visibility;
6. isolate cache namespaces;
7. define quota hierarchy;
8. define hard and soft quotas;
9. define concurrency budgets;
10. define compute budgets;
11. define storage quotas;
12. define bandwidth quotas;
13. define runner-pool quotas;
14. define device quotas;
15. define RBE quotas;
16. define API limits;
17. define plugin limits;
18. define provider-operation limits;
19. support scheduler fairness;
20. prevent noisy neighbors;
21. support reservations;
22. support burst capacity;
23. support usage metering;
24. support over-quota handling;
25. support admin overrides;
26. support per-tenant observability;
27. support cost attribution;
28. support standalone mode without unnecessary complexity;
29. support distributed SaaS/enterprise mode;
30. preserve correctness across HA/failover.

---

# 4. Non-Goals

This subsystem does not itself define:

```text
customer billing
payment processing
tax
invoicing
commercial pricing plans
```

It provides accurate resource-governance and usage primitives that such systems can consume later.

---

# 5. Workspace Structure

```text
crates/governance/
├── forgeyard-governance/
├── forgeyard-governance-model/
├── forgeyard-tenant/
├── forgeyard-organization/
├── forgeyard-quota/
├── forgeyard-budget/
├── forgeyard-fairness/
├── forgeyard-usage/
├── forgeyard-metering/
├── forgeyard-reservation/
├── forgeyard-overage/
├── forgeyard-resource-policy/
├── forgeyard-governance-store-api/
├── forgeyard-governance-health/
└── forgeyard-governance-testkit/
```

Use modules first; split crates only at real runtime/security/dependency boundaries.

---

# 6. Tenant Identity

```rust
pub struct TenantId(Ulid);
```

Stable business isolation boundary.

---

# 7. Tenant

```rust
pub struct Tenant {
    pub id: TenantId,
    pub name: TenantName,
    pub state: TenantState,
}
```

---

# 8. Tenant State

```rust
pub enum TenantState {
    Active,
    Suspended,
    ReadOnly,
    Closing,
    Closed,
}
```

---

# 9. Organization

```rust
pub struct OrganizationId(Ulid);
```

---

# 10. Organization Scope

A tenant may contain multiple organizations/business units.

---

# 11. Project Scope

`ProjectId` always belongs to exactly one tenant.

---

# 12. Environment Scope

Deployment environments inherit tenant/project ownership.

---

# 13. Resource Scope

```rust
pub enum ResourceScope {
    Tenant(TenantId),
    Organization(OrganizationId),
    Project(ProjectId),
    Environment(EnvironmentId),
}
```

---

# 14. Ownership Invariant

Every persisted resource with security/resource significance must resolve to one tenant.

---

# 15. Examples

```text
Run
Job
Artifact
SecretRef
Release
Deployment
Device reservation
RBE instance
Plugin state
```

---

# 16. No Tenant-by-Name Authority

Use stable `TenantId`.

---

# 17. Tenant Context

```rust
pub struct TenantContext {
    pub tenant: TenantId,
    pub organization: Option<OrganizationId>,
    pub project: Option<ProjectId>,
}
```

---

# 18. Request Boundary

API authentication resolves principal.

Resource resolution determines tenant scope.

Authz verifies principal has rights in that tenant.

---

# 19. No Client-Supplied Tenant Trust

Client header/field alone cannot choose arbitrary tenant.

---

# 20. Store Isolation

Every distributed metadata query is tenant-scoped unless explicitly system-level.

---

# 21. SQL Strategy

Recommended baseline:

```text
tenant_id column
+
indexes
+
query-layer enforcement
+
optional PostgreSQL RLS defense-in-depth
```

---

# 22. PostgreSQL RLS

Can provide additional isolation in hosted/high-assurance mode.

---

# 23. RLS Is Defense-in-Depth

Application still performs authz.

---

# 24. Query API

Store methods should carry `TenantId` explicitly.

---

# 25. Bad Pattern

```rust
get_run(run_id)
```

when tenant context is required.

---

# 26. Better Pattern

```rust
get_run(tenant_id, run_id)
```

or typed scoped repository.

---

# 27. Scoped Store

```rust
pub struct TenantStore<'a> {
    tenant: TenantId,
    inner: &'a dyn ForgeyardStore,
}
```

---

# 28. Type-System Goal

Reduce accidental unscoped access.

---

# 29. Global System Tables

Only true global data:

```text
cluster membership
global migrations
system trust metadata
```

---

# 30. Secret Isolation

`SecretRef` includes scope.

---

# 31. Secret Provider Path

Never infer tenant from secret name string.

---

# 32. CAS Visibility

Physical bytes may deduplicate globally.

Authorization remains metadata-scoped.

---

# 33. CAS Read

Requires authorized artifact/object reference within tenant scope.

---

# 34. Raw Digest Read

Not automatically authorized.

Knowing digest does not grant access.

---

# 35. Cross-Tenant Dedup

Optional physical optimization.

---

# 36. High-Assurance Mode

Can disable physical cross-tenant dedup if timing/existence leakage is unacceptable.

---

# 37. Cache Namespace

```rust
pub struct CacheNamespaceId(Digest);
```

---

# 38. Default Cache Scope

```text
tenant + project + execution semantics
```

---

# 39. Cross-Tenant Cache

Off by default.

---

# 40. Explicit Shared Cache

Requires:

```text
policy
trust equivalence
artifact confidentiality rules
cache provenance
```

---

# 41. Cache Poisoning

A tenant cannot write another tenant's cache namespace.

---

# 42. Quota

```rust
pub struct QuotaDefinition {
    pub id: QuotaId,
    pub scope: ResourceScope,
    pub resource: QuotaResource,
    pub limit: QuotaLimit,
    pub enforcement: QuotaEnforcement,
}
```

---

# 43. QuotaId

```rust
pub struct QuotaId(Ulid);
```

---

# 44. Quota Resource

```rust
pub enum QuotaResource {
    ConcurrentJobs,
    CpuSeconds,
    MemoryByteSeconds,
    GpuSeconds,
    CasBytes,
    ArtifactBytes,
    LogBytes,
    ApiRequests,
    UploadBytes,
    DownloadBytes,
    DeviceMinutes,
    RbeExecutions,
    PluginCpuSeconds,
    ProviderOperations,
    Custom(QuotaResourceId),
}
```

---

# 45. Quota Limit

```rust
pub enum QuotaLimit {
    Count(u64),
    Bytes(u64),
    Duration(Duration),
    Rate(RateLimit),
    Unlimited,
}
```

---

# 46. Hard vs Soft

```rust
pub enum QuotaEnforcement {
    Hard,
    Soft,
    WarnOnly,
}
```

---

# 47. Hard Quota

New consumption denied when exceeded.

---

# 48. Soft Quota

May continue with warning/overage state.

---

# 49. WarnOnly

Observability/admin.

---

# 50. Hierarchical Quota

Example:

```text
Tenant: 100 concurrent jobs
  Organization A: 60
    Project X: 30
```

---

# 51. Effective Quota

Minimum/combined constraints across hierarchy.

---

# 52. Quota Inheritance

Child may tighten parent.

Cannot exceed parent without explicit delegated reservation.

---

# 53. Reserved Capacity

```rust
pub struct CapacityReservation {
    pub id: ReservationId,
    pub scope: ResourceScope,
    pub resource: QuotaResource,
    pub amount: ResourceAmount,
    pub valid_from: Timestamp,
    pub valid_until: Timestamp,
}
```

---

# 54. Use Cases

```text
release window
large migration
scheduled benchmark
critical production build
```

---

# 55. Reservation Does Not Bypass Global Physical Capacity

It guarantees priority/budget within available infrastructure.

---

# 56. Burst Capacity

```rust
pub struct BurstPolicy {
    pub baseline: ResourceAmount,
    pub burst: ResourceAmount,
    pub max_duration: Duration,
}
```

---

# 57. Burst Use

Short workload spikes.

---

# 58. Burst Exhaustion

Return to baseline.

---

# 59. Concurrency Budget

Most important operational quota.

---

# 60. Concurrent Job Counter

Count active scheduler reservations/leases.

---

# 61. Pending Jobs

Do not count as active compute usage.

---

# 62. Preparing

Usually counts once resources reserved.

---

# 63. Device Concurrency

Count DeviceLease.

---

# 64. RBE Concurrency

RBE jobs use same job concurrency plus optional RBE-specific cap.

---

# 65. Release/Deployment Concurrency

Separate operational limits.

---

# 66. Resource Budget

Long-term consumption.

---

# 67. CPU Usage

Meter:

```text
allocated or measured CPU-seconds
```

---

# 68. Memory Usage

Potential:

```text
memory byte-seconds
```

---

# 69. GPU

GPU-seconds/device-seconds.

---

# 70. Measurement Policy

Must define whether metering is:

```text
requested
reserved
actual measured
```

---

# 71. Recommended

Use reserved for enforceable real-time limits, actual measured for reporting/cost attribution.

---

# 72. Usage Record

```rust
pub struct UsageRecord {
    pub id: UsageRecordId,
    pub tenant: TenantId,
    pub project: Option<ProjectId>,
    pub resource: UsageResource,
    pub amount: UsageAmount,
    pub period: UsagePeriod,
    pub source: UsageSource,
}
```

---

# 73. UsageRecordId

```rust
pub struct UsageRecordId(Ulid);
```

---

# 74. Usage Source

```rust
pub enum UsageSource {
    JobAttempt(JobAttemptId),
    Artifact(ArtifactId),
    DeviceLease(DeviceLeaseId),
    RbeExecution(RbeExecutionId),
    ApiOperation(ApiOperationId),
}
```

---

# 75. Usage Idempotency

Same source/event cannot double-count.

---

# 76. Metering Semantics

At-least-once events + idempotent usage aggregation.

---

# 77. No Exactly-Once Claim

Use source identity/dedup.

---

# 78. Usage Event

```text
ResourceUsageObserved
```

---

# 79. Aggregation

Hourly/daily/monthly rollups.

---

# 80. Raw Usage Retention

Bounded.

---

# 81. Aggregates

Longer retention.

---

# 82. Usage Auditability

Need enough lineage to explain totals.

---

# 83. Quota Counter Authority

Real-time enforceable counters must derive from authoritative reservations/state.

---

# 84. Metrics Are Not Quota Authority

Critical.

---

# 85. Scheduler Fairness

Quota answers:

```text
may tenant consume more?
```

Fairness answers:

```text
who gets next scarce resource?
```

---

# 86. Weighted Fair Queueing

Existing scheduler baseline.

---

# 87. Tenant Weight

```rust
pub struct TenantSchedulingWeight(NonZeroU32);
```

---

# 88. Project Weight

Optional within tenant.

---

# 89. Aging

Prevents starvation.

---

# 90. DRF

Dominant Resource Fairness later for multi-dimensional fairness.

---

# 91. No Self-Assigned Priority

Tenant/user cannot raise weight via pipeline.

---

# 92. Priority Ceiling

Per scope/policy.

---

# 93. Queue Partition

Logical by tenant, not separate scheduler instance.

---

# 94. Shared Runner Pool

Fairly multiplex tenants.

---

# 95. Dedicated Runner Pool

Tenant-specific.

---

# 96. Pool Access Policy

```text
shared
tenant-dedicated
organization-dedicated
project-dedicated
```

---

# 97. Runner Trust

Separate from tenancy.

---

# 98. Dedicated Pool

Useful for:

```text
compliance
performance isolation
proprietary workloads
```

---

# 99. Noisy Neighbor

Sources:

```text
CPU
memory
disk
CAS bandwidth
API requests
large logs
device queue
```

---

# 100. Compute Isolation

Runner sandbox/cgroups enforce per-job resource limits.

---

# 101. Host Headroom

Scheduler reserves headroom.

---

# 102. Tenant Host Saturation

Tenant cannot exceed configured pool/global quota.

---

# 103. Storage Quota

Track logical tenant-owned bytes.

---

# 104. Physical Dedup Accounting

Need defined accounting policy.

---

# 105. Logical Accounting

Recommended:

```text
charge each tenant logical referenced bytes
```

even if physical CAS deduplicates.

---

# 106. Why

Prevents one tenant exploiting dedup accounting ambiguity.

---

# 107. Shared Blob

Each tenant referencing it counts toward logical quota.

---

# 108. CAS Quota

Can distinguish:

```text
cache bytes
artifact bytes
release-retained bytes
```

---

# 109. Ephemeral Cache Quota

LRU/GC when exceeded.

---

# 110. Durable Artifact Quota

New upload blocked/warned depending policy.

---

# 111. Release-Critical Artifact

Cannot be deleted merely to enforce generic cache quota.

---

# 112. Retention vs Quota Conflict

Policy resolves.

---

# 113. Safe Behavior

If retention requires object, charge usage and block new writes rather than delete protected release.

---

# 114. Log Quota

Prevent unbounded logs.

---

# 115. Per-Job Log Limit

Bound.

---

# 116. Tenant Log Retention

Quota/retention.

---

# 117. API Quota

Rate/concurrency.

---

# 118. Upload Quota

Bytes/time.

---

# 119. Download Quota

Optional.

---

# 120. RBE Quota

Specific:

```text
CAS bytes
action cache bytes
concurrent executes
execution CPU
```

---

# 121. Device Lab Quota

```text
concurrent DeviceLease
device minutes
pool-specific limits
```

---

# 122. Rare Device Reservation

Policy may restrict.

---

# 123. Plugin Quota

External plugin process resources.

---

# 124. Provider Operations

Protect SCM/cloud API rate budgets.

---

# 125. Secret Operations

Can rate-limit high-risk secret/sign operations.

---

# 126. Signing Quota

Not commercial quota necessarily—security/risk limit.

---

# 127. Release Promotion Limit

Prevent accidental mass operations.

---

# 128. Deployment Concurrency

Environment-level plus tenant-level.

---

# 129. Quota Evaluation

```rust
pub trait QuotaService {
    async fn evaluate(
        &self,
        request: QuotaRequest,
    ) -> Result<QuotaDecision, QuotaError>;
}
```

---

# 130. QuotaRequest

```rust
pub struct QuotaRequest {
    pub scope: ResourceScope,
    pub resource: QuotaResource,
    pub amount: ResourceAmount,
    pub operation: QuotaOperation,
}
```

---

# 131. Decision

```rust
pub enum QuotaDecision {
    Allowed,
    AllowedWithWarning(QuotaWarning),
    Denied(QuotaViolation),
}
```

---

# 132. Reservation API

Real-time scarce resources should reserve quota atomically with scheduling/resource reservation.

---

# 133. QuotaReservation

```rust
pub struct QuotaReservation {
    pub id: QuotaReservationId,
    pub tenant: TenantId,
    pub resource: QuotaResource,
    pub amount: ResourceAmount,
    pub lease: Option<LeaseId>,
    pub expires_at: Timestamp,
}
```

---

# 134. Scheduler Transaction

Where feasible:

```text
check quota
reserve tenant capacity
reserve runner resources
create JobLease
```

atomic/serializable enough to prevent oversubscription.

---

# 135. Concurrency Race

Two scheduler workers cannot both exceed hard quota.

---

# 136. Postgres Enforcement

Use locked/atomic counters or reservation rows.

---

# 137. HA Scheduler

Epoch fencing still applies.

---

# 138. Quota Reservation Expiry

Reconcile stale reservations.

---

# 139. Job Completion

Release real-time concurrency reservation.

---

# 140. CPU Budget

Usage charged after attempt based on measured/reserved policy.

---

# 141. Failed Jobs

Still consume compute usage.

---

# 142. Cancelled Jobs

Charge actual/reserved time consumed.

---

# 143. Cache Hit

Consumes little/no runner compute but may consume CAS/API bandwidth.

---

# 144. Retry

Each attempt consumes usage.

---

# 145. Infrastructure Failure

Policy may choose whether customer/tenant usage accounting excludes provider-caused infra waste.

---

# 146. Governance vs Billing

Governance can record both raw usage and billable-adjusted usage separately later.

---

# 147. Usage Classification

```rust
pub enum UsageClass {
    UserWork,
    Retry,
    InfrastructureWaste,
    SystemMaintenance,
}
```

---

# 148. System Maintenance

Not charged to tenant quota unless explicitly configured.

---

# 149. Overage State

```rust
pub enum OverageState {
    WithinLimit,
    NearLimit,
    SoftExceeded,
    HardExceeded,
}
```

---

# 150. Near-Limit Threshold

Example:

```text
80%
90%
```

configurable.

---

# 151. Notification

Warn tenant/admin.

---

# 152. Hard Exceeded

Block new resource allocations.

---

# 153. Do Not Kill Existing Jobs by Default

Hard quota crossing should generally block new work, not terminate already-running valid work.

---

# 154. Emergency Protection

Global system overload policy may cancel/preempt lower-priority work if explicitly supported later.

---

# 155. Storage Hard Quota

Cannot permit unbounded active uploads.

---

# 156. Upload Preflight

Reserve expected size.

---

# 157. Unknown Size Upload

Use maximum bound/chunk accounting.

---

# 158. Finalize

Adjust reservation to actual size.

---

# 159. Failed Upload

Release reservation.

---

# 160. Artifact GC

Quota-aware but retention-safe.

---

# 161. Tenant Suspension

```text
Active -> Suspended
```

---

# 162. Suspended Tenant

Recommended:

```text
read access maybe allowed by policy
no new runs
no release/deploy
no uploads
```

---

# 163. ReadOnly Tenant

Explicit.

---

# 164. Closing Tenant

Deletion/export lifecycle.

---

# 165. Tenant Deletion

High-risk, separate retention/legal workflow.

---

# 166. No Immediate CAS Erase

Respect shared physical dedup and retention.

Remove tenant references/keys according to policy.

---

# 167. Crypto-Erasure

If tenant-specific encryption keys used, possible high-assurance deletion mechanism.

---

# 168. Data Export

Enterprise portability.

---

# 169. Export Scope

```text
metadata
artifacts
audit references
config
```

subject to policy.

---

# 170. Secrets Export

Not by default.

---

# 171. Tenant Move

Project transfer between tenants is complex.

---

# 172. Recommendation

Do not support arbitrary tenant transfer initially.

Use export/import or explicit migration tool.

---

# 173. Project Move Within Tenant

Simpler.

---

# 174. Shared Resources

Examples:

```text
runner pool
device pool
plugin installation
SCM installation
```

---

# 175. Share Grant

Explicit ACL/policy.

---

# 176. Shared Runner Pool

Does not share secrets/data.

---

# 177. Shared Device Pool

Requires sanitization.

---

# 178. Shared SCM Installation

Repository-specific access.

---

# 179. Shared Plugin Installation

Config/state still tenant scoped.

---

# 180. Usage Attribution

Every scarce operation carries tenant context.

---

# 181. Run Attribution

Run -> Project -> Tenant.

---

# 182. RBE Attribution

Authenticated instance mapping -> Tenant.

---

# 183. Device Attribution

JobLease -> Tenant.

---

# 184. API Attribution

Principal/session + resource scope.

---

# 185. Release/Deploy Attribution

Project/environment.

---

# 186. Audit Attribution

Tenant where applicable.

---

# 187. Metering Event

```rust
pub struct UsageObservation {
    pub tenant: TenantId,
    pub resource: UsageResource,
    pub amount: UsageAmount,
    pub occurred_at: Timestamp,
    pub idempotency_key: UsageObservationId,
}
```

---

# 188. Metering Pipeline

```text
domain event/state transition
  ↓
usage observation
  ↓
durable outbox
  ↓
metering consumer
  ↓
usage record/rollup
```

---

# 189. Reconciliation

Usage reconciler can recompute from authoritative jobs/artifacts for bounded windows.

---

# 190. Drift Detection

Compare:

```text
quota reservation rows
active JobLeases
active DeviceLeases
storage references
```

---

# 191. Quota Drift

Repair stale counters.

---

# 192. Counter Design

Prefer reservation rows/derived counts over opaque mutable counters where practical.

---

# 193. Cached Counter

Optimization.

---

# 194. Authority

Reservation/entity rows.

---

# 195. Rate Limiting

API rate limiter and quota service coordinate but remain distinct.

---

# 196. Rate Limit

Short-window protection.

---

# 197. Quota

Longer business/resource governance.

---

# 198. Example

```text
100 requests/sec
+
1M requests/month
```

different mechanisms.

---

# 199. Scheduler Queue Quota

Maximum queued jobs per tenant/project.

---

# 200. Why

Prevent millions of pending jobs exhausting metadata/scheduler.

---

# 201. Matrix Explosion Guard

Pipeline planner already bounds matrix.

Tenant quota adds global queued-run/job bounds.

---

# 202. Maximum Active Runs

Optional.

---

# 203. Maximum Pipeline Size

Resource governance.

---

# 204. Maximum Artifact Count

Bound metadata pressure.

---

# 205. Maximum Secrets

Optional administrative quota.

---

# 206. Maximum Plugins

Optional.

---

# 207. Maximum SCM Bindings

Optional.

---

# 208. Quota Profiles

```rust
pub struct QuotaProfileId(BoundedString);
```

---

# 209. Example Profiles

```text
standalone
team
enterprise
high-assurance
```

---

# 210. Hosted Plans

Could map commercial plans externally, but core only sees effective quota profile.

---

# 211. Plan Name Is Not Authorization

Critical.

---

# 212. Effective Quota Snapshot

```rust
pub struct EffectiveQuotaSnapshot {
    pub scope: ResourceScope,
    pub version: QuotaPolicyVersion,
    pub entries: Vec<EffectiveQuota>,
}
```

---

# 213. Quota Policy Version

Immutable/versioned.

---

# 214. Running Job

Binds quota snapshot only for reservation decision; later reductions do not invalidate current lease by default.

---

# 215. Quota Change

Affects new allocations.

---

# 216. Emergency Quota Reduction

Explicit option to drain/cancel later, not baseline.

---

# 217. Admin Override

```rust
pub struct QuotaOverride {
    pub scope: ResourceScope,
    pub resource: QuotaResource,
    pub temporary_limit: QuotaLimit,
    pub expires_at: Timestamp,
    pub reason: BoundedString,
}
```

---

# 218. Override

Audited.

---

# 219. No Generic Unlimited Forever

Require explicit policy.

---

# 220. Temporary Burst Override

Good for release windows.

---

# 221. Break-Glass

Can override quota only with separate permission.

---

# 222. Cannot Bypass Security

Quota override never bypasses authz/trust/sandbox.

---

# 223. Fairness and Reserved Capacity

Reserved capacity can increase scheduler weight/access within limits.

---

# 224. Priority Classes

```rust
pub enum SchedulingClass {
    Background,
    Normal,
    Interactive,
    ReleaseCritical,
}
```

---

# 225. Class Assignment

Policy/service controlled.

---

# 226. User Request

Can request class but server clamps/authorizes.

---

# 227. ReleaseCritical

Only trusted release pipeline.

---

# 228. Starvation

Aging ensures Background eventually progresses.

---

# 229. DRF Later

For CPU/GPU/device multi-resource fairness.

---

# 230. Device Fairness

Rare device pools can use separate weighted queues.

---

# 231. RBE Fairness

Shares same scheduler.

---

# 232. Hosted RBE Instance

Cannot bypass tenant weight via direct REAPI.

---

# 233. Plugin Resource Governance

Supervisor enforces per-plugin CPU/memory/requests.

---

# 234. Provider Budget

SCM/API quota can prioritize final checks/integration over cosmetic comments.

---

# 235. Tenant Provider Budget

Avoid one tenant consuming global GitHub rate limit.

---

# 236. Provider Installation Budget

Track per installation.

---

# 237. Signing Rate

Protect HSM/KMS.

---

# 238. Deployment Target Rate

Avoid cloud API storm.

---

# 239. Global System Limits

Separate from tenant quota.

---

# 240. Global Limit Examples

```text
max DB pool
max CAS bandwidth
max active jobs
max agent connections
```

---

# 241. Global Protection

System can reject all tenants proportionally/fairly under overload.

---

# 242. Admission Control

Before resource-intensive work.

---

# 243. Admission Decision

```rust
pub struct AdmissionDecision {
    pub quota: QuotaDecision,
    pub policy: PolicyDecisionSummary,
    pub capacity: CapacityDecision,
}
```

---

# 244. Admission Order

Recommended:

```text
validate
authz/policy
quota
capacity/scheduler
```

---

# 245. Do Not Reserve Before Authz

Avoid abuse.

---

# 246. Quota Error

Stable code.

---

# 247. API Response

Examples:

```text
QUOTA_EXCEEDED
CONCURRENCY_LIMIT
STORAGE_LIMIT
DEVICE_QUOTA_EXCEEDED
```

---

# 248. HTTP Status

Usually:

```text
429
```

for rate/concurrency, or

```text
403/409/422
```

depending semantics.

Keep stable API code primary.

---

# 249. Retryability

Quota response includes:

```text
retryable
reset_at
current
limit
```

where safe.

---

# 250. UI

Governance pages:

```text
Usage
Quotas
Concurrency
Storage
Runner Capacity
Device Usage
RBE Usage
Overrides
```

---

# 251. Tenant Dashboard

Shows:

```text
current jobs
queue
CPU usage
CAS usage
device minutes
API usage
```

---

# 252. Project Dashboard

Scoped usage.

---

# 253. Quota Gauge

Show current/limit.

---

# 254. Near Limit

Warning.

---

# 255. Hard Limit

Clear blocked action reason.

---

# 256. Admin Quota Editor

Permission-gated.

---

# 257. Override Dialog

Requires:

```text
new limit
duration
reason
```

---

# 258. Usage Export

CSV/JSON via API.

---

# 259. No Sensitive Artifact Names

Usage reports can aggregate without exposing content to billing/admin systems unnecessarily.

---

# 260. API

Potential:

```text
GET  /v1/usage
GET  /v1/quotas
GET  /v1/quotas/effective
POST /v1/admin/quotas
POST /v1/admin/quota-overrides
DELETE /v1/admin/quota-overrides/{id}
```

---

# 261. Permissions

```text
usage.read
quota.read
quota.manage
quota.override
tenant.manage
```

---

# 262. Tenant Admin

Can manage child project budgets within delegated limits.

---

# 263. System Admin

Global.

---

# 264. Delegation

Parent can allocate sub-budgets.

---

# 265. Sub-Budget

Cannot exceed parent reservation.

---

# 266. Usage Privacy

Tenant sees only own usage.

---

# 267. System Operator

May see aggregates across tenants depending permission.

---

# 268. Metrics

```text
governance_quota_denied_total
governance_quota_near_limit
governance_active_reservations
governance_usage_lag
scheduler_tenant_queue_wait
scheduler_fairness_starvation
```

---

# 269. Metric Labels

Low cardinality:

```text
resource_type
enforcement
result
```

---

# 270. TenantId Metric Label

Avoid on global Prometheus metrics.

Use logs/traces/admin usage DB.

---

# 271. Per-Tenant Dashboards

Query usage store, not metric labels.

---

# 272. Tracing

```text
quota.evaluate
quota.reserve
quota.release
usage.observe
usage.aggregate
fairness.select
```

---

# 273. Health

Governance health:

```text
usage lag
stale reservations
quota store
reconciliation
```

---

# 274. Doctor

```text
forgeyard governance doctor
```

---

# 275. Doctor Checks

```text
stale quota reservations
negative/invalid usage
tenant ownership gaps
cross-tenant reference violations
quota hierarchy cycles
```

---

# 276. Security Scan

Periodic tenant-isolation invariant checker.

---

# 277. Ownership Gap

Any resource without valid TenantId is critical in hosted mode.

---

# 278. Cross-Tenant Reference

Examples:

```text
Deployment in tenant A referencing Release tenant B
Job tenant A using Secret tenant B
```

reject.

---

# 279. Allowed Shared Reference

Only explicit shared/system resource types.

---

# 280. Typed SharedResourceRef

Avoid general cross-tenant foreign key.

---

# 281. Database Constraints

Use composite tenant/resource keys where practical.

---

# 282. Example

```text
(tenant_id, project_id)
```

foreign key.

---

# 283. Defense in Depth

App scope + DB constraints + RLS optional.

---

# 284. Tenant Encryption

Optional high-assurance.

---

# 285. Per-Tenant Encryption Key

Can encrypt sensitive metadata/artifacts.

---

# 286. Complexity

Not required baseline because CAS/object storage and DB encryption already exist.

---

# 287. Hosted High-Assurance

May use per-tenant KEK.

---

# 288. Crypto Erasure

Then possible.

---

# 289. Tenant Export/Closure

Separate lifecycle.

---

# 290. Closure Sequence

```text
suspend writes
export if requested
enforce retention
remove credentials/bindings
delete logical refs
GC eligible bytes later
```

---

# 291. Shared CAS Bytes

Physical blob remains if another tenant references.

---

# 292. Tenant Artifact Confidentiality

Authorization metadata remains separate from physical blob.

---

# 293. Signed URL

Always tenant-authorized before issue.

---

# 294. RBE Digest Guess

Cannot read blob without instance/tenant auth.

---

# 295. API ID Guess

Tenant isolation enforced.

---

# 296. Event Streams

Tenant filtered.

---

# 297. SSE

Never stream other tenant events.

---

# 298. Audit

Tenant scoped.

---

# 299. Plugins

Plugin callback/event projection tenant scoped.

---

# 300. SCM Webhook

Verified repository binding resolves tenant.

---

# 301. Device Pool Shared

Device sanitation plus policy.

---

# 302. Runner Shared

Sandbox isolation plus tenant-scoped secrets.

---

# 303. Runner Local Cache

Cross-tenant cache rules apply.

---

# 304. Workspace Cleanup

Mandatory.

---

# 305. Hostile Multi-Tenant Workload

Use VM-capable isolation for stronger boundary.

---

# 306. Tenant Trust Classes

Optional:

```rust
pub enum TenantIsolationClass {
    Cooperative,
    Standard,
    Hostile,
    Dedicated,
}
```

---

# 307. Cooperative

Internal teams.

---

# 308. Hostile

Untrusted external tenants.

---

# 309. Dedicated

Dedicated runner/device resources.

---

# 310. Scheduler Requirement

Isolation class maps to runner trust/isolation capability.

---

# 311. Hosted SaaS Default

Standard/hostile depending threat model.

---

# 312. Billing Readiness

Usage system should emit stable records suitable for future billing.

---

# 313. But No Money Types in Core Governance

Avoid mixing commercial pricing with resource truth.

---

# 314. Cost Attribution

```rust
pub struct CostAttributionRecord {
    pub tenant: TenantId,
    pub resource: UsageResource,
    pub quantity: UsageAmount,
    pub infrastructure_class: InfrastructureClass,
}
```

---

# 315. Actual Cost

Can be computed externally using provider pricing.

---

# 316. Cost Hints

Scheduler may use cost as soft score.

---

# 317. Tenant Budget

Optional monetary budget later, outside baseline.

---

# 318. Resource Budget First

Technically deterministic.

---

# 319. Quota Config RON

Example:

```ron
(
    tenant: "tenant-a",
    quotas: [
        (
            resource: ConcurrentJobs,
            limit: Count(50),
            enforcement: Hard,
        ),
        (
            resource: CasBytes,
            limit: Bytes(500000000000),
            enforcement: Hard,
        ),
    ],
)
```

---

# 320. Config vs DB

Enterprise dynamic quota changes stored as versioned metadata.

RON can seed defaults.

---

# 321. Default Quota

Safe finite defaults in hosted mode.

---

# 322. Self-Hosted Enterprise

Can choose Unlimited for trusted internal deployment.

---

# 323. Standalone

Quotas mostly disabled except safety bounds:

```text
max logs
max uploads
disk pressure
matrix size
```

---

# 324. Same Model

Standalone uses TenantId representing local installation/default tenant.

---

# 325. Avoid Branching Domain Logic

Single-tenant is special case of multi-tenant model.

---

# 326. Testkit

```text
forgeyard-governance-testkit/src/
├── lib.rs
├── tenant.rs
├── quota.rs
├── usage.rs
├── reservation.rs
├── fairness.rs
├── isolation.rs
└── assertions.rs
```

---

# 327. Unit Tests

Quota hierarchy/effective limit.

---

# 328. Tenant Isolation Test

Tenant A cannot read Tenant B Run.

---

# 329. Secret Isolation Test

Tenant A Job cannot resolve Tenant B SecretRef.

---

# 330. CAS Isolation Test

Knowing digest does not grant Tenant B artifact.

---

# 331. RBE Isolation Test

Instance A cannot read instance B CAS/cache.

---

# 332. SSE Isolation Test

No cross-tenant events.

---

# 333. Scheduler Fairness Test

Heavy tenant cannot starve small tenant indefinitely.

---

# 334. Hard Concurrency Test

Parallel scheduler attempts never exceed tenant limit.

---

# 335. HA Concurrency Test

Leader failover does not leak quota reservation.

---

# 336. Reservation Expiry Test

Stale reservation reclaimed.

---

# 337. Retry Accounting Test

Attempts counted correctly.

---

# 338. Cache Hit Accounting Test

No runner compute usage.

---

# 339. Storage Accounting Test

Shared physical blob charged logically per tenant.

---

# 340. Protected Artifact Test

Quota enforcement never deletes release-critical artifact.

---

# 341. Device Quota Test

DeviceLease blocked beyond limit.

---

# 342. RBE Priority Test

RBE cannot bypass tenant scheduling class.

---

# 343. Plugin Quota Test

Plugin resources bounded.

---

# 344. Suspended Tenant Test

New work denied; configured reads remain.

---

# 345. Quota Override Test

Temporary, scoped, audited, expires.

---

# 346. Usage Dedup Test

Duplicate event does not double-charge.

---

# 347. Usage Reconcile Test

Metering lag repaired.

---

# 348. DB Constraint Test

Cross-tenant foreign reference rejected where modeled.

---

# 349. Fuzzing

Fuzz quota config/hierarchy/cursor/usage decoders.

---

# 350. Property Tests

Effective quota never exceeds hard parent.

---

# 351. Chaos Test

Job completion lost/replayed -> reservation eventually correct.

---

# 352. Load Test

Thousands of tenants and large queues.

---

# 353. Fairness Scale

Measure queue latency distribution.

---

# 354. Noisy Neighbor Test

One tenant maxes CAS/API/compute while others remain usable.

---

# 355. Implementation Phase 1 — Tenant Ownership Model

Add explicit TenantId propagation/store constraints.

---

# 356. Phase 2 — API/Authz Isolation

End-to-end tenant scoping.

---

# 357. Phase 3 — Scheduler Concurrency Quotas

Hard reservations.

---

# 358. Phase 4 — CAS/Storage Quotas

Logical accounting.

---

# 359. Phase 5 — Usage Metering

Durable observations/rollups.

---

# 360. Phase 6 — Fairness

Weighted queues/aging.

---

# 361. Phase 7 — Device/RBE/Plugin Quotas

All alternate execution surfaces.

---

# 362. Phase 8 — Admin/UI

Usage/quota dashboards.

---

# 363. Phase 9 — Isolation Hardening

RLS/composite constraints/invariant scans.

---

# 364. Phase 10 — Cost Attribution

Infrastructure classes.

---

# 365. Phase 11 — Tenant Lifecycle

Suspend/export/close.

---

# 366. Phase 12 — Scale/Chaos

Hosted production hardening.

---

# 367. Acceptance Tests

1. Every Project belongs to exactly one TenantId.
2. Every Run/Job/Release/Deployment resolves to one TenantId.
3. Tenant IDs are stable and not name-based authority.
4. API clients cannot select arbitrary tenant by header alone.
5. Store queries are tenant scoped.
6. Tenant A cannot read Tenant B metadata.
7. Tenant A cannot use Tenant B SecretRef.
8. Knowing another tenant's CAS digest does not grant access.
9. Cross-tenant Action Cache is disabled by default.
10. Shared physical CAS dedup does not create logical authorization sharing.
11. Hard concurrent-job quota cannot be exceeded under scheduler races.
12. HA failover does not leak quota reservations.
13. Quota reservation is released after job termination.
14. Scheduler fairness prevents indefinite starvation.
15. User cannot self-elevate scheduler weight/priority.
16. Dedicated runner pools remain tenant restricted.
17. Storage quota uses defined logical accounting.
18. Release-critical artifacts are never deleted merely to satisfy cache quota.
19. Upload reservation prevents exceeding storage quota.
20. DeviceLease honors tenant device quota.
21. RBE execution obeys same tenant quotas/fairness.
22. Plugin execution obeys plugin/resource limits.
23. Provider/API budgets prevent one tenant exhausting shared integration rate limits.
24. Usage events are idempotent.
25. Metering reconciliation repairs dropped/duplicate observations.
26. Metrics are not quota authority.
27. Hard quota normally blocks new allocations instead of killing valid running jobs.
28. Temporary overrides are scoped, expiring, and audited.
29. Tenant suspension blocks new protected work.
30. Tenant closure respects retention/shared-CAS semantics.
31. Cross-tenant foreign references are rejected except explicit shared resource types.
32. Standalone mode uses same model with simplified/default quotas.
33. High-assurance deployments can disable cross-tenant physical dedup.
34. No alternate protocol—RBE, plugin, device, SCM—bypasses governance.
35. Forgeyard's own hosted deployment can report and govern its tenant resource usage.

---

# 368. Production Readiness Gates

Do not call hosted multi-tenancy production-ready until:

```text
TenantId ownership propagated through all major entities
API/store isolation tests pass
secret isolation passes
CAS authorization-by-reference passes
scheduler hard concurrency quota is race-safe
fairness/noisy-neighbor tests pass
RBE/device/plugin alternate paths obey quotas
usage metering is idempotent/reconcilable
storage accounting/retention behavior is proven
tenant suspension/override/audit flows work
```

---

# 369. Architectural Invariants

1. every business resource belongs to an explicit tenant;
2. tenant name is never security authority;
3. tenant context is validated server-side;
4. store access is tenant-scoped;
5. authz remains mandatory even with DB RLS;
6. secrets never cross tenant scope;
7. knowing CAS digest never grants access;
8. physical dedup never implies logical sharing;
9. cross-tenant cache is off by default;
10. quotas and scheduler fairness are separate concepts;
11. hard quotas use authoritative reservations/state;
12. metrics are never quota authority;
13. quota races cannot oversubscribe hard limits;
14. stale reservations are reconciled;
15. existing valid jobs are not normally killed by later quota exhaustion;
16. protected release artifacts are never casually GC'd for quota;
17. device/RBE/plugin/provider paths obey the same governance;
18. users cannot self-assign priority/weight;
19. dedicated pools remain scoped;
20. usage observations are idempotent;
21. usage can be reconciled from authoritative state;
22. admin overrides are explicit/temporary/audited;
23. tenant suspension is explicit state;
24. tenant closure preserves retention/shared-object correctness;
25. global system protection is separate from per-tenant quota;
26. hosted hostile tenants can require stronger runner isolation;
27. billing may consume usage data but does not define resource truth;
28. standalone is a one-tenant specialization of same model;
29. HA failover preserves quota correctness;
30. Forgeyard dogfoods resource governance in hosted deployments.

---

# 370. Final Target Architecture

```text
                         Tenant
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
             Org         Project    Environment
               │           │           │
               └───────────┼───────────┘
                           ▼
                    Effective Scope
                           │
           ┌───────────────┼────────────────┐
           ▼               ▼                ▼
        Authz           Quotas          Fairness
           │               │                │
           └───────────────┼────────────────┘
                           ▼
                     Admission Control
                           │
          ┌────────────────┼─────────────────┐
          ▼                ▼                 ▼
      Scheduler           CAS/API        Device/RBE
          │                │                 │
          └────────────────┼─────────────────┘
                           ▼
                      Usage Records
                           │
                           ▼
                 Governance / Reporting
```

---

# 371. Final Architectural Position

Tenant isolation:

```text
Principal
+
Resource
+
TenantId
  ↓
Authz
  ↓
tenant-scoped store/CAS/secret access
```

Quota admission:

```text
requested resource
+
effective hierarchical quota
+
current authoritative reservations
  ↓
Allowed / Warning / Denied
```

Scheduler fairness:

```text
eligible tenants
+
weights
+
aging
+
scarcity
+
capacity
  ↓
next placement
```

Usage:

```text
JobAttempt / DeviceLease / Artifact / RBE / API
  ↓
idempotent usage observation
  ↓
rollup
  ↓
quota/reporting/cost attribution
```

The key guarantee is:

> **Forgeyard can safely host many independent teams or customers on shared infrastructure without confusing sharing of hardware with sharing of authority. Tenant ownership is explicit, quotas bound consumption, scheduler fairness prevents starvation, shared data planes remain access-controlled, and every alternate execution path is governed by the same isolation rules.**

---

# 372. Extended Architecture Sequence

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
```
