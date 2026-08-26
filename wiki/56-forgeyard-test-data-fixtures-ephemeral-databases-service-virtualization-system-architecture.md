# 56 — Forgeyard Test Data, Fixtures, Ephemeral Databases, Service Virtualization & Integration-Test Environment System Architecture

**Document type:** Core Test Data, Fixtures, Ephemeral Databases, Service Virtualization, Integration-Test Environment & Test-State Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** deterministic fixtures, seeded databases, synthetic datasets, masked production-derived datasets, ephemeral services, dependency emulators, service virtualization, test environment composition, data snapshots, reset/cleanup, TTL, privacy controls, test-state isolation, data lineage, integration-test orchestration, and reusable test-environment definitions  
**Architecture style:** Immutable test-data identities, disposable environments, deterministic initialization, explicit provenance, privacy-first handling, service virtualization, isolated state, declarative dependencies, cleanup by ownership proof, and no hidden shared mutable state  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Test Results, Sandbox/Executor, Developer Experience, Infrastructure-as-Code/Preview Environments, Secrets, Data Lifecycle, Security, CAS, Pipeline IR, Device Lab, Deployment, Dependency Governance, and Failure Diagnosis. This subsystem standardizes the data and environment layer required by reliable integration/E2E testing.

---

# 1. Purpose

Reliable tests need more than code and executors.

They often depend on:

```text
database state
message queues
object stores
mock APIs
third-party services
test accounts
seed data
generated files
emulators
temporary credentials
device state
```

Without explicit architecture, test pipelines drift toward:

```text
shared staging databases
manual seed scripts
mutable test accounts
production data copies
long-lived credentials
non-deterministic service state
```

This causes:

```text
flaky tests
privacy risk
hard-to-reproduce failures
cross-test contamination
race conditions
hidden dependencies
slow cleanup
environment drift
```

The central rule is:

> **A test should depend on explicit, versioned, reproducible test-state inputs rather than undocumented shared mutable environments.**

A second rule is:

> **Production-derived data is forbidden by default. If a policy permits derived datasets, they must be minimized, masked, provenance-tracked, access-controlled, and lifecycle-governed before they become test inputs.**

A third rule is:

> **Service virtualization and emulation are test tools, not substitutes for explicitly declared real-environment validation where policy requires it.**

---

# 2. Architectural Position

```text
                  Test Definition
                        │
                        ▼
                 Test EnvironmentSpec
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
         Fixtures     Services     Databases
            │           │           │
            └───────────┼───────────┘
                        ▼
                Provision / Seed
                        │
                        ▼
                 Isolated Test Run
                        │
                        ▼
                Evidence / Artifacts
                        │
                        ▼
                  Reset / Destroy
```

---

# 3. Goals

The subsystem MUST:

1. define test-data identities;
2. define fixture identities;
3. define test-environment identities;
4. support deterministic seeding;
5. support ephemeral databases;
6. support local database snapshots;
7. support synthetic data generation;
8. support masked data where policy allows;
9. support service virtualization;
10. support dependency emulators;
11. support disposable integration environments;
12. support environment templates;
13. support parallel-test isolation;
14. support reset/cleanup;
15. support TTL;
16. support test accounts;
17. support test secrets;
18. support fixture provenance;
19. support schema-version compatibility;
20. support test-data lineage;
21. support privacy classification;
22. support policy gates;
23. support reproducible local development;
24. support remote runners;
25. support devices;
26. support API/UI/CLI;
27. support observability;
28. support lifecycle governance;
29. support failure reproduction;
30. eliminate hidden mutable test state.

---

# 4. Non-Goals

This subsystem does not:

```text
replace production databases
replace application migration systems
replace IaC
replace integration-test frameworks
replace device-lab management
replace external third-party systems
```

---

# 5. Workspace Structure

```text
crates/testenv/
├── forgeyard-testenv/
├── forgeyard-testenv-model/
├── forgeyard-testenv-fixture/
├── forgeyard-testenv-data/
├── forgeyard-testenv-service/
├── forgeyard-testenv-database/
├── forgeyard-testenv-virtualization/
├── forgeyard-testenv-seed/
├── forgeyard-testenv-reset/
├── forgeyard-testenv-reconcile/
├── forgeyard-testenv-health/
└── forgeyard-testenv-testkit/
```

Optional adapters:

```text
crates/testenv-adapters/
├── forgeyard-testenv-postgres/
├── forgeyard-testenv-stoolap/
├── forgeyard-testenv-redis-compatible/
├── forgeyard-testenv-kafka-compatible/
├── forgeyard-testenv-s3-compatible/
├── forgeyard-testenv-http-mock/
├── forgeyard-testenv-kubernetes/
└── forgeyard-testenv-custom/
```

---

# 6. TestEnvironmentId

```rust
pub struct TestEnvironmentId(Ulid);
```

One provisioned logical test environment.

---

# 7. TestEnvironmentSpecId

```rust
pub struct TestEnvironmentSpecId(Digest);
```

Immutable identity of desired environment.

---

# 8. Test Environment Spec

```rust
pub struct TestEnvironmentSpec {
    pub id: TestEnvironmentSpecId,
    pub services: Vec<TestServiceSpec>,
    pub databases: Vec<TestDatabaseSpec>,
    pub fixtures: Vec<FixtureRef>,
    pub network: TestNetworkPolicy,
    pub ttl: Duration,
}
```

---

# 9. Environment Scope

```rust
pub enum TestEnvironmentScope {
    TestCase,
    TestSuite,
    Job,
    Run,
    SharedReadOnly,
}
```

---

# 10. Default

Prefer:

```text
TestSuite
or
Job
```

depending cost/performance.

---

# 11. Shared Mutable Environment

Not baseline.

---

# 12. Hidden Shared State

Architecture violation for correctness-critical tests.

---

# 13. FixtureSetId

```rust
pub struct FixtureSetId(Digest);
```

---

# 14. Fixture Set

```rust
pub struct FixtureSet {
    pub id: FixtureSetId,
    pub schema: FixtureSchemaVersion,
    pub objects: Vec<CasObjectRef>,
    pub generator: Option<FixtureGeneratorRef>,
}
```

---

# 15. Fixture Identity

Includes:

```text
schema version
fixture content
generator version
seed
normalization version
```

---

# 16. Deterministic Generator

Preferred.

---

# 17. Generator Seed

```rust
pub struct FixtureSeed(u128);
```

---

# 18. Random Test Data

Must record seed if reproducibility matters.

---

# 19. No Unrecorded Randomness

Critical.

---

# 20. Fixture Sources

```rust
pub enum FixtureSource {
    Static,
    Generated,
    DerivedSynthetic,
    MaskedProductionDerived,
}
```

---

# 21. Static

Checked-in or CAS-backed.

---

# 22. Generated

Created from deterministic generator.

---

# 23. DerivedSynthetic

Generated to resemble production shape without real records.

---

# 24. MaskedProductionDerived

High-risk, policy-controlled.

---

# 25. Synthetic-First Principle

Prefer synthetic data.

---

# 26. Production-Derived Data

Not default.

---

# 27. ProductionDataPolicy

```rust
pub enum ProductionDataPolicy {
    Forbidden,
    MaskedOnly,
    ApprovedSnapshotOnly,
}
```

---

# 28. Masked Dataset

Must have:

```text
source authorization
masking transform identity
privacy scan
review
retention
```

---

# 29. MaskingTransformId

```rust
pub struct MaskingTransformId(Digest);
```

---

# 30. Masking Transform

Versioned.

---

# 31. Example Transformations

```text
email substitution
name substitution
phone tokenization
address generalization
date shifting
ID remapping
free-text removal
```

---

# 32. Irreversible Where Possible

Preferred.

---

# 33. Free Text

High-risk because it may contain arbitrary PII/secrets.

---

# 34. Default

Drop/redact unless specifically needed.

---

# 35. TestDatasetId

```rust
pub struct TestDatasetId(Digest);
```

---

# 36. Test Dataset

```rust
pub struct TestDataset {
    pub id: TestDatasetId,
    pub source: FixtureSource,
    pub sensitivity: DataSensitivity,
    pub lineage: TestDataLineage,
    pub artifacts: Vec<CasObjectRef>,
}
```

---

# 37. Lineage

```rust
pub struct TestDataLineage {
    pub source_ref: Option<DataSourceRef>,
    pub transform: Option<MaskingTransformId>,
    pub created_at: Timestamp,
}
```

---

# 38. Privacy

Part 46 governs retention/deletion.

---

# 39. Access

Restricted by tenant/project.

---

# 40. Cross-Tenant Dataset Reuse

Forbidden by default.

---

# 41. Public Synthetic Fixture

Can be shared where declared.

---

# 42. Database Spec

```rust
pub struct TestDatabaseSpec {
    pub name: TestDatabaseName,
    pub engine: TestDatabaseEngine,
    pub schema: DatabaseSchemaRef,
    pub dataset: Option<TestDatasetId>,
    pub isolation: DatabaseIsolationMode,
}
```

---

# 43. DatabaseIsolationMode

```rust
pub enum DatabaseIsolationMode {
    DedicatedProcess,
    DedicatedDatabase,
    DedicatedSchema,
    TransactionRollback,
    SnapshotClone,
}
```

---

# 44. Selection

Depends on engine/test semantics.

---

# 45. Dedicated Process

Strong isolation, higher cost.

---

# 46. Dedicated Database

Good balance.

---

# 47. Dedicated Schema

Faster but weaker.

---

# 48. Transaction Rollback

Only for tests compatible with one transaction/session model.

---

# 49. Snapshot Clone

Fast for databases supporting efficient snapshots.

---

# 50. Schema Identity

```rust
pub struct DatabaseSchemaRef {
    pub migration_set: MigrationSetId,
    pub schema_digest: Digest,
}
```

---

# 51. Application DB Migrations

Run explicitly.

---

# 52. No "whatever schema is currently on shared staging"

Critical.

---

# 53. Seed Flow

```text
create DB
  ↓
apply exact migrations
  ↓
load fixture dataset
  ↓
verify seed
  ↓
mark Ready
```

---

# 54. Seed Verification

Check:

```text
schema digest
fixture count
optional checksums
required records
```

---

# 55. SeedState

```rust
pub enum SeedState {
    Pending,
    Seeding,
    Verified,
    Failed,
}
```

---

# 56. Seed Failure

Environment never Ready.

---

# 57. Service Spec

```rust
pub struct TestServiceSpec {
    pub name: TestServiceName,
    pub implementation: TestServiceImplementation,
    pub config: TestServiceConfig,
}
```

---

# 58. Service Implementation

```rust
pub enum TestServiceImplementation {
    RealContainer(ImageDigest),
    ForgeyardProcess(ToolchainArtifactRef),
    Emulator(EmulatorRef),
    VirtualService(ServiceVirtualizationRef),
    ExternalSandbox(ExternalSandboxRef),
}
```

---

# 59. Real Container

Exact image digest.

---

# 60. No Mutable `latest`

Critical.

---

# 61. Emulator

Examples:

```text
S3 emulator
SMTP sink
payment sandbox adapter
queue emulator
identity mock
```

---

# 62. Virtual Service

Protocol-level deterministic behavior.

---

# 63. ServiceVirtualizationRef

```rust
pub struct ServiceVirtualizationRef {
    pub contract: ServiceContractRef,
    pub behavior: VirtualBehaviorRef,
}
```

---

# 64. Service Contract

Could be:

```text
OpenAPI
protobuf
AsyncAPI
custom
```

---

# 65. Contract-Bound Virtualization

Preferred.

---

# 66. Virtual Behavior

Explicit scenarios.

---

# 67. Example

```text
GET /users/1 -> 200
POST /charge -> 402
third request -> timeout
```

---

# 68. Fault Injection

First-class.

---

# 69. TestFaultSpec

```rust
pub enum TestFaultSpec {
    Latency(Duration),
    Timeout,
    Disconnect,
    HttpStatus(u16),
    CorruptPayload,
    RateLimit,
}
```

---

# 70. Fault Injection

Used only in test environment.

---

# 71. No Production Target Accident

Critical.

---

# 72. Target Verification

Fault-injection runner validates environment class.

---

# 73. Service Virtualization Limit

Passing against mock does not prove real-service interoperability.

---

# 74. Policy

Can require:

```text
virtualized tests
+
real sandbox integration test
```

---

# 75. External Sandbox

Examples:

```text
payment provider sandbox
SCM test organization
cloud dev account
```

---

# 76. External Sandbox Credentials

SecretRef.

---

# 77. Test Account Identity

```rust
pub struct TestAccountId(Ulid);
```

---

# 78. Test Account Lease

```rust
pub struct TestAccountLease {
    pub account: TestAccountId,
    pub owner: JobAttemptId,
    pub expires_at: Timestamp,
}
```

---

# 79. Shared External Accounts

Need lease/reservation.

---

# 80. Reset

After use.

---

# 81. Test Account Secrets

Short-lived where possible.

---

# 82. No Production Account Reuse

Critical.

---

# 83. Test Network Policy

```rust
pub struct TestNetworkPolicy {
    pub allowed_services: Vec<TestServiceName>,
    pub external_access: ExternalNetworkAccess,
}
```

---

# 84. ExternalNetworkAccess

```rust
pub enum ExternalNetworkAccess {
    Deny,
    Allowlisted,
    Full,
}
```

---

# 85. Default Integration Test

Allow only declared dependencies where enforceable.

---

# 86. Hermetic Test

ExternalNetworkAccess::Deny.

---

# 87. Real Sandbox Test

Allowlisted.

---

# 88. Full Internet

Rare/high-risk.

---

# 89. Environment Provisioning

Can reuse Part 53 infrastructure primitives.

---

# 90. Lightweight Local

Process/container sandbox.

---

# 91. Heavy Integration

Kubernetes/VM/preview environment.

---

# 92. Test Environment Provider

```rust
#[async_trait]
pub trait TestEnvironmentProvider {
    async fn provision(
        &self,
        request: TestEnvironmentProvisionRequest,
    ) -> Result<TestEnvironmentHandle, TestEnvironmentError>;

    async fn reset(
        &self,
        handle: &TestEnvironmentHandle,
    ) -> Result<(), TestEnvironmentError>;

    async fn destroy(
        &self,
        handle: &TestEnvironmentHandle,
    ) -> Result<(), TestEnvironmentError>;
}
```

---

# 93. Provision Intent

Persist before external effects.

---

# 94. ProvisionState

```rust
pub enum TestEnvironmentState {
    Requested,
    Provisioning,
    Seeding,
    Ready,
    InUse,
    Resetting,
    Expired,
    Destroying,
    Destroyed,
    Failed,
    Unknown,
}
```

---

# 95. Unknown Provider Outcome

Inspect before retry.

---

# 96. No Blind Duplicate Environment Creation

Critical.

---

# 97. Ownership Proof

Every created external resource records:

```text
TestEnvironmentId
TenantId
ProjectId
RunId/JobId
```

as metadata/tags where possible.

---

# 98. Tags Not Sole Proof

Use provider IDs + persisted state.

---

# 99. TTL

Mandatory for ephemeral external environments.

---

# 100. TestEnvironmentLease

```rust
pub struct TestEnvironmentLease {
    pub environment: TestEnvironmentId,
    pub holder: JobAttemptId,
    pub expires_at: Timestamp,
}
```

---

# 101. Lease Expiry

Does not kill active job blindly.

---

# 102. Coordinator

Checks active ownership.

---

# 103. Cleanup Intent

First-class.

---

# 104. Cleanup Workflow

```text
job complete/cancel
  ↓
revoke test credentials
  ↓
collect required evidence
  ↓
reset/destroy
  ↓
verify
```

---

# 105. Failed Cleanup

Tracked.

---

# 106. Cleanup Backlog

Visible.

---

# 107. Orphan Detection

Periodic.

---

# 108. Unknown Ownership

Manual/quarantine.

---

# 109. No Blind Orphan Delete

Critical.

---

# 110. Environment Reuse

Possible.

---

# 111. Reuse Classes

```rust
pub enum TestEnvironmentReuse {
    Never,
    ResetBetweenSuites,
    ReadOnlyShared,
    Pool,
}
```

---

# 112. Never

Strongest reproducibility.

---

# 113. ResetBetweenSuites

Useful for expensive DB/service sets.

---

# 114. Pool

Pre-warmed isolated environments.

---

# 115. Pool Correctness

Environment must pass reset verification before reissue.

---

# 116. ResetVerification

```rust
pub struct ResetVerification {
    pub schema: Digest,
    pub dataset: TestDatasetId,
    pub service_states: Vec<ServiceStateDigest>,
}
```

---

# 117. Failed Reset

Destroy environment.

---

# 118. No "probably clean" reuse

Critical.

---

# 119. Parallel Test Isolation

Use unique:

```text
database/schema
queue namespace
bucket prefix
tenant/test account
port/network namespace
```

---

# 120. NamespaceId

Derived from:

```text
RunId/JobAttemptId/TestSuite shard
```

---

# 121. Collision

Forbidden.

---

# 122. Test Sharding

Part 32.

Each shard gets isolation metadata.

---

# 123. Shared Read-Only Fixture

Allowed.

---

# 124. Shared Mutable Fixture

Avoid.

---

# 125. Clock

Tests may need controlled time.

---

# 126. TestClockMode

```rust
pub enum TestClockMode {
    Real,
    Fixed(Timestamp),
    Virtual,
}
```

---

# 127. Virtual Time

Only if application/testing framework supports.

---

# 128. Time Zone

Explicit.

---

# 129. Locale

Explicit.

---

# 130. Random Seed

Explicit.

---

# 131. Reproduction Context

Records:

```text
FixtureSetId
TestDatasetId
TestEnvironmentSpecId
seed
clock
service versions
```

---

# 132. Failure Diagnosis

Part 48 can recreate exact test state.

---

# 133. ReproductionBundle

Can include non-secret fixture closure.

---

# 134. Database Snapshot

CAS reference where safe.

---

# 135. Sensitive Snapshot

Restricted CAS class.

---

# 136. Snapshot Encryption

Required by sensitivity.

---

# 137. Snapshot Clone

Fast environment creation.

---

# 138. Snapshot Identity

```rust
pub struct TestDatabaseSnapshotId(Digest);
```

---

# 139. Snapshot Includes

```text
engine version
schema digest
dataset identity
normalization
```

---

# 140. Engine Compatibility

Must validate.

---

# 141. No Snapshot Into Incompatible DB Version Silently

---

# 142. Migration Testing

Important use case.

---

# 143. Database Migration Test

Flow:

```text
old schema snapshot
  ↓
apply candidate migration
  ↓
validate data/schema
  ↓
optionally rollback simulation
```

---

# 144. Production-Like Scale

Synthetic dataset can be large.

---

# 145. Scale Profile

```rust
pub enum TestDataScale {
    Tiny,
    Small,
    Representative,
    Stress,
    Custom(u64),
}
```

---

# 146. Representative

Shape/volume approximate, not real production data.

---

# 147. Benchmark Isolation

Part 33 may use stable seeded datasets.

---

# 148. Benchmark Fixture

Immutable.

---

# 149. Performance Comparability

Dataset identity must match baseline.

---

# 150. Contract Testing

Provider/consumer contract fixtures.

---

# 151. ContractFixtureId

```rust
pub struct ContractFixtureId(Digest);
```

---

# 152. Consumer-Driven Contract

Can generate virtual service behavior.

---

# 153. Contract Evidence

Part 32/37 integration.

---

# 154. API Schema Drift

Virtualization spec versioned.

---

# 155. Message Queue Fixtures

Need unique topic/queue names.

---

# 156. Event Order

Deterministic when test requires.

---

# 157. Object Store Fixtures

Unique bucket/prefix.

---

# 158. Filesystem Fixtures

CAS tree mounted read-only where possible.

---

# 159. SMTP

Use sink/mock server.

---

# 160. Email Assertions

Capture message metadata/content in test environment.

---

# 161. SMS/Push

Use fake provider baseline.

---

# 162. Payment Providers

Use provider sandbox or virtual adapter.

---

# 163. Never Real Charge

Critical.

---

# 164. Production Endpoint Guard

Explicit denylist/allowlist.

---

# 165. Test Environment Egress Guard

Can block known production domains/accounts.

---

# 166. Cloud Account Separation

Recommended.

---

# 167. `TestOnly` Provider Binding

```rust
pub struct TestProviderBinding {
    pub provider: ProviderId,
    pub environment_class: ExternalEnvironmentClass,
}
```

---

# 168. ExternalEnvironmentClass

```rust
pub enum ExternalEnvironmentClass {
    Emulator,
    Sandbox,
    TestAccount,
    Production,
}
```

---

# 169. Production

Forbidden for destructive/fault tests.

---

# 170. Test Secrets

Separate scope from production secrets.

---

# 171. Secret Policy

```text
test/*
preview/*
production/*
```

---

# 172. Secret Scope Widening

Forbidden.

---

# 173. Dynamic Test Credentials

Preferred.

---

# 174. Revocation

On cleanup.

---

# 175. Environment Definition in Pipeline

Pipeline IR references `TestEnvironmentSpecId` or declarative spec.

---

# 176. No Arbitrary Setup Script as Sole Environment Definition

Critical.

---

# 177. Setup Hooks

Allowed but explicit, sandboxed, provenance-bound.

---

# 178. SetupHookId

```rust
pub struct SetupHookId(Digest);
```

---

# 179. Hook Inputs

Included in environment identity if correctness-affecting.

---

# 180. Environment Cache

Can reuse prebuilt image/snapshot.

---

# 181. Cache Key

Includes spec/dataset/tool versions.

---

# 182. Cache Never Hides Mutable State

Critical.

---

# 183. Golden Paths

Part 42 can define reusable test-environment templates.

---

# 184. Example

```text
standard-postgres-integration
standard-web-e2e
standard-payment-sandbox
```

---

# 185. Developer Experience

Local command:

```text
forgeyard testenv up
forgeyard testenv reset
forgeyard testenv down
```

---

# 186. Local/CI Parity

Same environment spec.

---

# 187. Local Differences

Explicit capabilities.

---

# 188. IDE Integration

Expose:

```text
connection strings
ports
service URLs
```

through ephemeral local env variables.

---

# 189. Secret Values

Not persisted to IDE config.

---

# 190. Dioxus UI

Pages:

```text
Test Environments
Fixtures
Datasets
Service Virtualization
Cleanup
```

---

# 191. Environment Detail

Shows:

```text
spec
owner
run/job
services
database state
dataset
TTL
cleanup
```

---

# 192. Dataset Detail

Shows:

```text
source class
sensitivity
lineage
masking transform
retention
```

---

# 193. Privacy Warning

Visible for production-derived data.

---

# 194. CLI

```text
forgeyard testenv list
forgeyard testenv show
forgeyard testenv up
forgeyard testenv reset
forgeyard testenv destroy
forgeyard fixture list
forgeyard fixture generate
forgeyard dataset inspect
forgeyard testenv doctor
```

---

# 195. API

Potential:

```text
GET  /v1/test-environments
POST /v1/test-environments
POST /v1/test-environments/{id}/reset
DELETE /v1/test-environments/{id}
GET  /v1/test-datasets
```

---

# 196. Permissions

```text
testenv.read
testenv.create
testenv.destroy
testdata.read
testdata.production_derived
testdata.manage
```

---

# 197. Production-Derived Access

Separate high-risk permission.

---

# 198. Audit

Audit:

```text
production-derived dataset creation
masking policy change
production-like external sandbox binding
manual environment retention
sensitive dataset export
```

---

# 199. Routine Ephemeral Creation

Operational events.

---

# 200. Data Lifecycle

Part 46.

---

# 201. Default Retention

Ephemeral.

---

# 202. Failed Test Reproduction Pin

Can extend.

---

# 203. Sensitive Data

Short retention.

---

# 204. Test Logs

Redacted.

---

# 205. Legal/Security Hold

Can retain required evidence.

---

# 206. Cost

Part 45.

Meter:

```text
ephemeral DB time
cluster time
storage
external sandbox usage
```

---

# 207. Cost Guardrails

Optional environments.

---

# 208. Do Not Share Mutable State Merely to Save Cost

Critical.

---

# 209. Reliability

Part 50 can track:

```text
testenv provisioning success
seed latency
cleanup success
orphan count
```

---

# 210. Observability Metrics

```text
testenv_active_total
testenv_provision_total
testenv_seed_failures_total
testenv_cleanup_failures_total
testenv_orphans_total
testdata_sensitive_datasets_total
```

---

# 211. Labels

Low-cardinality.

---

# 212. Tracing

```text
testenv.provision
testenv.seed
testenv.lease
testenv.reset
testenv.destroy
testdata.generate
testdata.mask
```

---

# 213. Health

Checks:

```text
provider availability
cleanup backlog
stale leases
dataset registry
masking pipeline
```

---

# 214. Doctor

```text
forgeyard testenv doctor
```

Checks:

```text
orphan environments
expired leases
production endpoint bindings
unmasked sensitive data
reset failures
snapshot compatibility
```

---

# 215. Security Threats

```text
production data leakage
production endpoint misuse
cross-test contamination
secret reuse
orphan resources
malicious fixture
archive traversal
resource exhaustion
```

---

# 216. Fixture Parser Safety

Bound sizes.

---

# 217. Database Import

Sandbox parsing/load.

---

# 218. SQL Seed

Potentially dangerous.

---

# 219. Run only against isolated test DB.

---

# 220. No host/system DB connection from seed worker.

Critical.

---

# 221. Data Exfiltration

Test environment egress restrictions.

---

# 222. Cross-Tenant Isolation

Every environment/dataset tenant scoped.

---

# 223. Dataset Export

Permission controlled.

---

# 224. Device Testing

Part 20.

---

# 225. Device State Fixture

Examples:

```text
app data
account state
media files
network profile
```

---

# 226. Device Reset

Before/after test.

---

# 227. Device Fixture Identity

Explicit.

---

# 228. No Personal Device Production Data

Critical.

---

# 229. Mobile Backend Test

Can combine device lease + ephemeral backend.

---

# 230. E2E Environment

Composition:

```text
device
backend
database
virtual third-party
test account
```

---

# 231. Environment Composition Graph

Typed DAG.

---

# 232. Provision Order

Dependencies.

---

# 233. Destroy Order

Reverse dependencies where safe.

---

# 234. Partial Provision Failure

Destroy known-owned created resources.

---

# 235. Unknown Resource Outcome

Inspect provider.

---

# 236. No Blind Cleanup

Existing invariant.

---

# 237. Federation

Part 51.

Test environment should run in allowed region/site.

---

# 238. Residency

Sensitive test data stays allowed region.

---

# 239. Air-Gap

Synthetic/static fixtures ideal.

---

# 240. External Sandbox

Unavailable offline.

---

# 241. Offline Environment

Use local emulators.

---

# 242. Test Dataset Replication

Only by policy.

---

# 243. Sensitive Dataset

May be non-replicable.

---

# 244. CAS Encryption

Where required.

---

# 245. Snapshot Replication

Digest verified.

---

# 246. Infrastructure Integration

Part 53 handles heavy resource provisioning.

---

# 247. Division

TestEnv decides:

```text
what test dependencies are needed
```

IaC decides:

```text
how heavy infrastructure is provisioned
```

---

# 248. Deployment Integration

Test environments can receive exact built artifacts.

---

# 249. No Rebuild

Existing invariant.

---

# 250. Preview Environment Difference

Preview environment is human-reviewable application environment.

Test environment is primarily automated isolated test state.

---

# 251. They can share infrastructure components.

---

# 252. Failure Diagnosis

Part 48 records:

```text
environment spec
dataset
fixture seed
service versions
```

---

# 253. Exact Reproduction

Recreate.

---

# 254. If external sandbox state unavailable

Fidelity downgraded.

---

# 255. ReproductionFidelity

Existing Part 48.

---

# 256. Test Result Integration

Part 32 observation references:

```text
TestEnvironmentId
FixtureSetId
TestDatasetId
```

---

# 257. Flake Intelligence

Can correlate failures with environment/dataset.

---

# 258. Environment-Specific Flake

First-class.

---

# 259. Static Analysis

Fixture files can be scanned for secrets.

---

# 260. Secret Scanner

Part 37.

---

# 261. Dataset Privacy Scan

Separate classifier.

---

# 262. Masking Verification

Run checks such as:

```text
known identifier absence
uniqueness changes
pattern detection
free-text scan
```

---

# 263. No Claim Perfect De-identification

Critical.

---

# 264. Production-Derived Dataset Approval

Explicit.

---

# 265. Dataset State

```rust
pub enum TestDatasetState {
    Generated,
    AwaitingReview,
    Approved,
    Restricted,
    Revoked,
    Expired,
}
```

---

# 266. Revoked Dataset

Cannot be used in new tests.

---

# 267. Existing Evidence

Historical references remain.

---

# 268. Fixture Versioning

Immutable.

---

# 269. `latest fixture`

Human selector only.

---

# 270. Pipeline Plan

Resolves selector to exact FixtureSetId.

---

# 271. Environment Plan Identity

Exact fixture IDs included.

---

# 272. Schema Drift

If app migration changes, old fixtures may be incompatible.

---

# 273. FixtureCompatibility

```rust
pub enum FixtureCompatibility {
    Compatible,
    MigrationRequired,
    Incompatible,
    Unknown,
}
```

---

# 274. No Silent Auto-Fix of Fixture

Critical.

---

# 275. Fixture Migration

Explicit tool.

---

# 276. Synthetic Generator Version

Pinned.

---

# 277. Generator Upgrade

Creates new dataset.

---

# 278. Deterministic Synthetic Data

Same generator+seed -> same canonical dataset when guaranteed.

---

# 279. If generator non-deterministic

Record output digest and mark reproducibility lower.

---

# 280. High-Volume Data

Generate streaming/chunked.

---

# 281. CAS

Store canonical data artifacts if reusable.

---

# 282. Large Data

May use external object storage with CAS manifest.

---

# 283. Data Compression

Allowed.

---

# 284. Digest of canonical content/manifest.

---

# 285. Test Environment State Machine

```text
Requested
  ↓
Provisioning
  ↓
Seeding
  ↓
Ready
  ↓
InUse
  ↓
Resetting / Expired
  ↓
Destroying
  ↓
Destroyed
```

Failure paths go to:

```text
Failed
Unknown
```

then reconciliation.

---

# 286. Environment Reconciler

Checks:

```text
provider state
lease
owner run/job
TTL
cleanup
dataset validity
```

---

# 287. HA

Multiple workers claim environments safely.

---

# 288. No Raft Requirement

Normal DB leases.

---

# 289. Standalone Mode

Can use:

```text
local processes
containers
Stoolap
local Postgres
emulators
```

---

# 290. Distributed Mode

Can use:

```text
remote runners
Kubernetes
cloud DB
device lab
external sandbox
```

---

# 291. Developer Local Mode

Same fixture/environment descriptors.

---

# 292. Local Service Ports

Dynamically allocated.

---

# 293. Collision Avoidance

Port reservation.

---

# 294. Docker/Podman

Adapter choice, not core assumption.

---

# 295. Process Sandbox

Pure native option where available.

---

# 296. Container Image

Exact digest.

---

# 297. Kubernetes Pod

Exact image digest.

---

# 298. Test Environment Cache Pool

Optional performance optimization.

---

# 299. Prewarm

Allowed.

---

# 300. Prewarm State

Must reset to canonical snapshot.

---

# 301. Warm Pool Cost

Part 45.

---

# 302. Pool Health

Failed reset destroys member.

---

# 303. Testkit

```text
forgeyard-testenv-testkit/src/
├── lib.rs
├── fixture.rs
├── dataset.rs
├── database.rs
├── service.rs
├── provision.rs
├── reset.rs
├── cleanup.rs
└── assertions.rs
```

---

# 304. Unit Tests

Fixture identity determinism.

---

# 305. Random Seed Test

Recorded seed reproduces dataset.

---

# 306. Schema Test

Exact migration set applied.

---

# 307. Shared State Test

Parallel suites isolated.

---

# 308. Reset Test

Canonical state restored.

---

# 309. Failed Reset Test

Environment destroyed.

---

# 310. TTL Test

Expired env cleaned.

---

# 311. Active Lease Test

Not prematurely destroyed.

---

# 312. Ownership Test

Unknown resource not deleted.

---

# 313. Production Endpoint Test

Destructive/fault test blocked.

---

# 314. Secret Scope Test

Production secret unavailable.

---

# 315. Dataset Privacy Test

Restricted data cannot cross tenant/region.

---

# 316. Masking Test

Transform version/provenance recorded.

---

# 317. Revoked Dataset Test

New run denied.

---

# 318. Virtualization Test

Contract behavior deterministic.

---

# 319. Fault Injection Test

Only test target affected.

---

# 320. Snapshot Compatibility Test

DB version mismatch detected.

---

# 321. Migration Test

Old snapshot -> migration candidate validated.

---

# 322. Flake Correlation Test

Environment identity attached to failure.

---

# 323. Federation Test

Residency enforced.

---

# 324. Air-Gap Test

Local fixtures/emulators work offline.

---

# 325. DR Test

Dataset metadata restores; ephemeral envs reconcile/destroy.

---

# 326. Fuzzing

Fuzz:

```text
fixture parsers
dataset manifests
virtual service definitions
seed metadata
```

---

# 327. Adversarial Tests

```text
archive traversal
SQL seed attempts host connection
malicious fixture
secret-bearing logs
production endpoint spoof
```

---

# 328. Scale Test

Thousands of parallel ephemeral environments.

---

# 329. Chaos Tests

```text
DB startup failure
seed failure
provider timeout
cleanup worker crash
external sandbox outage
```

---

# 330. Implementation Phase 1 — Fixture/Dataset Model

Core identities.

---

# 331. Phase 2 — Local Database/Service Environments

Standalone/dev.

---

# 332. Phase 3 — Seed/Reset/Reproduction

Correctness.

---

# 333. Phase 4 — Virtual Services/Emulators

Integration testing.

---

# 334. Phase 5 — Kubernetes/Heavy Environments

Scale.

---

# 335. Phase 6 — Synthetic Data Generator Framework

Privacy.

---

# 336. Phase 7 — Production-Derived Masked Data Governance

Optional enterprise.

---

# 337. Phase 8 — Device/E2E Composition

Mobile/desktop.

---

# 338. Phase 9 — Warm Pools

Performance.

---

# 339. Phase 10 — Federation/Air-Gap

Distributed.

---

# 340. Phase 11 — UI/CLI/Doctor

Operations.

---

# 341. Phase 12 — Chaos/Fuzz/Privacy Hardening

Production readiness.

---

# 342. Acceptance Tests

1. Test environment state is explicit and versioned.
2. Fixture sets have immutable identities.
3. Correctness-sensitive randomness records seeds.
4. Test DB schema binds exact migration/schema identity.
5. Shared mutable staging state is not baseline correctness input.
6. Parallel test suites receive isolated state.
7. Reset/reuse requires verification.
8. Failed reset removes environment from reuse pool.
9. Synthetic data is preferred over production-derived data.
10. Production-derived datasets require explicit policy/approval.
11. Masking transforms are versioned/provenance-tracked.
12. Forgeyard does not claim perfect anonymization from heuristic masking.
13. Cross-tenant dataset reuse is forbidden by default.
14. Sensitive datasets obey residency/lifecycle controls.
15. Production secrets are unavailable to test environments by default.
16. Production service endpoints are not used for destructive/fault tests.
17. External sandbox accounts are leased and reset.
18. Service virtualization is contract/version bound.
19. Passing virtual-service tests does not replace mandatory real integration tests.
20. Test environment provision effects are reconciled after ambiguity.
21. TTL creates cleanup intent rather than blind resource deletion.
22. Unknown/unowned resources are never automatically destroyed.
23. Exact environment/fixture identities are attached to test observations.
24. Failure reproduction can recreate fixtures/environment when inputs remain available.
25. DB snapshots validate engine/schema compatibility.
26. Benchmark comparisons bind exact dataset identity.
27. Cache/prewarm never hides mutable leftover state.
28. Heavy environment provisioning integrates with Part 53.
29. Device tests can compose device lease + backend + data + virtual services.
30. Air-gapped testing can operate with local fixtures/emulators.
31. Standalone/distributed share environment semantics.
32. Sensitive test data is encrypted/restricted.
33. Cleanup backlog/orphans are observable.
34. Environment workers are restart-safe/reconciled.
35. Forgeyard dogfoods test environments for its own integration/E2E test suites.

---

# 343. Production Readiness Gates

Do not call test-environment architecture production-ready until:

```text
fixture/dataset identities are stable
parallel isolation tests pass
seed/reset determinism is verified
cleanup/TTL reconciliation is safe
production endpoint/secret guards work
sensitive dataset policy is enforced
service virtualization is versioned
failure reproduction captures environment identity
heavy-provider timeout handling is safe
privacy/adversarial/chaos tests pass
```

---

# 344. Architectural Invariants

1. test state is explicit, not hidden;
2. fixtures are immutable/versioned;
3. randomness affecting correctness is recorded;
4. test schema is exact;
5. shared mutable state is not baseline truth;
6. parallel tests are isolated;
7. reuse requires verified reset;
8. synthetic data is preferred;
9. production-derived data is high-risk/optional;
10. masking is versioned and not claimed perfect;
11. sensitive data obeys tenant/residency/lifecycle;
12. production secrets are not test secrets;
13. production endpoints are forbidden for destructive tests;
14. external test accounts are leased/reset;
15. virtual services do not replace required real integration;
16. provision/destroy effects are reconciled;
17. TTL does not authorize blind deletion;
18. ownership proof is required for cleanup;
19. environment identity is attached to evidence;
20. failure reproduction reuses exact environment inputs;
21. database snapshot compatibility is explicit;
22. benchmark dataset identity is exact;
23. caches/pools never preserve hidden mutable state;
24. heavy provisioning delegates to infrastructure subsystem;
25. device/E2E environments compose explicitly;
26. air-gap operation can use local fixtures/emulators;
27. environment workers are idempotent/reconciled;
28. standalone/distributed share semantics;
29. privacy/security controls remain mandatory;
30. Forgeyard dogfoods its own test-environment system.

---

# 345. Final Target Architecture

```text
                     Test Definition
                           │
                           ▼
                  TestEnvironmentSpec
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
          Dataset        Database      Services
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                    Provision / Seed
                           │
                           ▼
                      Ready State
                           │
                           ▼
                       Test Run
                           │
                           ▼
                     Evidence
                           │
                           ▼
                   Reset / Destroy
```

Reproducibility:

```text
SourceSnapshotId
+
TestEnvironmentSpecId
+
FixtureSetId
+
TestDatasetId
+
seed
+
service image/tool versions
  ↓
recreatable test state
```

Privacy-safe production-derived flow:

```text
authorized source
  ↓
minimize
  ↓
versioned masking transform
  ↓
privacy validation
  ↓
restricted TestDatasetId
  ↓
short retention
```

The key guarantee is:

> **Forgeyard can make integration and E2E tests reproducible by treating data, databases, emulators, accounts, and service state as explicit test inputs rather than invisible shared infrastructure. The system prefers synthetic deterministic fixtures, isolates mutable state per test scope, and never trades privacy or production safety for test convenience.**

---

# 346. Extended Architecture Sequence

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
```
