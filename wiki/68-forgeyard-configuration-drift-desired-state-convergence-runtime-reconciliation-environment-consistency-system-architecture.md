# 68 — Forgeyard Configuration Drift Detection, Desired-State Convergence, Runtime Reconciliation & Environment Consistency System Architecture

**Document type:** Core Configuration Drift Detection, Desired-State Convergence, Runtime Reconciliation, Environment Consistency, Exception & Remediation Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** desired-vs-observed state, configuration drift, deployment drift, infrastructure drift integration, runner baseline drift, network drift, secret-reference drift, release-channel drift, environment consistency, reconciliation loops, remediation policy, exception windows, stale observation handling, convergence safety, and drift evidence  
**Architecture style:** Explicit desired state, independently observed actual state, typed drift, evidence-backed reconciliation, safe convergence, ownership-aware remediation, stale-data rejection, bounded exceptions, auditability, and no blind auto-healing when the system cannot prove what changed or who owns it  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Events/Reconciliation, Configuration/Feature Flags, Infrastructure-as-Code/Drift, Deployment, Runner Image Factory, Network Connectivity, Secrets/Trust, Release Lifecycle, Multi-Region Federation, Audit, Incident Management, Preflight, and Reliability. This subsystem unifies drift semantics across Forgeyard’s control planes without collapsing their separate authorities.

---

## 1. Purpose

Forgeyard manages desired state across many different domains:

```text
runtime configuration
feature flags
deployed ReleaseIds
infrastructure resources
runner baselines
network policies
private resource bindings
secret references
release channel pointers
environment policies
```

Over time, observed reality can differ from intended state because of:

```text
manual administrator changes
provider-side mutation
partial deployment
failed reconciliation
stale runner host
configuration hotfix
network/firewall edit
external operator action
rollback
incident mitigation
restore/DR
out-of-band database change
```

A naive system often handles drift with one of two bad extremes:

```text
ignore everything until failure
or
automatically overwrite anything different
```

Both are dangerous.

The central rule is:

> **Drift exists only when Forgeyard can compare an explicit approved desired state with sufficiently fresh observed state for the same authoritative subject.**

A second rule is:

> **Auto-remediation is permitted only when ownership, authority, expected state, and remediation safety are known. Unknown drift is investigated, not blindly overwritten.**

A third rule is:

> **The drift subsystem never becomes a second authority for deployment, infrastructure, network, runner, secrets, or release state. It delegates remediation to the subsystem that owns the mutation.**

---

## 2. Architectural Position

```text
                   Approved Desired State
                           │
                           ▼
                    DesiredStateId
                           │
                           ▼
                    Observation Cycle
                           │
                           ▼
                    ObservedStateId
                           │
                           ▼
                     Drift Analysis
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
           In Sync       Known Drift   Unknown
                           │
                           ▼
                  Remediation Policy
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
          Observe       Auto-Converge   Manual
```

---

## 3. Goals

The subsystem MUST:

1. define desired-state identity;
2. define observed-state identity;
3. define drift identity;
4. support configuration drift;
5. support deployment drift;
6. support infrastructure drift integration;
7. support runner baseline drift;
8. support network drift;
9. support release-channel drift;
10. support environment policy drift;
11. support secret-reference drift;
12. support stale observation handling;
13. support drift confidence;
14. support ownership-aware classification;
15. support approved exceptions;
16. support safe auto-remediation;
17. support manual remediation;
18. support drift freeze/quarantine;
19. support reconciliation after outages;
20. support multi-region consistency;
21. support air-gapped/local operation;
22. support incident correlation;
23. support audit;
24. support Dioxus UI/API/CLI;
25. support historical drift evidence;
26. support policy gates;
27. support drift SLOs;
28. support HA;
29. preserve subsystem authority boundaries;
30. never overwrite unknown state blindly.

---

## 4. Non-Goals

This subsystem does not replace:

```text
Infrastructure-as-Code
Deployment
Configuration
Runner Image Factory
Network Policy
Secrets Management
Release Lifecycle
Database Migration
```

It observes and coordinates consistency across them.

---

## 5. Workspace Structure

```text
crates/drift/
├── forgeyard-drift/
├── forgeyard-drift-model/
├── forgeyard-drift-desired/
├── forgeyard-drift-observed/
├── forgeyard-drift-analysis/
├── forgeyard-drift-policy/
├── forgeyard-drift-remediation/
├── forgeyard-drift-exception/
├── forgeyard-drift-reconcile/
├── forgeyard-drift-health/
└── forgeyard-drift-testkit/
```

Adapters:

```text
crates/drift-adapters/
├── forgeyard-drift-config/
├── forgeyard-drift-deployment/
├── forgeyard-drift-infrastructure/
├── forgeyard-drift-runner/
├── forgeyard-drift-network/
├── forgeyard-drift-release/
└── forgeyard-drift-custom/
```

---

## 6. DesiredStateId

```rust
pub struct DesiredStateId(Digest);
```

Immutable identity of the normalized desired state for a subject.

---

## 7. ObservedStateId

```rust
pub struct ObservedStateId(Digest);
```

Immutable identity of one concrete observation.

---

## 8. DriftId

```rust
pub struct DriftId(Ulid);
```

One drift finding or drift lifecycle record.

---

## 9. DriftSubject

```rust
pub enum DriftSubject {
    Configuration(ConfigSubjectId),
    Environment(EnvironmentId),
    Deployment(DeploymentTargetId),
    Infrastructure(InfrastructureEnvironmentId),
    Runner(RunnerId),
    RunnerFleet(RunnerFleetId),
    Network(NetworkSubjectId),
    ReleaseChannel(ReleaseChannelId),
    SecretBinding(SecretBindingId),
    Custom(CustomDriftSubjectId),
}
```

---

## 10. Subject Identity

Must be typed and unambiguous.

Avoid:

```text
prod
stable
runner-1
```

without namespace/context.

---

## 11. Desired State Source

Desired state can originate from:

```text
approved config snapshot
deployment record
IaC specification
runner baseline binding
network capability/policy
release-channel pointer
environment policy
```

---

## 12. Desired State Is Approved State

Drafts do not become drift baseline.

---

## 13. DesiredStateSource

```rust
pub enum DesiredStateSource {
    Config(ConfigSnapshotId),
    Deployment(DeploymentId),
    Infrastructure(InfrastructureSpecId),
    RunnerBaseline(RunnerBaselineId),
    Network(NetworkCapabilityId),
    ReleaseChannel(ChannelPointer),
    Policy(PolicyDigest),
}
```

---

## 14. Observation

Observation is independent evidence from actual system/provider state.

---

## 15. ObservationSource

```rust
pub enum ObservationSource {
    Agent,
    ProviderApi,
    DatabaseIntrospection,
    Registry,
    LoadBalancer,
    NetworkConnector,
    RunnerAttestation,
    ControlPlane,
    Custom(ObservationSourceId),
}
```

---

## 16. Desired != Observed

Critical.

Do not infer actual state from last command.

---

## 17. Observation Timestamp

Mandatory.

---

## 18. Observation Freshness

```rust
pub enum ObservationFreshness {
    Fresh,
    Aging,
    Stale,
    Unknown,
}
```

---

## 19. Stale Observation

Cannot prove current drift state.

---

## 20. No Auto-Remediation From Stale Observation

Critical.

---

## 21. Drift Classification

```rust
pub enum DriftClass {
    None,
    Expected,
    Unauthorized,
    ManualApproved,
    PartialApply,
    ProviderMutation,
    VersionSkew,
    SecurityRelevant,
    Unknown,
}
```

---

## 22. Expected Drift

Temporary difference explicitly permitted by policy.

Example:

```text
progressive rollout wave
```

---

## 23. Unauthorized Drift

Observed state differs with no approved exception/change.

---

## 24. ManualApproved Drift

Known temporary operational override.

---

## 25. PartialApply

Desired change partially completed.

---

## 26. ProviderMutation

External system altered state.

---

## 27. VersionSkew

Different acceptable component generations during rolling transition.

---

## 28. SecurityRelevant

Difference affects trust/security floor.

---

## 29. Unknown

Cannot determine why/what changed.

---

## 30. Drift Severity

```rust
pub enum DriftSeverity {
    Informational,
    Low,
    Moderate,
    High,
    Critical,
}
```

---

## 31. Drift Confidence

```rust
pub enum DriftConfidence {
    Confirmed,
    Strong,
    Moderate,
    Weak,
    Unknown,
}
```

---

## 32. Confidence Is Separate From Severity

Critical.

---

## 33. Drift Finding

```rust
pub struct DriftFinding {
    pub id: DriftId,
    pub subject: DriftSubject,
    pub desired: DesiredStateId,
    pub observed: ObservedStateId,
    pub class: DriftClass,
    pub severity: DriftSeverity,
    pub confidence: DriftConfidence,
}
```

---

## 34. Drift Diff

Typed, not raw text where possible.

---

## 35. DriftChange

```rust
pub enum DriftChange {
    Added(StateField),
    Removed(StateField),
    Modified {
        field: StateField,
        desired: StateValue,
        observed: StateValue,
    },
}
```

---

## 36. Secret Values

Never included in drift diff.

---

## 37. Secret Drift

Compare:

```text
SecretRef identity
provider version metadata
binding state
rotation generation
```

not plaintext secret.

---

## 38. Config Drift

Part 39.

Example:

```text
desired feature flag false
observed provider flag true
```

---

## 39. Runtime Config

Can have intentional dynamic state.

Need ownership metadata.

---

## 40. Config Ownership

```rust
pub enum ConfigOwnership {
    ForgeyardManaged,
    ExternalManaged,
    Shared,
}
```

---

## 41. External Managed

Forgeyard observes but does not auto-converge.

---

## 42. Shared Ownership

Requires field-level ownership.

---

## 43. FieldOwnership

```rust
pub struct FieldOwnership {
    pub field: StateFieldPath,
    pub authority: AuthorityRef,
}
```

---

## 44. No Whole-Object Overwrite On Shared Ownership

Critical.

---

## 45. Deployment Drift

Desired:

```text
ReleaseId X
```

Observed:

```text
ReleaseId Y
traffic split
replica set
runtime version
```

---

## 46. Deployment Drift Examples

```text
manual rollback
partial rollout
provider auto-update
failed replica convergence
```

---

## 47. Progressive Delivery

Part 62 can make mixed versions expected.

---

## 48. Expected Deployment Skew

Encoded as allowed wave/ring state.

---

## 49. Not Drift During Valid Rollout

Critical.

---

## 50. Infrastructure Drift

Part 53 remains authority.

This subsystem consumes infrastructure drift findings.

---

## 51. No Duplicate IaC Diff Engine

Critical.

---

## 52. Runner Drift

Part 58.

Consume:

```text
baseline mismatch
package drift
kernel/driver drift
taint
attestation expiry
```

---

## 53. Network Drift

Part 59.

Examples:

```text
unexpected public route
missing private connector
wrong DNS policy
extra firewall rule
```

---

## 54. Release Channel Drift

Desired pointer:

```text
stable -> ReleaseId A
```

Observed external tag/update feed:

```text
stable -> ReleaseId B
```

---

## 55. External Registry Tag Drift

Important.

---

## 56. Canonical Channel Authority

Part 67 Forgeyard metadata.

External tags are projections.

---

## 57. Drift Remediation

Can republish canonical pointer through Part 67/52.

---

## 58. Environment Consistency

Environment can be modeled as aggregate desired state.

---

## 59. EnvironmentDesiredStateId

```rust
pub struct EnvironmentDesiredStateId(Digest);
```

---

## 60. Environment Composition

```text
ReleaseIds
ConfigSnapshotId
InfrastructureSpecId
NetworkPolicy
Secret bindings
Feature flags
Migration generation
```

---

## 61. Environment Consistency Snapshot

```rust
pub struct EnvironmentConsistencySnapshot {
    pub environment: EnvironmentId,
    pub desired: EnvironmentDesiredStateId,
    pub findings: Vec<DriftId>,
    pub observed_at: Timestamp,
}
```

---

## 62. Environment Health

Drift contributes but does not equal health.

---

## 63. Example

Minor benign config drift may not affect availability.

---

## 64. Consistency State

```rust
pub enum ConsistencyState {
    Converged,
    Converging,
    Drifted,
    Unknown,
    Quarantined,
}
```

---

## 65. Converging

Desired change actively being applied.

---

## 66. Drifted

No approved in-progress transition explains difference.

---

## 67. Unknown

Observation insufficient.

---

## 68. Quarantined

Unsafe to auto-remediate.

---

## 69. Drift Policy

```rust
pub struct DriftPolicyId(Digest);
```

---

## 70. Drift Policy

```rust
pub struct DriftPolicy {
    pub subject_kind: DriftSubjectKind,
    pub detection: DetectionPolicy,
    pub remediation: RemediationPolicy,
}
```

---

## 71. Detection Policy

Controls:

```text
frequency
freshness threshold
severity mapping
ownership rules
```

---

## 72. Remediation Policy

```rust
pub enum RemediationPolicy {
    ObserveOnly,
    Notify,
    AutoConvergeSafe,
    RequireApproval,
    Quarantine,
}
```

---

## 73. AutoConvergeSafe

Only for changes with deterministic safe remediation.

---

## 74. Example Safe

Re-publish missing read-only config projection.

---

## 75. Example Unsafe

Overwrite unknown database schema/manual production change.

---

## 76. No Generic Auto-Heal

Critical.

---

## 77. Remediation Plan

```rust
pub struct DriftRemediationPlanId(Digest);
```

---

## 78. Remediation Plan

Contains:

```text
subject
current observation
desired state
owning subsystem
action
expected version
risk
```

---

## 79. Remediation Delegation

Examples:

```text
deployment drift -> Deployment
infra drift -> IaC
runner drift -> Runner Image/Fleet
network drift -> Network
channel drift -> Release Lifecycle
config drift -> Config
```

---

## 80. Drift Subsystem Does Not Mutate Directly

Critical.

---

## 81. Preflight

Part 66 can preview remediation impact.

---

## 82. High-Risk Remediation

Requires preflight/approval.

---

## 83. DriftExceptionId

```rust
pub struct DriftExceptionId(Ulid);
```

---

## 84. Drift Exception

```rust
pub struct DriftException {
    pub id: DriftExceptionId,
    pub subject: DriftSubject,
    pub allowed_diff: DriftPredicate,
    pub expires_at: Timestamp,
    pub reason: BoundedString,
}
```

---

## 85. Exception Is Scoped

No blanket “ignore drift.”

---

## 86. Exception Expiry Mandatory

Baseline.

---

## 87. Long-Lived Exception

Needs periodic review.

---

## 88. Incident Exception

Can be linked to IncidentId.

---

## 89. Incident Mitigation

May intentionally create temporary drift.

---

## 90. Example

Manual traffic shift during outage.

---

## 91. Incident Ends

Drift exception does not auto-delete without policy.

---

## 92. Reconcile Before Returning To Normal

Critical.

---

## 93. Drift Lifecycle

```rust
pub enum DriftFindingState {
    Open,
    Acknowledged,
    ExceptionActive,
    Remediating,
    Resolved,
    Superseded,
    Quarantined,
}
```

---

## 94. Resolved

Observed state now matches accepted desired state or approved new desired state.

---

## 95. Superseded

Desired state changed, replacing old finding.

---

## 96. No Delete Old Finding

Historical record retained.

---

## 97. Reconciliation Loop

```text
load desired state
  ↓
observe actual
  ↓
compare
  ↓
classify
  ↓
policy
  ↓
delegate remediation if safe
  ↓
observe again
```

---

## 98. Eventual Convergence

Not instant guarantee.

---

## 99. Reconcile Interval

Per subject type.

---

## 100. Event-Driven Reconcile

Preferred where provider events exist.

---

## 101. Periodic Sweep

Safety net.

---

## 102. No Busy Polling

Critical.

---

## 103. Desired State Generation

```rust
pub struct DesiredStateGeneration(u64);
```

---

## 104. Remediation Uses Expected Generation

---

## 105. Stale Reconciler

Fenced by generation/version.

---

## 106. Concurrency

Part 60.

Protected remediation acquires relevant scope lease if required.

---

## 107. Drift Observation Race

Desired state may change during observation.

---

## 108. Solution

Compare with exact desired generation.

---

## 109. If Desired Changed

Discard/recompute.

---

## 110. No Remediation Against Old Desired

Critical.

---

## 111. Observation Race

Observed state may change after read.

---

## 112. Expected Provider Version

Use if possible.

---

## 113. Provider Without Versioning

Re-observe after action.

---

## 114. Unknown Outcome

Reconcile.

---

## 115. Drift Detection Frequency

Risk-based.

Examples:

```text
security/network → frequent
release channel → frequent
runner fleet → heartbeat-driven
low-risk config → moderate
```

---

## 116. Highest Supported Frequency

Implementation-dependent, not policy guarantee.

---

## 117. Observation Cost

Part 45.

---

## 118. Expensive Provider APIs

Use event + periodic hybrid.

---

## 119. Rate Limits

Respect provider.

---

## 120. Drift Detection SLO

Possible:

```text
critical drift detected within X minutes
```

---

## 121. SLO

Part 50.

---

## 122. Drift Remediation SLO

Separate.

---

## 123. Detection vs Repair

Critical distinction.

---

## 124. Stale Desired State

If control-plane data may be outdated after partition/restore, freeze auto-remediation.

---

## 125. Federation

Part 51.

Only accepted authority site's desired state can drive mutation.

---

## 126. Regional Observation

Can occur locally.

---

## 127. Global Desired State

Authority-controlled.

---

## 128. Partitioned Site

May report drift but cannot mutate global-authority state unless delegated.

---

## 129. No Split-Brain Auto-Heal

Critical.

---

## 130. AuthorityEpoch

Included in remediation.

---

## 131. Site Failover

Old authority observations remain evidence only.

---

## 132. Air-Gap

Local drift detection works with local desired state.

---

## 133. Disconnected Operation

Exceptions/delegated authority explicit.

---

## 134. Reconnect

Compare local observed state against current accepted desired state.

---

## 135. No Last-Write-Wins Reconciliation

Critical.

---

## 136. DR

After restore:

```text
load restored desired state
  ↓
invalidate stale leases
  ↓
observe actual external state
  ↓
classify divergence
  ↓
manual/safe reconcile
```

---

## 137. Never Blindly Push Restored Snapshot To Production

Critical.

---

## 138. Restore Drift

First-class category.

---

## 139. RestoreReconciliationState

```rust
pub enum RestoreReconciliationState {
    PendingObservation,
    Comparing,
    SafeToConverge,
    ManualReview,
    Completed,
}
```

---

## 140. Database Schema

Part 63.

DB drift consumed from migration subsystem.

---

## 141. No Automatic Schema Repair Baseline

Critical.

---

## 142. Secrets Drift

Examples:

```text
SecretRef points to deleted provider object
rotation generation stale
binding permission changed
```

---

## 143. Secret Value Comparison

Never.

---

## 144. Secret Rotation

Part 12 authority.

---

## 145. Policy Drift

Possible:

```text
external enforcement policy differs from Forgeyard policy projection
```

---

## 146. Canonical Policy

Part 11/39.

---

## 147. Runtime Feature Flag Drift

If external provider backs flags, observe pointer/value state.

---

## 148. Flag Drift

Can be auto-remediated if Forgeyard has sole ownership and safe semantics.

---

## 149. Shared External Flag

Observe-only unless field ownership proves authority.

---

## 150. Runner Fleet Drift

Aggregate:

```text
percentage on wrong baseline
stale attestation
tainted runners
```

---

## 151. FleetConsistency

```rust
pub struct FleetConsistency {
    pub desired_baseline: RunnerBaselineId,
    pub compliant: u64,
    pub noncompliant: u64,
    pub unknown: u64,
}
```

---

## 152. Unknown Runner

Not counted compliant.

---

## 153. Channel Drift

External registry/update projection may differ.

---

## 154. Channel Projection

Can be re-published safely if:

```text
canonical pointer current
external provider version known
security state valid
```

---

## 155. No Re-Publish Revoked Release

Critical.

---

## 156. Network Drift

Public exposure drift can be critical.

---

## 157. Immediate Response

Can:

```text
quarantine
notify
delegate network remediation
```

---

## 158. Security Drift

May trigger IncidentId.

---

## 159. DriftToIncidentPolicy

```rust
pub struct DriftToIncidentPolicy {
    pub severity_threshold: DriftSeverity,
    pub classes: Vec<DriftClass>,
}
```

---

## 160. Incident Creation

Deterministic policy/manual.

---

## 161. AI

Can summarize drift clusters.

---

## 162. AI Does Not Auto-Repair High-Risk Drift

Critical.

---

## 163. Drift Correlation

Many findings may share cause.

Example:

```text
provider account policy changed
  ↓
network + deployment + registry drift
```

---

## 164. Correlation

Advisory.

---

## 165. No Auto-Merge Canonical Findings

---

## 166. Historical Drift

Useful for:

```text
audit
incident investigation
recurrence
reliability
```

---

## 167. DriftSnapshotId

```rust
pub struct DriftSnapshotId(Digest);
```

---

## 168. Snapshot

Point-in-time set of findings for environment/fleet.

---

## 169. Environment Baseline

Can be exported.

---

## 170. Compliance

Part 28.

Drift against security/compliance baseline can become compliance evidence.

---

## 171. Drift != Compliance Violation Automatically

Policy decides.

---

## 172. Data Lifecycle

Part 46.

Retain:

```text
drift finding
observation metadata
remediation decision
exception
resolution evidence
```

---

## 173. Raw Provider State

May be large/sensitive.

Store normalized relevant subset.

---

## 174. No Secret Payload

Critical.

---

## 175. Audit

Audit events:

```text
drift exception creation
manual acknowledge
auto-remediation enablement
high-risk remediation approval
desired-state adoption
quarantine release
exception expiry override
```

---

## 176. Routine Detection

Operational event.

---

## 177. Desired-State Adoption

Sometimes observed state is correct and desired state should change.

---

## 178. Adoption

```text
observe drift
  ↓
review
  ↓
create normal canonical change
  ↓
approve
  ↓
new DesiredStateId
```

---

## 179. Do Not "Accept Drift" By Mutating Baseline Silently

Critical.

---

## 180. Adoption Creates New Change History

---

## 181. Dioxus UI

Pages:

```text
Drift
Environment Consistency
Drift Exceptions
Remediation
Drift History
```

---

## 182. Environment View

Shows:

```text
desired generation
observed freshness
consistency state
open drift
critical drift
active exceptions
```

---

## 183. Drift Detail

Shows:

```text
typed diff
desired source
observation source
ownership
severity/confidence
recommended action
```

---

## 184. Remediation Preview

Can invoke Part 66.

---

## 185. CLI

```text
forgeyard drift list
forgeyard drift show
forgeyard drift scan
forgeyard drift explain
forgeyard drift remediate
forgeyard drift exception create
forgeyard drift exception revoke
forgeyard drift adopt
forgeyard drift doctor
```

---

## 186. API

Potential:

```text
GET  /v1/drift
GET  /v1/drift/{id}
POST /v1/drift/scan
POST /v1/drift/{id}/remediate
POST /v1/drift/{id}/exceptions
POST /v1/drift/{id}/adopt
```

---

## 187. Permissions

```text
drift.read
drift.scan
drift.remediate
drift.exception.manage
drift.adopt
drift.quarantine
```

---

## 188. Adopt

High privilege.

---

## 189. Quarantine

High privilege/security.

---

## 190. Observability Metrics

```text
drift_findings_open
drift_findings_total
drift_critical_total
drift_unknown_total
drift_auto_remediation_total
drift_remediation_failures_total
drift_observation_stale_total
```

---

## 191. Labels

Low-cardinality:

```text
subject_kind
drift_class
severity
state
```

---

## 192. Tracing

```text
drift.observe
drift.compare
drift.classify
drift.remediate
drift.exception
drift.reconcile
```

---

## 193. Health

```rust
pub enum DriftSubsystemHealth {
    Healthy,
    ObservationDegraded,
    RemediationDegraded,
    HighUnknownDrift,
    Unhealthy,
}
```

---

## 194. Doctor

```text
forgeyard drift doctor
```

Checks:

```text
stale observations
expired exceptions
unknown critical drift
remediation loops
desired-state generation mismatch
authority mismatch
external projection divergence
```

---

## 195. Remediation Loop Detection

Important.

---

## 196. Example

```text
Forgeyard sets X
external operator sets Y
Forgeyard sets X
external operator sets Y
```

---

## 197. Flapping Drift

```rust
pub enum DriftStability {
    Stable,
    Flapping,
    Unknown,
}
```

---

## 198. Flapping

Auto-remediation pauses and escalates.

---

## 199. No Infinite Tug-of-War

Critical.

---

## 200. Ownership Conflict

Likely root cause.

---

## 201. Adoption/Ownership Review

Required.

---

## 202. External Controllers

Kubernetes/operator/cloud policy may also reconcile.

---

## 203. Controller Conflict Detection

Track observed actor/provider metadata where available.

---

## 204. No Fighting Another Controller Blindly

Critical.

---

## 205. Kubernetes

If Argo/operator owns field, Forgeyard observes or integrates rather than overwrite.

---

## 206. Field Manager

Can use provider field-ownership semantics.

---

## 207. Cloud Resources

Tags/metadata may identify Forgeyard-managed fields.

---

## 208. Manual Change Policy

Organizations can choose:

```text
forbid
allow with exception
observe-only
auto-revert safe fields
```

---

## 209. Manual Emergency Change

Link IncidentId.

---

## 210. After Emergency

Reconcile/adopt explicitly.

---

## 211. Preflight

Remediation can be simulated.

---

## 212. Progressive Delivery

Mixed state during active rollout is expected, not drift.

---

## 213. Release Lifecycle

Channel movement planned transition is expected.

---

## 214. Migration

Schema transitional state expected according migration phase.

---

## 215. ExpectedTransitionId

```rust
pub struct ExpectedTransitionId(Ulid);
```

---

## 216. Transition Window

Drift analyzer uses active transition state.

---

## 217. No Static Desired-State Comparison During Active Transition

Critical.

---

## 218. Transition-Aware Desired State

Can specify acceptable intermediate states.

---

## 219. AcceptableObservedSet

```rust
pub struct AcceptableObservedSet {
    pub states: Vec<DesiredStateId>,
    pub expires_at: Timestamp,
}
```

---

## 220. Expiry

After rollout/migration window.

---

## 221. Transition Stuck

Becomes drift/reliability issue.

---

## 222. Environment Consistency SLO

Possible:

```text
99.9% of production environments converge within 10 minutes of approved change
```

---

## 223. SLO Terms

Must be precise per subject class.

---

## 224. Drift Budget

Optional concept.

---

## 225. Drift Budget

Counts tolerated non-critical exceptions.

---

## 226. Not Substitute For Security Floor

Critical.

---

## 227. Cost

Observation/remediation cost tracked.

---

## 228. Auto-Remediation Cost

Can matter at fleet scale.

---

## 229. Cost Does Not Override Critical Security Drift

---

## 230. Federation Consistency

Global desired state + regional observed state.

---

## 231. SiteConsistencySummary

```rust
pub struct SiteConsistencySummary {
    pub site: SiteId,
    pub state: ConsistencyState,
    pub critical: u64,
    pub unknown: u64,
}
```

---

## 232. Global UI

Shows regional consistency.

---

## 233. Air-Gap Export

Drift bundle can export:

```text
desired snapshot
observed snapshot
findings
exceptions
```

---

## 234. Signed/verified where high assurance.

---

## 235. Reconciliation On Reconnect

Uses current authority epoch.

---

## 236. No Replay Of Old Remediation Commands

Critical.

---

## 237. Testkit

```text
forgeyard-drift-testkit/src/
├── lib.rs
├── desired.rs
├── observed.rs
├── classify.rs
├── remediation.rs
├── exception.rs
├── transition.rs
├── federation.rs
└── assertions.rs
```

---

## 238. Core Tests

### Desired/Observed
- desired and observed identities deterministic;
- last command is not accepted as observation.

### Freshness
- stale observation blocks auto-remediation;
- unknown observation remains Unknown.

### Ownership
- external-managed field not overwritten;
- shared field ownership respected.

### Transition
- progressive rollout intermediate state not false drift;
- stuck transition becomes finding after expiry.

### Remediation
- safe field auto-converges through owner subsystem;
- unknown state quarantines/manual review;
- remediation uses current desired generation.

### Flapping
- repeated controller conflict pauses auto-remediation.

### Security
- public network exposure classified high/critical;
- secret values never appear in diff.

### DR/Federation
- restored desired state does not blindly overwrite external reality;
- stale authority cannot remediate after failover.

---

## 239. Chaos Tests

Inject:

```text
provider observation timeout
network partition
controller crash during remediation
external operator changing same field
DB restore
regional authority failover
```

Expected:

```text
Unknown/stale state visible
no blind overwrite
stale controller fenced
reconciliation resumes safely
```

---

## 240. Scale Tests

Test:

```text
millions of observed config fields
large runner fleets
many environments
multi-region consistency sweeps
high-frequency network drift
```

---

## 241. Implementation Phases

### Phase 1 — Drift Model
Desired/observed/findings.

### Phase 2 — Config/Deployment Drift
Core product use cases.

### Phase 3 — Runner/Network/Channel Drift
Security-sensitive projections.

### Phase 4 — Exception Governance
Controlled temporary divergence.

### Phase 5 — Safe Auto-Remediation
Delegated convergence.

### Phase 6 — Environment Aggregate Consistency
Unified view.

### Phase 7 — Transition-Aware Drift
Deploy/migration/rollout integration.

### Phase 8 — Federation/DR
Authority-aware reconciliation.

### Phase 9 — Drift SLO/Analytics
Reliability.

### Phase 10 — UI/CLI/Doctor
Operability.

### Phase 11 — Flapping/Controller Conflict Detection
Advanced safety.

### Phase 12 — Chaos/Scale/Security Hardening
Production readiness.

---

## 242. Acceptance Tests

1. Drift compares explicit desired state with independent observed state.
2. Last command is never treated as observation.
3. Observation freshness is explicit.
4. Stale observation cannot drive automatic remediation.
5. Drift severity and confidence are separate.
6. Secret values never appear in drift diff.
7. Field ownership is respected.
8. External-managed fields are not overwritten.
9. Shared ownership does not trigger whole-object replacement.
10. Progressive rollout intermediate states are not false drift.
11. Migration transitional states are understood.
12. Expired/stuck transition becomes drift.
13. Safe auto-remediation delegates to owning subsystem.
14. Drift subsystem never directly becomes second mutation authority.
15. Unknown drift is not blindly auto-healed.
16. Desired-state generation is checked before remediation.
17. Stale controller is fenced.
18. Provider timeout becomes Unknown/reconcile.
19. Drift exceptions are scoped and expiring.
20. Emergency incident drift is explicitly reconciled/adopted afterward.
21. “Accept drift” creates normal desired-state change history.
22. Flapping/controller conflict pauses automatic remediation.
23. Security-critical drift can trigger incident/quarantine.
24. Restore does not blindly push old desired state.
25. Federation authority controls remediation rights.
26. Air-gap/local drift detection works independently.
27. Environment consistency aggregates without replacing subsystem health.
28. Drift history is retained according lifecycle policy.
29. Dioxus/API/CLI explain why state is considered drift.
30. Forgeyard dogfoods drift/convergence for its own config, runner fleets, network, deployment, and release-channel projections.

---

## 243. Production Readiness Gates

Do not call drift/convergence production-ready until:

```text
independent observation is proven
freshness rules are enforced
field ownership is implemented
transition-aware drift works
safe remediation delegation is tested
flapping detection exists
exceptions expire correctly
DR/federation authority tests pass
security-sensitive drift paths are incident-integrated
chaos/scale tests pass
```

---

## 244. Architectural Invariants

1. desired state and observed state are separate;
2. desired state is approved state;
3. observation is independently sourced;
4. stale observation is not authoritative;
5. Unknown is first-class;
6. severity and confidence are distinct;
7. secrets are never diffed by plaintext value;
8. ownership is field/subject aware;
9. drift subsystem does not become second mutation authority;
10. safe auto-remediation is explicitly policy-bound;
11. unknown drift is never blindly overwritten;
12. desired generation is rechecked before remediation;
13. stale controllers are fenced;
14. exceptions are scoped/expiring;
15. active transitions define acceptable intermediate states;
16. stuck transitions become drift;
17. flapping pauses auto-remediation;
18. controller conflicts are surfaced;
19. restore requires observation before convergence;
20. federation authority gates remediation;
21. no LWW reconciliation;
22. manual emergency drift is explicit;
23. adoption creates canonical change history;
24. security drift can quarantine/escalate;
25. environment consistency is aggregate evidence, not new authority;
26. drift retention/lifecycle is explicit;
27. cost cannot suppress critical remediation;
28. air-gap/local operation remains valid;
29. explanation/evidence is always available;
30. Forgeyard dogfoods its own drift system.

---

## 245. Final Target Architecture

```text
                  Approved Desired State
                           │
                           ▼
                     DesiredStateId
                           │
                           ▼
                   Independent Observe
                           │
                           ▼
                    ObservedStateId
                           │
                           ▼
                      Drift Analysis
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
         Converged       Expected        Drift
                                           │
                                           ▼
                                 Ownership + Safety
                                           │
                           ┌───────────────┼───────────────┐
                           ▼               ▼               ▼
                       Observe         Auto-Converge      Manual
                                           │
                                           ▼
                               Owning Subsystem Action
                                           │
                                           ▼
                                      Re-Observe
```

Restore/failover:

```text
restored desired state
       ↓
observe external reality
       ↓
compare
       ↓
safe converge / manual review
```

The key guarantee is:

> **Forgeyard can continuously detect and correct configuration/environment drift without becoming an unsafe “overwrite reality” controller. Every convergence decision depends on fresh observation, known ownership, exact desired generation, current authority, and remediation safety—and the actual mutation remains owned by the subsystem responsible for that state.**

---

## 246. Extended Architecture Sequence

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
67 Artifact Promotion Policy / Release Train / Environment Channel / Lifecycle Governance
68 Configuration Drift Detection / Desired-State Convergence / Runtime Reconciliation / Environment Consistency
```
