# 66 — Forgeyard Change Risk Assessment, Preflight Simulation, Policy Preview & What-If Analysis System Architecture

**Document type:** Core Change Risk Assessment, Preflight Simulation, Policy Preview, What-If Planning, Impact Forecasting & Safe Change Analysis System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** preflight evaluation, hypothetical plan execution, change-risk classification, policy simulation, deployment impact preview, infrastructure plan comparison, migration risk preview, compatibility impact, affected-work expansion, security risk analysis, resource/cost forecast, blast-radius estimation, approval preview, and explainable decision support  
**Architecture style:** Deterministic first, evidence-backed, read-only simulation, exact immutable subjects, explicit uncertainty, no side effects, policy-as-data evaluation, typed risk facts, bounded forecast confidence, and strict separation between simulation and authority  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Pipeline Planning, Policy/Authz, Static Analysis, Compatibility Governance, Monorepo Intelligence, Infrastructure-as-Code, Deployment, Progressive Delivery, Database Migration, Cost/FinOps, Reliability, Security, Incident Management, Merge Queue, and AI Assistance. This subsystem provides a unified “what will happen if we do this?” layer before protected changes execute.

---

## 1. Purpose

Forgeyard already understands many change dimensions:

```text
source changes
pipeline changes
dependency changes
policy changes
config changes
infrastructure changes
database migrations
deployment changes
release changes
runner-image changes
network changes
```

Before execution, engineers need answers such as:

```text
What will this change affect?
Which pipelines/jobs will run?
Which components are at risk?
Will this violate policy?
Will this break API/ABI compatibility?
What environments may be impacted?
Will deployment require downtime?
Will this migration rewrite a large table?
How much will this cost?
What approvals will be required?
What could block the merge?
What rollout strategy is recommended?
```

The central rule is:

> **Preflight analysis is a read-only simulation of intended effects against an exact immutable change subject and current known system state. It must never perform the protected external effect it is analyzing.**

A second rule is:

> **Risk is represented as typed facts with provenance and uncertainty, not a magical opaque score.**

A third rule is:

> **A simulation result can explain likely consequences and policy outcomes, but real execution evidence remains authoritative once the change actually runs.**

---

## 2. Architectural Position

```text
                    Proposed Change
                         │
                         ▼
                  Preflight Subject
                         │
                         ▼
                 Deterministic Planning
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Policy         Impact         Risk Facts
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                   What-If Result
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Explain     Approvals   Blockers
                         │
                         ▼
                  Human / Merge / CI
```

---

## 3. Goals

The subsystem MUST:

1. define preflight identity;
2. define simulation subject identity;
3. support source-change preflight;
4. support pipeline preflight;
5. support policy preview;
6. support compatibility preview;
7. support infrastructure preview;
8. support deployment preview;
9. support migration preview;
10. support cost/resource forecast;
11. support security impact analysis;
12. support affected-work impact;
13. support blast-radius estimation;
14. support approval prediction;
15. support merge-blocker prediction;
16. support rollout recommendation inputs;
17. support explicit uncertainty;
18. support historical comparison;
19. support scenario comparison;
20. support “what if config X changes?”;
21. support “what if policy Y changes?”;
22. support UI/API/CLI;
23. support audit;
24. support caching;
25. support multi-tenancy;
26. support federation;
27. support air-gap;
28. support deterministic reproducibility;
29. remain side-effect free;
30. never replace real execution evidence.

---

## 4. Non-Goals

This subsystem does not:

```text
execute deployments
apply infrastructure
run database migrations
approve changes
merge code
publish releases
sign artifacts
guarantee future production behavior
```

---

## 5. Workspace Structure

```text
crates/preflight/
├── forgeyard-preflight/
├── forgeyard-preflight-model/
├── forgeyard-preflight-subject/
├── forgeyard-preflight-impact/
├── forgeyard-preflight-policy/
├── forgeyard-preflight-risk/
├── forgeyard-preflight-cost/
├── forgeyard-preflight-security/
├── forgeyard-preflight-compatibility/
├── forgeyard-preflight-scenario/
├── forgeyard-preflight-explain/
├── forgeyard-preflight-cache/
├── forgeyard-preflight-health/
└── forgeyard-preflight-testkit/
```

---

## 6. PreflightId

```rust
pub struct PreflightId(Ulid);
```

One preflight evaluation.

---

## 7. PreflightPlanId

```rust
pub struct PreflightPlanId(Digest);
```

Immutable identity of the exact simulation inputs.

---

## 8. Preflight Subject

```rust
pub enum PreflightSubject {
    SourceChange(ChangeProposalRevisionId),
    IntegrationCandidate(IntegrationCandidateId),
    Pipeline(PipelineIrId),
    Release(ReleaseId),
    Deployment(DeploymentPlanId),
    Infrastructure(InfrastructurePlanId),
    Migration(MigrationPlanId),
    Config(ConfigSnapshotId),
    Policy(PolicyDigest),
    RunnerImage(RunnerImageDefinitionId),
    NetworkPolicy(NetworkCapabilityId),
}
```

---

## 9. Exact Subject

Branch names, mutable environment aliases, and “latest” are resolved before simulation.

---

## 10. Simulation Context

```rust
pub struct PreflightContext {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub environment: Option<EnvironmentId>,
    pub config: ConfigSnapshotId,
    pub policy: PolicyDigest,
    pub catalog_snapshot: CatalogSnapshotId,
}
```

---

## 11. Current State Snapshot

Simulation may depend on mutable current state.

Therefore capture exact point-in-time snapshot references where possible.

---

## 12. PreflightContextId

```rust
pub struct PreflightContextId(Digest);
```

---

## 13. Side-Effect-Free Invariant

Preflight adapters MUST not:

```text
deploy
publish
merge
apply
delete
rotate
sign
mutate production state
```

---

## 14. Read-Only Provider Calls

Allowed where required.

---

## 15. Provider Plan APIs

Allowed if provider contract guarantees read-only planning.

---

## 16. Unknown Provider Side Effect

If provider “plan” API can mutate, adapter must not use it in normal preflight.

---

## 17. Risk Fact

```rust
pub struct RiskFact {
    pub kind: RiskFactKind,
    pub severity: RiskSeverity,
    pub confidence: RiskConfidence,
    pub evidence: Vec<EvidenceRef>,
}
```

---

## 18. RiskFactKind

```rust
pub enum RiskFactKind {
    CompatibilityBreak,
    DestructiveSchemaChange,
    LargeBackfill,
    SecurityBoundaryChange,
    PrivilegeExpansion,
    NetworkExpansion,
    ProductionBlastRadius,
    IrreversibleChange,
    HighCost,
    LowRollbackConfidence,
    SloRisk,
    CapacityRisk,
    DataResidencyRisk,
    DependencyRisk,
    UnknownImpact,
}
```

---

## 19. Risk Severity

```rust
pub enum RiskSeverity {
    Informational,
    Low,
    Moderate,
    High,
    Critical,
}
```

---

## 20. Risk Confidence

```rust
pub enum RiskConfidence {
    Confirmed,
    Strong,
    Moderate,
    Weak,
    Unknown,
}
```

---

## 21. No Single Magic Score

Critical.

---

## 22. Optional Aggregate Risk Profile

```rust
pub struct RiskProfile {
    pub facts: Vec<RiskFact>,
    pub highest_severity: RiskSeverity,
}
```

---

## 23. Weighted Score

Can exist for sorting only.

Never sole policy authority.

---

## 24. Source Change Impact

Uses Part 34 dependency graph.

---

## 25. Affected Components

```rust
pub struct AffectedComponent {
    pub component: SoftwareComponentId,
    pub reason: ImpactReason,
    pub confidence: ImpactConfidence,
}
```

---

## 26. Impact Reasons

```text
direct file change
dependency edge
generated output
schema dependency
API consumer
deployment dependency
```

---

## 27. Unknown Dependency

Broadens impact.

---

## 28. No Optimistic Underestimation

Critical.

---

## 29. Affected Work

Part 34 authoritative.

---

## 30. Pipeline Preflight

Can answer:

```text
which jobs will exist
which matrix cells expand
which runners are required
which secrets are referenced
which network capabilities are needed
```

---

## 31. Pipeline Compile

Uses normal Part 04 compiler.

---

## 32. No Alternate Simulation Parser

Critical.

---

## 33. Job Count Forecast

Exact if conditions/matrix fully known.

---

## 34. Dynamic Runtime Conditions

May remain uncertain.

---

## 35. Resource Forecast

```rust
pub struct ResourceForecast {
    pub cpu_seconds: ForecastRange<u64>,
    pub memory_peak: ForecastRange<MemoryBytes>,
    pub gpu_seconds: ForecastRange<u64>,
}
```

---

## 36. Forecast Range

```rust
pub struct ForecastRange<T> {
    pub low: T,
    pub expected: T,
    pub high: T,
    pub confidence: ForecastConfidence,
}
```

---

## 37. No Exact Prediction Without Evidence

Critical.

---

## 38. Historical Basis

Part 33/45.

---

## 39. Cost Forecast

```rust
pub struct CostForecast {
    pub expected: Money,
    pub low: Money,
    pub high: Money,
    pub basis: CostForecastBasis,
}
```

---

## 40. Cost Basis

```text
historical jobs
provider pricing snapshot
runner class
estimated runtime
storage/egress
```

---

## 41. Pricing Snapshot

Exact.

---

## 42. Cost Is Forecast

Not invoice.

---

## 43. Policy Preview

Given hypothetical subject/context:

```text
which rules match?
which approvals required?
which gates fail?
which exceptions apply?
```

---

## 44. PolicyPreviewId

```rust
pub struct PolicyPreviewId(Digest);
```

---

## 45. Policy Explanation

```rust
pub struct PolicyPreview {
    pub decision: PolicyDecision,
    pub matched_rules: Vec<PolicyRuleRef>,
    pub required_approvals: Vec<ApprovalRequirement>,
}
```

---

## 46. Preview Decision

Not authorization token.

---

## 47. Critical Rule

Policy preview never authorizes future execution.

Real execution re-evaluates current policy.

---

## 48. Policy Change What-If

Compare:

```text
current PolicyDigest
vs
candidate PolicyDigest
```

---

## 49. Show

```text
newly allowed actions
newly denied actions
approval changes
affected projects/environments
```

---

## 50. Privilege Expansion

High-risk fact.

---

## 51. Security What-If

Can analyze:

```text
new permissions
network expansion
secret scope expansion
runner trust reduction
unsigned artifact allowance
policy exception
```

---

## 52. Security Authority

Parts 11/40 remain authoritative.

---

## 53. Compatibility Preview

Part 57.

---

## 54. Contract Diff

Can be part of preflight.

---

## 55. Breaking Change

Typed risk fact.

---

## 56. Consumer Impact

Known/unknown consumers included.

---

## 57. Infrastructure Preflight

Part 53 plan is primary evidence.

---

## 58. Infrastructure Risk Facts

Examples:

```text
resource deletion
database replacement
public endpoint exposure
IAM expansion
region move
storage destruction
```

---

## 59. No Apply

Critical.

---

## 60. Infrastructure Cost Forecast

Can use provider plan estimates.

---

## 61. Deployment Preflight

Can answer:

```text
target release
current release
compatibility
rollback target
blast radius
required rollout strategy
```

---

## 62. Deployment Health

Current environment observed state included.

---

## 63. Future Health

Cannot be known exactly.

---

## 64. Rollback Readiness

```rust
pub enum RollbackReadiness {
    Ready,
    Conditional,
    Unsafe,
    Unknown,
}
```

---

## 65. Migration Preflight

Part 63.

---

## 66. Migration Risk Facts

```text
table rewrite
large backfill
lock risk
irreversible transform
old client incompatibility
insufficient backup
```

---

## 67. Backfill Estimate

Can use row counts/table size.

---

## 68. Estimated Duration

Range only.

---

## 69. Schema Drift

Blocks reliable plan.

---

## 70. Progressive Delivery Recommendation Inputs

Preflight may recommend:

```text
immediate
canary
regional
tenant pilot
manual approval
maintenance window
```

---

## 71. Recommendation Is Advisory

Part 62 policy decides.

---

## 72. Blast Radius Preview

```rust
pub struct BlastRadiusPreview {
    pub components: Vec<SoftwareComponentId>,
    pub environments: Vec<EnvironmentId>,
    pub tenants: ImpactSet<TenantId>,
    pub regions: Vec<RegionId>,
}
```

---

## 73. Unknown Population

Represented explicitly.

---

## 74. No "0 affected" When Catalog Is Incomplete

Critical.

---

## 75. Approval Preview

Can answer:

```text
security approval
release approval
migration cutover approval
production deployment approval
break-glass requirement
```

---

## 76. Approval Requirement

Derived from policy.

---

## 77. Approval Preview Freshness

Becomes stale if:

```text
policy changes
subject changes
environment changes
```

---

## 78. Merge Blocker Preview

Can combine:

```text
tests
compatibility
security
policy
ownership
required approvals
```

---

## 79. Merge Queue

Part 54 remains authority.

---

## 80. Preflight On Integration Candidate

Preferred for final merge prediction.

---

## 81. Proposal Head vs Candidate

Distinct.

---

## 82. Historical Comparison

Compare new preflight against prior accepted change.

---

## 83. ChangeDelta

```rust
pub struct PreflightDelta {
    pub added_risks: Vec<RiskFact>,
    pub removed_risks: Vec<RiskFact>,
    pub changed_impacts: Vec<ImpactChange>,
}
```

---

## 84. ScenarioId

```rust
pub struct ScenarioId(Ulid);
```

---

## 85. What-If Scenario

```rust
pub struct WhatIfScenario {
    pub base: PreflightContextId,
    pub modifications: Vec<HypotheticalChange>,
}
```

---

## 86. HypotheticalChange

```rust
pub enum HypotheticalChange {
    Config(ConfigPatch),
    Policy(PolicyPatch),
    RunnerClass(CapacityClassId),
    Region(RegionId),
    RolloutStrategy(RolloutStrategy),
    ResourceLimit(ResourceOverride),
}
```

---

## 87. Scenario Must Be Non-Mutating

Critical.

---

## 88. Scenario Compare

Useful:

```text
What if we use 8-core runners instead of 4?
What if production requires canary?
What if this policy blocks public egress?
```

---

## 89. Scenario Result

New immutable preflight result.

---

## 90. No “Apply Scenario” Shortcut

If chosen, convert to normal canonical change workflow.

---

## 91. Preflight Result

```rust
pub struct PreflightResult {
    pub id: PreflightId,
    pub plan: PreflightPlanId,
    pub risks: Vec<RiskFact>,
    pub impacts: ImpactSummary,
    pub policy: PolicyPreview,
    pub cost: Option<CostForecast>,
    pub blockers: Vec<PreflightBlocker>,
}
```

---

## 92. Preflight Blocker

```rust
pub enum PreflightBlocker {
    PolicyDenied,
    MissingEvidence,
    CompatibilityBreaking,
    InfrastructureUnknown,
    SchemaDrift,
    SecurityCritical,
    UnsupportedOperation,
}
```

---

## 93. Warning vs Blocker

Explicit.

---

## 94. Preflight Completeness

```rust
pub enum PreflightCompleteness {
    Complete,
    Partial,
    Incomplete,
}
```

---

## 95. Missing Inputs

List explicitly.

---

## 96. No Green Summary On Incomplete High-Risk Analysis

Critical.

---

## 97. Evidence Provenance

Every finding references:

```text
source graph
policy rule
compatibility report
infra plan
migration plan
cost snapshot
observability snapshot
```

---

## 98. Explainability

User can ask:

```text
why high risk?
why approval required?
why this component affected?
why estimated cost increased?
```

---

## 99. Explain Graph

```rust
pub struct ExplanationEdge {
    pub from: EvidenceRef,
    pub relation: ExplanationRelation,
    pub to: PreflightFindingRef,
}
```

---

## 100. No Opaque Black-Box Block

Critical.

---

## 101. AI Assistance

Part 55 may summarize preflight.

---

## 102. AI Cannot Create Canonical Risk Facts Without Evidence

---

## 103. AI Suggested Risk

Must be labeled advisory.

---

## 104. Deterministic Risk Fact

Preferred for gates.

---

## 105. Simulation Cache

Preflight can be cached by:

```text
subject
context
policy
catalog snapshot
provider state snapshot
engine version
```

---

## 106. PreflightCacheKey

```rust
pub struct PreflightCacheKey(Digest);
```

---

## 107. Cache Freshness

Critical.

---

## 108. Mutable Inputs

Provider/environment state changes invalidate.

---

## 109. TTL

Short for operational state.

---

## 110. Long-Lived Static Analysis

Reusable.

---

## 111. No Stale Preflight As Execution Authority

Critical.

---

## 112. Real Execution Rechecks

At protected action time.

---

## 113. Current State Changes

Preflight may become stale.

---

## 114. PreflightFreshness

```rust
pub enum PreflightFreshness {
    Current,
    SubjectChanged,
    PolicyChanged,
    EnvironmentChanged,
    CatalogChanged,
    ProviderStateChanged,
    Expired,
    Unknown,
}
```

---

## 115. Merge Queue Integration

Final candidate preflight can be required before landing.

---

## 116. Release Integration

Release preflight can summarize:

```text
compatibility
supply-chain
promotion
rollback
```

---

## 117. Deployment Integration

Preflight may produce deployment plan preview.

---

## 118. Migration Integration

Preflight may produce migration risk preview.

---

## 119. Runner Image Integration

Can preview:

```text
fleet impact
security generation
rollout risk
```

---

## 120. Network Policy Integration

Can preview newly reachable destinations.

---

## 121. Secrets Integration

Can preview newly referenced secret scopes, not secret values.

---

## 122. Configuration Integration

Can compare effective config.

---

## 123. Config Diff

Typed.

---

## 124. No Secret Value In Diff

Critical.

---

## 125. Multi-Tenancy

Impact queries tenant-scoped.

---

## 126. Cross-Tenant Global Change

System-level only.

---

## 127. Tenant Risk

Can show:

```text
number of affected tenants
critical tenants
pilot eligibility
```

without exposing identities where user lacks permission.

---

## 128. Federation

Preflight considers:

```text
region/site health
residency
authority
artifact availability
```

---

## 129. Disconnected Site

Global preflight may be incomplete.

---

## 130. Incomplete Connectivity

Explicit.

---

## 131. Air-Gap

Local simulation works with local evidence.

---

## 132. External Provider Forecast

Unavailable offline.

---

## 133. Cost

Can be incomplete offline.

---

## 134. Security

Current policy always applies to viewing sensitive impact details.

---

## 135. Preflight Data Leakage

Potential threat.

---

## 136. Example

User can infer existence of secret environment/tenant.

---

## 137. Response Redaction

Impact summaries permission-filtered.

---

## 138. No Authorization Oracle

Critical.

---

## 139. Policy Preview Visibility

Show only rules caller is allowed to inspect.

---

## 140. Still explain decision safely.

---

## 141. Audit

Audit:

```text
privileged policy what-if
production infrastructure preflight
security-sensitive impact preview
blast-radius override scenario
```

---

## 142. Routine Preflight

Operational event.

---

## 143. Dioxus UI

Pages/panels:

```text
Preflight
Risk
Impact
Policy Preview
Cost Forecast
Scenario Compare
```

---

## 144. Change Proposal UI

Preflight tab.

---

## 145. Shows

```text
affected components
required checks
risk facts
required approvals
estimated cost
rollout suggestion
blockers
```

---

## 146. Risk Visualization

Do not collapse everything into color only.

---

## 147. Accessibility

Text labels.

---

## 148. Scenario Compare UI

Side-by-side.

---

## 149. CLI

```text
forgeyard preflight change <proposal>
forgeyard preflight candidate <candidate>
forgeyard preflight deploy <plan>
forgeyard preflight migration <plan>
forgeyard preflight policy
forgeyard preflight scenario
forgeyard preflight explain
forgeyard preflight doctor
```

---

## 150. API

Potential:

```text
POST /v1/preflight
GET  /v1/preflight/{id}
POST /v1/preflight/scenarios
GET  /v1/preflight/{id}/explain
```

---

## 151. Permissions

```text
preflight.read
preflight.run
preflight.scenario
preflight.policy_preview
preflight.sensitive_impact
```

---

## 152. Policy Preview

May require elevated permission.

---

## 153. Observability Metrics

```text
preflight_total
preflight_blocked_total
preflight_incomplete_total
preflight_cache_hit_total
preflight_duration_seconds
```

---

## 154. Labels

Low-cardinality:

```text
subject_kind
result
completeness
```

---

## 155. Tracing

```text
preflight.resolve
preflight.impact
preflight.policy
preflight.compatibility
preflight.cost
preflight.explain
```

---

## 156. Health

```rust
pub enum PreflightSubsystemHealth {
    Healthy,
    ImpactDegraded,
    PolicyDegraded,
    ProviderReadDegraded,
    IncompleteData,
    Unhealthy,
}
```

---

## 157. Doctor

```text
forgeyard preflight doctor
```

Checks:

```text
stale catalog snapshot
missing compatibility adapter
provider plan access unavailable
cost snapshot stale
policy evaluator mismatch
cache invalidation failures
```

---

## 158. Reliability

Preflight failure should not mutate target state.

---

## 159. Protected Change

May block if policy requires successful preflight.

---

## 160. Optional Change

Can proceed without forecast if policy allows.

---

## 161. No Side Effect On Failure

Critical.

---

## 162. Data Lifecycle

Preflight results can be retained as:

```text
merge evidence
release evidence
audit evidence
```

---

## 163. Scenario Drafts

Short retention.

---

## 164. Sensitive Impact

Restricted.

---

## 165. Historical Preflight

Can be replayed using Part 65 if context snapshots retained.

---

## 166. Compare Forecast vs Actual

Useful for calibration.

---

## 167. ForecastAccuracyRecord

```rust
pub struct ForecastAccuracyRecord {
    pub preflight: PreflightId,
    pub actual_run: Option<RunId>,
    pub actual_deployment: Option<DeploymentId>,
}
```

---

## 168. Calibration

Improve cost/duration/blast-radius models.

---

## 169. Do Not Rewrite Historical Forecast

Critical.

---

## 170. Model Version

Record.

---

## 171. ForecastModelId

```rust
pub struct ForecastModelId(Digest);
```

---

## 172. Statistical Model

Optional.

---

## 173. AI/ML Model

Optional advisory.

---

## 174. Deterministic Gates

Never depend solely on opaque learned model.

---

## 175. Change Risk History

Useful for organization learning.

---

## 176. No Developer Ranking

Critical.

---

## 177. Risk Correlation

Can identify classes of change that frequently fail.

---

## 178. Not causal proof.

---

## 179. Incident Integration

Preflight can show:

```text
active incident on target
recent incident recurrence
change freeze
```

---

## 180. Incident History

Advisory context.

---

## 181. Active Change Freeze

Deterministic blocker.

---

## 182. Progressive Delivery

Risk profile can select minimum required rollout policy.

---

## 183. Example

Critical DB/schema change → at least manual staged rollout.

---

## 184. Security Patch

May permit faster rollout with explicit exception.

---

## 185. Approval Preview

Does not consume approval.

---

## 186. Approval Object Created Only in real workflow.

---

## 187. External Provider Read Credentials

Least privilege.

---

## 188. Preflight Worker

Dedicated read-only service identity where possible.

---

## 189. No Production Write Credentials

Critical.

---

## 190. Infrastructure Plan Adapter

If provider requires write-like permission to generate plan, document risk and isolate worker.

---

## 191. Network

Part 59 restricts preflight worker to read-only provider endpoints.

---

## 192. Secrets

Provider read credential is SecretRef.

---

## 193. Concurrency

Preflight normally needs no exclusive lock.

---

## 194. Current State Snapshot

May use consistent-read transaction.

---

## 195. No Locking Production Just To Simulate

Critical.

---

## 196. Schema Introspection

Read-only.

---

## 197. Large Query

Bounded.

---

## 198. Rate Limits

Prevent simulation DoS.

---

## 199. Quotas

Part 27.

---

## 200. Cost Of Preflight

Metered if expensive.

---

## 201. Preflight Priority

Lower than production control operations.

---

## 202. Cache

Reduces repeated work.

---

## 203. Testkit

```text
forgeyard-preflight-testkit/src/
├── lib.rs
├── subject.rs
├── impact.rs
├── risk.rs
├── policy.rs
├── scenario.rs
├── cache.rs
└── assertions.rs
```

---

## 204. Core Tests

### Identity
- same subject/context yields same plan identity;
- branch name resolves exact snapshot.

### Side Effects
- no deploy/apply/merge/publish during preflight;
- provider mutation attempt rejected.

### Risk
- destructive schema change yields explicit risk fact;
- unknown dependency widens impact.

### Policy
- preview does not grant authorization;
- actual policy change invalidates preview freshness.

### Compatibility
- breaking API change surfaced.

### Cost
- forecast records basis/model/pricing snapshot.

### Infrastructure
- delete/replace exposure surfaced.

### Scenario
- hypothetical config/policy change remains non-mutating.

### Security
- sensitive tenant/resource existence not leaked to unauthorized caller.

### Cache
- provider/environment state change invalidates mutable result.

---

## 205. Chaos Tests

Inject:

```text
provider read API outage
catalog unavailable
policy evaluator restart
cost service outage
compatibility analyzer failure
```

Expected:

```text
partial/incomplete result
no false green
no side effects
```

---

## 206. Scale Tests

Test:

```text
very large monorepo
thousands of impacted components
large tenant fleet
large infrastructure plans
many concurrent preflight requests
```

---

## 207. Implementation Phases

### Phase 1 — Preflight Subject/Result Model
Core identities.

### Phase 2 — Pipeline + Affected Work
Source change analysis.

### Phase 3 — Policy Preview
Approval/blocker explanation.

### Phase 4 — Compatibility/Security
Risk facts.

### Phase 5 — Cost/Resource Forecast
Planning.

### Phase 6 — Infrastructure/Deployment
Operational preview.

### Phase 7 — Migration Preview
DB risk.

### Phase 8 — Scenario Comparison
What-if workflows.

### Phase 9 — Progressive Delivery Integration
Rollout recommendations.

### Phase 10 — Federation/Air-Gap
Distributed planning.

### Phase 11 — Calibration
Forecast vs actual.

### Phase 12 — Security/Chaos/Scale Hardening
Production readiness.

---

## 208. Acceptance Tests

1. Preflight always binds an exact immutable subject.
2. Preflight context is versioned/snapshotted where needed.
3. Preflight performs no protected external side effect.
4. Provider planning is read-only or explicitly isolated.
5. Risk is expressed as typed facts.
6. Unknown risk/impact remains explicit.
7. No opaque score is sole policy authority.
8. Pipeline preflight uses canonical parser/planner.
9. Affected-work analysis uses Part 34 semantics.
10. Policy preview does not authorize execution.
11. Real execution re-evaluates current policy.
12. Compatibility preview uses exact contracts.
13. Infrastructure preflight never applies.
14. Migration preview never mutates schema/data.
15. Cost output is labeled forecast, not invoice.
16. Blast-radius unknowns do not become zero.
17. Approval preview never consumes approval.
18. Scenario analysis remains non-mutating.
19. Chosen scenario converts into normal canonical change.
20. Preflight freshness is explicit.
21. Stale preflight cannot become execution authority.
22. Missing providers/evidence produce Partial/Incomplete, not false Pass.
23. Sensitive impact information is authorization-filtered.
24. Preflight cannot be used as an authorization oracle.
25. AI can summarize but cannot replace deterministic risk facts.
26. Historical forecast remains immutable after actual outcome.
27. Forecast-vs-actual calibration is separate.
28. Federation/residency constraints appear in impact.
29. Air-gap preflight works with available local evidence.
30. Forgeyard dogfoods preflight on its own policy, infrastructure, migrations, deployments, runner images, and releases.

---

## 209. Production Readiness Gates

Do not call preflight architecture production-ready until:

```text
side-effect-free adapters are enforced
exact subject/context identity is stable
policy preview cannot issue authority
unknown/incomplete cannot appear green
sensitive impact redaction passes
cache freshness/invalidation is correct
infra/migration/deployment preview integration works
forecast provenance is recorded
federation/air-gap behavior is tested
chaos/scale/security tests pass
```

---

## 210. Architectural Invariants

1. preflight is read-only;
2. subject identity is exact;
3. mutable context is versioned;
4. risk facts have provenance;
5. uncertainty is explicit;
6. no opaque score is authority;
7. preview does not authorize;
8. real execution rechecks policy;
9. canonical planners are reused;
10. no alternate simulation parser exists;
11. infrastructure preview never applies;
12. migration preview never mutates;
13. cost is forecast;
14. blast radius unknown is not zero;
15. approval preview does not consume approval;
16. scenario comparison is non-mutating;
17. stale result is not authority;
18. missing evidence cannot become green;
19. sensitive impact is permission-filtered;
20. preflight cannot become authz oracle;
21. AI remains advisory;
22. deterministic risk facts drive gates;
23. cache keys include relevant context;
24. mutable provider state invalidates;
25. historical forecast remains unchanged;
26. calibration is separate;
27. federation/residency are explicit;
28. preflight workers use least-privilege read identities;
29. no production write credential is required baseline;
30. Forgeyard dogfoods its own preflight system.

---

## 211. Final Target Architecture

```text
                    Proposed Change
                         │
                         ▼
                    Exact Subject
                         │
                         ▼
                  Preflight Context
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      Impact Graph     Policy         Risk Facts
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                   Preflight Result
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Blockers    Approvals   Forecasts
                         │
                         ▼
                 Normal Change Workflow
```

What-if:

```text
current state
    +
hypothetical config/policy/strategy
    ↓
read-only scenario
    ↓
compare risk / impact / cost
    ↓
choose
    ↓
create normal canonical change
```

The key guarantee is:

> **Forgeyard can tell engineers what a change is likely to affect before they execute it, while preserving a strict boundary between prediction and authority. Preflight analysis explains risk, impact, approvals, compatibility, and cost without applying the change or pretending uncertain future behavior is already known.**

---

## 212. Extended Architecture Sequence

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
64 Remote Development Environments / Cloud Workspaces / Developer Workspace Orchestration
65 Build Graph Replay / Historical Reproducibility / Time-Travel CI / Evidence Reconstruction
66 Change Risk Assessment / Preflight Simulation / Policy Preview / What-If Analysis
```
