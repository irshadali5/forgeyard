# 57 — Forgeyard API/ABI/Schema/Protocol Compatibility, Contract Evolution & Breaking-Change Governance System Architecture

**Document type:** Core API/ABI/Schema/Protocol Compatibility, Contract Evolution, Breaking-Change Detection, Migration Governance & Compatibility Evidence System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** public API compatibility, Rust library/API compatibility, C ABI compatibility, CLI contract compatibility, REST/OpenAPI compatibility, Postcard/protobuf/internal protocol evolution, database schema/migration compatibility, configuration schema evolution, plugin contracts, persisted-state compatibility, package format compatibility, release/update compatibility, consumer impact analysis, deprecation policy, compatibility baselines, breaking-change approvals, migration evidence, and compatibility scorecards  
**Architecture style:** Contract-first, explicit baselines, typed compatibility domains, source/binary/wire/schema separation, migration-aware evolution, consumer-aware impact, evidence-backed decisions, versioned compatibility policy, and no silent breaking changes  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Core Protocol, API/Axum, Packaging, Release, Update Delivery, Plugin Architecture, Configuration, Storage/Migrations, RBE, Artifact Registry, Monorepo Intelligence, Change Proposal, Merge Queue, Supply Chain, Migration, and Test Intelligence. This subsystem provides the cross-cutting compatibility authority needed to evolve Forgeyard itself and the software Forgeyard builds without accidental ecosystem breakage.

---

# 1. Purpose

A mature CI/CD platform evolves continuously.

Forgeyard itself has many contracts:

```text
Rust crates
CLI commands/options/output
REST/JSON APIs
QUIC/Postcard internal messages
database schemas
RON configuration
plugin IPC
RBE/gRPC interoperability
update metadata
artifact/package formats
runner/daemon protocols
```

Forgeyard users also build software with compatibility obligations:

```text
libraries
services
SDKs
CLIs
mobile apps
desktop apps
schemas
plugins
packages
firmware
```

Without explicit compatibility architecture, breaking changes appear as:

```text
"it compiled here"
"the new daemon works with the new agent"
"the migration passed on one database"
"the endpoint still returns 200"
"the struct fields look similar"
```

Those statements are not compatibility guarantees.

The central rule is:

> **Compatibility is always compatibility of a specific contract, against a specific baseline, under a specific compatibility dimension and policy.**

A second rule is:

> **Source compatibility, binary compatibility, wire compatibility, schema compatibility, behavioral compatibility, migration compatibility, and deployment/update compatibility are distinct. Forgeyard must never collapse them into one vague “compatible” flag.**

A third rule is:

> **Every breaking change requires an explicit migration/deprecation strategy or an explicit policy-approved break.**

---

# 2. Architectural Position

```text
                  Proposed Change
                        │
                        ▼
                Contract Extraction
                        │
                        ▼
                 Compatibility Baseline
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
          API          ABI        Protocol
            │           │           │
            └───────────┼───────────┘
                        ▼
               Compatibility Analysis
                        │
                        ▼
                Impact / Consumers
                        │
                        ▼
                  Policy Decision
                        │
              ┌─────────┼─────────┐
              ▼         ▼         ▼
           Accept    Migrate   Reject/Defer
```

---

# 3. Goals

The subsystem MUST:

1. define contract identity;
2. define compatibility baseline identity;
3. define compatibility dimensions;
4. distinguish source/API compatibility;
5. distinguish ABI/binary compatibility;
6. distinguish wire/protocol compatibility;
7. distinguish schema/data compatibility;
8. distinguish config compatibility;
9. distinguish behavioral compatibility;
10. distinguish deployment/update compatibility;
11. support Rust public API analysis;
12. support C/C++ ABI analysis;
13. support CLI contract analysis;
14. support REST/OpenAPI compatibility;
15. support internal Postcard protocol evolution;
16. support protobuf/gRPC compatibility;
17. support database migration compatibility;
18. support RON/config schema evolution;
19. support plugin protocol compatibility;
20. support package/artifact format compatibility;
21. support consumer impact analysis;
22. support deprecation lifecycle;
23. support migration plans;
24. support compatibility policy;
25. support evidence and audit;
26. support UI/API/CLI;
27. support merge/release gates;
28. support historical baselines;
29. support air-gap/multi-version environments;
30. avoid accidental breaking change.

---

# 4. Non-Goals

This subsystem does not:

```text
prove semantic equivalence of arbitrary software
replace tests
replace package managers
replace database migration engines
replace versioning policy
guarantee third-party undocumented behavior
```

---

# 5. Workspace Structure

```text
crates/compatibility/
├── forgeyard-compatibility/
├── forgeyard-compatibility-model/
├── forgeyard-compatibility-contract/
├── forgeyard-compatibility-baseline/
├── forgeyard-compatibility-api/
├── forgeyard-compatibility-abi/
├── forgeyard-compatibility-protocol/
├── forgeyard-compatibility-schema/
├── forgeyard-compatibility-config/
├── forgeyard-compatibility-impact/
├── forgeyard-compatibility-policy/
├── forgeyard-compatibility-evidence/
├── forgeyard-compatibility-health/
└── forgeyard-compatibility-testkit/
```

Adapters:

```text
crates/compatibility-adapters/
├── forgeyard-compat-rust/
├── forgeyard-compat-c-abi/
├── forgeyard-compat-openapi/
├── forgeyard-compat-protobuf/
├── forgeyard-compat-postcard/
├── forgeyard-compat-sql/
├── forgeyard-compat-ron/
├── forgeyard-compat-cli/
└── forgeyard-compat-custom/
```

---

# 6. ContractId

```rust
pub struct ContractId(Digest);
```

Immutable identity for one extracted contract representation.

---

# 7. ContractKind

```rust
pub enum ContractKind {
    RustApi,
    CAbi,
    Cli,
    RestApi,
    OpenApi,
    Protobuf,
    PostcardProtocol,
    DatabaseSchema,
    ConfigSchema,
    PluginProtocol,
    ArtifactFormat,
    PackageFormat,
    UpdateProtocol,
    Custom(ContractKindId),
}
```

---

# 8. ContractSubject

```rust
pub enum ContractSubject {
    Package(PackageId),
    Artifact(ArtifactId),
    Project(ProjectId),
    Component(SoftwareComponentId),
    Service(SoftwareComponentId),
    Protocol(ProtocolId),
    Database(DatabaseId),
    ConfigDomain(ConfigDomainId),
}
```

---

# 9. Contract Extraction

A contract is extracted from exact immutable input.

Examples:

```text
Rust crate at SourceSnapshotId
OpenAPI file digest
protobuf descriptor set
database schema snapshot
CLI command model
plugin protocol schema
```

---

# 10. Contract Extraction Version

```rust
pub struct ContractExtractorVersion(u16);
```

Extraction logic changes over time.

---

# 11. CompatibilityBaselineId

```rust
pub struct CompatibilityBaselineId(Digest);
```

Represents exact reference contract.

---

# 12. Baseline Sources

```rust
pub enum CompatibilityBaselineSource {
    PreviousRelease(ReleaseId),
    PreviousPackage(PackageVersionId),
    ExplicitContract(ContractId),
    TargetBranch(SourceSnapshotId),
    InstalledVersion(InstalledVersionRef),
    ConsumerSelected(ContractId),
}
```

---

# 13. No Mutable Branch Name as Final Baseline Identity

Resolve to exact snapshot.

---

# 14. CompatibilityDimension

```rust
pub enum CompatibilityDimension {
    Source,
    Binary,
    Wire,
    Schema,
    Behavioral,
    Migration,
    Deployment,
    Configuration,
}
```

---

# 15. CompatibilityResult

```rust
pub enum CompatibilityResult {
    Compatible,
    CompatibleWithMigration,
    PotentiallyBreaking,
    Breaking,
    Inconclusive,
    NotApplicable,
}
```

---

# 16. Inconclusive Is First-Class

Critical.

---

# 17. CompatibilityFinding

```rust
pub struct CompatibilityFinding {
    pub dimension: CompatibilityDimension,
    pub result: CompatibilityResult,
    pub path: CompatibilityPath,
    pub old: Option<ContractFragmentRef>,
    pub new: Option<ContractFragmentRef>,
    pub rationale: BoundedString,
}
```

---

# 18. CompatibilityReportId

```rust
pub struct CompatibilityReportId(Digest);
```

---

# 19. Compatibility Report

```rust
pub struct CompatibilityReport {
    pub id: CompatibilityReportId,
    pub baseline: ContractId,
    pub candidate: ContractId,
    pub policy: CompatibilityPolicyId,
    pub findings: Vec<CompatibilityFinding>,
}
```

---

# 20. Compatibility Policy

```rust
pub struct CompatibilityPolicyId(Digest);
```

---

# 21. Policy Examples

```text
public Rust crate: no source-breaking change in minor release
C ABI: no binary break in patch/minor
REST v1: additive-only
internal daemon-agent: N/N-1 wire support required
database: expand-contract required
config: old config must parse for one major release
CLI: existing automation output stable
```

---

# 22. Versioning Policy

SemVer may be an input, not the entire compatibility engine.

---

# 23. SemVer

Helpful for package/public API policy.

---

# 24. SemVer Does Not Prove Compatibility

Critical.

---

# 25. Rust API Compatibility

Analyze public:

```text
types
traits
functions
methods
consts
features
generic bounds
visibility
repr attributes
```

---

# 26. Source Breaking Examples

```text
remove public function
rename method
tighten generic bound
remove enum variant in exhaustive public enum
change trait requirement
```

---

# 27. Rust Non-Exhaustive

Can intentionally preserve evolution room.

---

# 28. Feature Flags

Public feature removal/change can be breaking.

---

# 29. MSRV

Minimum Supported Rust Version can be compatibility dimension.

---

# 30. RustToolchainCompatibility

```rust
pub struct RustToolchainCompatibility {
    pub old_msrv: RustVersion,
    pub new_msrv: RustVersion,
}
```

---

# 31. MSRV Increase

Policy-controlled breaking/major/minor decision.

---

# 32. Cargo Features

Default feature changes can be behavior/build compatibility issue.

---

# 33. Cargo Public Dependency

May expose semver implications.

---

# 34. Rust ABI

Rust ABI is generally not stable unless explicitly controlled.

---

# 35. Critical Rule

Do not promise Rust ABI compatibility by default.

---

# 36. C ABI

For exported stable FFI.

---

# 37. C ABI Contract

Extract:

```text
symbol names
calling convention
parameter types
struct layout
alignment
enum repr
visibility
```

---

# 38. `repr(C)`

Relevant.

---

# 39. Binary Compatibility

Requires platform/architecture context.

---

# 40. AbiPlatform

```rust
pub struct AbiPlatform {
    pub os: OperatingSystem,
    pub arch: Architecture,
    pub abi: AbiFamily,
}
```

---

# 41. Struct Layout Change

Potentially breaking.

---

# 42. Symbol Removal

Breaking.

---

# 43. Symbol Addition

Usually compatible.

---

# 44. C++ ABI

Compiler/toolchain-specific.

---

# 45. If Supported

Record compiler/stdlib/ABI version.

---

# 46. CLI Contract

CLIs are APIs for humans and automation.

---

# 47. CliContract

Includes:

```text
command names
subcommands
flags/options
required args
exit codes
stdout/stderr machine formats
JSON/RON schema
```

---

# 48. Human Text

May be less stable unless explicitly contractually frozen.

---

# 49. Machine Output

Must be versioned/stable.

---

# 50. Remove Flag

Breaking.

---

# 51. Change Default

Behaviorally breaking.

---

# 52. Exit Code Change

Can break automation.

---

# 53. REST API

Analyze:

```text
paths
methods
request schema
response schema
status codes
auth requirements
pagination
headers
error schema
```

---

# 54. REST Additive Change

Often compatible.

---

# 55. Required Request Field Addition

Breaking.

---

# 56. Removing Response Field

Breaking for strict consumers.

---

# 57. Widening Enum

Potentially breaking for exhaustive consumers.

---

# 58. Policy Must Define

Critical.

---

# 59. OpenAPI

Useful contract representation.

---

# 60. OpenAPI Diff

Adapter normalizes.

---

# 61. REST Behavior

Schema-only diff cannot prove behavior.

---

# 62. Behavioral Compatibility

Use tests/contract tests.

---

# 63. Wire Protocol

Forgeyard internal:

```text
QUIC + Postcard Envelope<T>
```

---

# 64. ProtocolVersion

Already existing.

---

# 65. Postcard Compatibility

Postcard serialization can be fragile to enum/field layout changes depending representation.

---

# 66. Critical Rule

Never assume Rust struct evolution is wire-compatible.

---

# 67. Versioned Messages

Prefer:

```rust
enum AgentMessage {
    V1(AgentMessageV1),
    V2(AgentMessageV2),
}
```

or negotiated schema/version.

---

# 68. Envelope Version

Explicit.

---

# 69. Compatibility Matrix

```rust
pub struct ProtocolCompatibilityMatrix {
    pub local: ProtocolVersion,
    pub supported_remote: VersionRange,
}
```

---

# 70. N/N-1

Common rolling-upgrade requirement.

---

# 71. Unknown Field Handling

Depends on format/schema strategy.

---

# 72. Protobuf

Unknown fields preserved/ignored according implementation.

Still evaluate:

```text
field number reuse
wire type change
required semantics
enum values
service methods
```

---

# 73. Never Reuse Protobuf Field Number

Critical.

---

# 74. Removed Field

Reserve number/name.

---

# 75. gRPC Method Removal

Breaking.

---

# 76. RBE Compatibility

Part 23.

---

# 77. Database Schema

Compatibility dimensions:

```text
read compatibility
write compatibility
migration compatibility
rollback compatibility
mixed-version app compatibility
```

---

# 78. SchemaSnapshotId

```rust
pub struct SchemaSnapshotId(Digest);
```

---

# 79. MigrationSetId

Existing/migration identity.

---

# 80. Database Compatibility

```rust
pub struct DatabaseCompatibility {
    pub old: SchemaSnapshotId,
    pub new: SchemaSnapshotId,
    pub migration: MigrationSetId,
}
```

---

# 81. Expand-Contract

Preferred for rolling upgrades.

---

# 82. Example

```text
add nullable/new column
deploy readers/writers compatible with both
backfill
switch reads
remove old column later
```

---

# 83. Destructive Migration

Requires explicit policy/backup/recovery evidence.

---

# 84. Mixed-Version Window

Critical.

---

# 85. Database Migration Test

Part 56 can test old snapshot -> new migration.

---

# 86. Rollback

May be impossible after data transformation.

---

# 87. RollbackCompatibility

```rust
pub enum RollbackCompatibility {
    Safe,
    SafeBeforeCutover,
    ManualRecovery,
    Irreversible,
    Unknown,
}
```

---

# 88. Configuration Compatibility

RON config evolves.

---

# 89. ConfigContract

Includes:

```text
fields
types
defaults
required/optional
scope
deprecations
```

---

# 90. Remove Config Field

Breaking unless migration/ignored legacy behavior exists.

---

# 91. Change Default

Behavioral change.

---

# 92. Config Version

Explicit where needed.

---

# 93. Legacy Config Parser

Can support old version.

---

# 94. Migration Tool

Transforms old config to new.

---

# 95. No Silent Meaning Change

Critical.

---

# 96. Plugin Compatibility

Part 24 process IPC avoids stable Rust ABI.

---

# 97. PluginProtocolVersion

Explicit.

---

# 98. Plugin Manifest Compatibility

Includes:

```text
host protocol
permissions
capabilities
config schema
```

---

# 99. Host Upgrade

May support N/N-1 plugin protocol.

---

# 100. Unsupported Plugin

Fail clearly.

---

# 101. Artifact Format Compatibility

Examples:

```text
Forgeyard offline bundle
reproduction bundle
update bundle
air-gap bundle
```

---

# 102. FormatVersion

Always explicit.

---

# 103. Reader Compatibility

New Forgeyard should read old supported archive versions.

---

# 104. Writer Compatibility

May optionally produce older format for interoperability.

---

# 105. Update Protocol Compatibility

Part 41.

---

# 106. Installer/Updater

Must understand signed metadata format/version.

---

# 107. Anti-Rollback

Compatibility does not override minimum security generation.

---

# 108. Agent/Daemon Compatibility

High importance.

---

# 109. Compatibility Domains

```text
daemon ↔ agent
daemon ↔ CLI
daemon ↔ UI
daemon ↔ device agent
daemon ↔ signing worker
```

---

# 110. ComponentCompatibilityEdge

```rust
pub struct ComponentCompatibilityEdge {
    pub producer: ComponentVersionRef,
    pub consumer: ComponentVersionRef,
    pub contract: ContractId,
}
```

---

# 111. Compatibility Graph

Derived.

---

# 112. Consumer-Aware Impact

A contract can be technically breaking but have no active consumers.

---

# 113. ConsumerImpactId

```rust
pub struct ConsumerImpactId(Digest);
```

---

# 114. Consumer Sources

```text
monorepo dependency graph
package downloads/lockfiles
service catalog relations
deployment inventory
plugin registry
explicit SDK consumers
```

---

# 115. Consumer Confidence

```rust
pub enum ConsumerConfidence {
    Exact,
    Declared,
    Observed,
    Inferred,
    Unknown,
}
```

---

# 116. No "No Consumers" Without Coverage

Critical.

---

# 117. Impact Scope

```text
same repository
organization
tenant
installation
external/public
```

---

# 118. Public Package

Unknown external consumers must be assumed.

---

# 119. Internal Private Contract

Can analyze known consumers more strongly.

---

# 120. Compatibility Impact Report

```rust
pub struct CompatibilityImpactReport {
    pub contract: ContractId,
    pub findings: Vec<CompatibilityFinding>,
    pub consumers: Vec<ConsumerImpact>,
}
```

---

# 121. Affected Work

Part 34 can identify consumer projects/tests to run.

---

# 122. Contract Test Selection

When API changes, require relevant consumer/contract tests.

---

# 123. Change Proposal Evidence

Compatibility report is semantic evidence.

---

# 124. Merge Queue

Protected target policy can require compatibility pass.

---

# 125. Release Gate

Public package release can require compatibility policy.

---

# 126. Breaking Change Approval

```rust
pub struct BreakingChangeApproval {
    pub report: CompatibilityReportId,
    pub reason: BoundedString,
    pub migration_plan: Option<MigrationPlanRef>,
    pub approved_by: PrincipalId,
}
```

---

# 127. Breaking Approval

Binds exact report/candidate.

---

# 128. New Change

Invalidates approval.

---

# 129. Deprecation

First-class.

---

# 130. DeprecationId

```rust
pub struct DeprecationId(Ulid);
```

---

# 131. Deprecation Lifecycle

```rust
pub enum DeprecationState {
    Announced,
    Active,
    RemovalEligible,
    Removed,
    Cancelled,
}
```

---

# 132. Deprecation Metadata

```text
introduced version
replacement
removal earliest version/date
migration docs
```

---

# 133. No Instant Removal by Default

For protected/public contracts.

---

# 134. Deprecation Window

Policy-defined.

---

# 135. Usage Telemetry

Can inform whether deprecated feature still used.

---

# 136. Privacy

Aggregate carefully.

---

# 137. No Telemetry Needed for Public Guarantee

Unknown consumers still matter.

---

# 138. Migration Plan

Breaking change should have explicit migration.

---

# 139. CompatibilityMigrationPlan

```rust
pub struct CompatibilityMigrationPlan {
    pub from: ContractId,
    pub to: ContractId,
    pub steps: Vec<MigrationStep>,
    pub rollback: RollbackCompatibility,
}
```

---

# 140. Migration Step

Examples:

```text
update client SDK
dual-read old/new field
deploy server first
migrate DB
switch client
remove old field later
```

---

# 141. Deployment Order

First-class.

---

# 142. UpgradeOrder

```rust
pub struct UpgradeOrder {
    pub steps: Vec<ComponentUpgradeStep>,
}
```

---

# 143. Rolling Upgrade

Example:

```text
daemon supports agent N/N-1
  ↓
upgrade daemon
  ↓
upgrade agents
  ↓
drop N-1 in later major/minor
```

---

# 144. Downgrade Compatibility

Separate.

---

# 145. DowngradePolicy

```rust
pub enum DowngradePolicy {
    Supported,
    SupportedBeforeMigrationCommit,
    ManualRecovery,
    Forbidden,
}
```

---

# 146. Compatibility Window

```rust
pub struct CompatibilityWindow {
    pub min_supported: VersionRef,
    pub max_supported: VersionRef,
}
```

---

# 147. Installation Mixed-Version Matrix

UI/doctor.

---

# 148. Unsupported Version Pair

Block upgrade/connection.

---

# 149. No Hope-Based Mixed Versions

Critical.

---

# 150. Compatibility Evidence

```rust
pub enum CompatibilityEvidence {
    StaticDiff,
    ContractTest,
    ConsumerBuild,
    ConsumerTest,
    MigrationTest,
    MixedVersionTest,
    DeploymentCanary,
    ManualReview,
}
```

---

# 151. Static Diff

Useful but limited.

---

# 152. Contract Test

Behavioral confidence.

---

# 153. Consumer Build/Test

High-value internal evidence.

---

# 154. Migration Test

Schema/config/state.

---

# 155. Mixed-Version Test

Daemon/agent/plugin/DB compatibility.

---

# 156. Compatibility Confidence

```rust
pub enum CompatibilityConfidence {
    High,
    Medium,
    Low,
    Unknown,
}
```

---

# 157. Confidence Separate From Result

Compatible + Low confidence possible.

---

# 158. Inconclusive

Never silently accepted.

---

# 159. Contract Test Environment

Part 56.

---

# 160. Service Virtualization

Can test consumer/provider contracts.

---

# 161. Real Mixed-Version Test

Required for critical protocol changes.

---

# 162. API Versioning

REST:

```text
/v1
/v2
```

where major break needed.

---

# 163. Avoid Version for Every Additive Change

---

# 164. Header/Media Versioning

Optional.

---

# 165. Internal Protocol Negotiation

Version range handshake.

---

# 166. Feature Negotiation

Capabilities explicit.

---

# 167. ProtocolFeatureId

```rust
pub struct ProtocolFeatureId(u32);
```

---

# 168. Feature Availability

Negotiated after version.

---

# 169. Unknown Feature

Ignored/rejected according protocol.

---

# 170. No Implicit Feature Based on Binary Version String Alone

Critical.

---

# 171. Database Compatibility With Rolling Deploy

App versions:

```text
old app
new app
```

must coexist during migration window if rolling update.

---

# 172. DB Write Compatibility

Both versions may write.

---

# 173. Contract Matrix

```text
old app ↔ transitional schema
new app ↔ transitional schema
```

---

# 174. Migration Commit Point

After which old app no longer supported.

---

# 175. Update Subsystem

Part 41 can block rollback past commit point.

---

# 176. Config Mixed Version

Old agent may ignore new optional field if safe.

---

# 177. Required New Config Field

Can break old version.

---

# 178. Config Projection

Send version-compatible subset.

---

# 179. No Secret Downgrade

Compatibility never means exposing old insecure secret mechanism.

---

# 180. Security Overrides Compatibility

Critical.

---

# 181. Vulnerable Old Protocol

Can be revoked even inside previous compatibility window.

---

# 182. SecurityMinimumVersion

```rust
pub struct SecurityMinimumVersion(VersionRef);
```

---

# 183. Compatibility Policy Must Respect Security Floor

---

# 184. Artifact Registry

Part 52 package metadata can expose:

```text
compatibility report
minimum runtime
deprecation
```

---

# 185. Package Metadata

Optional.

---

# 186. Rust Crate Release

Compatibility report attached to package release.

---

# 187. SDK Generation

OpenAPI/protobuf change can regenerate SDK and compile consumers.

---

# 188. Generated Code

Not sole compatibility evidence.

---

# 189. CLI Automation Consumer

Can test scripts using machine output.

---

# 190. Golden Fixtures

CLI output schema fixtures.

---

# 191. Exit Code Matrix

Explicit.

---

# 192. Persisted State Compatibility

Beyond DB:

```text
local Stoolap
cache metadata
runner state
offline bundles
client state
```

---

# 193. PersistedStateContract

```rust
pub struct PersistedStateContract {
    pub format: ContractId,
    pub reader_versions: VersionRange,
    pub writer_versions: VersionRange,
}
```

---

# 194. Cache

Can be discarded if incompatible.

---

# 195. Authoritative State

Must migrate.

---

# 196. Client Local DB

Migration path mandatory.

---

# 197. Offline Site

Part 51 may return after multiple versions.

---

# 198. Upgrade Bridge

Need supported migration path.

---

# 199. Direct Skip Upgrade

Policy.

---

# 200. Migration Graph

```rust
pub struct VersionMigrationGraph {
    pub edges: Vec<SupportedMigrationEdge>,
}
```

---

# 201. Upgrade Planner

Find supported path.

---

# 202. No Unsupported Version Leap

Critical.

---

# 203. Plugin Protocol

Plugin can declare host compatibility range.

---

# 204. Plugin Installation

Reject incompatible host.

---

# 205. Plugin Upgrade

Can test protocol handshake.

---

# 206. RBE

Maintain REAPI version semantics.

---

# 207. OCI/Registry

Standards adapter compatibility tracked separately from internal contract.

---

# 208. Public Standards

Use conformance suites where available.

---

# 209. Compatibility Policy Scope

```rust
pub enum CompatibilityPolicyScope {
    Installation,
    Organization,
    Project,
    Package,
    Contract,
}
```

---

# 210. Lower Scope

Can be stricter than mandatory floor.

---

# 211. Cannot Weaken Mandatory System Compatibility Requirement

---

# 212. BreakingChangeClass

```rust
pub enum BreakingChangeClass {
    Source,
    Binary,
    Wire,
    Data,
    Config,
    Behavioral,
    Operational,
    Security,
}
```

---

# 213. Severity

```rust
pub enum BreakingSeverity {
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 214. Example Critical

```text
old agent accepts new daemon message incorrectly
DB migration corrupts old client writes
```

---

# 215. Compatibility Budget

Not error budget.

Avoid.

---

# 216. No Arbitrary "X Breaking Changes Allowed"

Critical.

---

# 217. Compatibility Exceptions

Explicit/time-bound/version-bound.

---

# 218. ExceptionRecord

```rust
pub struct CompatibilityException {
    pub report: CompatibilityReportId,
    pub scope: CompatibilityPath,
    pub reason: BoundedString,
    pub expires_at: Option<Timestamp>,
}
```

---

# 219. Exception Does Not Alter Finding

Critical.

---

# 220. It alters policy decision.

---

# 221. Compatibility Gate

```rust
pub enum CompatibilityGateResult {
    Pass,
    Warning,
    Fail,
    Incomplete,
}
```

---

# 222. Gate Inputs

Exact report + policy + exceptions.

---

# 223. Merge Gate

Change Proposal.

---

# 224. Release Gate

Release candidate.

---

# 225. Update Gate

Forgeyard component update.

---

# 226. Compatibility Snapshot

For release:

```rust
pub struct ReleaseCompatibilitySnapshot {
    pub release: ReleaseId,
    pub contracts: Vec<ContractId>,
    pub reports: Vec<CompatibilityReportId>,
}
```

---

# 227. Release Notes

Can derive breaking/deprecation sections.

---

# 228. Changelog

Machine-assisted.

---

# 229. AI

Part 55 may summarize compatibility report.

---

# 230. AI Cannot override result.

---

# 231. Compatibility Diff UI

Dioxus pages:

```text
Compatibility
Contracts
Breaking Changes
Deprecations
Migration Plans
Version Matrix
```

---

# 232. Report Detail

Shows:

```text
baseline
candidate
dimension
findings
consumers
evidence
policy decision
```

---

# 233. Side-by-Side Contract Diff

Useful.

---

# 234. Consumer Impact UI

Shows known consumers and confidence.

---

# 235. Version Matrix UI

Example:

```text
Daemon 5.2 ↔ Agent 5.1  Supported
Daemon 5.2 ↔ Agent 5.0  Unsupported
```

---

# 236. CLI

```text
forgeyard compat check
forgeyard compat api
forgeyard compat abi
forgeyard compat protocol
forgeyard compat schema
forgeyard compat consumers
forgeyard compat explain
forgeyard compat matrix
forgeyard compat doctor
```

---

# 237. Example

```text
forgeyard compat check --baseline release:1.4.0
```

---

# 238. API

Potential:

```text
POST /v1/compatibility/check
GET  /v1/compatibility/reports/{id}
GET  /v1/compatibility/contracts/{id}
GET  /v1/compatibility/matrix
GET  /v1/compatibility/deprecations
```

---

# 239. Permissions

```text
compatibility.read
compatibility.policy.manage
compatibility.exception.approve
compatibility.baseline.manage
```

---

# 240. Breaking Exception

High privilege depending contract.

---

# 241. Audit

Audit:

```text
compat policy change
breaking exception
baseline override
deprecation removal
migration approval
```

---

# 242. Routine compatibility runs

Operational evidence.

---

# 243. Notifications

Examples:

```text
breaking change detected
deprecation removal eligible
consumer build failed
mixed-version support lost
```

---

# 244. Search

Part 31 indexes:

```text
contracts
breaks
deprecations
consumers
```

---

# 245. Catalog

Part 49 component page can show compatibility status/contracts.

---

# 246. Monorepo

Part 34 maps changed contract to affected consumers.

---

# 247. Merge Queue

Part 54 can require compatibility evidence for exact integration candidate.

---

# 248. Candidate Contract

Extract from result snapshot.

---

# 249. Not Proposal Head Only

Critical.

---

# 250. Release

Part 15 uses exact release candidate contract.

---

# 251. Packaging

Package may declare runtime compatibility metadata.

---

# 252. Update

Part 41 consults compatibility matrix before installation.

---

# 253. Federation

Part 51 mixed-version site joins consult protocol/version matrix.

---

# 254. Disconnected Site

Cannot reconnect with unsupported direct version gap.

---

# 255. Upgrade Bridge

May require intermediate version.

---

# 256. Doctor

```text
forgeyard compat doctor
```

Checks:

```text
missing baseline
unsupported component matrix
stale compatibility report
unresolved breaking exception
deprecation removal without migration
DB migration lacking mixed-version test
```

---

# 257. Health

```rust
pub enum CompatibilityHealth {
    Healthy,
    Warnings,
    BreakingPending,
    Incomplete,
    Unhealthy,
}
```

---

# 258. Observability Metrics

```text
compatibility_checks_total
compatibility_breaking_findings_total
compatibility_inconclusive_total
compatibility_consumer_failures_total
compatibility_exceptions_active
```

---

# 259. Labels

Low cardinality:

```text
contract_kind
dimension
result
```

---

# 260. Tracing

```text
compat.extract
compat.baseline
compat.diff
compat.consumer
compat.policy
compat.migration
```

---

# 261. Data Lifecycle

Part 46.

Compatibility reports for public releases may be long-lived.

---

# 262. Contract Artifacts

CAS-backed.

---

# 263. Consumer Telemetry

Privacy policy.

---

# 264. Historical Baseline

Retain exact release contract even after source deletion if release evidence requires.

---

# 265. Air-Gap

Compatibility checking works from bundled contracts/tooling.

---

# 266. External Consumer

Unknown source.

---

# 267. Public Contract Policy

Conservative.

---

# 268. Internal Contract Policy

Can use exact consumer inventory.

---

# 269. Test Matrix

Critical protocols need:

```text
N server ↔ N client
N server ↔ N-1 client
N-1 server ↔ N client if supported
```

---

# 270. Mixed-Version E2E

Part 56 test environments.

---

# 271. Database Matrix

```text
old app + transitional schema
new app + transitional schema
```

---

# 272. Plugin Matrix

Host/plugin versions.

---

# 273. CLI Matrix

Old scripts vs new CLI.

---

# 274. Contract Fuzzing

Useful for wire/schema robustness.

---

# 275. Protocol Fuzz

Unknown/old/new messages.

---

# 276. Deserialization Safety

Never panic/UB on unsupported message.

---

# 277. Reject Clearly

---

# 278. Schema Unknown

Explicit error/version negotiation.

---

# 279. Testkit

```text
forgeyard-compatibility-testkit/src/
├── lib.rs
├── contract.rs
├── api.rs
├── abi.rs
├── protocol.rs
├── schema.rs
├── config.rs
├── consumer.rs
└── assertions.rs
```

---

# 280. Unit Tests

Contract identity determinism.

---

# 281. Rust API Test

Removed public function detected.

---

# 282. Rust MSRV Test

Policy detects unsupported increase.

---

# 283. C ABI Test

Symbol/layout break detected.

---

# 284. CLI Test

Flag/exit-code/machine-output break detected.

---

# 285. REST Test

Required field addition detected.

---

# 286. Enum Widening Test

Policy-dependent classification.

---

# 287. Postcard Test

Changed enum layout not assumed compatible.

---

# 288. Protobuf Test

Field-number reuse blocked.

---

# 289. DB Expand-Contract Test

Mixed versions pass.

---

# 290. Destructive Migration Test

Requires explicit policy/evidence.

---

# 291. Config Default Test

Behavioral change surfaced.

---

# 292. Plugin Protocol Test

Unsupported host/plugin pair rejected.

---

# 293. Update Matrix Test

Unsupported direct upgrade blocked.

---

# 294. Security Floor Test

Vulnerable old version rejected even if compatibility window says otherwise.

---

# 295. Consumer Impact Test

Known consumers listed with confidence.

---

# 296. Public Consumer Test

Unknown external consumers remain assumed.

---

# 297. Exception Test

Finding remains Breaking; policy gate exception only.

---

# 298. Deprecation Test

Removal before minimum window blocked.

---

# 299. Stale Report Test

Candidate change invalidates report.

---

# 300. Merge Candidate Test

Contract extracted from integrated result.

---

# 301. Federation Test

Unsupported site protocol version cannot join authority.

---

# 302. Fuzzing

Fuzz:

```text
contract parsers
wire compatibility parser
schema diff
CLI schema
config migration payload
```

---

# 303. Property Tests

Compatible result must never include a known policy-defined breaking finding.

---

# 304. Scale Test

Large monorepo/public API graph.

---

# 305. Chaos Tests

```text
baseline artifact missing
consumer graph partial
mixed-version test failure
migration worker crash
```

---

# 306. Implementation Phase 1 — Contract/Baseline Model

Core abstractions.

---

# 307. Phase 2 — Rust API + CLI Compatibility

Dogfood Forgeyard.

---

# 308. Phase 3 — Internal Protocol/Postcard Matrix

Critical upgrades.

---

# 309. Phase 4 — Database/Config Compatibility

Operations.

---

# 310. Phase 5 — REST/OpenAPI

Public API.

---

# 311. Phase 6 — Plugin/RBE/Protobuf

Interop.

---

# 312. Phase 7 — Consumer Impact Graph

Part 34/49.

---

# 313. Phase 8 — Deprecation/Migration Governance

Enterprise.

---

# 314. Phase 9 — Release/Merge/Update Gates

Enforcement.

---

# 315. Phase 10 — C ABI/C++ ABI

Selective.

---

# 316. Phase 11 — Federation/Air-Gap Version Matrix

Distributed.

---

# 317. Phase 12 — Fuzz/Scale/DR Hardening

Production readiness.

---

# 318. Acceptance Tests

1. Every compatibility decision identifies exact baseline and candidate contracts.
2. Compatibility dimensions remain distinct.
3. Source compatibility is not conflated with ABI compatibility.
4. Wire compatibility is not inferred from Rust type similarity.
5. Behavioral compatibility is not inferred solely from schema diff.
6. Inconclusive remains first-class.
7. Rust public API breaks are detected.
8. Rust ABI is not promised by default.
9. Stable C ABI analysis is platform/toolchain aware.
10. CLI machine contracts include flags, exit codes, and structured output.
11. REST/OpenAPI required-field/path/method breaks are detected.
12. Protobuf field-number reuse is rejected.
13. Postcard protocol changes require explicit version/schema strategy.
14. Daemon/agent rolling-upgrade matrix is explicit.
15. DB migrations model mixed-version read/write compatibility.
16. Destructive migrations require explicit migration/recovery policy.
17. Config meaning changes are surfaced.
18. Plugin protocol compatibility is explicit.
19. Persisted authoritative state has supported migration paths.
20. Unsupported direct version leaps are blocked.
21. Security minimum version overrides ordinary compatibility promises.
22. Consumer impact confidence is explicit.
23. Public contracts assume unknown consumers.
24. Breaking exceptions do not rewrite findings.
25. Deprecation removal follows configured lifecycle.
26. Breaking approvals bind exact report/candidate.
27. Merge queue evaluates compatibility of exact integration result where required.
28. Release/update gates consume exact compatibility evidence.
29. Mixed-version test evidence can be required.
30. Compatibility reports are retained/versioned.
31. Standalone/distributed share compatibility semantics.
32. Air-gap installations can evaluate bundled compatibility data.
33. Federation refuses unsupported authority/version pairs.
34. Forgeyard update planner consults migration/version graph.
35. Forgeyard dogfoods compatibility governance for its own CLI, Rust crates, APIs, protocols, DB, config, plugins, and update system.

---

# 319. Production Readiness Gates

Do not call compatibility governance production-ready until:

```text
contract extraction is deterministic
baseline identity is immutable
Rust API/CLI dogfood gates work
daemon-agent N/N-1 protocol tests pass
DB/config mixed-version tests exist
breaking exceptions are audited
consumer confidence is explicit
merge/release/update integration works
security minimum version overrides compatibility
fuzz/scale/DR tests pass
```

---

# 320. Architectural Invariants

1. compatibility always names exact baseline;
2. compatibility always names exact candidate;
3. compatibility dimensions remain distinct;
4. source compatibility does not imply binary compatibility;
5. schema compatibility does not imply behavioral compatibility;
6. Rust ABI is not stable by default;
7. wire compatibility requires explicit strategy;
8. database evolution includes mixed-version semantics;
9. config evolution cannot silently change meaning;
10. breaking changes remain explicit findings;
11. policy exceptions do not rewrite technical truth;
12. deprecations are versioned/lifecycle-governed;
13. consumer impact includes confidence/coverage;
14. public consumers are assumed unknown;
15. security minimum versions override normal compatibility windows;
16. migration paths are explicit;
17. unsupported upgrade leaps are blocked;
18. merge/release/update gates bind exact compatibility reports;
19. mixed-version tests can be required evidence;
20. persisted authoritative state must migrate;
21. cache may be discarded rather than migrated when safe;
22. plugin compatibility is protocol-based, not Rust ABI-based;
23. standards adapters use their own conformance semantics;
24. contract extraction/versioning is reproducible;
25. air-gap evaluation is possible from bundled artifacts;
26. federation consults compatibility before authority/site join;
27. AI may summarize but never override compatibility result;
28. audit captures breaking exceptions/policy changes;
29. compatibility remains evidence, not vague labels;
30. Forgeyard dogfoods its own compatibility system.

---

# 321. Final Target Architecture

```text
                  Previous Contract
                         │
                         ▼
                  Baseline ContractId
                         │
                         │
Proposed Change ────────┼──────── Candidate ContractId
                         │
                         ▼
                Compatibility Analysis
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
          API/ABI       Wire       Schema/Config
            │            │            │
            └────────────┼────────────┘
                         ▼
                  Consumer Impact
                         │
                         ▼
                 Policy / Migration
                         │
                         ▼
                 Merge / Release Gate
```

Protocol evolution:

```text
Protocol N
+
Protocol N-1
  ↓
explicit compatibility matrix
  ↓
mixed-version tests
  ↓
rolling upgrade support
```

Database evolution:

```text
old schema
  ↓
expand
  ↓
old + new application coexist
  ↓
backfill / switch
  ↓
contract
  ↓
old version no longer supported
```

Breaking change:

```text
breaking finding
  ↓
migration/deprecation plan
  ↓
consumer impact
  ↓
explicit policy approval
  ↓
version/release boundary
```

The key guarantee is:

> **Forgeyard can evolve rapidly without treating compatibility as guesswork. Every compatibility claim is tied to an exact contract baseline, a precise compatibility dimension, concrete evidence, known/unknown consumers, and an explicit migration or versioning policy—so breaking changes are intentional rather than accidental.**

---

# 322. Extended Architecture Sequence

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
```
