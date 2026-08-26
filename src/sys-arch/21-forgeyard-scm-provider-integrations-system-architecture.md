# 21 — Forgeyard SCM Provider Integrations System Architecture

**Document type:** Core Source-Control Hosting Provider Integration System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** SCM hosting providers, repository bindings, provider app/OAuth integrations, webhook ingestion, exact revision resolution, Change Proposal mapping, check/status publication, comments/reviews, merge/integration operations, installation lifecycle, rate limits, drift reconciliation, and provider-specific capability adapters  
**Architecture style:** Provider-neutral core, VCS/provider separation, exact immutable source identity, verified webhook ingress, at-least-once provider synchronization, provider-specific adapters behind capability traits, and reconciliation for missed/ambiguous external state  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on the VCS-neutral architecture, Change Proposal system, API/Axum, Events/Reconciliation, Identity/Authz/Policy, Secrets/Trust, Run/Job, Release, and Supply-Chain systems. It deliberately separates source-control **hosting providers** from source-control **VCS implementations**.

---

# 1. Purpose

Forgeyard must integrate cleanly with SCM hosting services such as:

```text
GitHub
GitLab
Bitbucket
Gitea/Forgejo
self-hosted enterprise forges
future provider APIs
```

These providers expose capabilities such as:

```text
repository discovery
webhooks
pull/merge requests
checks/statuses
comments
reviews
labels
branch protection
installation/authentication
release/tag views
```

But Forgeyard must not let provider semantics become its internal domain model.

The central rule is:

> **GitHub PR, GitLab MR, Bitbucket PR, and similar concepts map into Forgeyard `ChangeProposal`; provider-specific names never become core business types.**

A second rule is:

> **A provider repository/ref/revision is always resolved to an exact immutable `SourceSnapshotId` before Forgeyard schedules trusted work. Mutable branch names are navigation metadata only.**

A third rule is:

> **Webhook delivery is an optimization/trigger. Reconciliation against provider state repairs missed, duplicated, delayed, or ambiguous webhook effects.**

---

# 2. Architectural Position

```text
                SCM Hosting Provider
                        │
             ┌──────────┼──────────┐
             ▼          ▼          ▼
          Webhook      REST       App/OAuth
             │          │          │
             └──────────┼──────────┘
                        ▼
                 Provider Adapter
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
     Repository      Change         Checks
      Binding       Proposal        /Status
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                   Forgeyard Core
                        │
          ┌─────────────┼──────────────┐
          ▼             ▼              ▼
          VCS       SourceSnapshot   Policy/Run
        Adapter          Id
```

---

# 3. Goals

The subsystem MUST:

1. separate VCS from provider;
2. support provider installations/accounts;
3. support repository bindings;
4. support provider-native webhook verification;
5. deduplicate webhook deliveries;
6. normalize provider events;
7. resolve exact revisions;
8. create/update Change Proposals;
9. publish checks/statuses;
10. publish comments;
11. support review/approval mappings where provider supports;
12. support provider labels/metadata;
13. support integration/merge submission;
14. verify resulting integrated revision;
15. support provider rate limits;
16. support retries/backoff;
17. support missed-webhook reconciliation;
18. support ambiguous-submit reconciliation;
19. support provider credential rotation;
20. support enterprise/self-hosted endpoints;
21. support installation revocation;
22. support multi-tenant separation;
23. support provider capability discovery;
24. support SCM deep links;
25. support provider-specific diagnostics;
26. keep provider SDKs adapter-local;
27. never trust mutable refs for build identity;
28. never treat provider UI approval as universally authoritative without normalization;
29. preserve native IDs for navigation/audit;
30. remain extensible to new providers.

---

# 4. Non-Goals

This subsystem does not:

```text
implement Git/Mercurial itself
replace Forgeyard Change Proposal
replace SourceSnapshot identity
replace policy engine
replace provider's full UI
```

---

# 5. Workspace Structure

```text
crates/scm/
├── forgeyard-scm/
├── forgeyard-scm-model/
├── forgeyard-scm-provider/
├── forgeyard-scm-binding/
├── forgeyard-scm-webhook/
├── forgeyard-scm-change/
├── forgeyard-scm-check/
├── forgeyard-scm-comment/
├── forgeyard-scm-review/
├── forgeyard-scm-label/
├── forgeyard-scm-integration/
├── forgeyard-scm-reconcile/
├── forgeyard-scm-health/
├── forgeyard-scm-testkit/
│
├── forgeyard-scm-github/
├── forgeyard-scm-gitlab/
├── forgeyard-scm-bitbucket/
├── forgeyard-scm-gitea/
└── forgeyard-scm-forgejo/
```

VCS remains separate:

```text
crates/vcs/
├── forgeyard-vcs/
├── forgeyard-vcs-git/
├── forgeyard-vcs-mercurial/
└── ...
```

---

# 6. ProviderId

```rust
pub struct ScmProviderId(Ulid);
```

Represents configured provider instance.

---

# 7. Provider Kind

```rust
pub enum ScmProviderKind {
    GitHub,
    GitLab,
    Bitbucket,
    Gitea,
    Forgejo,
    Custom(ScmProviderKindId),
}
```

---

# 8. Provider Instance

Example:

```text
github.com
gitlab.com
git.company.internal
forgejo.internal
```

---

# 9. Provider Config

```rust
pub struct ScmProviderConfig {
    pub id: ScmProviderId,
    pub kind: ScmProviderKind,
    pub base_url: Url,
    pub api_url: Url,
    pub auth: ScmProviderAuthRef,
}
```

---

# 10. Public Cloud vs Self-Hosted

Provider adapter must not hardcode only public endpoint.

---

# 11. Installation Identity

```rust
pub struct ProviderInstallationId(Ulid);
```

Forgeyard semantic installation identity.

---

# 12. External Installation ID

Provider-native installation/app/account ID retained.

---

# 13. Installation Record

```rust
pub struct ProviderInstallation {
    pub id: ProviderInstallationId,
    pub provider: ScmProviderId,
    pub external_id: BoundedString,
    pub tenant: TenantId,
    pub state: ProviderInstallationState,
}
```

---

# 14. Installation State

```rust
pub enum ProviderInstallationState {
    Active,
    Suspended,
    Revoked,
    Unavailable,
}
```

---

# 15. Auth Types

Potential:

```rust
pub enum ScmProviderAuthKind {
    AppInstallation,
    OAuthApp,
    ServiceAccount,
    PersonalToken,
}
```

---

# 16. Preferred Authentication

Provider app/installation token where available.

---

# 17. Avoid Human PAT

Use PAT only fallback.

---

# 18. Credential Storage

SecretRef.

---

# 19. Short-Lived Token

Generate/refresh installation token when provider supports.

---

# 20. Provider Credential Scope

Only repositories/installations needed.

---

# 21. RepositoryBindingId

```rust
pub struct RepositoryBindingId(Ulid);
```

---

# 22. Repository Binding

```rust
pub struct RepositoryBinding {
    pub id: RepositoryBindingId,
    pub project: ProjectId,
    pub repository: RepositoryId,
    pub provider: ScmProviderId,
    pub installation: ProviderInstallationId,
    pub external_repository: ExternalRepositoryRef,
}
```

---

# 23. RepositoryId

Forgeyard VCS-neutral repository identity.

---

# 24. ExternalRepositoryRef

```rust
pub struct ExternalRepositoryRef {
    pub provider_repository_id: BoundedString,
    pub owner: BoundedString,
    pub name: BoundedString,
}
```

---

# 25. Provider Repo ID vs Name

Native repository ID is more stable than owner/name path.

Retain both.

---

# 26. Repository Rename

Binding remains through native ID.

---

# 27. Repository Transfer

Reconcile owner/namespace change.

---

# 28. Binding Authority

Only authorized admin/provider installation can bind repo.

---

# 29. VCS Type

Provider binding includes underlying VCS:

```text
Git
Mercurial
```

if provider supports multiple.

---

# 30. Provider vs VCS

Example:

```text
GitHub = hosting provider
Git = VCS
```

---

# 31. Provider Trait

```rust
#[async_trait]
pub trait ScmProvider {
    fn capabilities(&self) -> ScmCapabilities;

    async fn repository(
        &self,
        repo: &ExternalRepositoryRef,
    ) -> Result<ScmRepository, ScmError>;

    async fn change(
        &self,
        request: ExternalChangeRef,
    ) -> Result<ScmChange, ScmError>;

    async fn publish_check(
        &self,
        request: PublishCheckRequest,
    ) -> Result<PublishCheckResult, ScmError>;

    async fn submit_integration(
        &self,
        request: IntegrationSubmissionRequest,
    ) -> Result<IntegrationSubmissionResult, ScmError>;
}
```

---

# 32. Provider Capabilities

```rust
pub struct ScmCapabilities {
    pub webhooks: bool,
    pub checks: bool,
    pub commit_statuses: bool,
    pub reviews: bool,
    pub comments: bool,
    pub merge_queue: bool,
    pub branch_protection: bool,
    pub app_installations: bool,
}
```

---

# 33. Capability Honesty

Provider features vary by plan/version/self-hosted release.

Do not fake unsupported capability.

---

# 34. Native Change Ref

```rust
pub struct ExternalChangeRef {
    pub repository: ExternalRepositoryRef,
    pub number: BoundedString,
}
```

---

# 35. Forgeyard ChangeProposalId

Independent internal ID.

---

# 36. Provider Change Mapping

```text
GitHub Pull Request
GitLab Merge Request
Bitbucket Pull Request
  ↓
ChangeProposal
```

---

# 37. ChangeProposal External Binding

```rust
pub struct ChangeProposalExternalBinding {
    pub proposal: ChangeProposalId,
    pub provider: ScmProviderId,
    pub repository: RepositoryBindingId,
    pub external_change: ExternalChangeRef,
}
```

---

# 38. Provider State Normalization

Normalize:

```text
open
closed
merged/integrated
draft
```

to internal lifecycle facts.

---

# 39. Do Not Mirror Provider State Blindly

Forgeyard has richer separate states:

```text
review
checks
policy
mergeability
integration
```

---

# 40. Proposal Revision

Each external source update creates:

```text
new ProposalRevisionId
```

---

# 41. Exact Source Revision

Provider webhook/API supplies exact source revision ID.

---

# 42. Resolve Source

```text
provider revision
  ↓
VCS adapter fetch/materialize
  ↓
canonical tree
  ↓
SourceSnapshotId
```

---

# 43. Mutable Branch

Only contextual metadata.

---

# 44. Webhook Ingress

Path:

```text
/webhooks/{provider}/{binding-or-installation}
```

---

# 45. Webhook Flow

```text
raw request
  ↓
body size bound
  ↓
signature verification
  ↓
delivery ID dedup
  ↓
persist accepted delivery
  ↓
normalize
  ↓
process asynchronously
```

---

# 46. Webhook Signature

Provider-specific adapter.

---

# 47. Raw Body

Preserve exact bytes for signature verification where required.

---

# 48. DeliveryId

```rust
pub struct ProviderDeliveryId(BoundedString);
```

---

# 49. Dedup Key

```text
provider + installation + delivery ID
```

---

# 50. Webhook Event Types

Examples:

```text
repository changed
change opened
change updated
change closed
change merged
review submitted
push
installation revoked
```

---

# 51. Normalized Provider Event

```rust
pub enum ScmNormalizedEvent {
    RepositoryChanged(...),
    ChangeCreated(...),
    ChangeUpdated(...),
    ChangeClosed(...),
    ChangeIntegrated(...),
    ReviewChanged(...),
    RefUpdated(...),
    InstallationChanged(...),
}
```

---

# 52. Provider Event Is Not Domain Event Yet

Normalize/resolve then domain service decides transition.

---

# 53. Webhook Fast Ack

Return accepted after durable persistence/verification.

---

# 54. Processing Failure

Retry internally.

Provider need not wait.

---

# 55. Duplicate Provider Delivery

Safe.

---

# 56. Missed Webhook

Reconciliation.

---

# 57. Ref Update Event

Never schedule release build from branch name alone.

Resolve exact revision.

---

# 58. Push Event

Can trigger pipeline according to policy.

---

# 59. SourceSnapshot Binding

Triggered run stores exact SourceSnapshotId.

---

# 60. Change Event Processing

```text
fetch provider change
  ↓
read exact head/base revisions
  ↓
resolve VCS snapshots
  ↓
append ProposalRevision
  ↓
evaluate policy/checks
```

---

# 61. Provider Diff

May use provider API for navigation/metadata.

Forgeyard can compute canonical diff from snapshots where needed.

---

# 62. Review Mapping

Provider reviews normalize to Forgeyard ReviewVerdict.

---

# 63. Review Verdict

Use existing:

```text
Comment
Approve
RequestChanges
```

---

# 64. Native Provider Review ID

Stored for linkage.

---

# 65. Review Identity Mapping

Provider actor -> external actor.

If linked to Forgeyard principal:

```text
verified mapping
```

---

# 66. No Username Equality Trust

GitHub username != Forgeyard Principal automatically.

---

# 67. Unlinked External Review

May be preserved but policy decides whether it counts.

---

# 68. Provider Approval Counting

Do not assume every provider "approved" means Forgeyard policy approval.

Normalize then evaluate.

---

# 69. Comments

Forgeyard can publish:

```text
check summary
failure explanation
policy block
```

---

# 70. Comment Idempotency

Use marker/provider external comment ID.

---

# 71. Avoid Comment Spam

Update existing Forgeyard summary comment where provider supports.

---

# 72. Check Publishing

Forgeyard maps Run/Check result to provider check/status.

---

# 73. Internal Check Identity

```rust
pub struct CheckRunId(Ulid);
```

or existing check identity.

---

# 74. Provider Check Binding

```rust
pub struct ProviderCheckBinding {
    pub check: CheckRunId,
    pub provider: ScmProviderId,
    pub external_id: BoundedString,
}
```

---

# 75. Check Status

Normalize provider concepts.

---

# 76. Check Publication State

```rust
pub enum CheckPublicationState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Unknown,
}
```

---

# 77. Unknown Publication

Reconcile.

---

# 78. Check Idempotency

Same internal check/source snapshot updates same provider check where possible.

---

# 79. Commit Status Fallback

If provider lacks rich checks API.

---

# 80. Check Context Name

Stable Forgeyard-defined name.

---

# 81. Check Details URL

Deep link to Forgeyard run/check.

---

# 82. Status Security

Never publish secret/log raw data.

---

# 83. Check Summary

Sanitized.

---

# 84. Annotation

Provider-specific limit.

Bound count/size.

---

# 85. Provider Rate Limits

First-class.

---

# 86. Rate Limit Model

```rust
pub struct ProviderRateLimit {
    pub remaining: Option<u64>,
    pub reset_at: Option<Timestamp>,
    pub retry_after: Option<Duration>,
}
```

---

# 87. Rate-Limit Backoff

Respect provider hints.

---

# 88. Priority

Critical operations:

```text
integration verification
status finalization
webhook reconcile
```

before cosmetic comments.

---

# 89. Provider Request Budget

Per installation/provider.

---

# 90. Caching

Cache provider metadata with ETag/Last-Modified where supported.

---

# 91. Do Not Cache Mutable Critical State Too Long

Integration target state must be fresh.

---

# 92. Reconciliation

Periodic targeted sync.

---

# 93. Repository Reconcile

Checks:

```text
binding still valid
repo renamed/transferred
default branch changed
installation access revoked
```

---

# 94. Change Reconcile

Checks:

```text
provider head revision
base revision
open/closed state
reviews
mergeability
```

---

# 95. Check Reconcile

Checks:

```text
provider check missing/stale
status publication Unknown
```

---

# 96. Installation Reconcile

Checks permissions/access.

---

# 97. Ref Reconcile

For configured watched refs.

---

# 98. Missed Webhook Recovery

Reconcile identifies provider state newer than last known.

---

# 99. Reconcile Cursor

Per repository/provider.

---

# 100. Updated-Since

Use provider API if reliable.

---

# 101. Fallback Full Window

Bounded recent change scan.

---

# 102. No Unbounded Full Provider Crawl

Keyset/provider pagination.

---

# 103. Integration Submission

Change Proposal integration may be executed through provider.

---

# 104. Integration Request

```rust
pub struct IntegrationSubmissionRequest {
    pub proposal: ChangeProposalId,
    pub proposal_revision: ProposalRevisionId,
    pub expected_target_revision: RevisionId,
    pub candidate_snapshot: SourceSnapshotId,
    pub strategy: IntegrationStrategy,
}
```

---

# 105. Precondition

Provider target revision must still equal expected base.

---

# 106. Provider Merge Strategy

Map internal:

```text
FastForward
Merge
Rebase
Squash
BackendNative
```

to provider-supported operation.

---

# 107. Unsupported Strategy

Fail before submit.

---

# 108. Ambiguous Submission

Example:

```text
HTTP timeout after merge request accepted
```

state:

```text
Unknown
```

---

# 109. Unknown Integration Reconcile

```text
inspect target revision
inspect proposal state
materialize resulting revision
compare SourceSnapshotId
```

---

# 110. Never Blind Merge Retry

Critical.

---

# 111. Post-Integration Verification

After provider reports merged:

```text
resolve resulting revision
  ↓
materialize snapshot
  ↓
verify equals approved integration candidate
```

---

# 112. Mismatch

Critical integrity issue.

---

# 113. Provider Native Merge Queue

Optional adapter.

---

# 114. Forgeyard Queue

Can remain authoritative integration queue.

---

# 115. Hybrid

Forgeyard may submit to provider queue then reconcile result.

---

# 116. Branch Protection

Provider branch protection can be inspected/configured if API supports.

---

# 117. Provider Enforcement vs Forgeyard Enforcement

Expose distinction.

---

# 118. Protected Target Policy

Forgeyard policy remains authority for Forgeyard actions.

---

# 119. Provider Direct Push

If provider allows bypass outside Forgeyard, Forgeyard cannot guarantee block unless branch protection/provider permissions enforce it.

---

# 120. Honesty

UI/documentation must state protection coverage.

---

# 121. Repository Write Permission

Forgeyard integration app should have minimum needed.

---

# 122. Read-Only Mode

For CI-only integration:

```text
read repo
write checks
```

without merge permission.

---

# 123. Merge Permission

Separate optional scope.

---

# 124. Comment Permission

Separate if provider allows.

---

# 125. Installation Scope UI

Show requested provider permissions.

---

# 126. Least Privilege Installation

Recommended.

---

# 127. Webhook Secret Rotation

Dual-secret validation window if needed.

---

# 128. App Private Key

Secret/KMS managed.

---

# 129. OAuth Refresh Token

SecretRef/encrypted store.

---

# 130. Provider Token Caching

Short-lived in memory.

---

# 131. Credential Revocation

Provider 401/403 triggers installation health degradation/reconcile.

---

# 132. Installation Removed

Mark Suspended/Revoked.

---

# 133. Existing Runs

Can continue from already-materialized SourceSnapshot according to policy.

---

# 134. New Provider Actions

Blocked.

---

# 135. Provider Health

```rust
pub struct ScmProviderHealth {
    pub api: HealthStatus,
    pub auth: HealthStatus,
    pub webhook: HealthStatus,
    pub rate_limit: ProviderRateLimitHealth,
}
```

---

# 136. Repository Binding Health

```text
Healthy
Degraded
Revoked
Unknown
```

---

# 137. Doctor

```text
forgeyard scm doctor
```

---

# 138. Doctor Checks

```text
provider reachability
auth
repository access
webhook registration
check publication
rate-limit state
```

---

# 139. Safe Probe

Do not create spam comments/status unless explicitly testing.

---

# 140. Webhook Registration

Provider adapter can create/update webhook/app subscription where supported.

---

# 141. Webhook Ownership Marker

Identify Forgeyard-managed hook.

---

# 142. Duplicate Webhooks

Detect/avoid multiple equivalent hooks.

---

# 143. Self-Hosted TLS

Support custom trusted CA bundle via Trust subsystem.

---

# 144. Insecure TLS

Forbidden by default.

---

# 145. Proxy

Provider HTTP client may support configured proxy.

---

# 146. HTTP Client

Central hardened provider HTTP client.

---

# 147. Timeouts

Bound connect/request/read.

---

# 148. Retry

Only safe/idempotent provider operations automatically.

---

# 149. POST Retry

Only with provider idempotency/semantic reconciliation.

---

# 150. User Agent

Identify Forgeyard/version.

---

# 151. API Version Headers

Provider adapter-specific.

---

# 152. GitHub Adapter

Potential capabilities:

```text
GitHub App
installations
webhooks
pull requests
checks
statuses
reviews
comments
merge
branch protection
```

---

# 153. GitLab Adapter

Potential:

```text
OAuth/token/app model
webhooks
merge requests
pipelines/status
notes
approvals
merge
protected branches
```

---

# 154. Bitbucket Adapter

Normalize available capabilities.

---

# 155. Gitea/Forgejo Adapter

Useful self-hosted integration.

---

# 156. Provider API Drift

Self-hosted versions vary.

Capability discovery/version gates.

---

# 157. Adapter Version Support

Document/test supported provider versions.

---

# 158. Provider-Specific DTOs

Stay inside adapter.

---

# 159. Core Scm DTOs

Normalized stable internal model.

---

# 160. ScmRepository

```rust
pub struct ScmRepository {
    pub external: ExternalRepositoryRef,
    pub default_ref: Option<RefName>,
    pub visibility: RepositoryVisibility,
    pub archived: bool,
}
```

---

# 161. ScmChange

```rust
pub struct ScmChange {
    pub external: ExternalChangeRef,
    pub title: BoundedString,
    pub state: ScmChangeState,
    pub source_revision: RevisionId,
    pub target_revision: RevisionId,
    pub source_ref: Option<RefName>,
    pub target_ref: Option<RefName>,
}
```

---

# 162. RevisionId

VCS-native immutable revision ID.

---

# 163. SourceSnapshotId

Resolved canonical tree identity.

---

# 164. Provider Change Number

Navigation only.

---

# 165. Change Title/Body

Untrusted text.

Sanitize UI rendering.

---

# 166. Provider Labels

Normalize strings.

---

# 167. Labels Are Not Trust

Never grant privileged capability solely from external label unless explicit policy mapping.

---

# 168. Label Mapping

Policy can map trusted provider labels into workflow facts only if configured.

---

# 169. Review Comments

Untrusted.

---

# 170. Suggested Changes

Provider suggestion mapping can create new source revision only when user/provider actually applies.

---

# 171. Outdated Comments

Provider/native anchor state can supplement Forgeyard anchor logic.

---

# 172. Forgeyard Native Review Mode

Provider may be external navigation only.

---

# 173. External Review Mode

Provider review can be imported.

---

# 174. Hybrid Review Mode

Both Forgeyard and provider evidence.

---

# 175. Review Conflict

Keep source/origin.

Do not merge distinct review identities incorrectly.

---

# 176. Provider Checks Import

Forgeyard may ingest external checks.

---

# 177. External Check Trust

Policy decides which provider check contexts count.

---

# 178. Check Origin

```rust
pub enum CheckOrigin {
    Forgeyard,
    ProviderExternal,
    ThirdParty,
}
```

---

# 179. Required Check

Policy binds by trusted check identity, not display string alone.

---

# 180. Provider Context Spoofing

Prevent external user from creating similarly named status that satisfies Forgeyard requirement.

---

# 181. Trusted Check Binding

Map:

```text
provider app/integration identity + context
```

---

# 182. Webhook Event Security

Verify:

```text
signature
installation/repo binding
delivery dedup
event repository ID
```

---

# 183. Cross-Tenant Spoof

Webhook route cannot specify arbitrary tenant without verified binding.

---

# 184. Provider URL Validation

Avoid SSRF from provider-supplied URLs.

---

# 185. API URLs

Construct from configured trusted provider base, not arbitrary event links.

---

# 186. Clone URL

Validate allowed schemes/hosts.

---

# 187. Provider Redirect

Bound/validate.

---

# 188. Archive Download

If provider provides source tarball, still canonicalize into SourceSnapshotId.

---

# 189. Prefer VCS Fetch

Use VCS adapter where possible.

---

# 190. Provider Archive Semantics

Not assumed equivalent unless verified tree.

---

# 191. Submodules/Subrepos

Handled by VCS architecture.

Provider metadata may help authentication.

---

# 192. Private Submodule Credentials

Secret/provider token scoped.

---

# 193. Fork Proposal

Source repository may differ from target repository.

---

# 194. Fork Trust

`SourceTrust::Fork`.

---

# 195. Token Security for Fork

Never expose target repo write token to fork job.

---

# 196. Proposal Build Credentials

Read-only source fetch as needed.

---

# 197. Provider Check Token

Control plane publishes checks; job does not get provider token.

---

# 198. Security Boundary

Untrusted build cannot call provider API with Forgeyard installation credential.

---

# 199. Provider Actions

Only daemon/restricted integration worker.

---

# 200. SCM Worker

Optional dedicated worker for provider calls.

---

# 201. Daemon Initially

Provider calls can run in daemon service with bounded async.

---

# 202. Scale-Out Later

Separate integration worker queue.

---

# 203. Provider Operation Record

```rust
pub struct ScmOperation {
    pub id: ScmOperationId,
    pub provider: ScmProviderId,
    pub kind: ScmOperationKind,
    pub state: ScmOperationState,
}
```

---

# 204. Operation State

```text
Pending
InProgress
Succeeded
Failed
Unknown
```

---

# 205. Operation Types

```text
PublishCheck
PublishComment
SubmitReview
SubmitIntegration
RegisterWebhook
UpdateLabel
```

---

# 206. State-Changing Provider Calls

Persist desired operation before call when correctness matters.

---

# 207. Comment Publication

Lower criticality; can still use outbox.

---

# 208. Check Finalization

Important for provider UI consistency.

---

# 209. Integration Submission

Highest criticality.

---

# 210. Operation Idempotency

Adapter supplies semantic key.

---

# 211. Provider Retry Class

```rust
pub enum ScmRetryClass {
    SafeRetry,
    RetryAfterInspect,
    DoNotRetry,
}
```

---

# 212. 429

Safe after delay.

---

# 213. 5xx GET

Retry.

---

# 214. Merge Timeout

Inspect first.

---

# 215. Merge Conflict

No retry without state change.

---

# 216. Provider Pagination

Adapter consumes provider-specific pagination.

---

# 217. Core Pagination

Normalized cursor internal.

---

# 218. Repository Discovery

Installation can list accessible repositories.

---

# 219. Bind Flow

```text
select provider installation
  ↓
list accessible repos
  ↓
choose repo
  ↓
verify VCS access
  ↓
create RepositoryBinding
  ↓
register webhook
  ↓
initial reconcile
```

---

# 220. Initial Reconcile

Fetch:

```text
repo metadata
default branch
open changes
current protection/status config
```

bounded.

---

# 221. Import Existing Changes

Policy-controlled.

---

# 222. Backfill Limit

Do not ingest entire historical PR universe by default.

---

# 223. Unbind

Disable provider sync.

---

# 224. Unbind Does Not Delete Forgeyard History

Existing source/run/change history retained.

---

# 225. Rebind

Can restore same external repo to same internal RepositoryId if verified.

---

# 226. Repository Clone Auth

Provider credential helper/token generation.

---

# 227. VCS Adapter Interface

SCM provides authenticated fetch material; VCS performs fetch/tree resolution.

---

# 228. Separation Example

```text
GitHub App creates short-lived token
  ↓
Git adapter fetches revision
  ↓
Forgeyard canonicalizes SourceSnapshot
```

---

# 229. No GitHub SDK in VCS Core

Critical.

---

# 230. No Git Logic in GitHub Adapter Core Model

Provider adapter may expose clone URLs/revisions, but tree semantics remain VCS adapter.

---

# 231. Check Run Lifecycle

```text
Queued
InProgress
Completed
```

provider mapping.

---

# 232. Forgeyard Job State Mapping

Not one-to-one.

Aggregate into provider-safe summary.

---

# 233. Multiple Forgeyard Jobs

Can map to:

```text
one overall check
+
individual checks
```

configuration.

---

# 234. Recommended

One check per required workflow plus aggregate summary.

---

# 235. Check Annotation Limits

Truncate with link to Forgeyard detail.

---

# 236. Provider UI Is Secondary

Full logs/evidence remain Forgeyard.

---

# 237. Commit Status Race

Check published for exact revision.

---

# 238. Never Publish to Moving Branch Ref

Always revision SHA/ID.

---

# 239. Release Tag Integration

SCM provider may create tag/release view.

Release subsystem remains authority.

---

# 240. Tag Creation

Exact integrated revision.

---

# 241. Tag Signing

Supply-chain/release policy.

---

# 242. Provider Release View

Mirror Forgeyard release.

---

# 243. Provider Release Deletion

Does not delete Forgeyard release history.

---

# 244. Provider Webhook Event Retention

Raw accepted webhook can be retained short term for debugging.

---

# 245. Sensitive Raw Payload

Protected retention.

---

# 246. Normalized Event Retention

Longer if domain history requires.

---

# 247. Webhook Replay

Operator can replay accepted raw webhook through normalization after bug fix.

---

# 248. Replay Safety

Dedup/domain idempotency.

---

# 249. API Request Logging

No tokens.

---

# 250. Provider Error Logging

Sanitize bodies.

---

# 251. Provider Request Trace

Include:

```text
provider kind
operation
status
rate-limit info
```

not token.

---

# 252. Metrics

```text
scm_api_requests
scm_api_latency
scm_rate_limited
scm_webhook_received
scm_webhook_invalid
scm_webhook_duplicate
scm_reconcile_lag
scm_check_publish_failures
scm_integration_unknown
```

---

# 253. Metric Labels

Low-cardinality:

```text
provider_kind
operation
result_class
```

---

# 254. No RepositoryId Metric Label

Use traces/logs.

---

# 255. Tracing

```text
scm.webhook.verify
scm.webhook.normalize
scm.change.fetch
scm.revision.resolve
scm.check.publish
scm.integration.submit
scm.reconcile
```

---

# 256. Health

Provider health contributes to project integration health.

---

# 257. Degraded Mode

Provider unavailable:

```text
already-running jobs continue
new source triggers delayed
provider status sync delayed
```

---

# 258. Source Fetch Availability

If exact SourceSnapshot already in CAS, some reruns can continue without provider.

---

# 259. New Change Runs

May block until exact source fetched.

---

# 260. API Rate Limit Exhausted

Reconciliation/status delayed.

---

# 261. Prioritization

Protect:

```text
source resolution
integration correctness
final checks
```

---

# 262. UI

SCM provider pages:

```text
Providers
Installations
Repositories
Webhooks
Health
Rate Limits
```

---

# 263. Repository Integration Page

Shows:

```text
provider
external repo
VCS
binding health
webhook status
last reconcile
```

---

# 264. Change Proposal UI

Provider deep link shown.

---

# 265. Check Publication UI

Status + provider link.

---

# 266. Integration UI

Shows provider capability/strategy.

---

# 267. Provider Limitation

Example:

```text
provider cannot perform requested merge strategy
```

clear.

---

# 268. Installation Permission UI

Show granted/missing permissions.

---

# 269. Webhook Doctor UI

Last delivery/verification/reconcile.

---

# 270. Rate Limit UI

Admin diagnostics.

---

# 271. API Endpoints

Potential:

```text
GET  /v1/scm/providers
POST /v1/scm/providers
GET  /v1/scm/installations
POST /v1/scm/bindings
GET  /v1/scm/bindings/{id}
POST /v1/scm/bindings/{id}/reconcile
GET  /v1/scm/bindings/{id}/health
```

---

# 272. Authorization Permissions

```text
scm.read
scm.bind
scm.admin
scm.reconcile
scm.integrate
```

---

# 273. Provider Installation Admin

Tenant/org scoped.

---

# 274. Repository Binding Admin

Project/org scoped.

---

# 275. Integration Permission

Change integration still requires central Change/Policy permission.

---

# 276. Provider Permission Is Additional

Provider token must also have right.

---

# 277. Two-Layer Authorization

```text
Forgeyard authz
+
provider API permission
```

---

# 278. Provider Denial

Surface as external authorization failure.

---

# 279. Local Standalone

Can use local Git without provider integration.

---

# 280. Provider Optional

Forgeyard does not require GitHub/GitLab.

---

# 281. Offline Mode

Local VCS-only workflows continue.

---

# 282. SCM Testkit

```text
forgeyard-scm-testkit/src/
├── lib.rs
├── provider.rs
├── webhook.rs
├── repository.rs
├── change.rs
├── check.rs
├── integration.rs
├── rate_limit.rs
└── assertions.rs
```

---

# 283. Adapter Conformance Tests

Every provider adapter tests:

1. repository fetch;
2. change fetch;
3. webhook verification;
4. check publication;
5. pagination;
6. rate limit mapping;
7. integration capability if supported.

---

# 284. Webhook Tests

```text
invalid signature
duplicate delivery
unknown event
oversized body
repo binding mismatch
```

---

# 285. Source Identity Test

Provider branch update resolves exact revision + SourceSnapshotId.

---

# 286. Mutable Ref Test

Branch moves after webhook; run still uses webhook exact revision.

---

# 287. Fork Test

Source fork identity/source trust correct.

---

# 288. Review Mapping Test

Provider approval maps to normalized review with exact revision.

---

# 289. Actor Mapping Test

Same username without verified binding does not become internal PrincipalId.

---

# 290. Check Spoof Test

Untrusted third-party status cannot satisfy trusted required check.

---

# 291. Check Idempotency Test

Retry updates same semantic provider check.

---

# 292. Rate Limit Test

429 honors reset/retry.

---

# 293. Installation Revocation Test

New provider operations fail; existing Forgeyard history remains.

---

# 294. Missed Webhook Test

Drop provider event; reconcile catches updated proposal revision.

---

# 295. Integration Ambiguity Test

Provider merges then response times out; reconcile verifies resulting revision.

---

# 296. Integration Mismatch Test

Provider result snapshot differs from candidate -> critical failure.

---

# 297. Repository Rename Test

Native repository ID preserves binding.

---

# 298. Self-Hosted Version Test

Capability differences handled.

---

# 299. Credential Rotation Test

New provider credential works without binding recreation.

---

# 300. Security Test

Build workload cannot obtain provider installation token.

---

# 301. SSRF Test

Provider payload URL cannot force arbitrary outbound fetch.

---

# 302. Fuzzing

Fuzz:

```text
webhook parser
normalized event conversion
provider pagination cursor
review/comment text handling
```

---

# 303. Failure Injection

```text
provider outage
DNS failure
TLS failure
429
500
timeout
credential revocation
```

---

# 304. Scale Tests

```text
thousands of repositories
many webhook events
large number of open changes
provider rate-limit pressure
```

---

# 305. Reconcile Scale

Use incremental cursors and bounded pages.

---

# 306. Implementation Phase 1 — Provider-Neutral Model

Implement:

```text
ScmProviderId
RepositoryBinding
ScmCapabilities
normalized change/check models
```

---

# 307. Phase 2 — Webhook Framework

Signature, dedup, persistence, normalization.

---

# 308. Phase 3 — GitHub

First Tier-0 provider adapter.

---

# 309. Phase 4 — Change Proposal Mapping

Exact revision/source snapshot binding.

---

# 310. Phase 5 — Check/Status Publication

Run/check deep links.

---

# 311. Phase 6 — Provider Reconciliation

Missed webhook/status drift.

---

# 312. Phase 7 — Integration Submission

Precondition + post-merge verification.

---

# 313. Phase 8 — GitLab

Second major adapter.

---

# 314. Phase 9 — Gitea/Forgejo/Bitbucket

Additional adapters.

---

# 315. Phase 10 — Review/Comment/Label Enrichment

Optional features.

---

# 316. Phase 11 — Enterprise/Self-Hosted Hardening

Custom CA, proxy, provider versions.

---

# 317. Phase 12 — Scale/Fuzz/Failure Testing

Production hardening.

---

# 318. Acceptance Tests

1. VCS and SCM provider are separate abstractions.
2. GitHub PR maps to ChangeProposal, not core GitHubPR type.
3. GitLab MR maps to same ChangeProposal model.
4. Repository binding uses stable provider repository ID where available.
5. Repo rename does not break binding.
6. Webhook signature is verified before semantic processing.
7. Duplicate webhook delivery is idempotent.
8. Webhook repo identity must match binding.
9. Mutable branch names never become build identity.
10. Exact provider revision is resolved before run creation.
11. SourceSnapshotId is canonical authority.
12. Fork proposals are marked untrusted/fork.
13. Provider token is never exposed to build workload.
14. Provider approval is normalized before policy evaluation.
15. Unlinked external actor does not automatically become Forgeyard principal.
16. Trusted required checks cannot be spoofed by same display name.
17. Check publication binds exact revision.
18. Check retries are idempotent.
19. Provider rate limits are honored.
20. Missed webhook is repaired by reconciliation.
21. Installation revocation blocks new provider actions.
22. Existing Forgeyard history remains after provider revocation.
23. Integration submit checks expected target revision.
24. Ambiguous merge result is inspected before retry.
25. Resulting provider revision is materialized and verified against approved candidate snapshot.
26. Mismatch after provider integration is critical.
27. Provider-specific SDKs remain adapter-local.
28. Self-hosted provider base URLs/custom trust are supported.
29. Provider feature limitations are exposed honestly.
30. Local VCS workflows work without SCM provider.
31. SCM outages do not corrupt already-authoritative Forgeyard state.
32. External release/tag views do not replace Forgeyard release authority.
33. Provider webhooks/events never carry secret values into domain events.
34. Standalone/distributed share normalized SCM semantics.
35. Forgeyard's own repository integration dogfoods this subsystem.

---

# 319. Production Readiness Gates

Do not call SCM integrations production-ready until:

```text
provider/VCS separation enforced
webhook verification/dedup stable
exact revision resolution proven
Change Proposal mapping stable
check publication idempotent
rate limit handling tested
provider reconciliation catches missed events
integration ambiguity handling proven
post-integration snapshot verification works
credential isolation tested
```

Additional providers can graduate independently after adapter conformance.

---

# 320. Architectural Invariants

1. SCM provider != VCS;
2. provider names never become core business types;
3. ChangeProposal is provider-neutral;
4. SourceSnapshotId is source authority;
5. mutable refs are navigation only;
6. webhooks are verified and deduplicated;
7. webhooks are triggers, not sole correctness mechanism;
8. reconciliation repairs missed provider events;
9. provider actors require verified identity mapping;
10. provider approvals are normalized before policy;
11. trusted checks bind trusted origin, not display text only;
12. check publication binds exact revision;
13. provider credentials never reach untrusted build workloads;
14. provider SDKs remain adapter-local;
15. provider rate limits are first-class;
16. state-changing provider calls are idempotent/reconciled;
17. ambiguous integration is inspected before retry;
18. integration checks expected target revision;
19. resulting revision is re-materialized and snapshot-verified;
20. provider UI is secondary to Forgeyard authority;
21. repo rename/transfer is reconciled;
22. installation revocation does not erase history;
23. self-hosted endpoints are supported;
24. insecure TLS is forbidden by default;
25. provider-supplied URLs do not bypass outbound security;
26. fork trust is explicit;
27. local VCS mode does not require provider integration;
28. provider capability limitations are honest;
29. standalone/distributed share SCM semantics;
30. Forgeyard dogfoods its SCM integration system.

---

# 321. Final Target Architecture

```text
                    SCM Provider
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
        Webhook         API        App/OAuth
           │             │             │
           └─────────────┼─────────────┘
                         ▼
                   Provider Adapter
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
      Repository      Change        Check/Review
       Binding       Normalizer       Publisher
           │             │             │
           └─────────────┼─────────────┘
                         ▼
                  Forgeyard Domain
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
       ChangeProposal   VCS      SourceSnapshotId
                         │
                         ▼
                   Run / Policy
```

---

# 322. Final Architectural Position

Inbound change flow:

```text
verified webhook/provider poll
  ↓
external repository/change ID
  ↓
exact VCS source revision
  ↓
VCS adapter materialization
  ↓
SourceSnapshotId
  ↓
ProposalRevision
  ↓
checks/policy/run
```

Outbound check flow:

```text
Forgeyard check result
  ↓
exact source revision
  ↓
provider check/status adapter
  ↓
idempotent publication
  ↓
reconciliation
```

Integration flow:

```text
approved ProposalRevision
+
expected target revision
+
approved integration candidate snapshot
  ↓
provider submit
  ↓
success / failure / unknown
  ↓
inspect resulting revision
  ↓
materialize
  ↓
verify SourceSnapshotId
```

The key guarantee is:

> **Forgeyard can integrate deeply with GitHub, GitLab, Bitbucket, Forgejo, and future hosting providers without becoming structurally dependent on any of them. Provider APIs are synchronization edges; Forgeyard's own VCS-neutral source snapshots, Change Proposals, policies, runs, and release records remain the internal source of truth.**

---

# 323. New-Repository Sequence

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
