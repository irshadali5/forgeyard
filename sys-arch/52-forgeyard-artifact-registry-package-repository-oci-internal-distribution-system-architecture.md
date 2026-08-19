# 52 — Forgeyard Artifact Registry, Package Repository, OCI Distribution & Internal Software Distribution System Architecture

**Document type:** Core Artifact Registry, Package Repository, OCI Distribution, Internal Software Distribution & Repository Hosting System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** first-party artifact hosting, OCI registry, generic binary repositories, Rust crate repositories, npm/Python/JVM/Go-style package repositories, immutable package versions, mutable tags/channels, namespaces, access control, upload/download protocols, promotion, quarantine, signing/provenance integration, package indexes, proxy/cache separation, replication, retention, tenant isolation, air-gap distribution, and release-backed internal software distribution  
**Architecture style:** Immutable content, metadata-over-CAS, release/policy-governed promotion, standards-compatible edges, strict namespace isolation, digest-first identity, signed provenance, transport-neutral repository services, and no registry-side privilege escalation  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on CAS, Dependency/Registry Governance, Packaging, Release, Supply Chain, Security, Multi-Tenancy, Data Lifecycle, Federation, Update Distribution, Entitlements, API/Axum, Search/Analytics, and Audit. This subsystem turns Forgeyard from a CI/CD producer that publishes elsewhere into an optional software-distribution authority that can host its own trusted artifacts and packages.

---

## 1. Purpose

Forgeyard can already:

```text
build artifacts
package software
sign releases
publish to external destinations
mirror third-party dependencies
```

But many organizations also need an internal first-party distribution service for software they produce themselves.

Examples:

```text
OCI images
Rust crates
npm packages
Python wheels/sdists
Maven/JVM packages
Go modules
generic archives
firmware
desktop installers
mobile packages
SBOM/provenance attachments
```

A mature Forgeyard installation should be able to answer:

```text
where are our internal packages hosted?
who may publish a package?
can an untrusted job overwrite a release?
how do tags differ from immutable versions?
how is an OCI manifest represented?
how are signed artifacts promoted from staging to production?
how do we expose Cargo/npm/PyPI/Maven-compatible endpoints?
how do we replicate registries across regions?
how do we quarantine a compromised package?
how do consumers prove what bytes they downloaded?
```

The central rule is:

> **The Forgeyard registry stores, indexes, and distributes software; it does not decide that software is trustworthy. Trust is established by Release, Policy, Supply-Chain, Signing, and Dependency-Governance subsystems.**

A second rule is:

> **Published immutable versions are content-bound and cannot be silently overwritten. Mutable aliases such as tags and channels are pointers to immutable package identities, never package identity themselves.**

A third rule is:

> **Proxying external dependencies and hosting first-party software are separate trust domains even when they share storage primitives.**

---

# 2. Architectural Position

```text
                   Forgeyard Build/Package
                           │
                           ▼
                     Candidate Artifact
                           │
                           ▼
                 Policy / Release / Signing
                           │
                           ▼
                    Registry Promotion
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
           OCI          Packages       Generic
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                         CAS
                           │
                           ▼
                   Authorized Consumers
```

The registry is downstream of trusted release decisions.

---

# 3. Goals

The subsystem MUST:

1. define registry identity;
2. define repository identity;
3. define package namespace identity;
4. support immutable package versions;
5. support OCI images/artifacts;
6. support generic artifacts;
7. support Cargo-compatible distribution;
8. support npm-compatible distribution;
9. support PyPI-compatible distribution;
10. support Maven-compatible distribution;
11. support Go module distribution where practical;
12. support content-addressed storage;
13. support package metadata indexes;
14. support tags/channels;
15. support promotion;
16. support quarantine;
17. support yanking/deprecation;
18. support tenant/org/project namespaces;
19. support upload authorization;
20. support download authorization;
21. support short-lived credentials;
22. support signed/provenanced packages;
23. support SBOM/VEX attachments;
24. support replication;
25. support air-gap export/import;
26. support lifecycle/retention;
27. support search/discovery;
28. support audit;
29. support API/UI/CLI;
30. preserve trust separation.

---

# 4. Non-Goals

This subsystem does not:

```text
replace CAS
replace Packaging
replace Release
replace Dependency Governance
replace public registries
turn every ecosystem protocol into core domain semantics
allow package upload to bypass policy
```

---

# 5. Workspace Structure

```text
crates/registry/
├── forgeyard-registry/
├── forgeyard-registry-model/
├── forgeyard-registry-repository/
├── forgeyard-registry-publish/
├── forgeyard-registry-fetch/
├── forgeyard-registry-promotion/
├── forgeyard-registry-quarantine/
├── forgeyard-registry-index/
├── forgeyard-registry-auth/
├── forgeyard-registry-replication/
├── forgeyard-registry-health/
└── forgeyard-registry-testkit/
```

Protocol adapters:

```text
crates/registry-protocols/
├── forgeyard-registry-oci/
├── forgeyard-registry-cargo/
├── forgeyard-registry-npm/
├── forgeyard-registry-pypi/
├── forgeyard-registry-maven/
├── forgeyard-registry-go/
└── forgeyard-registry-generic/
```

Core registry crates remain ecosystem-neutral.

---

# 6. RegistryId

```rust
pub struct RegistryId(Ulid);
```

Represents one logical registry service.

---

# 7. RepositoryId

Do not reuse VCS `RepositoryId`.

Use:

```rust
pub struct ArtifactRepositoryId(Ulid);
```

---

# 8. Repository Kind

```rust
pub enum ArtifactRepositoryKind {
    Oci,
    Cargo,
    Npm,
    Python,
    Maven,
    Go,
    Generic,
    Firmware,
    Custom(ArtifactRepositoryKindId),
}
```

---

# 9. Repository Scope

```rust
pub enum ArtifactRepositoryScope {
    Installation,
    Tenant(TenantId),
    Organization(OrganizationId),
    Project(ProjectId),
}
```

---

# 10. Artifact Repository

```rust
pub struct ArtifactRepository {
    pub id: ArtifactRepositoryId,
    pub kind: ArtifactRepositoryKind,
    pub scope: ArtifactRepositoryScope,
    pub name: ArtifactRepositoryName,
    pub visibility: RepositoryVisibility,
}
```

---

# 11. Visibility

```rust
pub enum RepositoryVisibility {
    Private,
    Organization,
    Installation,
    Public,
}
```

Visibility affects read access, not publish authority.

---

# 12. Package Namespace

```rust
pub struct PackageNamespace(BoundedString);
```

Examples:

```text
com.example
@org
org/project
internal
```

Namespace rules are ecosystem-specific at adapter edges.

---

# 13. PackageName

```rust
pub struct PackageName(BoundedString);
```

---

# 14. PackageCoordinate

```rust
pub struct HostedPackageCoordinate {
    pub repository: ArtifactRepositoryId,
    pub namespace: Option<PackageNamespace>,
    pub name: PackageName,
}
```

---

# 15. PackageVersionId

```rust
pub struct PackageVersionId(Digest);
```

Immutable exact version identity.

---

# 16. Version String

Human/ecosystem metadata.

```rust
pub struct PackageVersionString(BoundedString);
```

---

# 17. Immutable Package Version

```rust
pub struct HostedPackageVersion {
    pub id: PackageVersionId,
    pub coordinate: HostedPackageCoordinate,
    pub version: PackageVersionString,
    pub artifact: CasObjectRef,
    pub digest: DigestSet,
    pub state: PackageVersionState,
}
```

---

# 18. PackageVersionState

```rust
pub enum PackageVersionState {
    Uploaded,
    Verified,
    Candidate,
    Promoted,
    Yanked,
    Quarantined,
    Revoked,
}
```

---

# 19. No Overwrite

Once an ecosystem version is published/promoted, the same version string cannot point to different bytes within the same repository namespace unless an ecosystem explicitly requires different semantics and Forgeyard can preserve safe identity.

Default:

```text
same coordinate + version
+
different digest
=
equivocation/conflict
```

---

# 20. Equivocation

```rust
pub struct PackageEquivocation {
    pub coordinate: HostedPackageCoordinate,
    pub version: PackageVersionString,
    pub existing: Digest,
    pub attempted: Digest,
}
```

This is security-significant.

---

# 21. Digest Identity

Internal canonical digest:

```text
BLAKE3
```

Interoperability aliases:

```text
SHA-256
OCI digest
ecosystem-native checksum
```

---

# 22. Registry Blob Storage

All immutable bytes stored in CAS.

Registry metadata stores:

```text
coordinate
version
digest aliases
state
provenance refs
index metadata
visibility
```

---

# 23. Metadata/CAS Separation

Existing invariant.

---

# 24. Publish Operation

```rust
pub struct RegistryPublishRequest {
    pub repository: ArtifactRepositoryId,
    pub coordinate: HostedPackageCoordinate,
    pub version: PackageVersionString,
    pub package: PackageId,
    pub actor: PrincipalId,
}
```

---

# 25. Upload vs Publish

Separate.

```text
upload bytes
  ↓
verify
  ↓
candidate
  ↓
policy/release evaluation
  ↓
promote/publish
```

---

# 26. Raw Upload

Does not create trusted published package.

---

# 27. Upload Session

```rust
pub struct RegistryUploadSessionId(Ulid);
```

---

# 28. Upload Session State

```rust
pub enum UploadSessionState {
    Created,
    Uploading,
    Uploaded,
    Verifying,
    Completed,
    Failed,
    Expired,
}
```

---

# 29. Resumable Upload

Supported for large packages/images.

---

# 30. Chunk Verification

Where protocol permits.

---

# 31. Final Digest Verification

Mandatory.

---

# 32. Upload Credentials

Short-lived/scoped.

---

# 33. No General Registry Write Token in Build Job

Preferred model:

```text
build job
  ↓
produces PackageId/ArtifactId
  ↓
release/publish service
  ↓
registry write
```

---

# 34. Direct Job Publish

Only explicitly permitted low-risk repositories.

---

# 35. Release-Backed Promotion

For protected production package repositories:

```text
ReleaseCandidate
  ↓
Release approval
  ↓
sign
  ↓
RegistryPromotion
```

---

# 36. RegistryPromotionId

```rust
pub struct RegistryPromotionId(Ulid);
```

---

# 37. Promotion

```rust
pub struct RegistryPromotion {
    pub id: RegistryPromotionId,
    pub package: PackageVersionId,
    pub source_state: PackageVersionState,
    pub target_repository: ArtifactRepositoryId,
    pub release: Option<ReleaseId>,
    pub policy: PolicyDigest,
}
```

---

# 38. Promotion Does Not Rebuild

Critical.

---

# 39. Promotion Does Not Mutate Bytes

Same digest.

---

# 40. Cross-Repository Promotion

Allowed when policy permits.

Example:

```text
staging registry
  ↓
production registry
```

---

# 41. Metadata Promotion

May create new repository reference to same CAS object.

---

# 42. Channel/Tag

```rust
pub struct RegistryAlias {
    pub repository: ArtifactRepositoryId,
    pub alias: RegistryAliasName,
    pub target: PackageVersionId,
}
```

---

# 43. Alias Examples

```text
latest
stable
beta
1
1.2
production
```

---

# 44. Alias Is Mutable

PackageVersionId is immutable.

---

# 45. Alias Update

Audited/policy-governed for protected repos.

---

# 46. OCI Tags

Map naturally to aliases.

---

# 47. OCI Digest

Authoritative content identity.

---

# 48. OCI Support

Implement standards-compatible endpoints.

Support:

```text
blobs
manifests
indexes
tags
referrers
```

where relevant.

---

# 49. OCI Artifact Types

Can host:

```text
container images
Helm-like OCI artifacts
SBOMs
signatures
provenance
generic OCI artifacts
```

---

# 50. OCI Manifest Identity

SHA-256 compatibility alias retained.

---

# 51. Multi-Architecture Images

OCI index/manifests.

---

# 52. Exact Platform

Each manifest references exact blob digests.

---

# 53. Referrers

Useful for:

```text
SBOM
signature
attestation
VEX
provenance
```

---

# 54. Supply-Chain Evidence

Part 13 remains authority for evidence semantics.

Registry stores/distributes attachments.

---

# 55. Cargo Registry

Support internal crate distribution.

Potential architecture:

```text
crate archive bytes in CAS
+
index metadata
+
checksum
```

---

# 56. Cargo Sparse Index

Preferred modern compatible edge.

---

# 57. Cargo Checksums

Preserve ecosystem-native checksum.

---

# 58. Cargo Yank

Maps to package version state `Yanked`.

---

# 59. Cargo Publish Permission

Namespace/package owner policy.

---

# 60. npm Registry

Support:

```text
package metadata
tarball
dist-tags
versions
```

---

# 61. npm dist-tags

Map to mutable aliases.

---

# 62. npm Immutable Version

No overwrite.

---

# 63. npm Scripts

Registry hosts bytes; execution occurs later in sandboxed dependency/build stages.

---

# 64. Python/PyPI

Support:

```text
simple index
wheel/sdist files
metadata
hashes
```

---

# 65. Wheel Preferred

But registry can host sdist.

---

# 66. sdist Trust

Execution risk handled by Part 36/build sandbox.

---

# 67. Maven

Support:

```text
group/artifact/version
POM
JAR
metadata
checksums
```

---

# 68. Snapshot Versions

Need explicit policy.

---

# 69. Maven SNAPSHOT

Mutable ecosystem semantics are risky.

Recommended:

```text
development repository only
```

Protected release repositories require immutable resolved artifact identity.

---

# 70. Go Modules

Can support proxy-style module serving.

---

# 71. Go Version

Exact module version + checksum.

---

# 72. Generic Repository

For:

```text
zip
tar.gz
binary
firmware
installer
symbols
debug bundles
```

---

# 73. GenericArtifactDescriptor

```rust
pub struct GenericArtifactDescriptor {
    pub name: BoundedString,
    pub version: BoundedString,
    pub media_type: MediaType,
    pub digest: DigestSet,
}
```

---

# 74. Repository Namespace Governance

Critical to prevent name confusion.

---

# 75. NamespaceClaim

```rust
pub struct NamespaceClaim {
    pub repository: ArtifactRepositoryId,
    pub namespace: PackageNamespace,
    pub owner: ResourceScope,
}
```

---

# 76. Private Namespace Binding

Similar to Part 36 dependency confusion defenses.

---

# 77. No Public Fallback

A known private namespace must never silently resolve from public registry.

---

# 78. Internal Package Resolution

Consumers can configure Forgeyard registry endpoint.

---

# 79. Dependency Governance Integration

Part 36 decides whether dependency is approved.

Registry availability does not imply approval.

---

# 80. First-Party vs Mirrored Third-Party

Separate repositories/namespaces.

Recommended:

```text
internal-release
internal-dev
third-party-mirror
quarantine
```

---

# 81. Do Not Mix Trust States

Critical.

---

# 82. Proxy Repository

If Forgeyard exposes proxy/cache endpoints for public registries:

```text
external fetch
  ↓
verify
  ↓
mirror/cache
```

This is Part 36 behavior surfaced through repository protocol.

---

# 83. Proxy Presence != Approval

Existing invariant.

---

# 84. Hosted Repository

First-party authoritative package publication.

---

# 85. RepositoryMode

```rust
pub enum RepositoryMode {
    Hosted,
    Proxy,
    Group,
}
```

---

# 86. Group Repository

Read-only aggregate over multiple repositories.

---

# 87. Group Precedence

Explicit.

---

# 88. Dependency Confusion Defense

Private hosted repositories first for private namespace.

No public fallback unless policy explicitly permits.

---

# 89. Group Write

Forbidden.

---

# 90. Publish Target

Must be explicit hosted repository.

---

# 91. Quarantine Repository

Restricted.

---

# 92. Quarantine State

Package cannot be normal dependency.

---

# 93. Quarantine Reasons

```text
signature invalid
malware
vulnerability
license
equivocation
manual security hold
provenance missing
```

---

# 94. Quarantine Does Not Delete

Critical.

---

# 95. Revocation

Stronger security state.

---

# 96. Revoked Package

New download/use can be blocked by policy.

---

# 97. Existing Historical Releases

Remain historically referential.

---

# 98. Yanking

Prevents normal new resolution but does not erase bytes/history.

---

# 99. Deprecation

Advisory metadata.

---

# 100. Package Ownership

Separate from authz.

---

# 101. PackageOwner

May route reviews/notifications.

---

# 102. Publish Authorization

Part 11 permissions/policy.

---

# 103. Example Permissions

```text
registry.read
registry.publish
registry.promote
registry.yank
registry.quarantine
registry.admin
registry.alias.manage
```

---

# 104. Protected Repository

Requires release service identity for publish/promote.

---

# 105. Human Direct Publish

Disabled by default for production repositories.

---

# 106. Service Identity

Preferred.

---

# 107. Workload Identity

Short-lived.

---

# 108. Registry Tokens

```rust
pub struct RegistryTokenScope {
    pub repository: ArtifactRepositoryId,
    pub action: RegistryAction,
    pub expires_at: Timestamp,
}
```

---

# 109. Token Storage

Hashed or delegated auth token.

---

# 110. OAuth/OIDC

Can integrate.

---

# 111. CLI Credentials

Short-lived where possible.

---

# 112. Ecosystem Clients

May require basic/bearer/token formats.

Adapter maps to Forgeyard auth.

---

# 113. No Permanent Shared Publish Password

Critical.

---

# 114. Download Authorization

Can be:

```text
public
tenant-scoped
org-scoped
project-scoped
principal/service
```

---

# 115. Presigned Downloads

Short-lived.

---

# 116. CAS Authorization

Digest possession is not authorization.

Existing invariant.

---

# 117. Package Metadata Index

Derived/rebuildable from registry metadata.

---

# 118. Index Requirements

```text
coordinate lookup
version listing
alias/tag resolution
search
dependency metadata
```

---

# 119. Search

Part 31.

---

# 120. Search Fields

```text
name
namespace
version
ecosystem
owner
release
SBOM status
security state
```

---

# 121. Search Authorization

Strict.

---

# 122. Private Package Enumeration

Do not leak names/version existence.

---

# 123. Package Detail

Can show:

```text
digest
release
provenance
SBOM
signature
download commands
dependencies
security state
```

---

# 124. Dioxus UI

Pages:

```text
Registries
Repositories
Packages
Versions
OCI Images
Quarantine
Promotions
Replication
```

---

# 125. Package Version Page

Shows exact immutable identity.

---

# 126. Alias Page

Shows mutable tag/channel history.

---

# 127. Promotion UI

Protected action.

---

# 128. Quarantine UI

Security/admin.

---

# 129. CLI

```text
forgeyard registry list
forgeyard registry repo list
forgeyard registry package show
forgeyard registry publish
forgeyard registry promote
forgeyard registry yank
forgeyard registry quarantine
forgeyard registry alias set
forgeyard registry doctor
```

---

# 130. Ecosystem Native Clients

Also supported:

```text
docker/podman
cargo
npm/pnpm/yarn
pip/uv
maven/gradle
go
```

where protocol adapter exists.

---

# 131. API

Potential native admin API:

```text
GET  /v1/registries
GET  /v1/repositories
GET  /v1/packages
POST /v1/packages/{id}/promote
POST /v1/packages/{id}/yank
POST /v1/packages/{id}/quarantine
```

Protocol-specific compatibility endpoints remain separate.

---

# 132. Public vs Native API

Do not force ecosystem clients through Forgeyard REST model.

Use standards-compatible protocol edges.

---

# 133. Protocol Parser Safety

Registry endpoints parse untrusted input.

Bound:

```text
manifest size
metadata size
name/version length
archive size
request body
header count
```

---

# 134. OCI Upload Safety

Chunk/body limits.

---

# 135. Package Archive Safety

Do not unpack merely to store.

---

# 136. Metadata Inspection

If unpacking for validation:

```text
sandbox
path traversal defense
zip bomb limits
symlink/device entry checks
```

---

# 137. Malware/Security Scan

Can happen asynchronously after upload.

---

# 138. Protected Promotion

Requires scan/policy evidence as configured.

---

# 139. Scan Failure

Does not become "clean".

---

# 140. Analysis Completeness

Part 37.

---

# 141. SBOM

Can be generated at build time.

---

# 142. Registry

Links SBOM to exact package digest.

---

# 143. Provenance

Links exact package to:

```text
SourceSnapshotId
PipelinePlanId
RunId
JobId
PackageId
ReleaseId
```

where available.

---

# 144. Signature

Exact digest.

---

# 145. Signature Verification

Registry can verify on promotion.

---

# 146. Registry Is Not Signing Authority

Critical.

---

# 147. OCI Signature/Attestation

Store as OCI referrer or generic evidence object.

---

# 148. Package Promotion Policy

Inputs may include:

```text
release status
signature
SBOM
vulnerability
license
test evidence
reproducibility
```

---

# 149. Policy Remains Part 11

Registry applies decision.

---

# 150. No Registry-Specific Shadow Policy Engine

Critical.

---

# 151. Repository Policy

Can reference central policy bundle.

---

# 152. Retention

Part 46.

---

# 153. Retention Classes

Examples:

```text
development snapshots
promoted release package
yanked package
quarantined package
OCI layer
orphan upload
```

---

# 154. Release Package

Long-lived.

---

# 155. Orphan Upload

Short-lived cleanup.

---

# 156. OCI Shared Layer

Physical blob retained while any manifest references.

---

# 157. CAS Roots

Registry metadata creates roots.

---

# 158. Delete Package Version

High-risk.

---

# 159. Default

Yank/deprecate instead of delete.

---

# 160. Physical Delete

Only lifecycle subsystem after no retained refs/holds.

---

# 161. Legal Hold

Can pin package bytes.

---

# 162. Quarantine

May extend retention.

---

# 163. Replication

Part 51.

---

# 164. Repository Replication Policy

```rust
pub enum RegistryReplicationPolicy {
    Local,
    Regional(u8),
    AllTrustedSites,
    OnDemand,
    AirGapOnly,
}
```

---

# 165. Replication

Copies immutable blobs + authoritative metadata projections.

---

# 166. Mutable Alias Authority

One authority domain.

---

# 167. Avoid Tag Split-Brain

Critical.

---

# 168. Regional Pull

Can serve local verified blob.

---

# 169. Metadata Freshness

Alias/tag resolution may require authoritative or bounded-stale read depending use.

---

# 170. Protected Deployment Pull

Prefer digest pin, not mutable tag.

---

# 171. Air-Gap Registry Bundle

```rust
pub struct RegistryOfflineBundle {
    pub repositories: Vec<ArtifactRepositoryRef>,
    pub packages: Vec<PackageVersionId>,
    pub manifests: CasObjectRef,
    pub signatures: Vec<SignatureRef>,
}
```

---

# 172. Export

Can bundle selected dependency/application closure.

---

# 173. Import

Verify:

```text
manifest
digests
signatures
repository scope
policy
```

---

# 174. Offline Consumer

Can use local registry endpoint backed by imported bundle.

---

# 175. Federation

Air-gapped site can host a local replica.

---

# 176. Package Pull Through CDN

For public artifacts.

---

# 177. CDN Is Transport Only

Same trust principle Part 41.

---

# 178. Immutable URLs

Digest/version based.

---

# 179. Alias URL

Mutable metadata.

---

# 180. HTTP Caching

Safe for immutable blobs.

---

# 181. ETag

Digest.

---

# 182. Range Requests

Useful for large blobs.

---

# 183. Download Resume

Supported.

---

# 184. Partial Download

Never considered complete until digest verified.

---

# 185. Garbage Collection

Registry references feed CAS GC.

---

# 186. OCI Blob Dedup

Natural via CAS.

---

# 187. Cross-Package Dedup

Internal CAS may dedup identical bytes.

---

# 188. Tenant Authorization

Physical dedup does not create cross-tenant access.

Critical.

---

# 189. Logical Accounting

Part 45.

---

# 190. Physical Storage Cost

CAS/provider cost.

---

# 191. Logical Usage

Per tenant/repository package refs.

---

# 192. Quotas

Part 27.

---

# 193. Registry Quotas

Potential:

```text
storage bytes
package count
version count
egress
upload bandwidth
```

---

# 194. Quota Does Not Delete Existing Release Artifacts Automatically

---

# 195. Budget

Part 45 can notify/guard optional dev storage.

---

# 196. Entitlement

Part 30 may gate:

```text
private registries
retention tiers
geo-replication
```

---

# 197. Security Baseline

Never paywalled:

```text
digest verification
tenant isolation
authz
audit
signature support
```

---

# 198. Registry Health

```rust
pub enum RegistryHealth {
    Healthy,
    ReadOnly,
    StorageDegraded,
    IndexDegraded,
    ReplicationDegraded,
    Unhealthy,
}
```

---

# 199. ReadOnly Mode

Can serve downloads while publish disabled.

---

# 200. CAS Unavailable

Downloads of cached local objects may work if safe.

---

# 201. Metadata DB Unavailable

Do not invent alias resolution.

---

# 202. Degraded Behavior

Typed.

---

# 203. Doctor

```text
forgeyard registry doctor
```

Checks:

```text
CAS integrity
repository metadata
alias consistency
orphan uploads
equivocation
replication lag
token config
protocol endpoint health
```

---

# 204. Observability Metrics

```text
registry_uploads_total
registry_downloads_total
registry_bytes_uploaded_total
registry_bytes_downloaded_total
registry_publish_failures_total
registry_quarantined_versions_total
registry_equivocation_total
registry_replication_backlog_bytes
```

---

# 205. Labels

Low-cardinality:

```text
repository_kind
operation
result
```

Avoid package-name cardinality in metrics.

---

# 206. Tracing

```text
registry.upload
registry.verify
registry.publish
registry.promote
registry.fetch
registry.alias
registry.replicate
registry.quarantine
```

---

# 207. Audit

Audit:

```text
repository create/delete
publish protected package
promotion
alias/tag change
yank
quarantine/revoke
namespace ownership change
token/admin change
```

---

# 208. Ordinary public download

Not privileged audit per request.

---

# 209. Notifications

Examples:

```text
package equivocation detected
promotion failed
replication degraded
namespace conflict
quarantine activated
```

---

# 210. Search/Analytics

Part 31.

---

# 211. Useful Analytics

```text
downloads
active package versions
deprecated package usage
storage
replication
```

---

# 212. Privacy

Package download telemetry should not become developer surveillance.

---

# 213. Consumption Evidence

For dependency governance, dependency resolver records exact package identity/digest.

Registry itself need not build exhaustive per-user tracking.

---

# 214. Namespace Transfer

High-risk.

---

# 215. NamespaceTransferPlan

```rust
pub struct NamespaceTransferPlan {
    pub namespace: PackageNamespace,
    pub from: ResourceScope,
    pub to: ResourceScope,
}
```

---

# 216. Existing Versions

Remain historically attributed.

---

# 217. New Publish Rights

Transfer after approval.

---

# 218. No Name Takeover

Tombstones/reservations prevent deleted namespace hijacking where policy requires.

---

# 219. Package Rename

Usually publish new coordinate.

---

# 220. Alias Migration

Explicit.

---

# 221. Repository Deletion

High-risk.

---

# 222. Default

Archive/read-only first.

---

# 223. RepositoryLifecycle

```rust
pub enum RepositoryLifecycle {
    Active,
    ReadOnly,
    Archived,
    Retired,
}
```

---

# 224. Retired

No normal publish.

---

# 225. Historical downloads

According lifecycle policy.

---

# 226. Registry Backup

Metadata DB backed up.

---

# 227. CAS

Independent durability/backup.

---

# 228. Replica Is Not Backup

Existing invariant.

---

# 229. DR

Restore metadata and verify CAS roots.

---

# 230. Alias History

Retain for audit.

---

# 231. Equivocation History

Security evidence.

---

# 232. Index Rebuild

Derived.

---

# 233. Protocol Index Rebuild

From canonical metadata.

---

# 234. No Ecosystem Index as Sole Truth

Critical.

---

# 235. OCI Tag Index

Derived from aliases.

---

# 236. Cargo Sparse Index

Derived.

---

# 237. npm Metadata

Derived.

---

# 238. PyPI Simple Index

Derived.

---

# 239. Maven Metadata

Derived.

---

# 240. Go Proxy Metadata

Derived.

---

# 241. Standards Compatibility

External protocols should be implemented at adapter edges.

Core uses normalized domain types.

---

# 242. JSON/XML

Use where ecosystem requires.

---

# 243. RON/Postcard

Internal/config/wire where appropriate.

---

# 244. No Forced Postcard on Public Ecosystem Protocols

Critical.

---

# 245. API Versioning

Protocol adapters track ecosystem compatibility.

---

# 246. Feature Capability Matrix

Some ecosystem behaviors may be unsupported.

---

# 247. Honest Compatibility

Expose limits.

---

# 248. Migration

Part 47 can import from:

```text
Harbor
Artifactory
Nexus
GitHub Packages
GitLab Package Registry
ECR/GCR/ACR
private npm/PyPI
```

later via adapters.

---

# 249. Migration Rules

Imported package version preserves exact digest/source trust state.

---

# 250. Imported Package Trust

```rust
pub enum PackageOriginTrust {
    ForgeyardBuilt,
    ForgeyardReleased,
    ExternalImported,
    MirroredThirdParty,
}
```

---

# 251. Imported != Trusted Release

Critical.

---

# 252. Registry as Deployment Source

Deployment should pin exact digest/PackageVersionId.

---

# 253. No Production Tag-Only Deployment

Recommended policy.

---

# 254. Registry as Update Source

Part 41 can consume exact release packages from registry.

---

# 255. Self-Hosting

Forgeyard can host its own:

```text
CLI binaries
daemon packages
container images
SBOM/provenance
```

---

# 256. Bootstrap Escape Path

Still preserve external/manual signed release path.

Registry must not be the only way to recover Forgeyard.

---

# 257. Testkit

```text
forgeyard-registry-testkit/src/
├── lib.rs
├── repository.rs
├── upload.rs
├── publish.rs
├── alias.rs
├── promotion.rs
├── quarantine.rs
├── protocol.rs
└── assertions.rs
```

---

# 258. Unit Tests

Coordinate/version identity.

---

# 259. Immutable Version Test

Same version/different bytes rejected.

---

# 260. Alias Test

Alias can move without mutating version.

---

# 261. Promotion Test

Exact digest preserved.

---

# 262. Release Policy Test

Unreleased candidate cannot enter protected production repo.

---

# 263. Direct Job Publish Test

Denied by default protected repo.

---

# 264. Token Scope Test

Cannot publish outside repository.

---

# 265. Tenant Isolation Test

Cross-tenant digest access denied.

---

# 266. Dedup Test

Same physical blob does not imply cross-tenant authorization.

---

# 267. Quarantine Test

Normal resolver cannot consume.

---

# 268. Yank Test

Historical digest remains.

---

# 269. Equivocation Test

Security event emitted.

---

# 270. OCI Test

Manifest/blob/tag/referrer compatibility.

---

# 271. Cargo Test

Sparse index/checksum/yank.

---

# 272. npm Test

versions/dist-tags/tarball.

---

# 273. PyPI Test

simple index/hashes.

---

# 274. Maven Test

immutable releases/SNAPSHOT policy.

---

# 275. Group Repository Test

Resolution precedence deterministic.

---

# 276. Dependency Confusion Test

Private namespace cannot fall back publicly.

---

# 277. Protocol Parser Test

Hostile metadata bounded.

---

# 278. Large Upload Test

Resume/digest.

---

# 279. Partial Upload Test

Never published.

---

# 280. Replication Test

Destination digest verified.

---

# 281. Alias Split-Brain Test

Federation authority prevents concurrent tag changes.

---

# 282. Lifecycle Test

Referenced OCI layer not GC'd.

---

# 283. DR Test

Metadata restored/index rebuilt.

---

# 284. Air-Gap Test

Bundle import/serve offline.

---

# 285. Fuzzing

Fuzz:

```text
OCI manifests
npm metadata
PyPI metadata
Maven metadata
Cargo index metadata
generic upload descriptors
```

---

# 286. Scale Test

Millions of versions/blobs.

---

# 287. Failure Injection

```text
CAS outage
DB restart
partial upload
replication timeout
index rebuild failure
token service unavailable
```

---

# 288. Implementation Phase 1 — Generic Registry Model

Hosted generic artifacts.

---

# 289. Phase 2 — OCI

Highest-value interoperable standard.

---

# 290. Phase 3 — Promotion/Quarantine/Policy

Trusted distribution.

---

# 291. Phase 4 — Cargo Registry

Dogfood Rust ecosystem.

---

# 292. Phase 5 — npm/PyPI

Broad ecosystem.

---

# 293. Phase 6 — Maven/Go

Enterprise/polyglot.

---

# 294. Phase 7 — Proxy/Group Repositories

Integrate Part 36.

---

# 295. Phase 8 — Federation/Replication

Part 51.

---

# 296. Phase 9 — Air-Gap Bundles

Enterprise.

---

# 297. Phase 10 — Search/UI/Analytics

Usability.

---

# 298. Phase 11 — Migration Imports

Legacy registry adoption.

---

# 299. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 300. Acceptance Tests

1. Registry bytes live in CAS; metadata remains separate.
2. Published immutable package versions cannot be overwritten.
3. Same coordinate/version with different bytes is detected as equivocation.
4. Mutable aliases/tags never become immutable package identity.
5. Upload and trusted publish are separate states.
6. Protected repositories require policy/release-backed promotion.
7. Promotion never rebuilds or mutates package bytes.
8. Build jobs do not receive broad permanent publish credentials.
9. Registry trust does not replace Release/Policy/Supply-Chain trust.
10. OCI digests remain exact content identity.
11. SBOM/provenance/signature attachments bind exact package digest.
12. Registry stores signatures but is not signing authority.
13. First-party hosted and third-party mirrored repositories remain separate trust domains.
14. Proxy presence never implies dependency approval.
15. Group repositories are read-only aggregates.
16. Private namespace resolution cannot silently fall back to public sources.
17. Quarantined packages cannot satisfy normal dependency resolution.
18. Yanking does not erase historical package bytes.
19. Physical dedup never grants cross-tenant access.
20. Ecosystem indexes are derived/rebuildable, not canonical truth.
21. Public ecosystem protocols use their required formats rather than forcing RON/Postcard.
22. Protected production deployment uses exact digest/PackageVersionId rather than mutable tag alone.
23. Repository deletion defaults to archive/read-only before destructive removal.
24. Registry lifecycle integrates with CAS roots/legal holds.
25. Regional replicas verify every digest.
26. Mutable aliases have one federation authority domain.
27. Air-gap bundles verify and serve without internet.
28. Imported external packages retain explicit origin/trust state.
29. Registry tokens are short-lived/scoped where possible.
30. Protocol parsers are hardened against hostile metadata.
31. Standalone can host a local internal registry.
32. Distributed mode supports multi-tenant private registries.
33. DR restores metadata and rebuilds indexes.
34. Forgeyard update delivery can consume registry-hosted exact releases without making registry sole recovery path.
35. Forgeyard dogfoods the registry for its own OCI images, Rust crates, binaries, SBOMs, provenance, and release packages.

---

# 301. Production Readiness Gates

Do not call the artifact registry production-ready until:

```text
immutable version enforcement passes
equivocation detection is reliable
tenant authorization is enforced at blob access
upload/publish separation is complete
protected promotion integrates Release/Policy
OCI compatibility passes
at least one language registry is dogfooded
private namespace confusion tests pass
CAS GC/root integration is safe
DR/index rebuild/federation replication tests pass
```

---

# 302. Architectural Invariants

1. registry distributes software; it does not invent trust;
2. immutable package versions cannot be overwritten;
3. aliases/tags are mutable pointers only;
4. digest identity outranks version/tag;
5. upload is not trusted publication;
6. protected promotion is policy/release governed;
7. promotion never rebuilds;
8. CAS stores bytes, registry metadata stores relationships/state;
9. physical dedup does not grant authorization;
10. package equivocation is security-significant;
11. first-party hosted and mirrored third-party trust remain separate;
12. proxy presence does not equal dependency approval;
13. private namespace fallback is controlled;
14. quarantined packages are not normally consumable;
15. yanking preserves history;
16. registry can verify signatures but is not signing authority;
17. evidence attachments bind exact package digest;
18. ecosystem indexes are derived;
19. ecosystem-native protocol formats remain at edges;
20. protected deployments pin exact package identity;
21. registry tokens are scoped/short-lived where possible;
22. broad permanent build-job publish credentials are forbidden by default;
23. lifecycle/holds govern deletion;
24. replicas verify digests;
25. mutable alias authority is single-writer in federation;
26. air-gap registry operation is supported;
27. imported packages retain origin/trust provenance;
28. registry does not become sole Forgeyard recovery path;
29. standalone/distributed share trust semantics;
30. Forgeyard dogfoods its own registry.

---

# 303. Final Target Architecture

```text
                    Build / Package
                         │
                         ▼
                 Immutable Package
                         │
                         ▼
             Verify / Scan / Provenance
                         │
                         ▼
                Release / Policy Gate
                         │
                         ▼
                  Registry Promotion
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
            OCI        Language     Generic
          Registry     Packages     Artifacts
             │           │           │
             └───────────┼───────────┘
                         ▼
                        CAS
```

Publication:

```text
PackageId
+
Release/Policy evidence
+
exact digest
  ↓
PackageVersionId
  ↓
promoted repository reference
```

Consumption:

```text
package coordinate / alias
  ↓
authorized resolution
  ↓
exact immutable PackageVersionId/digest
  ↓
download
  ↓
digest verification
```

Promotion:

```text
candidate package bytes
  ↓
policy/release approval
  ↓
metadata promotion
  ↓
same bytes
```

The key guarantee is:

> **Forgeyard can host and distribute its own software across OCI and language-native package ecosystems without turning the registry into a second release authority. Packages are immutable, digest-bound, policy-governed, provenance-linked, tenant-isolated, and promoted as the same bytes that were already built and verified.**

---

# 304. Extended Architecture Sequence

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
```
