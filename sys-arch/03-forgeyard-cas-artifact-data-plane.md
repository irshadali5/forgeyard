# 03 — Forgeyard CAS & Artifact Data Plane System Architecture

**Document type:** Core Infrastructure System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Content-addressed storage, artifact/data plane, immutable object model, local/tiered/cloud/P2P backends, transfer, replication, verification, retention, garbage collection, artifact manifests, source objects, logs/reports, cache integration, air-gap bundles, and data-plane observability  
**Architecture style:** Content-addressed, immutable, backend-neutral, metadata-separated, integrity-first  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on `01-forgeyard-core-domain-foundation.md` and `02-forgeyard-storage-metadata.md`. It assumes the previously defined VCS-neutral source snapshot architecture, hermetic/reproducible build architecture, Change Proposal model, and language ecosystems. It does not redefine those systems; it defines the shared byte/object data plane they use.

---

# 1. Purpose

Forgeyard needs a single, coherent data plane for all large or immutable build-related bytes.

This includes:

```text
source blobs
source trees
build inputs
build outputs
artifacts
package files
logs
test reports
coverage reports
SBOMs
VEX
provenance
attestations
reproduction evidence
toolchain objects
dependency archives
binary caches
release artifacts
air-gap bundles
```

The data plane must work in:

```text
MODE 1
local standalone

MODE 2
distributed team

MODE 3
enterprise / HA / multi-region
```

without changing domain semantics.

The central rule is:

> **Forgeyard stores immutable bytes by content identity and stores business meaning separately as metadata references.**

A second rule:

> **CAS verifies bytes; metadata describes what those bytes mean.**

A third rule:

> **P2P acceleration, local caches, cloud object stores, and remote mirrors are transport/storage implementations—not sources of semantic truth.**

---

# 2. Architectural Position

```text
                    Domain Services
                         │
                         ▼
                    Metadata Store
                         │
                         ▼
                    CAS ObjectRef
                         │
                         ▼
                  Forgeyard CAS API
                         │
        ┌────────────────┼─────────────────┐
        ▼                ▼                 ▼
     Local CAS      Object Storage     Iroh/P2P
        │                │                 │
        └────────────────┼─────────────────┘
                         ▼
                  Verified Immutable Bytes
```

---

# 3. Goals

The subsystem MUST:

1. use content-addressed immutable objects;
2. support BLAKE3 as default internal digest;
3. support SHA-256 aliases for interoperability;
4. support local standalone CAS;
5. support S3-compatible object storage;
6. support GCS;
7. support Azure Blob-compatible storage;
8. support Iroh/P2P acceleration;
9. support tiered lookup;
10. verify object integrity on read/write;
11. support resumable transfers;
12. support large objects;
13. support streaming;
14. support chunked transfer where useful;
15. support manifests/trees;
16. support source snapshot storage;
17. support artifact storage;
18. support log/report storage;
19. support action-cache references;
20. support replication;
21. support multi-region storage;
22. support retention;
23. support GC;
24. support legal hold/pinning;
25. support air-gap export/import;
26. support encryption in transit;
27. support tenant isolation policy;
28. support observability;
29. support repair/reconciliation;
30. avoid making any one backend authoritative by architecture.

---

# 4. Non-Goals

CAS is not:

```text
relational metadata database
VCS history database
policy engine
scheduler
artifact catalog
search engine
```

Those systems store references to CAS objects.

CAS should not know:

```text
"this object is a successful release"
```

unless metadata is supplied in generic object annotations outside identity.

---

# 5. Workspace Structure

```text
crates/cas/
├── forgeyard-cas/
├── forgeyard-cas-model/
├── forgeyard-cas-local/
├── forgeyard-cas-s3/
├── forgeyard-cas-gcs/
├── forgeyard-cas-azure/
├── forgeyard-cas-iroh/
├── forgeyard-cas-tiered/
├── forgeyard-cas-transfer/
├── forgeyard-cas-chunk/
├── forgeyard-cas-manifest/
├── forgeyard-cas-replication/
├── forgeyard-cas-gc/
├── forgeyard-cas-retention/
├── forgeyard-cas-integrity/
├── forgeyard-cas-airgap/
├── forgeyard-cas-health/
├── forgeyard-cas-metrics/
└── forgeyard-cas-testkit/
```

Related artifact layer:

```text
crates/artifact/
├── forgeyard-artifact/
├── forgeyard-artifact-model/
├── forgeyard-artifact-store/
├── forgeyard-artifact-manifest/
├── forgeyard-artifact-index/
├── forgeyard-artifact-retention/
├── forgeyard-artifact-download/
└── forgeyard-artifact-testkit/
```

---

# 6. `forgeyard-cas`

Primary backend-neutral interface.

It owns:

```text
put
get
exists
head/stat
delete internal/admin
stream
batch lookup
manifest closure access
health
```

It MUST NOT depend on:

```text
AWS SDK
GCS SDK
Azure SDK
Iroh implementation
filesystem-specific implementation
```

---

# 7. CAS Object Identity

```rust
pub struct CasObjectId {
    pub digest: Digest,
}
```

Usually:

```text
BLAKE3
```

Interop aliases:

```text
SHA-256
backend-native ETag/checksum
```

---

# 8. Object Reference

```rust
pub struct CasObjectRef {
    pub id: CasObjectId,
    pub size: ByteSize,
    pub media_type: Option<MediaType>,
}
```

Size is part of validation metadata but not necessarily primary identity.

---

# 9. Content Identity Rule

For raw blob object:

```text
CasObjectId =
BLAKE3(raw bytes)
```

For structured manifest/tree:

```text
CasObjectId =
BLAKE3(
    schema version
    +
    canonical encoded manifest
)
```

---

# 10. Immutable Object Rule

Once stored:

```text
object ID X
```

must always map to the same bytes.

If backend returns different bytes:

```text
IntegrityViolation
```

and backend/object is quarantined.

---

# 11. No Mutable Overwrite

CAS API must not expose:

```text
overwrite object X with new bytes
```

Mutable names belong in metadata:

```text
artifact alias
release name
cache key
source ref
```

---

# 12. Digest Algorithm

Default:

```text
BLAKE3
```

Reasons:

```text
high throughput
parallel hashing
streaming
strong modern cryptographic design
efficient Rust implementation
```

---

# 13. SHA-256 Interop

Needed for:

```text
OCI
Bazel RBE
SLSA/in-toto ecosystems
external checksum APIs
```

CAS can store:

```rust
pub struct DigestAliases {
    pub primary: Digest,
    pub aliases: Vec<Digest>,
}
```

---

# 14. Digest Alias Verification

Never trust declared alias.

Compute/verify:

```text
bytes
  ↓
BLAKE3
SHA-256
```

and store mapping only after verification.

---

# 15. Blob Object

```rust
pub struct BlobObject {
    pub id: CasObjectId,
    pub size: ByteSize,
}
```

No semantic meaning.

---

# 16. Manifest Object

Used to represent object collections.

```rust
pub struct CasManifest {
    pub schema: ManifestSchemaVersion,
    pub entries: Vec<ManifestEntry>,
}
```

---

# 17. Manifest Entry

```rust
pub struct ManifestEntry {
    pub name: ManifestName,
    pub object: CasObjectRef,
    pub role: ManifestRole,
}
```

---

# 18. Manifest Roles

Examples:

```text
file
tree
debug-symbols
signature
SBOM
provenance
log-chunk
test-report
package
```

Role is metadata inside manifest; object digest remains content identity.

---

# 19. Tree Object

Source/artifact directory trees:

```rust
pub struct TreeManifest {
    pub schema: TreeSchemaVersion,
    pub entries: Vec<TreeEntry>,
}
```

---

# 20. Tree Entry

```rust
pub struct TreeEntry {
    pub path: CanonicalPathComponent,
    pub kind: TreeEntryKind,
    pub object: Option<CasObjectRef>,
    pub executable: bool,
    pub symlink_target: Option<BoundedBytes>,
}
```

---

# 21. Canonical Tree Encoding

Rules:

```text
sorted entries
stable schema
stable path bytes
explicit entry kind
explicit executable bit
explicit symlink handling
```

No filesystem iteration order.

---

# 22. Source Snapshot Integration

VCS subsystem stores:

```text
source blobs
canonical source trees
snapshot manifests
```

in CAS.

Metadata DB stores:

```text
SourceSnapshotId
root tree ref
provenance ref
```

---

# 23. Hermetic Store Integration

Hermetic functional store uses CAS for:

```text
toolchains
dependency closures
fixed-output fetches
derivation outputs
binary substitutions
```

Do not create another independent blob store.

---

# 24. Artifact Integration

Artifact subsystem stores:

```text
ArtifactId
type
producer
retention
CAS ref
```

in metadata.

CAS stores actual artifact bytes.

---

# 25. Artifact Types

```rust
pub enum ArtifactKind {
    BuildOutput,
    Package,
    TestReport,
    CoverageReport,
    LogBundle,
    Sbom,
    Vex,
    Provenance,
    Attestation,
    DebugSymbols,
    ReproductionEvidence,
    SourceBundle,
    Custom(ArtifactKindId),
}
```

---

# 26. Artifact Identity vs CAS Identity

`ArtifactId`:

```text
business entity identity
```

`CasObjectId`:

```text
byte content identity
```

Two artifacts may point to same CAS object.

---

# 27. Deduplication

Automatic via content addressing.

If object already exists:

```text
verify expected metadata
return existing ref
```

No duplicate byte storage required.

---

# 28. Cross-Project Deduplication

Physically possible.

But multi-tenant deployments must consider:

```text
existence side channels
billing
privacy policy
retention
```

Logical access control remains tenant/project scoped.

---

# 29. Tenant Isolation

CAS namespace strategy can be:

```text
global content pool
+
metadata ACL
```

or:

```text
tenant-partitioned physical namespace
```

depending deployment security policy.

---

# 30. Default Multi-Tenant Recommendation

Enterprise hosted mode:

```text
logical tenant ACL
+
careful object existence API
+
optional tenant storage partition
```

Do not expose:

```text
"object exists globally"
```

to untrusted tenants.

---

# 31. CAS Backend Trait

```rust
#[async_trait]
pub trait CasBackend: Send + Sync {
    async fn put(
        &self,
        input: CasPutRequest,
    ) -> Result<CasObjectRef, CasError>;

    async fn get(
        &self,
        id: &CasObjectId,
    ) -> Result<CasRead, CasError>;

    async fn head(
        &self,
        id: &CasObjectId,
    ) -> Result<Option<CasObjectMetadata>, CasError>;

    async fn contains(
        &self,
        id: &CasObjectId,
    ) -> Result<bool, CasError>;
}
```

---

# 32. Streaming Read

```rust
pub struct CasRead {
    pub metadata: CasObjectMetadata,
    pub stream: CasByteStream,
}
```

Do not load large objects fully into memory.

---

# 33. Streaming Write

`CasPutRequest` should support stream input:

```rust
pub struct CasPutRequest {
    pub expected: Option<CasExpectedObject>,
    pub stream: CasInputStream,
}
```

---

# 34. Expected Object

```rust
pub struct CasExpectedObject {
    pub digest: Option<Digest>,
    pub size: Option<ByteSize>,
}
```

If supplied, write fails on mismatch.

---

# 35. Put Result

CAS computes authoritative digest while streaming.

---

# 36. Read Integrity

Configurable modes:

```rust
pub enum ReadVerification {
    Always,
    OnColdBackend,
    Sampled,
    BackendTrustedPlusPeriodicAudit,
}
```

Critical/release reads may force full verification.

---

# 37. Default Integrity Policy

Local/cold ingest:

```text
verify full digest
```

Subsequent reads:

```text
backend integrity + periodic/full verification policy
```

Do not silently trust object-store metadata as cryptographic proof.

---

# 38. Local CAS

```text
crates/cas/forgeyard-cas-local/
```

Tree:

```text
src/
├── lib.rs
├── backend.rs
├── layout.rs
├── file.rs
├── temp.rs
├── atomic.rs
├── fsync.rs
├── verify.rs
├── gc.rs
├── health.rs
└── error.rs
```

---

# 39. Local CAS Layout

Example:

```text
$FORGEYARD_DATA/cas/
├── objects/
│   └── blake3/
│       ├── ab/
│       │   └── cdef...
│       └── ...
├── temp/
├── quarantine/
└── metadata/
```

Sharding avoids huge single directories.

---

# 40. Atomic Local Write

Correct:

```text
write temp
  ↓
hash/verify
  ↓
fsync as required
  ↓
atomic rename into object path
```

If object already exists:

```text
verify/accept
```

---

# 41. Partial File Safety

Never make partially written object visible at canonical object path.

---

# 42. Local Fsync Policy

Configurable durability:

```rust
pub enum LocalDurability {
    BestEffort,
    Data,
    DataAndMetadata,
}
```

Standalone release artifacts should prefer stronger durability.

---

# 43. Local CAS Permissions

Use private application data permissions by default.

---

# 44. Local Quarantine

Corrupt object:

```text
move/quarantine
mark unhealthy
attempt recovery from another backend
```

---

# 45. S3-Compatible Backend

```text
forgeyard-cas-s3
```

Supports:

```text
AWS S3
MinIO
compatible object stores
```

where API behavior fits.

---

# 46. S3 Object Key

Derived from digest.

Example:

```text
objects/blake3/ab/cdef...
```

No mutable user filename as primary key.

---

# 47. S3 Metadata

Optional metadata:

```text
digest
size
schema
```

but authoritative integrity remains Forgeyard digest.

---

# 48. Multipart Upload

Large objects use multipart upload.

Need:

```text
resume state
part checks
abort stale multipart
```

---

# 49. Multipart Cleanup

Background reconciler removes abandoned uploads after TTL.

---

# 50. GCS Backend

Same semantic contract.

No GCS-specific fields leak into domain.

---

# 51. Azure Backend

Same semantic contract.

---

# 52. Object Backend Capability

```rust
pub struct CasBackendCapabilities {
    pub ranged_read: bool,
    pub multipart_upload: bool,
    pub server_side_copy: bool,
    pub conditional_put: bool,
    pub object_lock: bool,
}
```

---

# 53. Tiered CAS

```text
forgeyard-cas-tiered
```

Lookup order example:

```text
runner local L1
  ↓
site/local shared L2
  ↓
Iroh peer
  ↓
regional object store
  ↓
remote durable object store
```

---

# 54. Tier Rule

Correctness never depends on earlier tiers.

All returned bytes are digest verified according to policy.

---

# 55. Tiered Read

```text
for backend in policy order:
    try head/get
    verify
    optionally promote to faster tier
```

---

# 56. Tier Promotion

When remote object read:

```text
cloud
 ↓
runner local cache
```

may persist for future reads.

---

# 57. Tiered Write

Two strategies:

```text
write-through
write-back
```

---

# 58. Default Durable Write

For important durable artifacts:

```text
write local/temp
verify
write authoritative durable backend
acknowledge
```

Optionally seed faster tiers.

---

# 59. Write-Back Cache

May be used for noncritical cache objects.

But release/source/toolchain durability should not rely only on asynchronous write-back.

---

# 60. Authoritative Durable Backend

In enterprise mode, policy declares:

```text
durability class
```

not a hardcoded backend name.

---

# 61. Durability Classes

```rust
pub enum DurabilityClass {
    Ephemeral,
    Cache,
    Durable,
    ReleaseCritical,
}
```

---

# 62. Storage Policy

```rust
pub struct CasStoragePolicy {
    pub durability: DurabilityClass,
    pub replicas_required: u8,
    pub regions_required: u8,
    pub retention: RetentionPolicy,
}
```

---

# 63. Ephemeral Objects

Examples:

```text
temporary intermediate diagnostics
short-lived cache
```

May live only local/regional.

---

# 64. Release-Critical Objects

Examples:

```text
signed release package
SBOM
provenance
source snapshot
reproduction evidence
```

Require durable verified storage.

---

# 65. Iroh CAS Integration

```text
forgeyard-cas-iroh
```

Purpose:

```text
peer discovery
P2P transfer
local-network acceleration
WAN acceleration where useful
```

---

# 66. Iroh Is Not Authority

Never treat:

```text
peer has object
```

as equivalent to:

```text
object durably retained
```

unless separate durability policy explicitly guarantees peer retention—which Forgeyard should not assume.

---

# 67. P2P Read Flow

```text
need object X
  ↓
local miss
  ↓
trusted/eligible Iroh peers
  ↓
download
  ↓
verify digest X
  ↓
store local
```

---

# 68. P2P Security

Peers can provide malicious bytes.

Digest verification prevents content substitution.

Authorization still controls whether peer is allowed to request/serve object.

---

# 69. P2P Metadata Privacy

Do not broadcast sensitive project object IDs indiscriminately in multi-tenant deployments.

---

# 70. Chunking

Chunking can improve:

```text
resume
parallel transfer
large-object dedup
partial recovery
```

but increases complexity.

---

# 71. Chunking Policy

Do not chunk every tiny object.

Possible threshold:

```text
large object > configured threshold
```

---

# 72. Chunk Manifest

```rust
pub struct ChunkedObjectManifest {
    pub schema: ChunkSchemaVersion,
    pub total_size: ByteSize,
    pub chunks: Vec<ChunkRef>,
}
```

---

# 73. Chunk Ref

```rust
pub struct ChunkRef {
    pub offset: u64,
    pub length: ByteSize,
    pub object: CasObjectRef,
}
```

---

# 74. Whole Object Identity

Even when chunked:

```text
whole-object digest
```

must be computed/verified.

Chunks are transport/storage optimization.

---

# 75. Content-Defined Chunking

Optional future optimization.

Do not require initially.

Fixed chunks are simpler.

---

# 76. Resumable Download

Client tracks:

```text
object ID
verified ranges/chunks
temporary file
```

Resume only verified chunks.

---

# 77. Resumable Upload

Backend-specific multipart/session abstraction.

---

# 78. Transfer Protocol

```text
forgeyard-cas-transfer
```

Internal agent/daemon transfer can use QUIC/Postcard control + streaming bytes.

---

# 79. Transfer Control Messages

Examples:

```rust
pub enum CasTransferMessage {
    NeedObjects(NeedObjects),
    ObjectAvailable(ObjectAvailable),
    BeginUpload(BeginUpload),
    UploadAccepted(UploadAccepted),
    TransferComplete(TransferComplete),
}
```

---

# 80. Batch Missing Check

Before transfer:

```text
client sends object IDs
server returns missing subset
```

reduces duplicate transfer.

---

# 81. Bloom Filters

Optional optimization for large peer/object sets.

Never correctness authority.

False positives must fall back to normal lookup.

---

# 82. Range Reads

Useful for:

```text
large archives
debug symbols
log bundles
```

but only if consumer can validate/use safely.

---

# 83. Compression

CAS object identity should be based on logical raw content unless format itself is intentionally compressed artifact.

Transport may compress independently.

---

# 84. Transparent Compression

Backend may store compressed representation internally if it can reproduce exact logical bytes.

But this complicates ranged reads and implementation.

Initial recommendation:

```text
store exact bytes
```

for simplicity.

---

# 85. Archive Artifacts

If user artifact is `.tar.zst`:

```text
compressed bytes themselves
```

are content identity.

Do not transparently unpack and re-identify unless artifact manifest says so.

---

# 86. Artifact Manifest

```rust
pub struct ArtifactManifest {
    pub schema: ArtifactManifestVersion,
    pub primary: CasObjectRef,
    pub attachments: Vec<ArtifactAttachment>,
}
```

---

# 87. Attachments

Examples:

```text
debug symbols
SBOM
signature
provenance
source map
test metadata
```

---

# 88. Multi-File Artifact

Example desktop release:

```text
binary
license
config template
debug symbols
```

represented by tree/manifest.

---

# 89. Package Identity

Package file digest is CAS identity.

Package semantic identity:

```text
name
version
target
format
```

belongs in artifact/package metadata.

---

# 90. Build Output Capture

Runner:

```text
declared output paths
  ↓
canonical capture
  ↓
files/trees hashed
  ↓
CAS upload
  ↓
output manifest
```

---

# 91. Undeclared Outputs

Hermetic system may ignore or flag them.

Do not silently store entire workspace as output.

---

# 92. Output Tree

Directory output becomes canonical tree object.

---

# 93. Symlink Policy

Artifact tree captures symlink semantics explicitly.

Unsafe symlinks may be rejected depending package/output policy.

---

# 94. File Metadata

CAS tree identity should include only semantically intended metadata:

```text
path
entry type
executable bit
symlink target
content
```

Do not include:

```text
mtime
uid
gid
```

unless explicitly required.

---

# 95. Reproducibility

Canonical tree avoids runner-specific metadata.

---

# 96. Log Data Plane

Logs can be large/high-volume.

Store them as chunked append-style objects/manifests.

---

# 97. Log Stream Model

```rust
pub struct LogStreamRef {
    pub stream_id: LogStreamId,
    pub manifest: CasObjectRef,
    pub first_seq: LogSeq,
    pub last_seq: LogSeq,
}
```

---

# 98. Log Chunk

```rust
pub struct LogChunk {
    pub first_seq: LogSeq,
    pub last_seq: LogSeq,
    pub object: CasObjectRef,
}
```

---

# 99. Log Ordering

Sequence number is authoritative.

Wall-clock timestamps are supplementary.

---

# 100. Live Logs

Live stream:

```text
agent -> daemon/UI
```

while chunks are periodically durably flushed.

---

# 101. Reconnect

Client can request:

```text
from sequence N
```

Daemon fetches recent/live or CAS chunks.

---

# 102. Test Reports

Structured parsed summary in metadata DB.

Full raw report:

```text
CAS
```

---

# 103. Coverage Reports

Same pattern.

---

# 104. SBOM / Provenance

Store immutable documents in CAS.

Metadata DB stores:

```text
subject
format
digest
producer
```

---

# 105. Signatures

Signature bytes may be CAS objects.

But private signing keys never CAS objects.

---

# 106. Cache Integration

Action cache maps:

```text
ActionKey -> OutputManifestRef
```

Metadata/cache index separate from CAS bytes.

---

# 107. Cache Hit Validation

On cache hit:

```text
lookup mapping
  ↓
verify output objects available
  ↓
optionally integrity check
```

Missing object invalidates cache entry.

---

# 108. Cache Poisoning Prevention

Never trust unverified remote cache bytes.

Check:

```text
expected digest
manifest
policy trust
```

---

# 109. Cross-Tenant Cache

Default:

```text
disabled
```

for untrusted multi-tenant environments.

Optional trusted organization-wide cache with policy.

---

# 110. Cache Trust Level

```rust
pub enum CacheTrust {
    LocalTrusted,
    OrganizationTrusted,
    ExternalUntrusted,
}
```

---

# 111. Toolchain Objects

Toolchains can be huge.

Store:

```text
toolchain archive/tree
manifest
identity metadata
```

in CAS.

---

# 112. Toolchain Installation

Runner:

```text
need ToolchainId
  ↓
resolve manifest
  ↓
fetch CAS closure
  ↓
materialize immutable store path
```

---

# 113. Fixed-Output Fetches

Hermetic fetcher computes known digest and imports into CAS.

---

# 114. Dependency Archives

Downloaded source archives/packages may be CAS objects.

---

# 115. Source Mirror

VCS source blobs become CAS objects, so origin outage does not block already-imported builds.

---

# 116. Replication

```text
forgeyard-cas-replication
```

Responsible for durability copies.

---

# 117. Replication Record

Metadata store may track:

```rust
pub struct CasReplicaState {
    pub object: CasObjectId,
    pub backend: CasBackendId,
    pub region: RegionId,
    pub state: ReplicaState,
}
```

---

# 118. Replica States

```rust
pub enum ReplicaState {
    Pending,
    Available,
    Degraded,
    Missing,
    Quarantined,
}
```

---

# 119. Replication Is Metadata

CAS backend itself only knows objects.

Replication controller tracks desired durability.

---

# 120. Replication Workflow

```text
object committed
  ↓
durability policy evaluated
  ↓
replication tasks
  ↓
copy
  ↓
verify digest
  ↓
mark replica available
```

---

# 121. Server-Side Copy

Can optimize same-cloud-region copy.

Still verify final object identity/metadata.

---

# 122. Multi-Region

Policy may require:

```text
2 regions
```

for release-critical data.

---

# 123. Region Awareness

```rust
pub struct CasBackendDescriptor {
    pub id: CasBackendId,
    pub region: Option<RegionId>,
    pub durability: BackendDurability,
    pub cost_class: CostClass,
}
```

---

# 124. Read Locality

Scheduler can prefer runner near object replicas.

---

# 125. Data Locality Score

Scheduler input:

```text
required input digests
available regional/local replicas
```

Optimization only.

---

# 126. Replication Lag

Observable.

Release policy may require desired replicas before promotion.

---

# 127. Retention

```text
forgeyard-cas-retention
```

Objects are retained because metadata roots reference them.

---

# 128. Retention Roots

Examples:

```text
active run
recent run
pinned artifact
release
source snapshot
toolchain
audit/legal hold
cache
```

---

# 129. Retention Policy

```rust
pub enum RetentionPolicy {
    Ephemeral { ttl: Duration },
    Cache { ttl: Duration },
    Project { ttl: Duration },
    Pinned,
    Release,
    LegalHold,
}
```

---

# 130. Reference Counting

Do not rely exclusively on immediate reference counts in distributed systems.

GC should use mark/sweep or root-derived liveness with grace periods.

---

# 131. GC Architecture

```text
forgeyard-cas-gc
```

Phases:

```text
snapshot roots
  ↓
mark reachable
  ↓
wait grace
  ↓
recheck
  ↓
sweep
```

---

# 132. Why Grace Period

Protects against:

```text
metadata/CAS race
in-flight upload
eventual replication
temporary orphan
```

---

# 133. GC Epoch

```rust
pub struct GcEpoch(U64);
```

Can help mark consistency.

---

# 134. GC Root Snapshot

Metadata store supplies consistent root view.

---

# 135. Recursive Mark

Manifests/trees/chunk manifests reference child objects.

GC follows closure.

---

# 136. Cycle Handling

CAS manifests should ideally be DAGs.

But marker must protect against cycles/malformed manifests.

---

# 137. Manifest Trust

Only parse manifests of known schema/media type.

Untrusted arbitrary blobs are not traversed.

---

# 138. GC Safety

Never delete object if:

```text
legal hold
pinned
release critical
recently written within grace
replication repair pending
```

---

# 139. Cache GC

Cache policy may evict mappings independently.

CAS bytes become collectible only if no other roots.

---

# 140. Local LRU Eviction

Runner-local L1 cache can use:

```text
LRU/size pressure
```

independently from durable CAS GC.

---

# 141. Local Cache Is Not Durability

Deleting L1 is safe if durable backend exists.

---

# 142. Offline Standalone

In Mode 1, local CAS may be only durable store.

GC must therefore respect all local metadata roots.

---

# 143. Standalone CAS Backup

Backup can include:

```text
all rooted objects
manifest
metadata backup
```

---

# 144. Air-Gap Architecture

```text
forgeyard-cas-airgap
```

Exports selected object closure.

---

# 145. Air-Gap Bundle

```rust
pub struct AirgapBundleManifest {
    pub schema: AirgapSchemaVersion,
    pub roots: Vec<CasObjectRef>,
    pub objects: Vec<CasObjectRef>,
    pub provenance: Vec<CasObjectRef>,
}
```

---

# 146. Export Flow

```text
select roots
  ↓
walk closure
  ↓
verify
  ↓
write deterministic bundle
  ↓
sign bundle manifest optionally
```

---

# 147. Import Flow

```text
verify bundle
  ↓
verify every object digest
  ↓
ingest CAS
  ↓
register metadata if authorized
```

---

# 148. Bundle Format

Possible:

```text
tar.zst
Forgeyard-native packed stream
```

Transport format not object identity.

---

# 149. Bundle Reproducibility

Bundle ordering/timestamps should be deterministic where practical.

---

# 150. Integrity Service

```text
forgeyard-cas-integrity
```

Responsibilities:

```text
full scan
sample scan
manifest verification
replica comparison
repair
quarantine
```

---

# 151. Integrity States

```rust
pub enum ObjectIntegrity {
    Unknown,
    Verified,
    Corrupt,
    Missing,
    Quarantined,
}
```

---

# 152. Integrity Audit

Enterprise deployments should periodically verify cold objects.

---

# 153. Bit Rot

Detected by digest mismatch.

Repair from healthy replica.

---

# 154. Replica Repair

```text
replica A corrupt
replica B healthy
  ↓
copy B -> A
  ↓
verify
```

---

# 155. No Healthy Replica

If object required:

```text
critical data loss alert
```

Do not fabricate.

---

# 156. Quarantine

Corrupt backend/object path is isolated from normal reads.

---

# 157. Backend Health

```rust
pub enum CasHealth {
    Healthy,
    Degraded,
    ReadOnly,
    Unavailable,
}
```

---

# 158. Read-Only Backend

Useful during:

```text
maintenance
credential issue
capacity incident
```

Tiered CAS can continue reads.

---

# 159. Backpressure

CAS writes must apply bounded concurrency.

Avoid saturating:

```text
disk
network
object-store requests
memory
```

---

# 160. Transfer Scheduler

Can prioritize:

```text
job critical input
live log
release artifact
background replication
GC scan
```

---

# 161. Priority Classes

```rust
pub enum TransferPriority {
    Interactive,
    JobCritical,
    ReleaseCritical,
    Normal,
    Background,
}
```

---

# 162. Bandwidth Limits

Per runner/site/backend.

---

# 163. Concurrency Limits

Per:

```text
tenant
backend
runner
object size class
```

---

# 164. Memory Limits

Streaming buffer size bounded.

---

# 165. Zero-Copy / Efficient I/O

Where possible:

```text
sendfile/splice
io_uring optional
mmap carefully
```

are performance optimizations.

Not correctness dependencies.

---

# 166. Tokio / Rayon Split

Tokio:

```text
network
async filesystem orchestration
object-store requests
```

Rayon:

```text
hashing large buffers
manifest canonicalization
compression if CPU-heavy
```

Use bounded pools.

---

# 167. I/O Uring

Optional Linux optimization behind:

```text
io-uring
```

feature.

Fallback must remain correct.

---

# 168. Chunk Hash Parallelism

Parallel compute allowed while preserving order/whole hash.

---

# 169. Temporary Disk

Large transfers may need staging.

Runner capability reports:

```text
temp disk available
```

---

# 170. Streaming Without Staging

Prefer when backend/protocol supports verified streaming safely.

But if output digest unknown until end, final visibility must wait for successful completion.

---

# 171. Upload Intent

Artifact service may create:

```rust
pub struct UploadIntent {
    pub id: UploadIntentId,
    pub expected_size: Option<ByteSize>,
    pub expected_digest: Option<Digest>,
    pub expires_at: Timestamp,
}
```

---

# 172. Upload Intent Is Metadata

CAS itself can accept direct puts.

Upload intent coordinates user/agent API lifecycle.

---

# 173. Direct-to-Object-Store Upload

Possible optimization:

```text
daemon creates scoped upload authorization
client uploads directly
daemon verifies completion/digest
```

---

# 174. Presigned URLs

May be used for compatible object stores.

But:

```text
short TTL
scoped object key
expected size/digest
no list permission
```

---

# 175. Client Trust

Never trust client "upload complete."

Server verifies object metadata/content digest according to policy.

---

# 176. Download Authorization

Artifact service authorizes access first.

Then may return:

```text
proxied stream
short-lived signed download
```

---

# 177. CAS API Is Not Public Artifact API

Users should not generally fetch arbitrary digest if they know it.

Access goes through artifact/source authorization.

---

# 178. Object Enumeration

CAS backend API for admin/internal may enumerate.

Do not expose globally to tenant users.

---

# 179. Media Type

```rust
pub struct MediaType(BoundedString);
```

Examples:

```text
application/octet-stream
application/vnd.forgeyard.tree+postcard
application/vnd.forgeyard.manifest+postcard
```

---

# 180. Schema Registry

Structured CAS objects should have explicit media/schema IDs.

---

# 181. Postcard Structured Objects

Internal manifests may serialize using canonical/versioned Postcard-compatible representation.

But default serde output must not be assumed canonical without explicit design.

---

# 182. Canonical Manifest Encoding

Implement dedicated encoder if object identity depends on it.

---

# 183. Manifest Upgrade

Old manifest schema must remain readable while retained.

New schema gets new object ID.

---

# 184. Source Tree Upgrade

Changing tree schema produces new tree/snapshot identity.

Migration must be explicit.

---

# 185. Artifact Download Naming

Friendly filename is metadata:

```text
forgeyard-linux-x86_64.tar.zst
```

not CAS key.

---

# 186. Content-Disposition

Artifact API can provide desired filename safely.

---

# 187. Duplicate Friendly Names

Allowed if underlying artifact IDs differ.

---

# 188. Artifact Index

Metadata DB supports searches:

```text
project
run
job
type
target
created_at
release
```

CAS does not.

---

# 189. Artifact Lifecycle

```text
Pending
Available
Quarantined
Expired
Deleted
```

Bytes may remain until GC after logical deletion.

---

# 190. Artifact Quarantine

Security scanner or integrity failure can block download/promotion.

---

# 191. Malware/Policy Scan

Artifact service can attach scan evidence.

CAS stores bytes/evidence.

---

# 192. Retention Override

Release promotion can upgrade artifact retention:

```text
Cache -> Release
```

without rewriting bytes.

---

# 193. Same Bytes, New Meaning

Same object can be:

```text
build artifact
release artifact
```

through different metadata references.

---

# 194. Artifact Lineage

Metadata tracks:

```text
producer job
source snapshot
derivation
parent artifact
```

not CAS itself.

---

# 195. Provenance Closure

Release verification can fetch:

```text
artifact
SBOM
provenance
source snapshot
toolchain refs
```

through metadata + CAS closure.

---

# 196. Data Plane Security

Threats:

```text
content substitution
corrupt backend
unauthorized object access
cross-tenant existence leak
path traversal in materialization
archive bomb
malicious manifest
resource exhaustion
P2P malicious peer
presigned URL leakage
```

---

# 197. Content Substitution Defense

Digest verification.

---

# 198. Unauthorized Access Defense

Artifact/source authorization outside raw CAS.

---

# 199. Path Traversal Defense

Tree materialization validates canonical paths.

---

# 200. Archive Bomb Defense

Archive extraction is separate safe unpacking subsystem with size/file count limits.

---

# 201. Manifest Bomb Defense

Manifest parser limits:

```text
max entries
max depth
max total closure
```

---

# 202. Recursive Closure Limits

Prevent malicious manifest DAG causing unbounded traversal.

---

# 203. P2P Peer Authorization

Peers authenticate via Forgeyard/transport identity.

Do not allow arbitrary internet peer to query private CAS.

---

# 204. Signed URL Leakage

Short TTL and narrow scope reduce impact.

Audit issuance for sensitive release artifacts.

---

# 205. Encryption in Transit

```text
QUIC/TLS
HTTPS
cloud backend TLS
```

required in distributed production.

---

# 206. Encryption at Rest

Delegated to:

```text
disk encryption
object-store server-side encryption
KMS-backed storage policy
```

Forgeyard tracks policy but does not invent custom crypto storage format initially.

---

# 207. Client-Side Encryption

Optional future capability for highly sensitive artifacts.

Complex because it affects dedup/search/range access.

Not default architecture.

---

# 208. Secret Data

Secrets must never be uploaded into normal CAS by design.

---

# 209. Secret Scan

Artifact/source scan may detect accidental secrets and quarantine according to policy.

---

# 210. Logs and Secrets

Log redaction occurs before durable storage where possible.

But treat logs as potentially sensitive.

---

# 211. Audit

CAS admin operations:

```text
delete
repair
quarantine
air-gap export
retention pin
```

must be audited.

---

# 212. Normal Reads

Do not audit every internal byte read at high volume unless compliance requires it.

Use structured access logs selectively.

---

# 213. Metrics

Examples:

```text
cas_put_bytes
cas_get_bytes
cas_put_latency
cas_get_latency
cas_hit
cas_miss
cas_integrity_failure
cas_replication_lag
cas_gc_mark_count
cas_gc_delete_count
cas_local_cache_hit
cas_p2p_hit
cas_backend_errors
```

---

# 214. Backend Metrics

Tag by:

```text
backend ID
region
operation
status
```

Avoid high-cardinality object IDs in metrics.

---

# 215. Tracing

Spans:

```text
cas.head
cas.get
cas.put
cas.verify
cas.replicate
cas.gc.mark
cas.gc.sweep
cas.airgap.export
```

---

# 216. Logging

Logs include object digest prefix only if useful and safe.

No secret URLs.

---

# 217. Health Check

Per backend:

```text
connectivity
read test
write test optional
latency
credential validity
capacity
```

---

# 218. Readiness

Daemon can remain ready if one non-authoritative cache tier fails.

Not ready if required durable CAS is unavailable for operations that require writes.

---

# 219. Degraded Mode

Example:

```text
Iroh down
S3 healthy
```

system remains healthy/degraded performance only.

---

# 220. Critical Degraded Mode

Example:

```text
durable backend read-only
```

allow reads/builds from existing data but block new durable release artifacts if policy requires.

---

# 221. Capacity Monitoring

Local CAS:

```text
free bytes
used bytes
inode pressure
```

Cloud:

```text
quota/API throttling/cost metrics
```

---

# 222. Disk Pressure

Local cache eviction before catastrophic full disk.

---

# 223. Reserved Space

Keep emergency reserve to allow metadata/logging/cleanup.

---

# 224. File Descriptor Pressure

Large parallel transfers must use bounded descriptors.

---

# 225. Object Count Scaling

Directory layout and DB indexes must handle millions/billions of objects in enterprise scale.

---

# 226. Backend Enumeration Scaling

GC should not rely on naive full listing from slow backend if metadata/mark indexes can reduce work.

---

# 227. Mark Index

Optional DB/state can track known object refs and last seen root epoch.

But CAS identity remains independent.

---

# 228. GC Source of Truth

Metadata roots + known structured object graph.

Not raw backend last-access timestamp.

---

# 229. Access Timestamp

Avoid updating "last accessed" metadata for every read if it creates heavy write load.

L1 cache can track locally.

---

# 230. Retention by Metadata

Better than access-based GC for durable artifacts.

---

# 231. Background Tasks

```text
replication
integrity scan
GC
orphan multipart cleanup
tier promotion cleanup
local cache eviction
```

run through worker/reconciliation subsystem.

---

# 232. Task Idempotency

All background tasks must tolerate retry.

---

# 233. Replication Idempotency

Copying existing object to target is safe after digest verification.

---

# 234. GC Idempotency

Deleting already-missing eligible object is success/no-op.

---

# 235. Repair Idempotency

Repeated repair should converge.

---

# 236. CAS Error Model

```rust
pub enum CasError {
    NotFound,
    AlreadyExists,
    DigestMismatch,
    SizeMismatch,
    Corrupt,
    Quarantined,
    PermissionDenied,
    Unavailable,
    Timeout,
    CapacityExceeded,
    RateLimited,
    Unsupported,
    InvalidManifest,
    Internal,
}
```

---

# 237. Retry Classification

```text
NotFound -> no retry unless replica lookup
RateLimited -> backoff
Unavailable -> backoff/reconcile
DigestMismatch -> no blind retry from same source
CapacityExceeded -> reroute/operator
Corrupt -> quarantine/repair
```

---

# 238. Backend Error Translation

Cloud-specific errors stay adapter-local.

---

# 239. Download Fallback

Tiered backend:

```text
backend A corrupt
  ↓
quarantine
  ↓
try backend B
```

---

# 240. Upload Fallback

If durable backend A unavailable and policy allows B:

```text
reroute
```

If required replication impossible:

```text
object may be temporarily available but durability requirement unmet
```

Metadata should expose state.

---

# 241. Durability State

```rust
pub enum ObjectDurabilityState {
    Pending,
    Satisfied,
    Degraded,
    Violated,
}
```

---

# 242. Release Gate

Release promotion may require:

```text
DurabilityState::Satisfied
```

for all critical objects.

---

# 243. Source Durability

Imported source snapshots needed for reproducibility should be durable according to project policy.

---

# 244. Cache Object Durability

May remain `Cache` class.

---

# 245. Artifact Manifest Example

```ron
(
    schema: 1,

    artifact: (
        primary: (
            digest: "blake3:...",
            size: 12582912,
        ),

        attachments: [
            (
                role: "sbom",
                digest: "blake3:...",
            ),
            (
                role: "debug-symbols",
                digest: "blake3:...",
            ),
        ],
    ),
)
```

RON here is illustrative/debug representation; internal manifest identity uses canonical binary encoding.

---

# 246. CAS Configuration Example

```ron
(
    cas: (
        backends: [
            (
                id: "local",
                kind: Local,
                path: "/var/lib/forgeyard/cas",
            ),
            (
                id: "durable",
                kind: S3,
                bucket: "forgeyard-cas",
                region: "region-a",
                credential: Secret("cas/s3"),
            ),
        ],

        tiers: [
            "local",
            "iroh",
            "durable",
        ],
    ),
)
```

---

# 247. Standalone Configuration

```ron
(
    cas: (
        backends: [
            (
                id: "local",
                kind: Local,
                path: Auto,
            ),
        ],

        durable_backend: "local",
    ),
)
```

---

# 248. Enterprise Configuration

```ron
(
    cas: (
        durable: [
            "region-a",
            "region-b",
        ],

        p2p: (
            enabled: true,
        ),

        replication: (
            release_critical: (
                regions_required: 2,
            ),
        ),
    ),
)
```

---

# 249. CAS CLI

```text
forgeyard cas status
forgeyard cas doctor
forgeyard cas put
forgeyard cas get
forgeyard cas head
forgeyard cas verify
forgeyard cas replicate
forgeyard cas repair
forgeyard cas gc
forgeyard cas pin
forgeyard cas unpin
forgeyard cas export
forgeyard cas import
forgeyard cas stats
```

---

# 250. `cas put`

Admin/dev command.

Normal artifact/source ingestion goes through subsystem services.

---

# 251. `cas verify`

Modes:

```text
object
manifest closure
backend sample
full backend
```

---

# 252. `cas gc`

Support:

```text
plan
mark
dry-run
sweep
```

Production should default to dry-run/plan before destructive first use.

---

# 253. `cas repair`

Find healthy replica and restore corrupt/missing target.

---

# 254. `cas stats`

Shows:

```text
objects
bytes
backend usage
replication
GC candidates
local hit rate
P2P hit rate
```

---

# 255. CAS Doctor

Categories:

```text
REQUIRED
OPTIONAL
PERFORMANCE
SECURITY
```

Examples:

```text
REQUIRED: durable backend writable
SECURITY: TLS verification disabled
PERFORMANCE: local cache unavailable
OPTIONAL: Iroh disabled
```

---

# 256. Artifact CLI

```text
forgeyard artifacts list
forgeyard artifacts show
forgeyard artifacts download
forgeyard artifacts verify
forgeyard artifacts pin
forgeyard artifacts provenance
```

---

# 257. Public Artifact API

Potential:

```text
GET /v1/artifacts/{id}
GET /v1/artifacts/{id}/download
GET /v1/artifacts/{id}/manifest
GET /v1/artifacts/{id}/provenance
```

No raw unauthenticated CAS digest API.

---

# 258. Internal CAS API

Internal daemon/agent:

```text
HEAD object
GET object
PUT object
batch missing
transfer session
```

over authenticated transport.

---

# 259. Direct Worker Access

Agent may access object store directly via scoped credentials/presigned requests.

But metadata/business authorization remains daemon-driven.

---

# 260. Credential Scope

Runner credentials should permit only:

```text
specific prefix/object
short duration
required operation
```

where backend supports it.

---

# 261. Build Input Materialization

Runner flow:

```text
job lease
  ↓
input manifest
  ↓
batch missing local
  ↓
fetch CAS
  ↓
verify
  ↓
materialize read-only source/toolchain/input paths
```

---

# 262. Output Upload

```text
execute
  ↓
capture declared outputs
  ↓
hash/tree
  ↓
upload CAS
  ↓
verify
  ↓
send output refs
  ↓
daemon transactionally commits job result metadata
```

---

# 263. Lost Agent During Upload

Objects already uploaded may remain orphaned.

GC handles them after grace.

Job result not committed without valid lease/attempt.

---

# 264. Duplicate Upload

Safe.

Same bytes -> same ID.

---

# 265. Stale Job Completion

Even if CAS upload succeeds, daemon rejects stale lease completion.

Objects remain unreferenced/GC-eligible.

---

# 266. Build Cache Publish

Only publish action-cache mapping after all output objects verified/available.

---

# 267. Reproduction Output

Independent reproducer writes outputs to same CAS.

If bytes equal:

```text
same CAS IDs
```

naturally.

---

# 268. Repro Mismatch

Different output objects preserved for diff/quarantine evidence.

---

# 269. Binary Diff Evidence

Store diff reports in CAS.

---

# 270. Release Promotion

Promotion changes metadata/retention/policy state.

Does not copy/rebuild bytes unnecessarily.

---

# 271. Cross-Region Promotion

May require replication before promotion completes.

---

# 272. Signing

Unsigned object:

```text
CAS A
```

Signed object:

```text
CAS B
```

Signature changes bytes, therefore new content identity.

---

# 273. Notarization

Notarized/stapled artifact likewise becomes new object if bytes change.

---

# 274. Provenance Relationship

Metadata links:

```text
unsigned -> signed -> notarized
```

---

# 275. OCI Integration

OCI layers/manifests may be stored/aliased via SHA-256.

Forgeyard CAS can maintain internal BLAKE3 alias.

---

# 276. RBE Integration

Bazel RBE typically uses SHA-256 digests.

Adapter maps:

```text
SHA-256 RBE digest
↔
Forgeyard object alias
```

---

# 277. Alias Lookup

Efficient alias index may live in metadata DB.

CAS backend does not need separate duplicate object.

---

# 278. Alias Collision Safety

Cryptographic collisions are treated as catastrophic integrity events.

Never map two different byte contents to same alias.

---

# 279. Backend Migration

Moving from local/S3 provider A to provider B:

```text
replicate objects
verify
update backend policy
drain old
```

No domain object IDs change.

---

# 280. Backend Re-Key/Encryption Change

Storage implementation may rewrite physical object bytes encrypted at rest, while logical CAS content stays same.

---

# 281. Region Migration

Same.

---

# 282. Cost Optimization

Tiered policy may move cold cache objects to cheaper backend.

Release-critical retention policy remains stronger.

---

# 283. Storage Class

Cloud-specific storage classes stay adapter configuration.

---

# 284. Cold Retrieval

If object in archival storage:

```text
restore pending
```

must be represented explicitly.

Do not pretend instant availability.

---

# 285. Availability State

```rust
pub enum ObjectAvailability {
    Immediate,
    Restoring,
    Unavailable,
}
```

---

# 286. Scheduler Awareness

Do not schedule job requiring cold object until available unless workflow tolerates delay.

---

# 287. Prewarming

Before release/test wave:

```text
prewarm toolchains/dependencies
```

to runner/site caches.

---

# 288. Predictive Cache

Can suggest prefetch.

Optimization only.

---

# 289. Site Cache

Optional shared LAN CAS proxy.

Same backend trait.

---

# 290. Edge Runner

Remote runners can use local site cache with central durable fallback.

---

# 291. Disconnected Runner

A fully offline runner can execute only if required CAS closure already present/imported.

---

# 292. Air-Gap Runner

Uses imported bundle/local CAS.

---

# 293. Source Snapshot Bundle

Air-gap source bundle includes source tree closure and provenance.

---

# 294. Toolchain Bundle

Same for toolchains/dependencies.

---

# 295. Full Build Bundle

Can combine:

```text
source
toolchain
dependencies
pipeline inputs
```

for disconnected reproducibility.

---

# 296. Object Graph

Generic structured objects form DAG:

```text
root manifest
 ├── object A
 ├── tree B
 │    ├── blob C
 │    └── blob D
 └── evidence E
```

---

# 297. Graph Traversal API

```rust
pub trait CasGraph {
    async fn children(
        &self,
        object: &CasObjectId,
    ) -> Result<Vec<CasObjectRef>, CasError>;
}
```

Only for known structured object types.

---

# 298. Closure Export

```text
root(s)
  ↓
traverse
  ↓
deduplicate
  ↓
ordered export
```

---

# 299. Closure Size Calculation

Useful before:

```text
air-gap export
replication
download
```

---

# 300. Storage Quotas

Optional per tenant/project:

```text
logical artifact usage
cache usage
release storage
```

---

# 301. Physical vs Logical Usage

Dedup means physical bytes ≠ logical billed usage.

Define policy explicitly.

---

# 302. Default Quota Accounting

For SaaS/enterprise:

```text
logical referenced bytes
```

may be easier/fairer than physical dedup share.

---

# 303. Quota Enforcement

Do not block critical release retention unexpectedly.

Use warnings/reserved policy.

---

# 304. Cache Quota

Evict cache first.

---

# 305. Release Quota

Requires explicit operator/admin policy.

---

# 306. Legal Hold

CAS object becomes non-GC while any legal-hold metadata root exists.

---

# 307. Object Deletion API

Raw delete should be restricted to:

```text
GC
repair/admin
```

Normal users delete metadata references.

---

# 308. Secure Erasure

Cloud/object stores may not guarantee immediate physical erasure due to replication/versioning.

Document backend semantics.

---

# 309. Versioned Buckets

If object-store versioning enabled, GC may need lifecycle configuration to remove old physical versions eventually.

---

# 310. Object Lock

Cloud WORM/object-lock can protect release/audit artifacts optionally.

Do not require for all deployments.

---

# 311. Supply Chain WORM

High-assurance deployment may store provenance/signatures in immutable storage class.

---

# 312. CAS Testkit

```text
forgeyard-cas-testkit/src/
├── lib.rs
├── put_get.rs
├── duplicate.rs
├── integrity.rs
├── stream.rs
├── range.rs
├── large.rs
├── concurrent.rs
├── missing.rs
├── corruption.rs
├── manifest.rs
├── gc.rs
└── replication.rs
```

---

# 313. Backend Conformance

Every backend must pass:

1. put/get exact bytes;
2. duplicate put;
3. missing object;
4. digest mismatch;
5. large streaming object;
6. concurrent put same ID;
7. interrupted upload;
8. read verification;
9. conditional semantics;
10. health/error translation.

---

# 314. Tiered CAS Tests

Test:

```text
L1 miss/L2 hit
corrupt L1 fallback
P2P hit
durable fallback
promotion
```

---

# 315. GC Tests

1. rooted object retained;
2. child of rooted manifest retained;
3. orphan retained during grace;
4. orphan deleted after grace;
5. legal hold retained;
6. concurrent new reference protected.

---

# 316. Replication Tests

1. required replica count achieved;
2. corrupt replica repaired;
3. region outage degrades state;
4. release gate blocks insufficient durability.

---

# 317. Air-Gap Tests

1. export closure complete;
2. deterministic manifest;
3. tampered object rejected;
4. import deduplicates existing objects;
5. build succeeds offline from imported closure.

---

# 318. Security Tests

1. unauthorized artifact cannot download;
2. raw digest cannot bypass ACL;
3. P2P malicious bytes rejected;
4. manifest depth bomb rejected;
5. unsafe tree path rejected;
6. presigned URL expires;
7. secret value never enters normal CAS path.

---

# 319. Performance Benchmarks

```text
BLAKE3 throughput
local put/get
parallel upload
manifest encode/decode
tree construction
tier lookup
chunk transfer
```

---

# 320. Large Object Benchmark

Test:

```text
1 GB+
```

without whole-object memory loading.

---

# 321. Many Small Object Benchmark

Important for source trees/package managers.

---

# 322. Metadata Overhead

Avoid one SQL row per tiny source blob unless necessary.

CAS object index strategy should be scalable.

---

# 323. Local CAS Index

May derive path from digest without separate DB index.

---

# 324. Cloud Object Index

Same.

Metadata store only tracks semantic refs and optional replica status.

---

# 325. Object Existence Batch

Backend should support efficient batch existence if possible.

Fallback parallel bounded heads.

---

# 326. Head Storm Avoidance

For huge closures, use manifests/backend index/bloom hints where safe.

---

# 327. Transfer Plan

```rust
pub struct TransferPlan {
    pub required: Vec<CasObjectRef>,
    pub missing: Vec<CasObjectRef>,
    pub priority: TransferPriority,
}
```

---

# 328. Transfer Resume State

Stored locally/ephemerally.

Not durable business metadata unless long-lived session requires it.

---

# 329. Agent CAS Cache

Directory:

```text
$FORGEYARD_AGENT_DATA/cas/
```

Separate from workspace.

---

# 330. Workspace Materialization

Never execute directly inside raw CAS object path if process could mutate it.

Materialize/read-only mount.

---

# 331. Immutable Store Paths

Toolchain/dependency trees can be mounted read-only.

---

# 332. Copy-on-Write

Optional optimization for writable build inputs.

---

# 333. Hardlinks/Reflinks

Possible local optimization.

Must preserve CAS immutability.

Never let build mutate shared CAS file via hardlink.

---

# 334. Reflink Preferred

If filesystem supports copy-on-write safely.

---

# 335. Hardlink Hazard

Writable hardlink can corrupt CAS.

Do not use without read-only enforcement.

---

# 336. Filesystem Verification

Local CAS can mark files read-only.

Still verify permissions/security.

---

# 337. Materialization Cache

Separate from CAS.

Can cache extracted tree layout.

Must be invalidatable and verified against object ID.

---

# 338. Tree Materialization

```text
tree manifest
  ↓
create temp dir
  ↓
materialize entries
  ↓
verify
  ↓
atomic publish cache path
```

---

# 339. Tree Cache Key

```text
TreeObjectId + materialization policy
```

---

# 340. Platform Semantics

Windows/macOS/Linux differ in:

```text
symlink
executable bit
case sensitivity
path rules
```

Canonical tree remains neutral; materializer enforces compatibility.

---

# 341. Incompatible Tree

Example case-collision on Windows:

```text
materialization error
```

not silent overwrite.

---

# 342. Long Paths

Platform materializer handles/validates.

---

# 343. Extended Attributes

Default:

```text
not part of generic CAS tree
```

unless artifact/platform package explicitly models them.

---

# 344. macOS Bundle Metadata

Package builder should encode required metadata in final artifact/archive format.

Not rely on host filesystem xattrs unless explicitly captured.

---

# 345. Executable Bit

Explicit tree field.

---

# 346. Ownership UID/GID

Not generic source/build tree identity.

Package format may add ownership metadata at packaging step.

---

# 347. Timestamps

Not generic tree identity.

Deterministic package builder sets them explicitly where required.

---

# 348. CAS Schema Versioning

Every structured object format has independent schema version.

---

# 349. CAS Protocol Versioning

Network transfer protocol separately versioned.

---

# 350. Backend Configuration Version

Config schema handles backend settings evolution.

---

# 351. Migration of Structured Objects

Usually:

```text
old objects remain readable
new writes use new schema
```

Avoid rewriting entire CAS unless necessary.

---

# 352. Schema Decoder Registry

Known structured media types map to decoder versions.

---

# 353. Unknown Structured Object

Treat as opaque blob unless operation requires traversal.

GC roots cannot traverse unknown manifest type automatically.

Such objects must be retained via explicit root.

---

# 354. Artifact Manifest Compatibility

Keep old readers for supported retained release history.

---

# 355. Storage Backend Migration Tool

```text
forgeyard cas replicate --from A --to B
forgeyard cas verify --backend B
forgeyard cas drain A
```

---

# 356. Drain Mode

Backend states:

```rust
pub enum BackendLifecycle {
    Active,
    ReadOnly,
    Draining,
    Disabled,
}
```

---

# 357. Drain Behavior

No new writes.

Reads allowed until replicas complete.

---

# 358. Backend Removal

Only after:

```text
required objects replicated
metadata updated
verification complete
```

---

# 359. Cost Class

```rust
pub enum CostClass {
    Local,
    Low,
    Standard,
    High,
    Archive,
}
```

Scheduler/storage policy may use it.

---

# 360. Latency Class

```rust
pub enum LatencyClass {
    MemoryLike,
    LocalDisk,
    Lan,
    Regional,
    Remote,
    Archive,
}
```

---

# 361. Backend Selection

Policy considers:

```text
durability
region
cost
latency
object class
```

---

# 362. No Business Semantics in Backend Selection

Backend doesn't know "release."

Storage policy gets durability class from artifact/source metadata.

---

# 363. Data Plane API Layer

Potential internal service:

```text
crates/data-plane/
├── forgeyard-data-plane/
├── forgeyard-data-plane-service/
└── forgeyard-data-plane-policy/
```

Optional if orchestration complexity warrants.

---

# 364. Initial Recommendation

Do not create separate data-plane service crate until CAS/artifact orchestration grows.

Start with CAS + artifact services.

---

# 365. Artifact Store Interface

```rust
#[async_trait]
pub trait ArtifactStore {
    async fn register(
        &self,
        artifact: NewArtifact,
    ) -> Result<ArtifactRecord, ArtifactError>;

    async fn resolve(
        &self,
        id: ArtifactId,
    ) -> Result<ArtifactRecord, ArtifactError>;
}
```

Metadata implementation uses `ForgeyardStore`.

Bytes implementation uses `CasBackend`.

---

# 366. Artifact Service

Coordinates:

```text
authorization
CAS
metadata
retention
provenance
download
```

---

# 367. Artifact Upload Transaction

Correct:

```text
create pending artifact metadata
  ↓
upload object
  ↓
verify
  ↓
transactionally mark Available + object ref
```

or upload first then register; exact flow depends API.

---

# 368. Pending Upload Cleanup

Expired pending artifacts removed.

Orphan CAS handled by GC.

---

# 369. Artifact Download

```text
authorize
  ↓
resolve artifact
  ↓
resolve CAS ref
  ↓
stream
```

---

# 370. Integrity Header

Download API can expose digest headers.

---

# 371. Client Verification

CLI verifies downloaded artifact digest automatically.

---

# 372. Browser Download

Browser may rely on HTTPS + server verification, but checksum visible.

---

# 373. Release Verification CLI

```text
forgeyard release verify
```

fetches artifact/provenance/signature refs and verifies content.

---

# 374. Observability Dashboard

CAS UI:

```text
backend health
storage usage
hit rate
P2P hit rate
replication lag
GC status
integrity failures
```

---

# 375. Artifact UI

Shows:

```text
artifact ID
kind
digest
size
producer
source snapshot
retention
durability
attachments
provenance
```

---

# 376. Raw CAS UI

Admin-only object inspector.

---

# 377. Object Inspector

Displays:

```text
digest aliases
size
replicas
integrity
roots/reference count summary
structured type
```

---

# 378. Reference Summary

Do not compute expensive full reverse refs live without index.

May show known semantic references.

---

# 379. GC UI

Admin:

```text
last mark
next sweep
candidate bytes
deleted bytes
errors
```

---

# 380. Replication UI

Shows object classes/regions, not individual millions of objects by default.

---

# 381. Alerting

Alert on:

```text
durable backend unavailable
release durability violation
integrity mismatch
replication backlog
local disk nearly full
GC stalled
multipart leak
```

---

# 382. SLOs

Possible:

```text
artifact durability
download availability
replication completion
```

Deployment-specific values.

---

# 383. Backup Relationship

CAS backup may be:

```text
replication to independent backend
object-store versioning
snapshot/export
```

Metadata DB backup alone is insufficient.

---

# 384. Restore

After metadata restore:

```text
verify referenced CAS objects
repair missing from replicas/backups
reconcile durability state
```

---

# 385. Disaster Recovery

If regional CAS lost:

```text
route reads to replica
recreate region
replicate back
```

Object IDs unchanged.

---

# 386. Standalone Disaster Recovery

Restore local DB + local CAS backup.

---

# 387. Partial Recovery

If metadata survives but some cache objects lost:

```text
cache miss
rebuild/refetch
```

If release/source critical object lost:

```text
data loss incident
```

unless replica/backup exists.

---

# 388. Data Importance Classification

```rust
pub enum ObjectClass {
    RebuildableCache,
    SourceCritical,
    ToolchainCritical,
    ArtifactNormal,
    ReleaseCritical,
    AuditCritical,
}
```

Used to derive durability/retention policy.

---

# 389. Rebuildable Cache

Can be discarded.

---

# 390. Source Critical

Needed for reproducibility/history.

---

# 391. Toolchain Critical

Needed to reproduce locked builds.

---

# 392. Audit Critical

Evidence may require long retention.

---

# 393. Policy Mapping

```text
ObjectClass
  ↓
DurabilityClass
RetentionPolicy
ReplicationPolicy
IntegrityFrequency
```

---

# 394. Default Policy Example

```text
RebuildableCache -> 1 copy, TTL
ArtifactNormal -> durable 1 region
SourceCritical -> durable 2 copies
ReleaseCritical -> 2 regions + pin
AuditCritical -> durable + legal/compliance retention
```

Deployment can customize.

---

# 395. Implementation Phase 1 — Core CAS

Implement:

```text
CasObjectId
CasObjectRef
CasBackend
streaming put/get
BLAKE3 verification
local backend
```

Exit:

```text
standalone can persist/retrieve immutable objects
```

---

# 396. Phase 2 — Manifest / Tree

Implement:

```text
canonical manifest
tree manifest
source/output tree capture
recursive closure
```

---

# 397. Phase 3 — Artifact Service

Implement:

```text
ArtifactId
artifact metadata
pending/available lifecycle
upload/download
retention
```

---

# 398. Phase 4 — Transfer

Implement:

```text
batch missing
QUIC streaming
resumable large transfers
agent local cache
```

---

# 399. Phase 5 — Tiered CAS

Implement:

```text
L1 local
durable backend
promotion/fallback
```

---

# 400. Phase 6 — S3-Compatible

Implement first production remote durable backend.

Then GCS/Azure adapters.

---

# 401. Phase 7 — Iroh

Add P2P acceleration after durable CAS semantics are stable.

---

# 402. Phase 8 — Replication

Implement:

```text
replica tracking
durability policy
regional replication
repair
```

---

# 403. Phase 9 — GC / Retention

Implement:

```text
root snapshot
mark
grace
sweep
pin/legal hold
```

---

# 404. Phase 10 — Logs / Reports

Integrate:

```text
log chunks
test reports
coverage
SBOM/provenance
```

---

# 405. Phase 11 — Air-Gap

Implement closure export/import.

---

# 406. Phase 12 — Hardening

Implement:

```text
integrity scans
capacity controls
quota
security fuzzing
large-scale performance
multi-region failure tests
```

---

# 407. Acceptance Tests

1. Same bytes always produce same CAS ID.
2. Different bytes produce different CAS ID.
3. Duplicate upload does not duplicate logical object.
4. Digest mismatch is rejected.
5. Partial local write never becomes visible.
6. Large object streams without full memory buffering.
7. Source tree manifest is deterministic.
8. Case/path incompatibility fails materialization safely.
9. Artifact metadata never stores artifact bytes.
10. Missing CAS object invalidates cache mapping.
11. Local L1 miss falls back to durable backend.
12. Corrupt L1 object falls back and repairs.
13. P2P malicious bytes fail digest verification.
14. Iroh outage does not break durable CAS correctness.
15. Required release replicas gate promotion.
16. GC preserves rooted manifest closure.
17. GC deletes orphan after grace.
18. Legal hold prevents GC.
19. Interrupted multipart upload is cleaned.
20. Air-gap export/import preserves object IDs.
21. Offline runner can build from imported closure.
22. Stale job output upload remains unreferenced and GC-eligible.
23. Signed artifact gets new content ID.
24. BLAKE3↔SHA-256 alias is verified.
25. Unauthorized user cannot fetch arbitrary object by digest.
26. Tenant isolation prevents existence leakage through public APIs.
27. Large manifest/depth bombs are bounded.
28. CAS backend migration preserves object IDs.
29. Metadata restore can verify referenced object availability.
30. Standalone local CAS works with no network/cloud.

---

# 408. Production Readiness Gates

Do not call CAS/data plane production-ready until:

```text
local backend durable
streaming works
digest verification works
canonical manifest/tree stable
artifact metadata separation works
agent transfer works
durable remote backend works
tier fallback works
GC is safe
retention/pin works
integrity repair works
security boundaries tested
backup/restore relationship documented
metrics/doctor exist
```

P2P, multi-region, and advanced chunking can mature after core production readiness.

---

# 409. Architectural Invariants

1. CAS object identity is content-derived.
2. CAS objects are immutable.
3. Business meaning is not CAS identity.
4. Metadata and bytes are separate.
5. BLAKE3 is internal default.
6. SHA-256 aliases are verified.
7. Large objects stream.
8. Partially written objects never become canonical.
9. Every backend verifies/participates in integrity policy.
10. Iroh/P2P is acceleration, not authority.
11. Local cache is not durability unless standalone policy says so.
12. Release-critical objects have explicit durability requirements.
13. Source snapshots and toolchains use same CAS foundation.
14. Action cache maps keys to CAS outputs; cache does not store bytes separately.
15. GC derives liveness from metadata roots and structured object closure.
16. GC uses grace periods.
17. Legal hold/pin overrides GC.
18. Raw CAS APIs do not bypass artifact/source authorization.
19. Secrets never intentionally enter normal CAS.
20. Structured object schemas are versioned.
21. Canonical encoding is deterministic.
22. Materialization never allows path traversal.
23. CAS backend implementations do not leak into domain services.
24. Backend migration does not change object IDs.
25. Multi-region replication is policy, not identity.
26. Signed/notarized changed bytes get new IDs.
27. Restore includes CAS integrity reconciliation.
28. Background replication/GC/repair are idempotent.
29. No advanced performance feature is a correctness dependency.
30. The same CAS architecture works from one local disk to enterprise multi-region storage.

---

# 410. Final Target Architecture

```text
                        Forgeyard Services
                               │
                               ▼
                        Artifact / Source
                           Metadata
                               │
                               ▼
                          CasObjectRef
                               │
                               ▼
                        Forgeyard CAS API
                               │
           ┌───────────────────┼───────────────────┐
           ▼                   ▼                   ▼
       Local L1/L2         Iroh/P2P          Durable Object
           │                   │                Storage
           └───────────────────┼───────────────────┘
                               ▼
                       Digest Verification
                               │
                 ┌─────────────┼─────────────┐
                 ▼             ▼             ▼
              Replication    Retention       GC
                 │             │             │
                 └─────────────┼─────────────┘
                               ▼
                       Integrity / Repair
```

---

# 411. Final Architectural Position

Object ingestion:

```text
bytes
  ↓
stream/hash
  ↓
verify expected digest/size
  ↓
store immutable object
  ↓
CasObjectRef
  ↓
register semantic metadata
```

Build input:

```text
SourceSnapshot / Toolchain / Dependencies
  ↓
CAS closure
  ↓
runner local missing check
  ↓
tiered fetch
  ↓
digest verification
  ↓
read-only materialization
```

Build output:

```text
declared output
  ↓
canonical capture
  ↓
CAS object/tree
  ↓
artifact metadata
  ↓
retention/durability policy
```

Durability:

```text
object
  ↓
replication policy
  ↓
verified replicas
  ↓
DurabilityState::Satisfied
```

Cleanup:

```text
metadata roots
  ↓
mark structured closure
  ↓
grace
  ↓
recheck
  ↓
sweep unreferenced objects
```

The key guarantee is:

> **Forgeyard can move the same immutable object across local disk, P2P peers, cloud object stores, regions, caches, backups, and air-gapped bundles without changing its content identity or the business meaning attached to it.**

---

# 412. New-Repository Sequence

The new Forgeyard implementation sequence is now:

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

The already-completed hermetic, VCS-neutral, Change Proposal, and language ecosystem architectures plug into this sequence at their respective integration points.
