# 53 — Forgeyard Infrastructure-as-Code, Environment Provisioning, Preview Environments & Drift Reconciliation System Architecture

**Document type:** Core Infrastructure-as-Code, Environment Provisioning, Preview Environment, Drift Detection & Infrastructure Reconciliation System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** declarative infrastructure specifications, environment provisioning, IaC adapters, cloud/Kubernetes resource creation, plan/apply separation, preview environments, ephemeral environments, state ownership, drift detection, destroy workflows, credentials, policy gates, environment promotion, TTL/cost governance, and infrastructure evidence  
**Architecture style:** Desired-state infrastructure, explicit plan/apply, immutable plan identity, provider-neutral orchestration, state separation, policy-governed mutation, short-lived credentials, reconciliation, and no hidden infrastructure side effects inside generic pipeline scripts  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Deployment, Policy/Authz, Secrets, Configuration, Triggers, Cost/FinOps, Catalog, Multi-Tenancy, Audit, Reliability, Federation, Release, and Developer Experience. This subsystem governs infrastructure creation and preview environments without collapsing infrastructure authority into arbitrary job execution.

---

# 1. Purpose

Forgeyard can build, package, release, and deploy software, but many real deployments also require infrastructure changes such as:

```text
Kubernetes namespaces
cloud VMs
load balancers
databases
object stores
DNS
queues
secrets infrastructure
preview environments
test stacks
network resources
service accounts
```

If every project handles those changes through arbitrary shell scripts such as:

```text
terraform apply
kubectl apply
az resource create
aws ...
gcloud ...
```

inside generic CI jobs, Forgeyard loses:

```text
plan visibility
policy control
resource ownership
drift knowledge
destroy safety
auditability
cost governance
reconciliation
```

The central rule is:

> **Infrastructure mutation is a first-class controlled effect, not an opaque shell side effect hidden inside a build job.**

A second rule is:

> **Forgeyard separates infrastructure intent, plan, approval, apply, observed state, and reconciliation. The IaC engine may calculate/provider-apply changes, but Forgeyard remains the orchestration and governance authority.**

A third rule is:

> **Preview environments are disposable by design, but disposal must still be explicit, scoped, idempotent, and safe. A TTL is not permission to destroy resources Forgeyard cannot prove belong to that environment.**

---

# 2. Architectural Position

```text
                  Infrastructure Source
                         │
                         ▼
               Infrastructure Definition
                         │
                         ▼
                  Plan / Validate
                         │
                    Policy Gate
                         │
                         ▼
               Infrastructure Plan
                         │
                    Approval
                         │
                         ▼
                       Apply
                         │
                         ▼
                  Observed Resources
                         │
                         ▼
                 Drift Reconciliation
```

For preview environments:

```text
ChangeProposalRevision
        ↓
PreviewEnvironmentSpec
        ↓
Plan
        ↓
Provision
        ↓
Deploy exact artifact
        ↓
Test / Review
        ↓
TTL / Close / Merge
        ↓
Safe Destroy
```

---

# 3. Goals

The subsystem MUST:

1. define infrastructure-environment identity;
2. define declarative infrastructure specifications;
3. separate infrastructure plan from apply;
4. support immutable plan identity;
5. support cloud providers through adapters;
6. support Kubernetes through adapters;
7. support Terraform/OpenTofu-compatible orchestration;
8. support other IaC engines where safe;
9. support Forgeyard-native simple resource providers;
10. support infrastructure state ownership;
11. support drift detection;
12. support reconciliation;
13. support preview environments;
14. support ephemeral environment TTL;
15. support safe destroy;
16. support protected production environments;
17. support policy checks;
18. support cost estimation;
19. support short-lived credentials;
20. support tenant/project isolation;
21. support environment promotion;
22. support plan diff;
23. support manual approval;
24. support rollback/recovery guidance;
25. support audit;
26. support UI/API/CLI;
27. support multi-region/federation;
28. support offline planning where possible;
29. support disaster recovery;
30. never turn generic build jobs into implicit infrastructure authority.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Terraform/OpenTofu/Pulumi
replace Kubernetes controllers
replace cloud providers
replace Deployment
replace Secrets
replace Policy
make every infrastructure resource Forgeyard-native
```

Forgeyard orchestrates and governs.

---

# 5. Workspace Structure

```text
crates/infrastructure/
├── forgeyard-infrastructure/
├── forgeyard-infrastructure-model/
├── forgeyard-infrastructure-spec/
├── forgeyard-infrastructure-plan/
├── forgeyard-infrastructure-apply/
├── forgeyard-infrastructure-state/
├── forgeyard-infrastructure-drift/
├── forgeyard-infrastructure-preview/
├── forgeyard-infrastructure-destroy/
├── forgeyard-infrastructure-reconcile/
├── forgeyard-infrastructure-health/
└── forgeyard-infrastructure-testkit/
```

Provider/IaC adapters:

```text
crates/infrastructure-adapters/
├── forgeyard-infra-opentofu/
├── forgeyard-infra-terraform/
├── forgeyard-infra-kubernetes/
├── forgeyard-infra-aws/
├── forgeyard-infra-azure/
├── forgeyard-infra-gcp/
├── forgeyard-infra-libvirt/
└── forgeyard-infra-custom/
```

Core infrastructure crates remain provider-neutral.

---

# 6. InfrastructureEnvironmentId

```rust
pub struct InfrastructureEnvironmentId(Ulid);
```

Stable logical infrastructure environment identity.

Examples:

```text
dev
staging
production
preview/123
load-test/2026-08
```

---

# 7. InfrastructureEnvironmentKind

```rust
pub enum InfrastructureEnvironmentKind {
    Development,
    Test,
    Preview,
    Staging,
    Production,
    DisasterRecovery,
    Benchmark,
    Custom(EnvironmentKindId),
}
```

---

# 8. Environment Identity vs Deployment Environment

Infrastructure and deployment environments can be related but are not identical.

Example:

```text
InfrastructureEnvironmentId
    ↓ creates
Kubernetes namespace + DB + DNS
    ↓
EnvironmentId (Deployment)
    ↓ receives
ReleaseId
```

---

# 9. Binding

```rust
pub struct InfrastructureDeploymentBinding {
    pub infrastructure: InfrastructureEnvironmentId,
    pub deployment_environment: EnvironmentId,
}
```

---

# 10. InfrastructureSpecId

```rust
pub struct InfrastructureSpecId(Digest);
```

Immutable digest of normalized desired infrastructure intent.

---

# 11. Infrastructure Definition

```rust
pub struct InfrastructureSpec {
    pub id: InfrastructureSpecId,
    pub environment: InfrastructureEnvironmentId,
    pub source: InfrastructureSourceRef,
    pub providers: Vec<InfrastructureProviderRef>,
    pub parameters: InfrastructureParameterSet,
}
```

---

# 12. Source

Could originate from:

```text
repository IaC
organization template
generated preview spec
admin-managed infrastructure config
```

---

# 13. Repository IaC

Untrusted input until validated/policy-approved.

---

# 14. Infrastructure Source Types

```rust
pub enum InfrastructureSourceRef {
    OpenTofu(CasObjectRef),
    Terraform(CasObjectRef),
    Kubernetes(CasObjectRef),
    ForgeyardNative(CasObjectRef),
    Custom(InfrastructureSourceKindId, CasObjectRef),
}
```

---

# 15. Source Snapshot Binding

Repository-based IaC binds exact:

```text
SourceSnapshotId
```

---

# 16. Mutable Branch Names

Never infrastructure plan identity.

---

# 17. Infrastructure Parameters

Typed.

---

# 18. Secret Values

Never embedded.

Use:

```text
SecretRef
```

---

# 19. Provider Credentials

Resolved only for plan/apply worker.

---

# 20. InfrastructurePlanId

```rust
pub struct InfrastructurePlanId(Digest);
```

Content-derived immutable plan identity.

---

# 21. Plan Inputs

At minimum:

```text
InfrastructureSpecId
current observed/state identity
provider versions
IaC engine version
parameter digest
policy/config context
```

---

# 22. Infrastructure Plan

```rust
pub struct InfrastructurePlan {
    pub id: InfrastructurePlanId,
    pub environment: InfrastructureEnvironmentId,
    pub spec: InfrastructureSpecId,
    pub changes: Vec<InfrastructureChange>,
    pub risk: InfrastructureRiskSummary,
}
```

---

# 23. Change Kind

```rust
pub enum InfrastructureChangeKind {
    Create,
    Update,
    Replace,
    Delete,
    NoOp,
    Unknown,
}
```

---

# 24. Unknown

Must be visible.

---

# 25. Plan Is Not Apply

Critical.

---

# 26. Plan Freshness

```rust
pub enum InfrastructurePlanFreshness {
    Current,
    SpecChanged,
    StateChanged,
    ProviderChanged,
    PolicyChanged,
    Unknown,
}
```

---

# 27. Revalidate Before Apply

Protected apply requires current plan.

---

# 28. No Applying Stale Plan

Critical.

---

# 29. InfrastructureChange

```rust
pub struct InfrastructureChange {
    pub resource: InfrastructureResourceAddress,
    pub kind: InfrastructureChangeKind,
    pub before: Option<InfrastructureResourceSummary>,
    pub after: Option<InfrastructureResourceSummary>,
}
```

---

# 30. Sensitive Fields

Redacted.

---

# 31. Plan Redaction

Provider output may contain secrets.

Must sanitize before persistence/UI.

---

# 32. Raw Plan

If retained, restricted/encrypted with lifecycle policy.

---

# 33. Infrastructure Risk

```rust
pub enum InfrastructureRisk {
    Low,
    Moderate,
    High,
    Critical,
    Unknown,
}
```

---

# 34. Risk Factors

Examples:

```text
resource deletion
database replacement
network exposure
IAM change
production
KMS/signing change
large scale-out
cross-region change
```

---

# 35. Policy

Central Part 11 evaluates risk/evidence.

---

# 36. IaC Engine

```rust
#[async_trait]
pub trait InfrastructureEngine {
    async fn validate(
        &self,
        spec: &InfrastructureSpec,
    ) -> Result<InfrastructureValidation, InfrastructureError>;

    async fn plan(
        &self,
        request: InfrastructurePlanRequest,
    ) -> Result<InfrastructurePlanResult, InfrastructureError>;

    async fn apply(
        &self,
        request: InfrastructureApplyRequest,
    ) -> Result<InfrastructureApplyResult, InfrastructureError>;

    async fn destroy(
        &self,
        request: InfrastructureDestroyRequest,
    ) -> Result<InfrastructureDestroyResult, InfrastructureError>;
}
```

---

# 37. Engine Is Adapter

Core does not depend on Terraform internals.

---

# 38. OpenTofu/Terraform

Run in restricted worker/sandbox.

---

# 39. Never Run Provider Plugins Inside Main Daemon

Critical.

---

# 40. Infrastructure Worker

Dedicated execution class.

---

# 41. Capabilities

```text
provider network
provider credentials
state backend access
IaC binary/plugin access
```

---

# 42. Build Runner Separation

Normal untrusted build runner should not automatically possess infrastructure authority.

---

# 43. Infrastructure Worker Trust

Higher trust class.

---

# 44. Provider Credentials

Short-lived cloud workload identity preferred.

---

# 45. No Long-Lived Static Cloud Keys by Default

---

# 46. State Ownership

Critical.

IaC engines often maintain state.

Forgeyard must define where authority lives.

---

# 47. InfrastructureStateRef

```rust
pub struct InfrastructureStateRef {
    pub environment: InfrastructureEnvironmentId,
    pub backend: InfrastructureStateBackend,
    pub generation: InfrastructureStateGeneration,
}
```

---

# 48. State Backend

```rust
pub enum InfrastructureStateBackend {
    ForgeyardManaged,
    ExternalManaged,
}
```

---

# 49. Forgeyard-Managed State

Encrypted CAS/object/DB-backed metadata.

---

# 50. External-Managed

Example:

```text
Terraform Cloud
S3 backend
Azure Blob
GCS
```

---

# 51. External State

Still references exact backend/workspace identity.

---

# 52. State Locking

Required.

---

# 53. InfrastructureLockId

```rust
pub struct InfrastructureLockId(Ulid);
```

---

# 54. Lock Scope

One mutable infrastructure environment.

---

# 55. No Concurrent Apply

Critical baseline.

---

# 56. Lock Is Not Enough

Provider-side duplicate effects still require reconciliation/idempotency.

---

# 57. Apply Request

```rust
pub struct InfrastructureApplyRequest {
    pub plan: InfrastructurePlanId,
    pub environment: InfrastructureEnvironmentId,
    pub actor: PrincipalId,
    pub authority_epoch: Option<AuthorityEpoch>,
}
```

---

# 58. Protected Apply

Requires:

```text
fresh plan
authz
policy
required approval
valid provider credentials
state lock
```

---

# 59. Apply External Effects

Can be ambiguous.

---

# 60. ApplyState

```rust
pub enum InfrastructureApplyState {
    Requested,
    Applying,
    Succeeded,
    Failed,
    Unknown,
    Reconciling,
}
```

---

# 61. Unknown Outcome

Inspect provider/state before retrying.

---

# 62. Never Blindly Re-Apply After Timeout

Critical.

---

# 63. Apply Evidence

```rust
pub struct InfrastructureApplyEvidence {
    pub plan: InfrastructurePlanId,
    pub state_before: InfrastructureStateRef,
    pub state_after: Option<InfrastructureStateRef>,
    pub result: InfrastructureApplyState,
}
```

---

# 64. Observed Infrastructure

Separate from IaC state.

---

# 65. ObservedResource

```rust
pub struct ObservedInfrastructureResource {
    pub address: InfrastructureResourceAddress,
    pub provider_id: ProviderResourceId,
    pub observed_digest: Digest,
    pub observed_at: Timestamp,
}
```

---

# 66. Drift

```rust
pub enum InfrastructureDrift {
    InSync,
    Changed,
    Missing,
    Unexpected,
    Unknown,
}
```

---

# 67. Drift Detection

Compare:

```text
desired spec
IaC state
provider observed state
```

---

# 68. Three-Way Model

Important.

---

# 69. State Alone Is Not Reality

Critical.

---

# 70. Provider Observation

Can detect out-of-band changes.

---

# 71. Drift Classification

```rust
pub enum DriftClass {
    Benign,
    Configuration,
    SecurityRelevant,
    Destructive,
    Unknown,
}
```

---

# 72. Security-Relevant Drift

Examples:

```text
public network exposure
IAM change
security group change
KMS change
secret access policy
```

---

# 73. Drift Action

```rust
pub enum DriftAction {
    Notify,
    PlanReconcile,
    RequireApproval,
    FreezeEnvironment,
    ManualInvestigation,
}
```

---

# 74. No Automatic Reconcile by Default for Production

Critical.

---

# 75. Auto-Reconcile

Can be allowed for narrowly safe resources.

---

# 76. Drift Suppression

Temporary/versioned/audited.

---

# 77. Preview Environment

First-class.

---

# 78. PreviewEnvironmentId

```rust
pub struct PreviewEnvironmentId(Ulid);
```

---

# 79. Preview Environment Subject

Usually exact:

```text
ChangeProposalRevisionId
SourceSnapshotId
```

---

# 80. Preview Spec

```rust
pub struct PreviewEnvironmentSpec {
    pub id: PreviewEnvironmentId,
    pub project: ProjectId,
    pub source: SourceSnapshotId,
    pub template: Option<TemplateRef>,
    pub ttl: Duration,
    pub isolation: PreviewIsolationClass,
}
```

---

# 81. Preview Isolation

```rust
pub enum PreviewIsolationClass {
    Namespace,
    DedicatedRuntime,
    DedicatedNetwork,
    DedicatedAccountProject,
}
```

---

# 82. Default

Cheapest safe isolation for project risk profile.

---

# 83. Preview Environment Naming

Human alias only.

Use immutable ID internally.

---

# 84. Preview Lifecycle

```rust
pub enum PreviewEnvironmentState {
    Requested,
    Planning,
    Provisioning,
    Ready,
    Deploying,
    Active,
    Expired,
    Destroying,
    Destroyed,
    Failed,
}
```

---

# 85. Preview Flow

```text
proposal revision
  ↓
plan infra
  ↓
provision
  ↓
deploy exact build/release candidate
  ↓
health
  ↓
publish preview URL
```

---

# 86. Preview URL

Derived output.

---

# 87. No Secret Exposure

Preview environment uses restricted secrets.

---

# 88. Production Secrets

Forbidden by default.

---

# 89. Preview Data

Synthetic/test data by default.

---

# 90. Production Clone

Requires explicit high-risk policy.

---

# 91. TTL

Mandatory default.

---

# 92. PreviewTtl

```rust
pub struct PreviewTtl {
    pub expires_at: Timestamp,
    pub grace: Duration,
}
```

---

# 93. TTL Expiry

Creates destroy intent.

---

# 94. TTL Does Not Mean Blind Delete

Critical.

---

# 95. Destroy Ownership Proof

Forgeyard must prove resources belong to preview environment.

---

# 96. Resource Ownership Tag

```text
forgeyard.preview_id
forgeyard.environment_id
forgeyard.tenant_id
```

---

# 97. Tags Are Supporting Evidence

Not sole authority.

---

# 98. State/metadata also required.

---

# 99. Unknown Resource

Never delete automatically.

---

# 100. Preview Destroy

```text
mark Expired
  ↓
stop deployment traffic
  ↓
revoke temporary credentials
  ↓
plan destroy
  ↓
verify owned resources
  ↓
destroy
  ↓
reconcile leftovers
```

---

# 101. Active Investigation Pin

Can extend TTL.

---

# 102. Pin Expiry

Required.

---

# 103. Change Proposal Close/Merge

Can trigger destroy.

---

# 104. Merge

Production deployment remains separate.

---

# 105. No Preview-to-Production Mutable Promotion

Critical.

Production should provision/apply from its own exact protected plan.

---

# 106. Environment Templates

Part 42.

Example golden path:

```text
web preview environment
service staging environment
standard production cluster
```

---

# 107. Template Expansion

Produces canonical InfrastructureSpec.

---

# 108. No Hidden Template Apply

---

# 109. Infrastructure Promotion

Could promote spec/config, not mutable resources.

---

# 110. Example

```text
staging InfrastructureSpec
  ↓
reviewed parameter changes
  ↓
production InfrastructureSpec
```

---

# 111. Production Plan

Recomputed against production state.

---

# 112. Never Reuse Staging Plan for Production

Critical.

---

# 113. Environment Parameterization

Typed.

Examples:

```text
region
instance class
replica count
network class
domain
```

---

# 114. SecretRef

For secret-backed values.

---

# 115. Policy Constraints

Production parameter range.

---

# 116. Cost Estimation

Part 45 integration.

---

# 117. InfrastructureCostEstimate

```rust
pub struct InfrastructureCostEstimate {
    pub plan: InfrastructurePlanId,
    pub monthly_estimate: Option<Money>,
    pub confidence: AttributionConfidence,
}
```

---

# 118. Estimate

Advisory.

---

# 119. Cost Increase

Policy can require approval.

---

# 120. Example

```text
monthly +25%
absolute +$500
```

configured externally.

---

# 121. No Cost Override of Security

---

# 122. Preview Cost Guardrail

Examples:

```text
max TTL
max instance class
max resources
max estimated monthly/hourly cost
```

---

# 123. Quotas

Part 27.

---

# 124. Tenant Preview Limit

```text
max active previews
max aggregate CPU
max cloud spend estimate
```

---

# 125. Budget

Part 45.

---

# 126. Budget Exceeded

Can prevent new optional previews.

---

# 127. Existing Production

Never destroyed due budget.

---

# 128. Environment Ownership

Part 49 catalog can show owner.

---

# 129. Ownership Does Not Grant Apply Permission

Existing invariant.

---

# 130. Infrastructure Policy

Can constrain:

```text
providers
regions
resource types
network exposure
IAM
encryption
instance size
public IP
database class
```

---

# 131. Policy Evaluation

On normalized plan changes.

---

# 132. Policy Input

Not raw provider text only.

---

# 133. Normalized Resource Change

Provider adapter maps to common security/cost/resource facts.

---

# 134. Provider-Specific Fields

Can remain extension data.

---

# 135. Unknown Provider Semantics

Policy can fail closed.

---

# 136. Plan Review

Dioxus UI should show:

```text
create 5
update 2
replace 1
delete 1
risk High
cost +12%
```

---

# 137. Resource Diff

Expandable.

---

# 138. Sensitive Data

Redacted.

---

# 139. Approval Binding

Approval binds exact:

```text
InfrastructurePlanId
PolicyDigest
```

---

# 140. State Changes After Approval

Plan stale.

---

# 141. Re-plan and re-approve if required.

---

# 142. Apply Worker

Dedicated app/service optional:

```text
forgeyard-infrastructure-worker
```

---

# 143. Least Privilege

Worker gets provider credentials for exact environment/account scope.

---

# 144. No General Admin Credentials

---

# 145. Workload Identity Federation

Preferred.

---

# 146. Kubernetes

Use:

```text
ServiceAccount
namespace-scoped RBAC
short-lived token
```

where possible.

---

# 147. Terraform/OpenTofu Provider Plugins

Untrusted-ish third-party code.

---

# 148. Run in sandboxed infrastructure worker.

---

# 149. Provider Plugin Supply Chain

Part 36 dependency governance.

---

# 150. Version Pinning

Exact provider versions.

---

# 151. Provider Lockfile

Preserve ecosystem-native lock.

---

# 152. No "latest" provider.

---

# 153. Plugin Download

Resolve/fetch stage.

---

# 154. Apply Network

Allowed only to provider endpoints/required APIs where enforceable.

---

# 155. State Secrets

Terraform state can contain secrets.

---

# 156. High Sensitivity

Critical.

---

# 157. State Encryption

Mandatory for Forgeyard-managed state.

---

# 158. State Access

Restricted.

---

# 159. State Never Stored in Normal CAS Namespace Unclassified

---

# 160. Sensitive State Class

```rust
pub enum InfrastructureStateSensitivity {
    Internal,
    Confidential,
    Restricted,
}
```

---

# 161. State Redaction

UI never exposes raw secret values.

---

# 162. State Backup

Part 25.

---

# 163. State Deletion

Part 46.

---

# 164. State Lock Recovery

If worker crashes:

```text
inspect lock
inspect provider
inspect backend
recover/release only if safe
```

---

# 165. No Force Unlock Blindly

Critical.

---

# 166. Manual Force Unlock

High privilege/audited.

---

# 167. Provider Operation Idempotency

Use provider request token where possible.

---

# 168. Ambiguous Create

Inspect before retry.

---

# 169. Ambiguous Delete

Inspect before retry.

---

# 170. DestroyRequestId

```rust
pub struct InfrastructureDestroyRequestId(Ulid);
```

---

# 171. Destroy Modes

```rust
pub enum DestroyMode {
    PreviewExpiry,
    Manual,
    ProjectDeletion,
    EnvironmentRetirement,
    Emergency,
}
```

---

# 172. Destroy Preconditions

```text
authz
current state
ownership proof
holds
active deployment check
backup/export policy
```

---

# 173. Production Destroy

Highest-risk.

---

# 174. Require typed confirmation/approval.

---

# 175. Environment Retirement

Separate from destroy.

---

# 176. Retired Environment

May remain read-only/history.

---

# 177. Disaster Recovery Infrastructure

Special.

---

# 178. Destroying DR resources

Needs policy awareness.

---

# 179. Dependency Graph

Infrastructure resources may depend on each other.

---

# 180. Destroy Order

Engine/provider plan.

---

# 181. Forgeyard Validates High-Level Constraints

---

# 182. Database Destruction

Can require backup evidence.

---

# 183. Data Store Replacement

Can require migration plan.

---

# 184. Network Destruction

Can affect multiple services.

---

# 185. Component Catalog

Part 49 blast radius.

---

# 186. Change Proposal Integration

Infrastructure change proposal can show:

```text
plan diff
cost diff
security impact
preview environment
```

---

# 187. PR Plan

Side-effect free.

---

# 188. Fork PR

No privileged provider credentials by default.

---

# 189. Preview From Fork

Highly restricted or disabled.

---

# 190. Trigger Integration

Part 44:

```text
proposal opened/updated -> plan/preview
proposal closed -> destroy preview
scheduled drift scan -> inspect
manual apply -> protected dispatch
```

---

# 191. Deployment Integration

Infrastructure must be Ready before software deployment if dependency.

---

# 192. Dependency

```text
InfrastructureEnvironmentReady
  ↓
Deployment allowed
```

---

# 193. But infrastructure apply and app deployment are separate state machines.

---

# 194. Release Integration

Production apply may be part of release/deploy workflow but remains exact independent evidence.

---

# 195. No Build-Time Infrastructure Side Effect

Build stage should remain hermetic where intended.

---

# 196. Preview Artifact

Deploy exact CI-built artifact.

---

# 197. No Rebuild Inside Preview Provisioner

Critical.

---

# 198. Drift Scan

Scheduled.

---

# 199. DriftScanId

```rust
pub struct DriftScanId(Ulid);
```

---

# 200. Drift Scan Flow

```text
load desired spec
  ↓
read IaC state
  ↓
observe provider
  ↓
classify differences
  ↓
produce drift report
```

---

# 201. Drift Report

```rust
pub struct InfrastructureDriftReport {
    pub id: DriftScanId,
    pub environment: InfrastructureEnvironmentId,
    pub findings: Vec<InfrastructureDriftFinding>,
}
```

---

# 202. Drift Finding

Contains:

```text
resource
class
before/observed
security impact
suggested action
```

---

# 203. No Automatic Production Apply From Drift Detection Baseline

---

# 204. Security Drift

Part 40 incident integration.

---

# 205. Unauthorized IAM Drift

Can trigger high-priority alert.

---

# 206. Drift Ignore Rules

Explicit, narrow, versioned.

---

# 207. Never broad wildcard ignore by default.

---

# 208. Reconciliation

Desired state controllers can retry safe convergent operations.

---

# 209. Reconciliation vs Apply

Apply is explicit change execution.

Reconciliation ensures state machine completion/observation.

---

# 210. No perpetual hidden Terraform apply loop baseline.

---

# 211. Kubernetes Exception

Kubernetes itself is declarative controller system.

Forgeyard can manage desired manifests and observe convergence.

---

# 212. Kubernetes Apply

Prefer server-side apply where appropriate.

---

# 213. Field Ownership

Explicit manager identity.

---

# 214. Avoid Clobbering Other Controllers

Critical.

---

# 215. Kubernetes Namespace Preview

Good baseline preview backend.

---

# 216. Stronger Isolation

Dedicated cluster/account/project where risk requires.

---

# 217. Preview Database

Use isolated DB/schema based on policy.

---

# 218. Data Sanitization

No production PII by default.

---

# 219. Preview DNS

Unique.

---

# 220. Preview TLS

Automated certificates if allowed.

---

# 221. Preview Ingress

Auth optional/policy.

---

# 222. Private Preview

Default for sensitive projects.

---

# 223. Public Preview

Explicit.

---

# 224. Preview Access Control

Can bind Change Proposal participants/team.

---

# 225. Preview Expiry Notification

Before destroy.

---

# 226. Extend TTL

Permission/policy.

---

# 227. Permanent Preview

Forbidden baseline.

---

# 228. Orphan Preview Detection

No active proposal/owner/TTL.

---

# 229. Orphan Cleanup

Plan + ownership proof.

---

# 230. Environment Lease

Optional.

```rust
pub struct EnvironmentLease {
    pub environment: InfrastructureEnvironmentId,
    pub expires_at: Timestamp,
}
```

---

# 231. Preview Lease

Typical.

---

# 232. Shared Test Environment

May use reservations/concurrency.

---

# 233. Environment Concurrency

Avoid simultaneous destructive applies.

---

# 234. Infrastructure Mutation Queue

Per environment.

---

# 235. Queue Policy

```text
serial
cancel queued stale plans
```

---

# 236. Running Apply

Do not arbitrarily cancel unless engine supports safe interruption.

---

# 237. Cancel Semantics

Explicit.

---

# 238. Apply Timeout

Timeout does not mean rollback.

---

# 239. Unknown External State

Reconcile.

---

# 240. Rollback

Infrastructure rollback is not generic.

---

# 241. Why

Some changes are irreversible:

```text
database deletion
data migration
IAM propagation
resource replacement
```

---

# 242. InfrastructureRollbackClass

```rust
pub enum InfrastructureRollbackClass {
    ReapplyPreviousSpec,
    ProviderRollbackSupported,
    ManualRecoveryRequired,
    Irreversible,
    Unknown,
}
```

---

# 243. UI Must Be Honest

No generic "Rollback" button.

---

# 244. Previous Spec

Can be re-planned, not blindly applied as old plan.

---

# 245. Recovery

New plan against current observed state.

---

# 246. Environment Clone

Useful.

---

# 247. Clone Spec

Not live resource copy unless explicit.

---

# 248. Disaster Recovery

Infrastructure definitions should be reproducible.

---

# 249. DR Environment

Can be pre-provisioned or cold.

---

# 250. Recovery Drill

Part 50 resilience tests.

---

# 251. Federation

Part 51 authority domain per infrastructure environment.

---

# 252. Cross-Region Apply

Only authority site may orchestrate mutation.

---

# 253. Provider Resource Region

Residency policy.

---

# 254. Failover

Infrastructure authority transfer uses Federation epoch.

---

# 255. No Two Regions Apply Same Environment Concurrently

Critical.

---

# 256. Air-Gap

Planning can work if:

```text
provider schemas/plugins
state snapshot
configuration
```

available.

---

# 257. Apply in Air-Gap

Only to reachable local/on-prem providers.

---

# 258. External Cloud Apply

Impossible without connectivity; fail explicitly.

---

# 259. Offline Plan Bundle

Can export for review.

---

# 260. Infrastructure Evidence

Store:

```text
plan digest
apply result
state generations
provider versions
policy decision
approval
drift reports
```

---

# 261. Supply Chain

Infrastructure provenance can be an evidence type.

---

# 262. InfrastructurePlanProvenance

```rust
pub struct InfrastructurePlanProvenance {
    pub source: SourceSnapshotId,
    pub spec: InfrastructureSpecId,
    pub engine: InfrastructureEngineVersion,
    pub providers: Vec<ProviderVersion>,
}
```

---

# 263. Attestation

Optional signed plan/apply evidence.

---

# 264. Audit

Audit:

```text
production plan approval
apply
destroy
force unlock
provider credential change
drift ignore
preview TTL extension
```

---

# 265. Routine plan

Operational evidence.

---

# 266. Notification

Examples:

```text
production plan awaiting approval
drift detected
preview expiring
destroy failed
state lock stuck
provider auth failure
cost increase
```

---

# 267. API

Potential:

```text
GET  /v1/infrastructure/environments
POST /v1/infrastructure/plan
POST /v1/infrastructure/apply
POST /v1/infrastructure/destroy
GET  /v1/infrastructure/drift
GET  /v1/previews
POST /v1/previews
```

---

# 268. Permissions

```text
infrastructure.read
infrastructure.plan
infrastructure.apply
infrastructure.destroy
infrastructure.state.read
infrastructure.force_unlock
preview.create
preview.extend
preview.destroy
```

---

# 269. Production Apply Permission

Separate high privilege.

---

# 270. Force Unlock

Highest privilege.

---

# 271. Dioxus UI

Pages:

```text
Infrastructure
Plans
Environments
Drift
Preview Environments
State
```

---

# 272. Infrastructure Overview

Shows:

```text
environment
provider
last apply
drift
cost estimate
health
```

---

# 273. Plan Detail

Shows:

```text
resource counts
risk
cost
security-impacting changes
approval status
```

---

# 274. Preview Page

Shows:

```text
proposal
URL
source revision
artifact
TTL
cost
status
```

---

# 275. State UI

Never raw-secret dump.

---

# 276. CLI

```text
forgeyard infra validate
forgeyard infra plan
forgeyard infra apply
forgeyard infra drift
forgeyard infra destroy
forgeyard infra status
forgeyard preview create
forgeyard preview list
forgeyard preview extend
forgeyard preview destroy
forgeyard infra doctor
```

---

# 277. Machine Output

JSON/RON.

---

# 278. Observability Metrics

```text
infrastructure_plans_total
infrastructure_apply_total
infrastructure_apply_failures_total
infrastructure_drift_findings_total
preview_environments_active
preview_destroy_failures_total
infrastructure_state_lock_age_seconds
```

---

# 279. Labels

Low-cardinality:

```text
environment_kind
provider_kind
result
drift_class
```

---

# 280. Tracing

```text
infra.validate
infra.plan
infra.policy
infra.apply
infra.observe
infra.drift
infra.destroy
preview.provision
preview.destroy
```

---

# 281. Health

Checks:

```text
state backend
provider auth
stuck locks
drift scan freshness
preview cleanup backlog
```

---

# 282. Doctor

```text
forgeyard infra doctor
```

Checks:

```text
provider/plugin pinning
state encryption
state backend access
stale locks
unowned resources
drift backlog
preview TTL violations
```

---

# 283. Search/Catalog

Part 49 catalog can show:

```text
environment infrastructure owner
provider
region
linked components
preview environments
```

---

# 284. Search

Part 31 indexes non-sensitive metadata.

---

# 285. Data Lifecycle

Part 46 governs:

```text
plans
state snapshots
drift evidence
preview logs
destroy evidence
```

---

# 286. Sensitive State

Longer/stricter rules.

---

# 287. Cost

Part 45 actual infrastructure cost may come from provider.

---

# 288. Plan Estimate vs Actual

Separate.

---

# 289. Reliability

Part 50 can track:

```text
plan latency
apply success
drift detection freshness
preview provisioning SLO
```

---

# 290. Security

Part 40 threat model includes infrastructure workers/provider credentials/state leakage.

---

# 291. Threats

```text
malicious IaC
provider plugin compromise
state secret exposure
overprivileged cloud credential
resource ownership spoofing
destroy attack
public network exposure
```

---

# 292. IaC Source Trust

Fork/untrusted source cannot obtain privileged apply.

---

# 293. Plan on Untrusted Source

Allowed in isolated environment with no provider mutation credentials if provider schema resolution permits.

---

# 294. Apply

Requires trusted source/policy.

---

# 295. Plan-Time Provider Calls

Some IaC engines perform reads.

---

# 296. Plan Credentials

Read-only provider credentials where possible.

---

# 297. Apply Credentials

Write-scoped.

---

# 298. Separate Plan/Apply Credentials

Recommended.

---

# 299. State Supply Chain

State backend integrity matters.

---

# 300. State Tamper

Detected through version/locking/checksums where possible.

---

# 301. State Snapshot Id

```rust
pub struct InfrastructureStateSnapshotId(Digest);
```

---

# 302. State History

Retained according policy.

---

# 303. Restore State

High-risk/manual plan.

---

# 304. Do Not Restore Old State Blindly

Provider reality may differ.

---

# 305. Import Existing Infrastructure

Need adoption workflow.

---

# 306. InfrastructureImportId

```rust
pub struct InfrastructureImportId(Ulid);
```

---

# 307. Import Flow

```text
discover resource
  ↓
prove ownership/permission
  ↓
generate/import state mapping
  ↓
plan no-op expected
  ↓
review
  ↓
adopt
```

---

# 308. Imported Resource

Not automatically safe to destroy.

---

# 309. DestructionProtection

```rust
pub enum DestructionProtection {
    None,
    Protected,
    RequireApproval,
    ExternalManaged,
}
```

---

# 310. Existing Critical DB

Mark Protected.

---

# 311. ExternalManaged

Forgeyard observes but does not mutate.

---

# 312. Hybrid Environment

Some resources Forgeyard-managed, some external.

---

# 313. ResourceAuthority

```rust
pub enum ResourceAuthority {
    ForgeyardManaged,
    ExternalManaged,
    ObservedOnly,
}
```

---

# 314. No Destroy of ExternalManaged/ObservedOnly

Critical.

---

# 315. Brownfield Adoption

Important enterprise workflow.

---

# 316. Migration

Part 47 can import existing CI IaC workflows into first-class infrastructure orchestration.

---

# 317. Legacy Terraform Pipeline

Migration result:

```text
generic shell terraform apply
  ↓
forgeyard infrastructure plan/apply subsystem
```

---

# 318. Policy Improvement

Explicit.

---

# 319. Testkit

```text
forgeyard-infrastructure-testkit/src/
├── lib.rs
├── spec.rs
├── plan.rs
├── apply.rs
├── state.rs
├── drift.rs
├── preview.rs
├── destroy.rs
└── assertions.rs
```

---

# 320. Unit Tests

Plan identity determinism.

---

# 321. Stale Plan Test

State change invalidates plan.

---

# 322. Approval Binding Test

Approval applies exact plan only.

---

# 323. Secret Redaction Test

No secret in UI/report/log.

---

# 324. Provider Timeout Test

Unknown -> inspect/reconcile.

---

# 325. Concurrent Apply Test

Second apply blocked.

---

# 326. Force Unlock Test

High privilege/audit.

---

# 327. Drift Test

Out-of-band change detected.

---

# 328. Security Drift Test

IAM/public exposure high severity.

---

# 329. Preview TTL Test

Expired environment gets destroy intent.

---

# 330. Ownership Test

Unknown resources never auto-deleted.

---

# 331. Fork Preview Test

Production secret/provider access denied.

---

# 332. Preview Data Test

Production data not used by default.

---

# 333. Production Destroy Test

Approval/protection required.

---

# 334. Database Replacement Test

Risk classification high/critical.

---

# 335. State Secret Test

Encrypted/restricted.

---

# 336. Terraform Plugin Test

Exact provider version pinned.

---

# 337. Kubernetes Field Ownership Test

Other controller fields not clobbered.

---

# 338. Federation Test

Two sites cannot apply same environment.

---

# 339. Cost Guardrail Test

Optional preview blocked over limit.

---

# 340. DR Test

State/evidence restored, provider reconciled.

---

# 341. Imported Resource Test

No-op plan required before adoption.

---

# 342. ExternalManaged Test

Destroy rejected.

---

# 343. Fuzzing

Fuzz:

```text
normalized plan parser
provider adapter metadata
IaC manifest inputs
state metadata
```

---

# 344. Scale Test

Thousands of previews/environments.

---

# 345. Chaos Tests

```text
provider outage
state backend outage
worker crash during apply
network partition
credential expiry
```

---

# 346. Implementation Phase 1 — Infrastructure Model/Plan

Provider-neutral types.

---

# 347. Phase 2 — OpenTofu Adapter

Primary IaC engine.

---

# 348. Phase 3 — Apply/State Locking

Controlled mutation.

---

# 349. Phase 4 — Kubernetes Adapter

Preview environments.

---

# 350. Phase 5 — Drift

Observation/reconciliation.

---

# 351. Phase 6 — Preview Lifecycle

TTL/destroy.

---

# 352. Phase 7 — Policy/Cost/Security Plan Facts

Governance.

---

# 353. Phase 8 — Brownfield Import

Enterprise adoption.

---

# 354. Phase 9 — Federation

Multi-region authority.

---

# 355. Phase 10 — UI/CLI/Doctor

Operations.

---

# 356. Phase 11 — Additional Cloud Native Adapters

Selective.

---

# 357. Phase 12 — Scale/Chaos/Fuzz/DR Hardening

Production readiness.

---

# 358. Acceptance Tests

1. Infrastructure mutation is first-class, not hidden arbitrary shell side effect.
2. IaC sources bind exact SourceSnapshotId/spec identity.
3. InfrastructurePlanId is immutable/content-derived.
4. Plan and apply are separate operations.
5. Protected apply requires a fresh exact plan.
6. Approval binds exact plan/policy identity.
7. State/provider changes can make a plan stale.
8. IaC engines/provider plugins do not execute in main daemon.
9. Infrastructure workers use dedicated least-privilege credentials.
10. Build runners do not automatically obtain infrastructure authority.
11. Provider credentials are short-lived/federated where possible.
12. Sensitive IaC state is encrypted/restricted.
13. Raw state/plan secrets never leak to normal UI/logs.
14. Concurrent apply to same environment is blocked.
15. Provider timeout becomes Unknown and is inspected before retry.
16. Drift compares desired spec, IaC state, and provider-observed reality.
17. Production drift is not auto-reconciled by default.
18. Security-relevant drift is explicitly classified.
19. Preview environments bind exact ChangeProposalRevision/SourceSnapshot.
20. Preview deployments use already-built exact artifacts, not rebuilds.
21. Preview environments have bounded TTL by default.
22. TTL expiration creates governed destroy intent, not blind deletion.
23. Unknown/unowned resources are never automatically destroyed.
24. Production secrets/data are not available to previews by default.
25. Preview cost/quota limits can block optional creation.
26. Production infrastructure cannot be destroyed due to budget pressure.
27. Staging plans are never reused as production plans.
28. Previous infrastructure specs are re-planned against current state for recovery.
29. Generic rollback is never promised where infrastructure change is irreversible.
30. ExternalManaged/ObservedOnly resources cannot be destroyed by Forgeyard.
31. Brownfield import requires explicit adoption and expected no-op/understood plan.
32. Kubernetes field ownership avoids clobbering unrelated controllers.
33. Federation guarantees one authority site per infrastructure environment.
34. Standalone/distributed share infrastructure semantics.
35. Forgeyard dogfoods the subsystem for its own preview/test/deployment infrastructure where practical.

---

# 359. Production Readiness Gates

Do not call infrastructure orchestration production-ready until:

```text
plan/apply separation is enforced
plan freshness and approval binding work
state encryption/locking is stable
provider timeout reconciliation is safe
preview destroy ownership proof passes
production destruction protections work
drift detection is reliable
fork/untrusted-source credential isolation passes
federation single-authority tests pass
chaos/DR/fuzz tests pass
```

---

# 360. Architectural Invariants

1. infrastructure mutation is explicit first-class state;
2. generic build jobs are not implicit infrastructure authority;
3. plan and apply are separate;
4. plans are immutable and freshness-checked;
5. approvals bind exact plan;
6. state is sensitive and governed;
7. concurrent apply to one environment is forbidden by baseline;
8. provider effects are ambiguous/reconciled;
9. IaC plugins run outside main daemon;
10. provider credentials are least privilege;
11. plan/apply credentials can differ;
12. drift uses desired/state/observed reality;
13. production drift is not blindly auto-fixed;
14. preview environments are exact-source bound;
15. preview deploys exact existing artifact;
16. preview TTL does not authorize blind delete;
17. destroy requires ownership proof;
18. unknown resources are never auto-deleted;
19. production secrets/data are excluded from preview by default;
20. cost only governs optional safe choices;
21. staging plan cannot become production plan;
22. rollback capability is explicit/honest;
23. external-managed resources remain outside Forgeyard mutation authority;
24. infrastructure state backups do not replace provider reconciliation;
25. federation gives each environment one mutation authority;
26. audit records protected apply/destroy/force-unlock;
27. lifecycle governs plan/state evidence;
28. standalone/distributed share semantics;
29. infrastructure adapters do not become shadow policy engines;
30. Forgeyard dogfoods its own infrastructure subsystem.

---

# 361. Final Target Architecture

```text
                 Desired Infrastructure
                          │
                          ▼
                  InfrastructureSpec
                          │
                          ▼
                  Validate / Plan
                          │
                          ▼
                InfrastructurePlanId
                          │
                  Policy / Approval
                          │
                          ▼
                         Apply
                          │
                          ▼
                Provider Observed State
                          │
                          ▼
                    Drift Detection
                          │
                          ▼
                  Reconcile / Review
```

Preview environments:

```text
ChangeProposalRevision
        ↓
exact SourceSnapshotId
        ↓
Preview InfrastructureSpec
        ↓
plan + policy
        ↓
provision
        ↓
deploy exact artifact
        ↓
review/test
        ↓
TTL / proposal close
        ↓
safe destroy
```

The key guarantee is:

> **Forgeyard can provision and manage infrastructure without hiding provider mutation inside opaque CI scripts. Every protected change has an exact desired spec, immutable plan, policy/approval decision, controlled apply, observed state, and drift history; preview environments remain cheap and disposable without ever making destruction unsafe or ambiguous.**

---

# 362. Extended Architecture Sequence

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
51 Multi-Region Federation / Edge Sites / Disconnected Operation / Cross-Site Replication
52 Artifact Registry / Package Repository / OCI Distribution / Internal Software Distribution
53 Infrastructure-as-Code / Environment Provisioning / Preview Environments / Drift Reconciliation
```
