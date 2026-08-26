# 11 — Forgeyard Policy, Authorization & Identity System Architecture

**Document type:** Core Security & Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Identity, principals, authentication boundaries, sessions/tokens, OIDC/SAML/SCIM integration, service/workload identity, runner identity, permission-based RBAC, authorization decisions, policy bundles, protected targets, separation of duties, break-glass, revocation, caching, auditing, and policy evaluation  
**Architecture style:** Identity-provider-neutral authentication, permission-based authorization, deterministic policy evaluation, explicit tenant/project scoping, immutable policy snapshots for critical workflows, and deny-by-default protected actions  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on the core foundation, metadata store, events/reconciliation, Change Proposal, Run/Job, Scheduler, Runner, and transport architectures. It defines who/what may request actions and under which policy, while remaining separate from secret storage and execution isolation.

---

# 1. Purpose

Forgeyard needs one coherent security model for:

```text
humans
service accounts
runners
automation
bots
provider integrations
workloads
signing workers
device agents
```

and for actions such as:

```text
view project
modify pipeline
cancel run
approve change
submit integration
read artifact
use secret
sign artifact
deploy production
administer runner
change policy
```

The central rule is:

> **Authentication answers "who or what is this?" Authorization answers "may this principal perform this action on this resource under the current policy?"**

A second rule is:

> **Forgeyard authorizes permissions, not UI roles. Roles are only convenient permission bundles.**

A third rule is:

> **Security-critical decisions are bound to explicit identities, resources, immutable inputs, and policy digests; they are never inferred from mutable labels or provider-specific role names alone.**

---

# 2. Architectural Position

```text
External Identity Provider / Local Identity
                │
                ▼
          Authentication
                │
                ▼
          PrincipalContext
                │
                ▼
        Authorization Service
         ┌──────┼─────────┐
         ▼      ▼         ▼
       RBAC   Policy    Resource
               │
               ▼
        AuthorizationDecision
               │
               ▼
           Domain Service
               │
               ▼
             Audit
```

---

# 3. Goals

The subsystem MUST:

1. define stable `PrincipalId`;
2. support human identities;
3. support service identities;
4. support runner identities;
5. support workload identities;
6. support local standalone authentication;
7. support OIDC;
8. support SAML through an enterprise adapter if required;
9. support SCIM provisioning;
10. support API tokens;
11. support short-lived sessions;
12. support permission-based RBAC;
13. support tenant/project/resource scoping;
14. support policy evaluation;
15. support immutable policy digests;
16. support protected targets;
17. support separation of duties;
18. support break-glass;
19. support approval requirements;
20. support revocation;
21. support authorization caching safely;
22. support audit coupling;
23. support runner trust classes;
24. support workload identity;
25. support provider integrations;
26. support policy exceptions;
27. support multi-tenancy;
28. support standalone mode without enterprise IdP;
29. support enterprise federation without coupling domain logic to provider;
30. remain deterministic and testable.

---

# 4. Non-Goals

This subsystem does not:

```text
store secret values
execute jobs
isolate workloads
implement CAS
schedule jobs
replace enterprise identity providers
```

It consumes/verifies identity and decides authority.

---

# 5. Workspace Structure

```text
crates/identity/
├── forgeyard-identity/
├── forgeyard-identity-model/
├── forgeyard-identity-local/
├── forgeyard-identity-oidc/
├── forgeyard-identity-saml/
├── forgeyard-identity-scim/
├── forgeyard-identity-token/
├── forgeyard-identity-session/
├── forgeyard-identity-runner/
├── forgeyard-identity-workload/
├── forgeyard-identity-provider/
├── forgeyard-identity-health/
└── forgeyard-identity-testkit/
```

Authorization:

```text
crates/authz/
├── forgeyard-authz/
├── forgeyard-authz-model/
├── forgeyard-authz-permission/
├── forgeyard-authz-role/
├── forgeyard-authz-scope/
├── forgeyard-authz-service/
├── forgeyard-authz-cache/
├── forgeyard-authz-breakglass/
├── forgeyard-authz-separation/
└── forgeyard-authz-testkit/
```

Policy:

```text
crates/policy/
├── forgeyard-policy/
├── forgeyard-policy-model/
├── forgeyard-policy-bundle/
├── forgeyard-policy-eval/
├── forgeyard-policy-store-api/
├── forgeyard-policy-exception/
├── forgeyard-policy-protected-target/
├── forgeyard-policy-compiler/
├── forgeyard-policy-diagnostic/
└── forgeyard-policy-testkit/
```

---

# 6. Principal

```rust
pub struct PrincipalId(Ulid);
```

Stable internal identity.

External IdP subject identifiers are bindings, not Forgeyard authority IDs.

---

# 7. Principal Kinds

```rust
pub enum PrincipalKind {
    Human,
    Service,
    Runner,
    Workload,
    Bot,
    System,
}
```

---

# 8. Principal Record

```rust
pub struct Principal {
    pub id: PrincipalId,
    pub kind: PrincipalKind,
    pub state: PrincipalState,
    pub display_name: Option<PrincipalDisplayName>,
    pub created_at: Timestamp,
}
```

---

# 9. Principal State

```rust
pub enum PrincipalState {
    Active,
    Suspended,
    Revoked,
    Deleted,
}
```

---

# 10. External Identity Binding

```rust
pub struct ExternalIdentityBinding {
    pub principal: PrincipalId,
    pub provider: IdentityProviderId,
    pub subject: ExternalSubjectId,
}
```

Unique:

```text
provider + subject
```

---

# 11. Never Use Email as Stable Identity

Email may change.

Use provider subject / internal `PrincipalId`.

---

# 12. Local Standalone Identity

Mode 1 may use:

```text
LocalOwner
```

or local OS-authenticated principal.

---

# 13. Standalone Security

Do not force OIDC for single-user offline mode.

But map local identity into same `PrincipalId` model.

---

# 14. Authentication Methods

```rust
pub enum AuthenticationMethod {
    Local,
    PasswordlessLocal,
    Oidc,
    Saml,
    ApiToken,
    ClientCertificate,
    WorkloadIdentity,
}
```

---

# 15. OIDC

Preferred human enterprise/cloud authentication mechanism.

Forgeyard acts as relying party.

---

# 16. OIDC Responsibilities

```text
discovery
authorization code flow
PKCE where appropriate
issuer validation
audience validation
nonce/state
JWKS rotation
subject mapping
```

---

# 17. OIDC Claims

Treat IdP claims as external attributes.

Map only explicitly configured claims into Forgeyard attributes/groups.

---

# 18. No Blind Group Trust

IdP group string does not automatically become Forgeyard admin.

Explicit mapping required.

---

# 19. SAML

Optional enterprise adapter.

Converts SAML assertion into normalized external identity context.

---

# 20. SCIM

Provisioning/synchronization:

```text
users
groups
activation/deactivation
```

SCIM is not login.

---

# 21. SCIM State

Provisioned identity can exist before first authentication.

---

# 22. Directory Groups

```rust
pub struct ExternalGroupBinding {
    pub provider: IdentityProviderId,
    pub external_group: ExternalGroupId,
    pub forgeyard_role: RoleId,
    pub scope: AuthorizationScope,
}
```

---

# 23. Session

Browser/UI session:

```rust
pub struct UserSession {
    pub id: SessionId,
    pub principal: PrincipalId,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
    pub authn_context: AuthenticationContext,
}
```

---

# 24. Session Security

```text
short-lived
HttpOnly
Secure
SameSite
CSRF protection
rotation
```

where cookie-based.

---

# 25. API Tokens

```rust
pub struct ApiTokenId(Ulid);
```

Token value stored only hashed/secret-managed.

---

# 26. API Token Scope

Token has explicit:

```text
permissions
resource scope
expiry
```

---

# 27. No Permanent Omnipotent Token by Default

Admin tokens require explicit creation/expiry policy.

---

# 28. Personal Access Token

Human-owned automation token.

---

# 29. Service Account

```rust
pub struct ServiceIdentity {
    pub principal: PrincipalId,
    pub owner: Option<PrincipalId>,
}
```

---

# 30. Workload Identity

Jobs can receive short-lived workload identity rather than long-lived secret.

---

# 31. Workload Identity Binding

```rust
pub struct WorkloadIdentityBinding {
    pub run: RunId,
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub principal: PrincipalId,
    pub expires_at: Timestamp,
}
```

---

# 32. Workload Identity Rule

Identity expires with attempt/job lifetime.

---

# 33. Runner Identity

Runner uses:

```text
RunnerId
+
principal/certificate binding
```

---

# 34. Runner Trust

```rust
pub enum RunnerTrustClass {
    GeneralUntrustedWorkload,
    InternalTrusted,
    SigningRestricted,
    Confidential,
}
```

---

# 35. Runner Cannot Self-Promote

Trust class comes from enrollment/admin policy, not capability self-report.

---

# 36. Signing Worker Identity

Separate principal kind/service role.

---

# 37. Device Agent Identity

Same strong client authentication model.

---

# 38. Authentication Context

```rust
pub struct AuthenticationContext {
    pub method: AuthenticationMethod,
    pub provider: Option<IdentityProviderId>,
    pub authenticated_at: Timestamp,
    pub assurance: AuthenticationAssurance,
}
```

---

# 39. Assurance Level

```rust
pub enum AuthenticationAssurance {
    Local,
    SingleFactor,
    MultiFactor,
    HardwareBacked,
}
```

---

# 40. Step-Up Authentication

Sensitive actions may require stronger recent authentication:

```text
change policy
break-glass
production deploy
create signing key binding
```

---

# 41. Permission Model

Permission strings/IDs are stable typed constants.

Examples:

```text
project.read
project.write
pipeline.edit
run.create
run.cancel
artifact.read
artifact.delete
change.review
change.approve
change.integrate
policy.read
policy.write
secret.use
secret.admin
runner.read
runner.admin
release.promote
deployment.create
deployment.production
audit.read
system.admin
```

---

# 42. Permission Type

```rust
pub struct PermissionId(&'static str);
```

or generated stable enum/newtype.

---

# 43. Role

Role is a permission bundle.

```rust
pub struct Role {
    pub id: RoleId,
    pub permissions: BTreeSet<PermissionId>,
}
```

---

# 44. Built-In Roles

Examples:

```text
Viewer
Developer
Maintainer
Reviewer
ReleaseManager
SecurityAdmin
OrganizationAdmin
SystemAdmin
```

---

# 45. Roles Are Not Domain Logic

Domain checks:

```text
permission
```

not:

```text
if role == Admin
```

---

# 46. Custom Roles

Enterprise can define custom permission bundles.

---

# 47. Scope

```rust
pub enum AuthorizationScope {
    System,
    Tenant(TenantId),
    Organization(OrganizationId),
    Project(ProjectId),
    Repository(RepositoryId),
    Environment(EnvironmentId),
}
```

---

# 48. Scope Hierarchy

Potential:

```text
System
  ↓
Tenant
  ↓
Organization
  ↓
Project
```

Repository/environment may be children of project.

---

# 49. Scope Inheritance

Role assignment at organization may inherit to projects unless policy disables.

---

# 50. Explicit Deny

Support if needed:

```rust
pub enum PermissionEffect {
    Allow,
    Deny,
}
```

Recommended:

```text
deny overrides allow
```

---

# 51. Assignment

```rust
pub struct RoleBinding {
    pub principal: PrincipalId,
    pub role: RoleId,
    pub scope: AuthorizationScope,
}
```

---

# 52. Group Assignment

Directory/internal group can hold role binding.

---

# 53. Authorization Request

```rust
pub struct AuthorizationRequest {
    pub principal: PrincipalId,
    pub action: PermissionId,
    pub resource: ResourceRef,
    pub context: AuthorizationContext,
}
```

---

# 54. Authorization Context

```rust
pub struct AuthorizationContext {
    pub tenant: TenantId,
    pub project: Option<ProjectId>,
    pub authn: AuthenticationContext,
    pub source_trust: Option<SourceTrust>,
    pub environment: Option<EnvironmentId>,
}
```

---

# 55. Decision

```rust
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reasons: Vec<AuthorizationReason>,
    pub policy_digest: PolicyDigest,
}
```

---

# 56. Deny by Default

If no explicit permission/policy grants:

```text
deny
```

---

# 57. Authorization Service

```rust
#[async_trait]
pub trait AuthorizationService {
    async fn authorize(
        &self,
        request: AuthorizationRequest,
    ) -> Result<AuthorizationDecision, AuthorizationError>;
}
```

---

# 58. Domain Service Pattern

```text
API
  ↓
authenticate
  ↓
authorize permission/resource
  ↓
domain command
```

---

# 59. UI Is Not Security Boundary

Hiding button is UX only.

Server authorizes every action.

---

# 60. Agent Authorization

Agents do not call generic human permission APIs.

They act using server-issued leases/workload-specific authority.

---

# 61. Policy vs RBAC

RBAC:

```text
who may attempt action
```

Policy:

```text
under what conditions may action/workflow proceed
```

---

# 62. Example

RBAC:

```text
Alice has change.integrate
```

Policy:

```text
integration requires 2 approvals + passing CI + protected target queue
```

Both must pass.

---

# 63. Policy Bundle

```rust
pub struct PolicyBundle {
    pub id: PolicyBundleId,
    pub version: PolicyVersion,
    pub rules: Vec<PolicyRule>,
}
```

---

# 64. Policy Digest

Canonical:

```rust
pub struct PolicyDigest(Digest);
```

Critical decisions record exact digest.

---

# 65. Policy Sources

```text
system
tenant
organization
project
repository
environment
protected target
```

---

# 66. Policy Precedence

Higher-level policy may set minimum protections.

Lower-level policy may tighten, not weaken protected constraints.

---

# 67. Policy Merge

Deterministic.

---

# 68. Effective Policy

```rust
pub struct EffectivePolicy {
    pub digest: PolicyDigest,
    pub rules: Vec<EffectivePolicyRule>,
}
```

---

# 69. Policy Input

```rust
pub struct PolicyInput {
    pub principal: PrincipalId,
    pub action: PolicyAction,
    pub resource: ResourceRef,
    pub source_snapshot: Option<SourceSnapshotId>,
    pub proposal_revision: Option<ProposalRevisionId>,
    pub run: Option<RunId>,
    pub environment: Option<EnvironmentId>,
    pub context: PolicyContext,
}
```

---

# 70. Policy Decision

```rust
pub enum PolicyDecision {
    Allow,
    Deny(Vec<PolicyViolation>),
    Require(Vec<PolicyRequirement>),
}
```

---

# 71. Requirements

Examples:

```text
approval
specific reviewer group
required check
reproducibility
signed artifact
trusted runner
MFA
queue-only integration
```

---

# 72. Policy Must Be Deterministic

Same immutable input + same policy digest -> same decision.

---

# 73. No Network in Core Policy Eval

Policy evaluator should be pure over supplied input.

External facts are resolved before evaluation and included explicitly.

---

# 74. Policy Language

Prefer typed Rust-native policy model initially.

RON for human policy config.

---

# 75. Avoid General Embedded Scripting Initially

Do not embed unrestricted Lua/JS/Rego-like interpreter unless need justifies complexity.

---

# 76. Policy Compiler

```text
RON policy
  ↓
parse
  ↓
validate
  ↓
normalize
  ↓
PolicyBundle
  ↓
digest
```

---

# 77. Policy Rule Example

Illustrative RON:

```ron
(
    protected_targets: [
        (
            pattern: "main",
            requirements: [
                Approvals(2),
                Check("ci"),
                QueueOnly,
            ],
        ),
    ],
)
```

---

# 78. Protected Target

```rust
pub struct ProtectedTargetPolicy {
    pub target: ProtectedTargetSelector,
    pub requirements: Vec<ProtectionRequirement>,
}
```

---

# 79. Protection Requirements

```rust
pub enum ProtectionRequirement {
    RequiredApprovals(u8),
    RequiredOwners,
    RequiredCheck(CheckKind),
    QueueOnly,
    SignedIntegration,
    NoDirectPush,
    SeparationOfDuties,
}
```

---

# 80. Direct Push

VCS/provider integration can detect and reject/report according to backend capability.

Forgeyard cannot magically block external VCS writes unless provider/backend enforcement exists.

---

# 81. Policy Honesty

Distinguish:

```text
Forgeyard-enforced
provider-enforced
advisory
```

---

# 82. Change Proposal Integration

Reviews/approvals bind:

```text
ProposalRevisionId
SourceSnapshotId
```

Policy decides whether evidence satisfies target requirements.

---

# 83. Approval Invalidation

If source revision changes, policy re-evaluates.

---

# 84. Separation of Duties

Examples:

```text
author != approver
approver != integrator
release creator != release approver
security approver distinct from code owner
```

---

# 85. Separation Model

```rust
pub struct SeparationRule {
    pub action_a: Responsibility,
    pub action_b: Responsibility,
    pub relation: SeparationRelation,
}
```

---

# 86. Break-Glass

Emergency privileged bypass.

---

# 87. Break-Glass Rule

Never a hidden "admin bypass."

Requires:

```text
explicit permission
strong auth
reason
scope
expiry
audit
```

---

# 88. Break-Glass Grant

```rust
pub struct BreakGlassGrant {
    pub id: BreakGlassId,
    pub principal: PrincipalId,
    pub scope: AuthorizationScope,
    pub permissions: BTreeSet<PermissionId>,
    pub reason: BreakGlassReason,
    pub expires_at: Timestamp,
}
```

---

# 89. Break-Glass Audit

Mandatory.

---

# 90. Break-Glass Notification

Enterprise policy may notify security/admin team.

---

# 91. Break-Glass Cannot Reveal Secret Automatically

Secret access remains separately scoped and audited.

---

# 92. Policy Exception

A policy violation can be explicitly excepted.

```rust
pub struct PolicyException {
    pub id: PolicyExceptionId,
    pub violation: PolicyViolationId,
    pub granted_by: PrincipalId,
    pub scope: ExceptionScope,
    pub expires_at: Option<Timestamp>,
}
```

---

# 93. Exception Does Not Erase Violation

Decision records:

```text
violation
+
exception
```

---

# 94. Exception Scope

Exact:

```text
proposal revision
run
release
target
time window
```

Avoid broad permanent bypass.

---

# 95. Source Trust

```rust
pub enum SourceTrust {
    TrustedInternal,
    ExternalContribution,
    Fork,
    Unknown,
}
```

---

# 96. Untrusted Proposal Policy

Restrict:

```text
production secrets
signing workers
deployment credentials
privileged networks
```

---

# 97. Secret Use Authorization

Pipeline contains `SecretRef`.

At runtime:

```text
principal/workload
+
secret
+
job/source trust
+
policy
```

must authorize.

---

# 98. Runner Trust Authorization

Scheduler filters trust capability.

Policy determines minimum trust for job.

---

# 99. Workload Permission

A job may get only:

```text
artifact upload
specific secret use
specific deployment API
```

not user’s full permissions.

---

# 100. Delegation

Human starts run.

Do not simply run job as human's unrestricted identity.

Issue constrained workload identity.

---

# 101. Delegation Chain

```text
Human Principal
  ↓
Run
  ↓
Workload Principal
  ↓
specific capabilities
```

---

# 102. Delegation Audit

Record initiating principal.

---

# 103. Service-to-Service Identity

Daemon internal components generally run within same trusted process.

External workers use certificates/tokens.

---

# 104. Provider Integration Identity

SCM provider app/webhook identity maps to service principal/provider binding.

---

# 105. Webhook Authentication

Signature verification identifies provider delivery source, not human actor.

Human actor from provider event remains external metadata.

---

# 106. Provider Actor Mapping

If external actor binding exists, map carefully.

Do not assume provider username equals Forgeyard identity.

---

# 107. Identity Linking

User can link external provider account to Forgeyard principal through verified flow.

---

# 108. Audit Actor

`ActorRef` may be:

```text
human principal
service principal
runner
system
```

---

# 109. System Actor

Do not create fake user `"system"`.

Use typed `SystemActor`.

---

# 110. Authorization Cache

High-volume permission checks may be cached.

---

# 111. Cache Key

```text
principal
permission
scope/resource class
role-binding version
policy digest
```

---

# 112. Cache TTL

Short.

---

# 113. Revocation

Security-sensitive revocation must invalidate quickly.

---

# 114. Authorization Epoch

Potential:

```rust
pub struct AuthorizationEpoch(u64);
```

Increment on role/policy changes.

Cache includes epoch.

---

# 115. Tenant Policy Epoch

Can avoid global cache flush.

---

# 116. Token Revocation

Store token state/hash.

---

# 117. Session Revocation

User logout/admin suspension invalidates session.

---

# 118. Principal Suspension

Immediately deny new actions.

---

# 119. Active Jobs After Suspension

Policy decides.

Usually:

```text
existing unprivileged build can continue
```

but privileged deployments/signing may be cancelled.

---

# 120. Runner Revocation

Close transport session; leases eventually cancel/lost.

---

# 121. Workload Identity Revocation

Expires quickly and can be explicitly revoked if run cancelled.

---

# 122. Policy Snapshot for Run

Run records:

```text
PolicyDigest
```

for immutable execution semantics.

---

# 123. Mid-Run Policy Change

Does not silently mutate already planned ordinary job semantics.

Security emergency policy may impose immediate deny/cancel through explicit override mechanism.

---

# 124. Security Override

Examples:

```text
revoke compromised secret
disable runner
block vulnerable toolchain
```

May override existing plans.

Must be explicit/audited.

---

# 125. Release Policy Snapshot

Release approval/promotion records exact policy digest.

---

# 126. Deployment Policy Snapshot

Same.

---

# 127. Change Proposal Policy Snapshot

Check/approval decisions bind policy digest.

---

# 128. Policy Re-evaluation

If policy changes before integration:

```text
re-evaluate
```

old evidence may still be valid but requirements may change.

---

# 129. Authorization Error Model

```rust
pub enum AuthorizationError {
    Unauthenticated,
    PrincipalRevoked,
    PermissionDenied,
    ScopeMismatch,
    StepUpRequired,
    PolicyDenied,
    PolicyEvaluationFailed,
    Internal,
}
```

---

# 130. User-Safe Denial

Explain:

```text
missing permission
required approval
MFA required
protected target
```

without leaking sensitive details.

---

# 131. Policy Diagnostics

```rust
pub struct PolicyDiagnostic {
    pub code: PolicyDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: BoundedString,
}
```

---

# 132. Explain Authorization

CLI/UI can show:

```text
principal
permission
scope
role bindings
policy requirements
decision
```

subject to permission to inspect policy.

---

# 133. `forgeyard authz explain`

Example:

```text
forgeyard authz explain --action change.integrate --proposal 42
```

---

# 134. Explainability

Critical for enterprise debugging.

---

# 135. Do Not Leak Hidden Security Policy

Some details may be redacted for non-admin users.

---

# 136. Identity Store

Metadata stores:

```text
principals
external bindings
sessions
token metadata
role bindings
groups
policy refs
```

---

# 137. Secret Token Values

Never plaintext in DB.

---

# 138. Token Hashing

Use strong keyed/hash mechanism appropriate for high-entropy tokens.

---

# 139. Session Cookie Secrets

Store securely.

---

# 140. OIDC Tokens

Avoid long-term storage of provider access tokens unless integration needs them.

---

# 141. Refresh Tokens

If stored, use secret provider/encrypted storage and minimal scopes.

---

# 142. Identity Provider Configuration

RON:

```ron
(
    providers: [
        (
            id: "company-oidc",
            kind: Oidc,
            issuer: "https://id.example.com",
            client_id: Secret("oidc/client-id"),
            client_secret: Secret("oidc/client-secret"),
        ),
    ],
)
```

---

# 143. IdP Secrets

SecretRef, not literal.

---

# 144. Local Bootstrap Admin

First standalone/server setup needs safe bootstrap.

---

# 145. Bootstrap Token

One-time short-lived local setup token or console command.

---

# 146. Bootstrap Completion

Disable bootstrap path after initial admin created.

---

# 147. No Default Password

Never ship static default credentials.

---

# 148. MFA

Forgeyard should rely on IdP MFA where federated.

Local admin can support additional step-up later.

---

# 149. MFA Claim

Validate assurance claim only from configured trusted IdP.

---

# 150. Reauthentication Age

Sensitive action may require auth within last N minutes.

---

# 151. Role Binding Versioning

Mutable bindings have `EntityVersion`.

---

# 152. Role Changes

Emit events:

```text
RoleBindingCreated
RoleBindingRevoked
```

and invalidate caches.

---

# 153. Policy Change Event

```text
PolicyBundleActivated
```

---

# 154. Audit Coupling

Security-sensitive changes require audit in same transaction where possible.

---

# 155. Audit Examples

```text
role change
policy change
break-glass
token creation/revocation
runner trust change
secret permission change
production deploy authorization
```

---

# 156. Approval Audit

Already covered by Change Proposal but integrates principal/policy.

---

# 157. Identity Deletion

Do not erase historical audit actor identity blindly.

Can pseudonymize display/profile while retaining stable internal audit ID.

---

# 158. SCIM Deprovision

Principal -> Suspended/Revoked.

---

# 159. Group Removal

Role bindings derived from group recalculate.

---

# 160. SCIM Reconciliation

Periodic sync can repair missed provisioning events.

---

# 161. OIDC JIT Provisioning

Optional:

```text
first valid login creates Principal
```

if policy permits.

---

# 162. Enterprise Recommendation

SCIM + OIDC:

```text
SCIM lifecycle
OIDC authentication
```

---

# 163. Group Mapping

Explicit configuration.

---

# 164. Name Collisions

External display names do not determine identity.

---

# 165. Multi-Tenant Federation

Different tenants may use different IdPs.

---

# 166. IdP Routing

Tenant/domain selection at login.

---

# 167. Tenant Discovery Security

Avoid leaking private tenant existence unnecessarily.

---

# 168. System Admin

System-scoped permission distinct from tenant admin.

---

# 169. Tenant Admin

Cannot cross tenant.

---

# 170. Project Admin

Cannot change system identity provider settings.

---

# 171. Resource Ownership

Ownership can inform policy but does not replace permission.

---

# 172. CODEOWNERS

Change ownership maps reviewer requirements, not general RBAC admin.

---

# 173. Protected Environment

```rust
pub struct EnvironmentPolicy {
    pub environment: EnvironmentId,
    pub required_permissions: BTreeSet<PermissionId>,
    pub approvals: Vec<ApprovalRequirement>,
}
```

---

# 174. Production Environment

Can require:

```text
deployment.production
MFA
release manager approval
signed artifact
```

---

# 175. Artifact Authorization

Artifact metadata has project/tenant scope.

CAS digest knowledge alone is insufficient.

---

# 176. Audit Authorization

`audit.read` can be separately scoped.

---

# 177. Secret Admin vs Use

Separate:

```text
secret.admin
secret.use
```

A developer can use secret without reading value.

---

# 178. Signing Permission

Separate:

```text
signing.request
signing.admin
```

---

# 179. Runner Admin

Separate:

```text
runner.read
runner.drain
runner.admin
runner.trust.manage
```

---

# 180. Principle of Least Privilege

Split permissions when actions have materially different risk.

---

# 181. Permission Registry

Stable centrally documented list.

---

# 182. Permission Evolution

Adding permission should not silently grant through broad wildcard unless explicitly intended.

---

# 183. Wildcards

Avoid overly broad:

```text
*
```

except system-admin internal role.

---

# 184. Custom Role Validation

Cannot include unknown permission.

---

# 185. System Admin Role

Explicitly high risk.

---

# 186. Policy Rule Types

Potential:

```rust
pub enum PolicyRule {
    RequirePermission,
    RequireApproval,
    RequireCheck,
    RequireOwnership,
    RequireMfa,
    RequireTrustedRunner,
    RequireReproducibility,
    DenySourceTrust,
    RestrictEnvironment,
    RestrictSecret,
    SeparationOfDuties,
}
```

---

# 187. Policy Evaluation Output

Include:

```text
matched rules
unmet requirements
violations
exceptions
```

---

# 188. Policy Engine Must Not Mutate

Pure decision only.

Services execute resulting requirements/workflows.

---

# 189. Policy Compiler Safety

Bound config size/rule count.

---

# 190. Policy Cycle

If rules reference groups/policies, detect cycles.

---

# 191. Policy Include

If supported:

```text
immutable/local config
```

No moving remote policy during evaluation.

---

# 192. Policy Testing

Provide CLI:

```text
forgeyard policy test
```

---

# 193. Policy Unit Cases

RON can include fixtures:

```text
input
expected decision
```

---

# 194. Policy Simulation

Before activation:

```text
show which projects/actions would change
```

---

# 195. Policy Activation

Versioned/atomic.

---

# 196. Policy Rollback

Activate prior known policy bundle.

---

# 197. Policy History

Retain:

```text
bundle digest
creator
activation time
```

---

# 198. Policy Diff

Semantic diff:

```text
new required approval
removed permission
new production restriction
```

---

# 199. Policy Change Review

High-assurance installations may require Change Proposal-like review of policy config itself.

---

# 200. Self-Hosting Policy

Forgeyard repository can protect:

```text
main
release tags
signing
```

using same policy engine.

---

# 201. API Authentication

Axum middleware extracts authenticated principal context.

---

# 202. Middleware Does Not Authorize Everything Globally

Route/service calls permission-specific authz.

---

# 203. CLI Authentication

Normal CLI uses browser/OIDC/device flow/token as supported.

---

# 204. Dioxus UI

Uses user session.

---

# 205. Agent Authentication

mTLS.

---

# 206. Provider Authentication

Webhook signature/app credential.

---

# 207. Internal System Tasks

Use `SystemActor` + explicit system capability, not spoofed admin user.

---

# 208. Authorization Context Injection

Application layer constructs context.

Domain service can require `Authorized<T>` wrapper if useful.

---

# 209. Capability Token Pattern

For some internal flows:

```rust
pub struct AuthorizedAction<T> {
    pub request: T,
    pub decision: AuthorizationDecisionRef,
}
```

Avoid passing raw user permission booleans.

---

# 210. Security-Critical Recheck

Some long workflows re-authorize at final action.

Example:

```text
deployment approval hours ago
```

before production action ensure principal/policy still valid if required.

---

# 211. Approval Evidence vs Permission

Approval can remain evidence even if approver later leaves organization depending policy.

Policy defines.

---

# 212. Revoked Malicious Approval

Security admin may invalidate explicitly.

---

# 213. Policy Exception Expiry

Durable timer/reconciler expires.

---

# 214. Token Expiry

Validated on each request.

---

# 215. Session Expiry

Same.

---

# 216. API Rate Limits

Can be identity-scoped.

Detailed rate limiter elsewhere.

---

# 217. Audit IP/User Agent

Optional diagnostic metadata.

Not identity authority.

---

# 218. Privacy

Minimize stored identity profile data.

---

# 219. Display Name

Can be synced from IdP but historical audit should not depend on current display value.

---

# 220. Email

Contact metadata, not authorization identity.

---

# 221. Authorization Metrics

```text
authn_success
authn_failure
authz_allow
authz_deny
authz_stepup
policy_denials
breakglass_active
```

---

# 222. IdP Metrics

```text
oidc_login_latency
jwks_refresh_failure
scim_sync_failure
```

---

# 223. Cache Metrics

```text
authz_cache_hit
authz_cache_miss
authz_cache_invalidation
```

---

# 224. High Cardinality

No PrincipalId in metrics labels.

---

# 225. Tracing

Spans:

```text
authn.oidc
authn.token
authz.evaluate
policy.evaluate
policy.compile
identity.scim.sync
```

---

# 226. Audit Logs

Do not log token values/claims wholesale.

---

# 227. Health

Checks:

```text
IdP discovery
JWKS validity
session store
policy store
SCIM status
```

---

# 228. IdP Outage

Existing valid sessions may continue according to policy until expiry.

New login may fail.

---

# 229. Policy Store Outage

Critical authorization should fail closed if decision cannot be determined.

---

# 230. Authz Cache During Outage

Only bounded/still-valid cached decisions according to risk.

---

# 231. Fail Closed

Protected write actions default deny on uncertainty.

---

# 232. Read-Only Degraded Mode

May allow some cached/public reads.

---

# 233. Doctor

```text
forgeyard identity doctor
forgeyard authz doctor
forgeyard policy doctor
```

---

# 234. CLI

```text
forgeyard auth whoami
forgeyard auth login
forgeyard auth logout
forgeyard auth token create
forgeyard auth token revoke

forgeyard role list
forgeyard role bind
forgeyard role unbind

forgeyard policy validate
forgeyard policy compile
forgeyard policy diff
forgeyard policy test
forgeyard policy activate

forgeyard authz explain
```

---

# 235. `whoami`

Shows:

```text
PrincipalId
auth method
tenant
role bindings summary
session expiry
```

---

# 236. Token Create

Shows value exactly once.

---

# 237. Token Storage

CLI/user stores securely.

---

# 238. Dioxus UI

Admin/security sections:

```text
Users
Groups
Service Accounts
Roles
Permissions
Identity Providers
Policies
Protected Targets
Break-Glass
Audit
```

---

# 239. Policy UI

Should show compiled effective policy and source.

---

# 240. Authorization Explain UI

Useful for:

```text
why can't I merge?
why can't I deploy?
why can't this job use secret?
```

---

# 241. Testkit

```text
forgeyard-identity-testkit/src/
├── lib.rs
├── principal.rs
├── oidc.rs
├── session.rs
├── token.rs
└── assertions.rs
```

Authz:

```text
forgeyard-authz-testkit/src/
├── lib.rs
├── roles.rs
├── permissions.rs
├── scopes.rs
├── decisions.rs
└── assertions.rs
```

Policy:

```text
forgeyard-policy-testkit/src/
├── lib.rs
├── bundle.rs
├── input.rs
├── decision.rs
├── exception.rs
└── assertions.rs
```

---

# 242. Unit Tests

Test:

```text
scope inheritance
deny precedence
custom roles
policy merge
policy digest
```

---

# 243. OIDC Tests

```text
issuer mismatch
audience mismatch
expired token
nonce mismatch
JWKS rotation
```

---

# 244. SCIM Tests

```text
create
update
deactivate
group membership
idempotent sync
```

---

# 245. Authorization Tests

1. no binding -> deny;
2. project role grants project only;
3. tenant admin cannot cross tenant;
4. explicit deny wins;
5. revoked principal denied;
6. expired token denied.

---

# 246. Policy Tests

1. same input+digest -> same decision;
2. lower-level policy cannot weaken protected minimum;
3. exception preserves violation record;
4. expired exception no longer applies;
5. protected target requires queue.

---

# 247. Separation Tests

Author cannot self-approve when policy forbids.

---

# 248. Break-Glass Tests

Requires:

```text
permission
MFA/step-up
reason
expiry
audit
```

---

# 249. Runner Trust Tests

General runner cannot self-report SigningRestricted.

---

# 250. Workload Identity Tests

Job receives only exact allowed scopes and expires.

---

# 251. Cache Invalidation Tests

Role change invalidates effective authz cache.

---

# 252. Multi-Tenant Tests

Cross-tenant resource access denied even if ID guessed.

---

# 253. Failure Injection

```text
IdP unavailable
policy store unavailable
JWKS refresh failure
SCIM partial failure
cache stale
```

---

# 254. Security Fuzzing

Fuzz:

```text
policy parser
scope parser
permission inputs
OIDC callback/state parsing
```

Use vetted libraries for JWT/SAML cryptographic parsing.

---

# 255. Performance Tests

Measure:

```text
permission decision
effective role expansion
policy evaluation
cache invalidation
large group membership
```

---

# 256. Large Enterprise Scale

Test:

```text
100k principals
thousands of groups
many projects
many role bindings
```

---

# 257. Authorization Indexing

Store indexes by:

```text
principal
scope
role
group
```

---

# 258. Group Membership Expansion

Avoid expensive recursive expansion every request.

Use cached/materialized effective memberships with versioning.

---

# 259. Group Cycles

Reject internal group cycles.

---

# 260. External Group Sync

Normalize into membership table.

---

# 261. Event Integration

Events:

```text
PrincipalSuspended
RoleBindingChanged
PolicyActivated
RunnerTrustChanged
TokenRevoked
```

---

# 262. Reconciliation

Identity/provider reconcile:

```text
SCIM state drift
expired sessions/tokens cleanup
policy exception expiry
OIDC provider health
```

---

# 263. Durable Timers

Use for:

```text
break-glass expiry
exception expiry
temporary role grants
```

---

# 264. Temporary Role Grant

```rust
pub struct TemporaryRoleBinding {
    pub binding: RoleBinding,
    pub expires_at: Timestamp,
}
```

---

# 265. Just-In-Time Privilege

Useful enterprise capability.

---

# 266. JIT Grant Requires Approval

Policy-defined.

---

# 267. JIT Expiry

Automatic through timer/reconcile.

---

# 268. Session Fixation Defense

Rotate session ID after login/privilege change.

---

# 269. CSRF

Public UI mutation endpoints protected.

---

# 270. CORS

Restrictive.

---

# 271. API Token in Browser

Avoid.

---

# 272. Device Authorization Flow

CLI login can use OIDC device flow if browser callback impractical.

---

# 273. Local CLI Auth

Standalone can use local socket/OS user trust.

---

# 274. Local Socket Permissions

Protect daemon local admin socket.

---

# 275. Unix Peer Credentials

Optional local auth mechanism.

---

# 276. Windows Named Pipe ACL

Equivalent.

---

# 277. Authn Plugin Boundary

Identity provider adapters implement normalized trait.

---

# 278. IdentityProvider Trait

```rust
#[async_trait]
pub trait IdentityProvider {
    async fn authenticate(
        &self,
        request: AuthenticationRequest,
    ) -> Result<AuthenticatedExternalIdentity, IdentityError>;
}
```

Exact methods vary by flow; keep provider-specific concerns adapter-local.

---

# 279. Policy Store API

```rust
#[async_trait]
pub trait PolicyStore {
    async fn effective_policy(
        &self,
        scope: AuthorizationScope,
    ) -> Result<EffectivePolicy, PolicyStoreError>;
}
```

---

# 280. Role Store API

```rust
#[async_trait]
pub trait RoleStore {
    async fn bindings_for(
        &self,
        principal: PrincipalId,
        scope: AuthorizationScope,
    ) -> Result<Vec<RoleBinding>, StoreError>;
}
```

---

# 281. Authorization Service Composition

```text
Principal state
+
role/group bindings
+
scope
+
effective policy
+
request context
  ↓
AuthorizationDecision
```

---

# 282. Decision Evidence

For critical operations, persist:

```text
principal
permission
policy digest
decision
requirements
timestamp
```

---

# 283. Decision Reuse

Do not reuse old decision indefinitely.

Critical long-running workflows either bind evidence or recheck at action boundary.

---

# 284. Change Approval Evidence

Approval remains explicit domain evidence.

Authorization decision only proves reviewer had permission at approval time.

---

# 285. Production Deploy

Final deployment action may require current permission + prior approvals.

---

# 286. Release Sign

Signing worker also validates server-issued request/policy proof.

---

# 287. Policy Proof

Potential signed internal capability token for restricted workers.

Detailed in Secrets & Trust next.

---

# 288. Implementation Phase 1 — Principal/Permission Model

Implement:

```text
PrincipalId
PrincipalKind
PermissionId
Role
Scope
AuthorizationRequest/Decision
```

---

# 289. Phase 2 — Local Identity

Standalone/local server bootstrap.

---

# 290. Phase 3 — Role Bindings/Authz

Permission-based server enforcement.

---

# 291. Phase 4 — Policy Compiler/Evaluator

RON -> validated bundle -> digest -> deterministic decision.

---

# 292. Phase 5 — OIDC

Enterprise/user login.

---

# 293. Phase 6 — Sessions/Tokens

Browser/CLI/API automation.

---

# 294. Phase 7 — Protected Targets/Separation

Integrate Change Proposal.

---

# 295. Phase 8 — Runner/Workload Identity

Integrate mTLS and job delegation.

---

# 296. Phase 9 — SCIM/SAML

Enterprise adapters.

---

# 297. Phase 10 — Break-Glass/Exceptions/JIT

High-risk governance.

---

# 298. Phase 11 — Caching/Reconciliation

Scale/hardening.

---

# 299. Phase 12 — Security Hardening

Threat modeling, fuzzing, federation tests, audit review.

---

# 300. Acceptance Tests

1. Internal identity never relies on email.
2. OIDC subject maps to stable PrincipalId.
3. Invalid issuer/audience denied.
4. Suspended principal cannot act.
5. API token permissions are scoped.
6. Expired token/session denied.
7. Role grants permissions, domain never checks role names.
8. Project-scoped permission cannot cross project.
9. Tenant-scoped admin cannot cross tenant.
10. Explicit deny overrides allow where configured.
11. No matching permission -> deny.
12. Policy digest is deterministic.
13. Same policy input yields same decision.
14. Lower scope cannot weaken system/tenant protection.
15. Change approval checks reviewer permission at approval time.
16. Author cannot self-approve when separation rule forbids.
17. Protected target can require queue-only integration.
18. Policy exception preserves original violation.
19. Expired exception stops applying.
20. Break-glass requires strong auth/reason/expiry/audit.
21. Break-glass does not become permanent hidden admin.
22. Runner cannot self-promote trust.
23. Workload identity is short-lived and scope-limited.
24. Fork/untrusted proposal cannot receive privileged secret/signing permission.
25. Artifact digest knowledge does not bypass artifact authz.
26. Secret use can be granted without secret.admin.
27. Session/token revocation invalidates access.
28. Role/policy change invalidates authz cache.
29. SCIM deprovision suspends principal.
30. Existing IdP outage fails safely according to session policy.
31. Policy store uncertainty fails closed for protected writes.
32. Same authz semantics work in Stoolap/Postgres modes.
33. Audit captures security-sensitive permission/policy changes.
34. UI hidden controls are not relied on for enforcement.
35. Forgeyard self-hosting uses the same policy/authz/identity subsystem.

---

# 301. Production Readiness Gates

Do not call this subsystem production-ready until:

```text
permission model stable
scope isolation tested
local bootstrap safe
OIDC validation hardened
session/token lifecycle stable
policy compiler deterministic
policy digest binding works
protected targets integrated
separation-of-duties tested
runner trust binding works
workload identity constrained
revocation/cache invalidation tested
audit coverage verified
```

SCIM/SAML/break-glass/JIT can reach enterprise readiness incrementally.

---

# 302. Architectural Invariants

1. authentication and authorization are separate;
2. internal PrincipalId is stable authority identity;
3. email/display name are not identity authority;
4. external provider subjects are bindings;
5. roles are permission bundles only;
6. domain services check permissions, not role names;
7. authorization is scope-aware;
8. deny-by-default;
9. tenant isolation is explicit;
10. policy decisions are deterministic;
11. policy bundles have immutable digests;
12. lower policy cannot weaken protected higher-level requirements;
13. policy exceptions preserve violation evidence;
14. break-glass is explicit, expiring, strongly authenticated, audited;
15. runner trust is provisioned, not self-reported;
16. workloads receive delegated least-privilege identity;
17. source trust affects privileged access;
18. secret use is separate from secret administration;
19. protected writes fail closed on uncertain authz;
20. authorization cache is version/epoch bounded;
21. revocation invalidates active authority quickly;
22. sessions/tokens are short-lived/scoped;
23. public UI is never security authority;
24. provider usernames/roles do not directly become Forgeyard roles;
25. security-critical decisions record policy digest;
26. long workflows recheck where policy requires;
27. system tasks use typed system actor;
28. identity provider adapters do not leak into domain;
29. standalone and enterprise share same Principal/Permission model;
30. Forgeyard dogfoods its own policy and authorization rules.

---

# 303. Final Target Architecture

```text
               Human / Service / Runner / Workload
                              │
                              ▼
                       Authentication
                  ┌───────────┼───────────┐
                  ▼           ▼           ▼
                Local        OIDC       mTLS
                  │           │           │
                  └───────────┼───────────┘
                              ▼
                       PrincipalContext
                              │
                              ▼
                    Authorization Service
             ┌────────────────┼────────────────┐
             ▼                ▼                ▼
          Permissions       Scope           Policy
             │                │                │
             └────────────────┼────────────────┘
                              ▼
                    AuthorizationDecision
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
                  Deny                Allow
                                          │
                                          ▼
                                    Domain Action
                                          │
                                          ▼
                                         Audit
```

---

# 304. Final Architectural Position

Human request:

```text
authenticated PrincipalId
+
permission
+
resource scope
+
authn assurance
+
effective policy digest
  ↓
AuthorizationDecision
```

Workflow policy:

```text
source snapshot
proposal revision
checks
approvals
runner trust
environment
policy digest
  ↓
Allow / Deny / Require
```

Workload authority:

```text
initiating principal
  ↓
Run/Job
  ↓
short-lived constrained WorkloadIdentity
```

The key guarantee is:

> **Forgeyard never equates “logged in,” “has a role,” or “came from a trusted provider” with unrestricted authority. Every protected action is evaluated against an explicit principal, permission, resource scope, authentication context, and deterministic policy snapshot.**

---

# 305. New-Repository Sequence

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
