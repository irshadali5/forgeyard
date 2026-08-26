# 50 — Forgeyard Reliability Engineering, SLO, Error Budget, Availability & Resilience Governance System Architecture

**Document type:** Core Reliability Engineering, SLI/SLO, Error Budget, Availability, Resilience, Dependency Reliability & Service Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** service-level indicators, service-level objectives, error budgets, burn rates, availability windows, reliability policies, dependency SLOs, degraded modes, maintenance accounting, capacity protection, resilience tests, failover readiness, incident linkage, recovery objectives, reliability scorecards, and operational decision support  
**Architecture style:** Measurement-first, explicit objectives, error-budget governance, exact time-window semantics, dependency-aware reliability, evidence-backed degraded modes, conservative failure accounting, and no SLO metric becoming authority over real system state  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Observability/Health/Doctor, HA/Coordination, Operations/Backup/DR, Security, Runner Fleet Autoscaling, Notifications, Audit, Deployment, Cost/FinOps, Catalog, and Incident Response. This subsystem turns reliability from a collection of metrics into a governed engineering model.

---

# 1. Purpose

Forgeyard runs business-critical CI/CD workflows.

Operators need answers such as:

```text
is the control plane meeting its availability target?
how reliable is job dispatch?
how often do CAS operations fail?
how much error budget remains?
is a service burning reliability too quickly?
should we pause risky feature rollout?
which dependency is degrading user-visible reliability?
are maintenance windows excluded or counted?
did the last incident violate the SLO?
```

The central rule is:

> **SLOs describe expected reliability over explicit windows; they do not redefine the actual health or success state of Forgeyard operations.**

A second rule is:

> **Error budget is a governance signal, not permission to violate correctness, security, tenant isolation, or data-integrity guarantees.**

A third rule is:

> **Every SLO must be built from an explainable SLI whose numerator, denominator, scope, exclusions, and time semantics are explicit.**

---

# 2. Architectural Position

```text
                 Canonical Operational Events
          ┌────────────┼────────────┐
          ▼            ▼            ▼
        API          Runs          Storage
          │            │            │
          └────────────┼────────────┘
                       ▼
                     SLI Engine
                       │
                       ▼
                     SLO Engine
                       │
             ┌─────────┼─────────┐
             ▼         ▼         ▼
         Error Budget Burn Rate Reliability State
             │         │         │
             └─────────┼─────────┘
                       ▼
               Governance / Alerts
```

---

# 3. Goals

The subsystem MUST:

1. define SLI identity;
2. define SLO identity;
3. support availability SLOs;
4. support latency SLOs;
5. support freshness SLOs;
6. support correctness-adjacent operational SLIs;
7. support error budgets;
8. support burn rates;
9. support rolling windows;
10. support calendar windows;
11. support maintenance accounting;
12. support dependency attribution;
13. support degraded modes;
14. support recovery objectives;
15. support reliability policy;
16. support release/change governance;
17. support incident linkage;
18. support capacity protection;
19. support historical analysis;
20. support reliability scorecards;
21. support audit for objective changes;
22. support notifications;
23. support API/UI/CLI;
24. support tenant isolation;
25. support standalone mode;
26. support distributed mode;
27. support DR;
28. support reprocessing;
29. remain independent from raw observability transport;
30. never become execution authority.

---

# 4. Non-Goals

This subsystem does not:

```text
replace metrics/logs/traces
replace health checks
replace incident response
replace deployment rollback
replace HA consensus
replace policy
replace customer SLA/legal contracts
```

---

# 5. Workspace Structure

```text
crates/reliability/
├── forgeyard-reliability/
├── forgeyard-reliability-model/
├── forgeyard-reliability-sli/
├── forgeyard-reliability-slo/
├── forgeyard-reliability-budget/
├── forgeyard-reliability-burn/
├── forgeyard-reliability-dependency/
├── forgeyard-reliability-resilience/
├── forgeyard-reliability-governance/
├── forgeyard-reliability-health/
└── forgeyard-reliability-testkit/
```

Use modules first; split only where analytical/runtime boundaries justify.

---

# 6. ServiceLevelIndicatorId

```rust
pub struct ServiceLevelIndicatorId(Digest);
```

---

# 7. ServiceLevelObjectiveId

```rust
pub struct ServiceLevelObjectiveId(Digest);
```

---

# 8. SLI Definition

```rust
pub struct ServiceLevelIndicator {
    pub id: ServiceLevelIndicatorId,
    pub scope: ReliabilityScope,
    pub kind: SliKind,
    pub source: SliSource,
    pub semantics_version: SliSemanticsVersion,
}
```

---

# 9. Reliability Scope

```rust
pub enum ReliabilityScope {
    System,
    Component(ComponentKind),
    ApiRoute(ApiRouteClass),
    Tenant(TenantId),
    RunnerFleet(RunnerFleetId),
    Storage(StorageSubsystemId),
    Deployment(EnvironmentId),
    Custom(ReliabilityScopeId),
}
```

---

# 10. SLI Kind

```rust
pub enum SliKind {
    Availability,
    Latency,
    Freshness,
    SuccessRatio,
    QueueDelay,
    DispatchReliability,
    DataDurabilityObservation,
    RecoveryTime,
    Custom(SliKindId),
}
```

---

# 11. SLI Source

Must resolve to canonical operational evidence.

---

# 12. Examples

```text
API request outcomes
Run creation outcomes
Job dispatch outcomes
CAS read/write results
agent registration outcomes
deployment convergence events
```

---

# 13. No Dashboard-Scraped SLI

Critical.

Prefer stable event/metric definitions.

---

# 14. SLI Semantics Version

```rust
pub struct SliSemanticsVersion(u16);
```

---

# 15. Why

Changing success/exclusion semantics changes historical interpretation.

---

# 16. Objective

```rust
pub struct ServiceLevelObjective {
    pub id: ServiceLevelObjectiveId,
    pub indicator: ServiceLevelIndicatorId,
    pub target: SloTarget,
    pub window: SloWindow,
    pub policy: SloPolicy,
}
```

---

# 17. SLO Target

```rust
pub enum SloTarget {
    Ratio(Decimal),
    LatencyPercentile {
        percentile: Decimal,
        max: Duration,
    },
    Freshness {
        max_age: Duration,
    },
    Recovery {
        max_duration: Duration,
    },
}
```

---

# 18. Ratio Precision

Use decimal.

---

# 19. SLO Window

```rust
pub enum SloWindow {
    Rolling(Duration),
    CalendarDay(TimeZoneId),
    CalendarWeek(TimeZoneId),
    CalendarMonth(TimeZoneId),
}
```

---

# 20. Rolling Window

Recommended for burn-rate alerts.

---

# 21. Calendar Window

Useful for reporting.

---

# 22. Timezone

Explicit.

---

# 23. DST

Calendar-window semantics versioned/tested.

---

# 24. Availability SLI

Generic:

```text
good events / valid events
```

---

# 25. Good Event

Explicit predicate.

---

# 26. Valid Event

Explicit denominator predicate.

---

# 27. Exclusion

Explicit, minimal.

---

# 28. No Arbitrary Post-Hoc Exclusion

Critical.

---

# 29. Maintenance

Must be declared before or explicitly governed after incident review.

---

# 30. Maintenance Accounting Policy

```rust
pub enum MaintenanceAccounting {
    CountNormally,
    ExcludeApprovedWindows,
    SeparateReporting,
}
```

---

# 31. Recommended Baseline

Count normally for user-impact SLOs; separate maintenance view if needed.

---

# 32. No "Maintenance Means No User Impact" Assumption

Critical.

---

# 33. Availability Example

Control-plane API:

```text
valid authenticated requests
that return expected non-5xx terminal response
/
all valid authenticated requests
```

---

# 34. Client Error

Usually not service unavailability.

---

# 35. Policy Error

May be valid response.

---

# 36. 429

Classification depends on contract.

---

# 37. Queue Delay SLI

Measures:

```text
job eligible-to-dispatch latency
```

---

# 38. Important

Do not include time blocked by user's own policy/quota unless specifically desired.

---

# 39. Dispatch Reliability

```text
successful valid lease assignments
/
eligible dispatch attempts
```

---

# 40. Runner Failure

May impact dispatch/retry SLO depending scope.

---

# 41. CAS Availability

Separate:

```text
read
write
metadata lookup
```

---

# 42. CAS Durability

Hard to measure directly.

---

# 43. Durability Observation

Can use:

```text
scrub failures
replica loss
unrecoverable object count
```

but does not prove mathematical durability.

---

# 44. Honest Naming

Critical.

---

# 45. Freshness SLI

Useful for:

```text
search projection
analytics
catalog projection
config propagation
```

---

# 46. Freshness

Derived system may be healthy but stale.

---

# 47. Latency SLI

Percentiles over valid requests/events.

---

# 48. Histogram Source

Part 17 metrics.

---

# 49. Sampling

Must not bias SLO unexpectedly.

---

# 50. Tail Latency

p95/p99 where justified.

---

# 51. Error Budget

For ratio target:

```text
allowed bad fraction = 1 - target
```

---

# 52. ErrorBudget

```rust
pub struct ErrorBudget {
    pub slo: ServiceLevelObjectiveId,
    pub window: TimeRange,
    pub allowed_bad: Decimal,
    pub consumed_bad: Decimal,
}
```

---

# 53. Budget Remaining

```text
max(0, allowed - consumed)
```

---

# 54. Negative Remaining

Can represent overspend in reporting.

---

# 55. Budget Unit

Could be:

```text
requests
events
minutes
```

depending SLI semantics.

---

# 56. No Universal "minutes of downtime" Conversion

Critical.

---

# 57. Burn Rate

```text
actual error rate
/
allowed error rate
```

---

# 58. BurnRate

```rust
pub struct BurnRate(Decimal);
```

---

# 59. Burn Rate > 1

Budget being consumed too fast.

---

# 60. Multi-Window Burn

Recommended.

---

# 61. Example

Fast + slow windows:

```text
5m + 1h
30m + 6h
```

exact values configurable.

---

# 62. Burn Rule

```rust
pub struct BurnRule {
    pub short_window: Duration,
    pub long_window: Duration,
    pub threshold: Decimal,
    pub severity: AlertSeverity,
}
```

---

# 63. No Single Alert Threshold for Every SLO

---

# 64. Reliability State

```rust
pub enum ReliabilityState {
    Healthy,
    BudgetBurning,
    BudgetCritical,
    BudgetExhausted,
    Unknown,
}
```

---

# 65. Unknown

If telemetry/evidence incomplete.

---

# 66. Missing Data

Never silently treated as healthy.

Critical.

---

# 67. Telemetry Gap

Separate signal.

---

# 68. SloEvaluationId

```rust
pub struct SloEvaluationId(Digest);
```

---

# 69. Evaluation

```rust
pub struct SloEvaluation {
    pub id: SloEvaluationId,
    pub slo: ServiceLevelObjectiveId,
    pub window: TimeRange,
    pub good: Decimal,
    pub total: Decimal,
    pub achieved: Decimal,
    pub state: ReliabilityState,
}
```

---

# 70. Evaluation Provenance

Record source query/version.

---

# 71. Reprocessing

Can recompute history from retained raw events/metrics if semantics permit.

---

# 72. Historical Evaluations

Never mutate official reports silently.

---

# 73. Restatement

New evaluation version.

---

# 74. Dependency SLO

Forgeyard depends on:

```text
Postgres/Neon
CAS backend
SCM provider
secret provider
cloud provider
notification provider
```

---

# 75. DependencyReliabilityId

```rust
pub struct DependencyReliabilityId(Ulid);
```

---

# 76. Dependency Reliability

Track:

```text
availability
latency
error rate
freshness
```

where observable.

---

# 77. Provider SLO

External published SLA not same as Forgeyard-observed dependency SLI.

---

# 78. Keep Separate

Critical.

---

# 79. Dependency Attribution

If control plane degraded because CAS unavailable, record relation.

---

# 80. Causal Confidence

Explicit.

---

# 81. No Automatic "Provider Caused Incident" Without Evidence

---

# 82. Dependency Budget

Optional.

---

# 83. Cascading Reliability

User-visible SLO may depend on multiple components.

---

# 84. Do Not Multiply SLO Targets Blindly

Critical.

---

# 85. Model Observed User Journey

Preferred.

---

# 86. User Journey SLO

Examples:

```text
submit run → run accepted
eligible job → lease issued
artifact request → bytes available
```

---

# 87. Component SLO

Useful for ownership.

---

# 88. Journey SLO

Useful for user impact.

---

# 89. SLO Scope Hierarchy

```text
system
journey
component
dependency
```

---

# 90. Reliability Budget Policy

```rust
pub enum ErrorBudgetAction {
    Notify,
    FreezeRiskyRollout,
    RequireApproval,
    PauseOptionalChanges,
    NoAutomaticAction,
}
```

---

# 91. Default

Notify + human review.

---

# 92. Automation

Only explicit policy.

---

# 93. Error Budget Exhaustion

Must not:

```text
disable auth
disable audit
skip tests
weaken isolation
```

---

# 94. Critical.

---

# 95. Feature Flag Integration

Part 39.

---

# 96. Example

If new scheduler optimization burns budget:

```text
kill flag
```

if correlation/evidence supports.

---

# 97. Release Governance

Error budget can be policy input.

---

# 98. Example

Require extra approval for risky control-plane release when budget exhausted.

---

# 99. Emergency Security Fix

May bypass reliability freeze via audited exception.

---

# 100. Policy Exception

Part 11/28.

---

# 101. Deployment Integration

Canary health + SLO burn.

---

# 102. Deployment Rollout

Can pause if deployment-specific SLI degrades.

---

# 103. But Deployment subsystem remains action authority.

---

# 104. Autoscaling Integration

Part 43.

---

# 105. Queue SLO breach

May trigger scale-up recommendation.

---

# 106. Capacity Protection

Reliability subsystem can emit recommendation.

---

# 107. Scheduler/Autoscaler Policy

Decides action.

---

# 108. No Direct Capacity Mutation

Critical.

---

# 109. Degraded Modes

Part 17 introduced degraded health.

---

# 110. ReliabilityMode

```rust
pub enum ReliabilityMode {
    Normal,
    DegradedReadOnly,
    DegradedNoNewRuns,
    DegradedNoRelease,
    Recovery,
}
```

---

# 111. Mode Authority

Separate operational control/policy.

---

# 112. SLO System

May recommend/observe mode, not self-authorize unless explicit policy.

---

# 113. Example

Metadata DB read-only:

```text
UI read
history read
no new run
no release mutation
```

---

# 114. Graceful Degradation

Must be designed per subsystem.

---

# 115. Control Plane Degradation Matrix

```text
DB unavailable
CAS unavailable
SCM unavailable
secret provider unavailable
signing unavailable
```

---

# 116. Each Has Explicit Allowed Operations

---

# 117. No Generic "degraded" boolean.

Critical.

---

# 118. Resilience Scenario

```rust
pub struct ResilienceScenario {
    pub id: ResilienceScenarioId,
    pub fault: FaultInjectionSpec,
    pub expected_behavior: ExpectedResilienceBehavior,
}
```

---

# 119. Resilience Test Types

```text
dependency outage
network partition
DB failover
CAS corruption
runner loss
provider outage
certificate expiry
disk full
```

---

# 120. Chaos Testing

Controlled.

---

# 121. Production Chaos

Requires explicit permission/scope.

---

# 122. Baseline

Staging/pre-production.

---

# 123. Resilience Evidence

```rust
pub struct ResilienceEvidence {
    pub scenario: ResilienceScenarioId,
    pub result: ResilienceResult,
    pub recovery_time: Duration,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 124. Resilience Result

```rust
pub enum ResilienceResult {
    Passed,
    DegradedAsExpected,
    Failed,
    Inconclusive,
}
```

---

# 125. Recovery Time Objective

Operational target.

---

# 126. RTO

Part 25 DR may define.

---

# 127. Recovery Point Objective

Data-loss target.

---

# 128. RPO

Storage/backup domain.

---

# 129. Reliability subsystem tracks measured evidence against configured objectives.

---

# 130. Do Not Confuse RTO/RPO With SLO Availability

Critical.

---

# 131. SLO Ownership

Each SLO has owner.

---

# 132. SloOwner

```rust
pub struct SloOwner {
    pub team: OrganizationUnitId,
    pub escalation: Option<NotificationRouteId>,
}
```

---

# 133. Catalog Integration

Part 49 component owner can suggest SLO owner.

---

# 134. Authz

Ownership metadata does not grant manage permission automatically.

---

# 135. SLO Lifecycle

```rust
pub enum SloLifecycle {
    Draft,
    Active,
    Deprecated,
    Retired,
}
```

---

# 136. Draft

No governance effect.

---

# 137. Active

Evaluated.

---

# 138. Objective Change

Creates new version.

---

# 139. No In-Place Historical Target Mutation

Critical.

---

# 140. SloVersion

```rust
pub struct SloVersion(u64);
```

---

# 141. Target Change

New version effective at timestamp.

---

# 142. Historical Reports

Use historical version.

---

# 143. SLO Approval

High-impact objectives may require review.

---

# 144. Example

Production control-plane availability target.

---

# 145. Audit

Audit:

```text
SLO create/update/retire
exclusion rule change
maintenance accounting change
budget-action policy change
```

---

# 146. Routine evaluations

Not privileged audit.

---

# 147. Reliability Exception

Example planned experiment exclusion.

---

# 148. Must be explicit/time-bounded/audited.

---

# 149. No Retroactive Silent Exclusion

Critical.

---

# 150. MaintenanceWindowId

```rust
pub struct MaintenanceWindowId(Ulid);
```

---

# 151. Maintenance Window

```rust
pub struct MaintenanceWindow {
    pub id: MaintenanceWindowId,
    pub scope: ReliabilityScope,
    pub start: Timestamp,
    pub end: Timestamp,
    pub approved_by: PrincipalId,
    pub accounting: MaintenanceAccounting,
}
```

---

# 152. Planned Maintenance

Still visible in reporting.

---

# 153. Incident Link

```rust
pub struct ReliabilityIncidentLink {
    pub incident: IncidentId,
    pub affected_slos: Vec<ServiceLevelObjectiveId>,
}
```

---

# 154. Incident Postmortem

Can include:

```text
budget burned
SLO violation
recovery time
```

---

# 155. SLO Violation

Not every incident.

---

# 156. Incident

Not every SLO violation.

---

# 157. Keep Separate

Critical.

---

# 158. Error Budget Ledger

Not financial ledger.

---

# 159. BudgetConsumptionRecord

```rust
pub struct BudgetConsumptionRecord {
    pub slo: ServiceLevelObjectiveId,
    pub window: TimeRange,
    pub bad_events: Decimal,
    pub source: ReliabilityEvidenceRef,
}
```

---

# 160. Raw evidence remains observability/event data.

---

# 161. SLO Query Engine

Should use bounded pre-aggregated data for scale.

---

# 162. Materialized SLI Buckets

Example 1m/5m.

---

# 163. Raw Data Retention

Part 46.

---

# 164. If raw expires

Historical SLO report remains if retained.

---

# 165. Reprocessing Limit

Only while sufficient source evidence remains.

---

# 166. Unknown After Data Loss

Do not fabricate.

---

# 167. Reliability Scorecard

For component/service.

---

# 168. Scorecard Inputs

```text
active SLOs
budget state
recent incidents
resilience-test freshness
DR drill freshness
```

---

# 169. Scorecard Result

Advisory.

---

# 170. Policy

Can explicitly consume.

---

# 171. No Single Reliability Number as Truth

Critical.

---

# 172. ReliabilitySummary

```rust
pub struct ReliabilitySummary {
    pub slos: Vec<SloEvaluationRef>,
    pub open_incidents: u32,
    pub resilience_state: ResilienceReadiness,
}
```

---

# 173. Resilience Readiness

```rust
pub enum ResilienceReadiness {
    Current,
    Stale,
    Missing,
    Unknown,
}
```

---

# 174. SLO Templates

Part 42 can provide golden-path SLO definitions.

---

# 175. Example

Standard service golden path:

```text
API availability
API latency
deployment freshness
```

---

# 176. Project Can Tighten

Where policy permits.

---

# 177. Cannot Weaken Mandatory Minimum

If org policy.

---

# 178. Multi-Tenant

Tenant-specific SLO view possible.

---

# 179. System-wide SLO

For hosted Forgeyard.

---

# 180. Tenant-Specific SLO

May reveal usage patterns; isolate.

---

# 181. Shared Component SLO

Avoid leaking another tenant's data.

---

# 182. Aggregate carefully.

---

# 183. Reliability API

Potential:

```text
GET  /v1/reliability/slos
GET  /v1/reliability/slos/{id}
GET  /v1/reliability/slos/{id}/budget
GET  /v1/reliability/burn
GET  /v1/reliability/resilience
POST /v1/reliability/slos
POST /v1/reliability/maintenance
```

---

# 184. Permissions

```text
reliability.read
reliability.slo.manage
reliability.maintenance.manage
reliability.resilience.run
reliability.governance.manage
```

---

# 185. Dioxus UI

Pages:

```text
Reliability Overview
SLOs
Error Budgets
Burn Rates
Dependencies
Resilience Tests
Maintenance
```

---

# 186. SLO Detail

Shows:

```text
target
window
achieved
budget remaining
burn rate
violations
owner
```

---

# 187. Burn Chart

Time-series.

---

# 188. Budget Explanation

Show numerator/denominator.

---

# 189. SLI Explain

`forgeyard reliability explain <slo>`.

---

# 190. Must Show

```text
good-event predicate
valid-event predicate
exclusions
window
source
semantics version
```

---

# 191. CLI

```text
forgeyard reliability status
forgeyard reliability slo list
forgeyard reliability slo show
forgeyard reliability budget
forgeyard reliability burn
forgeyard reliability resilience
forgeyard reliability doctor
```

---

# 192. Machine Output

JSON/RON.

---

# 193. Notifications

Examples:

```text
fast burn
budget critical
budget exhausted
telemetry gap
resilience test stale
dependency SLO degraded
```

---

# 194. Alert Dedup

Part 29.

---

# 195. Alert Storm

Burn alerts deduplicated/grouped.

---

# 196. Escalation

SLO owner/team.

---

# 197. Observability Metrics

```text
reliability_slo_achieved_ratio
reliability_error_budget_remaining_ratio
reliability_burn_rate
reliability_slo_unknown
reliability_resilience_test_age_seconds
```

---

# 198. Labels

Controlled:

```text
slo_id
scope_kind
```

SLO IDs manageable cardinality.

---

# 199. Tracing

```text
reliability.evaluate
reliability.budget
reliability.burn
reliability.dependency
reliability.resilience
```

---

# 200. Health

Checks:

```text
evaluation lag
source telemetry availability
bucket freshness
SLO config validity
```

---

# 201. Doctor

```text
forgeyard reliability doctor
```

---

# 202. Doctor Checks

```text
SLO without owner
SLI missing source
invalid denominator
stale evaluation
telemetry gaps
conflicting maintenance policy
```

---

# 203. SLO Configuration

Part 39 typed config/metadata.

---

# 204. Repository Config

Can propose component SLO.

---

# 205. Organization Policy

Can require minimum.

---

# 206. Repository Cannot Exclude Arbitrary Failures

Critical.

---

# 207. Evaluation Worker

Derived/rebuildable.

---

# 208. HA

Multiple workers safe.

---

# 209. Window Bucket Claim

Idempotent.

---

# 210. No Raft Requirement

---

# 211. Standalone Mode

Local SLOs optional.

---

# 212. Distributed Mode

Full system/component/tenant SLO engine.

---

# 213. Air-Gap

Works from local telemetry.

---

# 214. DR

SLO definitions/official evaluations backed up.

---

# 215. Derived buckets can rebuild.

---

# 216. Reliability Data Lifecycle

Part 46.

---

# 217. Official SLO reports may have longer retention.

---

# 218. Raw telemetry shorter.

---

# 219. Cost Integration

Part 45.

---

# 220. Reliability vs Cost

Can compare:

```text
cost of redundancy
vs
reliability outcome
```

but no automatic unsafe tradeoff.

---

# 221. Example

Warm pool reduction may lower cost but hurt queue SLO.

---

# 222. FinOps

Advisory joint analysis.

---

# 223. Capacity Planning

Part 33/43.

---

# 224. SLO trend can inform capacity forecast.

---

# 225. Failure Diagnosis

Part 48 can explain repeated SLO violations.

---

# 226. Catalog

Part 49 component page surfaces SLO state.

---

# 227. Security

Security-critical SLOs:

```text
certificate rotation freshness
audit delivery freshness
```

possible, but security controls remain mandatory independent of SLO.

---

# 228. Security SLO Failure

Never means control can be skipped.

---

# 229. Reliability Objective Types

Recommended baseline:

```text
control-plane API availability
run-creation availability
job-dispatch latency
CAS read/write availability
agent reconnect/registration
search/catalog freshness
deployment convergence
```

---

# 230. High-Assurance Objective

Signing service availability may be tracked.

---

# 231. But unavailable signing fails closed.

---

# 232. Reliability Governance State

```rust
pub enum ReliabilityGovernanceState {
    Normal,
    Caution,
    ChangeRestricted,
    RecoveryFocused,
}
```

---

# 233. Derived recommendation.

---

# 234. Policy may map budget state → governance state.

---

# 235. Change Restriction

Could require:

```text
extra approval
canary
smaller rollout
```

---

# 236. Never disable urgent security fixes automatically.

---

# 237. Reliability Review

Periodic.

---

# 238. Review Inputs

```text
SLO attainment
budget burn
incident history
dependency health
resilience drills
```

---

# 239. Human Governance

Important.

---

# 240. SLO Target Setting

Should reflect actual user need.

---

# 241. No "five nines everywhere"

Critical.

---

# 242. Target Cost

Higher reliability can cost more.

---

# 243. Explicit tradeoff

Engineering/product decision.

---

# 244. External SLA

If product has contractual SLA, map separately.

---

# 245. SLA != SLO

Critical.

---

# 246. ServiceLevelAgreementRef

Optional external metadata.

---

# 247. SLO can be stricter than SLA.

---

# 248. Do not auto-generate legal SLA from SLO.

---

# 249. Reliability Event

```rust
pub enum ReliabilityEvent {
    SloViolated,
    BudgetThresholdCrossed,
    BurnRateTriggered,
    ResilienceTestFailed,
    TelemetryInsufficient,
}
```

---

# 250. Events

At-least-once/idempotent.

---

# 251. Reconciliation

Periodic recomputation ensures notification/event loss does not corrupt state.

---

# 252. Testkit

```text
forgeyard-reliability-testkit/src/
├── lib.rs
├── sli.rs
├── slo.rs
├── budget.rs
├── burn.rs
├── maintenance.rs
├── dependency.rs
├── resilience.rs
└── assertions.rs
```

---

# 253. Unit Tests

Ratio/latency/freshness calculations.

---

# 254. Denominator Test

Invalid/client events excluded only by declared semantics.

---

# 255. Missing Data Test

Returns Unknown.

---

# 256. Maintenance Test

Accounting follows explicit policy.

---

# 257. Retroactive Exclusion Test

Blocked/audited.

---

# 258. Burn Rate Test

Correct multi-window behavior.

---

# 259. Budget Exhaustion Test

State transitions.

---

# 260. Security Baseline Test

Budget action cannot disable security controls.

---

# 261. Release Governance Test

Error budget can require approval but not rewrite release outcome.

---

# 262. Autoscaler Integration Test

SLO recommendation does not directly create capacity.

---

# 263. Dependency Test

Provider outage correlation confidence explicit.

---

# 264. Tenant Isolation Test

No cross-tenant reliability leakage.

---

# 265. Historical Version Test

Old target remains associated with old period.

---

# 266. Reprocessing Test

New semantics creates restated evaluation, not silent overwrite.

---

# 267. Resilience Test

Expected degraded mode observed.

---

# 268. DR Test

RTO/RPO evidence linked but not conflated with availability SLO.

---

# 269. Telemetry Gap Test

No false green.

---

# 270. DST/Calendar Window Test

Correct time semantics.

---

# 271. Scale Test

Large SLI event volume via buckets.

---

# 272. Failure Injection

```text
metrics delayed
event stream interrupted
DB restart
evaluation worker crash
```

---

# 273. Implementation Phase 1 — SLI/SLO Model

Core.

---

# 274. Phase 2 — Ratio/Latency/Freshness Evaluation

Useful baseline.

---

# 275. Phase 3 — Error Budgets/Burn Rates

Governance.

---

# 276. Phase 4 — Notifications/UI

Operability.

---

# 277. Phase 5 — Dependency Reliability

Attribution.

---

# 278. Phase 6 — Deployment/Release Governance

Change safety.

---

# 279. Phase 7 — Resilience Scenarios

Testing.

---

# 280. Phase 8 — Catalog Integration

Ownership/discovery.

---

# 281. Phase 9 — Capacity/FinOps Integration

Tradeoff analysis.

---

# 282. Phase 10 — Tenant-Level SLOs

Hosted enterprise.

---

# 283. Phase 11 — SLA Metadata / Reporting

Optional enterprise.

---

# 284. Phase 12 — Scale/DR/Fuzz Hardening

Production readiness.

---

# 285. Acceptance Tests

1. Every SLO references a versioned explainable SLI.
2. Numerator and denominator are explicit.
3. Missing source data never becomes false green.
4. Maintenance accounting is explicit.
5. Retroactive exclusions cannot silently rewrite attainment.
6. SLO targets are versioned.
7. Historical periods retain historical objective version.
8. Error budget is derived from SLO semantics.
9. Burn rate uses explicit windows.
10. Budget exhaustion never weakens security/correctness controls.
11. Error budget actions are policy-governed.
12. SLO state does not rewrite actual health or job outcome.
13. Dependency observed reliability is separate from provider-published SLA.
14. Dependency attribution carries confidence.
15. Journey SLOs are preferred for user impact where practical.
16. Component SLOs remain useful for ownership.
17. Queue SLO excludes policy/quota wait only according to explicit semantics.
18. CAS durability observations are not mislabeled mathematical durability.
19. Degraded modes are typed per operation, not a generic boolean.
20. Reliability subsystem may recommend capacity changes but does not mutate fleets directly.
21. Resilience tests record expected behavior/recovery time.
22. RTO/RPO remain distinct from availability SLO.
23. SLO ownership does not automatically grant permission.
24. Tenant-specific reliability data remains isolated.
25. SLO reports preserve evaluation provenance.
26. Official restatements do not silently overwrite old reports.
27. Alerting handles fast/slow burn without storming.
28. Standalone/distributed share SLO semantics.
29. DR restores SLO definitions/official reports.
30. Derived evaluation workers are restart-safe.
31. Catalog surfaces reliability as projection only.
32. FinOps can analyze cost/reliability tradeoffs without unsafe automation.
33. SLA metadata is not confused with internal SLO.
34. Telemetry gaps are observable.
35. Forgeyard dogfoods its SLO/error-budget system for its own control plane, CAS, dispatch, and deployment paths.

---

# 286. Production Readiness Gates

Do not call reliability architecture production-ready until:

```text
SLI semantics are versioned/explainable
missing-data behavior is safe
error-budget/burn calculations are verified
historical target changes preserve old periods
maintenance/exclusion governance is enforced
dependency attribution confidence is explicit
release/deployment governance cannot weaken security
resilience tests are automated
DR/reprocessing works
high-volume evaluation performs acceptably
```

---

# 287. Architectural Invariants

1. SLO measures reliability; it does not redefine truth;
2. SLI numerator/denominator are explicit;
3. missing data is never healthy by default;
4. SLI semantics are versioned;
5. SLO target/window are versioned;
6. historical evaluations preserve historical objective versions;
7. maintenance accounting is explicit;
8. exclusions are governed;
9. retroactive silent exclusion is forbidden;
10. error budget is derived evidence;
11. burn rates use explicit windows;
12. budget exhaustion never weakens security/correctness;
13. SLO governance acts through policy, not hidden automation;
14. dependency reliability is observed separately from provider SLA;
15. causal attribution carries confidence;
16. journey SLOs model user-visible reliability;
17. degraded modes are typed;
18. capacity recommendations are advisory to fleet/scheduler;
19. RTO/RPO are distinct from availability SLO;
20. resilience tests are evidence, not guarantees;
21. SLO ownership does not imply authz;
22. tenant SLO data is isolated;
23. official evaluations expose provenance;
24. reprocessing creates restatements, not silent mutation;
25. raw telemetry and SLO reports have separate lifecycle;
26. cost/reliability analysis cannot authorize unsafe tradeoffs;
27. SLA and SLO remain distinct;
28. HA uses idempotent/rebuildable evaluation;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its own reliability system.

---

# 288. Final Target Architecture

```text
                   Operational Evidence
                          │
                          ▼
                      SLI Engine
                          │
                          ▼
                      SLO Engine
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
         Attainment    Error Budget   Burn Rate
             │            │            │
             └────────────┼────────────┘
                          ▼
                 Reliability Governance
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
          Notify      Require Review  Recommend Action
```

---

# 289. Final Architectural Position

SLI:

```text
canonical events/metrics
+
versioned good/valid predicates
  ↓
measured indicator
```

SLO:

```text
SLI
+
target
+
window
  ↓
attainment
+
error budget
+
burn rate
```

Governance:

```text
reliability state
+
policy
  ↓
notify / require approval / pause optional risky rollout
```

The key guarantee is:

> **Forgeyard can measure and govern its reliability without turning reliability targets into excuses or hidden control logic. SLOs remain explicit, versioned, evidence-backed objectives; missing telemetry becomes unknown rather than green; and error budgets influence change decisions only through visible policy while security, integrity, and execution correctness remain non-negotiable.**

---

# 290. Extended Architecture Sequence

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
47 CI/CD Migration / Import / Compatibility / Legacy-System Interoperability
48 Failure Diagnosis / Debugging / Reproduction / Bisect / Root-Cause Intelligence
49 Service Catalog / Component Ownership / Environment Inventory / Developer Portal
50 Reliability Engineering / SLO / Error Budget / Availability / Resilience Governance
```
