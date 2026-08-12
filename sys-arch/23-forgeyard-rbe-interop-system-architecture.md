# 23 — Forgeyard Remote Build Execution (RBE) Interoperability System Architecture

**Document type:** Core Remote Build Execution Interoperability System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Bazel Remote Execution API compatibility, gRPC edge service, REAPI CAS/action-cache translation, digest mapping, platform properties, action execution mapping, operation lifecycle, cancellation, retries, cache policy, multi-tenancy, observability, and interoperability boundaries  
**Architecture style:** Standards-compatible edge adapter over Forgeyard-native execution, CAS, scheduler, runner, and policy systems; no duplication of Forgeyard core semantics; no gRPC inside internal hot paths except interoperability boundary  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on CAS/Data Plane, Run/Job State Machine, Scheduler, Runner/Agent, Sandbox/Executor, Transport/QUIC, Policy/Authz/Identity, Supply Chain, API/Axum, HA/Coordination, and hermetic/reproducible build architecture. RBE is an interoperability surface, not a second execution engine.

---

# 1. Purpose

Forgeyard should interoperate with build systems and clients that already speak Remote Execution API semantics.

Primary target:

```text
Bazel Remote Execution API / REAPI
```

Typical externally expected capabilities include:

```text
Content Addressable Storage
Action Cache
Execution
Capabilities
ByteStream
Operations
```

The central rule is:

> **Forgeyard implements RBE as an edge compatibility layer. Internally, execution remains Forgeyard-native: Pipeline/Job/Attempt/Lease + scheduler + runner + CAS + policy.**

A second rule is:

> **External REAPI digests are interoperability identifiers; Forgeyard's internal CAS can retain BLAKE3 as its primary digest while maintaining SHA-256 aliases where REAPI requires them.**

A third rule is:

> **No RBE request bypasses Forgeyard authorization, tenant isolation, scheduler capability checks, sandbox policy, CAS integrity checks, or execution leases.**

---

# 2. Architectural Position

```text
                Bazel / REAPI Client
                        │
                        ▼
                  gRPC / REAPI
                        │
                        ▼
                Forgeyard RBE Edge
                        │
        ┌───────────────┼────────────────┐
        ▼               ▼                ▼
   REAPI CAS        Action Cache      Execute
        │               │                │
        ▼               ▼                ▼
   Digest Map      Forgeyard Cache    JobSpec Map
        │               │                │
        └───────────────┼────────────────┘
                        ▼
                  Forgeyard Core
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
          CAS        Scheduler      Runner
```

---

# 3. Goals

The subsystem MUST:

1. expose REAPI-compatible gRPC endpoints;
2. support CAS interoperability;
3. support ByteStream semantics;
4. support Action Cache;
5. support Execute;
6. support Operations;
7. support Capabilities;
8. support SHA-256 digest addressing externally;
9. map external digests to internal CAS objects;
10. preserve BLAKE3 internally;
11. enforce tenant isolation;
12. enforce authentication;
13. enforce authorization;
14. map platform properties to scheduler requirements;
15. map actions to Forgeyard JobSpec;
16. use normal Run/Job/Attempt/Lease semantics;
17. support execution cancellation;
18. support client retries safely;
19. support deterministic cache keys;
20. support output files/directories;
21. support stdout/stderr digests;
22. support tree/Directory objects;
23. support cache hit/miss semantics;
24. support bounded message sizes;
25. support streaming;
26. support observability;
27. support rate limiting;
28. support HA;
29. support rolling protocol upgrades;
30. remain isolated from Forgeyard's native QUIC/Postcard transport.

---

# 4. Non-Goals

RBE does not:

```text
replace Forgeyard pipeline syntax
replace Forgeyard scheduler
replace Forgeyard runner protocol
replace Forgeyard CAS internals
replace Forgeyard policy engine
```

---

# 5. Workspace Structure

```text
crates/rbe/
├── forgeyard-rbe/
├── forgeyard-rbe-model/
├── forgeyard-rbe-grpc/
├── forgeyard-rbe-reapi/
├── forgeyard-rbe-cas/
├── forgeyard-rbe-bytestream/
├── forgeyard-rbe-action-cache/
├── forgeyard-rbe-execution/
├── forgeyard-rbe-operation/
├── forgeyard-rbe-capabilities/
├── forgeyard-rbe-digest/
├── forgeyard-rbe-platform/
├── forgeyard-rbe-auth/
├── forgeyard-rbe-multitenancy/
├── forgeyard-rbe-health/
└── forgeyard-rbe-testkit/
```

---

# 6. Protocol Boundary

External:

```text
gRPC + protobuf + REAPI
```

Internal:

```text
Forgeyard Rust domain types
QUIC + Postcard for agents
```

---

# 7. No Internal gRPC Dependency

Agent/daemon execution protocol remains QUIC/Postcard.

---

# 8. Why

Avoid:

```text
duplicate hot-path transport stacks
protobuf everywhere
gRPC coupling in scheduler/runner
```

---

# 9. REAPI Service Families

Typical:

```text
ContentAddressableStorage
ActionCache
Execution
Capabilities
ByteStream
Operations
```

---

# 10. Generated Protobuf Types

Stay inside adapter boundary.

---

# 11. Domain Conversion

Every request:

```text
protobuf DTO
  ↓
validation
  ↓
Forgeyard normalized RBE model
  ↓
core services
```

---

# 12. No Protobuf Types in Core

Critical.

---

# 13. RbeInstanceName

```rust
pub struct RbeInstanceName(BoundedString);
```

---

# 14. Instance Mapping

Maps to:

```text
tenant/project/cache namespace
```

---

# 15. Instance Authority

Configured server-side.

Client cannot choose arbitrary tenant by string.

---

# 16. Tenant Isolation

Authentication context + instance mapping.

---

# 17. Digest Model

External REAPI digest typically:

```text
SHA-256
size_bytes
```

---

# 18. Internal Digest Model

Forgeyard:

```text
BLAKE3 primary
SHA-256 alias where required
```

---

# 19. Digest Alias Table

```rust
pub struct DigestAlias {
    pub internal: CasObjectId,
    pub algorithm: DigestAlgorithm,
    pub digest: Digest,
}
```

---

# 20. RBE Digest

```rust
pub struct RbeDigest {
    pub sha256: Sha256Digest,
    pub size_bytes: u64,
}
```

---

# 21. Digest Verification

Upload must recompute SHA-256.

---

# 22. Internal Mapping

After verify:

```text
compute BLAKE3
store CAS object
register SHA-256 alias
```

---

# 23. No Client-Claim Trust

Client digest alone not authority.

---

# 24. Size Verification

Exact.

---

# 25. Hash Collision Model

Use standard cryptographic digest assumptions.

---

# 26. CAS Object Types

REAPI objects include:

```text
Blob
Directory
Tree
Command
Action
ActionResult payloads
```

---

# 27. Object Validation

Bound:

```text
size
nesting
entry count
path length
```

---

# 28. Directory Validation

Reject:

```text
duplicate names
invalid names
path escape
malformed digest
```

---

# 29. Tree Validation

Validate all referenced directories.

---

# 30. REAPI CAS

Maps to Forgeyard CAS.

---

# 31. FindMissingBlobs

Use alias index/metadata.

---

# 32. BatchUpdateBlobs

Bound batch size.

---

# 33. BatchReadBlobs

Bound response.

---

# 34. GetTree

Stream/paginate according to REAPI semantics.

---

# 35. ByteStream Write

For large blobs.

---

# 36. ByteStream Read

Streaming.

---

# 37. Resumable Upload

Map ByteStream resource/session to Forgeyard upload session.

---

# 38. Upload Offset

Validated.

---

# 39. Upload Session TTL

Bound.

---

# 40. Duplicate Upload

Same digest:

```text
idempotent
```

---

# 41. Partial Upload

Temporary data not visible as CAS object until verified/finalized.

---

# 42. CAS Namespace

Tenant/project/cache namespace isolation.

---

# 43. Cross-Tenant Dedup

Physical dedup may exist internally but authorization/cache visibility remains isolated.

---

# 44. Cross-Tenant Cache

Off by default.

---

# 45. Action Cache

REAPI Action Cache maps action digest -> ActionResult.

---

# 46. ActionCacheKey

External SHA-256 action digest.

---

# 47. Internal Cache Key

Can map to Forgeyard action/derivation key.

---

# 48. Cache Semantics

Only store result if action considered cacheable.

---

# 49. Secret-Bearing Action

Default:

```text
not shared-cacheable
```

---

# 50. Non-Hermetic Action

Policy may disable remote cache.

---

# 51. Cache Trust

Cached result must come from trusted execution class according to policy.

---

# 52. Poisoning Defense

Validate:

```text
result subject digests
tenant scope
execution provenance/trust
```

---

# 53. Action Result

Contains:

```text
exit code
output files
output directories
stdout/stderr
execution metadata
```

---

# 54. Output Files

Stored in CAS.

---

# 55. Output Directories

Tree representation translated.

---

# 56. Stdout/Stderr

Inline small or digest-referenced.

---

# 57. Output Symlinks

Validate platform semantics.

---

# 58. Action Result Metadata

External metadata derived from Forgeyard Attempt/Executor info.

---

# 59. Cache Lookup Flow

```text
GetActionResult
  ↓
authz/instance
  ↓
lookup action alias
  ↓
policy/trust check
  ↓
return ActionResult
```

---

# 60. Cache Miss

Standard NOT_FOUND semantics.

---

# 61. UpdateActionResult

External client may try to populate Action Cache.

---

# 62. Default Security Position

Do not allow arbitrary untrusted client cache writes unless explicitly authorized.

---

# 63. Trusted Cache Writer

Permission:

```text
rbe.cache.write
```

---

# 64. Better Default

Forgeyard execution service populates Action Cache after successful trusted execution.

---

# 65. Execute

External client sends Action digest.

---

# 66. Action Resolution

```text
Action digest
  ↓
load Action
  ↓
load Command
  ↓
load input root
  ↓
validate closure
```

---

# 67. Closure Validation

All required digests exist.

---

# 68. Missing Input

Return failed precondition.

---

# 69. Command

Normalize:

```rust
pub struct RbeCommandSpec {
    pub arguments: Vec<BoundedString>,
    pub environment: Vec<RbeEnvVar>,
    pub output_paths: Vec<RelativePath>,
    pub working_directory: RelativePath,
    pub platform: RbePlatformProperties,
}
```

---

# 70. No Shell Assumption

Arguments represent argv.

---

# 71. Working Directory

Relative to input root/workspace.

---

# 72. Environment Variables

Bounded.

---

# 73. Secret Variables

RBE protocol has no Forgeyard-native SecretRef semantics.

Therefore remote RBE client cannot inject Forgeyard secrets unless explicit extension/API integration.

---

# 74. Safe Default

Treat provided env values as ordinary untrusted data.

---

# 75. Platform Properties

Map REAPI platform properties into Forgeyard scheduler capability requirements.

---

# 76. Known Properties

Potential:

```text
OSFamily
Arch
container-image
dockerNetwork
cpu
memory
toolchain
```

actual compatibility follows REAPI/client conventions.

---

# 77. Internal Normalization

```rust
pub struct RbePlatformRequirement {
    pub os: Option<TargetOs>,
    pub arch: Option<TargetArch>,
    pub properties: BTreeMap<RbePropertyKey, RbePropertyValue>,
}
```

---

# 78. Property Registry

Known properties typed.

Unknown properties handled via configured namespace.

---

# 79. Unknown Property

Default:

```text
unsatisfied / rejected
```

rather than silently ignored for correctness-affecting keys.

---

# 80. Scheduler Mapping

Hard filters first.

---

# 81. Example

```text
OSFamily=linux
Arch=x86_64
  ↓
runner.platform == linux/x86_64
```

---

# 82. Container Property

Can map to ContainerExecutor requirement.

---

# 83. Toolchain Property

Map to managed toolchain capability if configured.

---

# 84. Execution Identity

One RBE action becomes Forgeyard internal execution entity.

---

# 85. RbeExecutionId

```rust
pub struct RbeExecutionId(Ulid);
```

---

# 86. Internal Run Mapping

Recommended:

```text
RBE Execute request
  ↓
synthetic/interop Run
  ↓
one Job
  ↓
normal Attempt/Lease
```

---

# 87. Batch Client Session

Multiple actions may optionally group under one RBE session/run.

---

# 88. Simpler Initial

One Execute -> one interop Run/Job.

---

# 89. JobSpec

Generated deterministically from validated Action/Command/input root/platform.

---

# 90. JobSpecId

Part of normal Forgeyard semantics.

---

# 91. SourceSnapshotId

RBE input root is not necessarily VCS source.

Represent as:

```text
InputRootId / SourceSnapshot-like CAS tree
```

Do not falsely claim VCS provenance.

---

# 92. RbeInputRootId

```rust
pub struct RbeInputRootId(Digest);
```

---

# 93. Provenance

Can record:

```text
RBE Action digest
Command digest
InputRoot digest
```

---

# 94. Run Actor

Authenticated RBE principal/service account.

---

# 95. Policy

RBE action subject to:

```text
tenant policy
runner trust
sandbox
network
resource limits
```

---

# 96. Network

REAPI action may not explicitly define Forgeyard network policy.

Server profile determines.

---

# 97. Strict Default

Network denied unless configured platform property/profile allows.

---

# 98. Sandbox

Normal Forgeyard sandbox.

---

# 99. Executor

Normal executor selection.

---

# 100. Runner

Normal scheduler/agent.

---

# 101. JobLease

Normal.

---

# 102. Stale Completion

Normal rejection.

---

# 103. Operation

REAPI Execute returns long-running Operation.

---

# 104. Operation Mapping

```rust
pub struct RbeOperationBinding {
    pub operation_name: RbeOperationName,
    pub execution: RbeExecutionId,
    pub run: RunId,
    pub job: JobId,
}
```

---

# 105. Operation State

Maps from Run/Job state.

---

# 106. Pending

Queued/eligible.

---

# 107. Executing

Leased/preparing/running/uploading.

---

# 108. Completed

Final ActionResult/error.

---

# 109. Operation Name

Opaque stable string.

---

# 110. WaitExecution

Subscribes/streams updates.

---

# 111. GetOperation

Reads current authoritative state.

---

# 112. DeleteOperation

Semantics constrained.

Deleting operation view does not erase Forgeyard execution history.

---

# 113. CancelOperation

Maps to normal Run/Job cancellation request.

---

# 114. Cancellation Is Durable

Client disconnect alone does not cancel.

---

# 115. Execute Stream Disconnect

Execution continues.

Client reconnects via operation name.

---

# 116. Retry Execute

Same action may produce duplicate execution unless client uses operation semantics/cache.

---

# 117. Optional Request Dedup

Can derive/accept idempotency metadata where extension supports.

---

# 118. Cache Lookup Before Execute

Respect `skip_cache_lookup`.

---

# 119. If Cache Hit

Return completed operation/result without runner execution.

---

# 120. Cache Hit Evidence

Can record cache source/trust.

---

# 121. DoNotCache

Respect Action flag/semantics.

---

# 122. Action Timeout

Maps to Job timeout.

---

# 123. Server Maximum Timeout

Clamp/reject exceeding policy.

---

# 124. Resource Limits

Platform/config maps CPU/memory/storage.

---

# 125. Queue Timeout

Forgeyard scheduler policy.

---

# 126. Priority

REAPI request priority maps to bounded Forgeyard priority class.

---

# 127. No Arbitrary Privilege Escalation

Client priority cannot exceed authorized ceiling.

---

# 128. Execution Metadata

Expose:

```text
queued timestamp
worker start
execution start/end
output upload
```

where compatible.

---

# 129. Worker Identity

Public-safe worker identifier.

Do not leak internal hostname if privacy policy forbids.

---

# 130. RBE Error Mapping

Forgeyard typed errors -> gRPC/REAPI status.

---

# 131. Examples

```text
INVALID_ARGUMENT
NOT_FOUND
FAILED_PRECONDITION
PERMISSION_DENIED
RESOURCE_EXHAUSTED
UNAVAILABLE
DEADLINE_EXCEEDED
INTERNAL
```

---

# 132. Infrastructure Failure

Map appropriately without pretending action failed semantically.

---

# 133. Retry Information

gRPC metadata/status details where appropriate.

---

# 134. Capabilities Service

Reports supported:

```text
digest functions
execution enabled
cache enabled
API version
symlink strategy
priority support
```

---

# 135. Capability Honesty

Only advertise implemented semantics.

---

# 136. Digest Function

Externally SHA-256 baseline.

---

# 137. Internal BLAKE3

Invisible to standard client except optional extensions.

---

# 138. API Version

Track supported REAPI protocol versions.

---

# 139. Rolling Upgrade

Serve overlapping supported versions during daemon rollout.

---

# 140. RBE Endpoint

Example:

```text
rbe.forgeyard.example.com:443
```

---

# 141. TLS

Mandatory production.

---

# 142. Authentication

Options:

```text
Bearer token
mTLS
OIDC-derived service token
```

---

# 143. Bazel Auth Integration

Use standard gRPC auth metadata.

---

# 144. RBE Principal

Maps to Forgeyard PrincipalId.

---

# 145. Authorization Permissions

```text
rbe.read
rbe.execute
rbe.cas.read
rbe.cas.write
rbe.cache.read
rbe.cache.write
```

---

# 146. Instance Scope

Permission scoped to tenant/project/instance.

---

# 147. Anonymous Cache

Off by default.

---

# 148. Public Cache

Could be separate explicit deployment profile.

---

# 149. Multi-Tenancy

CAS and Action Cache namespace separation.

---

# 150. Cross-Tenant Timing Leakage

Physical dedup may leak existence via timing.

High-assurance mode can reduce/disable shared physical dedup.

---

# 151. Quotas

Per tenant:

```text
CAS storage
upload bandwidth
execute concurrency
queue
cache
```

---

# 152. Rate Limits

gRPC request classes.

---

# 153. Resource Exhausted

Use standard code.

---

# 154. ByteStream Limits

Bound concurrent streams.

---

# 155. CAS GC

RBE blobs rooted by:

```text
active actions
Action Cache
retention policy
other Forgeyard references
```

---

# 156. Action Cache Retention

TTL/LRU/policy.

---

# 157. Missing Blob After Cache Entry

Cache entry invalidated/miss.

---

# 158. Referential Integrity

ActionResult references must exist before cache publish.

---

# 159. CAS Repair

Normal CAS subsystem.

---

# 160. GetTree Pagination

Do not load entire huge tree in memory.

---

# 161. Directory Tree Conversion

Stream/iterate.

---

# 162. Input Materialization

Convert REAPI directory tree into Forgeyard workspace safely.

---

# 163. Path Safety

Reject:

```text
..
absolute path
invalid duplicate
unsupported node type
```

---

# 164. Symlink Policy

Respect REAPI capability/profile.

---

# 165. Unsafe Symlink

Reject escape from input root/workspace.

---

# 166. File Executability

Preserve executable bit.

---

# 167. Permissions

Normalize to sandbox semantics.

---

# 168. Timestamps

REAPI input trees do not rely on original host timestamps.

Use deterministic materialization.

---

# 169. Input Root Read-Only

Strict execution.

---

# 170. Output Paths

Declared.

---

# 171. Undeclared Output

Not returned/cached.

---

# 172. Output Collection

After success/failure according to REAPI semantics.

---

# 173. Output Directory Encoding

Generate Directory/Tree objects.

---

# 174. Stdout/Stderr

Capture using normal logging pipe but also produce REAPI digests/result fields.

---

# 175. Log Streaming

REAPI standard does not expose Forgeyard live log UI semantics directly.

Operation metadata can remain standard-compatible.

---

# 176. Forgeyard Deep Link

Optional metadata/auxiliary log URL if client extension supports.

---

# 177. Action Cache Write Timing

Only after all output CAS objects committed.

---

# 178. Completion Atomicity

Semantically:

```text
outputs durable
  ↓
ActionResult durable
  ↓
cache visible
```

---

# 179. Crash Between Outputs and Cache

Safe: blobs exist but cache miss.

---

# 180. Crash After Cache Write

Result is durable.

---

# 181. Exactly-Once

Not claimed.

---

# 182. Execute At-Least-Once

RBE client retries can create multiple actions unless cache/dedup.

Forgeyard ensures each internal lease/attempt semantics are safe.

---

# 183. Action Determinism

Client/build system responsibility + Forgeyard hermeticity profile.

---

# 184. Hermetic RBE Profile

Recommended profile:

```text
network deny
managed platform
input root read-only
declared outputs
controlled env
```

---

# 185. Strict Cache Eligibility

Only if profile meets configured cache requirements.

---

# 186. Non-Hermetic Profile

Can execute but skip shared cache.

---

# 187. Platform Execution Profiles

Server config:

```rust
pub struct RbeExecutionProfile {
    pub id: RbeExecutionProfileId,
    pub platform_match: RbePlatformMatcher,
    pub sandbox: SandboxProfileId,
    pub network: NetworkPolicy,
    pub cache_policy: RbeCachePolicy,
}
```

---

# 188. Profile Selection

Deterministic by platform properties + tenant policy.

---

# 189. Unknown Profile

Reject.

---

# 190. Container Image Property

If accepted, image must resolve to immutable digest.

---

# 191. Mutable Container Tag

Resolve before JobSpec freeze or reject strict mode.

---

# 192. Toolchain Image

Provenance records resolved digest.

---

# 193. RBE Worker Platform

Forgeyard runners publish capabilities.

Capabilities service can expose logical supported properties.

---

# 194. No Static Lie

If no matching runner pool exists, capability should reflect configured execution availability.

---

# 195. Temporary Capacity Loss

Capabilities can still advertise supported platform even if currently busy.

---

# 196. Unsupported Platform

Reject with failed precondition.

---

# 197. Scheduler Fairness

RBE jobs enter same scheduler fairness model.

---

# 198. RBE Queue Class

Can have bounded project/tenant weight.

---

# 199. No RBE Priority Bypass

Critical.

---

# 200. Cost Accounting

RBE usage can be attributed by principal/tenant/project.

---

# 201. RBE Run Visibility

Forgeyard UI can show RBE-origin runs.

---

# 202. Run Origin

```rust
pub enum RunOrigin {
    ForgeyardPipeline,
    Rbe,
    Api,
    Scm,
}
```

or equivalent existing model.

---

# 203. RBE UI

Admin/project page:

```text
Executions
Cache
CAS
Instances
Capabilities
Health
```

---

# 204. Execution Detail

Shows:

```text
Action digest
Command digest
InputRoot digest
RunId
JobId
runner
cache hit/miss
```

---

# 205. Cache Detail

No arbitrary blob browsing without authz.

---

# 206. CLI

```text
forgeyard rbe status
forgeyard rbe capabilities
forgeyard rbe cache stats
forgeyard rbe execution <id>
forgeyard rbe doctor
```

---

# 207. Doctor

Checks:

```text
gRPC listener
auth
CAS
Action Cache
scheduler
runner platform mapping
```

---

# 208. Health

Separate:

```text
rbe_api
rbe_cas
rbe_cache
rbe_execution
```

---

# 209. Metrics

```text
rbe_requests_total
rbe_request_duration
rbe_cas_upload_bytes
rbe_cas_download_bytes
rbe_cache_hits
rbe_cache_misses
rbe_execute_requests
rbe_execute_queue_seconds
rbe_execute_duration
rbe_operations_active
```

---

# 210. Labels

Low-cardinality:

```text
service
method
result
platform_class
cache_result
```

---

# 211. No Action Digest Metric Label

Use traces/logs.

---

# 212. Tracing

```text
rbe.cas.find_missing
rbe.cas.upload
rbe.cache.get
rbe.execute
rbe.operation.wait
```

---

# 213. Trace Propagation

gRPC metadata extracts W3C/OpenTelemetry-compatible context where client provides.

---

# 214. Log Correlation

Include:

```text
RbeExecutionId
RunId
JobId
```

---

# 215. Audit

Sensitive:

```text
cache write
execute
admin instance changes
```

according to policy.

---

# 216. Action Input Privacy

Command args/env may contain sensitive user data.

Avoid logging full content by default.

---

# 217. Environment Values

Do not log.

---

# 218. Action/Command Blob Visibility

Permission controlled.

---

# 219. CAS Encryption

Inherited from CAS/storage architecture.

---

# 220. Transport Security

TLS.

---

# 221. gRPC Reflection

Disabled by default in production or admin-only.

---

# 222. Message Limits

Explicit.

---

# 223. Decompression Limits

Explicit.

---

# 224. Keepalive

Configured to avoid dead connections/DoS.

---

# 225. Connection Limits

Per principal/IP as supplemental.

---

# 226. Cancellation Flood

Rate limited.

---

# 227. Operations Retention

Completed operation metadata retained for bounded period.

---

# 228. Forgeyard Run History

Normal retention independent of REAPI operation view.

---

# 229. DeleteOperation

May delete external operation handle metadata after policy window, not Run history.

---

# 230. WaitExecution Reconnect

Operation binding persists.

---

# 231. HA

Any daemon can serve gRPC RBE.

---

# 232. Operation State

Shared DB authority.

---

# 233. CAS

Shared/tiered authority.

---

# 234. Scheduler

HA leader semantics from Part 22.

---

# 235. Client Retry Across Daemon

Safe.

---

# 236. Sticky Session

Not required.

---

# 237. Long gRPC Streams

Reconnect to any daemon using operation name.

---

# 238. Rolling Upgrade

Old/new RBE endpoints serve compatible protocol.

---

# 239. Instance Config Reload

Versioned.

---

# 240. Capability Changes

Clients may cache; avoid rapid churn.

---

# 241. REAPI Version Support

Document compatibility matrix.

---

# 242. Standard Compliance Tests

Use official/protocol-compatible client tests where available.

---

# 243. Bazel Integration Test

Actual Bazel client against Forgeyard.

---

# 244. Remote Cache Only Mode

Support deployment profile:

```text
CAS + Action Cache
Execute disabled
```

---

# 245. Remote Execution Mode

Full.

---

# 246. Read-Only Cache Mode

Useful shared cache.

---

# 247. Capabilities Reflect Mode

Correctly.

---

# 248. Local Standalone

Can expose localhost RBE endpoint.

---

# 249. Development Use

Bazel remote cache/executor on one machine.

---

# 250. Distributed Mode

Scale across Forgeyard runners.

---

# 251. RBE Instance Config

Example:

```ron
(
    rbe: (
        enabled: true,
        instances: [
            (
                name: "main",
                project: "forgeyard",
                execute: true,
                cache_read: true,
                cache_write: true,
            ),
        ],
    ),
)
```

---

# 252. Instance Name Validation

Bounded and normalized.

---

# 253. Instance Mapping Is Configured

Not user-controlled path traversal.

---

# 254. Resource Name Parsing

ByteStream resource names strictly parsed.

---

# 255. Resource Name Fuzzing

Mandatory.

---

# 256. RBE Error Model

```rust
pub enum RbeError {
    InvalidRequest,
    UnknownInstance,
    MissingBlob,
    DigestMismatch,
    CacheDenied,
    ExecuteDenied,
    UnsupportedPlatform,
    ResourceExhausted,
    Unavailable,
    Internal,
}
```

---

# 257. Error Conversion

To tonic/gRPC Status at edge.

---

# 258. Tonic

Natural Rust gRPC implementation choice if compatible with selected protobuf stack.

Architecture remains crate-neutral.

---

# 259. Proto Generation

Build-time generated code isolated.

---

# 260. Vendored Proto

Pin supported REAPI schema/version.

---

# 261. License Review

Ensure protobuf/schema/dependencies permissible for Forgeyard distribution.

---

# 262. Backward Compatibility

No ad-hoc proto modifications.

---

# 263. Extensions

Use separate Forgeyard extension service/metadata, never break standard fields.

---

# 264. Forgeyard Extension

Potential:

```text
deep link
tenant diagnostics
execution provenance ref
```

optional.

---

# 265. Standard Client

Works without extensions.

---

# 266. Action Cache Federation

Future optional.

---

# 267. External CAS Backend

Could interoperate with existing RBE cache via adapter.

---

# 268. But Baseline

Forgeyard CAS authority.

---

# 269. Cache Import

External cache result must be validated/trusted per policy.

---

# 270. Cache Export

Can expose Forgeyard cache via REAPI.

---

# 271. Cache Namespace Version

Prevent semantic collision after major execution model change.

---

# 272. Cache Key Salt

Server-side execution profile/toolchain semantics may need to influence eligibility.

---

# 273. Important Interop Constraint

Standard REAPI Action digest is client-defined from Action/Command/input root.

Forgeyard must not silently change Action digest semantics.

---

# 274. Server Policy vs Key

If server execution profile changes semantics:

```text
invalidate/namespace cache
```

rather than pretending same standard Action digest guarantees same environment.

---

# 275. RbeCacheNamespaceId

```rust
pub struct RbeCacheNamespaceId(Digest);
```

---

# 276. Namespace Inputs

```text
tenant/project
execution profile version
platform mapping version
cache policy version
```

---

# 277. External Action Digest

Still returned as standard.

---

# 278. Internal Cache Record

Key:

```text
instance namespace + action SHA-256
```

---

# 279. Cross-Profile Cache

Forbidden unless proven equivalent.

---

# 280. Result Provenance

Internal metadata can retain:

```text
execution profile
runner trust
sandbox
toolchain image
```

---

# 281. Cache Result Serving

Policy checks required trust level.

---

# 282. Cache Eviction

Does not delete blobs still referenced elsewhere.

---

# 283. Negative Cache

Do not cache execution failures as ActionResult unless protocol/client semantics explicitly permit desired behavior.

---

# 284. ActionResult for Nonzero Exit

REAPI execution result can be cached if action semantics/cache policy allow.

---

# 285. Infrastructure Failure

Never cache as action result.

---

# 286. Timeout

Usually not shared cacheable.

---

# 287. Cancellation

Not cacheable.

---

# 288. Output Upload Failure

No cache publish.

---

# 289. Result Validation

Before publish:

```text
all output digests exist
stdout/stderr refs exist
metadata sane
```

---

# 290. Action Cache Race

Two equivalent executions may finish.

Use compare/idempotent record policy.

---

# 291. Different Result Same Action

Signals non-determinism/cache risk.

---

# 292. Non-Determinism Detection

Optional:

```text
same action digest
different ActionResult output digest
```

emit security/repro warning.

---

# 293. Cache Conflict Policy

Do not silently overwrite trusted result.

---

# 294. Conflict Evidence

Record both execution outcomes.

---

# 295. Strict Mode

Quarantine action cache key after conflicting trusted results until policy resolution.

---

# 296. Reproducibility Integration

Can elevate to reproducibility evidence.

---

# 297. RBE Input Provenance

Not VCS unless linked externally.

---

# 298. Bazel Invocation Metadata

Optional client metadata can be stored for diagnostics.

---

# 299. Privacy

Bound/sanitize user-supplied metadata.

---

# 300. Testkit

```text
forgeyard-rbe-testkit/src/
├── lib.rs
├── digest.rs
├── cas.rs
├── action.rs
├── command.rs
├── execution.rs
├── cache.rs
├── operation.rs
├── platform.rs
└── assertions.rs
```

---

# 301. Unit Tests

Test:

```text
digest mapping
resource parsing
platform mapping
error mapping
```

---

# 302. CAS Conformance

```text
FindMissingBlobs
BatchUpdate
BatchRead
GetTree
ByteStream
```

---

# 303. Digest Mismatch Test

Reject corrupted upload.

---

# 304. Upload Resume Test

Correct offset resumes.

---

# 305. Directory Path Safety Test

Reject malformed tree.

---

# 306. Cache Hit Test

No runner execution.

---

# 307. Cache Trust Test

Untrusted result not served to protected tenant if policy forbids.

---

# 308. Execute Test

Action -> Job -> Attempt -> output -> ActionResult.

---

# 309. Cancellation Test

CancelOperation -> durable cancel.

---

# 310. Disconnect Test

gRPC disconnect does not cancel execution.

---

# 311. Reconnect Test

WaitExecution resumes from operation name.

---

# 312. Platform Test

Unsupported property rejected.

---

# 313. Priority Ceiling Test

Client cannot self-elevate.

---

# 314. Tenant Isolation Test

Cannot read other instance CAS/cache.

---

# 315. Cross-Tenant Blob Test

Physical dedup does not grant visibility.

---

# 316. Cache Conflict Test

Same action different trusted outputs detected.

---

# 317. HA Test

Execute through node A, wait through node B after A fails.

---

# 318. Rolling Upgrade Test

Bazel client survives daemon rollout.

---

# 319. Bazel E2E

Run real Bazel remote cache/build.

---

# 320. Fuzzing

Fuzz:

```text
ByteStream resource names
Directory/Tree protobufs
platform properties
operation names
```

---

# 321. Failure Injection

```text
CAS timeout
DB timeout
scheduler unavailable
runner lost
gRPC disconnect
partial upload
```

---

# 322. Load Tests

```text
many small CAS blobs
large blobs
many concurrent executes
large trees
```

---

# 323. Memory Tests

Streaming prevents giant buffer use.

---

# 324. Performance Metrics

Measure:

```text
CAS throughput
cache lookup latency
queue latency
execution overhead vs native
```

---

# 325. Overhead Goal

RBE adapter overhead should be small relative to actual build execution/CAS transfer.

---

# 326. Implementation Phase 1 — Proto/Model Boundary

Implement generated REAPI types + normalized internal model.

---

# 327. Phase 2 — CAS / ByteStream

Remote cache foundation.

---

# 328. Phase 3 — Action Cache

Read/write policy.

---

# 329. Phase 4 — Capabilities

Accurate service advertisement.

---

# 330. Phase 5 — Execute Mapping

Action -> Forgeyard JobSpec.

---

# 331. Phase 6 — Operations / Cancellation

Long-running lifecycle.

---

# 332. Phase 7 — Platform/Scheduler Mapping

Capability profiles.

---

# 333. Phase 8 — Multi-Tenant/Auth/Quota

Production security.

---

# 334. Phase 9 — Bazel E2E/Compliance

Real interoperability.

---

# 335. Phase 10 — Cache Conflict/Repro Evidence

Hardening.

---

# 336. Phase 11 — HA/Scale

Distributed production.

---

# 337. Phase 12 — External Cache/Federation Extensions

Optional.

---

# 338. Acceptance Tests

1. REAPI is exposed only at interoperability edge.
2. Protobuf types do not leak into scheduler/runner/domain crates.
3. External SHA-256 digests are verified.
4. Internal CAS retains BLAKE3 identity.
5. SHA-256 aliases map to exact CAS objects.
6. ByteStream uploads are resumable and bounded.
7. Partial uploads never become visible CAS objects.
8. Action Cache is tenant/instance isolated.
9. Arbitrary clients cannot poison cache without permission.
10. Execute resolves complete Action/Command/input closure.
11. RBE action becomes normal Forgeyard Job/Attempt/Lease.
12. RBE jobs use normal scheduler fairness.
13. RBE client priority cannot exceed policy ceiling.
14. Platform properties map to hard scheduler requirements.
15. Unknown correctness-affecting platform properties are not silently ignored.
16. Sandbox/network policy remains Forgeyard-controlled.
17. Cache hit can complete without runner execution.
18. Infrastructure failures are never cached as successful action results.
19. Action outputs are durable before cache visibility.
20. Client disconnect does not cancel durable execution.
21. CancelOperation maps to normal cancellation.
22. Operation can be resumed through another daemon.
23. HA failover does not lose operation state.
24. Cross-tenant physical dedup does not grant logical access.
25. Same action/different trusted outputs is detected as non-determinism/conflict.
26. External RBE cache identity is namespaced by Forgeyard execution-profile semantics.
27. REAPI does not require internal gRPC agent transport.
28. Capabilities service advertises only implemented behavior.
29. Standalone mode can expose local RBE cache/execution.
30. Distributed mode scales through normal Forgeyard runners.
31. Real Bazel client passes integration tests.
32. RBE health/doctor exposes CAS/cache/execution status.
33. Secrets/provider credentials are not implicitly exposed through RBE env semantics.
34. Rolling upgrades preserve declared REAPI compatibility.
35. Forgeyard can dogfood RBE-compatible builds without changing core execution architecture.

---

# 339. Production Readiness Gates

Do not call RBE interoperability production-ready until:

```text
CAS/ByteStream conformance stable
SHA-256 alias integrity proven
Action Cache isolation/policy tested
Execute -> Job mapping stable
Operation/cancellation/reconnect semantics tested
platform property mapping documented
real Bazel E2E passes
HA failover passes
message/resource limits hardened
cache poisoning/conflict defenses tested
```

---

# 340. Architectural Invariants

1. RBE is an edge adapter, not a second execution core;
2. internal runner transport remains QUIC/Postcard;
3. protobuf/gRPC types stay adapter-local;
4. external digest semantics remain standards-compatible;
5. internal BLAKE3 remains primary Forgeyard CAS identity;
6. external SHA-256 is always verified;
7. tenant/instance isolation is enforced;
8. RBE actions use normal Forgeyard Job/Attempt/Lease;
9. scheduler fairness/policy applies equally to RBE work;
10. client priority cannot self-elevate;
11. unknown platform requirements are not silently ignored;
12. input trees are path-safe and bounded;
13. partial uploads are not CAS-visible;
14. output objects are durable before Action Cache publication;
15. untrusted clients cannot poison shared cache by default;
16. infrastructure failures are not cached as action results;
17. client disconnect does not imply cancellation;
18. cancellation is explicit/durable;
19. operation state is shared/HA-safe;
20. cross-tenant physical dedup does not imply access;
21. execution-profile differences namespace cache semantics;
22. mutable container/toolchain refs are resolved before strict execution;
23. secrets are not implicitly part of RBE protocol;
24. standard clients work without Forgeyard extensions;
25. Capabilities reports truth, not aspiration;
26. HA node failover is transparent to operation semantics;
27. large CAS transfers stream;
28. message/tree sizes are bounded;
29. standalone/distributed share the same RBE model;
30. Forgeyard dogfoods the interoperability layer without compromising native architecture.

---

# 341. Final Target Architecture

```text
                     Bazel / REAPI Client
                              │
                              ▼
                         gRPC / REAPI
                              │
                              ▼
                     Forgeyard RBE Edge
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
             CAS          Action Cache      Execute
              │               │               │
              ▼               ▼               ▼
        SHA-256 Alias      Cache Policy    JobSpec Adapter
              │               │               │
              └───────────────┼───────────────┘
                              ▼
                       Forgeyard Core
                ┌─────────────┼─────────────┐
                ▼             ▼             ▼
                CAS        Scheduler      Runner
```

---

# 342. Final Architectural Position

CAS interoperability:

```text
REAPI SHA-256 digest
  ↓
verify bytes
  ↓
compute BLAKE3
  ↓
store Forgeyard CAS object
  ↓
register SHA-256 alias
```

Execution interoperability:

```text
REAPI Action
+
Command
+
InputRoot
+
Platform
  ↓
validated normalized RBE spec
  ↓
Forgeyard JobSpec
  ↓
Scheduler
  ↓
JobLease
  ↓
Runner/Sandbox
  ↓
outputs in CAS
  ↓
ActionResult
```

Cache semantics:

```text
instance namespace
+
REAPI Action digest
+
Forgeyard execution-profile namespace
  ↓
trusted Action Cache record
```

The key guarantee is:

> **Forgeyard can behave like a standards-compatible remote cache and remote executor for Bazel/REAPI clients without allowing REAPI to redefine Forgeyard's internal architecture. The standard protocol ends at the edge; the same typed scheduler, leases, sandbox, runners, CAS integrity, policies, and reconciliation continue to govern execution inside the platform.**

---

# 343. New-Repository Sequence

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
