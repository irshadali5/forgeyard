# 45 — Forgeyard Cost Accounting, FinOps, Chargeback/Showback & Resource Economics System Architecture

**Document type:** Core Cost Attribution, FinOps, Budgeting, Showback/Chargeback, Forecasting & Resource Economics System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** raw resource metering, cost attribution, provider pricing, internal cost models, tenant/project/workload allocation, budgets, forecasts, anomalies, showback, chargeback, unit economics, cost-aware optimization inputs, storage/network/device cost, release/deployment cost, plugin/external-service cost, and financial reporting  
**Architecture style:** Raw usage first, pricing second, immutable attribution facts, explainable cost allocation, provider-neutral normalization, strong separation from billing authority, no cost-based correctness downgrade, and policy-governed optimization  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Multi-Tenancy/Quotas, Entitlements/Billing, Runner Fleet Autoscaling, Scheduler, CAS, Observability, Device Lab, Deployment, Search/Analytics, Dependency Governance, Notifications, and Audit. This subsystem explains what Forgeyard costs to operate and where that cost comes from without turning accounting into execution authority.

---

# 1. Purpose

Forgeyard can consume substantial resources:

```text
CPU
memory
GPU
runner hours
macOS hosts
device lab time
CAS/object storage
logs
network egress
database usage
external scanners
SCM/provider API usage
release distribution
deployment infrastructure
```

Operators and organizations need answers such as:

```text
what did this project cost this month?
which pipelines consume the most compute?
how much does a release cost?
how much does a cache hit save?
which runner pools are underutilized?
which tenant causes most egress?
what is the cost per successful build?
what will next month likely cost?
```

The central rule is:

> **Forgeyard records raw resource usage independently from price. Cost is a derived, versioned interpretation of immutable usage facts.**

A second rule is:

> **Cost accounting and customer billing are separate domains. Cost facts may inform billing, but Forgeyard must never confuse provider cost, internal transfer price, subscription price, or invoice amount.**

A third rule is:

> **Cost-aware scheduling or autoscaling may choose among semantically equivalent eligible options, but cost can never justify weaker isolation, weaker trust, missing tests, stale evidence, or policy bypass.**

---

# 2. Architectural Position

```text
                   Resource Consumption
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
     Runners          CAS           Network
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                  Usage Records
                       │
                       ▼
                Attribution Engine
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   Provider Cost   Internal Cost   Transfer Price
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                   Cost Facts
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
          Showback   Budget   Optimization
```

---

# 3. Goals

The subsystem MUST:

1. normalize raw resource usage;
2. attribute usage to tenant/project/run/job;
3. attribute storage consumption;
4. attribute network consumption;
5. attribute runner time;
6. attribute GPU/device usage;
7. attribute external-service usage;
8. support provider pricing snapshots;
9. support internal cost models;
10. support cost allocation;
11. support shared-cost allocation;
12. support showback;
13. support chargeback exports;
14. support budgets;
15. support forecast;
16. support anomaly detection;
17. support unit economics;
18. support cost-aware optimization inputs;
19. support historical pricing;
20. support pricing corrections;
21. support multi-currency normalization;
22. support tenant isolation;
23. support API/UI/CLI;
24. support notifications;
25. support audit for admin changes;
26. support standalone mode;
27. support distributed mode;
28. support reconciliation;
29. support DR;
30. remain separate from invoice authority.

---

# 4. Non-Goals

This subsystem does not:

```text
replace accounting software
issue invoices by itself
define subscription plans
become tax/accounting ledger authority
override scheduler hard constraints
replace provider invoices
```

---

# 5. Workspace Structure

```text
crates/cost/
├── forgeyard-cost/
├── forgeyard-cost-model/
├── forgeyard-cost-metering/
├── forgeyard-cost-attribution/
├── forgeyard-cost-pricing/
├── forgeyard-cost-allocation/
├── forgeyard-cost-budget/
├── forgeyard-cost-forecast/
├── forgeyard-cost-anomaly/
├── forgeyard-cost-unit-economics/
├── forgeyard-cost-export/
├── forgeyard-cost-health/
└── forgeyard-cost-testkit/
```

Provider pricing adapters:

```text
crates/cost-providers/
├── forgeyard-cost-aws/
├── forgeyard-cost-azure/
├── forgeyard-cost-gcp/
├── forgeyard-cost-neon/
├── forgeyard-cost-object-store/
├── forgeyard-cost-custom/
└── ...
```

Use modules first; split only where provider SDK dependencies justify.

---

# 6. Raw Usage Principle

Record usage independently from price.

Example:

```text
Runner CPUSeconds = 420
MemoryByteSeconds = ...
GpuSeconds = ...
```

not:

```text
cost = $0.37
```

as primary fact.

---

# 7. UsageRecordId

```rust
pub struct UsageRecordId(Ulid);
```

---

# 8. Usage Record

```rust
pub struct UsageRecord {
    pub id: UsageRecordId,
    pub tenant: TenantId,
    pub project: Option<ProjectId>,
    pub subject: UsageSubject,
    pub dimension: UsageDimension,
    pub quantity: UsageQuantity,
    pub occurred_at: TimeRange,
    pub source: UsageSource,
}
```

---

# 9. Usage Subject

```rust
pub enum UsageSubject {
    Run(RunId),
    Job(JobId),
    Attempt(JobAttemptId),
    Runner(RunnerId),
    Device(DeviceId),
    Artifact(ArtifactId),
    Deployment(DeploymentId),
    Release(ReleaseId),
    Tenant(TenantId),
    System,
}
```

---

# 10. Usage Dimension

```rust
pub enum UsageDimension {
    CpuSeconds,
    MemoryByteSeconds,
    GpuSeconds,
    RunnerSeconds,
    DeviceSeconds,
    StorageByteHours,
    LogByteHours,
    NetworkEgressBytes,
    NetworkIngressBytes,
    ApiRequests,
    ProviderOperations,
    ExternalScannerSeconds,
    DatabaseComputeSeconds,
    Custom(UsageDimensionId),
}
```

---

# 11. Usage Quantity

Strongly typed.

---

# 12. No Raw Floating Point for Money

Critical.

---

# 13. Decimal Money

Use decimal/fixed precision.

---

# 14. Currency

```rust
pub struct Money {
    pub amount: Decimal,
    pub currency: CurrencyCode,
}
```

---

# 15. PricingSnapshotId

```rust
pub struct PricingSnapshotId(Digest);
```

---

# 16. Pricing Snapshot

Immutable versioned pricing data.

---

# 17. Pricing Source

```rust
pub enum PricingSource {
    ProviderPublished,
    ContractedRate,
    InternalRateCard,
    ManualAdministrative,
}
```

---

# 18. Provider Published Price

May differ from actual invoice.

---

# 19. Contracted Rate

Organization-specific.

---

# 20. Internal Rate Card

For transfer/showback.

---

# 21. Cost Interpretation

```rust
pub struct CostInterpretation {
    pub usage: UsageRecordId,
    pub pricing: PricingSnapshotId,
    pub amount: Money,
    pub model: CostModelId,
}
```

---

# 22. Historical Stability

If provider pricing changes tomorrow, yesterday's historical cost interpretation stays tied to old snapshot.

---

# 23. Repricing

Can create new interpretation, never rewrite raw usage.

---

# 24. Cost Model

```rust
pub enum CostModel {
    DirectProvider,
    InternalFullyLoaded,
    TransferPrice,
    Custom(CostModelId),
}
```

---

# 25. DirectProvider

Estimated/raw provider cost.

---

# 26. FullyLoaded

Can include:

```text
compute
storage
network
reserved capacity
shared control plane allocation
```

---

# 27. Transfer Price

Internal organizational accounting.

---

# 28. Customer Price

Not here.

Part 30 billing/entitlement domain.

---

# 29. Usage Attribution

Every metered activity should map to:

```text
tenant
project
run/job if applicable
resource class
```

---

# 30. Runner Compute Attribution

Use actual attempt occupancy.

---

# 31. Example

If job runs 60 seconds on 4 vCPU capacity:

record based on chosen accounting semantics:

```text
reserved vCPU-seconds
or
measured CPU-seconds
```

---

# 32. Reserved vs Actual

Keep separate dimensions.

---

# 33. Why

Reserved resources reflect capacity cost.

Actual CPU reflects workload efficiency.

---

# 34. Runner Usage Class

```rust
pub enum ComputeUsageClass {
    Reserved,
    Measured,
}
```

---

# 35. Job Queue Time

Not compute cost.

---

# 36. Idle Runner

Fleet/system cost.

---

# 37. Idle Cost Attribution

Options:

```text
fleet overhead
tenant dedicated pool
organization shared cost
```

---

# 38. Allocation Policy

Versioned.

---

# 39. SharedCostAllocationId

```rust
pub struct SharedCostAllocationId(Digest);
```

---

# 40. Shared Cost Examples

```text
idle warm pool
control plane
shared DB
shared cache
```

---

# 41. Allocation Methods

```rust
pub enum SharedCostAllocationMethod {
    Direct,
    Equal,
    ProportionalUsage,
    Weighted,
    Unallocated,
}
```

---

# 42. Unallocated

Valid and honest.

---

# 43. Do Not Force False Precision

Critical.

---

# 44. CAS Storage Cost

Raw facts:

```text
logical referenced bytes
physical stored bytes
retention duration
```

---

# 45. Tenant Accounting

Use logical referenced bytes for fair attribution when physical CAS dedup exists.

---

# 46. Provider Cost

Physical storage bytes.

---

# 47. Distinction

Important.

---

# 48. Cross-Tenant Dedup

Can reduce provider cost without reducing tenant logical usage.

---

# 49. Cache Storage

Separate from durable artifact storage.

---

# 50. Cache Eviction Savings

Derived estimate.

---

# 51. Logs

Meter retained log bytes/time.

---

# 52. Network

Attribute egress to:

```text
artifact download
runner CAS transfer
release download
provider traffic
```

where measurable.

---

# 53. Ingress

Often free provider-side, but still raw usage.

---

# 54. Device Cost

Physical device lab:

```text
device minutes
host runner minutes
external lab fees
```

---

# 55. GPU

GPU seconds/capacity hours.

---

# 56. macOS Capacity

Often high fixed cost.

Track dedicated host idle/used separately.

---

# 57. External Services

Examples:

```text
security scanner
email provider
SMS/push
artifact signing/KMS
SCM premium API
device farm
```

---

# 58. ExternalServiceUsage

```rust
pub struct ExternalServiceUsage {
    pub provider: ExternalServiceId,
    pub operation: ExternalOperationKind,
    pub quantity: UsageQuantity,
}
```

---

# 59. Provider API Request Cost

Only if financially material.

---

# 60. Database Cost

Neon/Postgres provider usage can be allocated.

---

# 61. Database Shared Cost

Usually not precisely attributable per query.

---

# 62. Allocation

Use:

```text
tenant storage
transaction count
active usage
or unallocated shared
```

with explicit method.

---

# 63. Cost Attribution Confidence

```rust
pub enum AttributionConfidence {
    Exact,
    Measured,
    Estimated,
    Allocated,
    Unknown,
}
```

---

# 64. Every Cost Fact

Carries confidence.

---

# 65. Exact

Provider line item or exact direct rate.

---

# 66. Measured

Usage measured, price known.

---

# 67. Estimated

Approximation.

---

# 68. Allocated

Shared cost apportioned.

---

# 69. Unknown

Do not fabricate.

---

# 70. CostFact

```rust
pub struct CostFact {
    pub id: CostFactId,
    pub subject: CostSubject,
    pub period: TimeRange,
    pub amount: Money,
    pub confidence: AttributionConfidence,
    pub pricing: PricingSnapshotId,
}
```

---

# 71. CostFactId

Content-derived or stable immutable.

---

# 72. Period

Hour/day/month etc.

---

# 73. Rollups

Derived:

```text
hourly
daily
monthly
```

---

# 74. Raw Usage Retained

So rollups can rebuild.

---

# 75. Cost Corrections

Provider bills may differ.

---

# 76. Correction

Append adjustment.

---

# 77. Never Rewrite Historical Usage

Critical.

---

# 78. CostAdjustment

```rust
pub struct CostAdjustment {
    pub original: CostFactId,
    pub delta: Money,
    pub reason: CostAdjustmentReason,
}
```

---

# 79. Provider Invoice Reconciliation

Optional.

---

# 80. Import Provider Invoice

Can compare expected vs billed.

---

# 81. Billing Provider Connector

Separate adapter.

---

# 82. Provider Invoice Is External Source

Not execution authority.

---

# 83. Showback

Displays cost without internal financial transfer.

---

# 84. Chargeback

Exports allocated cost for internal accounting.

---

# 85. ChargebackExportId

```rust
pub struct ChargebackExportId(Digest);
```

---

# 86. Export Formats

```text
CSV
JSON
RON
```

---

# 87. Accounting Integration

Future plugin/export adapter.

---

# 88. No General Ledger Built Into Core

Critical.

---

# 89. Budget

```rust
pub struct CostBudget {
    pub id: CostBudgetId,
    pub scope: ResourceScope,
    pub period: BudgetPeriod,
    pub amount: Money,
    pub action: BudgetAction,
}
```

---

# 90. Budget Period

```text
daily
monthly
quarterly
custom
```

---

# 91. Budget Action

```rust
pub enum BudgetAction {
    Notify,
    RequireApproval,
    LimitOptionalCapacity,
    HardBlockNewOptionalWork,
}
```

---

# 92. Security-Critical Work

Should not be silently disabled by budget.

---

# 93. Hard Budget

Policy-defined.

---

# 94. Example

Block new non-production benchmark jobs after budget.

---

# 95. Existing Running Jobs

Normally continue.

---

# 96. Budget vs Quota

Quota:

```text
resource usage limit
```

Budget:

```text
financial limit
```

---

# 97. No Duplicate Quota Engine

Budget can feed governance/policy.

---

# 98. Cost Guardrail

Part 43 fleet cost guardrail.

---

# 99. Integration

Part 45 produces cost estimate/pricing inputs.

---

# 100. Budget Utilization

```rust
pub struct BudgetUtilization {
    pub budget: CostBudgetId,
    pub spent: Money,
    pub forecast: Option<Money>,
}
```

---

# 101. Forecast

Derived.

---

# 102. Forecast Model

Baseline simple:

```text
run rate
historical seasonality
known reservations
```

---

# 103. ForecastModelVersion

Versioned.

---

# 104. Forecast Confidence

Explicit.

---

# 105. No Guaranteed Forecast

Critical.

---

# 106. Cost Anomaly

Detect sudden deviations.

---

# 107. CostAnomalyId

```rust
pub struct CostAnomalyId(Ulid);
```

---

# 108. Anomaly Inputs

```text
historical baseline
current usage
pricing changes
resource class
```

---

# 109. Baseline

Deterministic/statistical.

---

# 110. No ML Required

---

# 111. Anomaly Confidence

Explicit.

---

# 112. Common Anomalies

```text
runaway runner fleet
unexpected egress
log explosion
cache storage spike
GPU overuse
device farm surge
```

---

# 113. Notification

Part 29.

---

# 114. Kill/Scale Response

Never automatic baseline for expensive but valid work.

---

# 115. Policy

Can automate safe responses, e.g.:

```text
freeze optional fleet scale-up
```

---

# 116. Unit Economics

Examples:

```text
cost per successful Run
cost per build minute
cost per release
cost per deployment
cost per active project
cost per tenant
```

---

# 117. UnitMetricId

```rust
pub struct UnitMetricId(BoundedString);
```

---

# 118. Unit Metric Formula

Versioned.

---

# 119. Denominator

Explicit.

---

# 120. Example

```text
monthly CI cost / successful Runs
```

---

# 121. Failed Runs

Can be separate.

---

# 122. Retry Waste

Cost attributed to actual usage.

---

# 123. Infrastructure Waste

Existing Part 27 `InfrastructureWaste`.

---

# 124. Cost Classification

```rust
pub enum CostUsageClass {
    UserWork,
    Retry,
    InfrastructureWaste,
    SystemMaintenance,
    IdleCapacity,
}
```

---

# 125. Useful for Optimization

---

# 126. Cache Savings

Estimate:

```text
counterfactual execution cost
```

---

# 127. Counterfactual

Must be labeled estimated.

---

# 128. No Claim Exact Savings

Critical.

---

# 129. Monorepo Incremental Savings

Likewise estimated.

---

# 130. Fleet Optimization

Cost facts can inform:

```text
warm pool size
spot mix
region choice
```

---

# 131. Hard Constraint

Only among eligible equivalent options.

---

# 132. Scheduler Cost Signal

```rust
pub struct CostPreference {
    pub relative_cost: Decimal,
    pub snapshot: PricingSnapshotId,
}
```

---

# 133. Scheduler

Soft score only.

---

# 134. Trust/Security First

Hard filter before cost.

---

# 135. Autoscaler

Can compare eligible fleets by cost.

---

# 136. Cost Staleness

Pricing snapshot age.

---

# 137. Stale Price

Can use last-known for estimates, marked stale.

---

# 138. Never Block Correctness Because Price API Down

Critical.

---

# 139. Provider Pricing Fetch

External asynchronous process.

---

# 140. Web Search Not Runtime Requirement

Provider APIs/files/config preferred.

---

# 141. Pricing Reconciler

Refresh snapshots.

---

# 142. Manual Contract Pricing

Admin input.

---

# 143. Sensitive Contract Rates

Restricted access.

---

# 144. Cost Permissions

```text
cost.read
cost.admin
budget.manage
pricing.manage
chargeback.export
```

---

# 145. Tenant View

Own cost only.

---

# 146. System Admin

Cross-tenant aggregated.

---

# 147. Confidentiality

Cost data may reveal infrastructure/use patterns.

---

# 148. API

Potential:

```text
GET  /v1/cost/summary
GET  /v1/cost/by-project
GET  /v1/cost/by-run
GET  /v1/cost/budgets
POST /v1/cost/budgets
GET  /v1/cost/forecast
GET  /v1/cost/anomalies
```

---

# 149. Query Scope

Tenant-scoped/authz.

---

# 150. Dioxus UI

Pages:

```text
Cost Overview
Projects
Runners
Storage
Network
Budgets
Forecast
Anomalies
Unit Economics
```

---

# 151. Run Detail

Can show estimated/actual resource cost.

---

# 152. Job Detail

Compute cost.

---

# 153. Release Detail

Aggregate:

```text
build
test
package
sign
distribution
```

---

# 154. Deployment Detail

Deployment-control cost may be small; target infrastructure cost may come from external provider integration.

---

# 155. Confidence Badge

Always visible.

---

# 156. Currency

Display selected/local reporting currency.

---

# 157. Multi-Currency

Store source currency + conversion snapshot.

---

# 158. FxRateSnapshotId

```rust
pub struct FxRateSnapshotId(Digest);
```

---

# 159. FX

Derived reporting only.

---

# 160. Historical FX

Immutable snapshot.

---

# 161. No Live FX for historical rewrite

Critical.

---

# 162. Base Reporting Currency

Configurable.

---

# 163. Cost Export

Include:

```text
raw usage refs
pricing snapshot
allocation method
confidence
```

for auditability.

---

# 164. Search/Analytics

Part 31 may power query UX, but canonical cost facts remain Part 45.

---

# 165. Cost Analytics

Materialized rollups.

---

# 166. Observability Metrics

```text
cost_usage_records_total
cost_attribution_failures_total
cost_pricing_snapshot_age_seconds
cost_budget_exceeded_total
cost_anomalies_open
```

---

# 167. Avoid Monetary Values as high-cardinality metrics.

---

# 168. Tracing

```text
cost.meter
cost.attribute
cost.price
cost.rollup
cost.forecast
cost.reconcile
```

---

# 169. Health

Checks:

```text
metering lag
pricing freshness
allocation backlog
provider reconciliation
```

---

# 170. Doctor

```text
forgeyard cost doctor
```

---

# 171. Doctor Checks

```text
missing pricing
unattributed usage
currency mismatch
budget config
provider pricing auth
```

---

# 172. Unattributed Usage

First-class.

---

# 173. CostAttributionState

```rust
pub enum CostAttributionState {
    Attributed,
    PartiallyAttributed,
    Unattributed,
    Unknown,
}
```

---

# 174. Never Silently Drop Unattributed Cost

Critical.

---

# 175. Shared System Cost

Can remain unallocated.

---

# 176. Reconciliation

Metering events are at-least-once.

---

# 177. Usage Dedup Key

Source resource + time bucket + dimension + sequence where applicable.

---

# 178. Idempotent Ingestion

Critical.

---

# 179. Runner Metering

Attempt lifecycle is authoritative for reserved time.

---

# 180. Measured Host Metrics

Observability source can supplement actual CPU/memory.

---

# 181. Metrics Are Not Authority for Job identity

Use Run/Job/Attempt IDs.

---

# 182. CAS Storage Metering

Periodic snapshot/reconciliation.

---

# 183. Storage Byte-Hours

Integrate over time.

---

# 184. Deletion

Stops future accumulation.

---

# 185. Release-Retained Artifacts

Cost attributed according to retention policy.

---

# 186. Network Metering

Provider/object-store reports can reconcile.

---

# 187. Provider Bill Reconciliation

Compare internal estimate.

---

# 188. Variance

```rust
pub struct ProviderCostVariance {
    pub expected: Money,
    pub billed: Money,
    pub difference: Money,
}
```

---

# 189. Large Variance

Anomaly.

---

# 190. Entitlement/Billing Integration

Part 30 raw billable usage may reuse/consume usage records.

---

# 191. Important Separation

```text
provider cost
internal cost
billable usage
customer price
invoice amount
```

all distinct.

---

# 192. No Customer Invoice From Provider Cost Automatically

Critical.

---

# 193. Standalone Mode

Can estimate local machine resource usage.

---

# 194. Local Hardware Cost

Optional internal configured rate.

---

# 195. Default

Usage-only if no cost model configured.

---

# 196. Distributed Mode

Full cost attribution.

---

# 197. Air-Gap

No provider pricing fetch required.

Use manual/internal rate cards.

---

# 198. Cost Snapshot Export

Can move into air-gap.

---

# 199. DR

Raw usage + budgets + pricing snapshots backed up.

---

# 200. Derived rollups can rebuild.

---

# 201. Audit

Audit:

```text
pricing model change
budget change
allocation rule change
manual cost adjustment
chargeback export
```

---

# 202. Routine usage records

Not audit every one.

---

# 203. Notification

Examples:

```text
budget 80%
budget exceeded
forecast exceeds budget
cost anomaly
provider pricing stale
```

---

# 204. Budget Thresholds

```text
50%
80%
100%
120%
```

configurable.

---

# 205. Quiet Hours

Critical budget alerts may bypass depending policy.

---

# 206. Cost Policy

Can constrain:

```text
optional benchmark
preview environment
fleet max
retention
```

---

# 207. Security Baseline

Never remove:

```text
authz
TLS
audit
secret protection
tenant isolation
```

for cost.

---

# 208. Release Gates

Do not fail security release because budget unless explicit business policy.

---

# 209. Emergency Security Release

Can bypass budget with audited exception.

---

# 210. Budget Exception

Use central PolicyException/break-glass semantics.

---

# 211. Cost Forecast Inputs

```text
historical usage
scheduled reservations
known releases
fleet min/warm
retention trends
```

---

# 212. Forecast Output

```rust
pub struct CostForecast {
    pub scope: ResourceScope,
    pub period: TimeRange,
    pub expected: Money,
    pub lower: Option<Money>,
    pub upper: Option<Money>,
    pub confidence: ForecastConfidence,
}
```

---

# 213. Forecast Confidence

```rust
pub enum ForecastConfidence {
    Low,
    Medium,
    High,
}
```

---

# 214. No False Exactness

Critical.

---

# 215. Budget Forecast Gate

Can warn before actual exceed.

---

# 216. Cost Anomaly Model

Simple baseline:

```text
rolling median/mean
seasonality bands
absolute threshold
```

---

# 217. Provider Price Change

Should not be misclassified as usage anomaly if explainable.

---

# 218. Decompose

```text
usage change
price change
allocation change
```

---

# 219. Cost Explainability

`forgeyard cost explain`.

---

# 220. Example

```text
Project A cost increased 28%
- GPU seconds +15%
- price +5%
- log retention +8%
```

---

# 221. No Opaque Score

Critical.

---

# 222. CLI

```text
forgeyard cost summary
forgeyard cost project <id>
forgeyard cost run <id>
forgeyard cost budget list
forgeyard cost forecast
forgeyard cost anomalies
forgeyard cost explain
forgeyard cost export
```

---

# 223. Machine Output

JSON/RON.

---

# 224. Cost Data Retention

Raw usage long enough for audit/repricing.

---

# 225. Provider Invoice Data

Sensitive.

---

# 226. Access Control

Restricted.

---

# 227. Cost Data Classification

```rust
pub enum CostDataClass {
    Internal,
    Sensitive,
    Restricted,
}
```

---

# 228. Multi-Tenant Export

Never cross-tenant by default.

---

# 229. Cost Model Version

```rust
pub struct CostModelVersion(u16);
```

---

# 230. Allocation Version

```rust
pub struct AllocationModelVersion(u16);
```

---

# 231. Reprocessing

Old usage can be repriced/reallocated under a new model.

---

# 232. Original Reports

Preserved.

---

# 233. Restatement

New view, not historical mutation.

---

# 234. Financial Period Close

Optional.

---

# 235. Closed Period

Freeze official internal chargeback view.

---

# 236. Correction

Append adjustment.

---

# 237. Not Accounting Ledger

Still keep boundary clear.

---

# 238. Testkit

```text
forgeyard-cost-testkit/src/
├── lib.rs
├── usage.rs
├── attribution.rs
├── pricing.rs
├── allocation.rs
├── budget.rs
├── forecast.rs
└── assertions.rs
```

---

# 239. Unit Tests

Money/currency precision.

---

# 240. Usage Dedup Test

Duplicate events do not double-charge.

---

# 241. Runner Attribution Test

Attempt duration attributed correctly.

---

# 242. Idle Cost Test

Shared allocation explicit.

---

# 243. CAS Dedup Test

Logical tenant bytes != physical provider bytes.

---

# 244. Pricing Snapshot Test

Historical cost remains tied to exact rate.

---

# 245. Repricing Test

Creates new interpretation.

---

# 246. Currency Test

Historical FX snapshot.

---

# 247. Budget Test

Threshold/forecast.

---

# 248. Quota Separation Test

Budget does not alter quota logic directly.

---

# 249. Cost-Aware Scheduler Test

Only soft score among equivalent candidates.

---

# 250. Security Constraint Test

Cheaper untrusted runner never selected over required trust.

---

# 251. Region Residency Test

Cheaper disallowed region rejected.

---

# 252. Unattributed Usage Test

Visible, not dropped.

---

# 253. Provider Invoice Variance Test

Reconciled.

---

# 254. Cost Correction Test

Append adjustment.

---

# 255. Tenant Isolation Test

No cross-tenant cost visibility.

---

# 256. DR Test

Raw usage/pricing/budgets restored.

---

# 257. Forecast Test

Confidence explicit.

---

# 258. Anomaly Decomposition Test

Price vs usage change.

---

# 259. Fuzzing

Fuzz cost/pricing import parsers.

---

# 260. Property Tests

Sum of allocated portions <= or == source shared cost according to method.

---

# 261. Scale Test

Billions of usage records via rollup/partition strategy.

---

# 262. Implementation Phase 1 — Raw Usage Model

Compute/storage/network.

---

# 263. Phase 2 — Cost Attribution

Run/job/project/tenant.

---

# 264. Phase 3 — Pricing Snapshots

Manual/provider.

---

# 265. Phase 4 — Showback

UI/API.

---

# 266. Phase 5 — Budgets/Notifications

Governance.

---

# 267. Phase 6 — Fleet/Scheduler Cost Signals

Optimization.

---

# 268. Phase 7 — Shared Cost Allocation

Fully loaded views.

---

# 269. Phase 8 — Forecast/Anomaly

Operations.

---

# 270. Phase 9 — Provider Invoice Reconciliation

Accuracy.

---

# 271. Phase 10 — Chargeback Export

Enterprise.

---

# 272. Phase 11 — Multi-Currency/Air-Gap

Global deployments.

---

# 273. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 274. Acceptance Tests

1. Raw usage is stored independently from price.
2. Usage records are immutable/idempotent.
3. Every usage record is tenant scoped.
4. Cost interpretation references exact pricing snapshot.
5. Historical pricing is never silently rewritten.
6. Repricing creates a new interpretation.
7. Provider cost, internal cost, transfer price, and customer price remain distinct.
8. Reserved and measured compute usage remain distinct.
9. Idle capacity cost is explicitly allocated or left unallocated.
10. CAS logical tenant usage is separate from physical provider storage.
11. Attribution confidence is explicit.
12. Unattributed usage is visible.
13. Provider invoice variance is recorded rather than hidden.
14. Budget and quota remain separate concepts.
15. Budgets cannot silently remove security controls.
16. Cost-aware scheduling only ranks already-eligible candidates.
17. Cheaper capacity cannot bypass trust/residency/platform requirements.
18. Cost API outage cannot stop correct execution.
19. Pricing staleness is visible.
20. Forecast confidence is explicit.
21. Cost anomalies distinguish price, usage, and allocation changes.
22. Cache/incremental "savings" are labeled estimates.
23. Chargeback exports include attribution/pricing provenance.
24. Cost corrections append adjustments.
25. Tenant A cannot read Tenant B costs.
26. Sensitive contract pricing is restricted.
27. Standalone can work in usage-only mode.
28. Air-gapped deployments can use manual rate cards.
29. Raw usage/pricing/budget state survives DR.
30. Derived rollups can rebuild.
31. Entitlement billing can consume usage without conflating internal cost.
32. Audit covers pricing/budget/allocation admin changes.
33. Standalone/distributed share cost semantics.
34. Cost views are explainable down to usage facts.
35. Forgeyard dogfoods cost accounting to understand its own CI resource economics.

---

# 275. Production Readiness Gates

Do not call FinOps architecture production-ready until:

```text
usage dedup/attribution is stable
money/currency precision is tested
pricing snapshots are immutable
tenant isolation passes
unattributed usage is surfaced
budget/quota separation is enforced
cost-aware scheduler/fleet integrations remain soft-only
provider invoice variance path works
DR/repricing tests pass
large-scale rollups perform acceptably
```

---

# 276. Architectural Invariants

1. raw usage precedes pricing;
2. usage is immutable/idempotent;
3. pricing is versioned;
4. historical costs bind exact pricing snapshots;
5. repricing does not rewrite raw usage;
6. provider/internal/transfer/customer pricing are separate domains;
7. reserved and measured resource usage are separate;
8. shared cost allocation is explicit/versioned;
9. unknown/unallocated cost is permitted;
10. false precision is avoided;
11. cost confidence is explicit;
12. tenant cost data is isolated;
13. budget is not quota;
14. budget does not weaken security baseline;
15. cost influences only equivalent safe execution choices;
16. cost cannot bypass scheduler hard filters;
17. cost cannot bypass residency/trust/policy;
18. pricing API outage does not block correctness;
19. stale pricing is visible;
20. forecast is advisory;
21. anomalies are explainable;
22. cache savings are estimated/counterfactual;
23. billing/customer invoices remain Part 30/external accounting;
24. corrections append adjustments;
25. raw usage supports rebuilding rollups;
26. provider invoice data is external evidence;
27. air-gap can operate with manual rate cards;
28. standalone/distributed share semantics;
29. admin cost model changes are audited;
30. Forgeyard dogfoods its own cost system.

---

# 277. Final Target Architecture

```text
                 Resource Consumption
                         │
                         ▼
                    Usage Records
                         │
                         ▼
                    Attribution
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
      Pricing Snapshot Allocation   Confidence
            │            │            │
            └────────────┼────────────┘
                         ▼
                      Cost Fact
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         Showback      Budget       Forecast
            │            │            │
            └────────────┼────────────┘
                         ▼
                Optional Optimization
```

---

# 278. Final Architectural Position

Cost derivation:

```text
immutable UsageRecord
+
immutable PricingSnapshot
+
versioned AllocationModel
  ↓
CostFact
```

Budget:

```text
CostFacts
+
Budget
+
Forecast
  ↓
notify / approval / optional-work guardrail
```

Scheduler/autoscaler:

```text
hard eligibility
  ↓
equivalent safe candidates
  ↓
cost preference
  ↓
soft ranking only
```

The key guarantee is:

> **Forgeyard can explain where infrastructure money goes without letting financial optimization corrupt CI/CD semantics. Costs are derived from immutable usage and pricing evidence, shared allocations are explicit, uncertainty is visible, and cheaper infrastructure is considered only after security, trust, platform, policy, tenant, and correctness requirements have already been satisfied.**

---

# 279. Extended Architecture Sequence

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
```
