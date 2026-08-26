# 67 — Forgeyard Artifact Promotion Policy, Release Train, Environment Channel & Lifecycle Governance System Architecture

**Document type:** Core Artifact Promotion Policy, Release Train, Environment Channel, Ring Progression, Freeze Window, Hotfix Lane & Release Lifecycle Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** release trains, promotion channels, environment rings, release eligibility, scheduled trains, freeze windows, maintenance windows, emergency hotfix lanes, channel inheritance, release progression, rollback lineage, release retirement, rollout readiness, compatibility/supply-chain/policy gating, and promotion lifecycle governance  
**Architecture style:** Immutable release identity, explicit lifecycle state, channel-as-pointer not artifact identity, scheduled progression, evidence-bound eligibility, policy-controlled exceptions, separate hotfix lane, deterministic ring promotion, and no rebuild between lifecycle stages  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Packaging, Release, Deployment, Artifact Registry, Progressive Delivery, Compatibility Governance, Supply Chain, Policy/Authz, Notifications, Incident Management, Change Freeze, Cost/FinOps, Federation, and Update Delivery. This subsystem adds an organization-level release lifecycle above individual deployments.

---

## 1. Purpose

A production CI/CD platform often needs more than:

```text
release -> deploy
```

Organizations may use:

```text
nightly
alpha
beta
rc
stable
lts
```

or environment rings:

```text
development
integration
staging
pilot
production
```

or scheduled release trains:

```text
every Tuesday
monthly enterprise train
quarterly LTS train
```

They may also need:

```text
release freezes
holiday freezes
maintenance windows
hotfix exceptions
security emergency lanes
support windows
end-of-life transitions
```

Without a dedicated architecture, these rules become:

```text
manual spreadsheets
release-manager memory
branch naming conventions
provider-specific scripts
untracked exceptions
```

The central rule is:

> **A release channel or release train governs eligibility and timing for an existing immutable ReleaseId; it never creates or modifies the artifact itself.**

A second rule is:

> **Promotion through channels/rings is a lifecycle state transition backed by exact evidence, compatibility, policy, and target readiness.**

A third rule is:

> **Emergency hotfix lanes may shorten ordinary cadence, but they never bypass release identity, signing, authorization, audit, or minimum security requirements.**

---

## 2. Architectural Position

```text
                      ReleaseId
                         │
                         ▼
                  Release Eligibility
                         │
                         ▼
                    Release Train
                         │
                ┌────────┼────────┐
                ▼        ▼        ▼
              Alpha     Beta      RC
                │        │        │
                └────────┼────────┘
                         ▼
                       Stable
                         │
                         ▼
                        LTS
```

Environment rings:

```text
dev -> integration -> staging -> pilot -> production
```

---

## 3. Goals

The subsystem MUST:

1. define channel identity;
2. define release train identity;
3. define ring identity;
4. define lifecycle eligibility;
5. bind exact ReleaseId;
6. support alpha/beta/rc/stable/lts;
7. support environment rings;
8. support scheduled release trains;
9. support manual trains;
10. support freeze windows;
11. support maintenance windows;
12. support hotfix lanes;
13. support security emergency promotion;
14. support release eligibility gates;
15. support channel-specific compatibility rules;
16. support supply-chain requirements;
17. support manual approvals;
18. support tenant/region/channel scopes;
19. support rollback lineage;
20. support channel alias movement;
21. support retirement/deprecation;
22. support EOL;
23. support audit;
24. support Dioxus UI/API/CLI;
25. support federation;
26. support air-gap;
27. support HA;
28. support DR;
29. preserve immutable artifact identity;
30. avoid branch-name-driven lifecycle truth.

---

## 4. Non-Goals

This subsystem does not replace:

```text
Release creation
artifact signing
Deployment
Progressive Delivery
Artifact Registry
package-manager version semantics
```

It governs release progression.

---

## 5. Workspace Structure

```text
crates/release-lifecycle/
├── forgeyard-release-lifecycle/
├── forgeyard-release-channel/
├── forgeyard-release-train/
├── forgeyard-release-ring/
├── forgeyard-release-eligibility/
├── forgeyard-release-freeze/
├── forgeyard-release-hotfix/
├── forgeyard-release-retirement/
├── forgeyard-release-reconcile/
├── forgeyard-release-health/
└── forgeyard-release-lifecycle-testkit/
```

---

## 6. ReleaseChannelId

```rust
pub struct ReleaseChannelId(Digest);
```

---

## 7. Release Channel

```rust
pub struct ReleaseChannel {
    pub id: ReleaseChannelId,
    pub name: BoundedString,
    pub policy: ChannelPolicyId,
    pub audience: ReleaseAudience,
}
```

---

## 8. Channel Examples

```text
nightly
alpha
beta
candidate
stable
lts
internal
enterprise
```

---

## 9. Channel Is A Mutable Pointer/Collection

Not artifact identity.

---

## 10. ReleaseId Remains Canonical

Critical.

---

## 11. Channel Pointer

```rust
pub struct ChannelPointer {
    pub channel: ReleaseChannelId,
    pub release: ReleaseId,
    pub generation: u64,
}
```

---

## 12. Pointer Update

Protected mutation.

---

## 13. Expected Generation

Required.

---

## 14. ReleaseTrainId

```rust
pub struct ReleaseTrainId(Ulid);
```

---

## 15. Release Train

```rust
pub struct ReleaseTrain {
    pub id: ReleaseTrainId,
    pub policy: ReleaseTrainPolicyId,
    pub target_channel: ReleaseChannelId,
    pub schedule: Option<ReleaseTrainSchedule>,
}
```

---

## 16. Train Types

```rust
pub enum ReleaseTrainKind {
    Continuous,
    Scheduled,
    Manual,
    LongTermSupport,
    Security,
    Hotfix,
}
```

---

## 17. Continuous

Eligible release moves when gates pass.

---

## 18. Scheduled

Only during configured release windows.

---

## 19. Manual

Release manager explicitly starts.

---

## 20. LTS

Additional lifecycle/support requirements.

---

## 21. Security Train

Prioritizes vulnerability remediation.

---

## 22. Hotfix

Expedited exceptional lane.

---

## 23. Release Train Schedule

```rust
pub struct ReleaseTrainSchedule {
    pub timezone: TimeZoneId,
    pub recurrence: CalendarSchedule,
}
```

---

## 24. Timezone Mandatory

No ambiguous UTC/local assumptions.

---

## 25. Missed Train

Policy options:

```rust
skip
run next window
manual decide
```

---

## 26. No Surprise Catch-Up

Critical.

---

## 27. RingId

```rust
pub struct ReleaseRingId(Digest);
```

---

## 28. Ring

Represents ordered exposure/lifecycle stage.

Examples:

```text
ring-0 internal
ring-1 pilot
ring-2 early adopters
ring-3 general availability
```

---

## 29. Ring Policy

```rust
pub struct RingPolicy {
    pub eligibility: EligibilityPolicyId,
    pub rollout: Option<PromotionPlanId>,
}
```

---

## 30. Ring Sequence

Directed acyclic graph baseline.

---

## 31. Linear Baseline

Preferred initially.

---

## 32. Release Lifecycle

```rust
pub enum ReleaseLifecycleState {
    Built,
    Verified,
    Candidate,
    ChannelEligible,
    Promoting,
    Active,
    Deprecated,
    Retiring,
    Retired,
    Revoked,
}
```

---

## 33. Built

Release artifacts exist.

---

## 34. Verified

Required release evidence passed.

---

## 35. Candidate

May enter promotion lifecycle.

---

## 36. ChannelEligible

Meets selected channel rules.

---

## 37. Active

Currently distributed/deployed via channel/ring.

---

## 38. Deprecated

Still available but no longer preferred.

---

## 39. Retired

No longer offered for normal new installs.

---

## 40. Revoked

Security/correctness prohibition.

---

## 41. Eligibility

```rust
pub struct ReleaseEligibilityId(Digest);
```

---

## 42. Eligibility Inputs

```text
ReleaseId
ChannelPolicyId
PolicyDigest
CompatibilityReportId
SupplyChainEvidence
Test/quality evidence
Target support matrix
```

---

## 43. Eligibility Result

```rust
pub enum EligibilityResult {
    Eligible,
    EligibleWithApproval,
    Ineligible,
    Incomplete,
    Unknown,
}
```

---

## 44. Unknown Is Not Eligible

Critical.

---

## 45. Channel Policy

Can require:

```text
reproducibility
SBOM
signature
vulnerability thresholds
compatibility
test level
manual approval
minimum soak time
```

---

## 46. Stable Stronger Than Beta

Typical but configurable.

---

## 47. LTS Strongest Support Requirements

Can require:

```text
longer validation
support commitment
upgrade compatibility
migration path
security maintenance
```

---

## 48. Release Soak

```rust
pub struct SoakRequirement {
    pub minimum_duration: Duration,
    pub source_ring: ReleaseRingId,
}
```

---

## 49. Soak Does Not Mean Healthy Automatically

Need evidence.

---

## 50. Soak Timer

Durable.

---

## 51. Evidence During Soak

Part 62/50.

---

## 52. Environment Rings

Can map:

```text
dev
integration
staging
pilot
production
```

---

## 53. Channel vs Environment

Distinct.

Channel:

```text
distribution audience/version stream
```

Environment:

```text
runtime deployment target
```

---

## 54. Example

`stable` channel can be deployed to multiple production environments.

---

## 55. Promotion Between Channels

Example:

```text
alpha -> beta -> rc -> stable
```

---

## 56. Promotion Between Environments

Part 62.

---

## 57. Release Lifecycle Can Require Both

Example:

```text
candidate
  ↓
staging
  ↓
pilot
  ↓
stable channel
```

---

## 58. PromotionRequestId

```rust
pub struct ReleaseLifecyclePromotionId(Ulid);
```

---

## 59. Promotion Request

```rust
pub struct ReleaseLifecyclePromotion {
    pub id: ReleaseLifecyclePromotionId,
    pub release: ReleaseId,
    pub from: Option<LifecycleStageRef>,
    pub to: LifecycleStageRef,
    pub eligibility: ReleaseEligibilityId,
}
```

---

## 60. Same Bytes

Promotion does not rebuild.

---

## 61. Same Signature?

Existing artifact signatures remain.

Channel metadata may have separately signed pointer/update metadata.

---

## 62. No Resigning To Pretend New Artifact

Critical.

---

## 63. Channel Metadata Signature

Optional/high assurance.

---

## 64. Stable Alias

Mutable pointer to exact release.

---

## 65. Alias Movement

Audit + expected generation.

---

## 66. Alias Rollback

Moves pointer to prior ReleaseId.

---

## 67. Old Release Must Still Be Valid

Security floor applies.

---

## 68. Freeze Window

```rust
pub struct ReleaseFreezeId(Ulid);
```

---

## 69. Freeze Scope

```rust
pub enum ReleaseFreezeScope {
    Installation,
    Project(ProjectId),
    Channel(ReleaseChannelId),
    Ring(ReleaseRingId),
    Region(RegionId),
}
```

---

## 70. Freeze Types

```rust
pub enum ReleaseFreezeKind {
    Calendar,
    Incident,
    Reliability,
    Manual,
    Compliance,
}
```

---

## 71. Calendar Freeze

Examples:

```text
holiday
quarter-end
exam season for school ERP
```

---

## 72. Freeze Behavior

Can block:

```text
normal promotion
channel pointer movement
new production rollout
```

---

## 73. Freeze Does Not Revoke Existing Release

Critical.

---

## 74. Emergency Exception

Explicit.

---

## 75. Hotfix Lane

```rust
pub struct HotfixLaneId(Digest);
```

---

## 76. Hotfix Purpose

Expedited release path for urgent production correction.

---

## 77. Hotfix Requirements

Still require minimum:

```text
exact ReleaseId
signature
provenance
security checks
compatibility
authorization
audit
```

---

## 78. Can Shorten

```text
soak time
release cadence
manual queue wait
```

if policy allows.

---

## 79. Cannot Skip

```text
artifact identity
trust/signing
minimum security floor
authorization
audit
```

---

## 80. Hotfix Base

Exact current production ReleaseId.

---

## 81. Hotfix Lineage

```rust
pub struct ReleaseLineage {
    pub parent: Option<ReleaseId>,
    pub kind: ReleaseLineageKind,
}
```

---

## 82. Lineage Kind

```rust
pub enum ReleaseLineageKind {
    Normal,
    Hotfix,
    SecurityPatch,
    Rollback,
    RebuildVerification,
}
```

---

## 83. Hotfix Merge-Back

Operational process.

---

## 84. Source Reconciliation

Hotfix source changes must return to main development line.

---

## 85. Forgeyard Tracks

```text
hotfix released
merge-back pending
merge-back completed
```

---

## 86. No Permanent Forked Hotfix Branch Assumption

Critical.

---

## 87. Security Emergency Lane

Can override calendar freeze.

---

## 88. Still Policy-Governed

---

## 89. Compatibility

Part 57.

Channel may define compatibility support.

Example:

```text
stable requires N/N-1
lts requires N/N-2
```

---

## 90. Support Matrix

First-class.

---

## 91. SupportWindowId

```rust
pub struct SupportWindowId(Digest);
```

---

## 92. Support Window

```rust
pub struct SupportWindow {
    pub release: ReleaseId,
    pub starts_at: Timestamp,
    pub ends_at: Option<Timestamp>,
    pub level: SupportLevel,
}
```

---

## 93. Support Level

```rust
pub enum SupportLevel {
    Experimental,
    Preview,
    Standard,
    Extended,
    SecurityOnly,
    Ended,
}
```

---

## 94. EOL

Explicit date/state.

---

## 95. EOL Does Not Delete Artifact Immediately

Lifecycle policy decides retention.

---

## 96. Release Retirement

```text
Active
  ↓
Deprecated
  ↓
Retiring
  ↓
Retired
```

---

## 97. Deprecation Notice

Can be emitted before retirement.

---

## 98. Installed Clients

May remain.

---

## 99. Update System

Part 41 can guide supported upgrade path.

---

## 100. Upgrade Path

Part 57 version migration graph.

---

## 101. No Retire If No Supported Upgrade Path

Policy may block.

---

## 102. Release Train Candidate Selection

Candidates ordered by policy.

---

## 103. Candidate Selection Inputs

```text
release timestamp
priority
security urgency
compatibility
required fixes
```

---

## 104. No Branch Head Selection At Train Time

Critical.

Train selects immutable ReleaseIds.

---

## 105. Train Manifest

```rust
pub struct ReleaseTrainManifestId(Digest);
```

---

## 106. Train Manifest

Contains exact releases selected for a train.

---

## 107. Multi-Component Train

Possible.

---

## 108. Coordinated Release Train

For related services/packages.

---

## 109. Compatibility Graph

Determines version combination.

---

## 110. ReleaseSetId

```rust
pub struct ReleaseSetId(Digest);
```

---

## 111. Release Set

```rust
pub struct ReleaseSet {
    pub releases: Vec<ReleaseId>,
    pub compatibility: CompatibilityReportId,
}
```

---

## 112. Atomic Train

May require all components eligible before progression.

---

## 113. Partial Train

Can allow independent components.

---

## 114. Policy Explicit

---

## 115. Release Bundle

Can group artifacts.

---

## 116. Bundle Identity

Digest of exact member ReleaseIds.

---

## 117. No Mutable "current bundle"

Pointer only.

---

## 118. Channel Inheritance

Example:

```text
stable policy
  └── beta requirements
       └── alpha requirements
```

---

## 119. Policy Composition

Explicit.

---

## 120. No Hidden Inheritance

Critical.

---

## 121. ChannelPromotionGraph

```rust
pub struct ChannelPromotionGraph {
    pub edges: Vec<ChannelPromotionEdge>,
}
```

---

## 122. Edge

```rust
pub struct ChannelPromotionEdge {
    pub from: ReleaseChannelId,
    pub to: ReleaseChannelId,
    pub policy: EligibilityPolicyId,
}
```

---

## 123. DAG

Cycles rejected unless explicitly modeled as rollback transition.

---

## 124. Rollback

Separate edge type.

---

## 125. RollbackTarget

Exact prior ReleaseId.

---

## 126. Rollback Eligibility

Check:

```text
security floor
compatibility
DB migration state
config compatibility
```

---

## 127. Progressive Delivery

Part 62 controls exposure inside target environment.

---

## 128. Lifecycle Promotion

Can initiate Part 62 plan.

---

## 129. Part 62 Completion

Can satisfy lifecycle gate.

---

## 130. No Duplicate Rollout Authority

Critical.

---

## 131. Deployment

Part 16 authoritative runtime deployment state.

---

## 132. Artifact Registry

Part 52 authoritative package/blob distribution.

---

## 133. Release Lifecycle

Coordinates eligibility/timing.

---

## 134. Supply Chain

Part 13 supplies:

```text
SBOM
provenance
signatures
VEX
```

---

## 135. Eligibility Snapshot

Exact evidence references.

---

## 136. Security Vulnerability After Promotion

Release may become:

```text
Blocked
Revoked
SecurityOnly
```

depending severity.

---

## 137. Channel Pointer

Can roll away.

---

## 138. Installed Versions

May need update campaign.

---

## 139. Incident

Part 61.

Active incident can freeze promotion.

---

## 140. Incident Hotfix

IncidentId can justify hotfix lane.

---

## 141. Incident Does Not Auto-Approve

Critical.

---

## 142. Reliability

Part 50.

Error budget policy may block normal release train.

---

## 143. Security Hotfix

Can have explicit exception.

---

## 144. Maintenance Windows

Release train can coordinate with allowed deployment times.

---

## 145. Window Boundary

If train starts but target deployment extends past window:

```text
continue current bounded action
pause future progression
```

baseline.

---

## 146. No Mid-Transaction Abrupt Kill

Critical.

---

## 147. Calendar

Part 44 scheduling infrastructure.

---

## 148. Business Calendar

Can support:

```text
holidays
fiscal quarter
school term
```

---

## 149. CalendarVersionId

```rust
pub struct BusinessCalendarId(Digest);
```

---

## 150. Calendar Change

Does not retroactively alter historical train.

---

## 151. Manual Release Manager

Human role.

---

## 152. Role != Authorization

Existing invariant.

---

## 153. Release Manager Can Coordinate

But needs actual permission to promote.

---

## 154. Promotion Approval

```rust
pub struct LifecyclePromotionApproval {
    pub promotion: ReleaseLifecyclePromotionId,
    pub release: ReleaseId,
    pub target: LifecycleStageRef,
    pub policy: PolicyDigest,
}
```

---

## 155. Approval Exactness

Subject change invalidates.

---

## 156. Release Notes

Can attach exact compatibility/security facts.

---

## 157. Release Notes Generation

AI can draft; confirmed facts canonical.

---

## 158. Channel-Specific Notes

Example:

```text
beta known limitations
stable upgrade notes
LTS support statement
```

---

## 159. Release Train Announcement

Part 29 notifications.

---

## 160. Subscribers

Teams/tenants/customers.

---

## 161. Scheduled Train Notification

Before/after train.

---

## 162. No Notification = No Promotion Block Unless Policy Says

---

## 163. Train State

```rust
pub enum ReleaseTrainState {
    Planned,
    CandidateSelection,
    AwaitingEligibility,
    AwaitingWindow,
    Promoting,
    Paused,
    Succeeded,
    Failed,
    Cancelled,
}
```

---

## 164. Pause

Possible due:

```text
incident
freeze
failed eligibility
target health
manual
```

---

## 165. Resume

Re-evaluate freshness.

---

## 166. No Old Eligibility Reuse Blindly

Critical.

---

## 167. Candidate Replacement

If train is not yet sealed, policy may allow.

---

## 168. Train Seal

```rust
pub struct ReleaseTrainSeal {
    pub manifest: ReleaseTrainManifestId,
    pub sealed_at: Timestamp,
}
```

---

## 169. After Seal

Candidate set immutable.

---

## 170. Change Candidate

Create new train/manifest generation.

---

## 171. No Mid-Train Silent Substitution

Critical.

---

## 172. Distributed Installation

Federation Part 51.

---

## 173. Regional Channel State

Mutable pointer authority explicit.

---

## 174. Global Stable Channel

One authority domain.

---

## 175. Regional Mirror

Read-only replicated pointer/data where configured.

---

## 176. Disconnected Site

Can consume last approved local channel snapshot.

---

## 177. Offline Promotion

Only if site has delegated authority/policy.

---

## 178. Air-Gap Train

Can import signed release-train bundle.

---

## 179. Train Bundle

```rust
pub struct ReleaseTrainBundleId(Digest);
```

---

## 180. Bundle Contains

```text
ReleaseIds
artifacts
signatures
SBOM/provenance
eligibility evidence
channel transition intent
```

---

## 181. No Secret Values

---

## 182. Offline Verification

Required.

---

## 183. Multi-Tenancy

Channels can be:

```text
global
tenant-specific
project-specific
```

---

## 184. Tenant Beta Channel

Possible.

---

## 185. Tenant-Specific Stable

Use cautiously.

---

## 186. Channel Scope

```rust
pub enum ReleaseChannelScope {
    Global,
    Tenant(TenantId),
    Project(ProjectId),
    Product(ProductId),
}
```

---

## 187. Cross-Tenant Data

No leakage.

---

## 188. Tenant Opt-In

Can be required for beta channel.

---

## 189. Entitlements

Part 30.

Channel access may depend on product entitlement.

---

## 190. Entitlement Does Not Change Artifact Trust

Critical.

---

## 191. Licensing

Can control distribution audience.

---

## 192. Commercial Release

May publish same ReleaseId to entitled tenant channel.

---

## 193. Lifecycle Policy

Versioned.

---

## 194. Policy Change

Does not rewrite historical promotion decision.

---

## 195. Current Policy

Used for next promotion.

---

## 196. Audit

Audit events:

```text
channel pointer move
promotion approval
hotfix lane use
freeze override
train seal
candidate replacement before seal
retirement
revocation
EOL change
```

---

## 197. Routine Train Evaluation

Operational event.

---

## 198. Dioxus UI

Pages:

```text
Release Channels
Release Trains
Promotion Rings
Freeze Calendar
Hotfixes
Support Lifecycle
```

---

## 199. Channel Detail

Shows:

```text
current ReleaseId
policy
audience
support level
promotion history
```

---

## 200. Release Train View

Shows:

```text
schedule
candidate set
eligibility
window
current stage
blockers
```

---

## 201. Freeze Calendar

Visual.

---

## 202. Hotfix View

Shows lineage and merge-back status.

---

## 203. CLI

```text
forgeyard lifecycle channel list
forgeyard lifecycle channel show
forgeyard lifecycle train list
forgeyard lifecycle train plan
forgeyard lifecycle train start
forgeyard lifecycle promote
forgeyard lifecycle freeze
forgeyard lifecycle hotfix
forgeyard lifecycle retire
forgeyard lifecycle doctor
```

---

## 204. API

Potential:

```text
GET  /v1/release-channels
POST /v1/release-trains
GET  /v1/release-trains/{id}
POST /v1/release-trains/{id}/start
POST /v1/releases/{id}/promote
POST /v1/release-freezes
POST /v1/releases/{id}/retire
```

---

## 205. Permissions

```text
release_lifecycle.read
release_lifecycle.promote
release_lifecycle.train.manage
release_lifecycle.freeze.manage
release_lifecycle.hotfix
release_lifecycle.retire
release_lifecycle.revoke
```

---

## 206. Revoke

Highest privilege/security.

---

## 207. Hotfix

Elevated permission.

---

## 208. Freeze Override

Elevated.

---

## 209. Observability Metrics

```text
release_train_total
release_train_failures_total
release_promotion_total
release_promotion_blocked_total
release_freeze_active
release_hotfix_total
release_retired_total
```

---

## 210. Labels

Low-cardinality:

```text
channel
train_kind
result
```

Avoid ReleaseId in metrics labels.

---

## 211. Tracing

```text
release_lifecycle.eligibility
release_lifecycle.train
release_lifecycle.promote
release_lifecycle.freeze
release_lifecycle.hotfix
release_lifecycle.retire
```

---

## 212. Health

```rust
pub enum ReleaseLifecycleHealth {
    Healthy,
    ScheduleDegraded,
    EligibilityDegraded,
    PromotionDegraded,
    Unhealthy,
}
```

---

## 213. Doctor

```text
forgeyard lifecycle doctor
```

Checks:

```text
channel points to revoked release
expired freeze
train stuck awaiting window
sealed train references missing release
hotfix merge-back overdue
LTS release without support window
retired release still default
```

---

## 214. Data Lifecycle

Part 46.

Retain:

```text
train manifests
promotion evidence
channel history
freeze history
support/EOL history
hotfix lineage
```

---

## 215. Artifact Deletion

Independent from lifecycle state.

---

## 216. Retired Release

May remain retained for replay/audit/support.

---

## 217. CAS GC Roots

Support window can create root.

---

## 218. EOL

May relax retention later.

---

## 219. Cost

Part 45.

Release train itself low compute.

But coordinated validation/promotion may incur:

```text
test
environment
replication
deployment
```

---

## 220. Cost Does Not Bypass Required Eligibility

---

## 221. Security

Threats:

```text
channel pointer hijack
hotfix abuse
freeze bypass
candidate substitution
rollback to vulnerable release
forged eligibility
```

---

## 222. Controls

```text
exact ReleaseId
expected channel generation
signed evidence
policy digest
audit
security floor
```

---

## 223. Pointer Hijack

Protected by authz + optimistic version + audit.

---

## 224. Eligibility Evidence

Immutable.

---

## 225. New Vulnerability

Eligibility freshness may invalidate.

---

## 226. Current Security State

Rechecked before promotion.

---

## 227. No Permanent "once eligible, always eligible"

Critical.

---

## 228. EligibilityFreshness

```rust
pub enum EligibilityFreshness {
    Current,
    SecurityChanged,
    PolicyChanged,
    CompatibilityChanged,
    SupportChanged,
    Expired,
    Unknown,
}
```

---

## 229. Reconciliation

Release lifecycle reconciler checks:

```text
channel pointer validity
train schedule
freeze state
eligibility freshness
support/EOL transitions
hotfix merge-back
```

---

## 230. HA

Multiple lifecycle controllers safe.

---

## 231. Concurrency

Part 60 locks channel pointer/trains.

---

## 232. Fencing

Protect stale controller.

---

## 233. Provider Effects

Channel metadata may publish to external registries/update systems.

Use idempotency + reconciliation.

---

## 234. No Blind Republish

Critical.

---

## 235. Update Delivery

Part 41 consumes channel state.

---

## 236. Client Check

Gets exact release/update metadata.

---

## 237. Stable Channel

Can drive desktop/agent update feed.

---

## 238. Ring Rollout

Part 62.

---

## 239. Artifact Registry

Part 52.

Channel can map to package tags/dist-tags where ecosystem requires.

---

## 240. Native Registry Tags

Adapters only.

---

## 241. Canonical Channel State

Forgeyard metadata.

---

## 242. npm dist-tag Example

Adapter maps `stable` to dist-tag.

---

## 243. OCI Tag Example

Adapter maps channel alias to tag.

---

## 244. Tag Not Artifact Identity

Critical.

---

## 245. Cargo

Crate version immutable; channel may exist only in Forgeyard release metadata rather than registry semantics.

---

## 246. Platform Packages

May have ring/channel concepts.

---

## 247. Testkit

```text
forgeyard-release-lifecycle-testkit/src/
├── lib.rs
├── channel.rs
├── train.rs
├── eligibility.rs
├── freeze.rs
├── hotfix.rs
├── retirement.rs
└── assertions.rs
```

---

## 248. Core Tests

### Channel
- channel pointer references exact ReleaseId;
- stale generation update rejected;
- tag alias cannot redefine artifact digest.

### Eligibility
- missing evidence => Incomplete;
- revoked release never eligible;
- security state change invalidates freshness.

### Train
- sealed candidate set immutable;
- missed window follows explicit policy;
- resume re-evaluates freshness.

### Freeze
- normal promotion blocked;
- hotfix exception explicit;
- existing active release not revoked merely by freeze.

### Hotfix
- exact lineage recorded;
- minimum security/trust gates remain;
- merge-back tracking persists.

### Retirement
- support/EOL transitions explicit;
- default channel cannot point to retired/revoked release by policy.

### Federation
- one global channel authority;
- offline site uses verified snapshot only.

---

## 249. Chaos Tests

Inject:

```text
controller crash during channel move
external registry timeout
calendar service failure
federation partition
eligibility evidence service outage
```

Expected:

```text
no duplicate promotion
channel pointer remains coherent
Unknown/Incomplete visible
reconciliation repairs
```

---

## 250. Scale Tests

Test:

```text
many products/channels
large tenant-specific beta programs
multi-component trains
large historical promotion graph
```

---

## 251. Implementation Phases

### Phase 1 — Channel/Lifecycle Model
Core.

### Phase 2 — Manual Promotion
Exact ReleaseId progression.

### Phase 3 — Eligibility Policies
Supply-chain/compatibility/test gates.

### Phase 4 — Scheduled Release Trains
Calendar/windows.

### Phase 5 — Freeze/Hotfix Lanes
Operational governance.

### Phase 6 — Support/EOL Lifecycle
Long-term maintenance.

### Phase 7 — Multi-Component Release Sets
Coordinated trains.

### Phase 8 — Progressive Delivery Integration
Ring deployment.

### Phase 9 — Registry/Update Channel Adapters
Distribution.

### Phase 10 — Federation/Air-Gap
Enterprise.

### Phase 11 — UI/CLI/Doctor
Operability.

### Phase 12 — Security/Chaos/Scale Hardening
Production readiness.

---

## 252. Acceptance Tests

1. Channel always references exact ReleaseId.
2. Channel alias never becomes artifact identity.
3. Promotion never rebuilds/resigns software to create fake stage identity.
4. Release train candidates are immutable once sealed.
5. Branch head is never train truth.
6. Eligibility is explicit and evidence-backed.
7. Unknown/incomplete is never eligible automatically.
8. Security freshness can invalidate eligibility.
9. Stable/LTS can enforce stricter gates than alpha/beta.
10. Scheduled train behavior is timezone-aware.
11. Missed train behavior is explicit.
12. Freeze blocks normal progression without revoking current release.
13. Hotfix lane retains minimum trust/security/authz/audit.
14. Hotfix lineage is explicit.
15. Hotfix merge-back is tracked.
16. Channel promotion and environment deployment remain distinct.
17. Part 62 remains rollout authority.
18. Part 15 remains Release authority.
19. Part 52 remains artifact registry authority.
20. Support/EOL state is explicit.
21. Retirement does not imply immediate artifact deletion.
22. Rollback checks security/compatibility state.
23. Revoked release cannot become automatic rollback target.
24. Multi-component train uses exact ReleaseSetId.
25. Channel policy inheritance is explicit.
26. Federation has one accepted mutable channel authority.
27. Offline/air-gap channel state is signed/verified.
28. External tag/dist-tag failures reconcile.
29. Historical lifecycle events remain immutable.
30. Forgeyard dogfoods release trains/channels for its own release lifecycle.

---

## 253. Production Readiness Gates

Do not call release lifecycle governance production-ready until:

```text
channel pointer generation checks are enforced
same-bytes promotion is proven
eligibility freshness works
train sealing is immutable
freeze/hotfix exception paths are audited
registry/update adapter reconciliation works
rollback security floor is enforced
support/EOL transitions are tested
federation/air-gap channel authority works
chaos/scale tests pass
```

---

## 254. Architectural Invariants

1. ReleaseId is canonical artifact identity;
2. channels are pointers/audiences, not identities;
3. trains select immutable ReleaseIds;
4. sealed train candidates do not change silently;
5. eligibility has exact evidence;
6. Unknown is not eligible;
7. eligibility can become stale;
8. promotion never rebuilds;
9. freeze does not revoke existing release;
10. hotfix lane does not bypass trust/security/authz/audit;
11. hotfix lineage is recorded;
12. merge-back is tracked;
13. support/EOL are explicit;
14. retirement is distinct from deletion;
15. rollback observes current security floor;
16. channel promotion is distinct from deployment;
17. progressive delivery remains Part 62 authority;
18. release remains Part 15 authority;
19. registry remains Part 52 authority;
20. channel pointer updates use expected generation;
21. external tag updates reconcile after ambiguity;
22. policy inheritance is explicit;
23. federation has one mutable authority;
24. offline state is verified;
25. audit preserves lifecycle history;
26. current policy controls new progression;
27. historical decisions are not rewritten;
28. AI may draft notes, not eligibility truth;
29. no branch-name lifecycle truth;
30. Forgeyard dogfoods its own release lifecycle.

---

## 255. Final Target Architecture

```text
                        ReleaseId
                           │
                           ▼
                     Eligibility
                           │
                           ▼
                    Release Train
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
             Alpha        Beta         RC
               │           │           │
               └───────────┼───────────┘
                           ▼
                         Stable
                           │
                           ▼
                           LTS
```

Hotfix:

```text
production ReleaseId
        ↓
urgent fix
        ↓
new exact ReleaseId
        ↓
minimum trust/security gates
        ↓
hotfix lane
        ↓
progressive production rollout
        ↓
merge-back tracking
```

The key guarantee is:

> **Forgeyard can run disciplined release trains, channels, rings, freezes, and hotfix lanes without ever confusing lifecycle metadata with software identity. Every lifecycle transition moves an existing immutable ReleaseId under explicit evidence and policy, preserving the build-once/promote-same-bytes architecture.**

---

## 256. Extended Architecture Sequence

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
```
