# Forgeyard Core Domain & Foundation System Architecture

**Document type:** Foundational System & Architecture  
**Project:** Forgeyard CI/CD  
**Architecture style:** Modular monolith, single Rust workspace  
**Scope:** Core domain primitives, typed identities, invariants, time, digests, errors, capabilities, configuration contracts, versioning, foundational dependency rules, and composition boundaries  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** This document assumes the already-defined workspace structure, VCS-neutral source model, Change Proposal model, hermetic/reproducible build system, and ecosystem architectures. It does not redefine those systems; it defines the foundation they depend on.

---

# 1. Purpose

The new Forgeyard codebase must begin with a small, stable, strongly typed foundation.

Most architectural failures in large systems begin when:

```text
IDs become strings
states become strings
timestamps become integers
errors become anyhow everywhere
configuration leaks into domain models
database rows become business models
transport DTOs become domain types
platform-specific code leaks into core
```

Forgeyard should prevent this from the first commit.

The foundation must provide:

- typed identifiers;
- typed digests;
- typed timestamps/durations/deadlines;
- stable capability models;
- clear domain invariants;
- explicit execution/application modes;
- domain-safe errors;
- versioned foundational serialization;
- configuration boundaries;
- strongly enforced dependency direction;
- test utilities;
- deterministic canonical primitives.

The central rule is:

> **Forgeyard core defines semantic truth but knows nothing about SQL, Git, Axum, Dioxus, Postgres, Stoolap, S3, operating systems, or language ecosystems.**

---

# 2. Architectural Position

```text
                     Applications
                         │
                         ▼
                Generic Services
                         │
                         ▼
                Domain Models / APIs
                         │
                         ▼
               Forgeyard Core Layer
              ┌──────────┼──────────┐
              ▼          ▼          ▼
            IDs       Digests      Time
              │          │          │
              └──────────┼──────────┘
                         ▼
                     Invariants
```

The core layer is the bottom of the dependency graph.

Nothing foundational depends on higher-level adapters.

---

# 3. New Repository Foundation

Recommended initial tree:

```text
forgeyard/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── architecture.ron
│
└── crates/
    ├── core/
    │   ├── forgeyard-core/
    │   └── forgeyard-core-test/
    ├── ids/
    │   └── forgeyard-ids/
    ├── digest/
    │   └── forgeyard-digest/
    ├── time/
    │   └── forgeyard-time/
    ├── error/
    │   └── forgeyard-error/
    ├── config/
    │   ├── forgeyard-config/
    │   ├── forgeyard-config-loader/
    │   ├── forgeyard-config-schema/
    │   └── forgeyard-config-policy/
    └── protocol/
        ├── forgeyard-envelope/
        ├── forgeyard-wire/
        ├── forgeyard-api-model/
        └── forgeyard-version/
```

This should be the first substantial implementation milestone in the new repository.

---

# 4. Foundational Crates

## 4.1 `forgeyard-core`

Purpose:

```text
system modes
tenancy primitives
project identity references
capability concepts
invariants
generic state principles
small foundational enums
```

Must remain small.

It must not become a dumping ground.

---

## 4.2 `forgeyard-ids`

Owns every strongly typed identifier.

Examples:

```rust
pub struct ProjectId(Ulid);
pub struct PipelineId(Ulid);
pub struct RunId(Ulid);
pub struct JobId(Ulid);
pub struct RunnerId(Ulid);
pub struct LeaseId(Ulid);
pub struct ArtifactId(Ulid);
pub struct ReleaseId(Ulid);
pub struct DeploymentId(Ulid);
pub struct RepositoryId(Digest);
pub struct SourceSnapshotId(Digest);
pub struct ChangeProposalId(Ulid);
pub struct PrincipalId(Ulid);
```

No subsystem should define ad-hoc ID aliases like:

```rust
type ProjectId = String;
```

---

# 5. ID Categories

Forgeyard uses three broad identity classes.

## 5.1 Entity IDs

Mutable lifecycle entities:

```text
ProjectId
RunId
JobId
RunnerId
ReleaseId
DeploymentId
ChangeProposalId
```

Use opaque sortable/random identities such as ULID-like identifiers.

---

## 5.2 Content IDs

Immutable content:

```text
SourceSnapshotId
BlobId
TreeObjectId
ArtifactDigest
DerivationId
ToolchainId
```

Use digest-backed identities.

---

## 5.3 External IDs

External systems:

```text
Git revision
GitHub PR number
Postgres row key
SCM delivery ID
```

Never reuse as internal identity directly.

Wrap them:

```rust
pub struct ExternalRevisionId(String);
pub struct ProviderProposalId(String);
```

---

# 6. ID Rules

1. IDs are never plain `String`.
2. Entity IDs are globally unique.
3. Content IDs derive from canonical content.
4. Display representation is not internal representation.
5. Parsing is fallible.
6. Serialization is version-safe.
7. Equality semantics are obvious from type.
8. IDs from different domains cannot be accidentally compared.

---

# 7. Typed Digest System

Crate:

```text
crates/digest/forgeyard-digest/
```

Tree:

```text
src/
├── lib.rs
├── digest.rs
├── blake3.rs
├── sha256.rs
├── alias.rs
├── encoding.rs
├── canonical.rs
└── error.rs
```

---

# 8. Digest Model

```rust
pub enum DigestAlgorithm {
    Blake3,
    Sha256,
}
```

```rust
pub struct Digest {
    pub algorithm: DigestAlgorithm,
    pub bytes: DigestBytes,
}
```

But content-specific types should wrap `Digest`.

---

# 9. Default Hashing Rule

Forgeyard internal content addressing:

```text
BLAKE3
```

Interop:

```text
SHA-256
```

Examples:

```text
OCI
Bazel RBE
external provenance standards
vendor checksums
```

---

# 10. Digest Aliases

Same bytes may have:

```text
BLAKE3 identity
SHA-256 alias
backend-native hash
```

Model explicitly:

```rust
pub struct DigestAlias {
    pub primary: Digest,
    pub alias: Digest,
}
```

---

# 11. Canonical Hashing Rule

Never hash raw Rust memory layout.

Canonical hashing must use stable schema encoding:

```text
schema version
+
canonical fields
+
deterministic ordering
```

---

# 12. Schema Version in Hash

Any ID whose semantics depend on encoding must include:

```text
schema_version
```

Example:

```text
SourceSnapshotId
DerivationId
TreeObjectId
PolicyDigest
```

This prevents silent identity breakage.

---

# 13. Time Architecture

Crate:

```text
crates/time/forgeyard-time/
```

Tree:

```text
src/
├── lib.rs
├── timestamp.rs
├── duration.rs
├── deadline.rs
├── expiry.rs
├── clock.rs
├── monotonic.rs
└── test_clock.rs
```

---

# 14. Time Types

```rust
pub struct Timestamp(...);
pub struct Duration(...);
pub struct Deadline(...);
pub struct LeaseExpiry(...);
```

Do not use raw `i64` milliseconds throughout domain code.

---

# 15. Wall Clock vs Monotonic Time

Wall clock:

```text
audit timestamp
created_at
release time
provider event time
```

Monotonic:

```text
timeouts
lease local countdown
performance duration
```

Do not mix them.

---

# 16. Clock Trait

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
    fn monotonic_now(&self) -> MonotonicInstant;
}
```

Production:

```text
SystemClock
```

Tests:

```text
TestClock
```

---

# 17. Why Inject Clock

Avoid:

```rust
SystemTime::now()
```

inside domain services.

Benefits:

```text
deterministic tests
expiry simulations
lease tests
approval expiry tests
release window tests
```

---

# 18. Core Error Architecture

Crate:

```text
crates/error/forgeyard-error/
```

Tree:

```text
src/
├── lib.rs
├── code.rs
├── category.rs
├── diagnostic.rs
├── retry.rs
├── user_message.rs
├── context.rs
└── source.rs
```

---

# 19. Error Philosophy

Do not create:

```rust
pub enum ForgeyardError {
    Everything...
}
```

Each subsystem owns typed errors.

Core provides common metadata.

---

# 20. Error Metadata

```rust
pub struct ErrorMetadata {
    pub code: ErrorCode,
    pub category: ErrorCategory,
    pub retry: RetryClass,
    pub user_safe: bool,
}
```

---

# 21. Error Categories

```rust
pub enum ErrorCategory {
    InvalidInput,
    Conflict,
    NotFound,
    PermissionDenied,
    Authentication,
    PolicyDenied,
    Infrastructure,
    DependencyUnavailable,
    Corruption,
    Timeout,
    Cancelled,
    InternalInvariant,
}
```

---

# 22. Retry Classification

```rust
pub enum RetryClass {
    Never,
    Immediate,
    Backoff,
    Reconcile,
    HumanActionRequired,
}
```

This is essential for distributed reliability.

---

# 23. Error Source Separation

Internal diagnostic:

```text
full technical context
```

User/API message:

```text
safe explanation
```

Audit/log:

```text
structured context
```

Never expose secrets through error formatting.

---

# 24. `anyhow` Policy

`anyhow` may be used:

```text
CLI composition
one-off admin tools
top-level application bootstrap
```

Do not use `anyhow::Error` as domain API.

---

# 25. `thiserror` / Typed Error Policy

Subsystem crates should expose:

```rust
#[derive(thiserror::Error)]
pub enum VcsError { ... }
```

or equivalent typed errors.

---

# 26. Forgeyard Modes

Core:

```rust
pub enum ForgeyardMode {
    Standalone,
    DistributedServer,
    DistributedAgent,
    ClientOnly,
}
```

Do not encode mode as booleans:

```text
is_server
is_local
is_distributed
```

---

# 27. Capability Architecture

Forgeyard should reason using capabilities.

```rust
pub trait Capability {
    type Id;
}
```

Examples:

```text
CanExecuteLinux
CanBuildRust
CanAccessAndroidDevice
CanSignWindows
CanUseXcode
CanRunConfidential
```

---

# 28. Capability Categories

```rust
pub enum CapabilityKind {
    Platform,
    Toolchain,
    Runtime,
    Device,
    Security,
    Network,
    Storage,
    Signing,
    Execution,
}
```

---

# 29. Capability IDs

Capabilities must be typed and stable.

Example:

```rust
pub struct CapabilityId(Digest);
```

A capability that includes versioned identity:

```text
Xcode 18.0 SDK X
```

should not collapse into:

```text
"macos"
```

---

# 30. Core Invariants

Crate module:

```text
forgeyard-core/src/invariant.rs
```

Provides utilities for invariant enforcement.

Examples:

```text
job terminal state cannot transition back to Running
expired lease cannot complete job
snapshot digest cannot change
candidate cannot be submitted against wrong target
```

Subsystems define actual rules.

Core provides pattern.

---

# 31. Invariant Violation

```rust
pub struct InvariantViolation {
    pub code: InvariantCode,
    pub entity: Option<EntityRef>,
    pub description: String,
}
```

Invariant violations are serious internal errors.

They should not be silently retried.

---

# 32. State Modeling Principle

Persisted distributed state:

```text
enum + transition validation
```

Do not use typestate as primary persisted-state architecture.

Example:

```rust
pub enum JobState {
    Pending,
    Eligible,
    Leased,
    Preparing,
    Running,
    UploadingOutputs,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
}
```

Transition logic lives separately.

---

# 33. State Transition Pattern

Recommended module:

```text
state.rs
transition.rs
invariant.rs
reason.rs
```

Example:

```rust
pub fn transition(
    from: JobState,
    event: JobEvent,
) -> Result<JobState, JobTransitionError>
```

---

# 34. Local Typestate Usage

Typestate is useful for:

```text
validated configuration builders
resource ownership
transaction builders
protocol handshake builders
candidate construction
```

Avoid across asynchronous persisted entity lifecycle.

---

# 35. Tenant Model

Core type:

```rust
pub struct TenantId(Ulid);
```

Optional standalone configuration may use:

```text
LocalTenant
```

but internal domain should remain tenant-aware enough for distributed/enterprise evolution.

---

# 36. Organization / Project Model

Recommended:

```text
Tenant
  ↓
Organization
  ↓
Project
  ↓
Repository / Pipelines / Runs
```

Do not require organization hierarchy in standalone UI.

But domain can support it.

---

# 37. ProjectRef

```rust
pub struct ProjectRef {
    pub tenant: TenantId,
    pub project: ProjectId,
}
```

Useful for isolation and audit.

---

# 38. Principal Model Boundary

Core only knows:

```rust
pub struct PrincipalId(Ulid);
```

Identity providers live elsewhere.

---

# 39. Principal Kinds

Higher identity model may represent:

```text
human
service
runner
bot
automation
```

Core should only define minimal neutral kind if widely needed.

---

# 40. Actor Model

Audit/events need actor:

```rust
pub enum ActorRef {
    Principal(PrincipalId),
    Runner(RunnerId),
    System(SystemActor),
}
```

Avoid fake users for system operations.

---

# 41. Core References

Avoid cross-crate object embedding.

Prefer identifiers:

```rust
pub struct ArtifactRef {
    pub id: ArtifactId,
}
```

not:

```rust
struct Run {
    artifact: FullArtifactObject,
}
```

for persisted distributed domains.

---

# 42. Version Types

Crate:

```text
crates/protocol/forgeyard-version/
```

Model:

```rust
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}
```

Also:

```text
ConfigSchemaVersion
EventSchemaVersion
SnapshotSchemaVersion
DerivationSchemaVersion
```

Do not reuse one global version for everything.

---

# 43. Versioning Domains

Forgeyard should version independently:

```text
wire protocol
REST API
config schema
event schema
database schema
source snapshot schema
derivation schema
lock schema
plugin API
```

---

# 44. Compatibility Rule

For internal protocol:

```text
N supports N and N-1 where practical
```

rolling upgrade strategy.

Major incompatibility:

```text
explicit handshake rejection
```

---

# 45. Envelope Architecture

Crate:

```text
forgeyard-envelope
```

Model:

```rust
pub struct Envelope<T> {
    pub version: ProtocolVersion,
    pub message_id: MessageId,
    pub correlation_id: Option<CorrelationId>,
    pub sent_at: Timestamp,
    pub payload: T,
}
```

---

# 46. Correlation IDs

```rust
pub struct CorrelationId(Ulid);
```

Used for:

```text
request/response
logs
tracing
reconciliation
audit context
```

---

# 47. Message IDs

At-least-once systems require stable message identity.

```rust
pub struct MessageId(Ulid);
```

---

# 48. Configuration Architecture

```text
crates/config/
├── forgeyard-config/
├── forgeyard-config-loader/
├── forgeyard-config-schema/
└── forgeyard-config-policy/
```

---

# 49. Config Separation

Three layers:

```text
RawConfig
    ↓ parse
ParsedConfig
    ↓ validate
ValidatedConfig
```

Do not let arbitrary raw RON become runtime authority.

---

# 50. Config Sources

Priority may include:

```text
compiled defaults
system config
project config
user config
environment
CLI overrides
```

Exact precedence must be deterministic.

---

# 51. Config Provenance

Each resolved config value may optionally expose source:

```rust
pub struct ConfigValue<T> {
    pub value: T,
    pub origin: ConfigOrigin,
}
```

Useful for `forgeyard doctor` and explainability.

---

# 52. Secret Separation

Configuration contains:

```text
SecretRef
```

not secret value.

Example:

```ron
database_password: Secret("prod/postgres")
```

---

# 53. Environment Variables

Environment is an input boundary.

Rules:

```text
explicit allowlist
typed parse
no hidden behavior
```

Avoid business logic directly calling:

```rust
std::env::var(...)
```

throughout code.

---

# 54. Config Validation

Examples:

```text
Postgres selected but connection config missing
OIDC enabled without issuer
mTLS enabled without trust root
Iroh selected as sole authoritative CAS -> reject
```

---

# 55. Config Policy

Some combinations are syntactically valid but architecturally unsafe.

`forgeyard-config-policy` handles these.

---

# 56. Config Schema Migration

Config schema versions:

```text
v1
v2
...
```

Migration can be:

```text
automatic safe migration
warning
manual required
```

---

# 57. Public API Model

Crate:

```text
forgeyard-api-model
```

Public DTOs must be separate from domain models.

Example:

```rust
pub struct RunResponse { ... }
```

Convert:

```text
domain -> DTO
DTO -> validated command
```

---

# 58. Internal Wire Model

Crate:

```text
forgeyard-wire
```

Contains versioned Postcard messages.

Do not serialize arbitrary internal structs directly.

---

# 59. Why Separate Domain / Wire / API Types

Allows:

```text
internal refactor
without API break
```

and:

```text
wire evolution
without DB/domain corruption
```

---

# 60. Serialization Policy

Domain types:

```text
not automatically serialized everywhere
```

Internal DTOs:

```text
serde + Postcard
```

Public DTOs:

```text
serde + JSON
```

Human config:

```text
serde + RON
```

---

# 61. Deterministic Serialization

For canonical identity:

```text
BTreeMap
sorted vectors
explicit enum representation
schema version
```

Avoid nondeterministic map order.

---

# 62. Path Types

Do not use raw `PathBuf` for remote/canonical domain paths.

Define:

```rust
pub struct CanonicalRepoPath(...);
pub struct SandboxPath(...);
pub struct HostPath(PathBuf);
```

Prevent mixing host path with repository path.

---

# 63. URI / URL Types

Typed:

```rust
pub struct RepositoryUrl(...);
pub struct ApiUrl(...);
pub struct SecretSafeUrl(...);
```

Avoid accidentally logging credentials.

---

# 64. Byte Size Types

```rust
pub struct ByteSize(u64);
```

Do not mix:

```text
bytes
KiB
MiB
```

through raw integers.

---

# 65. Resource Types

```rust
pub struct CpuCount(...);
pub struct MemoryBytes(...);
pub struct DiskBytes(...);
```

Scheduler will depend on these.

---

# 66. Percentage / Ratio Types

Use bounded types for:

```text
cache probability
resource utilization
retry jitter
```

Avoid arbitrary `f64`.

---

# 67. Retry Policy Types

```rust
pub struct RetryPolicy {
    pub max_attempts: NonZeroU16,
    pub backoff: BackoffPolicy,
}
```

---

# 68. Backoff Policy

```rust
pub enum BackoffPolicy {
    Fixed(Duration),
    Exponential {
        initial: Duration,
        max: Duration,
        jitter: JitterPolicy,
    },
}
```

---

# 69. Cancellation Token Boundary

Cancellation is operational, not persisted state.

Use Tokio cancellation primitives behind service interfaces.

Persist final cancellation intent/state separately.

---

# 70. Domain Command Pattern

Commands represent intent:

```rust
pub struct StartRun { ... }
pub struct CancelJob { ... }
pub struct RegisterRunner { ... }
```

Do not directly mutate domain structs from API handlers.

---

# 71. Query Pattern

Queries:

```rust
pub struct GetRun { ... }
pub struct ListJobs { ... }
```

Avoid forcing full CQRS complexity, but separate read intent when useful.

---

# 72. Domain Event Pattern

```rust
pub struct JobStarted { ... }
pub struct RunnerRegistered { ... }
```

Events describe facts, not commands.

---

# 73. Event ID

```rust
pub struct EventId(Ulid);
```

All durable events must have unique identity.

---

# 74. Event Envelope

```rust
pub struct EventEnvelope<E> {
    pub schema_version: EventSchemaVersion,
    pub event_id: EventId,
    pub occurred_at: Timestamp,
    pub actor: ActorRef,
    pub correlation_id: Option<CorrelationId>,
    pub event: E,
}
```

---

# 75. At-Least-Once Foundation

Core assumptions:

```text
messages may repeat
events may repeat
responses may be lost
workers may crash
```

Therefore:

```text
idempotency
leases
reconciliation
```

are first-class architectural principles.

---

# 76. Idempotency Key

```rust
pub struct IdempotencyKey(String);
```

Strong limits:

```text
non-empty
max length
safe character policy
```

---

# 77. Idempotency Scope

```rust
pub struct IdempotencyScope {
    pub tenant: TenantId,
    pub operation: OperationId,
}
```

---

# 78. Operation IDs

Typed operation identifiers help audit/retry:

```rust
pub struct OperationId(Ulid);
```

---

# 79. Capability Registry Boundary

Do not use global singleton registries.

Construct registries at application bootstrap.

```rust
pub struct CapabilityRegistry<T> {
    entries: ...
}
```

Owned by application/service context.

---

# 80. Registry Invariants

1. duplicate IDs rejected;
2. deterministic lookup;
3. immutable after bootstrap where possible;
4. explicit dynamic registration only where plugin architecture requires it.

---

# 81. Application Context

Potential crate:

```text
crates/app-context/forgeyard-app-context/
```

Contains application-layer references to services.

It is **not** a foundational dependency.

Domain crates must not depend on application context.

---

# 82. Logging Context

Core types for safe structured fields:

```text
TenantId
ProjectId
RunId
JobId
CorrelationId
```

Do not log entire arbitrary domain structs.

---

# 83. Secret Redaction

Implement redaction wrappers:

```rust
pub struct Secret<T>(T);
```

Debug output:

```text
<redacted>
```

But real secret storage belongs in `secrets` subsystem.

---

# 84. Sensitive String

For tokens/URLs that may contain secrets:

```rust
pub struct SensitiveString(...);
```

No `Display` unless explicitly safe.

---

# 85. Debug / Display Rules

IDs:

```text
Display allowed
```

Secrets:

```text
Display denied/redacted
```

Large domain structs:

```text
structured logging only
```

---

# 86. Validation Types

Use constructors that guarantee validity:

```rust
pub struct ProjectName(String);
```

```rust
impl ProjectName {
    pub fn new(value: String) -> Result<Self, ProjectNameError>;
}
```

No invalid value after construction.

---

# 87. Name Rules

Examples:

```text
ProjectName
PipelineName
RunnerName
LabelName
```

Validation may include:

```text
length
Unicode policy
reserved names
control character denial
```

---

# 88. Label Model

```rust
pub struct Label {
    pub key: LabelKey,
    pub value: Option<LabelValue>,
}
```

Useful across:

```text
runners
projects
changes
artifacts
deployments
```

---

# 89. Metadata Rule

Avoid generic unbounded:

```rust
HashMap<String, String>
```

for critical domain semantics.

Use typed fields.

Generic metadata is allowed only for extension/non-authoritative contexts.

---

# 90. Extension Metadata

```rust
pub struct ExtensionMetadata {
    pub namespace: ExtensionNamespace,
    pub payload: BoundedBytes,
}
```

Versioned and size-limited.

---

# 91. Bounded Types

Prevent unbounded inputs:

```text
BoundedString
BoundedBytes
BoundedVec
```

especially in:

```text
protocols
API
logs
comments
labels
provider payload metadata
```

---

# 92. Size Limits

Foundational constants/config:

```text
max message size
max label count
max config size
max metadata field size
```

Enforced before expensive processing.

---

# 93. Memory Safety Rule

No `unsafe` in foundational crates unless unavoidable and separately justified.

Prefer:

```rust
#![forbid(unsafe_code)]
```

for:

```text
forgeyard-core
forgeyard-ids
forgeyard-time
forgeyard-config-model
```

---

# 94. Unsafe Boundary

If unsafe is required later:

```text
native
platform
FFI
high-performance specialized crates
```

keep it isolated.

---

# 95. Lint Architecture

Workspace lints in root `Cargo.toml`.

Recommended classes:

```text
warnings
unused
clippy correctness
suspicious
complexity
perf
pedantic selectively
```

Do not enable noisy lint rules blindly.

---

# 96. Missing Docs

Public foundational API should require documentation.

```rust
#![warn(missing_docs)]
```

where practical.

---

# 97. Dependency Policy

Foundation crates should have minimal dependencies.

Example:

```text
forgeyard-ids:
  ulid
  serde optional
  forgeyard-digest
```

Avoid:

```text
tokio
reqwest
sqlx
axum
```

in foundational crates.

---

# 98. Feature Policy

Foundation features must be small and semantic.

Bad:

```text
full
everything
```

Good:

```text
serde
postcard
test-util
```

---

# 99. Test Utility Feature

`test-util` may expose:

```text
fake IDs
TestClock
deterministic digests
builders
```

but should not leak into release logic.

---

# 100. `forgeyard-core-test`

Dedicated test support crate:

```text
src/
├── lib.rs
├── id.rs
├── clock.rs
├── builder.rs
├── fixtures.rs
└── assertions.rs
```

---

# 101. Deterministic Test IDs

Tests may use seeded IDs.

Never rely on random ordering in assertions.

---

# 102. Property Tests

Foundation properties:

```text
ID round-trip
digest round-trip
canonical encoding stable
time ordering
bounded string validation
state transition invariants
```

---

# 103. Fuzz Targets

Initial fuzz:

```text
ID parsing
digest parsing
RON config parsing
Postcard envelope decoding
bounded input decoding
canonical serialization
```

---

# 104. Miri

Core crates are excellent candidates for periodic Miri testing.

---

# 105. Documentation

Each foundational crate gets:

```text
README.md
crate-level rustdoc
```

System-level docs:

```text
docs/architecture/core-foundation.md
```

---

# 106. Root Architecture Manifest

`architecture.ron` should classify foundation crates.

Example:

```ron
(
    crates: {
        "forgeyard-ids": (
            layer: Primitive,
            area: "foundation",
        ),

        "forgeyard-core": (
            layer: Domain,
            area: "foundation",
            allowed_dependency_layers: [Primitive],
        ),

        "forgeyard-config-loader": (
            layer: Adapter,
            area: "config",
        ),
    },
)
```

---

# 107. Architecture Checker

Tool:

```text
tools/forgeyard-architecture-check/
```

Checks:

```text
forbidden dependency direction
forbidden external dependencies
cycle-risk patterns
platform leakage
adapter leakage into domain
```

---

# 108. Example Forbidden Imports

Foundation crates must not depend on:

```text
sqlx
tokio-postgres
axum
dioxus
git libraries
cloud SDKs
Docker clients
Kubernetes clients
```

---

# 109. Example Dependency Flow

```text
forgeyard-ids
      ↑
forgeyard-core
      ↑
forgeyard-vcs-model
      ↑
forgeyard-vcs
      ↑
forgeyard-vcs-git
```

Never:

```text
forgeyard-core
      ↓
forgeyard-vcs-git
```

---

# 110. Configuration Dependency Flow

```text
forgeyard-config
      ↑
forgeyard-config-schema
      ↑
forgeyard-config-policy
      ↑
forgeyard-config-loader
```

`forgeyard-config` should not know:

```text
filesystem
environment
CLI
```

if those can be kept in loader/adapters.

---

# 111. Protocol Dependency Flow

```text
forgeyard-version
      ↑
forgeyard-envelope
      ↑
forgeyard-wire
```

Public API model remains separate.

---

# 112. Database Boundary

Foundation has no DB structs.

Bad:

```rust
#[derive(sqlx::FromRow)]
pub struct Project { ... }
```

inside domain.

Correct:

```text
PostgresRow
   ↓ convert
Project
```

---

# 113. UI Boundary

No Dioxus types in domain.

Correct:

```text
domain state
   ↓ API DTO
UI view model
```

---

# 114. Axum Boundary

No Axum extractor types in domain services.

Routes translate:

```text
HTTP request
  ↓
validated command
  ↓
service
```

---

# 115. Tokio Boundary

Async service traits may use async abstraction.

But simple value/model crates should not depend on Tokio.

---

# 116. Rayon Boundary

Rayon belongs in CPU-heavy implementation crates.

Never foundational domain.

---

# 117. File System Boundary

Core paths are semantic path types.

Host filesystem access belongs in adapters/services.

---

# 118. Environment Boundary

Core does not read process environment.

`forgeyard-config-loader` does.

---

# 119. Network Boundary

Core has no network clients.

---

# 120. Process Execution Boundary

Core never invokes commands.

All tool/process execution later goes through executor abstractions.

---

# 121. Serialization Upgrade Tests

Maintain golden encodings for important stable protocol/config objects.

Do not golden-test every internal Rust struct.

---

# 122. Canonical Encoding Tests

For hash-sensitive structures:

```text
field order changes must not silently change identity
```

Canonical format should be explicit rather than relying on derive behavior.

---

# 123. Compatibility Test Directory

```text
tests/compatibility/
├── protocol_n_minus_1.rs
├── config_upgrade.rs
├── snapshot_schema.rs
└── derivation_schema.rs
```

Foundation provides the version primitives these use.

---

# 124. New Forgeyard Bootstrap Order

Recommended implementation sequence:

```text
1. root workspace
2. forgeyard-digest
3. forgeyard-ids
4. forgeyard-time
5. forgeyard-error
6. forgeyard-version
7. forgeyard-envelope
8. forgeyard-core
9. forgeyard-config
10. config schema/loader/policy
11. architecture checker
12. core testkit
```

Only after this should higher domains start.

---

# 125. Phase 1 — Root Workspace

Create:

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
rustfmt.toml
clippy.toml
deny.toml
architecture.ron
README.md
```

Exit:

```text
cargo check --workspace
```

works on empty foundational crates.

---

# 126. Phase 2 — Digest

Implement:

```text
DigestAlgorithm
Digest
Blake3Digest
Sha256Digest
aliases
parsing/display
canonical hashing helpers
```

Acceptance:

```text
roundtrip
stable display
invalid input rejection
```

---

# 127. Phase 3 — IDs

Implement initial IDs for:

```text
tenant
project
pipeline
run
job
runner
lease
repository
snapshot
artifact
change
principal
```

---

# 128. Phase 4 — Time

Implement:

```text
Timestamp
Duration
Deadline
Clock
SystemClock
TestClock
```

---

# 129. Phase 5 — Error Metadata

Implement:

```text
ErrorCode
ErrorCategory
RetryClass
diagnostic metadata
safe user message
```

---

# 130. Phase 6 — Version / Envelope

Implement:

```text
ProtocolVersion
SchemaVersion types
MessageId
CorrelationId
Envelope<T>
```

---

# 131. Phase 7 — Core

Implement:

```text
ForgeyardMode
TenantRef
ProjectRef
ActorRef
Capability basics
InvariantViolation
```

Keep small.

---

# 132. Phase 8 — Config

Implement:

```text
raw parse
schema version
validation
source precedence
config provenance
```

---

# 133. Phase 9 — Architecture Check

Parse Cargo metadata.

Reject:

```text
domain -> adapter
primitive -> service
core -> platform
```

violations.

---

# 134. Phase 10 — Testkit

Add:

```text
TestClock
deterministic ID factories
assertion helpers
canonical digest fixtures
```

---

# 135. Acceptance Tests

1. `ProjectId` cannot be confused with `RunId`.
2. malformed ID parsing fails.
3. BLAKE3 digest round-trips.
4. SHA-256 alias can reference same content.
5. canonical hashing is deterministic.
6. `Timestamp` and `Duration` cannot be accidentally mixed.
7. `TestClock` controls expiry deterministically.
8. retry classification survives error conversion.
9. secrets are redacted in Debug/Display wrappers.
10. RON config parses into raw config.
11. invalid config never becomes `ValidatedConfig`.
12. config source precedence is deterministic.
13. protocol envelope rejects unsupported major version.
14. message ID/correlation ID survive Postcard round-trip.
15. architecture checker rejects core→Git adapter dependency.
16. architecture checker rejects domain→Postgres adapter dependency.
17. foundation crates build without network/database/UI dependencies.
18. core crate can compile with `#![forbid(unsafe_code)]`.
19. canonical serialization stable across repeated runs.
20. golden compatibility tests detect schema breaking changes.

---

# 136. Production Readiness Gates

Do not consider the foundation stable until:

```text
typed IDs are established
digest schema is explicit
clock abstraction is used
errors are typed/retry-classified
config validation pipeline works
schema versions are independent
protocol envelope works
architecture checker prevents invalid edges
foundation crates have no adapter leakage
```

---

# 137. Architectural Invariants

1. No core entity ID is a plain string.
2. Content IDs are digest-based.
3. External IDs never become internal authority directly.
4. Core has no SQL dependency.
5. Core has no Axum dependency.
6. Core has no Dioxus dependency.
7. Core has no VCS adapter dependency.
8. Core has no cloud SDK dependency.
9. Core never executes external processes.
10. Domain code does not read environment variables directly.
11. Domain code does not read wall clock directly.
12. Canonical identity formats include schema version.
13. Retry behavior is explicit.
14. Secret-bearing values are not printable by default.
15. Public API DTOs are separate from domain types.
16. Wire DTOs are separate from domain types.
17. Configuration is parsed then validated before use.
18. Persisted distributed states use explicit enums + transition rules.
19. Typestate is used only where it improves local correctness.
20. Architecture rules are machine-enforced.
21. Foundation crates remain small and stable.
22. Generic metadata cannot replace typed domain fields.
23. Platform-specific logic never leaks into foundation.
24. One subsystem cannot bypass capability APIs by importing adapters directly unless it is an application composition crate.
25. The new Forgeyard repository must build its architecture from this foundation upward.

---

# 138. Final Core Architecture

```text
                     forgeyard-core
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
  forgeyard-ids     forgeyard-time   forgeyard-digest
        │                 │                 │
        └─────────────────┼─────────────────┘
                          ▼
                   forgeyard-error
                          │
                          ▼
                  version/envelope
                          │
                          ▼
                  config contracts
                          │
                          ▼
                    domain layers
```

And the global dependency rule begins here:

```text
foundation
   ↑
domain
   ↑
capability APIs
   ↑
services
   ↑
adapters
   ↑
applications
```

Never invert this.

---

# 139. First Commit Recommendation

The first meaningful new Forgeyard commit should contain only:

```text
root workspace
architecture.ron
forgeyard-digest
forgeyard-ids
forgeyard-time
forgeyard-error
forgeyard-version
forgeyard-envelope
forgeyard-core
basic config types
architecture-check skeleton
tests
```

Do **not** immediately copy the old Forgeyard daemon, scheduler, runner, or UI into the new repository.

Those should be reintroduced subsystem-by-subsystem against the new contracts.

---

# 140. Old Forgeyard Migration Rule

Keep the old repository:

```text
forgeyard-legacy/
```

or archive it remotely as:

```text
forgeyard-legacy
```

Use it only for:

```text
working algorithms
test cases
protocol lessons
UI ideas
bug history
operational lessons
```

Do not mechanically copy its architecture.

Migration process:

```text
legacy behavior
  ↓
identify required capability
  ↓
map to new architecture
  ↓
write tests
  ↓
reimplement/adapt cleanly
```

This minimizes accidental preservation of old coupling.

---

# 141. Final Recommendation

Start the new Forgeyard.

Do not delete the old repository.

Treat the old Forgeyard as:

```text
reference implementation + historical knowledge
```

and the new Forgeyard as:

```text
architecturally authoritative production codebase
```

The new repository should be built from the bottom upward, beginning with this Core Domain & Foundation architecture, then adding storage, CAS, pipeline, scheduler, runner, sandbox/executor, security/policy, observability, release/deployment, API/UI, and finally advanced distributed/enterprise capabilities.
