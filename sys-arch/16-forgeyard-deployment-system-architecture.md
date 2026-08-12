# 16 — Forgeyard Deployment System Architecture

**Document type:** Core Deployment Orchestration System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Environments, deployment targets, desired/actual state, rollout plans, canary/blue-green/rolling strategies, health gates, migrations, rollback, drift detection, provider adapters, deployment credentials, approvals, reconciliation, and deployment observability  
**Architecture style:** Desired-state deployment controller consuming immutable released artifacts, with policy-gated plans, exact-digest rollout, provider-neutral adapters, explicit health transitions, idempotent external effects, and reconciliation-driven convergence  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on Release, Packaging, Supply Chain, Secrets & Trust, Policy/Authz/Identity, Events/Reconciliation, Run/Job, Scheduler, Runner, and CAS. It consumes already-released artifacts and release metadata. It does not rebuild, repackage, or resign application artifacts.

---

# 1. Purpose

Forgeyard needs a deployment subsystem that answers:

```text
what exact release should be running?
where should it run?
what is currently running?
how should rollout proceed?
what health conditions must pass?
what happens if rollout fails?
what database migration is required?
can we roll back safely?
did the provider actually apply the requested change?
has the environment drifted?
who approved production deployment?
```

A deployment is not:

```text
SSH into a server and run commands
kubectl apply whatever is in a workspace
build the image again
copy an unverified binary
```

The central rule is:

> **Deployment consumes immutable release artifacts and converges real infrastructure toward an explicitly declared desired state.**

A second rule is:

> **Desired deployment state and observed provider state are separate. Forgeyard reconciles the two until they converge or an operator is required.**

A third rule is:

> **Deployment never rebuilds, repackages, or resigns the release. It deploys exact approved release identities and digests.**

---

# 2. Architectural Position

```text
                 Released Artifact
                       │
                       ▼
                  ReleaseId
                       │
                       ▼
                   Environment
                       │
                       ▼
                 DeploymentPlan
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
      Policy        Approval       Health Gate
        │              │              │
        └──────────────┼──────────────┘
                       ▼
                 Desired State
                       │
                       ▼
                 Provider Adapter
                       │
                       ▼
                  Actual State
                       │
                       ▼
                  Reconciler
                       │
              ┌────────┼─────────┐
              ▼        ▼         ▼
           Healthy   Rollback   Operator
```

---

# 3. Goals

The deployment subsystem MUST:

1. define stable `EnvironmentId`;
2. define stable `DeploymentId`;
3. define stable `DeploymentPlanId`;
4. consume immutable `ReleaseId`;
5. consume exact package/artifact digests;
6. define desired deployment state;
7. observe actual provider state;
8. support deployment strategies;
9. support health gates;
10. support manual approval;
11. support automated promotion gates;
12. support production protection;
13. support rollback;
14. support migration orchestration;
15. support drift detection;
16. support Kubernetes;
17. support VMs;
18. support bare-metal/SSH-style managed targets;
19. support container platforms;
20. support serverless adapters;
21. support mobile/device deployment where relevant;
22. support environment-scoped secrets;
23. support short-lived workload credentials;
24. support idempotent provider operations;
25. support Unknown external outcomes;
26. support reconciliation;
27. support pause/resume;
28. support cancellation;
29. support audit;
30. never rebuild release artifacts.

---

# 4. Non-Goals

Deployment does not:

```text
compile source
package source
sign release artifacts
own package registry logic
replace Kubernetes
replace Terraform/Pulumi
replace cloud control planes
```

It orchestrates deployment of known release state.

---

# 5. Workspace Structure

```text
crates/deploy/
├── forgeyard-deploy/
├── forgeyard-deploy-model/
├── forgeyard-deploy-plan/
├── forgeyard-deploy-environment/
├── forgeyard-deploy-target/
├── forgeyard-deploy-state/
├── forgeyard-deploy-strategy/
├── forgeyard-deploy-health/
├── forgeyard-deploy-approval/
├── forgeyard-deploy-migration/
├── forgeyard-deploy-rollback/
├── forgeyard-deploy-drift/
├── forgeyard-deploy-provider/
├── forgeyard-deploy-kubernetes/
├── forgeyard-deploy-container/
├── forgeyard-deploy-vm/
├── forgeyard-deploy-baremetal/
├── forgeyard-deploy-serverless/
├── forgeyard-deploy-device/
├── forgeyard-deploy-reconcile/
├── forgeyard-deploy-healthcheck/
├── forgeyard-deploy-metrics/
└── forgeyard-deploy-testkit/
```

---

# 6. Environment

```rust
pub struct Environment {
    pub id: EnvironmentId,
    pub name: EnvironmentName,
    pub class: EnvironmentClass,
    pub policy_scope: AuthorizationScope,
}
```

---

# 7. EnvironmentId

```rust
pub struct EnvironmentId(Ulid);
```

Stable identity.

---

# 8. Environment Classes

```rust
pub enum EnvironmentClass {
    Development,
    Testing,
    Staging,
    Production,
    DisasterRecovery,
    Custom(EnvironmentClassId),
}
```

---

# 9. Environment Is Not Release Channel

Example:

```text
Stable release channel
```

can deploy to:

```text
staging
production
```

These are distinct concepts.

---

# 10. Deployment Target

```rust
pub struct DeploymentTarget {
    pub id: DeploymentTargetId,
    pub environment: EnvironmentId,
    pub provider: DeploymentProviderId,
    pub kind: DeploymentTargetKind,
}
```

---

# 11. Target Kind

```rust
pub enum DeploymentTargetKind {
    Kubernetes,
    ContainerService,
    VirtualMachine,
    BareMetal,
    Serverless,
    DeviceFleet,
    StaticHosting,
    Custom(DeploymentTargetKindId),
}
```

---

# 12. Target Scope

A deployment target represents a managed destination, not a hostname string.

---

# 13. Examples

```text
prod-k8s-eu
staging-vm-pool
android-test-fleet
static-doc-site
edge-cluster
```

---

# 14. DeploymentId

```rust
pub struct DeploymentId(Ulid);
```

Represents one concrete deployment execution/intent lifecycle.

---

# 15. DeploymentPlanId

```rust
pub struct DeploymentPlanId(Digest);
```

Canonical digest of immutable deployment intent.

---

# 16. Deployment Plan

```rust
pub struct DeploymentPlan {
    pub id: DeploymentPlanId,
    pub environment: EnvironmentId,
    pub target: DeploymentTargetId,
    pub release: ReleaseId,
    pub artifacts: Vec<DeploymentArtifactRef>,
    pub strategy: DeploymentStrategy,
    pub health: HealthPolicy,
    pub migration: Option<MigrationPlan>,
    pub rollback: RollbackPolicy,
    pub policy: PolicyDigest,
}
```

---

# 17. Deployment Artifact

```rust
pub struct DeploymentArtifactRef {
    pub release: ReleaseId,
    pub package: PackageId,
    pub object: CasObjectRef,
}
```

---

# 18. Exact Artifact Rule

Deployment never uses:

```text
latest
stable tag only
branch name
mutable image tag only
```

as authority.

---

# 19. Mutable Labels

May resolve to exact digest before plan creation.

---

# 20. Plan Freeze

Before deployment:

```text
resolve release
resolve target
resolve exact artifacts
resolve strategy
resolve migration
resolve policy
  ↓
canonical DeploymentPlan
  ↓
DeploymentPlanId
```

---

# 21. Plan Immutability

Changing:

```text
release
target
strategy
migration
health threshold
```

creates a new plan ID.

---

# 22. Deployment State

```rust
pub enum DeploymentState {
    Planned,
    AwaitingApproval,
    Ready,
    Applying,
    Verifying,
    Paused,
    Healthy,
    Degraded,
    Failed,
    RollingBack,
    RolledBack,
    Cancelled,
    Unknown,
}
```

---

# 23. Planned

Plan created but not yet authorized to apply.

---

# 24. AwaitingApproval

Protected environment requires human/policy approval.

---

# 25. Ready

All gates passed.

---

# 26. Applying

Provider changes in progress.

---

# 27. Verifying

Provider reports target applied; health evaluation ongoing.

---

# 28. Paused

Rollout intentionally paused.

---

# 29. Healthy

Desired state applied and health gates passed.

---

# 30. Degraded

Desired state mostly exists but health or replica conditions are degraded.

---

# 31. Failed

Deployment cannot meet required policy/health and no automatic rollback currently active.

---

# 32. RollingBack

Converging to prior known good desired state.

---

# 33. RolledBack

Rollback successfully converged.

---

# 34. Cancelled

Deployment stopped before successful completion.

---

# 35. Unknown

External provider outcome ambiguous.

---

# 36. Desired State

```rust
pub struct DesiredDeploymentState {
    pub plan: DeploymentPlanId,
    pub release: ReleaseId,
    pub target: DeploymentTargetId,
    pub generation: DeploymentGeneration,
}
```

---

# 37. Deployment Generation

```rust
pub struct DeploymentGeneration(u64);
```

Monotonically increases per target/environment application.

---

# 38. Actual State

```rust
pub struct ObservedDeploymentState {
    pub target: DeploymentTargetId,
    pub observed_release: Option<ReleaseId>,
    pub observed_artifacts: Vec<ObservedArtifact>,
    pub generation: Option<DeploymentGeneration>,
    pub health: ObservedHealth,
    pub observed_at: Timestamp,
}
```

---

# 39. Desired vs Actual

Core deployment controller pattern:

```text
Desired
  ↓ compare
Actual
  ↓
Action / No-op / Rollback / Operator
```

---

# 40. Reconciliation

Deployment correctness relies on reconciliation, not single API call success.

---

# 41. Provider Adapter

```rust
#[async_trait]
pub trait DeploymentProvider {
    async fn inspect(
        &self,
        target: DeploymentTargetId,
    ) -> Result<ObservedDeploymentState, DeployProviderError>;

    async fn apply(
        &self,
        request: ApplyDeploymentRequest,
    ) -> Result<ApplyDeploymentResult, DeployProviderError>;

    async fn rollback(
        &self,
        request: RollbackDeploymentRequest,
    ) -> Result<RollbackDeploymentResult, DeployProviderError>;
}
```

---

# 42. Provider Neutrality

Core does not depend on:

```text
Kubernetes API types
AWS SDK
Azure SDK
SSH library
```

---

# 43. Provider Capabilities

```rust
pub struct DeploymentProviderCapabilities {
    pub rolling: bool,
    pub canary: bool,
    pub blue_green: bool,
    pub traffic_split: bool,
    pub atomic_switch: bool,
    pub rollback: RollbackCapability,
}
```

---

# 44. Capability Honesty

If provider cannot do blue-green atomically:

```text
do not claim it can
```

---

# 45. Deployment Strategy

```rust
pub enum DeploymentStrategy {
    Recreate,
    Rolling(RollingStrategy),
    Canary(CanaryStrategy),
    BlueGreen(BlueGreenStrategy),
    Immediate,
    Custom(DeploymentStrategyId),
}
```

---

# 46. Recreate

```text
stop old
start new
```

Simple but downtime possible.

---

# 47. Rolling

Gradually replaces instances.

---

# 48. Rolling Strategy

```rust
pub struct RollingStrategy {
    pub max_unavailable: PercentageOrCount,
    pub max_surge: PercentageOrCount,
    pub step_timeout: Duration,
}
```

---

# 49. Canary

Small percentage receives new release first.

---

# 50. Canary Strategy

```rust
pub struct CanaryStrategy {
    pub steps: Vec<CanaryStep>,
    pub health_gate: HealthPolicy,
}
```

---

# 51. Canary Step

```rust
pub struct CanaryStep {
    pub percentage: Percent,
    pub minimum_duration: Duration,
}
```

---

# 52. Blue-Green

Maintain old and new environments/sets.

Switch traffic after health validation.

---

# 53. Blue-Green Strategy

```rust
pub struct BlueGreenStrategy {
    pub warmup: Duration,
    pub switch: TrafficSwitchPolicy,
    pub old_retention: Duration,
}
```

---

# 54. Immediate

Appropriate for:

```text
static file pointer
small internal service
```

where safe.

---

# 55. Strategy Selection

Plan/policy determines.

---

# 56. Strategy Capability Match

Provider must support required strategy.

---

# 57. Health Policy

```rust
pub struct HealthPolicy {
    pub checks: Vec<HealthCheckSpec>,
    pub success_window: Duration,
    pub failure_threshold: u32,
    pub timeout: Duration,
}
```

---

# 58. Health Check Types

```rust
pub enum HealthCheckKind {
    Http,
    Tcp,
    Process,
    ProviderNative,
    Metric,
    Synthetic,
    Custom(HealthCheckKindId),
}
```

---

# 59. HTTP Health

Validate:

```text
status
latency
optional response predicate
```

---

# 60. Provider-Native Health

Examples:

```text
Kubernetes readiness
VM service state
serverless deployment state
```

---

# 61. Metric Health

Examples:

```text
error rate
latency
saturation
```

Requires observability integration.

---

# 62. Synthetic Health

Run test request/workflow.

---

# 63. Health Gate

Deployment only advances when health gate satisfied.

---

# 64. Health Check Authority

Health checks are evidence, not release artifact identity.

---

# 65. Health Timeout

If no decision in bounded time:

```text
fail/pause/rollback according to policy
```

---

# 66. Canary Progression

```text
10%
  ↓ health
25%
  ↓ health
50%
  ↓ health
100%
```

---

# 67. Automatic Rollback

Policy can specify.

---

# 68. Rollback Policy

```rust
pub struct RollbackPolicy {
    pub automatic_on_failure: bool,
    pub target: RollbackTarget,
    pub migration: MigrationRollbackPolicy,
}
```

---

# 69. Rollback Target

```rust
pub enum RollbackTarget {
    PreviousHealthy,
    Specific(DeploymentRevisionId),
    Manual,
}
```

---

# 70. Previous Healthy

Persist known-good deployment revision.

---

# 71. Deployment Revision

```rust
pub struct DeploymentRevisionId(Ulid);
```

Represents immutable deployed desired state snapshot.

---

# 72. Deployment Revision

```rust
pub struct DeploymentRevision {
    pub id: DeploymentRevisionId,
    pub target: DeploymentTargetId,
    pub plan: DeploymentPlanId,
    pub release: ReleaseId,
    pub state: DeploymentRevisionState,
}
```

---

# 73. Known Good

Only after health gate passes.

---

# 74. Rollback Does Not Rebuild

Uses prior exact release/artifact refs.

---

# 75. Migration Challenge

Database/schema migrations can make rollback unsafe.

---

# 76. Migration Plan

```rust
pub struct MigrationPlan {
    pub migration_artifact: ArtifactId,
    pub direction: MigrationDirection,
    pub compatibility: MigrationCompatibility,
    pub gate: MigrationGate,
}
```

---

# 77. Migration Artifact

Exact immutable migration binary/script/package.

---

# 78. Migration Direction

```rust
pub enum MigrationDirection {
    Forward,
    Backward,
    Bidirectional,
}
```

---

# 79. Compatibility

```rust
pub enum MigrationCompatibility {
    BackwardCompatible,
    ExpandContract,
    Breaking,
    Unknown,
}
```

---

# 80. Recommended Strategy

Production schema change:

```text
expand
deploy compatible app
backfill
switch
contract later
```

---

# 81. Expand-Contract

Strongly preferred for zero-downtime systems.

---

# 82. Migration Gate

```rust
pub enum MigrationGate {
    BeforeRollout,
    AfterCanary,
    BeforeTrafficSwitch,
    AfterRollout,
    Manual,
}
```

---

# 83. Migration Execution

Runs through normal controlled job/executor or dedicated migration worker.

---

# 84. Migration Credentials

Environment-specific SecretRef/workload identity.

---

# 85. Migration Lock

Prevent concurrent incompatible migrations.

---

# 86. Migration Idempotency

Migration system must know whether migration already applied.

---

# 87. Migration State

```text
Pending
Applying
Applied
Failed
Unknown
RolledBack
```

---

# 88. Unknown Migration

Never blindly rerun destructive migration.

Inspect database/schema state first.

---

# 89. Rollback Compatibility

If migration is not backward-compatible:

```text
automatic app rollback may be unsafe
```

---

# 90. Deployment Plan Must Know

Rollback gate evaluates migration compatibility.

---

# 91. Irreversible Migration

Requires explicit approval/risk acknowledgement.

---

# 92. Database Backup

Policy may require backup/checkpoint before irreversible migration.

---

# 93. Deployment Approval

```rust
pub struct DeploymentApproval {
    pub deployment: DeploymentId,
    pub plan: DeploymentPlanId,
    pub environment: EnvironmentId,
    pub approver: PrincipalId,
    pub decision: ApprovalDecision,
    pub policy: PolicyDigest,
}
```

---

# 94. Approval Binds Plan

Plan change invalidates approval.

---

# 95. Production Approval

Can require:

```text
MFA
release manager
environment owner
security approval
```

---

# 96. Separation of Duties

Possible:

```text
release approver != deployment approver
developer != production deployer
```

---

# 97. Break-Glass Deployment

Explicit emergency path.

---

# 98. Break-Glass Requirements

```text
reason
strong auth
scope
expiry
audit
```

---

# 99. Break-Glass Does Not Bypass Artifact Identity

Still exact ReleaseId/digest.

---

# 100. Environment Policy

Examples:

```text
staging allows automatic deploy
production requires approval
DR requires operator
```

---

# 101. Deployment Authorization

Permissions:

```text
deployment.read
deployment.create
deployment.approve
deployment.execute
deployment.rollback
deployment.admin
```

---

# 102. Environment Scope

Permissions scoped to EnvironmentId.

---

# 103. Secret Scope

Production secrets only available to production deployment workload identity.

---

# 104. Workload Identity

Deployment worker receives short-lived identity.

---

# 105. Provider Credentials

Prefer:

```text
federated workload credentials
```

over static cloud keys.

---

# 106. Least Privilege

Credential can modify only target/environment resources.

---

# 107. No General Cloud Admin

Publisher/deployer should not receive full account owner permissions.

---

# 108. Kubernetes Adapter

Core inputs normalize into Kubernetes desired objects or controller operation.

---

# 109. Kubernetes Deployment

Use exact image digest:

```text
image@sha256:...
```

not mutable tag alone.

---

# 110. Kubernetes Manifests

Can be:

```text
generated
templated
prebuilt immutable artifact
```

but final applied manifest digest is recorded.

---

# 111. Kubernetes Manifest Identity

```rust
pub struct DeploymentManifestId(Digest);
```

---

# 112. Templating

Values resolved before plan freeze.

---

# 113. Runtime Secret References

Prefer Kubernetes/secret-provider integration rather than embedding plaintext.

---

# 114. Server-Side Apply

Potential adapter implementation.

---

# 115. Ownership

Use field manager/labels for Forgeyard-managed resources.

---

# 116. Kubernetes Drift

Compare managed fields/resources to desired.

---

# 117. Kubernetes Health

```text
Deployment rollout
StatefulSet readiness
Job completion
custom health
```

---

# 118. Helm

Can be supported as adapter/interoperability.

Do not make Helm core deployment model.

---

# 119. Kustomize

Likewise.

---

# 120. Container Service Adapter

Examples:

```text
ECS-like
Cloud Run-like
Azure Container Apps-like
```

provider-specific adapters.

---

# 121. Image Identity

Always exact OCI digest.

---

# 122. VM Deployment

Potential strategies:

```text
artifact copy + service switch
immutable image replacement
system package install
```

---

# 123. Preferred VM Strategy

Immutable machine/image replacement where practical.

---

# 124. In-Place VM Update

Supported for legacy/simple environments.

Requires stricter drift/rollback handling.

---

# 125. Bare-Metal

Managed target with explicit agent/SSH transport.

---

# 126. SSH Deployment

If supported:

```text
SSH is transport
```

not deployment authority.

---

# 127. SSH Commands

Generated from typed deployment plan.

Do not allow arbitrary interactive admin shell as normal deployment workflow.

---

# 128. Target Agent

Optional lightweight deployment agent can receive exact release commands.

---

# 129. Static Hosting

Deploy immutable site bundle.

---

# 130. Atomic Static Publish

Upload new version then switch pointer.

---

# 131. Serverless

Deploy exact function/package artifact.

---

# 132. Serverless Version

Provider version ID recorded.

---

# 133. Serverless Alias

Mutable alias points to immutable provider version.

---

# 134. Device Fleet

Deployment to managed devices/edge nodes.

---

# 135. Device Deployment

```text
release artifact
  ↓
fleet cohort
  ↓
install
  ↓
health/check-in
  ↓
progressive rollout
```

---

# 136. Device Offline

Desired state persists until device reconnects.

---

# 137. Device Rollout Ring

Can integrate with Release ring but deployment state remains device/fleet actual state.

---

# 138. Mobile App Store

Store publication belongs Release.

Installing to test devices belongs Deployment/Device Lab.

---

# 139. Configuration

Deployment config is separate from package/release artifact.

---

# 140. Runtime Configuration

Can vary by environment without rebuilding artifact.

---

# 141. Configuration Artifact

Non-secret config can be immutable CAS object.

---

# 142. Secret Configuration

SecretRef.

---

# 143. Twelve-Factor-Like Separation

Application artifact remains environment-neutral where possible.

---

# 144. Environment Variable Injection

At deployment runtime/provider configuration.

---

# 145. Configuration Version

Deployment plan binds exact config version/digest.

---

# 146. Config Drift

Detected.

---

# 147. Deployment Manifest

```rust
pub struct ForgeyardDeploymentManifest {
    pub plan: DeploymentPlanId,
    pub release: ReleaseId,
    pub target: DeploymentTargetId,
    pub artifacts: Vec<DeploymentArtifactRef>,
    pub config: Vec<DeploymentConfigRef>,
    pub strategy: DeploymentStrategy,
}
```

---

# 148. Manifest Storage

CAS.

---

# 149. Deployment Plan Creation

```text
ReleaseId
  ↓
resolve exact artifacts
  ↓
resolve target
  ↓
resolve config refs
  ↓
resolve strategy/health
  ↓
resolve migration
  ↓
policy
  ↓
DeploymentPlanId
```

---

# 150. Dry Run

```text
forgeyard deploy plan
```

shows intended actions.

---

# 151. Provider Diff

If supported:

```text
desired vs actual
```

before apply.

---

# 152. Plan Review

Protected environment may require approval after diff.

---

# 153. Deployment Apply

Creates desired generation.

---

# 154. Apply Request

```rust
pub struct ApplyDeploymentRequest {
    pub deployment: DeploymentId,
    pub plan: DeploymentPlanId,
    pub target: DeploymentTargetId,
    pub generation: DeploymentGeneration,
    pub idempotency: DeploymentIdempotencyKey,
}
```

---

# 155. Idempotency

Same semantic apply retry safe.

---

# 156. External Effect State

```rust
pub enum ProviderApplyState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Unknown,
}
```

---

# 157. Unknown Outcome

Network timeout after provider accepted change.

---

# 158. Unknown Handling

```text
inspect provider
compare generation/artifact
decide
```

---

# 159. Never Blind Retry Unknown

Could duplicate resources or trigger migration twice.

---

# 160. Provider Operation Identity

Use labels/tags/request IDs where possible.

---

# 161. Drift

Drift = actual provider state differs from desired Forgeyard-managed state.

---

# 162. Drift Types

```rust
pub enum DriftKind {
    Artifact,
    Configuration,
    ReplicaCount,
    Traffic,
    Resource,
    SecretReference,
    Unknown,
}
```

---

# 163. Drift Detection

Periodic reconciler/provider inspect.

---

# 164. Drift Policy

```rust
pub enum DriftPolicy {
    ReportOnly,
    ReconcileAutomatically,
    RequireApproval,
}
```

---

# 165. Production Default

Potential:

```text
report + controlled reconcile
```

depending organization.

---

# 166. Manual Hotfix Drift

If operator edits Kubernetes resource manually:

Forgeyard detects.

---

# 167. Import Drift

Operator can adopt actual state into new deployment plan only through explicit action.

---

# 168. No Silent Adoption

Never silently make drift desired state.

---

# 169. Desired State Authority

Forgeyard plan is authority for Forgeyard-managed fields/resources.

---

# 170. Shared Ownership

Provider-specific field ownership can avoid fighting other controllers.

---

# 171. Pausing

Deployment can pause between rollout steps.

---

# 172. Resume

Revalidate:

```text
plan
release
policy
provider state
```

before continuing if pause long.

---

# 173. Cancellation

Before provider changes:

```text
cancel safely
```

During rollout:

depends strategy.

---

# 174. Cancel Is Not Rollback

Cancellation stops further action.

Rollback intentionally converges to prior release.

---

# 175. Cancel During Canary

May leave canary running until explicit rollback/cleanup.

Plan defines.

---

# 176. Automatic Cleanup

Temporary canary/green resources cleaned after outcome.

---

# 177. Cleanup State

Track separately.

---

# 178. Cleanup Failure

Deployment can be Healthy but cleanup degraded.

---

# 179. Resource Leak Reconciler

Find temporary rollout resources.

---

# 180. Deployment Locks

Prevent conflicting concurrent deployment to same target.

---

# 181. DeploymentLock

```rust
pub struct DeploymentLock {
    pub target: DeploymentTargetId,
    pub deployment: DeploymentId,
    pub generation: DeploymentGeneration,
    pub expires_at: Timestamp,
}
```

---

# 182. Lock Scope

Per managed target/environment.

---

# 183. Parallel Components

If target supports multiple independent services, locks can be component-scoped.

---

# 184. Initial Recommendation

Start target/application-component scope.

---

# 185. Lock Expiry

Durable timer/reconcile.

---

# 186. HA Controllers

Multiple daemon replicas can reconcile same environment safely using claims/versioning/locks.

---

# 187. No Global Leader Required Initially

Postgres/store leases sufficient for most deployment workers.

---

# 188. Raft Later

May coordinate exclusive global operations.

---

# 189. Deployment Event Model

```text
DeploymentPlanned
DeploymentApproved
DeploymentStarted
MigrationStarted
MigrationCompleted
RolloutStepStarted
RolloutStepHealthy
DeploymentPaused
DeploymentHealthy
DeploymentDegraded
DeploymentFailed
RollbackStarted
DeploymentRolledBack
DriftDetected
```

---

# 190. Event Idempotency

Consumers use DeploymentId/generation/version.

---

# 191. Reconciler

```text
forgeyard-deploy-reconcile
```

checks:

```text
desired generation not applied
provider operation Unknown
health state stale
drift
lock expiry
migration unknown
rollback incomplete
temporary rollout resource leak
```

---

# 192. Reconcile Fixed Point

At convergence:

```text
actual == desired
health acceptable
no pending required rollout action
```

---

# 193. Provider Read Model

Provider-specific details normalized but raw provider reference retained.

---

# 194. Provider Resource Ref

```rust
pub struct ProviderResourceRef {
    pub provider: DeploymentProviderId,
    pub external_id: BoundedString,
}
```

---

# 195. External IDs

Diagnostic/integration identity, not Forgeyard business identity.

---

# 196. Health Observation

```rust
pub struct HealthObservation {
    pub check: HealthCheckId,
    pub status: HealthStatus,
    pub observed_at: Timestamp,
    pub evidence: Option<CasObjectRef>,
}
```

---

# 197. Health Status

```rust
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

---

# 198. Health Aggregation

Policy decides.

---

# 199. Health Stability Window

Avoid passing on one transient sample.

---

# 200. Metrics-Based Gate

Example:

```text
error rate < 1%
p95 latency < threshold
for 10 minutes
```

---

# 201. Baseline Comparison

Canary can compare to current stable baseline.

---

# 202. Observability Integration

Later `17` provides metrics query abstraction.

---

# 203. No Direct Prometheus Dependency in Core

Use health/metric provider trait.

---

# 204. Metric Provider Trait

```rust
#[async_trait]
pub trait DeploymentMetricProvider {
    async fn evaluate(
        &self,
        query: MetricHealthQuery,
    ) -> Result<MetricHealthResult, MetricError>;
}
```

---

# 205. Manual Health Approval

Some specialized systems may require operator gate.

---

# 206. Synthetic Test Job

Forgeyard can run normal CI job against deployed endpoint.

---

# 207. Synthetic Test Identity

Run/job linked to DeploymentId/generation.

---

# 208. Synthetic Result

Health evidence.

---

# 209. Deployment Artifact Verification

Before apply:

```text
artifact exists
release valid
signature valid if required
```

---

# 210. Pull-Time Verification

Target agent/provider can also verify digest/signature.

---

# 211. OCI Pull

Use exact digest.

---

# 212. Binary Install

Verify CAS/download digest before install.

---

# 213. Update Feed Deployment

For fleets, client verifies signed feed/package.

---

# 214. Artifact Availability

Release must be accessible to target.

---

# 215. Registry Replication

Deployment can require artifact replicated to region before rollout.

---

# 216. CAS/Registry Locality

Optimization/precondition.

---

# 217. Air-Gapped Deployment

Import release bundle into target-side Forgeyard/CAS then deploy exact artifacts.

---

# 218. Air-Gap Trust

Verify release/evidence/signature before registration.

---

# 219. No Internet Deployment

Supported if all dependencies/artifacts/config/providers local.

---

# 220. Rollback Data Safety

Application binary rollback is easy compared to data rollback.

---

# 221. Rollback Safety Check

```rust
pub struct RollbackSafety {
    pub application_compatible: bool,
    pub schema_compatible: bool,
    pub config_compatible: bool,
}
```

---

# 222. Rollback Unsafe

Require operator.

---

# 223. Forward Fix

Sometimes safer than rollback after irreversible migration.

---

# 224. Deployment History

Per target:

```text
revision 1 Release A
revision 2 Release B
revision 3 Release C
```

---

# 225. History Immutability

Do not rewrite.

---

# 226. Current Pointer

Target current desired revision is mutable pointer to immutable history.

---

# 227. Previous Healthy Revision

Tracked.

---

# 228. Rollback Candidate

Select from known healthy compatible revisions.

---

# 229. Deployment Diff

Between revisions:

```text
release change
artifact digest
config
strategy
migration
```

---

# 230. Environment Promotion

Staging -> production should create new DeploymentPlan using same ReleaseId.

---

# 231. No Artifact Rebuild Across Environments

Critical invariant.

---

# 232. Config May Differ

Environment-specific config/secret refs can differ.

---

# 233. Artifact Must Not Differ

If production binary differs, it is another ReleaseId/artifact.

---

# 234. Release Evidence Reuse

Same release evidence.

---

# 235. Environment-Specific Evidence

Deployment health/migration evidence separate.

---

# 236. Deployment Evidence

Examples:

```text
approval
provider apply result
health checks
migration result
drift scan
rollback result
```

---

# 237. Deployment Evidence Storage

Metadata + CAS refs.

---

# 238. Deployment Audit

Mandatory:

```text
plan freeze
approval
apply
break-glass
migration
rollback
drift adoption
manual override
```

---

# 239. Manual Override

Dangerous.

---

# 240. Manual Override Types

```text
pause
skip health gate
force rollback
adopt drift
continue after warning
```

---

# 241. Override Requirements

```text
permission
reason
audit
optional MFA
```

---

# 242. No "Mark Healthy"

Operator cannot simply fake health without explicit override evidence.

---

# 243. Deployment Config Model

Example RON:

```ron
(
    environments: [
        (
            name: "production",
            class: Production,
            targets: [
                (
                    name: "prod-k8s-eu",
                    provider: "kubernetes-prod",
                    strategy: Canary((
                        steps: [
                            (percentage: 10, minimum_duration: "10m"),
                            (percentage: 50, minimum_duration: "20m"),
                            (percentage: 100, minimum_duration: "30m"),
                        ],
                    )),
                ),
            ],
        ),
    ],
)
```

---

# 244. Config Validation

Check:

```text
target exists
strategy supported
health checks valid
credentials refs valid
rollback policy compatible
```

---

# 245. Config Version

```rust
pub struct DeploymentSchemaVersion(u16);
```

---

# 246. Deployment IR

Optional normalized:

```rust
pub struct DeploymentIr { ... }
```

If human config complex enough.

---

# 247. Plan Compiler Flow

```text
RON/config/API intent
  ↓
parse
  ↓
validate
  ↓
resolve exact ReleaseId/artifacts
  ↓
resolve target/provider capability
  ↓
resolve policy
  ↓
DeploymentPlan
  ↓
DeploymentPlanId
```

---

# 248. Provider Adapter Requirements

Must support:

```text
inspect
apply
health integration
rollback if claimed
idempotency/reconcile
```

---

# 249. Kubernetes Provider Layout

```text
forgeyard-deploy-kubernetes/
├── client.rs
├── model.rs
├── render.rs
├── apply.rs
├── inspect.rs
├── rollout.rs
├── drift.rs
└── health.rs
```

---

# 250. VM Adapter Layout

```text
forgeyard-deploy-vm/
├── transport.rs
├── install.rs
├── service.rs
├── inspect.rs
├── rollback.rs
└── health.rs
```

---

# 251. Serverless Adapter Layout

Provider SDK isolated.

---

# 252. Bare-Metal Agent

Could reuse Forgeyard agent transport concepts with narrower deployment role.

---

# 253. Deployment Worker Security

Restricted to deployment actions.

---

# 254. No Arbitrary Cloud API

Adapter code should expose only required provider operations.

---

# 255. Provider Credentials SecretRef

Config stores ref.

---

# 256. Credential Resolution

Late at apply/inspect.

---

# 257. Read-Only Inspect Credential

Could be separate from apply credential.

---

# 258. Principle of Least Privilege

Prefer separate:

```text
inspect
deploy
rollback
```

permissions where provider supports.

---

# 259. Deployment API

Potential:

```text
POST /v1/deployments/plan
POST /v1/deployments
POST /v1/deployments/{id}/approve
POST /v1/deployments/{id}/apply
POST /v1/deployments/{id}/pause
POST /v1/deployments/{id}/resume
POST /v1/deployments/{id}/rollback
GET  /v1/deployments/{id}
GET  /v1/environments/{id}/state
```

---

# 260. API Idempotency

Plan/apply/rollback accept idempotency keys.

---

# 261. CLI

```text
forgeyard deploy plan
forgeyard deploy diff
forgeyard deploy apply
forgeyard deploy approve
forgeyard deploy status
forgeyard deploy pause
forgeyard deploy resume
forgeyard deploy rollback
forgeyard deploy drift
forgeyard deploy history
forgeyard deploy verify
```

---

# 262. `deploy plan`

Pure planning/diff.

---

# 263. `deploy diff`

Desired vs actual/provider state.

---

# 264. `deploy apply`

Creates/applies exact plan.

---

# 265. `deploy verify`

Re-runs health/state verification.

---

# 266. `deploy drift`

Shows unmanaged divergence.

---

# 267. Dioxus UI

Deployment page:

```text
Overview
Plan
Diff
Rollout
Health
Migrations
Drift
Approvals
History
Rollback
Audit
```

---

# 268. Environment Dashboard

Shows:

```text
current release
desired release
health
drift
last deployment
previous healthy
```

---

# 269. Rollout UI

Visual:

```text
10% → 50% → 100%
```

with health at each step.

---

# 270. Migration UI

Shows:

```text
compatibility
status
backup requirement
rollback safety
```

---

# 271. Drift UI

Shows exact difference and ownership.

---

# 272. Unknown State UI

Must be explicit:

```text
provider outcome uncertain
reconciliation required
```

---

# 273. Metrics

```text
deployment_created
deployment_apply_duration
deployment_health_duration
deployment_success
deployment_failure
deployment_unknown
deployment_rollback
deployment_drift
migration_duration
migration_failure
```

---

# 274. Rollout Metrics

```text
canary_step_duration
bluegreen_switch_duration
rolling_progress
```

---

# 275. Health Metrics

By environment class/provider type.

---

# 276. No High-Cardinality IDs

Use tracing.

---

# 277. Tracing

```text
deploy.plan
deploy.diff
deploy.apply
deploy.provider
deploy.health
deploy.migration
deploy.rollout
deploy.rollback
deploy.reconcile
```

---

# 278. Health of Deployment Subsystem

Check:

```text
provider credentials
provider reachability
reconciler
metric provider
migration runner
```

---

# 279. Doctor

```text
forgeyard deploy doctor
```

---

# 280. Doctor Checks

```text
provider connectivity
target visibility
credential scope
strategy support
health-check provider
rollback capability
```

---

# 281. Event/Reconcile Integration

Every external effect has event fast path + reconcile slow path.

---

# 282. Timer Integration

Durable timers for:

```text
canary soak
health timeout
pause expiry
lock expiry
scheduled deploy
```

---

# 283. Scheduled Deployment

Optional.

---

# 284. Scheduled Deployment Gate

At due time recheck:

```text
release
policy
approval freshness
target state
```

---

# 285. Deployment Reconciler States

```text
Planned
Ready
Applying
Verifying
Paused
Unknown
RollingBack
```

are actively reconciled.

---

# 286. Healthy State Reconcile

Periodic drift/health checks can still update to Degraded.

---

# 287. Degraded State

May trigger:

```text
alert
auto rollback
manual review
```

---

# 288. Auto Rollback Delay

Could require stable degradation window to avoid flapping.

---

# 289. Flap Protection

Hysteresis/minimum duration.

---

# 290. Deployment Stability

```rust
pub struct StabilityPolicy {
    pub unhealthy_for: Duration,
    pub healthy_for: Duration,
}
```

---

# 291. Canary Metrics Baseline

Compare new vs old cohort.

---

# 292. Statistical Sophistication

Optional later.

Initial thresholds enough.

---

# 293. Deployment Cost

Optional scheduling/planning fact, not core correctness.

---

# 294. Multi-Region

Deployment can target multiple regions.

---

# 295. Region Deployment Set

```rust
pub struct DeploymentSetId(Ulid);
```

---

# 296. Multi-Target Deployment

One logical deployment may fan out:

```text
region A
region B
region C
```

---

# 297. Fan-Out Policy

```text
serial
parallel
wave
```

---

# 298. Wave Deployment

```text
region A
  ↓ health
region B
  ↓ health
region C
```

---

# 299. Failure Domain

Useful for safer rollout.

---

# 300. Global Rollback

Policy determines if one region failure rolls back all.

---

# 301. Partial Deployment

Explicit state per target.

---

# 302. Deployment Aggregate

```rust
pub struct DeploymentSetStatus {
    pub healthy: usize,
    pub failed: usize,
    pub unknown: usize,
    pub pending: usize,
}
```

---

# 303. No False Global Success

All required targets must satisfy policy.

---

# 304. Disaster Recovery

DR environment can deploy same release or designated fallback.

---

# 305. DR Promotion

Explicit policy/action.

---

# 306. Data Replication

Outside deployment core, but health gate can require replication readiness.

---

# 307. Static Asset/CDN Deployment

Upload immutable assets then switch manifest/pointer.

---

# 308. CDN Invalidation

External side effect.

Track/reconcile if required.

---

# 309. Edge Fleet

Desired version per cohort/device.

---

# 310. Offline Edge

Converges when devices reconnect.

---

# 311. Deployment GC

Old deployment metadata retained by policy.

Temporary rollout resources cleaned.

---

# 312. Release Artifact Retention

Current/rollback deployment revisions pin release artifacts.

---

# 313. Rollback Root

Previous healthy release retained.

---

# 314. CAS GC Integration

Deployment revisions are GC roots while needed.

---

# 315. Metadata Store

Entities:

```text
environments
deployment_targets
deployment_plans
deployments
deployment_revisions
deployment_approvals
deployment_locks
provider_operations
migration_operations
health_observations
drift_records
```

---

# 316. Provider Operation

```rust
pub struct ProviderOperation {
    pub id: ProviderOperationId,
    pub deployment: DeploymentId,
    pub kind: ProviderOperationKind,
    pub state: ProviderApplyState,
    pub external_ref: Option<ProviderResourceRef>,
}
```

---

# 317. Operation Attempts

Preserve retry history.

---

# 318. Deployment Error Model

```rust
pub enum DeploymentError {
    InvalidPlan,
    ApprovalRequired,
    PolicyDenied,
    ProviderUnavailable,
    Authentication,
    Authorization,
    Conflict,
    HealthFailed,
    MigrationFailed,
    RollbackUnsafe,
    UnknownOutcome,
    DriftConflict,
    Internal,
}
```

---

# 319. Conflict

Example:

```text
another deployment changed target generation
```

---

# 320. Compare-and-Swap Generation

Before apply:

```text
expected current generation
```

---

# 321. Prevent Lost Update

If target changed:

```text
replan
```

---

# 322. Terraform/IaC Interop

Forgeyard can deploy artifacts into infrastructure managed by Terraform/OpenTofu.

---

# 323. Do Not Replace IaC

Infrastructure provisioning remains separate.

---

# 324. IaC Deployment Adapter

Optional pipeline/deployment step can trigger pre-reviewed IaC plan/apply.

---

# 325. IaC State

External authority.

Reconcile provider result.

---

# 326. GitOps Interop

Forgeyard can update deployment repo/manifest via Change Proposal.

---

# 327. GitOps Mode

Desired state may be committed to Git and external controller applies.

---

# 328. GitOps Adapter

Forgeyard tracks:

```text
commit exact manifest
external controller health
```

---

# 329. GitOps Is Optional

Not core deployment architecture.

---

# 330. ArgoCD/Flux Interop

Potential external provider adapter.

---

# 331. Native Forgeyard Controller

Direct provider apply.

---

# 332. Push vs Pull Deployment

Support both conceptually:

```text
Push: Forgeyard calls provider
Pull: target/agent reconciles desired state
```

---

# 333. Pull Agent

Useful for:

```text
edge
air-gap
restricted networks
```

---

# 334. Pull Security

Agent requests desired state after authentication.

---

# 335. Desired State Signature

Can sign deployment manifest for high-assurance pull agents.

---

# 336. Target Verifies

```text
manifest signature
release signature
artifact digest
```

---

# 337. Offline Queue

Target applies when connectivity returns.

---

# 338. Deployment Testkit

```text
forgeyard-deploy-testkit/src/
├── lib.rs
├── environment.rs
├── target.rs
├── plan.rs
├── provider.rs
├── rollout.rs
├── health.rs
├── migration.rs
├── drift.rs
├── rollback.rs
└── assertions.rs
```

---

# 339. Unit Tests

Test:

```text
DeploymentPlanId
state transitions
strategy validation
rollback safety
```

---

# 340. Provider Conformance Tests

Every provider adapter:

1. inspect;
2. apply;
3. idempotent retry;
4. unknown reconcile;
5. health;
6. rollback capability.

---

# 341. Exact Artifact Test

Provider receives exact release/package digest.

---

# 342. No Rebuild Test

Deployment code path never invokes build/package/signing.

---

# 343. Approval Binding Test

Change plan -> previous approval invalid.

---

# 344. Generation Conflict Test

Two concurrent deploys cannot overwrite silently.

---

# 345. Canary Test

Failure at 10% prevents 50%.

---

# 346. Canary Auto-Rollback Test

Health failure -> previous healthy.

---

# 347. Rolling Test

Respects max unavailable/surge.

---

# 348. Blue-Green Test

Traffic switches only after new side healthy.

---

# 349. Pause/Resume Test

Resume revalidates provider/policy if needed.

---

# 350. Unknown Provider Result Test

Timeout after successful remote apply -> Unknown -> inspect -> Succeeded.

---

# 351. Migration Unknown Test

Never blindly reruns potentially destructive migration.

---

# 352. Irreversible Migration Test

Automatic rollback denied.

---

# 353. Drift Test

Manual provider edit detected.

---

# 354. Drift Adoption Test

Requires explicit new plan/action.

---

# 355. Secret Test

Only environment-scoped workload receives deploy credential.

---

# 356. Kubernetes Test

Image applied by digest.

---

# 357. VM Test

Installed package digest verified before activation.

---

# 358. Static Hosting Test

Pointer switches only after assets uploaded/verified.

---

# 359. Pull Agent Test

Offline target converges after reconnect.

---

# 360. Rollback Retention Test

Previous healthy artifact remains pinned.

---

# 361. Multi-Region Test

Required region failure prevents false aggregate success.

---

# 362. Failure Injection

```text
provider outage
credential revocation
health backend outage
network partition
migration failure
rollback failure
DB outage
```

---

# 363. Fuzzing

Fuzz:

```text
deployment manifest parser
target config
strategy config
provider normalized state
drift diff
```

---

# 364. Scale Tests

```text
thousands of environments
large Kubernetes fleets
large device fleets
multi-region waves
```

---

# 365. Reconcile Scale

Use indexed due/active states.

---

# 366. Implementation Phase 1 — Core Model

Implement:

```text
EnvironmentId
DeploymentTargetId
DeploymentId
DeploymentPlanId
Desired/Observed state
```

---

# 367. Phase 2 — Generic Provider API

Inspect/apply/reconcile.

---

# 368. Phase 3 — Basic Immediate/Recreate

Simple server/static target.

---

# 369. Phase 4 — Kubernetes

Digest-pinned native controller.

---

# 370. Phase 5 — Health Gates

HTTP/provider-native/synthetic.

---

# 371. Phase 6 — Rolling/Canary

Progressive rollout.

---

# 372. Phase 7 — Rollback

Previous-known-good model.

---

# 373. Phase 8 — Migrations

Expand-contract and migration safety.

---

# 374. Phase 9 — Drift

Detection/reconcile/adoption.

---

# 375. Phase 10 — VM/Bare Metal/Serverless

Additional adapters.

---

# 376. Phase 11 — Pull/Edge/Device

Offline-capable deployment.

---

# 377. Phase 12 — Hardening

HA, multi-region, failure injection, scale, fuzzing.

---

# 378. Acceptance Tests

1. Deployment plan references exact ReleaseId.
2. Deployment plan references exact artifact/package digests.
3. DeploymentPlanId is deterministic.
4. Plan change invalidates approval.
5. Production deployment requires configured approval.
6. Deployment never invokes build/package/signing.
7. Same ReleaseId can deploy to staging then production.
8. Artifact bytes remain identical across environments.
9. Environment configuration may differ without rebuilding artifact.
10. Provider adapter applies exact desired generation.
11. Concurrent deployment generation conflict is detected.
12. Unknown provider outcome is inspected before retry.
13. Provider idempotency prevents duplicate resources.
14. Canary does not advance after failed health gate.
15. Rolling strategy honors availability limits.
16. Blue-green does not switch traffic before health passes.
17. Automatic rollback uses exact prior known-good revision.
18. Rollback never rebuilds old release.
19. Irreversible migration can block automatic rollback.
20. Migration state is idempotent/reconciled.
21. Drift is detected.
22. Drift is not silently adopted.
23. Kubernetes uses image digest, not tag alone.
24. Deploy credential is environment/target scoped.
25. Workload identity expires after deployment.
26. Break-glass deployment remains exact-artifact and audited.
27. Release signatures/evidence can be reverified before production.
28. Pull target verifies signed desired/release state.
29. Offline target converges when connectivity returns.
30. Previous healthy release remains retained for rollback.
31. Multi-target deployment does not report false global success.
32. Event loss is repaired by deployment reconciler.
33. Same deployment model works standalone/distributed.
34. Provider-specific SDKs stay in adapter crates.
35. Forgeyard deploys its own services using this subsystem.

---

# 379. Production Readiness Gates

Do not call deployment production-ready until:

```text
exact ReleaseId/digest binding proven
desired/actual reconciliation stable
idempotent provider apply tested
Unknown result handling tested
deployment generation concurrency safe
health gates stable
rollback safety modeled
migration safety tested
secret/workload identity integration complete
drift detection available
audit complete
no-rebuild invariant tested
```

Advanced multi-region, device fleet, GitOps, serverless, and metric-based canary support can mature incrementally.

---

# 380. Architectural Invariants

1. deployment consumes immutable released artifacts;
2. deployment never rebuilds code;
3. deployment never repackages release artifacts;
4. deployment never resigns artifacts;
5. DeploymentPlanId binds exact release/target/config/strategy;
6. approvals bind exact plan;
7. desired and actual state are separate;
8. reconciliation is correctness mechanism;
9. external provider calls are idempotent/reconciled;
10. Unknown outcome is never blindly retried;
11. deployment generation prevents lost updates;
12. exact artifact digest is provider target identity where possible;
13. mutable tags/aliases are navigation only;
14. staging/production reuse same artifact bytes;
15. environment config stays separate;
16. production secrets are environment-scoped;
17. workload credentials are short-lived;
18. health gates are explicit;
19. rollout does not advance on failed required health;
20. rollback uses known immutable previous revision;
21. irreversible migrations can make rollback unsafe;
22. drift is never silently adopted;
23. provider capabilities are reported honestly;
24. deployment locks are durable;
25. deployment history is immutable;
26. previous healthy revisions are retained as rollback roots;
27. public/provider SDKs stay adapter-local;
28. push and pull deployment share desired-state semantics;
29. standalone/distributed share deployment semantics;
30. Forgeyard dogfoods its deployment system.

---

# 381. Final Target Architecture

```text
                    Immutable Release
                           │
                           ▼
                       ReleaseId
                           │
                           ▼
                    DeploymentPlan
                           │
                           ▼
                    DeploymentPlanId
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
          Policy        Approval       Health
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                      Desired State
                           │
                           ▼
                    Provider Adapter
                           │
                           ▼
                      Actual State
                           │
                           ▼
                      Reconciler
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
         Continue        Rollback       Operator
            │
            ▼
          Healthy
```

---

# 382. Final Architectural Position

Deployment identity:

```text
ReleaseId
+
exact artifact digests
+
EnvironmentId
+
DeploymentTargetId
+
config refs
+
strategy
+
migration plan
+
health policy
+
PolicyDigest
  ↓
DeploymentPlanId
```

Execution:

```text
Approved DeploymentPlanId
  ↓
desired generation
  ↓
provider apply
  ↓
observed state
  ↓
health gate
  ↓
Healthy / Rollback / Operator
```

Rollback:

```text
current unhealthy generation
  ↓
previous known-good DeploymentRevisionId
  ↓
migration compatibility check
  ↓
apply exact prior ReleaseId
```

The key guarantee is:

> **Forgeyard deployment is a reconciliation system over immutable release identities: it decides what exact release should be running, observes what is actually running, and safely converges the target without ever rebuilding the software it was asked to deploy.**

---

# 383. New-Repository Sequence

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
