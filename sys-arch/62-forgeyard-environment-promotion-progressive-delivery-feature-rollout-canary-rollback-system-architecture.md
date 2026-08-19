# 62 — Forgeyard Environment Promotion, Progressive Delivery, Feature Rollout, Canary Analysis & Automated Rollback Governance System Architecture

**Document type:** Core Environment Promotion, Progressive Delivery, Feature Rollout, Canary Analysis, Experiment-Controlled Deployment & Automated Rollback Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** staged environment promotion, canary deployment, percentage rollout, cohort rollout, region/site sequencing, feature rollout controls, health analysis, SLO/error-budget gates, promotion evidence, deployment waves, automated rollback, pause/resume, blast-radius governance, compatibility gating, tenant-aware rollout, and release-to-environment progression  
**Architecture style:** Exact release identity, evidence-bound promotion, progressively bounded blast radius, explicit rollout phases, deterministic gates, reversible automation where safe, health/SLO feedback, policy-governed rollback, and no rebuild between environments  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Release, Deployment, Configuration/Feature Flags, Reliability/SLO/Error Budget, Compatibility, Infrastructure-as-Code, Federation, Notifications, Incident Management, Policy/Authz, Supply Chain, Observability, Multi-Tenancy, and Artifact Registry. This subsystem adds a first-class promotion and progressive-delivery control layer over exact released artifacts and existing deployment primitives.

---

## 1. Purpose

Forgeyard already supports:

```text
build
test
package
sign
release
deploy
rollback
feature flags
SLO evaluation
multi-region execution
```

But enterprise delivery needs more than:

```text
deploy release X to production
```

Real systems often need:

```text
dev
  ↓
staging
  ↓
pilot
  ↓
5% production
  ↓
25%
  ↓
50%
  ↓
100%
```

or:

```text
region A
  ↓
region B
  ↓
region C
```

or:

```text
internal employees
  ↓
beta tenants
  ↓
low-risk customers
  ↓
all tenants
```

Without a dedicated architecture, progressive delivery gets implemented through ad-hoc shell scripts or hidden provider-specific logic.

The central rule is:

> **Promotion always moves the same immutable released bytes through explicitly governed environments and rollout phases. It never rebuilds the software for each stage.**

A second rule is:

> **A rollout phase may advance only when its exact deployment subject has sufficient fresh evidence under the active policy.**

A third rule is:

> **Automated rollback is allowed only when rollback semantics are known, safe enough, and explicitly enabled. “Health got worse” is not sufficient to blindly execute an unsafe reversal.**

---

## 2. Architectural Position

```text
                    ReleaseId
                       │
                       ▼
                  Promotion Plan
                       │
                       ▼
               Target Environment
                       │
              ┌────────┼────────┐
              ▼        ▼        ▼
           Canary    Wave 2   Wave N
              │        │        │
              └────────┼────────┘
                       ▼
                 Health Analysis
                       │
              ┌────────┼────────┐
              ▼        ▼        ▼
           Advance    Pause   Rollback
                       │
                       ▼
                    Evidence
```

---

## 3. Goals

The subsystem MUST:

1. define promotion identity;
2. define rollout-plan identity;
3. define rollout-wave identity;
4. bind exact ReleaseId;
5. bind exact target environment;
6. support dev→staging→production promotion;
7. support canary deployment;
8. support percentage rollout;
9. support tenant/cohort rollout;
10. support region/site rollout;
11. support availability-zone rollout;
12. support device/client population rollout;
13. support feature-flag-assisted rollout;
14. support deployment health analysis;
15. support SLO/error-budget gates;
16. support compatibility gates;
17. support infrastructure readiness gates;
18. support manual approval gates;
19. support automated pause;
20. support automated rollback;
21. support rollout resume;
22. support rollout cancellation;
23. support blast-radius limits;
24. support promotion evidence;
25. support rollback evidence;
26. support audit;
27. support Dioxus UI/API/CLI;
28. support HA/federation;
29. support DR;
30. never rebuild between promotion stages.

---

## 4. Non-Goals

This subsystem does not:

```text
replace Release
replace Deployment
replace Feature Flags
replace Infrastructure
replace SLO evaluation
replace service mesh/load balancer
replace experimentation platform
```

It orchestrates and governs them.

---

## 5. Workspace Structure

```text
crates/progressive-delivery/
├── forgeyard-progressive-delivery/
├── forgeyard-promotion-model/
├── forgeyard-promotion-plan/
├── forgeyard-promotion-gate/
├── forgeyard-canary/
├── forgeyard-rollout-wave/
├── forgeyard-rollout-analysis/
├── forgeyard-rollout-rollback/
├── forgeyard-rollout-reconcile/
├── forgeyard-rollout-health/
└── forgeyard-rollout-testkit/
```

Adapters:

```text
crates/progressive-delivery-adapters/
├── forgeyard-rollout-kubernetes/
├── forgeyard-rollout-aws/
├── forgeyard-rollout-azure/
├── forgeyard-rollout-gcp/
├── forgeyard-rollout-feature-flag/
├── forgeyard-rollout-load-balancer/
└── forgeyard-rollout-custom/
```

Core remains provider-neutral.

---

## 6. PromotionId

```rust
pub struct PromotionId(Ulid);
```

Represents one exact release progressing toward a target environment or channel.

---

## 7. Promotion Subject

```rust
pub struct PromotionSubject {
    pub release: ReleaseId,
    pub environment: EnvironmentId,
}
```

---

## 8. Same Bytes Rule

Promotion references exact artifacts/packages already released.

No:

```text
recompile
repackage
resign
```

unless a new ReleaseId is created.

---

## 9. PromotionPlanId

```rust
pub struct PromotionPlanId(Digest);
```

Immutable identity of rollout strategy and gates.

---

## 10. Promotion Plan

```rust
pub struct PromotionPlan {
    pub id: PromotionPlanId,
    pub release: ReleaseId,
    pub environment: EnvironmentId,
    pub strategy: RolloutStrategy,
    pub gates: Vec<PromotionGateSpec>,
}
```

---

## 11. Rollout Strategy

```rust
pub enum RolloutStrategy {
    Immediate,
    Canary,
    Percentage,
    Cohort,
    Regional,
    AvailabilityZone,
    BlueGreen,
    FeatureFlagAssisted,
    Custom(RolloutStrategyId),
}
```

---

## 12. Immediate

Still uses deployment subsystem.

No progressive phases.

---

## 13. Canary

Small bounded population first.

---

## 14. Percentage

Traffic/user/device percentages.

---

## 15. Cohort

Explicit population set.

---

## 16. Regional

Roll out site/region by site/region.

---

## 17. Availability Zone

Useful to reduce infrastructure blast radius.

---

## 18. Blue/Green

Deploy complete parallel environment, then switch traffic.

---

## 19. Feature Flag Assisted

Deploy code broadly while activating behavior progressively.

---

## 20. Strategy Does Not Change Release Identity

Critical.

---

## 21. RolloutWaveId

```rust
pub struct RolloutWaveId(Ulid);
```

---

## 22. Rollout Wave

```rust
pub struct RolloutWave {
    pub id: RolloutWaveId,
    pub promotion: PromotionId,
    pub ordinal: u32,
    pub target: RolloutTarget,
    pub state: RolloutWaveState,
}
```

---

## 23. RolloutWaveState

```rust
pub enum RolloutWaveState {
    Planned,
    Deploying,
    Observing,
    Passed,
    Failed,
    Paused,
    RolledBack,
    Cancelled,
    Unknown,
}
```

---

## 24. Rollout Target

```rust
pub enum RolloutTarget {
    TrafficPercent(Percentage),
    TenantSet(TenantSetRef),
    Region(RegionId),
    Site(SiteId),
    AvailabilityZone(ZoneId),
    DeviceCohort(DeviceCohortId),
    FeatureCohort(FeatureCohortId),
}
```

---

## 25. Percentages

Use fixed precision.

No floating-point policy comparisons.

---

## 26. Rollout Cohort Identity

Immutable selection definition.

---

## 27. Cohort Membership

If dynamic, exact evaluated membership snapshot is stored for rollout evidence.

---

## 28. No Hidden Random Cohort

Critical.

---

## 29. Deterministic Assignment

Where percentage cohort is generated:

```text
hash(stable subject id + rollout seed)
```

---

## 30. RolloutSeed

```rust
pub struct RolloutSeed(Digest);
```

---

## 31. Stable Assignment

Users/tenants should not oscillate between control/new version during analysis unless strategy explicitly requires.

---

## 32. Blast Radius

```rust
pub struct BlastRadiusLimit {
    pub max_traffic_percent: Option<Percentage>,
    pub max_tenants: Option<u64>,
    pub max_regions: Option<u32>,
}
```

---

## 33. Blast Radius Is Hard Limit

Not advisory.

---

## 34. Promotion Gates

```rust
pub enum PromotionGateSpec {
    DeploymentHealthy,
    SloHealthy(SloPolicyRef),
    ErrorBudget(ErrorBudgetPolicyRef),
    Metrics(RolloutMetricPolicyId),
    Compatibility(CompatibilityPolicyId),
    Security(SecurityGateRef),
    ManualApproval(ApprovalPolicyId),
    InfrastructureReady,
    NoActiveIncident(IncidentGatePolicy),
    Custom(PromotionGateKindId),
}
```

---

## 35. Gate Outcome

```rust
pub enum PromotionGateOutcome {
    Pass,
    Warning,
    Fail,
    Incomplete,
    Unknown,
}
```

---

## 36. Unknown Is Not Pass

Critical.

---

## 37. Gate Evidence

Each gate binds:

```text
ReleaseId
EnvironmentId
RolloutWaveId
observation window
policy digest
evidence references
```

---

## 38. Evidence Freshness

Must be current for exact wave.

---

## 39. Deployment Health

Deployment subsystem remains authoritative.

---

## 40. SLO Health

Part 50 authoritative.

---

## 41. Compatibility

Part 57 authoritative.

---

## 42. Infrastructure

Part 53 authoritative.

---

## 43. Security Findings

Part 37/40 authoritative.

---

## 44. Incident Gate

Can block rollout when target environment has active severe incident.

---

## 45. Incident Gate Does Not Replace IC Judgment

Can allow explicit emergency exception.

---

## 46. Analysis Window

```rust
pub struct RolloutObservationWindow {
    pub minimum: Duration,
    pub maximum: Duration,
}
```

---

## 47. Minimum Window

Prevents instant promotion on insufficient data.

---

## 48. Maximum Window

Prevents indefinite limbo.

---

## 49. Low Traffic

May make metrics inconclusive.

---

## 50. Inconclusive

Can:

```text
wait
extend
require manual approval
```

according policy.

---

## 51. Canary Metric

```rust
pub struct CanaryMetric {
    pub metric: MetricQueryRef,
    pub baseline: CanaryBaseline,
    pub threshold: CanaryThreshold,
}
```

---

## 52. Canary Baseline

```rust
pub enum CanaryBaseline {
    PreviousRelease,
    ControlCohort,
    HistoricalWindow,
    FixedThreshold,
}
```

---

## 53. Comparison

Need exact time windows and population.

---

## 54. No Cherry-Picked Window

Critical.

---

## 55. Metric Direction

```rust
pub enum MetricDirection {
    LowerIsBetter,
    HigherIsBetter,
    WithinRange,
}
```

---

## 56. Metric Threshold

Fixed precision.

---

## 57. Statistical Analysis

Optional.

Can use:

```text
confidence intervals
bootstrap comparison
Mann-Whitney
rate ratio
```

depending metric.

---

## 58. Statistical Significance

Not the only criterion.

---

## 59. Practical Significance

Required.

---

## 60. Example

Tiny latency increase can be statistically significant but operationally irrelevant.

---

## 61. CanaryClassification

```rust
pub enum CanaryClassification {
    Healthy,
    Degraded,
    Regressed,
    Improved,
    Inconclusive,
    Unknown,
}
```

---

## 62. Automated Advance

Allowed only when all required gates pass.

---

## 63. Automated Pause

Can happen on:

```text
metric regression
SLO burn
deployment health degradation
new incident
security event
compatibility violation
```

---

## 64. Automated Rollback

Policy-controlled.

---

## 65. RollbackPolicy

```rust
pub struct RollbackPolicy {
    pub automatic: bool,
    pub triggers: Vec<RollbackTrigger>,
    pub rollback_class: DeploymentRollbackClass,
}
```

---

## 66. Rollback Trigger

```rust
pub enum RollbackTrigger {
    DeploymentFailure,
    SloBurn,
    MetricRegression,
    SecurityIncident,
    Manual,
}
```

---

## 67. Rollback Class

Deployment subsystem supplies exact rollback semantics.

---

## 68. If Rollback Unsafe

Do not automate.

---

## 69. Example Unsafe

```text
irreversible DB migration
backward-incompatible state write
destructive infrastructure change
```

---

## 70. Rollback Compatibility

Part 57 compatibility report can inform.

---

## 71. Roll Forward

Sometimes safer than rollback.

---

## 72. RecoveryStrategy

```rust
pub enum RecoveryStrategy {
    Rollback,
    RollForward,
    Freeze,
    Manual,
}
```

---

## 73. No Generic Rollback Button

Existing invariant extended.

---

## 74. Blue/Green

```text
blue = current
green = candidate
```

Deploy exact ReleaseId to green.

---

## 75. Health Validate Green

Before traffic switch.

---

## 76. Switch

Uses deployment/load-balancer adapter.

---

## 77. Rollback

Switch back if old side still valid.

---

## 78. Database Compatibility

Still required.

---

## 79. Feature Flag Assisted Delivery

Part 39 flags are activation control.

---

## 80. Separation

```text
code deployment
!=
feature activation
```

---

## 81. Deploy Dark

New code can be deployed with feature disabled.

---

## 82. Gradual Enablement

Feature flag rollout controls behavior exposure.

---

## 83. Feature Flag Not Release Authority

Critical.

---

## 84. FeatureFlagRolloutId

```rust
pub struct FeatureFlagRolloutId(Ulid);
```

---

## 85. Flag Cohort

Exact policy.

---

## 86. Kill Switch

Can disable behavior without redeploy.

---

## 87. Kill Switch Audit

Part 39/28.

---

## 88. Feature Flag Cleanup

Temporary rollout flags must have owner/expiry/deletion plan.

---

## 89. Long-Lived Flags

Technical debt signal.

---

## 90. No Flag Explosion

Governance.

---

## 91. Environment Promotion

Canonical sequence:

```text
ReleaseId
  ↓
development validation
  ↓
staging validation
  ↓
production promotion plan
```

---

## 92. Promotion Between Environments

Moves ReleaseId pointer/intent, not bytes.

---

## 93. Environment-Specific Configuration

Separate ConfigSnapshotId.

---

## 94. Runtime Secrets

Resolved per environment.

---

## 95. Same Binary, Different Runtime Config

Expected.

---

## 96. Build-Time Config

Cannot vary without new ReleaseId.

---

## 97. Promotion Evidence

```rust
pub struct PromotionEvidence {
    pub release: ReleaseId,
    pub source_environment: Option<EnvironmentId>,
    pub target_environment: EnvironmentId,
    pub prior_results: Vec<EvidenceRef>,
}
```

---

## 98. Staging Success

May be required but is not proof production will succeed.

---

## 99. Production Gate

Always evaluates production-specific constraints.

---

## 100. Promotion Approval

```rust
pub struct PromotionApproval {
    pub promotion: PromotionId,
    pub plan: PromotionPlanId,
    pub release: ReleaseId,
    pub approver: PrincipalId,
}
```

---

## 101. Approval Exactness

Plan/release change invalidates.

---

## 102. No Approval Reuse Across New Release

Critical.

---

## 103. Manual Gate

Can occur before:

```text
production start
wave advance
global rollout
high-risk cohort
```

---

## 104. Separation of Duties

Optional.

---

## 105. Regional Rollout

Example:

```text
Region A
  ↓
observe
  ↓
Region B
  ↓
observe
  ↓
Region C
```

---

## 106. Federation Integration

Part 51 supplies site/region authority and residency.

---

## 107. Regional Failure

Does not automatically advance next region.

---

## 108. Region Eligibility

Hard constraints:

```text
residency
Release availability
infrastructure readiness
compatibility
site health
```

---

## 109. Follow-the-Sun Rollout

Optional timing strategy.

---

## 110. Time Windows

Explicit local timezone.

---

## 111. No Implicit "nighttime"

Critical.

---

## 112. Maintenance Window

```rust
pub struct DeploymentWindow {
    pub timezone: TimeZoneId,
    pub schedule: CalendarSchedule,
}
```

---

## 113. Window Close During Rollout

Policy:

```text
pause
continue current wave
rollback
```

must be explicit.

---

## 114. Tenant Rollout

Useful SaaS model.

---

## 115. Tenant Cohort

Could be:

```text
internal tenants
pilot tenants
opt-in tenants
low-risk tenants
all tenants
```

---

## 116. Tenant Consent

Optional feature for beta programs.

---

## 117. Tenant Criticality

Can influence order.

---

## 118. No Cross-Tenant Leakage

Metrics/evidence aggregated safely.

---

## 119. Tenant-Specific Rollback

Possible only if architecture supports isolated version routing.

---

## 120. Shared Backend

May make per-tenant version rollback impossible.

---

## 121. Honest Capability

Critical.

---

## 122. Client/Mobile Rollout

Different.

App stores may control availability.

---

## 123. Device Rollout

Use:

```text
channel
cohort
minimum version
update metadata
```

Part 41 integration.

---

## 124. Client Rollback

Often difficult/impossible after app-store release.

---

## 125. Roll Forward Preference

May apply.

---

## 126. Desktop Agent Update

Part 41 supports staged client rollout.

---

## 127. Runner Agent Rollout

Part 58 has baseline rollout.

Part 62 can provide general promotion mechanics, but Part 58 remains host baseline authority.

---

## 128. Infrastructure Rollout

Part 53.

Infrastructure changes can be staged, but exact plan/state semantics remain infrastructure authority.

---

## 129. Database Migration

Special.

---

## 130. Schema Change Wave

Must follow expand-contract compatibility.

---

## 131. No Traffic Canary for Irreversible DB Change Without Design

Critical.

---

## 132. Migration Stage

Can be:

```text
expand
deploy compatible app
backfill
switch
contract later
```

---

## 133. Progressive Delivery Plan

Can coordinate these phases but not hide them.

---

## 134. Deployment Wave State Machine

```text
Planned
  ↓
Deploying
  ↓
Observing
  ↓
Passed
  ↓
next wave
```

Failure:

```text
Observing
  ├─ Paused
  ├─ Failed
  └─ RolledBack
```

---

## 135. Promotion State

```rust
pub enum PromotionState {
    Planned,
    AwaitingApproval,
    Running,
    Paused,
    RollingBack,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}
```

---

## 136. Unknown

Provider side effect uncertain.

---

## 137. Unknown Recovery

Inspect deployment/provider before retry.

---

## 138. No Blind Repeat Traffic Shift

Critical.

---

## 139. Traffic Shift Identity

```rust
pub struct TrafficShiftId(Ulid);
```

---

## 140. Traffic Shift

External side effect.

---

## 141. Expected State

Use provider precondition where available.

---

## 142. Reconcile

Observe actual routing weights.

---

## 143. Desired vs Observed

```text
desired: 25% canary
observed: 25%
```

---

## 144. Drift

If observed differs, pause/reconcile.

---

## 145. No "Assume Load Balancer Updated"

Critical.

---

## 146. Progressive Rollout Lock

Part 60 concurrency.

One active production promotion per protected environment by default.

---

## 147. Multiple Independent Components

Can deploy concurrently if scopes and dependencies allow.

---

## 148. Environment-Wide Lock

Too broad by default.

---

## 149. Concurrency Scope

Prefer:

```text
component + environment
```

unless shared state requires broader exclusion.

---

## 150. Dependency-Aware Rollout

Part 49/57.

---

## 151. Service Dependency

May require:

```text
server before client
database before app
API provider before consumer
```

---

## 152. Upgrade Order

Compatibility system authoritative.

---

## 153. Promotion DAG

```rust
pub struct PromotionDependency {
    pub before: PromotionStepId,
    pub after: PromotionStepId,
}
```

---

## 154. Cycles

Rejected.

---

## 155. Health Analysis

Use canonical observability sources.

---

## 156. Analysis Inputs

Examples:

```text
request success
latency
error rate
queue depth
CPU/memory saturation
business metric
deployment health
```

---

## 157. Business Metric

Can be important but high-risk if interpretation weak.

---

## 158. Metric Provenance

Exact query/version.

---

## 159. Dashboard Screenshot

Not sufficient canonical gate evidence.

---

## 160. MetricQueryRef

Versioned.

---

## 161. Missing Metrics

`Incomplete` / `Unknown`, not green.

---

## 162. Telemetry Delay

Analysis waits or fails according policy.

---

## 163. Monitoring Blind Spot

Can pause rollout.

---

## 164. RolloutMetricPolicyId

```rust
pub struct RolloutMetricPolicyId(Digest);
```

---

## 165. Baseline Selection

Must avoid comparing different traffic classes incorrectly.

---

## 166. Control Cohort

Useful for concurrent comparison.

---

## 167. Previous Release Historical

Useful when no control cohort.

---

## 168. Seasonality

Historical baseline may be misleading.

---

## 169. Inference Transparency

Canary analysis records method.

---

## 170. Automatic Analysis

Deterministic/statistical engine.

---

## 171. AI

Part 55 may explain rollout anomalies.

AI cannot decide protected advance/rollback unless bounded deterministic policy separately authorizes action.

---

## 172. Error Budget

Part 50.

---

## 173. Rollout Under Exhausted Budget

Policy may:

```text
block
require approval
allow security hotfix
```

---

## 174. Security Hotfix

May be allowed despite reliability freeze.

---

## 175. Audit

Explicit.

---

## 176. Incident Integration

Part 61.

Sev0/Sev1 active on environment may freeze normal rollout.

---

## 177. Rollout Causing Incident

Auto-link IncidentId.

---

## 178. Incident Resolution

Does not automatically resume rollout.

---

## 179. Explicit Resume

Requires freshness re-check.

---

## 180. Pause

```rust
pub struct PromotionPause {
    pub promotion: PromotionId,
    pub reason: PromotionPauseReason,
}
```

---

## 181. Pause Reasons

```rust
pub enum PromotionPauseReason {
    GateFailed,
    HealthUnknown,
    Incident,
    Manual,
    ChangeFreeze,
    ProviderUncertain,
    Compatibility,
    Security,
}
```

---

## 182. Resume Preconditions

Re-evaluate:

```text
release validity
environment state
health
policy
compatibility
incident state
```

---

## 183. No Resume Based on Old Passed Gates

Critical if evidence stale.

---

## 184. Cancellation

Stops future waves.

---

## 185. Active Wave

Must reconcile current external effect.

---

## 186. Cancel != Rollback

Critical.

---

## 187. Rollback Intent

Separate.

---

## 188. Promotion Cancellation

Can leave partial rollout.

---

## 189. Partial Rollout State

Explicit.

---

## 190. PartialRollout

```rust
pub struct PartialRolloutState {
    pub completed_waves: Vec<RolloutWaveId>,
    pub current_wave: Option<RolloutWaveId>,
}
```

---

## 191. Cleanup

Policy chooses rollback/hold/roll-forward.

---

## 192. Release Revocation

If ReleaseId revoked:

```text
pause
prevent new waves
evaluate rollback/containment
```

---

## 193. Registry Availability

Part 52.

Every target site must access exact artifacts.

---

## 194. Artifact Replication

Part 51.

Can pre-stage bytes before rollout.

---

## 195. Pre-Positioning

Optimization.

---

## 196. No Deployment Before Digest Verification

Critical.

---

## 197. Release Channel

Promotion can move channel alias after successful rollout.

---

## 198. Channel Alias

Mutable pointer.

---

## 199. Alias Update

After target rollout policy says complete.

---

## 200. Do Not Move Stable Alias Before Rollout Actually Meets Policy

Critical.

---

## 201. Rollout Completion

```rust
pub struct PromotionCompletion {
    pub promotion: PromotionId,
    pub release: ReleaseId,
    pub completed_at: Timestamp,
    pub evidence: Vec<EvidenceRef>,
}
```

---

## 202. Rollback Identity

Rollback deploys exact previous approved ReleaseId.

---

## 203. No Rebuild Previous Version

Critical.

---

## 204. PreviousRelease Selection

Must be exact.

---

## 205. Rollback Candidate

```rust
pub struct RollbackTarget {
    pub release: ReleaseId,
    pub compatibility: CompatibilityReportId,
}
```

---

## 206. Rollback Safety

Check:

```text
DB/schema state
runtime config compatibility
external API state
infrastructure state
```

---

## 207. If Incompatible

Roll forward/manual.

---

## 208. Audit Events

```text
promotion start
manual approval
wave advance
automatic pause
manual pause
rollback decision
rollback execution
resume
cancel
blast-radius override
```

---

## 209. Blast Radius Override

High privilege.

---

## 210. No Silent Override

Critical.

---

## 211. Notifications

Examples:

```text
promotion awaiting approval
canary passed
canary failed
rollout paused
rollback started
promotion complete
```

---

## 212. Dioxus UI

Pages:

```text
Promotions
Progressive Rollouts
Canary Analysis
Feature Rollouts
Rollout History
```

---

## 213. Promotion Detail

Shows:

```text
ReleaseId
environment
strategy
current wave
population
gate status
health metrics
approvals
```

---

## 214. Wave Timeline

Visual.

---

## 215. Canary Comparison

Shows candidate vs baseline.

---

## 216. Evidence Links

Canonical.

---

## 217. Rollback Warning

Shows compatibility/recovery limitations.

---

## 218. CLI

```text
forgeyard promote plan
forgeyard promote start
forgeyard promote status
forgeyard promote pause
forgeyard promote resume
forgeyard promote approve
forgeyard promote rollback
forgeyard promote cancel
forgeyard promote explain
forgeyard promote doctor
```

---

## 219. API

Potential:

```text
POST /v1/promotions
GET  /v1/promotions/{id}
POST /v1/promotions/{id}/start
POST /v1/promotions/{id}/pause
POST /v1/promotions/{id}/resume
POST /v1/promotions/{id}/rollback
POST /v1/promotions/{id}/cancel
```

---

## 220. Permissions

```text
promotion.read
promotion.plan
promotion.start
promotion.approve
promotion.pause
promotion.resume
promotion.rollback
promotion.cancel
promotion.blast_radius.override
```

---

## 221. Rollback Permission

High privilege for production unless automated policy path.

---

## 222. Automated Rollback Identity

System service principal.

---

## 223. Automated Action Audit

Explicit trigger/evidence.

---

## 224. Promotion Explain

Must answer:

```text
why is this paused?
why did it advance?
why did it rollback?
which evidence was used?
```

---

## 225. Observability Metrics

```text
promotion_total
promotion_wave_total
promotion_wave_failures_total
promotion_rollbacks_total
promotion_pause_total
promotion_duration_seconds
canary_analysis_inconclusive_total
```

---

## 226. Labels

Low cardinality:

```text
strategy
environment_kind
result
```

---

## 227. Tracing

```text
promotion.plan
promotion.gate
promotion.deploy
promotion.observe
promotion.advance
promotion.pause
promotion.rollback
promotion.reconcile
```

---

## 228. Health

```rust
pub enum PromotionSubsystemHealth {
    Healthy,
    AnalysisDegraded,
    DeploymentDegraded,
    ProviderDegraded,
    Unhealthy,
}
```

---

## 229. Analysis Backend Down

Cannot claim gate pass.

---

## 230. Existing Rollout

Pause according policy.

---

## 231. Doctor

```text
forgeyard promote doctor
```

Checks:

```text
stuck promotions
stale gate evidence
partial rollouts
observed traffic drift
invalid rollback target
active rollout during change freeze
missing metrics
```

---

## 232. Search/Analytics

Part 31.

Useful:

```text
rollout duration
pause reasons
rollback frequency
canary inconclusive rate
```

---

## 233. No Developer Ranking

Critical.

---

## 234. Cost

Part 45.

Progressive delivery can increase temporary capacity due parallel versions.

---

## 235. Cost Estimate

Advisory.

---

## 236. Cost Cannot Skip Required Canary

---

## 237. Capacity

Scheduler/fleet must support both versions during canary/blue-green.

---

## 238. Capacity Precheck

Before rollout.

---

## 239. No Half-Provisioned Canary

Critical.

---

## 240. Infrastructure Readiness

Part 53 gate.

---

## 241. Network Readiness

Part 59.

---

## 242. Secrets Readiness

Part 12.

---

## 243. Compatibility Readiness

Part 57.

---

## 244. Release Trust

Part 15/13.

---

## 245. Concurrency

Part 60.

Promotion has protected mutation scope.

---

## 246. Data Lifecycle

Part 46.

Retain:

```text
promotion plan
wave history
gate evidence
rollback evidence
approvals
```

---

## 247. Federation

Regional promotion state has explicit authority.

---

## 248. Site Failure Mid-Wave

Pause/reconcile.

---

## 249. Region Unreachable

Unknown until observed/reconciled.

---

## 250. No Assume Failure == Rollback Completed

Critical.

---

## 251. Air-Gap

Progressive rollout works within local site.

---

## 252. Remote Metrics

May be unavailable.

---

## 253. Gate Policy

Can use local observability only.

---

## 254. DR

Promotion state backed up.

---

## 255. After Restore

Reconcile target environment actual versions/traffic before resuming.

---

## 256. Never Replay Wave Blindly

Critical.

---

## 257. Update Delivery

Part 41.

Client agent rollout can consume same wave model conceptually.

---

## 258. Artifact Registry

Part 52 serves exact bytes.

---

## 259. Runner Images

Part 58 specialized baseline rollout may reuse generic analysis primitives.

---

## 260. No Duplicate Authority

Runner-image baseline state remains Part 58.

---

## 261. Feature Flags

Part 39 remains flag authority.

---

## 262. Progressive Delivery

Coordinates rollout intent.

---

## 263. Flag State Projection

Read/observe from flag subsystem.

---

## 264. Rollout History

Append-only.

---

## 265. Rollout Event

```rust
pub enum PromotionEvent {
    Planned,
    Started,
    WaveStarted,
    GateEvaluated,
    WavePassed,
    WaveFailed,
    Paused,
    Resumed,
    RollbackStarted,
    RollbackCompleted,
    Completed,
    Cancelled,
}
```

---

## 266. Events

At-least-once.

---

## 267. Persisted State Authority

Postgres/Stoolap.

---

## 268. Reconciliation

Required.

---

## 269. HA

Multiple promotion workers safe.

---

## 270. Wave Claim

Lease.

---

## 271. Stale Controller

Fenced.

---

## 272. No Duplicate Traffic Shift

Use Part 60 idempotency/preconditions.

---

## 273. Security Threats

```text
blast-radius bypass
stale health data
manual hidden advance
wrong cohort
rollback to vulnerable release
traffic-shift race
tenant cohort leak
```

---

## 274. Controls

```text
exact release identity
exact cohort snapshot
policy-bound gates
security minimum version
fencing
audit
```

---

## 275. Rollback Security Floor

Never rollback to revoked/vulnerable release without explicit exceptional security decision.

---

## 276. Feature Flag Security

A disabled dangerous feature may still leave vulnerable code deployed.

Security decision independent.

---

## 277. Canary Security

Canary tenant/user selection must not exploit vulnerable/less-protected populations.

---

## 278. Testkit

```text
forgeyard-rollout-testkit/src/
├── lib.rs
├── plan.rs
├── wave.rs
├── cohort.rs
├── gate.rs
├── canary.rs
├── rollback.rs
├── federation.rs
└── assertions.rs
```

---

## 279. Core Tests

### Identity
- promotion binds exact ReleaseId;
- rollout strategy identity deterministic;
- cohort assignment stable.

### No Rebuild
- promoted artifact digest identical across environments.

### Gate
- Unknown/Incomplete cannot advance;
- stale evidence rejected;
- approval binds exact plan/release.

### Canary
- percentage cap enforced;
- metric window exact;
- control/candidate populations stable;
- insufficient sample → Inconclusive.

### Rollback
- safe rollback executes exact previous release;
- incompatible rollback blocked;
- revoked release cannot be automatic rollback target.

### Traffic
- desired/observed routing reconciled;
- timeout becomes Unknown;
- no blind duplicate shift.

### Incident
- active severe incident pauses normal rollout;
- incident resolution does not auto-resume stale rollout.

### Federation
- region sequence obeys residency/authority;
- site outage pauses/reconciles.

### DR
- restored promotion inspects actual environment before resuming.

---

## 280. Chaos Tests

Inject:

```text
load balancer API timeout
metrics backend outage
region partition
deployment controller crash
SLO data delay
incident declared mid-wave
```

Verify safe pause/reconciliation.

---

## 281. Scale Tests

Test:

```text
many tenant cohorts
many services/promotions
large regional rollouts
high event volume
```

---

## 282. Implementation Phases

### Phase 1 — Promotion Model
Release/environment/wave identity.

### Phase 2 — Manual Staged Promotion
Dev/staging/prod.

### Phase 3 — Canary + Percentage Rollout
Traffic wave support.

### Phase 4 — Health/SLO Gates
Automated analysis.

### Phase 5 — Pause/Resume/Rollback
Controlled automation.

### Phase 6 — Feature Flag Assisted Rollout
Behavior activation.

### Phase 7 — Regional/Tenant Cohorts
Enterprise/SaaS.

### Phase 8 — Compatibility/Infrastructure/Incident Gates
Cross-system safety.

### Phase 9 — Federation & Air-Gap
Distributed rollout.

### Phase 10 — Client/Device Rollout Integration
Update delivery.

### Phase 11 — Analytics/UI/Doctor
Operability.

### Phase 12 — Chaos/Scale/Security Hardening
Production readiness.

---

## 283. Acceptance Tests

1. Promotion always binds exact ReleaseId.
2. Promotion never rebuilds artifacts between environments.
3. Build-time configuration changes require a new release.
4. Runtime environment config remains separate.
5. Rollout plan is immutable/versioned.
6. Every rollout wave has exact target identity.
7. Dynamic cohorts retain exact evaluated membership when needed.
8. Percentage assignment is deterministic/stable.
9. Blast-radius limits are hard constraints.
10. Unknown/incomplete gate outcome cannot advance automatically.
11. Gate evidence binds exact release/environment/wave/policy.
12. Minimum observation windows are enforced.
13. Missing metrics never become green.
14. Automated rollback occurs only where explicitly safe/allowed.
15. Cancel is distinct from rollback.
16. Partial rollout state is explicit.
17. Rollback deploys exact previous ReleaseId without rebuilding.
18. Revoked/vulnerable release is not automatic rollback target.
19. Staging success does not automatically prove production readiness.
20. Production-specific policy is re-evaluated.
21. Manual approval binds exact release/plan.
22. Feature flags do not become release authority.
23. Feature flag activation is separate from code deployment.
24. Regional rollout obeys residency and site authority.
25. Tenant rollout cannot leak cross-tenant metrics/details.
26. Database/schema compatibility is checked before rollback.
27. Desired/observed traffic shift is reconciled.
28. Provider timeout becomes Unknown rather than blind retry.
29. Incident/change-freeze state can pause rollout.
30. Incident resolution does not automatically resume without freshness checks.
31. Capacity/infrastructure/network readiness can block rollout.
32. HA workers cannot duplicate wave effects.
33. DR inspects actual environment before resume.
34. Standalone/distributed share promotion semantics.
35. Forgeyard dogfoods progressive delivery for its own services/components.

---

## 284. Production Readiness Gates

Do not call progressive delivery production-ready until:

```text
same-bytes promotion is enforced
wave/cohort identity is stable
blast-radius limits are machine-enforced
gate freshness is enforced
canary metric analysis handles missing/inconclusive data safely
traffic-shift reconciliation is proven
rollback compatibility/security checks work
incident/change-freeze integration works
HA/DR/federation tests pass
chaos tests pass
```

---

## 285. Architectural Invariants

1. promotion uses same immutable release bytes;
2. rollout strategy does not change release identity;
3. each wave has exact target identity;
4. cohorts are deterministic/auditable;
5. blast radius is bounded;
6. Unknown is not Pass;
7. evidence freshness matters;
8. automated advance requires all required gates;
9. automated rollback requires known-safe rollback semantics;
10. cancel is not rollback;
11. partial rollout is explicit;
12. rollback never rebuilds;
13. security floor constrains rollback;
14. staging evidence does not replace production evidence;
15. approvals bind exact plan/release;
16. feature flags do not become release authority;
17. runtime config stays separate from build identity;
18. traffic shifts are reconciled external effects;
19. provider timeout becomes Unknown;
20. incident/change freeze can stop progression;
21. resume re-checks freshness;
22. compatibility governs upgrade/rollback order;
23. infrastructure/network/secrets readiness are explicit gates;
24. federation/residency constrain regional rollout;
25. tenant metrics remain isolated;
26. HA uses fenced/idempotent wave execution;
27. DR observes actual state before continuing;
28. lifecycle retains promotion evidence;
29. AI remains advisory;
30. Forgeyard dogfoods its own progressive-delivery subsystem.

---

## 286. Final Target Architecture

```text
                       ReleaseId
                           │
                           ▼
                    PromotionPlanId
                           │
                           ▼
                        Wave 1
                           │
                     deploy/observe
                           │
                           ▼
                      Gate Evidence
                           │
                 ┌─────────┼─────────┐
                 ▼         ▼         ▼
              Advance     Pause    Rollback
                 │
                 ▼
               Wave 2
                 │
                 ▼
                ...
                 │
                 ▼
             Fully Promoted
```

Canary:

```text
stable population
      +
candidate cohort
      ↓
same observation window
      ↓
health/SLO/metric comparison
      ↓
Pass / Fail / Inconclusive
```

Promotion:

```text
ReleaseId
  ↓
dev
  ↓
staging
  ↓
production canary
  ↓
production waves
  ↓
same exact artifact digests
```

The key guarantee is:

> **Forgeyard can make delivery progressively safer by limiting blast radius and advancing only on fresh evidence, while still preserving the fundamental release invariant: build once, verify once, promote the same bytes. Progressive delivery controls exposure and timing; it never creates a second build, release, policy, compatibility, or deployment authority.**

---

## 287. Extended Architecture Sequence

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
```
