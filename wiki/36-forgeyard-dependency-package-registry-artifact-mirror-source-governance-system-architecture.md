# 36 — Forgeyard Dependency, Package Registry, Artifact Mirror & Software-Source Governance System Architecture

**Document type:** Core Dependency Acquisition, Registry Proxy, Package Mirror, Source Governance & Trusted Dependency Promotion System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** dependency resolution, package registry adapters, internal mirrors/proxies, vendoring, lockfile enforcement, package provenance, checksum/signature verification, malicious-package defenses, dependency allow/deny policy, source promotion, artifact mirrors, offline/air-gapped dependency workflows, registry trust, cache poisoning resistance, and software-source governance  
**Architecture style:** Resolve once, pin exactly, verify before use, separate external/untrusted acquisition from trusted internal consumption, mirror immutably, make dependency provenance explicit, and never let registries become hidden mutable build inputs  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Hermetic/Reproducible Build System, CAS, Supply Chain/SBOM/Provenance, Security/Policy, Monorepo Intelligence, Developer Experience, Packaging, RBE, Operations/DR, and ecosystem adapters. This subsystem governs where third-party software comes from and how it becomes a trusted build input.

---

# 1. Purpose

Forgeyard builds software from source, but modern software rarely consists only of first-party code.

Builds consume:

```text
Rust crates
npm packages
Python wheels/sdists
Go modules
Maven/Gradle artifacts
C/C++ archives/source tarballs
Swift packages
container base images
toolchains
OS packages
SDKs
binary utilities
```

Without a dedicated dependency-acquisition architecture, builds can become dependent on:

```text
mutable registries
typosquatted packages
compromised maintainers
deleted versions
registry outages
network races
package substitution
unsigned binaries
untracked mirrors
```

The central rule is:

> **A release-quality Forgeyard build never treats “whatever the registry serves right now” as an acceptable dependency identity. Dependencies are resolved to immutable, verified identities before realization.**

A second rule is:

> **External registries are acquisition sources; Forgeyard CAS/internal mirrors are trusted distribution sources only after verification and policy approval.**

A third rule is:

> **Dependency fetching is allowed in explicit resolve/fetch phases. Hermetic build realization consumes pinned local/CAS/mirror inputs and normally has network disabled.**

---

# 2. Architectural Position

```text
                 External Registries / Sources
                           │
                           ▼
                     Resolver / Fetcher
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
           Identity     Integrity    Policy
               │           │           │
               └───────────┼───────────┘
                           ▼
                  Quarantine / Verify
                           │
                           ▼
                 Trusted Dependency CAS
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
             Mirror      Vendor      Offline Bundle
               │           │           │
               └───────────┼───────────┘
                           ▼
                    Hermetic Build
```

---

# 3. Goals

The subsystem MUST:

1. define dependency identity;
2. support ecosystem registries;
3. support source archives;
4. support binary dependencies;
5. support container bases;
6. support toolchain inputs;
7. support lockfiles;
8. support checksums;
9. support signatures where available;
10. support provenance metadata;
11. support internal mirrors;
12. support proxy mode;
13. support immutable promotion;
14. support dependency allow/deny policy;
15. support license/security checks;
16. support typosquat/namespace risk controls;
17. support quarantine;
18. support vendoring;
19. support offline/air-gap;
20. support mirror replication;
21. support source-of-truth tracking;
22. support registry outages;
23. support package deletion resilience;
24. support integrity re-verification;
25. support multi-tenancy;
26. support audit;
27. support observability;
28. support DR;
29. remain provider-neutral;
30. preserve hermeticity.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Cargo/npm/pip/Maven/Go module ecosystems
become a public package registry by default
rewrite third-party package contents
silently fork dependencies
replace SBOM/vulnerability/license analysis
```

---

# 5. Workspace Structure

```text
crates/dependency/
├── forgeyard-dependency/
├── forgeyard-dependency-model/
├── forgeyard-dependency-resolve/
├── forgeyard-dependency-fetch/
├── forgeyard-dependency-lock/
├── forgeyard-dependency-verify/
├── forgeyard-dependency-policy/
├── forgeyard-dependency-quarantine/
├── forgeyard-dependency-promote/
├── forgeyard-dependency-mirror/
├── forgeyard-dependency-vendor/
├── forgeyard-dependency-offline/
├── forgeyard-dependency-health/
└── forgeyard-dependency-testkit/
```

Registry adapters:

```text
crates/registry/
├── forgeyard-registry/
├── forgeyard-registry-cargo/
├── forgeyard-registry-npm/
├── forgeyard-registry-pypi/
├── forgeyard-registry-go/
├── forgeyard-registry-maven/
├── forgeyard-registry-oci/
├── forgeyard-registry-generic/
└── forgeyard-registry-testkit/
```

Use modules first; split only where ecosystem/client dependencies justify.

---

# 6. DependencyCoordinate

```rust
pub enum DependencyCoordinate {
    Cargo(CargoCoordinate),
    Npm(NpmCoordinate),
    Python(PythonCoordinate),
    Go(GoModuleCoordinate),
    Maven(MavenCoordinate),
    Swift(SwiftPackageCoordinate),
    Oci(OciCoordinate),
    Generic(GenericDependencyCoordinate),
}
```

---

# 7. Logical vs Resolved Identity

Logical:

```text
serde = "1"
```

Resolved:

```text
serde 1.0.xxx
exact checksum
registry source
artifact digest
```

---

# 8. ResolvedDependencyId

```rust
pub struct ResolvedDependencyId(Digest);
```

Content-derived.

---

# 9. Resolved Dependency

```rust
pub struct ResolvedDependency {
    pub id: ResolvedDependencyId,
    pub coordinate: DependencyCoordinate,
    pub source: DependencySource,
    pub version: DependencyVersion,
    pub artifact: CasObjectRef,
    pub integrity: DependencyIntegrity,
    pub provenance: DependencyProvenance,
}
```

---

# 10. Dependency Source

```rust
pub enum DependencySource {
    Registry(RegistrySourceId),
    Vcs(VcsDependencySource),
    Url(VerifiedUrlSource),
    LocalPath(LocalPathDependency),
    InternalMirror(MirrorId),
}
```

---

# 11. RegistrySourceId

Stable configured source identity.

---

# 12. Never Identify Source Only by Hostname String

Use configured registry source object + trust metadata.

---

# 13. Dependency Integrity

```rust
pub struct DependencyIntegrity {
    pub expected_digest: DigestSet,
    pub verified_digest: DigestSet,
    pub signature: Option<DependencySignatureEvidence>,
}
```

---

# 14. Digest

Use ecosystem-native digest where available plus Forgeyard BLAKE3 alias.

---

# 15. SHA-256

Needed for many ecosystems/interoperability.

---

# 16. BLAKE3

Internal CAS identity.

---

# 17. Verified Alias

Same bytes can have both.

---

# 18. Lockfile

Machine-resolved immutable dependency graph.

---

# 19. Existing Ecosystem Lockfiles

Examples:

```text
Cargo.lock
package-lock.json
pnpm-lock.yaml
poetry.lock
uv.lock
go.sum/go.mod
gradle lock files
```

---

# 20. Forgeyard Lock

Hermetic system may maintain:

```text
forgeyard.lock
```

for cross-ecosystem resolved metadata.

---

# 21. No Duplicate Truth

Forgeyard lock complements ecosystem lockfiles; it should record normalized immutable acquisition/provenance information rather than competing version semantics.

---

# 22. Lock Entry

```rust
pub struct DependencyLockEntry {
    pub resolved: ResolvedDependencyId,
    pub source: DependencySource,
    pub digests: DigestSet,
    pub artifact: CasObjectRef,
}
```

---

# 23. Lockfile Update

Explicit operation.

---

# 24. Build Does Not Silently Update Lock

Critical.

---

# 25. Locked Mode

Release/CI default.

---

# 26. Unlocked Resolution

Developer/update workflow only.

---

# 27. `forgeyard deps resolve`

Resolves and proposes lock changes.

---

# 28. `forgeyard deps update`

Explicit.

---

# 29. Resolution Output

```text
old version
new version
source
checksum
license/vulnerability summary
```

---

# 30. Resolution Is Network-Capable

Fetch phase.

---

# 31. Build Realization

Network denied by default.

---

# 32. Registry Adapter

```rust
#[async_trait]
pub trait RegistryAdapter {
    async fn resolve(
        &self,
        request: RegistryResolveRequest,
    ) -> Result<RegistryResolution, RegistryError>;

    async fn fetch(
        &self,
        resolved: &RegistryResolvedArtifact,
    ) -> Result<ByteStream, RegistryError>;
}
```

---

# 33. Adapter Does Not Decide Policy

Core dependency policy service does.

---

# 34. Registry Trust Class

```rust
pub enum RegistryTrustClass {
    PublicUntrusted,
    PublicVerified,
    OrganizationManaged,
    ForgeyardInternalTrusted,
}
```

---

# 35. PublicUntrusted

Normal public internet registry baseline.

---

# 36. Internal Trusted

Only after controlled promotion.

---

# 37. Mirror

```rust
pub struct DependencyMirror {
    pub id: MirrorId,
    pub scope: MirrorScope,
    pub backend: MirrorBackend,
    pub trust: RegistryTrustClass,
}
```

---

# 38. Mirror Scope

```text
Installation
Tenant
Organization
```

---

# 39. Mirror Modes

```rust
pub enum MirrorMode {
    PullThroughCache,
    Curated,
    OfflineOnly,
}
```

---

# 40. Pull-Through Cache

Fetch on miss, verify, store.

---

# 41. Curated

Only explicitly promoted dependencies.

---

# 42. OfflineOnly

No external fetch.

---

# 43. Production Recommendation

High-assurance release builds consume Curated or verified immutable mirror/CAS.

---

# 44. Pull-Through Cache Trust

Do not automatically equate cache presence with approval.

---

# 45. CachedExternal

Can be integrity-verified but still policy-unapproved.

---

# 46. Dependency State

```rust
pub enum DependencyAcquisitionState {
    Discovered,
    Fetched,
    IntegrityVerified,
    Scanned,
    Approved,
    Quarantined,
    Rejected,
    Promoted,
}
```

---

# 47. Promotion

Explicit transition into trusted internal dependency set.

---

# 48. Promotion Record

```rust
pub struct DependencyPromotion {
    pub dependency: ResolvedDependencyId,
    pub from: DependencySource,
    pub to: MirrorId,
    pub evidence: DependencyEvidenceBundleId,
    pub policy_digest: PolicyDigest,
}
```

---

# 49. Promotion Is Immutable

Promoting version X does not approve future X+1.

---

# 50. Namespace Approval

Optional broader policy, but resolved artifact still verified individually.

---

# 51. Quarantine

Downloaded object not allowed to normal build consumers.

---

# 52. Quarantine CAS Namespace

Logically isolated.

---

# 53. Quarantine Reason

Examples:

```text
checksum mismatch
signature failure
malware finding
license policy
vulnerability policy
unknown provenance
manual review required
```

---

# 54. Rejected Dependency

Do not delete evidence immediately.

Keep audit/evidence metadata.

---

# 55. Malware Scanning

Optional provider/scanner integration.

---

# 56. Vulnerability Scan

Part 13 supply-chain scanner.

---

# 57. License Scan

Part 13.

---

# 58. Dependency Evidence Bundle

```rust
pub struct DependencyEvidenceBundle {
    pub id: DependencyEvidenceBundleId,
    pub artifact: CasObjectRef,
    pub digests: DigestSet,
    pub source_metadata: DependencySourceMetadata,
    pub signature: Option<SignatureEvidence>,
    pub vulnerability: Option<EvidenceRef>,
    pub license: Option<EvidenceRef>,
}
```

---

# 59. Registry Metadata

Retain enough to prove origin.

---

# 60. Source URL

Normalized/sanitized.

---

# 61. TLS

Always verify for remote HTTPS registries.

---

# 62. Registry Credential

SecretRef.

---

# 63. No Credential in Lockfile

Critical.

---

# 64. VCS Dependencies

Resolve to exact immutable revision.

---

# 65. Branch/Tag VCS Dependency

Allowed only during resolution.

Lock records exact revision.

---

# 66. Build

Consumes exact source snapshot/archive.

---

# 67. Submodules

Explicit.

---

# 68. URL Dependencies

Require digest.

---

# 69. Unpinned URL

Rejected in strict/release mode.

---

# 70. Generic Archive

```rust
pub struct GenericDependencyCoordinate {
    pub name: BoundedString,
    pub version: BoundedString,
    pub url: SafeExternalUrl,
    pub digest: DigestSet,
}
```

---

# 71. Redirects

Bounded and recorded.

---

# 72. Final URL

Metadata.

---

# 73. TLS Pinning

Optional enterprise.

---

# 74. Registry Mutability

Some registries permit metadata changes/deprecations.

Artifact bytes must remain digest-bound.

---

# 75. Deleted Package

Internal CAS/mirror preserves bytes if previously acquired.

---

# 76. Yanked Package

Metadata retains yanked state.

Policy may block new resolutions but preserve historical builds.

---

# 77. Revoked Package

If ecosystem supports, record state and policy.

---

# 78. Mutable Tag

OCI tags are not dependency identity.

---

# 79. OCI Base Image

Resolve tag -> digest.

---

# 80. Build

Uses digest.

---

# 81. Multi-Arch OCI

Resolve exact platform manifest digest.

---

# 82. Toolchain Registry

Same principles.

---

# 83. Compiler Toolchains

Immutable descriptor + digest.

---

# 84. OS Packages

If consumed:

```text
repository snapshot
package version
package digest
```

not `apt install latest`.

---

# 85. Snapshot Repository

High-assurance.

---

# 86. Dependency Policy

Central Part 11 policy engine consumes dependency facts.

---

# 87. Policy Inputs

```text
registry trust
namespace/package
version
license
vulnerability severity
signature status
provenance
age
maintainer risk
```

---

# 88. No Separate Policy Language

Critical.

---

# 89. Allow Rules

Examples:

```text
allowed registries
allowed namespaces
minimum signature level
allowed licenses
max vulnerability severity
```

---

# 90. Deny Rules

Examples:

```text
package blocked
version blocked
registry blocked
license denied
known malicious hash
```

---

# 91. Hash Denylist

Useful emergency control.

---

# 92. Name Denylist

Supplemental, not enough.

---

# 93. Typosquat Defense

Potential signals:

```text
edit distance
namespace similarity
unexpected registry
new maintainer
sudden package-age anomaly
```

---

# 94. Typosquat Is Heuristic

Cannot be sole automatic rejection unless policy chooses.

---

# 95. Known Namespace Pinning

Example:

```text
@company/*
```

only from internal registry.

---

# 96. Dependency Confusion Defense

Critical.

---

# 97. Internal Package Namespace

Explicit registry binding.

---

# 98. No Public Fallback

For private namespaces.

---

# 99. Registry Priority

Must not silently choose public higher version over private package.

---

# 100. Resolver Rule

Namespace-to-registry binding precedes version resolution.

---

# 101. Package Source Policy

```rust
pub struct PackageSourceRule {
    pub pattern: PackagePattern,
    pub allowed_sources: BTreeSet<RegistrySourceId>,
    pub fallback: SourceFallbackPolicy,
}
```

---

# 102. Fallback Policy

```rust
pub enum SourceFallbackPolicy {
    Never,
    ExplicitOnly,
    Allowed,
}
```

---

# 103. Private Namespace Default

Never.

---

# 104. Lockfile Enforcement

Mismatch between manifest and lockfile:

```text
fail locked build
```

---

# 105. Missing Lock Entry

Fail release build.

---

# 106. Lockfile Tamper

Digest/signature/re-resolution can detect inconsistency.

---

# 107. Lock Review

Change Proposal shows dependency diff.

---

# 108. Dependency Diff

```rust
pub struct DependencyDiff {
    pub added: Vec<ResolvedDependencyId>,
    pub removed: Vec<ResolvedDependencyId>,
    pub changed: Vec<DependencyChange>,
}
```

---

# 109. Change Types

```text
version
source
checksum
features
registry
```

---

# 110. Dependency Change Evidence

Include SBOM/security/license deltas.

---

# 111. Review Routing

Sensitive dependency changes can require owner/security approval.

---

# 112. Lockfile Large Diff

Summarize safely.

---

# 113. Vendoring

```text
forgeyard deps vendor
```

---

# 114. Vendor Bundle

Exact dependency artifacts/sources.

---

# 115. Vendor Manifest

```rust
pub struct VendorManifest {
    pub graph: DependencyGraphId,
    pub entries: Vec<VendorEntry>,
    pub digest: Digest,
}
```

---

# 116. Vendor Directory

Can be materialized for ecosystem tools.

---

# 117. Vendor Is Derived

Canonical identity remains lock + CAS objects.

---

# 118. Commit Vendored Dependencies

Project choice.

---

# 119. Forgeyard Can Avoid Commit

Use CAS/offline bundle.

---

# 120. Air-Gap Bundle

```rust
pub struct DependencyOfflineBundle {
    pub id: DependencyOfflineBundleId,
    pub lock_digest: Digest,
    pub artifacts: Vec<CasObjectRef>,
    pub metadata: Vec<DependencyOfflineMetadata>,
}
```

---

# 121. Offline Bundle Contents

```text
dependency artifacts
checksums
registry/source metadata
licenses
SBOM fragments
verification evidence
```

---

# 122. Bundle Signature

Recommended.

---

# 123. Air-Gap Import

```text
verify manifest
verify signature
verify every artifact digest
apply policy
promote into internal mirror/CAS
```

---

# 124. No Internet Required

Critical.

---

# 125. Mirror Backend

Could use:

```text
local filesystem
S3-compatible object store
generic CAS
OCI registry for OCI artifacts
```

---

# 126. Mirror Logical API

Provider-neutral.

---

# 127. Mirror Object

Immutable by digest.

---

# 128. Mutable Metadata

Version/index metadata can update.

---

# 129. Artifact Mutation

Forbidden.

---

# 130. Same Coordinate Different Bytes

Critical incident.

---

# 131. Registry Equivocation Detection

If same version/source coordinate yields different bytes:

```text
quarantine
alert
audit
```

---

# 132. DependencyEquivocationId

```rust
pub struct DependencyEquivocationId(Ulid);
```

---

# 133. First-Seen Bytes

Retain.

---

# 134. Second-Seen Different Bytes

Never overwrite.

---

# 135. Build Policy

Historical build can still use original pinned bytes.

---

# 136. New Build

Policy may block until review.

---

# 137. Mirror Replication

Critical dependencies replicated.

---

# 138. Durability Class

```rust
pub enum DependencyDurability {
    Cache,
    BuildRequired,
    ReleaseCritical,
}
```

---

# 139. Release Critical Dependency

Pin/retain with release provenance.

---

# 140. CAS GC Root

Lockfiles/releases/offline bundles can pin.

---

# 141. Historical Rebuild

Requires retained dependency bytes.

---

# 142. Reproducibility

Dependency closure is part of derivation.

---

# 143. DependencyGraphId

```rust
pub struct DependencyGraphId(Digest);
```

---

# 144. Graph Inputs

```text
manifest
lockfile
registry mappings
feature/profile/platform
resolver semantics
```

---

# 145. Dependency Graph

Normalized across ecosystems where useful.

---

# 146. Monorepo Graph

Part 34 can reference external dependency graph but should not duplicate it.

---

# 147. External Dependency Node

Graph link.

---

# 148. Affected Analysis

Lockfile/dependency changes can broaden impact.

---

# 149. Dependency Update PR

Can run targeted security/compatibility tests.

---

# 150. Automated Updates

Future bot/service.

---

# 151. Auto Update

Not baseline authority.

---

# 152. Dependency Update Candidate

```rust
pub struct DependencyUpdateCandidate {
    pub current: ResolvedDependencyId,
    pub proposed: ResolvedDependencyId,
    pub reason: DependencyUpdateReason,
}
```

---

# 153. Reasons

```text
security
new version
policy
manual
```

---

# 154. Change Proposal

Generated update uses normal review/check path.

---

# 155. Security Update

Can prioritize.

---

# 156. No Auto-Merge by Default

Policy may allow for low-risk exact categories.

---

# 157. Registry Outage

If required dependencies already mirrored/CAS:

```text
build continues
```

---

# 158. Resolve New Version

Requires source availability.

---

# 159. Degraded Mode

Can build locked dependency closure while external registries unavailable.

---

# 160. Excellent Availability Property

Builds should not depend on internet once closure is cached/promoted.

---

# 161. Pull-Through Cache Miss

External network needed.

---

# 162. Strict Offline

Fail if missing object.

---

# 163. Network Policy

Fetch workers only.

---

# 164. Build Runner

No registry credentials.

---

# 165. Secret Boundary

Fetcher has registry credential scope.

---

# 166. Build Worker

Receives bytes only.

---

# 167. Credential Exfiltration

Untrusted build code cannot access private registry token.

---

# 168. Hosted Multi-Tenant Registry Credential

Tenant scoped.

---

# 169. Internal Global Mirror

Can contain public dependencies.

---

# 170. Tenant Private Dependencies

Isolated.

---

# 171. Cross-Tenant Dedup

Physical CAS dedup allowed according to Part 27 policy.

---

# 172. Authorization

Logical access remains tenant scoped.

---

# 173. Private Package

Never expose presence/version to other tenants.

---

# 174. Mirror Index Leakage

Avoid.

---

# 175. Registry Proxy

Public endpoint to ecosystem client.

---

# 176. Potential

Forgeyard may expose registry-compatible local endpoints for developer/CI tools.

---

# 177. Cargo Proxy

Sparse/index-compatible adapter if implemented.

---

# 178. npm Proxy

Registry API compatibility.

---

# 179. PyPI Proxy

Simple API compatibility.

---

# 180. Maven Proxy

Repository layout.

---

# 181. Scope

Optional.

---

# 182. Baseline

Forgeyard internal fetcher can materialize vendor/cache without exposing full proxy protocol.

---

# 183. Why

Registry-protocol compatibility is substantial complexity.

---

# 184. Incremental Rollout

Start internal resolution/mirror; add proxy APIs later.

---

# 185. Developer Experience

`forgeyard dev bootstrap` prefetches locked dependencies.

---

# 186. `forgeyard deps status`

Shows missing/unverified dependencies.

---

# 187. `forgeyard deps fetch`

Fetch closure.

---

# 188. `forgeyard deps verify`

Offline digest verification.

---

# 189. `forgeyard deps explain`

Shows:

```text
why dependency exists
source
version
digest
policy
```

---

# 190. `forgeyard deps graph`

Dependency graph.

---

# 191. `forgeyard deps diff`

Between snapshots/locks.

---

# 192. Dioxus UI

Pages:

```text
Dependencies
Registry Sources
Mirror
Quarantine
Dependency Updates
Dependency Policy
```

---

# 193. Dependency Detail

Shows:

```text
coordinate
version
source
digest
license
vulnerability
signature
promotion state
consumers
```

---

# 194. Quarantine UI

Review evidence.

---

# 195. Promotion Action

Permission-gated.

---

# 196. Permissions

```text
dependency.read
dependency.resolve
dependency.fetch
dependency.promote
dependency.quarantine
registry.manage
mirror.manage
```

---

# 197. Promotion Permission

High-risk supply-chain permission.

---

# 198. Audit

Audit:

```text
registry add/remove
source rule change
dependency promotion
quarantine override
hash denylist update
```

---

# 199. Normal Fetch

Not necessarily audit every public package request.

---

# 200. Supply Chain Evidence

Dependency source/provenance incorporated into SBOM/provenance.

---

# 201. Build Provenance

Can reference exact dependency graph/lock digest.

---

# 202. SBOM

Components use ResolvedDependencyId mapping.

---

# 203. License Evidence

Cached.

---

# 204. Vulnerability Scan Freshness

Can refresh without refetching artifact.

---

# 205. Package Signature

Where ecosystem provides.

---

# 206. Sigstore/TUF-Like Metadata

Adapters can verify standard ecosystem mechanisms.

---

# 207. No Homegrown Package Signature Scheme

Critical.

---

# 208. TUF

Potential for trusted repository metadata/mirror distribution.

---

# 209. Not Mandatory Baseline

Architecture compatible.

---

# 210. Registry Authentication

Supported types:

```text
token
basic auth
mTLS
cloud workload identity
```

---

# 211. Prefer Short-Lived Credentials

Where available.

---

# 212. Credential Injection

Fetcher only.

---

# 213. Request Logging

No Authorization header.

---

# 214. Registry Response Limits

Bound.

---

# 215. Archive Safety

Dependency archives are untrusted.

---

# 216. Extraction

Prevent:

```text
path traversal
absolute paths
symlink escape
device nodes
zip bombs
```

---

# 217. Prefer Store Raw Artifact

Extract only sandboxed/materialization stage.

---

# 218. Tar/Zip Validation

Strict.

---

# 219. Package Metadata Parser

Bounded.

---

# 220. npm Scripts

Package installation scripts can execute code.

---

# 221. Build Execution

Sandboxed with normal policy.

---

# 222. Resolve/Metadata

Should avoid executing lifecycle scripts.

---

# 223. Python sdists

Building wheel executes build backend code.

---

# 224. Treat As Build Step

Sandboxed.

---

# 225. Prebuilt Wheel

Verify exact bytes.

---

# 226. Maven Plugins

Can execute arbitrary code during build.

Normal sandbox.

---

# 227. Cargo build.rs/proc macros

Untrusted build code.

Normal sandbox.

---

# 228. Registry Fetcher

Never executes package code.

---

# 229. Binary Dependency

Potential malicious executable.

---

# 230. Verification

Digest/signature/policy.

---

# 231. Execution

Only sandboxed toolchain/build context.

---

# 232. Source Package Names

Untrusted strings.

Escape UI/logging.

---

# 233. Unicode Confusables

Potential typosquat signal.

---

# 234. Namespace Canonicalization

Ecosystem-specific.

---

# 235. Do Not Normalize Beyond Ecosystem Semantics

Could create collisions.

---

# 236. Dependency Policy Snapshot

Build/release can bind PolicyDigest.

---

# 237. Revoked Dependency After Build

Historical artifact still exists, but current release verification may fail if policy requires current security status.

---

# 238. Existing Released Artifact

Not silently deleted.

---

# 239. Security Response

Notify/re-evaluate/rebuild according to policy.

---

# 240. Emergency Deny

Hash denylist.

---

# 241. New Runs

Blocked from consuming denylisted dependency.

---

# 242. Historical Verification

Still records what was used.

---

# 243. Search Integration

Part 31 indexes safe dependency metadata.

---

# 244. Analytics

Examples:

```text
dependency age
update lag
registry usage
quarantine rate
vulnerability exposure
```

---

# 245. No Search Index Authority

Canonical metadata/lock remains.

---

# 246. Observability Metrics

```text
dependency_fetch_total
dependency_fetch_failures_total
dependency_verify_failures_total
dependency_quarantine_total
dependency_promotion_total
registry_latency_seconds
mirror_hit_ratio
dependency_equivocation_total
```

---

# 247. Labels

Low cardinality:

```text
ecosystem
registry_class
result
```

---

# 248. No Package Name Metric Labels

Use analytics/search.

---

# 249. Tracing

```text
dependency.resolve
dependency.fetch
dependency.verify
dependency.scan
dependency.promote
mirror.lookup
```

---

# 250. Health

Checks:

```text
registry connectivity
mirror storage
quarantine backlog
verification workers
replication lag
```

---

# 251. Doctor

```text
forgeyard deps doctor
```

---

# 252. Doctor Checks

```text
lock completeness
missing CAS objects
registry source mapping
private namespace fallback
mirror health
digest mismatch
```

---

# 253. Dependency Completeness

```rust
pub enum DependencyClosureState {
    Complete,
    MissingArtifacts,
    Unverified,
    PolicyBlocked,
    Unknown,
}
```

---

# 254. Release Gate

Require Complete + policy-approved closure.

---

# 255. Developer Mode

Can allow unapproved dependency for local experimentation if policy says.

---

# 256. Local Evidence

Not release-promotable.

---

# 257. Strict CI

Approved closure required.

---

# 258. Dependency Closure Manifest

```rust
pub struct DependencyClosureManifest {
    pub id: DependencyClosureId,
    pub graph: DependencyGraphId,
    pub artifacts: Vec<ResolvedDependencyId>,
    pub policy_digest: PolicyDigest,
}
```

---

# 259. Content-Addressed

Can join derivation/evidence.

---

# 260. Build Input

Hermetic build references closure manifest.

---

# 261. Fetch Completion

Before job scheduling or as prerequisite system job.

---

# 262. Scheduler

Should not run build job until required dependency closure available.

---

# 263. Fetch Job

Can run separately with network capability.

---

# 264. Security Boundary

Networked fetch step separated from untrusted build.

---

# 265. Cache Poisoning

Mirror object verified by digest before publication.

---

# 266. Same Digest

Content identical.

---

# 267. Metadata Poisoning

Index metadata signed/validated where ecosystem supports.

---

# 268. Mirror Key

Coordinate -> immutable artifact digest mapping.

---

# 269. Mapping Mutation

Versioned/audited.

---

# 270. External Registry Compromise

Previously promoted dependencies remain available.

---

# 271. New resolution

Policy can freeze external registry.

---

# 272. Registry Freeze

```rust
pub enum RegistryOperationalState {
    Active,
    ReadOnlyCached,
    Frozen,
    Disabled,
}
```

---

# 273. Frozen

No new acquisition.

Existing promoted objects usable.

---

# 274. Incident Response

Useful.

---

# 275. Dependency Update Freeze

During major incident/release freeze.

---

# 276. Multi-Region Mirror

Replicate release-critical closure.

---

# 277. DR

Mirror can rebuild from CAS/backup manifest.

---

# 278. Backup

Release-critical dependencies are CAS roots.

---

# 279. Offline Bundle

Independent DR source.

---

# 280. Mirror Loss

Rehydrate from CAS.

---

# 281. Registry Metadata Loss

Historical lock/provenance retained.

---

# 282. Air-Gap

Curated mirror is ideal.

---

# 283. Import Station

High-assurance workflow:

```text
internet-connected fetch
  ↓
verify/scan
  ↓
signed offline bundle
  ↓
air-gap import
  ↓
internal mirror
```

---

# 284. Two-Person Approval

Optional for high-assurance promotion.

---

# 285. Testkit

```text
forgeyard-dependency-testkit/src/
├── lib.rs
├── resolver.rs
├── registry.rs
├── fetch.rs
├── verify.rs
├── mirror.rs
├── quarantine.rs
├── promotion.rs
└── assertions.rs
```

---

# 286. Unit Tests

Identity/lock/source rules.

---

# 287. Locked Build Test

Manifest drift fails.

---

# 288. Checksum Test

Mismatch quarantined.

---

# 289. Equivocation Test

Same coordinate/different bytes detected.

---

# 290. Private Namespace Test

No public fallback.

---

# 291. Registry Outage Test

Locked/promoted build still works.

---

# 292. Deleted Package Test

Historical build works from mirror.

---

# 293. Yanked Package Test

New resolution policy vs historical build.

---

# 294. VCS Branch Test

Lock resolves exact revision.

---

# 295. OCI Tag Test

Resolves digest.

---

# 296. URL Unpinned Test

Strict mode rejects.

---

# 297. Archive Traversal Test

Rejected.

---

# 298. Zip Bomb Test

Bounded.

---

# 299. Credential Leakage Test

Build runner cannot access registry token.

---

# 300. Tenant Isolation Test

Private package invisible cross-tenant.

---

# 301. Promotion Test

Only exact approved artifact enters curated mirror.

---

# 302. Hash Deny Test

New use blocked.

---

# 303. Air-Gap Test

Bundle verifies/imports offline.

---

# 304. Mirror Corruption Test

Digest mismatch detected/repair.

---

# 305. DR Test

Release-critical closure restored.

---

# 306. Fuzzing

Fuzz:

```text
registry metadata parsers
lock normalization
archive readers
package coordinate parsers
```

---

# 307. Failure Injection

```text
registry timeout
mirror unavailable
CAS disk full
scanner unavailable
credential expired
```

---

# 308. Scale Test

Large dependency closures.

---

# 309. Concurrent Fetch Test

Same object fetched once/logically deduplicated.

---

# 310. Implementation Phase 1 — Normalized Dependency Model

Cargo first.

---

# 311. Phase 2 — Lock/Fetch/Verify

Digest-bound CAS.

---

# 312. Phase 3 — Internal Mirror

Immutable mapping/storage.

---

# 313. Phase 4 — Policy/Quarantine

Security governance.

---

# 314. Phase 5 — Supply Chain Evidence

SBOM/license/vuln.

---

# 315. Phase 6 — Private Registry Rules

Dependency-confusion protection.

---

# 316. Phase 7 — Vendoring/Offline Bundle

Air-gap.

---

# 317. Phase 8 — Additional Ecosystems

npm/Python/Go/Maven/OCI.

---

# 318. Phase 9 — Curated Promotion

Enterprise.

---

# 319. Phase 10 — Registry Proxy Compatibility

Optional.

---

# 320. Phase 11 — Multi-Region/DR

Durability.

---

# 321. Phase 12 — Scale/Fuzz/Security Hardening

Production readiness.

---

# 322. Acceptance Tests

1. Release builds never resolve mutable dependency ranges during realization.
2. Build-time dependency identities are exact and digest-bound.
3. Lockfile updates are explicit.
4. Missing/outdated lock data fails strict builds.
5. External registries are acquisition sources, not trusted build authority.
6. Registry credentials are never exposed to build jobs.
7. Dependency fetch and untrusted build execution are separate trust phases.
8. URL dependencies require digests in strict mode.
9. VCS dependencies resolve to exact immutable revisions.
10. OCI tags resolve to exact digests.
11. Same coordinate returning different bytes triggers equivocation handling.
12. Mirror objects are immutable by digest.
13. Pull-through cache presence does not imply policy approval.
14. Curated promotion binds exact dependency/evidence/policy.
15. Private namespaces do not silently fall back to public registries.
16. Dependency confusion is prevented by explicit source mapping.
17. Checksum mismatch quarantines artifact.
18. Archive traversal and decompression bombs are rejected.
19. Registry/package metadata is treated as untrusted input.
20. Dependency policy uses the central policy engine.
21. License/vulnerability evidence can be refreshed without changing artifact identity.
22. Locked builds continue during registry outage if closure is mirrored.
23. Deleted/yanked upstream packages do not destroy historical reproducibility.
24. Release-critical dependency closure is retained as CAS roots.
25. Air-gapped dependency bundles are independently verifiable.
26. Offline import does not require internet.
27. Tenant-private dependency metadata is isolated.
28. Build provenance records exact dependency closure/lock identity.
29. Dependency updates use normal Change Proposal review/checks.
30. Emergency hash deny rules block future consumption without rewriting history.
31. Search/analytics are derived from canonical dependency metadata.
32. Mirror corruption is detected by digest verification.
33. DR can restore release-critical dependency closure.
34. Standalone/distributed share dependency semantics.
35. Forgeyard dogfoods curated dependency acquisition for its own Rust workspace.

---

# 323. Production Readiness Gates

Do not call dependency governance production-ready until:

```text
exact dependency identity model is stable
locked builds never resolve implicitly
fetch/build trust separation works
registry credentials are isolated
private namespace mapping prevents confusion
digest verification/quarantine works
mirror immutability/equivocation detection works
air-gap bundle import/export is tested
supply-chain evidence joins dependency closure
registry outage/historical rebuild tests pass
```

---

# 324. Architectural Invariants

1. dependency identity is immutable and digest-bound;
2. lock updates are explicit;
3. strict builds do not mutate resolution state;
4. external registries are not runtime build authority;
5. fetch phases may use network; realization normally does not;
6. registry credentials never reach untrusted build code;
7. VCS dependencies resolve exact revisions;
8. OCI references resolve digests;
9. URL dependencies are pinned by digest;
10. mirrors never rewrite artifact bytes under same identity;
11. same coordinate/different bytes is a security incident;
12. pull-through cache does not equal approval;
13. curated promotion is exact and policy-bound;
14. private namespaces have explicit registry mappings;
15. public fallback for private namespaces is denied by default;
16. quarantine separates untrusted acquisition from trusted use;
17. package/archive parsers are bounded and hostile-input safe;
18. lifecycle/build scripts never run inside registry fetchers;
19. central policy decides dependency admissibility;
20. SBOM/license/vulnerability evidence remains separate from artifact identity;
21. historical builds can survive registry deletion/outage;
22. release-critical dependencies are retained;
23. air-gapped acquisition is supported;
24. tenant-private package metadata remains isolated;
25. dependency graph/closure is part of build provenance;
26. emergency deny rules affect future use, not historical truth;
27. search/analytics remain derived;
28. mirror corruption is detectable/repairable;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its dependency governance system.

---

# 325. Final Target Architecture

```text
                 External Registry / VCS / URL
                           │
                           ▼
                      Resolution
                           │
                           ▼
                        Fetch
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
            Digest      Signature    Metadata
               │           │           │
               └───────────┼───────────┘
                           ▼
                     Policy / Scan
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
         Quarantine      Reject       Promote
                                           │
                                           ▼
                                 Trusted CAS / Mirror
                                           │
                                           ▼
                                    Hermetic Build
```

---

# 326. Final Architectural Position

Resolution:

```text
manifest constraints
+
registry source mappings
+
lock policy
  ↓
exact version/revision/digest
  ↓
ResolvedDependencyId
```

Promotion:

```text
fetched bytes
+
digest verification
+
source metadata
+
license/vulnerability/signature evidence
+
PolicyDigest
  ↓
approved immutable dependency
  ↓
curated mirror
```

Build:

```text
forgeyard.lock / ecosystem lock
+
DependencyClosureManifest
+
trusted CAS/mirror bytes
  ↓
network-denied realization
```

The key guarantee is:

> **Forgeyard can keep builds reproducible and available even when the public software ecosystem is mutable or temporarily unavailable. Every third-party dependency has an exact identity, verified bytes, recorded origin, and explicit policy state before it becomes a trusted build input.**

---

# 327. Extended Architecture Sequence

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
```
