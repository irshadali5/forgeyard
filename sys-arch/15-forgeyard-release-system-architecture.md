# 15 — Forgeyard Release System Architecture

**Document type:** Core Release Orchestration & Promotion System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Release candidates, immutable package sets, evidence gates, approval workflows, release locks, promotion, channels/rings, repository/store publication, release notes, staged rollout metadata, rollback, external publication state, reconciliation, and release auditability  
**Architecture style:** Build-once → verify → sign → candidate → approve → promote exact bytes, with immutable release inputs, explicit policy gates, idempotent external publication, and no rebuild during release  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on Packaging, Supply Chain/SBOM/Provenance/Signing, Policy/Authz/Identity, Secrets & Trust, Change Proposal, CAS, Run/Job, Events/Reconciliation, and hermetic/reproducible build architecture. It produces release records and publication actions that later feed `16 — Deployment`.

---

# 1. Purpose

Forgeyard needs a release subsystem that answers:

```text
which exact package artifacts form this release?
are all required targets present?
did all required checks pass?
is supply-chain evidence complete?
are artifacts signed/notarized as required?
who approved the release?
which exact bytes were promoted?
which channels received them?
what external publication succeeded?
what is safe to roll back to?
```

A release is not:

```text
run a release script
rebuild code
upload whatever file is in ./dist
```

The central rule is:

> **A release promotes immutable, already-built, already-verified artifacts. Release orchestration never rebuilds application code.**

A second rule is:

> **Every approval, signing decision, publication, and promotion is bound to exact artifact/package digests and a specific release candidate identity.**

A third rule is:

> **External publication is treated as a reconciled side effect with explicit Pending/InProgress/Succeeded/Failed/Unknown state, never assumed successful from a single network call.**

---

# 2. Architectural Position

```text
                PackageSet
                    │
                    ▼
             Release Candidate
                    │
      ┌─────────────┼─────────────┐
      ▼             ▼             ▼
  Evidence Gate   Policy Gate   Completeness
      │             │             │
      └─────────────┼─────────────┘
                    ▼
                Approval
                    │
                    ▼
               Release Lock
                    │
                    ▼
                 Promote
                    │
       ┌────────────┼────────────┐
       ▼            ▼            ▼
    Download      Registry      App Store
    Channel       Repository    Publication
       │            │            │
       └────────────┼────────────┘
                    ▼
              Release Record
                    │
                    ▼
                 Deploy
```

---

# 3. Goals

The release subsystem MUST:

1. define stable `ReleaseId`;
2. define immutable `ReleaseCandidateId`;
3. consume immutable package artifacts;
4. consume immutable evidence bundles;
5. require exact package digests;
6. support multi-platform package sets;
7. support completeness rules;
8. support release approval;
9. support separation of duties;
10. support protected release channels;
11. support release locks;
12. support exact-byte promotion;
13. support release notes;
14. support changelog generation;
15. support release metadata/signing;
16. support multiple publication destinations;
17. support idempotent publication;
18. support ambiguous publication reconciliation;
19. support staged channels/rings;
20. support rollback;
21. support yanking/deprecation;
22. support immutable release history;
23. support release verification;
24. support release events;
25. support release audit;
26. support air-gapped release bundles;
27. support standalone/local releases;
28. support distributed/enterprise releases;
29. integrate with deployment;
30. never rebuild artifacts during release.

---

# 4. Non-Goals

Release does not:

```text
compile source
package source
own signing private keys
execute deployment targets directly
replace app stores/package registries
replace policy engine
```

---

# 5. Workspace Structure

```text
crates/release/
├── forgeyard-release/
├── forgeyard-release-model/
├── forgeyard-release-candidate/
├── forgeyard-release-set/
├── forgeyard-release-version/
├── forgeyard-release-policy/
├── forgeyard-release-approval/
├── forgeyard-release-lock/
├── forgeyard-release-promote/
├── forgeyard-release-channel/
├── forgeyard-release-publish/
├── forgeyard-release-publish-model/
├── forgeyard-release-publish-http/
├── forgeyard-release-publish-oci/
├── forgeyard-release-publish-apt/
├── forgeyard-release-publish-rpm/
├── forgeyard-release-publish-github/
├── forgeyard-release-publish-gitlab/
├── forgeyard-release-publish-play/
├── forgeyard-release-publish-apple/
├── forgeyard-release-notes/
├── forgeyard-release-changelog/
├── forgeyard-release-rollback/
├── forgeyard-release-reconcile/
├── forgeyard-release-health/
└── forgeyard-release-testkit/
```

Application composition stays in:

```text
apps/forgeyard-daemon/
apps/forgeyard-cli/
apps/forgeyard-ui/
```

---

# 6. ReleaseId

```rust
pub struct ReleaseId(Ulid);
```

Stable release entity identity.

---

# 7. Release Candidate Identity

```rust
pub struct ReleaseCandidateId(Digest);
```

Derived from immutable release inputs.

---

# 8. Candidate Inputs

Candidate identity includes:

```text
project
release version
SourceSnapshotId
PackageSetId
exact package object digests
evidence bundle digests
policy snapshot inputs
release manifest semantics
```

---

# 9. Candidate Immutability

After created:

```text
artifact/package contents cannot change
```

Any package replacement creates:

```text
new ReleaseCandidateId
```

---

# 10. Release

```rust
pub struct Release {
    pub id: ReleaseId,
    pub project: ProjectId,
    pub version: ReleaseVersion,
    pub candidate: ReleaseCandidateId,
    pub state: ReleaseState,
    pub created_by: PrincipalId,
    pub policy: PolicyDigest,
    pub created_at: Timestamp,
}
```

---

# 11. Release State

```rust
pub enum ReleaseState {
    Draft,
    Candidate,
    Verifying,
    AwaitingApproval,
    Approved,
    Promoting,
    Released,
    PartiallyReleased,
    Failed,
    Superseded,
    Yanked,
}
```

---

# 12. Draft

Mutable release planning metadata may still change.

No immutable candidate frozen yet.

---

# 13. Candidate

Release input set is frozen.

---

# 14. Verifying

Forgeyard evaluates:

```text
package completeness
evidence
signature
notarization
policy
```

---

# 15. AwaitingApproval

Technical gates passed.

Human/policy approval still required.

---

# 16. Approved

Exact candidate approved.

No package replacement allowed.

---

# 17. Promoting

One or more external/internal publication actions in progress.

---

# 18. Released

Required release destinations succeeded according to policy.

---

# 19. PartiallyReleased

Some destinations succeeded, some failed/unknown.

This is important.

Do not collapse into generic Failed.

---

# 20. Failed

Release could not meet required release policy and no required destination was successfully finalized, or policy explicitly declares failure.

---

# 21. Superseded

Another release/candidate replaces this unreleased candidate.

---

# 22. Yanked

Previously released version intentionally withdrawn/deprecated where destinations permit.

Bytes/history remain immutable.

---

# 23. Terminal vs Mutable Release Metadata

Release state can evolve.

Candidate contents cannot.

---

# 24. ReleaseVersion

```rust
pub struct ReleaseVersion(BoundedString);
```

Version parser/strategy may depend on project ecosystem.

---

# 25. Version Strategy

Support:

```text
SemVer
CalVer
custom validated
```

---

# 26. Release Version vs Package Versions

Usually consistent.

Policy can require all package set members share release version.

---

# 27. Immutable Version Mapping

Once Release reaches Released:

```text
version -> release candidate
```

should not be repointed silently.

---

# 28. Re-release Same Version

Default:

```text
forbidden
```

Use new version.

Exception only for ecosystem-specific legitimate metadata operation, explicit/audited.

---

# 29. PackageSet

From Packaging architecture.

---

# 30. Release Package Set

```rust
pub struct ReleasePackageSet {
    pub id: PackageSetId,
    pub packages: Vec<ReleasePackageRef>,
}
```

---

# 31. ReleasePackageRef

```rust
pub struct ReleasePackageRef {
    pub package: PackageId,
    pub object: CasObjectRef,
    pub target: PackageTarget,
    pub evidence: EvidenceBundleId,
}
```

---

# 32. Exact Object Rule

Release never points only to:

```text
filename
version
tag
```

It points to exact CAS digest.

---

# 33. Package Completeness

Release policy declares required targets.

---

# 34. Target Requirement

```rust
pub struct ReleaseTargetRequirement {
    pub target: PackageTargetSelector,
    pub required: bool,
}
```

---

# 35. Example Completeness

```text
Linux x86_64 required
Linux arm64 required
Windows x64 required
macOS arm64 required
Android AAB required
```

---

# 36. Optional Targets

Can be explicitly optional.

---

# 37. Missing Required Target

Candidate cannot reach Approved.

---

# 38. Cross-Target Source Consistency

By default all packages in one release must bind same:

```text
SourceSnapshotId
```

---

# 39. Cross-Target Version Consistency

Same logical release version.

---

# 40. Mixed Source Release

Allowed only if explicitly modeled.

Example:

```text
multi-component release
```

---

# 41. Release Manifest

```rust
pub struct ReleaseManifest {
    pub version: ReleaseVersion,
    pub source: SourceSnapshotId,
    pub packages: Vec<ReleasePackageRef>,
    pub notes: ReleaseNotesRef,
    pub channels: Vec<ReleaseChannelRef>,
    pub policy: PolicyDigest,
}
```

---

# 42. Manifest Storage

Canonical manifest stored in CAS.

Metadata stores ref/digest.

---

# 43. Manifest Digest

Part of ReleaseCandidateId.

---

# 44. Candidate Freeze

```text
draft release
  ↓
resolve package/evidence refs
  ↓
canonical manifest
  ↓
digest
  ↓
ReleaseCandidateId
```

---

# 45. Candidate Verification

```text
all package objects exist
all evidence bundles exist
all signatures verify
required notarization exists
policy requirements met
target completeness met
```

---

# 46. Supply-Chain Gate

Consumes `SupplyChainStatus`.

---

# 47. Required Evidence

Examples:

```text
SBOM
provenance
signature
reproducibility
vulnerability scan
license policy
VEX where needed
```

---

# 48. Evidence Freshness

Time-sensitive evidence must still be fresh at approval/promotion as policy requires.

---

# 49. Reverification

Before final promotion:

```text
verify again
```

critical items if policy/trust changed.

---

# 50. Trust Epoch

Signature verification considers current TrustEpoch/history.

---

# 51. Policy Snapshot

Candidate stores:

```text
PolicyDigest
```

used for candidate verification.

---

# 52. Policy Change Before Promotion

Protected release may require re-evaluation under current effective policy.

---

# 53. Policy Compatibility

Do not silently keep old approval if new policy adds mandatory gate.

---

# 54. Approval

```rust
pub struct ReleaseApproval {
    pub id: ReleaseApprovalId,
    pub release: ReleaseId,
    pub candidate: ReleaseCandidateId,
    pub approver: PrincipalId,
    pub decision: ApprovalDecision,
    pub policy: PolicyDigest,
    pub at: Timestamp,
}
```

---

# 55. Approval Decision

```rust
pub enum ApprovalDecision {
    Approve,
    Reject,
}
```

---

# 56. Approval Binds Candidate

Any candidate change invalidates approvals.

---

# 57. Approval Policy

Can require:

```text
N approvals
release manager
security approver
code owner
MFA
separation of duties
```

---

# 58. Separation of Duties

Possible:

```text
builder != approver
author != release approver
release approver != production publisher
```

---

# 59. Self-Approval

Policy-controlled.

Default for high-assurance release:

```text
not sufficient alone
```

---

# 60. Approval Expiry

Optional.

---

# 61. Approval Revocation

Explicit event.

---

# 62. Release Lock

Prevents competing promotion of conflicting candidates/version/channel.

---

# 63. ReleaseLock

```rust
pub struct ReleaseLock {
    pub id: ReleaseLockId,
    pub project: ProjectId,
    pub version: ReleaseVersion,
    pub candidate: ReleaseCandidateId,
    pub expires_at: Timestamp,
}
```

---

# 64. Lock Scope

Could include:

```text
project/version
channel
destination
```

---

# 65. Lock Authority

Stored transactionally.

---

# 66. Release Lock vs Raft

Can initially use Postgres/store lock.

Raft may later protect global/exclusive release operations but is not required initially.

---

# 67. Lock Expiry

Durable timer/reconciler.

---

# 68. Promotion

Promotion means:

```text
make exact already-verified artifact available in intended release channel/destination
```

---

# 69. Promotion Does Not Build

Hard invariant.

---

# 70. Internal Promotion

Examples:

```text
mark candidate Stable
publish manifest on download site
make package visible in internal registry
```

---

# 71. External Publication

Examples:

```text
GitHub Release
GitLab Release
OCI registry
APT repository
RPM repository
Google Play
Apple App Store / notarization distribution
generic HTTP/object storage
```

---

# 72. Publication Destination

```rust
pub enum ReleaseDestination {
    DownloadSite,
    GenericObjectStore,
    OciRegistry,
    AptRepository,
    RpmRepository,
    GitHubRelease,
    GitLabRelease,
    GooglePlay,
    AppleAppStore,
    Custom(ReleaseDestinationId),
}
```

---

# 73. Destination Adapter

```rust
#[async_trait]
pub trait ReleasePublisher {
    async fn publish(
        &self,
        request: PublishRequest,
    ) -> Result<PublishResult, PublishError>;

    async fn inspect(
        &self,
        query: PublishInspectQuery,
    ) -> Result<RemotePublishState, PublishError>;
}
```

---

# 74. PublishRequest

```rust
pub struct PublishRequest {
    pub publication: PublicationId,
    pub release: ReleaseId,
    pub candidate: ReleaseCandidateId,
    pub destination: ReleaseDestination,
    pub artifacts: Vec<PublishedArtifactRef>,
    pub metadata: PublishMetadata,
    pub idempotency: PublishIdempotencyKey,
}
```

---

# 75. PublicationId

```rust
pub struct PublicationId(Ulid);
```

---

# 76. Publication State

```rust
pub enum PublicationState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Unknown,
    RolledBack,
}
```

---

# 77. Unknown

Critical for:

```text
network timeout after upload/request
```

---

# 78. No Blind Retry

If publication is Unknown:

```text
inspect remote state
```

before retry.

---

# 79. Idempotency

Use provider idempotency keys if available.

Otherwise derive/check:

```text
release version
artifact digest
destination coordinates
```

---

# 80. Remote Coordinates

Examples:

```text
OCI repo/tag+digest
GitHub release ID/tag
APT repo path
Play track/version code
Apple version/build number
```

---

# 81. Mutable Tags

Can be used as navigation labels.

Digest remains authority.

---

# 82. OCI Publish

Push exact image manifest/layers by digest.

---

# 83. OCI Tag

Apply release tag after digest upload.

---

# 84. APT/RPM Publish

Repository metadata is itself an immutable release artifact/versioned snapshot.

---

# 85. Repository Atomicity

Build new repository index/snapshot, sign it, then atomically switch pointer where storage supports.

---

# 86. Repository Signing

Restricted signing path.

---

# 87. Download Site

Publish package + checksum/signature/evidence manifest.

---

# 88. GitHub/GitLab Release

Provider release entry is a publication view.

Forgeyard Release remains authority.

---

# 89. Provider Asset Upload

Verify remote size/digest where provider permits.

---

# 90. App Store Publication

Stateful long-running external workflow.

---

# 91. Google Play

Potential stages:

```text
upload artifact
create/edit release
assign track
commit
processing
available
```

---

# 92. Apple App Store

Potential stages:

```text
upload
processing
metadata
review/submission
available
```

---

# 93. App Store State Normalization

Use provider adapter states but preserve provider-specific details.

---

# 94. Publication Isn't Deployment

Publishing app/package is release distribution.

Installing/running in environment belongs to deployment.

---

# 95. Channels

```rust
pub enum ReleaseChannel {
    Nightly,
    Dev,
    Alpha,
    Beta,
    Candidate,
    Stable,
    LongTermSupport,
    Custom(ReleaseChannelId),
}
```

---

# 96. Channel Semantics

Project-defined policy.

---

# 97. Channel Pointer

Mutable pointer:

```text
stable -> ReleaseId
```

---

# 98. Digest Authority

Channel points to immutable ReleaseId/Candidate.

---

# 99. Promotion Across Channels

```text
Candidate
  ↓
Beta
  ↓
Stable
```

can reuse exact bytes.

---

# 100. No Rebuild Across Channels

Absolutely.

---

# 101. Ring / Cohort

```rust
pub struct ReleaseRing {
    pub id: ReleaseRingId,
    pub order: u16,
}
```

Useful for staged client update availability.

---

# 102. Release Ring vs Deployment Ring

Release ring means:

```text
who may discover/download/update to version
```

Deployment ring means:

```text
where software is actively deployed
```

Keep separate.

---

# 103. Channel Promotion Policy

Could require:

```text
time soak
no regressions
manual approval
telemetry health
```

Deployment/observability may feed facts.

---

# 104. Release Promotion Gate

```rust
pub struct PromotionRequirements {
    pub source_channel: ReleaseChannel,
    pub target_channel: ReleaseChannel,
    pub evidence: Vec<EvidenceRequirement>,
    pub approvals: Vec<ApprovalRequirement>,
}
```

---

# 105. Release Notes

First-class artifact/metadata.

---

# 106. ReleaseNotesRef

```rust
pub struct ReleaseNotesRef(CasObjectRef);
```

---

# 107. Release Notes Sources

Potential:

```text
Change Proposals
commit/change summaries
user-authored notes
issue tracker integrations
```

---

# 108. Generated Notes

Generated draft only.

Human can edit before candidate freeze.

---

# 109. Notes Provenance

Store source references used to generate notes.

---

# 110. Changelog

Project history over releases.

---

# 111. Changelog Entry

```rust
pub struct ChangelogEntry {
    pub release: ReleaseId,
    pub version: ReleaseVersion,
    pub changes: Vec<ChangeSummary>,
}
```

---

# 112. Change Summary Identity

Bind Change Proposal/revision/integration result.

---

# 113. No Mutable Branch-Based Notes

Use integrated exact revisions.

---

# 114. Release Notes Categories

Examples:

```text
Added
Changed
Fixed
Security
Deprecated
Removed
```

---

# 115. Security Notes

May require restricted/private release before disclosure.

---

# 116. Embargoed Release

Optional state/policy:

```text
metadata restricted
artifacts private
```

until release time.

---

# 117. Scheduled Release

Durable timer can trigger promotion after approval.

---

# 118. Schedule Does Not Skip Final Verification

At timer fire:

```text
recheck required gates
```

---

# 119. Release Time

Use control-plane time.

---

# 120. Release Candidate Expiry

Optional.

If evidence becomes stale, candidate returns to verification/approval requirement.

---

# 121. Candidate Supersession

New candidate for same version can supersede old before release.

---

# 122. Released Candidate Immutability

Cannot replace released version's candidate.

---

# 123. Hotfix

New version/release.

---

# 124. Yank

Does not delete artifacts.

---

# 125. Yank Semantics

```text
prevent new adoption
mark security/deprecation
```

where destination supports.

---

# 126. Unyank

Policy-controlled, audited.

---

# 127. Rollback

Release rollback does not necessarily mean deleting release.

---

# 128. Rollback Types

```rust
pub enum ReleaseRollbackKind {
    ChannelPointer,
    RepositoryPointer,
    UpdateFeed,
    DeploymentTrigger,
}
```

---

# 129. Channel Rollback

Move channel pointer from release B to previous release A.

---

# 130. Exact Previous Artifact

Rollback target is immutable known release.

---

# 131. Repository Rollback

Republish prior repository snapshot/index.

---

# 132. App Store Rollback

Often provider constraints prevent true binary rollback.

May require new release/build/version.

Adapter must report real capability.

---

# 133. Rollback Capability

```rust
pub enum RollbackCapability {
    ImmediatePointer,
    RepublishPrior,
    ProviderLimited,
    Unsupported,
}
```

---

# 134. No False Rollback Promise

UI must explain destination limitations.

---

# 135. Release History

Append-only semantic history.

---

# 136. Release Event Model

```text
ReleaseDraftCreated
ReleaseCandidateFrozen
ReleaseVerificationPassed
ReleaseApprovalAdded
ReleaseApproved
ReleasePromotionStarted
PublicationSucceeded
PublicationFailed
PublicationUnknown
ReleaseReleased
ReleaseYanked
ReleaseRolledBack
```

---

# 137. Release Event Scope

Events bind:

```text
ReleaseId
ReleaseCandidateId
```

---

# 138. Release State Transitions

Recommended:

```text
Draft -> Candidate
Candidate -> Verifying
Verifying -> AwaitingApproval
Verifying -> Failed
AwaitingApproval -> Approved
Approved -> Promoting
Promoting -> Released
Promoting -> PartiallyReleased
Promoting -> Failed
Released -> Yanked
Candidate/AwaitingApproval -> Superseded
```

---

# 139. Candidate Verification Failure

Can fix by creating new package/evidence then new candidate.

Do not mutate frozen candidate.

---

# 140. Approval Rejection

Release stays AwaitingApproval or becomes Failed/Rejected depending product UX.

---

# 141. Recommended Approval Status Separation

Keep:

```text
release state
approval aggregate
```

rather than adding many rejected states.

---

# 142. Release Approval Aggregate

```rust
pub struct ReleaseApprovalStatus {
    pub required: usize,
    pub approved: usize,
    pub rejected: usize,
    pub satisfied: bool,
}
```

---

# 143. Publication Aggregate

```rust
pub struct PublicationAggregate {
    pub required_succeeded: usize,
    pub required_total: usize,
    pub optional_failed: usize,
    pub unknown: usize,
}
```

---

# 144. Required vs Optional Destination

```rust
pub struct ReleaseDestinationSpec {
    pub destination: ReleaseDestination,
    pub required: bool,
}
```

---

# 145. Optional Publication Failure

May still allow Released.

---

# 146. Required Publication Unknown

Release cannot be final Released until reconciled.

---

# 147. Release Completion Evaluator

```text
all required publications Succeeded
  ↓
Released
```

unless policy requires additional condition.

---

# 148. Partial Release

Useful if required destinations diverged.

Operator/reconciler must resolve.

---

# 149. Publication Reconciler

For each nonterminal publication:

```text
inspect remote state
compare expected coordinates/digest
update state
```

---

# 150. Remote Digest Verification

Where provider supports exact hash.

---

# 151. Provider Without Digest

Use strongest available:

```text
size
provider asset ID
download + hash
version/build number
```

---

# 152. Download-and-Verify

For critical publication if provider allows reading back.

---

# 153. Publishing Credentials

Resolved late via Secrets subsystem.

---

# 154. Credential Scope

Per destination:

```text
upload/release only
repository path
app/project
```

---

# 155. No Build Credential

Publisher has no build authority.

---

# 156. Publisher Worker

Could run as restricted service/worker.

---

# 157. Release Worker Capability

```text
publish exact artifacts
modify release destination metadata
```

not arbitrary runner shell.

---

# 158. Provider SDK Isolation

SDK lives in adapter crate.

---

# 159. Release Publisher Trait

Core independent of GitHub/AWS/etc.

---

# 160. Publish Metadata

```rust
pub struct PublishMetadata {
    pub version: ReleaseVersion,
    pub notes: ReleaseNotesRef,
    pub channel: ReleaseChannel,
    pub prerelease: bool,
}
```

---

# 161. Checksums

Publish:

```text
SHA-256
BLAKE3
```

where useful.

---

# 162. Signature Assets

Publish detached signatures/public evidence.

---

# 163. SBOM/Provenance Assets

Can publish alongside release.

---

# 164. Evidence Manifest

Machine-readable release evidence index.

---

# 165. Release Index

```rust
pub struct ReleaseIndex {
    pub release: ReleaseId,
    pub candidate: ReleaseCandidateId,
    pub version: ReleaseVersion,
    pub packages: Vec<ReleaseIndexPackage>,
}
```

---

# 166. Public Release Manifest

JSON may be appropriate for download clients.

Internal canonical form separate.

---

# 167. Update Feed

Release subsystem can generate immutable feed snapshot.

---

# 168. Feed Entry

```text
version
channel
platform
arch
package URL/ref
digest
signature
minimum version
```

---

# 169. Feed Signing

Restricted signing.

---

# 170. Feed Pointer

Mutable channel endpoint points to immutable feed snapshot.

---

# 171. Client Update Security

Client verifies:

```text
feed signature
package signature/digest
```

---

# 172. Delta Updates

Release can publish delta artifacts produced by packaging.

---

# 173. Delta Verification

Client/release verifies reconstruction to exact target digest.

---

# 174. Release Policy Facts

```rust
pub struct ReleasePolicyFacts {
    pub target_completeness: bool,
    pub supply_chain_verified: bool,
    pub approvals: ReleaseApprovalStatus,
    pub signatures_valid: bool,
    pub notarization_valid: bool,
    pub evidence_fresh: bool,
}
```

---

# 175. Central Policy Engine

No separate release policy language.

---

# 176. Release Policy Examples

```text
Stable requires 2 approvals
Stable requires multi-party reproduction
Stable requires all Tier-1 platforms
Nightly can skip human approval
Production mobile release requires signed/notarized artifacts
```

---

# 177. Release Environment

Release channel is not deployment environment.

Keep distinct.

---

# 178. Manual Promotion

Authorized CLI/UI action.

---

# 179. Automatic Promotion

Policy/automation may promote after gates.

---

# 180. Automation Identity

Service principal.

---

# 181. Approval Cannot Be Faked by Automation Unless Policy Allows

Explicit.

---

# 182. Release Freeze Window

Policy can block stable releases during defined windows.

---

# 183. Emergency Release

Break-glass path.

---

# 184. Emergency Release Requirements

```text
explicit break-glass
reason
MFA
audit
limited scope
```

---

# 185. Break-Glass Does Not Change Artifact Digest

It bypasses policy requirement, not supply-chain identity.

---

# 186. Emergency Policy Evidence

Record exception.

---

# 187. Release Audit

Mandatory for:

```text
candidate freeze
approval
policy exception
promotion
publication
rollback
yank
```

---

# 188. Release Principal

Record human/service actor.

---

# 189. Release Lock Audit

For conflicts.

---

# 190. Concurrent Releases

Different versions can proceed concurrently.

Same version/channel protected by lock/policy.

---

# 191. Release Queue

Optional sequence for stable channel.

---

# 192. Release Queue Entry

```rust
pub struct ReleaseQueueEntry {
    pub release: ReleaseId,
    pub candidate: ReleaseCandidateId,
    pub channel: ReleaseChannel,
    pub position: u64,
}
```

---

# 193. Queue Use

Ensure ordered publication.

---

# 194. Queue Revalidation

Before promotion, target channel state may have changed.

---

# 195. Compare-and-Swap Channel Pointer

```text
expected current release
  ↓
set new release
```

---

# 196. Prevent Lost Update

Use version/transaction.

---

# 197. Channel Epoch

Optional:

```rust
pub struct ReleaseChannelVersion(u64);
```

---

# 198. Promotion Candidate Drift

Candidate cannot drift by design.

---

# 199. Publication Destination Drift

External state can drift.

Reconciler detects.

---

# 200. Release Verification

```text
forgeyard release verify
```

Should independently verify release record.

---

# 201. Verify Steps

```text
resolve manifest
verify manifest digest
verify package digests
verify signatures
verify evidence bundles
verify policy record
verify publication coordinates
```

---

# 202. Offline Verification

Works with air-gap release bundle.

---

# 203. Air-Gap Release Bundle

Contains:

```text
release manifest
package artifacts
checksums
signatures
SBOM
provenance
VEX
evidence
public trust chain
policy snapshot
```

---

# 204. Air-Gap Import

Target Forgeyard instance validates bundle before registering.

---

# 205. No Secret Material

Air-gap release bundle contains no private signing/deploy credentials.

---

# 206. Release Backup

Metadata + CAS roots.

---

# 207. Release GC Root

Released/yanked releases pin required package/evidence objects according to retention.

---

# 208. Yanked Release Retention

Still retained for audit/history/rollback policy.

---

# 209. Candidate Retention

Failed/superseded candidates can expire later.

---

# 210. Release Notes Retention

Keep with release.

---

# 211. Release Metadata Store

Tables/entities:

```text
releases
release_candidates
release_packages
release_approvals
release_locks
release_channels
publications
publication_attempts
release_notes
release_events
```

---

# 212. CAS Objects

```text
release manifest
notes
public manifest
update feed
repository index
evidence
```

---

# 213. Publication Attempt

```rust
pub struct PublicationAttempt {
    pub id: PublicationAttemptId,
    pub publication: PublicationId,
    pub number: AttemptNumber,
    pub state: PublicationAttemptState,
}
```

---

# 214. Attempt History

Preserved.

---

# 215. Publication Retry

New attempt.

Do not erase failure history.

---

# 216. Provider Idempotency Key

Stable across retries for same semantic publication when provider supports.

---

# 217. Provider Rate Limits

Map to retry/backoff.

---

# 218. Provider Auth Failure

No retry until credential fixed.

---

# 219. Publication Error Model

```rust
pub enum PublishError {
    Authentication,
    Authorization,
    RateLimited,
    DestinationUnavailable,
    Conflict,
    InvalidArtifact,
    RemoteRejected,
    UnknownOutcome,
    Internal,
}
```

---

# 220. Conflict

Example:

```text
version already exists
```

Reconcile remote digest/ownership.

---

# 221. Existing Same Digest

Can treat as idempotent success if semantics match.

---

# 222. Existing Different Digest

Critical conflict.

Never overwrite silently.

---

# 223. Immutable Registry Version

If destination supports immutability, enable.

---

# 224. Mutable Destination

Forgeyard still records exact remote identity/digest.

---

# 225. Git Tag

VCS tag may be part of release publication.

---

# 226. Tag Identity

Tag should point to exact integration revision/source state.

---

# 227. Tag Signing

Optional/required policy.

---

# 228. Tag Creation Ordering

Recommended:

```text
candidate approved
  ↓
release lock
  ↓
create signed tag
  ↓
publish packages
```

or according to project/provider semantics.

---

# 229. Tag Failure

External publication state tracked.

---

# 230. No Release Authority from Tag Alone

Forgeyard release record remains authority.

---

# 231. Change Proposal Link

Release notes and provenance link integrated changes.

---

# 232. Release Source Range

Previous released source snapshot -> current source snapshot.

---

# 233. Changelog Calculation

VCS-neutral change graph.

---

# 234. Semantic Change Categories

Can come from proposal labels/metadata.

---

# 235. AI-Generated Release Notes

Could be optional UI helper later.

Not authority.

Must cite/source internal change records if used.

---

# 236. Release Note Editing

Final notes frozen into candidate/manifest.

---

# 237. Notes Change After Approval

Creates new candidate if notes digest is policy-relevant/manifest-bound.

Recommended:

```text
yes
```

because published metadata is part of release.

---

# 238. Cosmetic Metadata

Even if artifact bytes unchanged, release candidate semantic identity can change.

---

# 239. Release Candidate Hash Inputs

Include all externally meaningful release metadata.

---

# 240. Publication Metadata Mutation

Changing release title/notes after release should create audited metadata revision, not mutate historical candidate identity.

---

# 241. Release Metadata Revision

```rust
pub struct ReleaseMetadataRevisionId(Ulid);
```

Optional post-release metadata edits.

---

# 242. Artifact Immutability Remains

No package replacement.

---

# 243. Release Schedule

```rust
pub struct ReleaseSchedule {
    pub release: ReleaseId,
    pub promote_at: Timestamp,
}
```

---

# 244. Scheduled Promotion Timer

Durable.

---

# 245. Scheduled Candidate Change

Cancels/resets schedule.

---

# 246. Time Zone

Store timestamps in UTC; UI renders locale.

---

# 247. Publication Order

Can be configured:

```text
upload artifacts
publish manifests
switch channel pointer
```

---

# 248. Atomic Visibility

For destinations under Forgeyard control, use:

```text
upload hidden/staging
verify
atomically publish index/pointer
```

---

# 249. External Stores Without Atomicity

Model partial visibility risk.

---

# 250. Release Transaction vs External Calls

Never hold DB transaction around long publish call.

---

# 251. Desired-State Pattern

```text
persist Publication Pending
  ↓
worker acts
  ↓
record result
  ↓
reconcile
```

---

# 252. Unknown Outcome Pattern

```text
call timed out
  ↓
Publication Unknown
  ↓
inspect remote
  ↓
Succeeded / Failed / retry
```

---

# 253. Release Reconciler

Checks:

```text
Approved but no promotion worker
publication Unknown
Released but required destination missing
channel pointer mismatch
remote digest mismatch
expired release lock
stale evidence
```

---

# 254. Security Mismatch

If remote object digest differs:

```text
Critical
```

Do not republish over it blindly.

---

# 255. Release Health

Subsystem health:

```text
publisher availability
signing availability
destination reachability
reconcile backlog
stuck releases
```

---

# 256. Destination Health

Advisory.

Do not block candidate creation.

---

# 257. Metrics

```text
release_candidates_created
release_verification_duration
release_approval_wait
release_promotion_duration
release_publication_success
release_publication_failure
release_publication_unknown
release_partial
release_rollbacks
release_yanks
```

---

# 258. Channel Metrics

Low-cardinality by channel.

---

# 259. Destination Metrics

Low-cardinality by destination type.

---

# 260. Tracing

```text
release.freeze
release.verify
release.approve
release.lock
release.promote
release.publish
release.reconcile
release.rollback
```

---

# 261. Audit

Every high-risk action logged.

---

# 262. Doctor

```text
forgeyard release doctor
```

Checks:

```text
signing worker
publication adapters
credentials refs
channel config
policy
repository state
```

---

# 263. CLI

```text
forgeyard release create
forgeyard release candidate
forgeyard release verify
forgeyard release approve
forgeyard release reject
forgeyard release promote
forgeyard release publish
forgeyard release status
forgeyard release rollback
forgeyard release yank
forgeyard release notes
forgeyard release history
```

---

# 264. `release create`

Creates draft.

---

# 265. `release candidate`

Freezes exact package/evidence manifest.

---

# 266. `release verify`

No mutation by default.

---

# 267. `release approve`

Binds exact candidate.

---

# 268. `release promote`

Requires policy/lock.

---

# 269. `release rollback`

Explains destination capability before action.

---

# 270. Dioxus UI

Release page:

```text
Overview
Candidate
Packages
Evidence
Approvals
Publications
Channels
Notes
Timeline
Rollback
Audit
```

---

# 271. Candidate UI

Shows exact:

```text
package digest
target
signature
SBOM
provenance
reproducibility
```

---

# 272. Approval UI

Shows:

```text
candidate digest
policy
required approvers
current decisions
```

---

# 273. Publication UI

Per destination:

```text
Pending
InProgress
Succeeded
Failed
Unknown
```

---

# 274. Unknown UI

Must clearly show reconciliation in progress/required.

---

# 275. Release Timeline

Append-only event timeline.

---

# 276. API

Potential:

```text
POST /v1/releases
POST /v1/releases/{id}/candidate
POST /v1/releases/{id}/verify
POST /v1/releases/{id}/approve
POST /v1/releases/{id}/promote
GET  /v1/releases/{id}
GET  /v1/releases/{id}/publications
```

---

# 277. API Idempotency

Create/promote/publish operations accept idempotency keys.

---

# 278. Authorization Permissions

```text
release.read
release.create
release.approve
release.promote
release.rollback
release.yank
release.admin
```

---

# 279. Publish Permission

Can be distinct:

```text
release.publish
```

---

# 280. Production Channel

May require stronger:

```text
release.promote.stable
```

if permission registry chooses granularity.

---

# 281. Workload Identity

Publication worker gets only destination-specific credential/permission.

---

# 282. No General Human PAT in Publisher

Prefer service identity/app installation/short-lived credentials.

---

# 283. Release Testkit

```text
forgeyard-release-testkit/src/
├── lib.rs
├── candidate.rs
├── packages.rs
├── approvals.rs
├── policy.rs
├── publisher.rs
├── publication.rs
├── rollback.rs
└── assertions.rs
```

---

# 284. Unit Tests

Test:

```text
candidate digest
state transitions
approval binding
completeness
channel pointers
```

---

# 285. Candidate Mutation Test

Changing one package digest creates new candidate.

---

# 286. Approval Invalidation Test

Old candidate approval does not apply to new candidate.

---

# 287. No Rebuild Test

Release promotion path has no executor/build invocation.

---

# 288. Exact Byte Test

Published package digest equals verified package digest.

---

# 289. Multi-Platform Completeness Test

Missing required target blocks approval.

---

# 290. Optional Target Test

Optional package failure does not block if policy allows.

---

# 291. Signature Test

Invalid signature blocks release.

---

# 292. Evidence Freshness Test

Stale vulnerability scan blocks Stable if required.

---

# 293. Separation Test

Builder cannot satisfy independent release approver role if policy forbids.

---

# 294. Release Lock Concurrency Test

Two workers cannot promote conflicting candidates same version/channel.

---

# 295. Publication Duplicate Test

Same publish request idempotent.

---

# 296. Ambiguous Publication Test

Timeout after remote success -> Unknown -> inspect -> Succeeded.

---

# 297. Conflict Test

Remote same version different digest -> critical failure.

---

# 298. Channel CAS Test

Concurrent channel updates use expected version/CAS.

---

# 299. Rollback Test

Channel pointer returns to exact prior ReleaseId.

---

# 300. Yank Test

Artifact retained; new adoption disabled metadata.

---

# 301. App Store Limitation Test

Adapter reports unsupported true rollback.

---

# 302. Air-Gap Test

Release bundle imports/verifies offline.

---

# 303. Reconciliation Test

Dropped PublicationSucceeded event eventually repaired by remote inspect.

---

# 304. Provider Outage Test

Publication retries/backoffs without candidate mutation.

---

# 305. Credentials Test

Publisher receives only target-specific short-lived secret.

---

# 306. Fuzzing

Fuzz:

```text
release manifest parser
public release index
channel update input
publication metadata
```

---

# 307. Failure Injection

```text
network timeout
remote 500
rate limit
auth failure
DB failure
CAS unavailable
signer unavailable
```

---

# 308. Scale Tests

Large release with:

```text
many platforms
many artifacts
many publication destinations
```

---

# 309. Historical Verification Test

Years-old release remains verifiable after key rotation/user deletion.

---

# 310. Implementation Phase 1 — Release Model

Implement:

```text
ReleaseId
ReleaseCandidateId
ReleaseState
ReleaseManifest
PackageSet binding
```

---

# 311. Phase 2 — Candidate Freeze / Verify

Completeness + evidence verification.

---

# 312. Phase 3 — Approval

Policy/RBAC/separation integration.

---

# 313. Phase 4 — Release Lock / Channel

Atomic promotion metadata.

---

# 314. Phase 5 — Generic Publication

Download/object storage publisher.

---

# 315. Phase 6 — OCI / Linux Repos

Registry/repository adapters.

---

# 316. Phase 7 — GitHub/GitLab Release Views

Provider publication.

---

# 317. Phase 8 — Mobile Stores

Google Play/Apple adapters.

---

# 318. Phase 9 — Notes/Changelog

VCS/change integration.

---

# 319. Phase 10 — Rollback/Yank

Destination-aware reversal semantics.

---

# 320. Phase 11 — Reconciliation

Unknown/partial/drift recovery.

---

# 321. Phase 12 — Hardening

Air-gap, historical verification, HA, scale, fuzzing.

---

# 322. Acceptance Tests

1. Release candidate contains exact package digests.
2. Release candidate contains exact evidence bundle refs.
3. Release candidate is immutable after freeze.
4. Package replacement creates new ReleaseCandidateId.
5. Approval binds exact ReleaseCandidateId.
6. Candidate change invalidates old approvals.
7. Required package completeness is enforced.
8. Cross-target source consistency is enforced by default.
9. Required evidence is verified before approval.
10. Required signatures are verified before promotion.
11. Stale evidence can block promotion.
12. Release promotion never compiles/rebuilds source.
13. Published bytes equal verified package digest.
14. Signing/notarization lineage remains immutable.
15. Channel promotion reuses exact bytes.
16. Release locks prevent conflicting same-version promotion.
17. Multiple versions can release concurrently.
18. Required destination Unknown prevents final Released.
19. Optional destination failure can be tolerated by policy.
20. Publication retries are idempotent.
21. Ambiguous remote result is inspected before retry.
22. Existing same remote digest can become idempotent success.
23. Existing different digest is never overwritten silently.
24. Channel pointer update is compare-and-swap safe.
25. Rollback points to exact prior immutable ReleaseId.
26. Yank preserves artifacts/history.
27. Destination rollback capability is reported honestly.
28. Release notes are frozen/versioned with candidate semantics.
29. Change Proposal links use exact integrated revisions.
30. Publish worker has no build authority.
31. Publish worker receives scoped credentials only.
32. Air-gap release bundle verifies offline.
33. Historical release verifies after key rotation.
34. Same release semantics work standalone/distributed.
35. Forgeyard releases itself using this exact release system.

---

# 323. Production Readiness Gates

Do not call release subsystem production-ready until:

```text
candidate immutability proven
exact-digest package binding proven
evidence verification integrated
approval binding correct
release lock/concurrency tested
generic publication idempotent
Unknown outcome reconciliation works
channel pointer CAS works
rollback semantics documented
audit complete
no-rebuild invariant tested
```

Provider-specific app-store/repository adapters can reach readiness independently.

---

# 324. Architectural Invariants

1. a release promotes immutable artifacts;
2. release never rebuilds application code;
3. ReleaseCandidateId binds exact package/evidence inputs;
4. frozen candidate cannot mutate;
5. approvals bind exact candidate;
6. candidate replacement invalidates approval;
7. release version cannot silently repoint after release;
8. required target completeness is explicit;
9. supply-chain evidence is verified before protected promotion;
10. signature trust is re-evaluated as policy requires;
11. byte-changing signing/notarization already produced new artifact identity;
12. promotion does not alter package bytes;
13. channels are mutable pointers to immutable releases;
14. channel changes use version/CAS semantics;
15. publication is external side effect with explicit state;
16. Unknown publication outcome is reconciled before retry;
17. external same-version different-digest conflict is never overwritten silently;
18. publisher credentials are least privilege;
19. publisher has no build authority;
20. release locks are durable;
21. rollback capability is destination-specific and honest;
22. yanking does not erase history;
23. release notes/changelog are provenance-linked;
24. air-gap verification works from immutable evidence bundle;
25. public download metadata never replaces digest authority;
26. release policy reuses central policy engine;
27. release events are facts and reconciled;
28. standalone/distributed share release semantics;
29. release history remains auditable;
30. Forgeyard dogfoods its release system.

---

# 325. Final Target Architecture

```text
                 Release PackageSet
                        │
                        ▼
                  Candidate Freeze
                        │
                        ▼
               ReleaseCandidateId
                        │
         ┌──────────────┼──────────────┐
         ▼              ▼              ▼
   Completeness      Evidence        Policy
         │              │              │
         └──────────────┼──────────────┘
                        ▼
                  Await Approval
                        │
                        ▼
                     Approved
                        │
                        ▼
                   Release Lock
                        │
                        ▼
                     Promote
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
    Download Site      OCI Repo       App Store
        │               │                │
        └───────────────┼────────────────┘
                        ▼
                 Reconciled Result
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
         Released    Partial      Failed
```

---

# 326. Final Architectural Position

Candidate identity:

```text
ReleaseVersion
+
SourceSnapshotId
+
PackageSet
+
exact package digests
+
evidence bundle digests
+
release metadata
+
policy digest
  ↓
ReleaseCandidateId
```

Approval:

```text
ReleaseCandidateId
+
approver identity
+
policy digest
+
required evidence
  ↓
Approved
```

Promotion:

```text
Approved exact candidate
  ↓
release lock
  ↓
publish exact immutable package bytes
  ↓
verify/reconcile remote state
  ↓
Released
```

Rollback:

```text
current channel release
  ↓
known previous immutable ReleaseId
  ↓
destination-specific rollback operation
```

The key guarantee is:

> **Forgeyard release orchestration never asks "what should we build now?" It asks "which exact, already-built, verified, signed artifacts have been approved, and where should those exact bytes be made available?"**

---

# 327. New-Repository Sequence

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
