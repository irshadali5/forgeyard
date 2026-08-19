# 38 — Forgeyard Cache Architecture, Build Acceleration, Remote Cache & Cache Correctness System Architecture

**Document type:** Core Build Cache, Remote Cache, Cache Reuse, Cache Integrity & Acceleration System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** cache-key construction, local/remote caches, CAS-backed cache entries, action/result cache, partial reuse, trust classes, cache poisoning defense, cache provenance, cache hierarchy, cache eviction, negative caching, cross-platform reuse, cache explainability, remote cache federation, cache quotas, cache invalidation semantics, and developer/CI acceleration  
**Architecture style:** Content-addressed, derivation-bound, provenance-aware, immutable-result oriented, layered cache hierarchy, correctness-before-speed, explicit trust, conservative reuse, and no hidden cache-based semantic shortcuts  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on CAS, Hermetic/Reproducible Builds, Pipeline IR, Run/Job State Machine, Scheduler, RBE, Monorepo Intelligence, Developer Experience, Dependency Governance, Supply Chain, Multi-Tenancy, and Operations/DR. This document centralizes all cache behavior into one explicit correctness model.

---

# 1. Purpose

Caching is one of the largest performance multipliers in CI/CD.

But incorrect caching is also one of the easiest ways to create:

```text
stale binaries
cross-branch contamination
cross-platform corruption
secret leakage
tenant data leakage
poisoned build results
non-reproducible releases
```

The central rule is:

> **Forgeyard reuses a cached result only when the cache key proves that every correctness-relevant input is equivalent.**

A second rule is:

> **Cache entries are accelerators over immutable artifacts and derivations. Cache presence never overrides policy, source identity, trust, or required evidence.**

A third rule is:

> **A cache hit may skip execution, but it cannot fabricate execution evidence that did not exist. Reused evidence must be explicitly linked to the original trusted result.**

---

# 2. Architectural Position

```text
                  Job / Action Request
                          │
                          ▼
                   Derivation Builder
                          │
                          ▼
                     Cache Key
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Local Cache  Remote Cache   RBE Cache
             │            │            │
             └────────────┼────────────┘
                          ▼
                   Candidate Result
                          │
                ┌─────────┼─────────┐
                ▼         ▼         ▼
             Integrity   Trust   Provenance
                │         │         │
                └─────────┼─────────┘
                          ▼
                    Reuse Decision
                          │
             ┌────────────┼────────────┐
             ▼                         ▼
           HIT                        MISS
             │                         │
             ▼                         ▼
      Materialize Result          Execute Job
```

---

# 3. Goals

The subsystem MUST:

1. define cache identity;
2. define cache key semantics;
3. define local cache;
4. define remote cache;
5. define hierarchy;
6. define action/result cache;
7. define trust classes;
8. define provenance requirements;
9. prevent poisoning;
10. support tenant isolation;
11. support cross-platform rules;
12. support partial cache reuse;
13. support cache misses;
14. support negative caching carefully;
15. support cache explainability;
16. support cache metrics;
17. support cache quotas;
18. support eviction;
19. support cache warming;
20. support RBE compatibility;
21. support local developer cache;
22. support remote shared cache;
23. support air-gap cache;
24. support DR;
25. support stale entry detection;
26. support key schema versioning;
27. support cache invalidation by semantics;
28. support trusted/untrusted result separation;
29. support cache policy;
30. remain rebuildable from authoritative artifacts.

---

# 4. Non-Goals

Cache does not:

```text
replace CAS
replace source identity
replace policy
replace provenance
replace build execution semantics
replace test evidence
replace scheduler correctness
```

---

# 5. Workspace Structure

```text
crates/cache/
├── forgeyard-cache/
├── forgeyard-cache-model/
├── forgeyard-cache-key/
├── forgeyard-cache-policy/
├── forgeyard-cache-local/
├── forgeyard-cache-remote/
├── forgeyard-cache-result/
├── forgeyard-cache-trust/
├── forgeyard-cache-explain/
├── forgeyard-cache-eviction/
├── forgeyard-cache-warm/
├── forgeyard-cache-reconcile/
├── forgeyard-cache-health/
└── forgeyard-cache-testkit/
```

RBE bridge:

```text
crates/rbe/
└── forgeyard-rbe-cache/
```

Use modules first; split only at real runtime/dependency boundaries.

---

# 6. CacheKey

```rust
pub struct CacheKey(Digest);
```

---

# 7. Cache Key Schema Version

```rust
pub struct CacheKeySchemaVersion(u16);
```

---

# 8. Cache Key Inputs

At minimum:

```text
job/action semantics
source inputs
toolchain
environment
platform
build profile
dependency closure
declared config
executor semantics
cache schema version
```

---

# 9. Derivation Alignment

Where possible:

```text
CacheKey = hash(Derivation)
```

or a stable projection of the same semantic input set.

---

# 10. No Ad-Hoc String Keys

Critical.

---

# 11. Canonical Cache Input

```rust
pub struct CacheInput {
    pub source: SourceSnapshotId,
    pub derivation: DerivationId,
    pub toolchain: ToolchainDescriptorId,
    pub environment: ExecutionEnvironmentId,
    pub platform: PlatformDescriptor,
    pub dependencies: DependencyClosureId,
    pub executor_profile: ExecutorProfileId,
    pub semantics_version: CacheSemanticsVersion,
}
```

---

# 12. CacheSemanticsVersion

Bump when meaning of execution changes.

---

# 13. Why

If sandbox/network/path semantics change, old results may no longer be safely reusable.

---

# 14. Host-Independent Key

Do not include ephemeral host identity unless correctness depends on it.

---

# 15. Platform-Specific Key

Include platform/ABI where outputs differ.

---

# 16. Example

Linux x86_64 build result cannot automatically satisfy macOS arm64.

---

# 17. Cross-Platform Reuse

Only if artifact/result semantics explicitly declare platform independence.

---

# 18. Platform Independence Type

```rust
pub enum CachePortability {
    ExactPlatform,
    CompatiblePlatformClass,
    PlatformIndependent,
}
```

---

# 19. Default

ExactPlatform.

---

# 20. Environment Variables

Only declared correctness-relevant variables enter key.

---

# 21. Undeclared Host Env

Should not affect hermetic result.

---

# 22. If It Does

Build is impure; cache trust reduced or disabled.

---

# 23. Secret Inputs

Do not include secret plaintext in cache key.

---

# 24. Secret-Sensitive Jobs

Usually non-cacheable unless explicit secret-version semantics prove safe.

---

# 25. Secret Version Ref

Can influence key if result legitimately depends on secret version.

---

# 26. Security Default

Jobs consuming secrets:

```text
cache disabled
```

unless explicitly reviewed.

---

# 27. Networked Jobs

Often non-cacheable unless fetched inputs are fully declared/pinned.

---

# 28. Cacheability

```rust
pub enum Cacheability {
    Cacheable,
    CacheableWithConstraints(CacheConstraintSet),
    Uncacheable(CacheDisableReason),
}
```

---

# 29. Disable Reasons

```text
secret dependency
undeclared network
nondeterministic clock
device interaction
external mutable service
interactive job
```

---

# 30. Pipeline IR

Job can declare cache intent.

---

# 31. CachePolicy

```rust
pub struct JobCachePolicy {
    pub mode: CacheMode,
    pub scope: CacheScope,
    pub portability: CachePortability,
}
```

---

# 32. CacheMode

```rust
pub enum CacheMode {
    Disabled,
    ReadOnly,
    ReadWrite,
    RequireHit,
}
```

---

# 33. RequireHit

Special offline/verified use case.

---

# 34. CacheScope

```rust
pub enum CacheScope {
    Local,
    Project,
    Tenant,
    Installation,
    ExplicitShared(CacheShareId),
}
```

---

# 35. Cross-Tenant Shared Cache

Disabled by default.

---

# 36. Explicit Shared

Requires strong policy/trust equivalence.

---

# 37. Cache Entry

```rust
pub struct CacheEntry {
    pub key: CacheKey,
    pub result: CachedResultRef,
    pub trust: CacheTrustClass,
    pub created_at: Timestamp,
    pub source: CacheEntrySource,
}
```

---

# 38. Cached Result

```rust
pub struct CachedResult {
    pub outputs: Vec<CasObjectRef>,
    pub metadata: CachedResultMetadata,
    pub provenance: Option<EvidenceRef>,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 39. Result Metadata

Includes:

```text
original RunId/JobId/AttemptId
toolchain
executor profile
output manifest
result state
```

---

# 40. Cache Hit Evidence

New job can reference original evidence.

---

# 41. No Fabricated Attempt

If cache hit avoids execution:

```text
Job succeeds via cache
```

without creating fake JobAttempt.

---

# 42. Existing Invariant

Part 5 already allows cache hit success without attempt.

---

# 43. CacheHitRecord

```rust
pub struct CacheHitRecord {
    pub job: JobId,
    pub key: CacheKey,
    pub entry: CacheEntryId,
    pub original_result: CachedResultRef,
}
```

---

# 44. Entry ID

```rust
pub struct CacheEntryId(Digest);
```

---

# 45. Cache Trust Class

```rust
pub enum CacheTrustClass {
    LocalUntrusted,
    ProjectTrusted,
    TenantTrusted,
    ReleaseTrusted,
    ExternalUntrusted,
}
```

---

# 46. LocalUntrusted

Developer workstation cache.

---

# 47. ProjectTrusted

Produced by trusted project CI runner.

---

# 48. ReleaseTrusted

Result satisfies stronger provenance/reproducibility requirements.

---

# 49. ExternalUntrusted

Imported RBE/remote cache result pending verification.

---

# 50. Trust Evaluation

Cache reuse policy considers trust class.

---

# 51. Example

PR build may use ProjectTrusted.

Stable release may require ReleaseTrusted or rebuild/verify.

---

# 52. Cache Provenance

Critical for high assurance.

---

# 53. Provenance Inputs

```text
builder identity
runner trust
executor
source
toolchain
derivation
result digest
```

---

# 54. Cache Entry Promotion

Like dependency promotion.

---

# 55. CachePromotion

```rust
pub struct CachePromotion {
    pub entry: CacheEntryId,
    pub from: CacheTrustClass,
    pub to: CacheTrustClass,
    pub evidence: EvidenceBundleId,
    pub policy_digest: PolicyDigest,
}
```

---

# 56. Promotion Does Not Change Bytes

Only trust metadata.

---

# 57. Cache Poisoning

Threats:

```text
malicious runner
compromised remote cache
incorrect key
cross-tenant injection
mutable output reference
```

---

# 58. Poison Defense

```text
content-addressed outputs
exact key
trusted writer identity
provenance verification
tenant namespace
output digest verification
```

---

# 59. Remote Cache Write

Requires permission/trust.

---

# 60. Untrusted Runner

May read cache but cannot populate trusted namespace.

---

# 61. Local Developer

Cannot push release-trusted cache.

---

# 62. Cache Writer Capability

```rust
pub struct CacheWriteCapability {
    pub scope: CacheScope,
    pub max_trust: CacheTrustClass,
}
```

---

# 63. No Generic "write cache" Boolean

Typed capability preferred.

---

# 64. CAS Integrity

Every output object verified by digest.

---

# 65. Cache Metadata Integrity

Metadata can be signed or DB-authoritative depending mode.

---

# 66. External Cache Import

Verify outputs before local trusted publication.

---

# 67. RBE Action Cache

Part 23.

---

# 68. RBE Bridge

External SHA-256 action digest maps to internal CacheKey/ExecutionProfile semantics carefully.

---

# 69. No Blind RBE Trust

Action cache result must meet tenant/trust policy.

---

# 70. Local Cache

Fastest.

---

# 71. Local Cache Layers

```text
process memory
local metadata index
local CAS
```

---

# 72. Remote Cache

Shared across runners.

---

# 73. Remote Cache Backend

Could be:

```text
Postgres metadata + CAS
object store
dedicated cache service
RBE-compatible cache
```

---

# 74. Cache Hierarchy

```text
memory
  ↓
local disk
  ↓
project/tenant remote
  ↓
optional shared trusted
```

---

# 75. Read Order

Nearest/cheapest first.

---

# 76. Write Order

Local immediately; remote asynchronously or synchronously per policy.

---

# 77. Write-Behind

Allowed for non-critical cache.

---

# 78. Release-Trusted Write

May require durable confirmation.

---

# 79. Remote Cache Outage

Cache miss fallback to execution.

---

# 80. Cache Is Not Availability Dependency

Critical.

---

# 81. Offline Mode

Can use local/air-gap cache only.

---

# 82. RequireHit

Useful for strictly offline reproduction.

---

# 83. Partial Cache Reuse

Examples:

```text
dependency fetch cache
compiled object cache
test discovery cache
package layer cache
```

---

# 84. Partial Result

Do not pretend full job cached.

---

# 85. Cache Artifact Kind

```rust
pub enum CacheArtifactKind {
    FullJobResult,
    BuildIntermediate,
    DependencyObject,
    TestInventory,
    GeneratedSource,
    PackageLayer,
    Custom(CacheArtifactKindId),
}
```

---

# 86. Intermediate Cache

More ecosystem-specific.

---

# 87. Core Rule

Intermediate reuse must still bind all relevant inputs.

---

# 88. Compiler Cache

Examples:

```text
sccache-like
ccache-like
```

---

# 89. Integration

Can be adapter/toolchain layer.

---

# 90. No Mandatory External Compiler Cache

Forgeyard can expose generic cache service.

---

# 91. Remote Compiler Cache Credentials

Scoped.

---

# 92. Negative Caching

Cache some failures cautiously.

---

# 93. Default

Do not cache arbitrary failed jobs.

---

# 94. Safe Negative Cache Candidates

Examples:

```text
dependency not found for immutable lock
known policy denial
```

---

# 95. Unsafe Negative Cache

```text
network timeout
test failure
compiler failure after source change ambiguity
```

---

# 96. Negative Cache TTL

Short and explicit.

---

# 97. NegativeCacheEntry

```rust
pub struct NegativeCacheEntry {
    pub key: CacheKey,
    pub reason: NegativeCacheReason,
    pub expires_at: Timestamp,
}
```

---

# 98. No Release Gate on Negative Cache Alone

Critical.

---

# 99. Cache Miss

Not error.

---

# 100. Miss Reason

Explainable.

---

# 101. CacheMissReason

```rust
pub enum CacheMissReason {
    NotFound,
    SourceChanged,
    ToolchainChanged,
    EnvironmentChanged,
    DependencyChanged,
    ExecutorSemanticsChanged,
    PolicyRejected,
    TrustInsufficient,
    EntryCorrupt,
    EntryExpired,
}
```

---

# 102. `forgeyard cache explain`

High-value developer/admin tool.

---

# 103. Explain Inputs

Can compare current key with nearest prior key.

---

# 104. Diff

```text
source digest changed
toolchain same
env same
dependency closure changed
```

---

# 105. No Secret Values in Explain

Critical.

---

# 106. Cache Hit Reason

Can show trust/provenance.

---

# 107. Cache Index

Metadata mapping:

```text
CacheKey -> CacheEntry
```

---

# 108. CAS

Stores actual bytes.

---

# 109. Immutable Output Manifest

Content-addressed.

---

# 110. Cache Entry Mutation

Do not overwrite existing trusted entry silently.

---

# 111. Same Key Different Outputs

Critical nondeterminism or poisoning incident.

---

# 112. Cache Equivocation

```rust
pub struct CacheEquivocation {
    pub key: CacheKey,
    pub existing: CachedResultRef,
    pub conflicting: CachedResultRef,
}
```

---

# 113. Response

```text
quarantine both/new result
alert
record nondeterminism
block trust promotion
```

---

# 114. Reproducibility Integration

Same derivation -> different output proves nondeterminism.

---

# 115. Existing FRBS

Use evidence.

---

# 116. First Result

Do not blindly overwrite.

---

# 117. Multiple Valid Outputs

If intentionally nondeterministic job, it should be Uncacheable.

---

# 118. Cache Entry State

```rust
pub enum CacheEntryState {
    Active,
    Quarantined,
    Corrupt,
    Expired,
    Evicted,
}
```

---

# 119. Expiry

Semantic or policy-driven.

---

# 120. Immutable Build Result

May not require time expiry.

---

# 121. Security Freshness

Evidence may expire even if bytes remain.

---

# 122. Cache Bytes vs Evidence Freshness

Separate.

---

# 123. Example

Cached artifact still valid bytes, but vulnerability scan stale.

Release policy can require re-scan.

---

# 124. Cache Cannot Skip Freshness-Required Evidence

Critical.

---

# 125. Test Cache

Potential reuse only when exact test inputs/environment equivalent.

---

# 126. Release Policy

May require fresh test execution.

---

# 127. Cache Policy Can Disable Test Result Reuse

---

# 128. Benchmark Cache

Usually disabled.

---

# 129. Static Analysis Cache

Possible with exact analyzer/ruleset subject.

---

# 130. Build Cache

Most valuable.

---

# 131. Package Cache

Possible if deterministic.

---

# 132. Signing

Never cache private signing operation as generic job result.

---

# 133. Signed Artifact

Can reuse exact already-signed immutable artifact as release input.

---

# 134. Secretful Deployment

Not cacheable.

---

# 135. Device Job

Usually not cacheable.

---

# 136. Cache Entry Scope

Tenant/project ownership.

---

# 137. Cross-Tenant Physical Dedup

CAS may deduplicate physically.

---

# 138. Cache Metadata Visibility

Tenant scoped.

---

# 139. Cross-Tenant Hit

Disabled by default.

---

# 140. Why

Potential confidentiality/timing leakage.

---

# 141. Shared Public OSS Cache

Explicit special mode possible.

---

# 142. Shared Cache Eligibility

Only content classified safe/public.

---

# 143. CacheShareId

```rust
pub struct CacheShareId(Ulid);
```

---

# 144. Share Policy

Defines:

```text
allowed tenants/projects
artifact classification
trust
writer classes
```

---

# 145. Quotas

Part 27.

---

# 146. Cache Storage Quota

Separate from durable artifact quota.

---

# 147. Cache Eviction

Allowed aggressively compared with durable release artifacts.

---

# 148. Eviction Priority

```text
least recently used
large low-hit entries
low-trust
old intermediate
```

---

# 149. Eviction Algorithm

Implementation detail; correctness independent.

---

# 150. Cache Root

Cache entries are not permanent CAS GC roots unless policy.

---

# 151. Evicted Entry

Job recomputes.

---

# 152. Release Artifact

Never evicted merely because cache policy.

---

# 153. Cache Residency

Can keep hot data local to runner pool.

---

# 154. Cache Locality Score

Scheduler can prefer runner with required objects.

---

# 155. Scheduler Integration

Soft score only.

---

# 156. Hard Placement

Not based on cache availability.

---

# 157. Cache Warming

Pre-populate likely inputs/results.

---

# 158. Warm Sources

```text
dependency closure
toolchain
base branch builds
popular artifacts
```

---

# 159. Warm Job

Normal system operation.

---

# 160. Warming Is Advisory

Failure does not block pipeline.

---

# 161. Predictive Warming

Future.

---

# 162. No Opaque ML Baseline

Simple usage history.

---

# 163. Branch Cache

Do not scope identity to branch unless branch is truly semantic input.

---

# 164. Exact Source Identity

Allows reuse across branches with identical content.

---

# 165. Good Property

Cache follows content, not branch names.

---

# 166. Monorepo

Part 34 affected work selection happens before cache.

---

# 167. Cache Granularity

Can be target-level via derivation.

---

# 168. Source Subtree Input

If graph proves target input subset, cache key need not include whole repo.

---

# 169. Confidence

Only with conservative complete input set.

---

# 170. Unknown Input

Broaden key/input set.

---

# 171. Developer Local Cache

Fast.

---

# 172. Remote Shared Cache

Optional.

---

# 173. `forgeyard cache status`

Shows:

```text
local size
remote availability
hit rate
trust
```

---

# 174. `forgeyard cache stats`

Usage.

---

# 175. `forgeyard cache explain <job>`

Miss/hit reason.

---

# 176. `forgeyard cache gc`

Local/admin.

---

# 177. `forgeyard cache verify`

Integrity sampling/full.

---

# 178. `forgeyard cache warm`

Explicit.

---

# 179. `forgeyard cache quarantine`

Admin/security.

---

# 180. UI

Pages:

```text
Cache Overview
Hit Rate
Storage
Remote Cache
Trust
Quarantine
Equivocation
```

---

# 181. Run UI

Job row can show:

```text
cache hit
cache miss
cache bypass
```

---

# 182. Cache Hit Detail

Shows:

```text
key
scope
trust
original result
provenance
```

---

# 183. No Sensitive Key Inputs

Use digests/safe names.

---

# 184. Miss Detail

Explain changed dimensions.

---

# 185. Cache Analytics

Examples:

```text
hit rate by job class
bytes saved
compute avoided
remote/local hit split
eviction
```

---

# 186. Compute Saved

Estimated, not exact if no execution occurred.

---

# 187. Label as Estimate

Critical.

---

# 188. Observability Metrics

```text
cache_lookup_total
cache_hit_total
cache_miss_total
cache_write_total
cache_write_failures_total
cache_corruption_total
cache_equivocation_total
cache_bytes
cache_evictions_total
```

---

# 189. Labels

Low cardinality:

```text
scope
trust
artifact_kind
result
```

---

# 190. No CacheKey Metric Label

Use analytics.

---

# 191. Tracing

```text
cache.lookup
cache.verify
cache.write
cache.promote
cache.evict
cache.explain
```

---

# 192. Health

Checks:

```text
local index
remote backend
CAS
write/read consistency
quarantine
```

---

# 193. Doctor

```text
forgeyard cache doctor
```

---

# 194. Doctor Checks

```text
remote connectivity
digest verification
namespace permissions
stale entries
equivocation incidents
```

---

# 195. API

Potential:

```text
GET  /v1/cache/status
GET  /v1/cache/stats
GET  /v1/jobs/{id}/cache
GET  /v1/admin/cache/entries
POST /v1/admin/cache/verify
POST /v1/admin/cache/gc
```

---

# 196. No Raw Cache Dump to Regular Users

Security.

---

# 197. Permissions

```text
cache.read
cache.write
cache.admin
cache.promote
cache.quarantine
```

---

# 198. Writer Permission

Scoped by trust.

---

# 199. Policy Integration

Central Part 11.

---

# 200. Cache Reuse Fact

Policy inputs:

```text
trust class
provenance available
source trust
job class
artifact classification
```

---

# 201. Example

External fork:

```text
may read public project cache
cannot write trusted cache
```

---

# 202. Untrusted Source

Secret-bearing trusted cache should not be exposed.

---

# 203. Supply Chain Integration

Cached output retains original provenance.

---

# 204. Reused Result Provenance

New job/run records reuse lineage.

---

# 205. Evidence Chain

```text
original build evidence
  ↓
cached result
  ↓
reuse record
```

---

# 206. No New SLSA Claim From Cache Hit Alone

Critical.

---

# 207. Reproducibility Verification

Can independently reproduce cached output.

---

# 208. Cache Verification Worker

Samples trusted cache entries.

---

# 209. Verification Strategy

```text
rehash outputs
verify manifests
optional rebuild/reproduce sample
```

---

# 210. Rebuild Sampling

Can detect poisoning/nondeterminism.

---

# 211. Quarantine Trigger

```text
digest mismatch
metadata mismatch
trust violation
equivocation
```

---

# 212. Quarantine Scope

Entry/key/producer.

---

# 213. Producer Trust Downgrade

If repeated poisoning.

---

# 214. Runner Reliability

Scheduler/trust subsystem integration.

---

# 215. Cache Writer Identity

Stored.

---

# 216. Incident Response

Can revoke writer capability and quarantine entries from producer.

---

# 217. Bulk Quarantine

By runner/session/time range.

---

# 218. Audit

Audit:

```text
cache trust promotion
bulk quarantine
shared cache configuration
writer capability change
```

---

# 219. Not every hit.

---

# 220. Notification

Critical cache corruption/equivocation alerts.

---

# 221. Cache Corruption

Object bytes fail digest.

---

# 222. Repair

Refetch/recompute.

---

# 223. Never Substitute Different Bytes Under Same Digest

Existing CAS invariant.

---

# 224. Remote Cache Federation

Optional.

---

# 225. Federation

Multiple sites/regions.

---

# 226. Read-Through Federation

Nearest site first.

---

# 227. Trust

Each peer has configured trust class.

---

# 228. Cross-Region Replication

Hot/important entries.

---

# 229. WAN Cost

Policy.

---

# 230. Remote Site Unavailable

Fallback other cache/execute.

---

# 231. Air-Gap Cache Bundle

Can export cache closure.

---

# 232. CacheBundleId

```rust
pub struct CacheBundleId(Digest);
```

---

# 233. Bundle Contents

```text
cache metadata
CAS objects
provenance
integrity manifest
```

---

# 234. Import

Verify all digests/trust.

---

# 235. Imported Trust

Never automatically highest trust.

---

# 236. DR

Cache can be lost entirely without business-data loss.

---

# 237. Important

Cache is reconstructible.

---

# 238. Backup

Optional for performance.

---

# 239. Release-Trusted Entries

Underlying artifacts/provenance already durable elsewhere.

---

# 240. No Cache Backup Correctness Dependency

Critical.

---

# 241. Cache Migration

Schema versioned.

---

# 242. Old Key Schema

Can coexist.

---

# 243. New Version

Misses old incompatible keys.

---

# 244. Do Not "Translate" Old Key Without Proof

---

# 245. CacheKey Migration

Usually natural cold misses.

---

# 246. API/RBE Compatibility

RBE action cache mapping versioned.

---

# 247. Compiler Cache

Can have its own native key.

---

# 248. Forgeyard Records Integration Boundary

---

# 249. Test Result Cache

Strict policy.

---

# 250. Local PR

May reuse exact test result if policy permits.

---

# 251. Stable Release

Often fresh test execution required.

---

# 252. Static Analysis Reuse

Can be allowed if exact analyzer/ruleset/DB snapshot same.

---

# 253. Coverage Reuse

Exact source + test execution semantics.

---

# 254. Benchmark

Default uncacheable.

---

# 255. Deployment

Uncacheable.

---

# 256. Notification

Not cacheable.

---

# 257. Search/Analytics

Separate query caches, not build cache.

---

# 258. Query Cache

Part 31.

---

# 259. Cache Security Classification

```rust
pub enum CacheDataClass {
    Public,
    Internal,
    Sensitive,
    Restricted,
}
```

---

# 260. Shared Cache

Only policy-approved classes.

---

# 261. Secret-Derived Output

Potentially Restricted.

---

# 262. Classification Propagation

Output inherits highest relevant input class unless sanitizer/declassifier explicitly trusted.

---

# 263. Declassification

High-risk explicit transform.

---

# 264. Not baseline automatic.

---

# 265. Tenant Cache Export

Scoped.

---

# 266. Local Developer Cache

OS file permissions.

---

# 267. Multi-User Machine

Separate cache namespace by user unless explicitly shared.

---

# 268. Shared Local Cache

Requires trust config.

---

# 269. NFS Cache

Possible but locking/integrity required.

---

# 270. Better

Use CAS/object service for multi-host.

---

# 271. Local Cache Index Corruption

Can rebuild from CAS metadata/manifests if enough info retained.

---

# 272. Cache Reconciler

Checks:

```text
metadata without objects
objects without metadata
expired entries
quarantined entries
stale reservations
```

---

# 273. Cache Write Transaction

Do not publish metadata until outputs durable/verified.

---

# 274. Correct Sequence

```text
upload outputs
  ↓
verify digests
  ↓
persist result manifest
  ↓
publish cache entry
```

---

# 275. Partial Upload

Invisible.

---

# 276. Crash After Upload Before Publish

Orphan CAS object, later GC.

---

# 277. Crash After Publish

Outputs already durable.

---

# 278. Cache Read Sequence

```text
lookup metadata
  ↓
verify scope/trust/policy
  ↓
ensure outputs available
  ↓
verify manifest/digests as required
  ↓
materialize
```

---

# 279. Missing Object

Treat as corrupt/miss.

---

# 280. No Partial Success

---

# 281. Cache Stampede

Multiple identical misses can trigger duplicate builds.

---

# 282. Singleflight

Optional optimization.

---

# 283. CacheFillLease

```rust
pub struct CacheFillLease {
    pub key: CacheKey,
    pub holder: JobId,
    pub expires_at: Timestamp,
}
```

---

# 284. Semantics

One producer preferred; others may wait or proceed depending policy.

---

# 285. Correctness

Duplicate builds are safe.

---

# 286. Do Not Make Fill Lease Global Correctness Dependency

---

# 287. Speculative Duplicate

Can be useful for tail latency.

---

# 288. If Different Outputs

Equivocation/nondeterminism detected.

---

# 289. Cache Warm Prediction

Derived analytics.

---

# 290. No authority.

---

# 291. Cache Retention Policy

By:

```text
age
hit frequency
size
trust
artifact kind
```

---

# 292. Protected Entries

Can pin.

---

# 293. Pin

```rust
pub struct CachePin {
    pub entry: CacheEntryId,
    pub reason: CachePinReason,
    pub expires_at: Option<Timestamp>,
}
```

---

# 294. Pin Reasons

```text
release candidate
offline bundle
debug reproduction
benchmark baseline
```

---

# 295. Permanent Pins

Avoid unless justified.

---

# 296. Cache Quota Pressure

Evict low-priority before blocking builds.

---

# 297. If Full

Cache writes may fail but build result still succeeds if durable artifact path succeeds.

---

# 298. Cache Write Failure

Should not turn successful build into failure unless cache required by explicit workflow.

---

# 299. RequireHit/RequireWrite

Special.

---

# 300. Build Correctness Separate

Critical.

---

# 301. Developer UX

Run summary:

```text
10 jobs
6 cache hits
4 executed
```

---

# 302. Explain Savings

Estimated.

---

# 303. Cache Miss Debugging

One of key UX differentiators.

---

# 304. Testkit

```text
forgeyard-cache-testkit/src/
├── lib.rs
├── key.rs
├── entry.rs
├── trust.rs
├── hierarchy.rs
├── poisoning.rs
├── eviction.rs
└── assertions.rs
```

---

# 305. Unit Tests

Cache key determinism.

---

# 306. Source Change Test

Changes key.

---

# 307. Toolchain Change Test

Changes key.

---

# 308. Irrelevant Host Env Test

Does not change hermetic key.

---

# 309. Platform Test

Cross-platform rejected by default.

---

# 310. Secret Job Test

Uncacheable by default.

---

# 311. Network Impurity Test

Cache disabled/reduced trust.

---

# 312. Local Trust Test

Developer cache cannot satisfy release-trusted policy.

---

# 313. Cross-Tenant Test

No cache hit leakage.

---

# 314. Poisoning Test

Untrusted writer cannot populate trusted cache.

---

# 315. Equivocation Test

Same key/different outputs quarantined.

---

# 316. Missing CAS Object Test

Entry treated corrupt/miss.

---

# 317. Partial Publish Test

Invisible until durable.

---

# 318. Remote Cache Outage Test

Falls back execution.

---

# 319. Eviction Test

Recompute safely.

---

# 320. Negative Cache Test

Transient network failure not negatively cached.

---

# 321. Evidence Freshness Test

Cached artifact does not satisfy stale vulnerability-scan requirement.

---

# 322. Test Cache Policy Test

Release requires fresh test despite exact prior result.

---

# 323. RBE Mapping Test

External action cache respects tenant/trust semantics.

---

# 324. Fill Lease Crash Test

Other producer eventually continues.

---

# 325. DR Test

Full cache loss does not lose business/release truth.

---

# 326. Corruption Test

Digest mismatch detected.

---

# 327. Fuzzing

Fuzz cache key serialization/metadata decoders.

---

# 328. Property Tests

Same semantic inputs -> same key.

Different correctness-relevant input -> different key.

---

# 329. Load Test

High concurrent cache lookup/write.

---

# 330. Stampede Test

Many same-key misses.

---

# 331. Implementation Phase 1 — Cache Key Model

Derivation alignment.

---

# 332. Phase 2 — Local Cache

Standalone/dev.

---

# 333. Phase 3 — Remote Shared Cache

Tenant/project scope.

---

# 334. Phase 4 — Trust/Provenance

Trusted writes.

---

# 335. Phase 5 — Cache Explain

Developer value.

---

# 336. Phase 6 — RBE Integration

Action cache bridge.

---

# 337. Phase 7 — Eviction/Quotas

Operations.

---

# 338. Phase 8 — Equivocation/Nondeterminism Detection

Correctness hardening.

---

# 339. Phase 9 — Cache Warming/Locality

Performance.

---

# 340. Phase 10 — Federation/Air-Gap

Enterprise.

---

# 341. Phase 11 — Verification Sampling

Supply-chain hardening.

---

# 342. Phase 12 — Scale/Fuzz/DR

Production readiness.

---

# 343. Acceptance Tests

1. Cache keys are deterministic and versioned.
2. Every correctness-relevant input affects the key.
3. Cache key construction aligns with derivation semantics.
4. Host-irrelevant noise does not invalidate hermetic cache keys.
5. Platform portability is explicit and conservative.
6. Secret-consuming jobs are uncacheable by default.
7. Undeclared-network jobs are not trusted cache producers.
8. Cache hit can complete Job without fabricating JobAttempt.
9. Cache reuse links to original result/evidence.
10. Local developer cache cannot satisfy release-trusted requirements by default.
11. Untrusted runners cannot populate trusted cache namespaces.
12. Tenant A cannot observe/use Tenant B cache entries.
13. Cross-tenant shared cache is disabled by default.
14. Output digests are verified before cache publication.
15. Cache metadata is published only after output durability.
16. Partial uploads are never visible as hits.
17. Same key/different output triggers equivocation/nondeterminism handling.
18. Cache corruption becomes miss/quarantine, not silent success.
19. Remote cache outage falls back to execution.
20. Cache eviction never loses authoritative release artifacts.
21. Cache write failure does not fail a successful build unless explicitly required.
22. Impact analysis occurs before cache lookup.
23. Cache does not define semantic affectedness.
24. Stale security/test evidence cannot be hidden by cached artifacts.
25. Benchmark/deployment/signing jobs remain uncacheable by default.
26. RBE action cache obeys Forgeyard tenant/trust rules.
27. Negative caching is limited to safe deterministic failures.
28. Cache explain can identify semantic miss dimensions.
29. Full cache loss is recoverable by recomputation.
30. Cache verification detects digest mismatch.
31. Fill lease failure cannot deadlock future builds.
32. Cache pins/retention are explicit.
33. Standalone/distributed share cache semantics.
34. Air-gap cache bundles are integrity verified.
35. Forgeyard dogfoods remote/local cache for its own monorepo.

---

# 344. Production Readiness Gates

Do not call cache architecture production-ready until:

```text
cache-key derivation is stable
local/remote cache hierarchy works
tenant isolation passes
trusted-writer enforcement works
same-key/different-output detection works
cache explain identifies major miss causes
remote outage fallback works
eviction/GC never damages durable artifacts
RBE mapping is verified
cache loss/rebuild DR tests pass
```

---

# 345. Architectural Invariants

1. cache accelerates correctness; it does not define correctness;
2. cache keys bind every correctness-relevant input;
3. key schema is versioned;
4. derivation/cache semantics stay aligned;
5. platform portability is explicit;
6. secretful/impure work is non-cacheable by default;
7. cache hit never fabricates execution evidence;
8. reuse points to original trusted evidence;
9. trust class is explicit;
10. untrusted producers cannot write trusted cache;
11. tenant cache metadata is isolated;
12. cross-tenant sharing is opt-in;
13. outputs are content-addressed/verified;
14. metadata publishes only after durable output commit;
15. same key/different outputs is an incident;
16. cache corruption degrades to miss/quarantine;
17. cache outage does not stop correct execution;
18. eviction loses speed, not truth;
19. cache lookup occurs after semantic work selection;
20. cache never substitutes stale required evidence;
21. negative caching is tightly constrained;
22. RBE cache cannot bypass Forgeyard trust/policy;
23. cache explainability is first-class;
24. cache write failure is usually non-fatal to build correctness;
25. fill leases are optimization only;
26. cache is reconstructible;
27. cache backup is not correctness-critical;
28. air-gap imports verify trust/digests;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its own cache correctness model.

---

# 346. Final Target Architecture

```text
                    Job Semantics
                         │
                         ▼
                      CacheKey
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         Local        Remote         RBE
            │            │            │
            └────────────┼────────────┘
                         ▼
                  Candidate Entry
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
          Integrity     Trust     Provenance
             │           │           │
             └───────────┼───────────┘
                         ▼
                   Reuse Decision
                         │
             ┌───────────┼───────────┐
             ▼                       ▼
            HIT                     MISS
             │                       │
             ▼                       ▼
       Reuse Evidence             Execute
```

---

# 347. Final Architectural Position

Cache key:

```text
source
+
derivation
+
toolchain
+
dependencies
+
platform
+
environment
+
executor semantics
+
cache schema
  ↓
CacheKey
```

Cache hit:

```text
CacheKey
  ↓
candidate entry
  ↓
scope/trust/policy
  ↓
output digest verification
  ↓
reuse original result/evidence
```

Cache miss:

```text
no valid trusted entry
  ↓
execute normally
  ↓
commit outputs
  ↓
publish cache only after durability
```

The key guarantee is:

> **Forgeyard can aggressively accelerate development and CI without letting cache state become hidden correctness state. A cache hit is accepted only when input identity, output integrity, trust, tenant scope, and evidence lineage all match the requested work; otherwise Forgeyard simply executes the job.**

---

# 348. Extended Architecture Sequence

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
```
