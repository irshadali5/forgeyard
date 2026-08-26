# 65 — Forgeyard Build Graph Replay, Historical Reproducibility, Time-Travel CI & Evidence Reconstruction System Architecture

**Document type:** Core Historical Build Replay, Time-Travel CI, Evidence Reconstruction, Archived Dependency Resolution & Reproducibility Recovery System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** historical run reconstruction, source/toolchain/config snapshot replay, archived dependency resolution, historical pipeline planning, exact environment reconstruction, old runner/image compatibility, evidence-gap handling, deterministic replay, semantic replay, best-effort reconstruction, historical debug, compliance replay, incident replay, and long-term reproducibility governance  
**Architecture style:** Immutable historical identities, replay manifests, evidence-first reconstruction, explicit fidelity levels, archived inputs, compatibility-aware execution, provenance preservation, no silent substitution, and honest handling of irrecoverable historical state  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Core Domain, CAS, Pipeline IR, Reproducibility/Hermetic Packaging, Toolchains, Runner Images, Test Data, Dependency Mirrors, Compatibility Governance, Failure Diagnosis, Audit, Data Lifecycle, Release Provenance, Self-Hosting, and Incident/Postmortem systems. This subsystem enables exact or bounded reconstruction of historical execution.

---

## 1. Purpose

Over time, engineers need to answer questions such as:

```text
Can we reproduce run #123 from six months ago?
Why did this artifact differ from today’s rebuild?
Which toolchain and dependency versions were used?
Can we recreate the exact integration candidate?
Can we replay the same failing test?
Can an auditor verify how this release was produced?
Can we rerun a historical pipeline after an incident?
```

Naive CI systems often retain only:

```text
logs
commit SHA
status
```

That is insufficient.

A reliable replay needs some or all of:

```text
exact source snapshot
pipeline IR
toolchain descriptors
dependency lockfiles
resolved package digests
configuration snapshot
policy digest
test-data identity
runner baseline
environment identity
network policy
secrets references
external service observations
```

The central rule is:

> **Historical replay is possible only to the fidelity supported by retained immutable inputs and external-state evidence. Forgeyard never claims exact replay when required historical inputs are missing or mutable external systems cannot be reconstructed.**

A second rule is:

> **Replay never substitutes newer dependencies, newer toolchains, newer base images, or newer configuration silently. Every substitution is explicit and lowers replay fidelity.**

A third rule is:

> **Historical replay creates a new execution record linked to the original; it never rewrites the historical Run/Job/evidence that actually occurred.**

---

## 2. Architectural Position

```text
                    Historical Run
                         │
                         ▼
                   Replay Manifest
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
            Source    Toolchains   Config
              │          │          │
              └──────────┼──────────┘
                         ▼
                Archived Dependencies
                         │
                         ▼
                Replay Environment Plan
                         │
                         ▼
                     New Replay Run
                         │
                         ▼
                 Compare / Reconstruct
```

---

## 3. Goals

The subsystem MUST:

1. define replay identity;
2. define replay manifest identity;
3. reconstruct exact source snapshot;
4. reconstruct pipeline IR;
5. reconstruct exact toolchains;
6. reconstruct dependency resolution;
7. reconstruct relevant config;
8. reconstruct test data/environment where possible;
9. reconstruct runner baseline where possible;
10. support deterministic replay;
11. support semantic replay;
12. support best-effort replay;
13. support historical failure reproduction;
14. support release verification replay;
15. support incident replay;
16. support compliance/audit replay;
17. support evidence reconstruction;
18. support old-version executors/tooling;
19. support replay compatibility bridges;
20. support replay bundles;
21. support air-gap replay;
22. support missing-input detection;
23. support explicit substitution;
24. support fidelity scoring/classification;
25. support UI/API/CLI;
26. support retention policy;
27. support audit;
28. support security;
29. support federation/DR;
30. never mutate historical truth.

---

## 4. Non-Goals

This subsystem does not:

```text
guarantee impossible reconstruction of deleted external state
guarantee bit-for-bit output for inherently nondeterministic software
replace normal CI
replace artifact provenance
replace backup
replace archival policy
```

---

## 5. Workspace Structure

```text
crates/replay/
├── forgeyard-replay/
├── forgeyard-replay-model/
├── forgeyard-replay-manifest/
├── forgeyard-replay-source/
├── forgeyard-replay-toolchain/
├── forgeyard-replay-dependency/
├── forgeyard-replay-environment/
├── forgeyard-replay-evidence/
├── forgeyard-replay-compare/
├── forgeyard-replay-reconcile/
├── forgeyard-replay-health/
└── forgeyard-replay-testkit/
```

---

## 6. ReplayId

```rust
pub struct ReplayId(Ulid);
```

One new execution attempt intended to reconstruct an earlier execution.

---

## 7. ReplayManifestId

```rust
pub struct ReplayManifestId(Digest);
```

Immutable identity of historical replay inputs.

---

## 8. Replay Manifest

```rust
pub struct ReplayManifest {
    pub id: ReplayManifestId,
    pub original_run: RunId,
    pub source: SourceSnapshotId,
    pub pipeline: PipelineIrId,
    pub toolchains: Vec<ToolchainDescriptorId>,
    pub dependencies: Vec<ResolvedDependencySetId>,
    pub config: ConfigSnapshotId,
    pub policy: PolicyDigest,
    pub test_data: Vec<TestDatasetId>,
    pub runner_baseline: Option<RunnerBaselineId>,
}
```

---

## 9. Manifest Source

Prefer generated automatically at original run time.

---

## 10. No Forensic Guessing First

Critical.

If exact manifest exists, use it.

For old runs predating manifest support, use reconstruction mode.

---

## 11. Replay Fidelity

```rust
pub enum ReplayFidelity {
    BitForBitExpected,
    DeterministicInputs,
    EquivalentEnvironment,
    SemanticReplay,
    BestEffort,
    EvidenceOnly,
    Impossible,
}
```

---

## 12. BitForBitExpected

All required deterministic inputs retained and original workload expected reproducible.

---

## 13. DeterministicInputs

Exact declared inputs exist, but output may still contain known nondeterminism.

---

## 14. EquivalentEnvironment

Exact original VM/image bytes unavailable, but verified equivalent baseline can be reconstructed.

---

## 15. SemanticReplay

Newer compatibility bridge/tool can execute same semantic test/workflow, but not exact environment.

---

## 16. BestEffort

Substitutions/missing historical inputs exist.

---

## 17. EvidenceOnly

Cannot execute meaningfully, but can reconstruct historical evidence/provenance.

---

## 18. Impossible

Required source/data/tooling unavailable.

---

## 19. Fidelity Is Explicit

Never reduce it silently.

---

## 20. Replay Subject

```rust
pub enum ReplaySubject {
    Run(RunId),
    Job(JobId),
    JobAttempt(JobAttemptId),
    Release(ReleaseId),
    IntegrationCandidate(IntegrationCandidateId),
    FailureObservation(FailureObservationId),
    Incident(IncidentId),
}
```

---

## 21. ReplayIntent

```rust
pub enum ReplayIntent {
    Debug,
    VerifyReproducibility,
    Audit,
    IncidentAnalysis,
    SecurityAnalysis,
    Regression,
    Compliance,
}
```

---

## 22. Historical Source

Canonical source is `SourceSnapshotId`.

---

## 23. VCS Commit Availability

Not required if canonical source tree retained in CAS.

---

## 24. Source CAS Retention

Part 46 determines availability.

---

## 25. Missing Source Snapshot

Replay impossible beyond evidence-only unless restored from authorized archive.

---

## 26. No Fetching Current Branch As Substitute

Critical.

---

## 27. Pipeline IR

Replay should use original normalized `PipelineIr`.

---

## 28. Pipeline Definition Source

Can also be retained for explanation.

---

## 29. Current Parser

May not parse historical syntax identically.

Therefore replay should prefer archived IR.

---

## 30. PipelineIrVersion

```rust
pub struct PipelineIrVersion(u16);
```

---

## 31. Historical IR Compatibility

Need readers/migration bridges.

---

## 32. Do Not Recompile Old Source Config With New Parser Unless Explicit

Critical.

---

## 33. Replay Bridge

```rust
pub struct ReplayCompatibilityBridgeId(Digest);
```

Transforms historical IR/schema into executable current internal representation while preserving semantics where proven.

---

## 34. Bridge Evidence

Compatibility report required.

---

## 35. Toolchain Reconstruction

Exact `ToolchainDescriptorId`.

---

## 36. Toolchain Archive

Can include:

```text
compiler
linker
SDK
runtime
system headers
package managers
```

---

## 37. Toolchain Availability

```rust
pub enum HistoricalArtifactAvailability {
    Available,
    Rebuildable,
    SubstituteRequired,
    Missing,
}
```

---

## 38. Rebuildable

Exact derivation retained, outputs can be rebuilt.

---

## 39. Substitute Required

Lowers fidelity.

---

## 40. No "latest compatible compiler" Silent Replacement

Critical.

---

## 41. Dependency Reconstruction

Use exact locks + resolved digests.

---

## 42. ResolvedDependencySetId

```rust
pub struct ResolvedDependencySetId(Digest);
```

---

## 43. Dependency Archive

Part 36/52 mirror/registry can retain exact package blobs.

---

## 44. Package Deleted Upstream

Should not matter if archived internally.

---

## 45. External Dependency Missing

Replay fidelity reduced/impossible.

---

## 46. Source Archive

VCS URL only is insufficient.

---

## 47. Container Image

Exact digest.

---

## 48. OCI Registry Retention

Required for historical replay if image is input.

---

## 49. Base OS

Part 58 runner-image baseline.

---

## 50. Runner Baseline Reconstruction

Modes:

```text
exact image artifact
provider image copy
rebuild from image definition
equivalent baseline
unavailable
```

---

## 51. Exact Host Hardware

Often unavailable.

---

## 52. HardwareProfile

```rust
pub struct HardwareProfile {
    pub arch: Architecture,
    pub cpu_features: CpuFeatureSet,
    pub gpu: Option<GpuIdentity>,
    pub memory_class: MemoryClass,
}
```

---

## 53. Performance Replay

May require hardware equivalence.

---

## 54. Functional Replay

Can tolerate broader equivalent hardware if policy permits.

---

## 55. Benchmark Replay

Part 33 requires stronger environment matching.

---

## 56. Test Environment Replay

Part 56 identities:

```text
FixtureSetId
TestDatasetId
TestEnvironmentSpecId
random seed
```

---

## 57. External Sandbox

Historical external state may be unrecoverable.

---

## 58. Mock/Virtual Service

Can replay exactly if behavior artifact retained.

---

## 59. Production-Derived Dataset

Retention/privacy may forbid historical persistence.

---

## 60. Privacy Overrides Replay Convenience

Critical.

---

## 61. Secret Replay

Secret values are not archived in replay manifest.

---

## 62. SecretRef Only

Historical secret value may have rotated/expired.

---

## 63. Replay Secret Policy

```rust
pub enum ReplaySecretMode {
    Deny,
    CurrentEquivalent,
    HistoricalEscrow,
    Synthetic,
}
```

---

## 64. Default

`Deny` or `Synthetic` for most debug replay.

---

## 65. Historical Secret Escrow

High-risk optional enterprise capability.

Not baseline.

---

## 66. Never Store Secret Values In CAS Replay Bundle

Critical.

---

## 67. CurrentEquivalent

Means use present credential with equivalent permission, not original secret value.

---

## 68. This Lowers Exactness

---

## 69. Network Replay

Historical network policy retained.

---

## 70. Hermetic Build

Easy to replay if all fixed inputs archived.

---

## 71. Networked Test

External state may differ.

---

## 72. NetworkObservation

Original run may retain endpoint identities/results.

---

## 73. ReplayNetworkMode

```rust
pub enum ReplayNetworkMode {
    ExactOfflineInputs,
    Virtualized,
    CurrentExternal,
    Denied,
}
```

---

## 74. CurrentExternal

Explicitly means not exact historical environment.

---

## 75. External API Time Dependence

Cannot be reconstructed without recorded/virtualized responses.

---

## 76. Replay Bundle

```rust
pub struct ReplayBundleId(Digest);
```

---

## 77. Replay Bundle Contents

Potential:

```text
ReplayManifest
source snapshot
pipeline IR
toolchain closure
dependency closure
test fixtures
config snapshots
non-secret environment metadata
required protocol schemas
```

---

## 78. Bundle Excludes Secrets

Critical.

---

## 79. Air-Gap Replay

Bundle can be imported into isolated installation.

---

## 80. Bundle Verification

Digest/signature.

---

## 81. ReplayPlanId

```rust
pub struct ReplayPlanId(Digest);
```

---

## 82. Replay Plan

```rust
pub struct ReplayPlan {
    pub id: ReplayPlanId,
    pub manifest: ReplayManifestId,
    pub fidelity: ReplayFidelity,
    pub substitutions: Vec<ReplaySubstitution>,
}
```

---

## 83. ReplaySubstitution

```rust
pub struct ReplaySubstitution {
    pub original: HistoricalInputRef,
    pub replacement: HistoricalInputRef,
    pub reason: ReplaySubstitutionReason,
}
```

---

## 84. Substitution Reasons

```rust
pub enum ReplaySubstitutionReason {
    MissingArtifact,
    SecurityRevocation,
    UnsupportedRuntime,
    PrivacyRestriction,
    UserRequested,
}
```

---

## 85. Security Revocation

Important.

A historical vulnerable image/tool may be unsafe to execute today.

---

## 86. Replay Security Policy

Can forbid execution of known malicious/vulnerable artifact.

---

## 87. Evidence-Only Mode

Always possible if metadata retained.

---

## 88. Quarantined Execution

High-risk historical artifacts can run only in isolated forensic sandbox.

---

## 89. ForensicReplayClass

```rust
pub enum ForensicReplayClass {
    Normal,
    Restricted,
    Quarantined,
    Forbidden,
}
```

---

## 90. Malware/Supply-Chain Incident

Historical compromised artifact may be intentionally replayed only in restricted lab.

---

## 91. No Network By Default

For forensic replay.

---

## 92. No Production Credentials

Critical.

---

## 93. Replay Runner

Dedicated executor profile.

---

## 94. Replay Runner Trust

Can differ from production build runner.

---

## 95. Historical Binary Compatibility

Old binaries may not run on current kernel/OS.

---

## 96. VM/Emulation

Can provide compatible environment.

---

## 97. Architecture Emulation

QEMU-style adapter possible.

---

## 98. Performance Results Under Emulation

Not comparable.

---

## 99. Fidelity Lowered

Explicit.

---

## 100. Historical Protocol

Agent/daemon protocol version may be obsolete.

---

## 101. Replay Worker

Should avoid requiring old agent talking directly to current daemon.

---

## 102. Preferred

Current replay worker executes archived job spec/tooling inside compatibility sandbox.

---

## 103. Legacy Worker Adapter

Optional for exact old behavior.

---

## 104. Security Isolation

Strong.

---

## 105. Historical Config

Exact `ConfigSnapshotId`.

---

## 106. Runtime Config

May contain references to now-deleted external resources.

---

## 107. Config Replay

Use archived value where non-secret and safe.

---

## 108. External Resource ID

May not exist.

---

## 109. Substitute

Explicit.

---

## 110. Policy Replay

Original `PolicyDigest` retained.

---

## 111. Policy Modes

```rust
pub enum ReplayPolicyMode {
    OriginalForAnalysis,
    CurrentForExecutionSafety,
}
```

---

## 112. Important Separation

Original policy explains historical decision.

Current policy governs whether replay execution is permitted today.

---

## 113. Historical Policy Cannot Override Current Security Floor

Critical.

---

## 114. Example

Old policy allowed vulnerable TLS.

Replay may simulate it offline but not expose live network.

---

## 115. Release Replay

For reproducible release verification:

```text
ReleaseId
  ↓
original derivation closure
  ↓
independent rebuild
  ↓
compare output digest
```

---

## 116. ReproductionResult

```rust
pub enum ReproductionResult {
    BitForBitMatch,
    NormalizedMatch,
    SemanticMatch,
    Mismatch,
    Inconclusive,
}
```

---

## 117. Mismatch

First-class evidence.

---

## 118. No Auto-Overwrite Original Artifact

Critical.

---

## 119. Rebuilt Artifact

New CAS object/evidence only.

---

## 120. Release Trust

Original remains original.

Replay may add verification evidence.

---

## 121. Failure Replay

Part 48.

---

## 122. HistoricalFailureReplay

Can answer:

```text
does failure still reproduce?
does exact environment reproduce?
does current environment reproduce?
```

---

## 123. Distinguish

```text
ExactHistoricalReplay
CurrentEnvironmentReplay
```

---

## 124. Regression Diagnosis

Useful.

---

## 125. Incident Replay

Part 61.

Can reconstruct sequence of:

```text
deploy
config
runner state
failure
```

---

## 126. Timeline Replay

Not simulation of reality unless inputs sufficient.

---

## 127. Event Reconstruction

Use original event log/outbox/audit snapshots.

---

## 128. Historical External Effects

Cannot necessarily replay safely.

---

## 129. Default

Do not repeat:

```text
production deploy
release publish
external webhook
payment-like effect
```

---

## 130. Replay Side-Effect Policy

```rust
pub enum ReplaySideEffectMode {
    Suppress,
    Virtualize,
    Sandbox,
    ExplicitLive,
}
```

---

## 131. Default

`Suppress` or `Virtualize`.

---

## 132. ExplicitLive

High-risk and usually not historical replay.

---

## 133. No Accidental Republish/Redeploy

Critical.

---

## 134. Notification Replay

Suppressed.

---

## 135. Webhook Replay

Virtualized/local capture.

---

## 136. Deployment Replay

Simulate/plan by default.

---

## 137. Database Migration Replay

Use test snapshot/environment.

Never replay against production by default.

---

## 138. Time

Historical time may affect behavior.

---

## 139. ReplayClock

```rust
pub enum ReplayClock {
    HistoricalFixed(Timestamp),
    Current,
    Virtual(TimeSimulationRef),
}
```

---

## 140. Historical Fixed Time

Useful for deterministic tests.

---

## 141. Expiring Certificates/Tokens

May fail if literally replayed.

---

## 142. Synthetic Equivalent

Can be used with lower fidelity.

---

## 143. Locale/Timezone

Retained.

---

## 144. Environment Variables

Non-secret declared values retained.

---

## 145. Hostname/PID

Nondeterministic.

---

## 146. Normalization

Part of reproducibility verification.

---

## 147. Replay Comparison

```rust
pub struct ReplayComparison {
    pub original: RunId,
    pub replay: RunId,
    pub output: Vec<OutputComparison>,
    pub evidence: Vec<EvidenceComparison>,
}
```

---

## 148. Output Comparison Types

```rust
pub enum OutputComparisonKind {
    Digest,
    NormalizedTree,
    Semantic,
    TestOutcome,
    Metric,
}
```

---

## 149. Exact Digest

Strongest.

---

## 150. Normalized Tree

Ignores defined nondeterministic metadata.

---

## 151. Semantic

Domain-specific.

---

## 152. Test Outcome

Pass/fail only, weaker.

---

## 153. Replay Gap

```rust
pub struct ReplayGap {
    pub input: HistoricalInputRef,
    pub reason: ReplayGapReason,
}
```

---

## 154. ReplayGapReason

```rust
pub enum ReplayGapReason {
    RetentionExpired,
    ExternalStateUnavailable,
    SecretUnavailable,
    ArtifactRevoked,
    UnsupportedTool,
    CorruptArchive,
    PrivacyRestriction,
    Unknown,
}
```

---

## 155. Gap Summary

Always shown before replay starts.

---

## 156. No Hidden Gap

Critical.

---

## 157. Historical Artifact Revocation

Security may prevent execution.

---

## 158. But

Metadata/provenance should remain available if retention allows.

---

## 159. Replay Authorization

```text
replay.read
replay.execute
replay.forensic
replay.export
replay.live_effect
```

---

## 160. Forensic

High privilege.

---

## 161. Live Effect

Highest privilege and usually disabled.

---

## 162. Tenant Isolation

Replay can access only original tenant/project data.

---

## 163. Cross-Tenant Replay

Forbidden.

---

## 164. Historical User Permission

Not enough.

Current user must be authorized now.

---

## 165. Historical Actor

Retained for provenance.

---

## 166. Current Principal

Controls replay authorization.

---

## 167. Audit

Audit:

```text
forensic replay
historical secret escrow access
replay bundle export
live external side-effect enablement
security-revoked artifact execution
```

---

## 168. Routine Debug Replay

Operational event.

---

## 169. Data Lifecycle

Part 46 is critical.

Replay capability depends on retained data.

---

## 170. Retention Tiers

Possible:

```text
metadata only
evidence
source + pipeline
full reproducibility closure
```

---

## 171. ReplayRetentionClass

```rust
pub enum ReplayRetentionClass {
    Metadata,
    Evidence,
    ReproducibleInputs,
    FullClosure,
}
```

---

## 172. Full Closure

Expensive.

---

## 173. Release Builds

Likely higher retention.

---

## 174. PR/Branch CI

Shorter.

---

## 175. Cost

Part 45.

Archival storage has cost.

---

## 176. Policy

Can choose retention by:

```text
release
project criticality
compliance
incident
```

---

## 177. No Claim Replay Forever Unless Retention Supports It

Critical.

---

## 178. Legal Hold

Can preserve replay closure.

---

## 179. Privacy Deletion

May intentionally make exact replay impossible.

---

## 180. Privacy Takes Priority

Critical.

---

## 181. Evidence Tombstone

Can record that input was legally deleted.

---

## 182. Search

Part 31 indexes replay metadata.

---

## 183. Dioxus UI

Pages:

```text
Historical Replay
Replay Plan
Replay Comparison
Replay Gaps
Replay Bundles
```

---

## 184. Run Detail

Button:

```text
Replay
```

---

## 185. Replay Preview

Shows:

```text
fidelity
available inputs
missing inputs
substitutions
security restrictions
estimated cost
```

---

## 186. Compare View

Side-by-side:

```text
original
replay
outputs
tests
logs
environment
```

---

## 187. CLI

```text
forgeyard replay plan <run>
forgeyard replay run <run>
forgeyard replay compare <replay>
forgeyard replay gaps <run>
forgeyard replay bundle create
forgeyard replay bundle inspect
forgeyard replay doctor
```

---

## 188. API

Potential:

```text
POST /v1/replays/plan
POST /v1/replays
GET  /v1/replays/{id}
GET  /v1/replays/{id}/comparison
GET  /v1/runs/{id}/replay-availability
```

---

## 189. Replay Availability

```rust
pub struct ReplayAvailability {
    pub fidelity: ReplayFidelity,
    pub missing: Vec<ReplayGap>,
}
```

---

## 190. Doctor

```text
forgeyard replay doctor
```

Checks:

```text
missing release closures
corrupt archived toolchains
unreadable old PipelineIr
missing compatibility bridge
expired replay retention
replay bundle verification
```

---

## 191. Health

```rust
pub enum ReplaySubsystemHealth {
    Healthy,
    ArchiveDegraded,
    CompatibilityDegraded,
    ToolchainArchiveDegraded,
    Unhealthy,
}
```

---

## 192. Observability Metrics

```text
replay_total
replay_exact_total
replay_best_effort_total
replay_failures_total
replay_gaps_total
replay_reproduction_mismatch_total
```

---

## 193. Labels

Low cardinality:

```text
fidelity
intent
result
```

---

## 194. No RunId Labels

---

## 195. Tracing

```text
replay.plan
replay.resolve
replay.materialize
replay.execute
replay.compare
replay.bundle
```

---

## 196. Compatibility

Part 57 central.

Historical replay needs compatibility for:

```text
PipelineIr
protocol schemas
toolchain metadata
runner image formats
bundle formats
```

---

## 197. Reader Migration

Current Forgeyard should read supported historical metadata versions.

---

## 198. Unsupported Historical Version

Evidence-only or compatibility bridge.

---

## 199. No Silent Metadata Rewrite

Critical.

---

## 200. CAS

Historical inputs stored by digest.

---

## 201. CAS GC Roots

Replay-retained closures become explicit roots.

---

## 202. Release Replay Root

Long-lived.

---

## 203. Incident Replay Root

Policy-based.

---

## 204. Garbage Collection

Must respect replay retention.

---

## 205. Dependency Mirror

Exact archived package blobs.

---

## 206. Artifact Registry

Exact container/tool images.

---

## 207. Runner Images

Historical baseline manifests/attestations.

---

## 208. Test Data

Historical fixture/dataset identity.

---

## 209. Workspace

Developer workspace itself is not replay input unless explicit snapshot was captured.

---

## 210. Build Runner

Historical host-local mutable cache is not replay input.

---

## 211. Cache

Rebuild can run cold.

---

## 212. Cache Hit Difference

Should not alter correct output.

---

## 213. Cache Reproduction

Optional for performance study only.

---

## 214. Performance Replay

Requires:

```text
hardware
load
runner baseline
cache state
```

Often lower confidence.

---

## 215. Historical Benchmark

Part 33 baseline matching.

---

## 216. Federation

Replay should preferably occur where retained data is authorized.

---

## 217. Residency

Historical data cannot move simply for replay convenience.

---

## 218. Regional Archive

May store local closure.

---

## 219. Air-Gap

Replay bundle ideal.

---

## 220. DR

Replay metadata/closures are part of backup strategy where promised.

---

## 221. Restore

CAS objects + metadata both needed.

---

## 222. Partial Restore

Replay availability recalculated.

---

## 223. No Assume Replay Closure Complete After DR

Critical.

---

## 224. Security

Threats:

```text
executing vulnerable old toolchains
secret resurrection
malware replay
cross-tenant archive leakage
replaying destructive effects
tampered archive
```

---

## 225. Controls

```text
current authz
current security floor
forensic isolation
no live effects by default
bundle signature verification
tenant scoping
no secret values in manifest
```

---

## 226. Historical Vulnerability

Can be investigated without restoring production access.

---

## 227. Quarantine Network

For forensic.

---

## 228. No Signing Credentials

Critical.

---

## 229. No Release Publish Credential

---

## 230. Replay Output

Marked:

```text
REPLAY / NON-PRODUCTION
```

where user-facing.

---

## 231. Artifact Promotion

Replay output cannot be promoted to release automatically.

---

## 232. If identical rebuild is desired

Normal release verification policy must explicitly accept evidence.

---

## 233. Replay Event

```rust
pub enum ReplayEvent {
    Planned,
    Materializing,
    Running,
    Completed,
    Failed,
    GapDetected,
    ComparisonCompleted,
}
```

---

## 234. Reconciler

Checks:

```text
missing archive materialization
stuck replay
provider VM state
bundle import state
comparison completion
```

---

## 235. HA

Replay workers independent/idempotent.

---

## 236. Concurrency

Part 60 can limit expensive forensic replays.

---

## 237. No Exclusive Lock On Original Run

Historical record immutable.

---

## 238. Testkit

```text
forgeyard-replay-testkit/src/
├── lib.rs
├── manifest.rs
├── archive.rs
├── environment.rs
├── replay.rs
├── compare.rs
├── gaps.rs
└── assertions.rs
```

---

## 239. Core Tests

### Manifest
- exact original identities captured;
- manifest digest deterministic.

### Source
- deleted VCS branch still replayable if SourceSnapshot retained;
- current branch never substituted automatically.

### Toolchain
- exact version resolved;
- missing compiler produces gap;
- substitute lowers fidelity.

### Pipeline
- archived IR preferred;
- incompatible historical IR requires bridge.

### Dependencies
- upstream package deletion does not matter if mirrored;
- missing archive explicit.

### Secrets
- manifest contains SecretRef only;
- no secret value in bundle.

### Side Effects
- publish/deploy/webhook suppressed by default.

### Replay
- new RunId created;
- original run remains unchanged.

### Comparison
- digest mismatch preserved as evidence;
- normalized match distinct from bit-for-bit.

### Security
- revoked toolchain runs only forensic/blocked by policy.

### DR
- partial archive restore reduces replay availability honestly.

---

## 240. Chaos Tests

Inject:

```text
archive object missing
CAS corruption
old image unreadable
toolchain materialization failure
provider VM timeout
compatibility bridge failure
```

Expected:

```text
fidelity/gaps updated
original evidence untouched
no silent substitution
```

---

## 241. Scale Tests

Test:

```text
years of run history
large monorepo source trees
large toolchain closures
many replay requests
air-gap bundle import
```

---

## 242. Implementation Phases

### Phase 1 — Replay Manifest
Capture at run time.

### Phase 2 — Source/Pipeline Replay
Historical core reconstruction.

### Phase 3 — Toolchain/Dependency Archive
Exact execution inputs.

### Phase 4 — Replay Comparison
Output/evidence diff.

### Phase 5 — Failure Replay
Part 48 integration.

### Phase 6 — Release Reproduction
Supply-chain verification.

### Phase 7 — Test Environment Replay
Fixtures/services.

### Phase 8 — Historical Compatibility Bridges
Long-term support.

### Phase 9 — Replay Bundles/Air-Gap
Portable reproduction.

### Phase 10 — Forensic/Security Replay
Restricted lab.

### Phase 11 — Retention/Cost Governance
Archive economics.

### Phase 12 — Chaos/Scale/DR Hardening
Production readiness.

---

## 243. Acceptance Tests

1. Historical replay creates a new Run/ReplayId.
2. Original Run/Job/evidence remains immutable.
3. Replay uses exact SourceSnapshotId when available.
4. Current branch state is never silently substituted.
5. Original PipelineIr is preferred over reparsing historical config.
6. Toolchains are exact/digest-bound.
7. Dependency blobs resolve by exact archived identity.
8. Missing dependencies become explicit ReplayGap.
9. Runner/environment substitution lowers fidelity.
10. Secret values are never stored in ReplayManifest.
11. Live external effects are suppressed by default.
12. Historical policy explains original decision but current policy governs execution safety.
13. Revoked/vulnerable artifacts can be blocked or sandboxed.
14. Reproduction mismatch is preserved as evidence.
15. Replay output cannot silently replace original artifact.
16. Test-data privacy restrictions can intentionally reduce replay fidelity.
17. Replay retention class is explicit.
18. CAS GC respects replay roots.
19. Release builds can retain full reproducibility closure.
20. Branch CI may have shorter replay retention.
21. Air-gap replay bundles verify digest/signature.
22. Replay bundles exclude secrets.
23. Federation/residency rules constrain archive movement.
24. DR recalculates replay availability after partial restore.
25. Unsupported historical metadata becomes explicit compatibility issue.
26. Performance replay distinguishes hardware/environment mismatch.
27. Forensic replay cannot access production credentials.
28. Tenant archive access is isolated.
29. Best-effort replay is labeled honestly.
30. Forgeyard dogfoods historical replay for its own release and incident investigations.

---

## 244. Production Readiness Gates

Do not call historical replay production-ready until:

```text
ReplayManifest is captured automatically
source/pipeline/toolchain identities are exact
dependency archive closure is verified
secret exclusion is proven
live side effects are suppressed by default
fidelity/gap reporting is machine-enforced
release reproduction works
old metadata compatibility path exists
DR/air-gap replay tests pass
security/forensic isolation tests pass
```

---

## 245. Architectural Invariants

1. replay never mutates historical truth;
2. every replay has explicit fidelity;
3. exact source is preferred;
4. no current branch substitution;
5. archived IR preferred over reparsing;
6. toolchain substitutions are explicit;
7. dependency substitutions are explicit;
8. missing inputs become gaps;
9. secrets are references, not archived values;
10. live side effects are suppressed by default;
11. current security policy governs replay execution;
12. original policy remains evidence;
13. revoked artifacts do not regain trust through replay;
14. replay output is new evidence/artifact;
15. mismatch is preserved;
16. privacy may intentionally limit replay;
17. retention determines replay capability;
18. CAS GC honors replay roots;
19. historical compatibility is versioned;
20. air-gap bundles are verifiable;
21. forensic replay is isolated;
22. tenant access remains isolated;
23. performance replay requires environment qualification;
24. DR may reduce replay fidelity honestly;
25. archive corruption is first-class;
26. no fake exact replay claim;
27. replay works independently of current VCS branch existence;
28. replay does not depend on warm cache;
29. historical external state is never invented;
30. Forgeyard dogfoods its own replay system.

---

## 246. Final Target Architecture

```text
                   Original RunId
                        │
                        ▼
                  ReplayManifestId
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       Source        Toolchains    Dependencies
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                 ReplayPlan + Fidelity
                        │
                        ▼
                    New Replay Run
                        │
                        ▼
                 Output/Evidence Compare
```

Fidelity:

```text
all exact inputs retained
      ↓
deterministic replay

some equivalent substitutions
      ↓
equivalent/semantic replay

critical inputs missing
      ↓
best-effort/evidence-only/impossible
```

The key guarantee is:

> **Forgeyard can explain and reproduce historical CI behavior to the exact fidelity supported by retained evidence. It never hides missing inputs, silently upgrades historical dependencies, resurrects old secrets, or repeats destructive side effects merely to claim reproducibility.**

---

## 247. Extended Architecture Sequence

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
```
