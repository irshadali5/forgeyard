# 48 — Forgeyard Failure Diagnosis, Debugging, Reproduction, Bisect & Root-Cause Intelligence System Architecture

**Document type:** Core Failure Diagnosis, Debugging, Reproduction, Bisect, Differential Analysis & Root-Cause Intelligence System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** failure signatures, failure clustering, exact reproduction, clean debug sandboxes, log/evidence correlation, first-bad-change bisect, environment/toolchain/config diffs, infrastructure-vs-product classification, flaky-vs-deterministic diagnosis, historical comparison, incident/debug bundles, advisory automated hypotheses, and developer/operator investigation workflows  
**Architecture style:** Evidence-first, immutable observations, deterministic reproduction where possible, clean-room debugging, causal humility, typed failure classification, source/config/toolchain exactness, explainable comparison, and no diagnostic system becoming execution or policy authority  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Run/Job State Machine, Observability, Test Intelligence, Benchmarking, Static Analysis, Monorepo Intelligence, Developer Experience, Cache Correctness, Security, Triggers, Migration, CAS, Reproducibility, and Search/Analytics. This subsystem centralizes how Forgeyard helps developers answer not just “what failed?” but “why did it fail, can I reproduce it, when did it begin, and what changed?”

---

# 1. Purpose

CI/CD failures are often expensive because the hard part is not detecting red status; it is determining:

```text
is this failure deterministic?
is it flaky?
is it caused by infrastructure?
is it caused by a code change?
is it caused by toolchain drift?
is it caused by config?
is it caused by dependency resolution?
can I reproduce it locally?
what was the last known-good revision?
what is the first bad revision?
what changed between good and bad?
```

The central rule is:

> **Forgeyard diagnostics interpret immutable execution evidence; they do not rewrite execution truth. A failed attempt remains failed even if a later retry passes, and a suspected cause remains a hypothesis until supported by evidence.**

A second rule is:

> **Debugging and reproduction should recreate the failed subject in a new clean sandbox from immutable source, toolchain, configuration, declared inputs, and environment—not by attaching to a contaminated failed sandbox as the default workflow.**

A third rule is:

> **Root-cause intelligence must distinguish correlation from causation. Forgeyard may rank likely causes, but it must expose the evidence and uncertainty behind every conclusion.**

---

# 2. Architectural Position

```text
                    Failed Run / Job
                          │
                          ▼
                  Failure Observation
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Logs         Evidence       Context
             │            │            │
             └────────────┼────────────┘
                          ▼
                  Failure Normalizer
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
        Signature      Classification  Cluster
             │            │            │
             └────────────┼────────────┘
                          ▼
                    Diagnosis Engine
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
      Reproduce         Compare          Bisect
          │               │               │
          └───────────────┼───────────────┘
                          ▼
                  Root-Cause Evidence
```

---

# 3. Goals

The subsystem MUST:

1. define failure identity;
2. define failure signatures;
3. classify failure sources;
4. correlate repeated failures;
5. distinguish flaky from deterministic;
6. distinguish infrastructure from workload failures;
7. support exact reproduction;
8. support local reproduction;
9. support remote reproduction;
10. support clean debug sandboxes;
11. support environment diff;
12. support toolchain diff;
13. support config diff;
14. support dependency diff;
15. support source diff;
16. support first-bad-change bisect;
17. support historical known-good comparison;
18. support failure clustering;
19. support developer-facing explanations;
20. support operator-facing explanations;
21. support debug bundles;
22. support advisory automated hypotheses;
23. support policy-safe diagnostics;
24. support tenant isolation;
25. support CLI/UI;
26. support notifications;
27. support search/analytics;
28. support standalone mode;
29. support distributed mode;
30. remain evidence-preserving and explainable.

---

# 4. Non-Goals

This subsystem does not:

```text
change Run/Job outcomes
replace test flake intelligence
replace observability
replace incident response
guarantee root cause
mutate source code automatically
```

---

# 5. Workspace Structure

```text
crates/diagnosis/
├── forgeyard-diagnosis/
├── forgeyard-diagnosis-model/
├── forgeyard-diagnosis-signature/
├── forgeyard-diagnosis-classify/
├── forgeyard-diagnosis-cluster/
├── forgeyard-diagnosis-reproduce/
├── forgeyard-diagnosis-diff/
├── forgeyard-diagnosis-bisect/
├── forgeyard-diagnosis-hypothesis/
├── forgeyard-diagnosis-bundle/
├── forgeyard-diagnosis-health/
└── forgeyard-diagnosis-testkit/
```

Use modules first; split only where algorithm/tool dependencies justify.

---

# 6. FailureObservationId

```rust
pub struct FailureObservationId(Ulid);
```

One immutable failure observation.

---

# 7. Failure Observation

```rust
pub struct FailureObservation {
    pub id: FailureObservationId,
    pub run: RunId,
    pub job: JobId,
    pub attempt: Option<JobAttemptId>,
    pub source: SourceSnapshotId,
    pub class: FailureClass,
    pub signature: FailureSignature,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 8. FailureClass

Reuse/extend existing Part 05 failure classes.

Recommended categories:

```rust
pub enum DiagnosticFailureClass {
    Workload,
    Test,
    Compiler,
    StaticAnalysis,
    Dependency,
    Toolchain,
    Configuration,
    Sandbox,
    Infrastructure,
    Network,
    Secret,
    Policy,
    Timeout,
    Cancellation,
    Preemption,
    Provider,
    Unknown,
}
```

---

# 9. Outcome Authority

Canonical Job/Attempt failure state remains Part 05.

---

# 10. Diagnostic Classification

Derived interpretation.

---

# 11. FailureSignature

```rust
pub struct FailureSignature(Digest);
```

---

# 12. Signature Inputs

Potential normalized inputs:

```text
failure class
exit code
sanitized stderr pattern
test identity
panic/backtrace frame pattern
tool diagnostic code
provider error class
```

---

# 13. No Secret Material

Critical.

---

# 14. Signature Version

```rust
pub struct FailureSignatureVersion(u16);
```

---

# 15. Why Version

Normalization changes.

---

# 16. Raw Evidence

Preserved separately.

---

# 17. FailureClusterId

```rust
pub struct FailureClusterId(Digest);
```

---

# 18. Cluster

Groups likely same failure family.

---

# 19. Correlation Confidence

```rust
pub enum FailureCorrelationConfidence {
    Exact,
    Strong,
    Heuristic,
    Unknown,
}
```

---

# 20. Exact

Same stable test/tool fingerprint.

---

# 21. Strong

Same normalized signature/context.

---

# 22. Heuristic

Presentation only.

---

# 23. No Heuristic Cluster as Causal Proof

Critical.

---

# 24. Failure Stability

```rust
pub enum FailureStability {
    Deterministic,
    LikelyDeterministic,
    Flaky,
    LikelyFlaky,
    Unknown,
}
```

---

# 25. Test Flake

Part 32 remains canonical for test-case stability.

---

# 26. Generic Flake

Can apply to build/network/provider failures too.

---

# 27. Infrastructure vs Workload

Important split.

---

# 28. Infrastructure Evidence

Examples:

```text
runner disappeared
spot interruption
provider outage
CAS unavailable
network transport reset
```

---

# 29. Workload Evidence

Examples:

```text
compiler error
assertion failure
deterministic command exit
```

---

# 30. Unknown

Prefer Unknown over false certainty.

---

# 31. FailureContext

```rust
pub struct FailureContext {
    pub source: SourceSnapshotId,
    pub pipeline_plan: PipelinePlanId,
    pub toolchain: ToolchainDescriptorId,
    pub config: ConfigSnapshotId,
    pub runner: Option<RunnerId>,
    pub platform: PlatformDescriptor,
    pub dependencies: Option<DependencyClosureId>,
}
```

---

# 32. Exact Context

Required for meaningful comparison.

---

# 33. ReproductionRequestId

```rust
pub struct ReproductionRequestId(Ulid);
```

---

# 34. Reproduction Request

```rust
pub struct ReproductionRequest {
    pub failure: FailureObservationId,
    pub mode: ReproductionMode,
    pub actor: PrincipalId,
}
```

---

# 35. Reproduction Mode

```rust
pub enum ReproductionMode {
    Local,
    RemoteSameClass,
    RemoteExactProfile,
    DebugInteractive,
}
```

---

# 36. Reproduction Principle

Rebuild clean sandbox from immutable inputs.

---

# 37. Do Not Attach to Failed Sandbox by Default

Critical.

---

# 38. Why

Failed sandbox may be contaminated by:

```text
mutated files
temporary secrets
partial outputs
timing side effects
```

---

# 39. ReproductionSpec

```rust
pub struct ReproductionSpec {
    pub source: SourceSnapshotId,
    pub job_ir: JobIrId,
    pub toolchain: ToolchainDescriptorId,
    pub config_digest: ConfigDigest,
    pub environment: ExecutionEnvironmentId,
    pub inputs: Vec<CasObjectRef>,
}
```

---

# 40. Secretful Reproduction

Requires fresh authorization and new secret resolution.

---

# 41. Never Reuse Captured Secret Value

Critical.

---

# 42. Reproduction Result

```rust
pub enum ReproductionResult {
    ReproducedSameFailure,
    ReproducedDifferentFailure,
    Passed,
    Inconclusive,
    InfrastructureFailed,
}
```

---

# 43. ReproducedSameFailure

Strong evidence of determinism.

---

# 44. Passed

Could indicate flake/environment difference.

---

# 45. Inconclusive

Missing capability/evidence.

---

# 46. Reproduction Count

Bounded.

---

# 47. Automatic Reproduction

Policy-limited.

---

# 48. No Infinite Retry Diagnosis

Critical.

---

# 49. Debug Sandbox

Clean recreation with optional interactive tools.

---

# 50. DebugSandboxId

```rust
pub struct DebugSandboxId(Ulid);
```

---

# 51. Debug Sandbox Characteristics

```text
new sandbox
same source
same toolchain
same declared inputs
same platform profile where possible
debug tooling enabled
fresh secret authorization
explicit expiry
```

---

# 52. Interactive Access

Strongly authenticated.

---

# 53. Remote Shell

Not baseline agent protocol.

---

# 54. Debug Session

Can be a separate restricted service/protocol.

---

# 55. DebugSessionId

```rust
pub struct DebugSessionId(Ulid);
```

---

# 56. Debug Session Permissions

```text
diagnosis.debug.start
diagnosis.debug.attach
```

---

# 57. Debug Session Expiry

Mandatory.

---

# 58. Debug Sandbox Network

Same or stricter than original unless explicit override.

---

# 59. Override

Audited and clearly marks environment divergence.

---

# 60. Debug Mutations

Do not become build/release artifacts automatically.

---

# 61. Export Patch

Developer can extract source diff as explicit patch.

---

# 62. No Hidden Source Mutation

Critical.

---

# 63. Environment Diff

Compare failed vs known-good.

---

# 64. EnvironmentDiff

```rust
pub struct EnvironmentDiff {
    pub platform: DiffField<PlatformDescriptor>,
    pub toolchain: DiffField<ToolchainDescriptorId>,
    pub config: Vec<ConfigFieldDiff>,
    pub dependencies: Vec<DependencyDiff>,
    pub capabilities: Vec<CapabilityDiff>,
}
```

---

# 65. Source Diff

Part 34 canonical tree diff.

---

# 66. Config Diff

Part 39 typed config diff.

---

# 67. Dependency Diff

Part 36.

---

# 68. Toolchain Diff

Immutable descriptors.

---

# 69. Runner Diff

Compare trust/class/image/capability, not incidental RunnerId only.

---

# 70. Known-Good Selection

Need explicit baseline.

---

# 71. KnownGoodSelector

```rust
pub enum KnownGoodSelector {
    ExplicitRun(RunId),
    LastSuccessfulSameJob,
    LastSuccessfulSameTargetBranch,
    LastRelease,
}
```

---

# 72. Resolve to Exact Run/Source

Before comparison.

---

# 73. No Mutable "last good" Stored as Authority

---

# 74. Differential Diagnosis

```text
failed context
-
known-good context
  ↓
candidate changes
```

---

# 75. Candidate Cause

Not root cause yet.

---

# 76. DiagnosticEvidenceKind

```rust
pub enum DiagnosticEvidenceKind {
    SourceChange,
    DependencyChange,
    ToolchainChange,
    ConfigChange,
    PlatformChange,
    RunnerImageChange,
    TestHistory,
    FailureSignatureMatch,
    ProviderIncident,
}
```

---

# 77. RootCauseHypothesisId

```rust
pub struct RootCauseHypothesisId(Ulid);
```

---

# 78. Root-Cause Hypothesis

```rust
pub struct RootCauseHypothesis {
    pub id: RootCauseHypothesisId,
    pub failure: FailureObservationId,
    pub subject: RootCauseSubject,
    pub confidence: HypothesisConfidence,
    pub evidence: Vec<DiagnosticEvidenceRef>,
    pub rationale: BoundedString,
}
```

---

# 79. Hypothesis Confidence

```rust
pub enum HypothesisConfidence {
    Low,
    Medium,
    High,
    Confirmed,
}
```

---

# 80. Confirmed

Requires strong experimental/evidence support.

---

# 81. Example Confirmed

Bisect finds exact first bad commit and reverting it reproduces recovery.

---

# 82. No "AI Says Root Cause"

Critical.

---

# 83. Automated Hypotheses

Can use rules/statistics/AI later.

---

# 84. AI

Advisory only.

---

# 85. AI Input

Sanitized evidence respecting tenant/privacy policy.

---

# 86. External AI

Source/code egress requires explicit policy.

---

# 87. Local AI

Optional.

---

# 88. No AI Required Baseline

Critical.

---

# 89. First-Bad-Change Bisect

Major capability.

---

# 90. BisectSessionId

```rust
pub struct BisectSessionId(Ulid);
```

---

# 91. Bisect Preconditions

Need:

```text
known-good revision
known-bad revision
deterministic-enough predicate
buildable intermediate revisions
```

---

# 92. Bisect Predicate

```rust
pub enum BisectPredicate {
    JobOutcome(JobSelector),
    TestOutcome(TestCaseId),
    ArtifactProperty(ArtifactPredicate),
    Custom(BisectPredicateId),
}
```

---

# 93. Predicate Must Be Deterministic Enough

---

# 94. Flaky Predicate

Use repeated evaluation/confidence.

---

# 95. Bisect Source Space

VCS-neutral revisions.

---

# 96. Git

Use commit ancestry.

---

# 97. Mercurial

Use changeset ancestry.

---

# 98. Canonical Source Snapshot

Every tested revision materializes exact `SourceSnapshotId`.

---

# 99. Merge Histories

Bisect strategy can be complex.

---

# 100. Baseline

Linear ancestry path where defined.

---

# 101. Nonlinear DAG

Explicit strategy.

---

# 102. BisectStrategy

```rust
pub enum BisectStrategy {
    LinearAncestry,
    FirstParent,
    VcsNative,
    Custom(BisectStrategyId),
}
```

---

# 103. No Hidden VCS Assumption

Critical.

---

# 104. Bisect Job

Normal Forgeyard Run/Job execution.

---

# 105. Bisect Does Not Get Privileged Shortcut

---

# 106. Cache

Can accelerate tested revisions safely.

---

# 107. Required Evidence

Each bisect point records:

```text
revision
SourceSnapshotId
predicate result
attempts
confidence
```

---

# 108. Bisect Result

```rust
pub struct BisectResult {
    pub first_bad: Option<RevisionId>,
    pub first_bad_snapshot: Option<SourceSnapshotId>,
    pub confidence: HypothesisConfidence,
    pub tested: Vec<BisectPoint>,
}
```

---

# 109. Inconclusive Bisect

First-class.

---

# 110. Causes

```text
unbuildable revisions
flaky predicate
missing platform
history rewrite
```

---

# 111. Unbuildable Revision

Not automatically bad.

---

# 112. Bisect Classification

```rust
pub enum BisectPointResult {
    Good,
    Bad,
    Skip,
    Inconclusive,
}
```

---

# 113. Skip

For untestable revision.

---

# 114. Bounded Search

Max runs/time/cost.

---

# 115. Cost Budget

Part 45 integration.

---

# 116. User/Policy Limits

Critical.

---

# 117. Parallel Bisect

Possible later.

---

# 118. Baseline

Sequential/adaptive.

---

# 119. Historical Comparison

Failure cluster timeline.

---

# 120. First Seen

Exact Run/Source.

---

# 121. Last Seen

Exact.

---

# 122. Frequency

By branch/platform/job.

---

# 123. Regression Window

```text
last known good
first known bad
```

---

# 124. Change Correlation

Part 34 affected graph can narrow suspects.

---

# 125. Example

Failure only on target affected by dependency X.

---

# 126. Correlation Not Proof

Critical.

---

# 127. Dependency Regression

Can bisect lockfile/source versions where exact historical states exist.

---

# 128. Toolchain Regression

Can compare pinned versions.

---

# 129. Toolchain Bisect

Future optional.

---

# 130. Runner Image Regression

Compare image generations.

---

# 131. Provider Incident Correlation

If many unrelated jobs fail same infrastructure class/time.

---

# 132. Infrastructure Incident Cluster

Useful for operator diagnosis.

---

# 133. Blast Radius

```rust
pub struct FailureBlastRadius {
    pub affected_projects: u32,
    pub affected_fleets: u32,
    pub affected_platforms: Vec<PlatformDescriptor>,
}
```

---

# 134. Cross-Tenant Analytics

Only aggregate/admin-authorized.

---

# 135. Tenant Privacy

No cross-tenant source/failure detail leakage.

---

# 136. Failure Bundle

```rust
pub struct FailureBundle {
    pub id: FailureBundleId,
    pub failure: FailureObservationId,
    pub logs: Vec<CasObjectRef>,
    pub evidence: Vec<EvidenceRef>,
    pub context: FailureContext,
    pub diagnostics: Vec<DiagnosticEvidenceRef>,
}
```

---

# 137. FailureBundleId

Content-derived.

---

# 138. Support Bundle Integration

Part 17.

---

# 139. Redaction

Secret-safe.

---

# 140. Export

Permission-gated.

---

# 141. Reproduction Bundle

Can package immutable non-secret inputs for offline/local reproduction.

---

# 142. ReproductionBundleId

```rust
pub struct ReproductionBundleId(Digest);
```

---

# 143. Contents

```text
source snapshot
toolchain descriptors
config projection
input manifests
job IR
expected platform requirements
```

---

# 144. No Secret Values

Critical.

---

# 145. Secret Refs

Can be present if safe, but local reproduction must reauthorize/resolve.

---

# 146. Air-Gap Debugging

Bundle usable offline if dependency/toolchain closure included.

---

# 147. Local Reproduction CLI

```text
forgeyard reproduce <job-id>
```

---

# 148. Default

Clean local sandbox.

---

# 149. If local platform incompatible

Explain and offer remote reproduction.

---

# 150. `forgeyard debug <job-id>`

Creates clean debug sandbox/session.

---

# 151. `forgeyard diagnosis explain <job-id>`

Shows classification/evidence/hypotheses.

---

# 152. `forgeyard diagnosis compare <bad> <good>`

Context/source differences.

---

# 153. `forgeyard bisect start`

Starts bounded bisect.

---

# 154. `forgeyard bisect status`

Progress.

---

# 155. `forgeyard bisect stop`

Cancel.

---

# 156. CLI Output

Human/JSON/RON.

---

# 157. Dioxus UI

Pages/panels:

```text
Failure Overview
Similar Failures
Reproduce
Compare with Last Good
Bisect
Failure Timeline
Debug Session
```

---

# 158. Failure Detail

Shows:

```text
class
signature
stability
first seen
similar failures
context
evidence
```

---

# 159. Hypothesis UI

Shows:

```text
candidate cause
confidence
supporting evidence
contradicting evidence
```

---

# 160. Contradicting Evidence

Important.

---

# 161. No Single Magic Root-Cause Badge

Critical.

---

# 162. Reproduction UI

Shows divergence from original environment.

---

# 163. Environment Difference Warning

If exact platform unavailable.

---

# 164. Bisect UI

Graph/timeline of tested revisions.

---

# 165. Search Integration

Part 31 indexes failure signatures/clusters.

---

# 166. Query

Examples:

```text
show all failures like this
show first occurrence
show Linux-only failures
```

---

# 167. Analytics

Examples:

```text
top failure classes
mean time to diagnosis
reproduction success rate
infrastructure failure rate
bisect effectiveness
```

---

# 168. No Developer Performance Ranking

Critical.

---

# 169. Notification

Examples:

```text
new widespread infrastructure cluster
confirmed regression
bisect completed
```

---

# 170. Audit

Audit:

```text
debug session access
sensitive bundle export
manual diagnosis override
```

---

# 171. Routine diagnosis

Operational metadata.

---

# 172. Permissions

```text
diagnosis.read
diagnosis.reproduce
diagnosis.debug.start
diagnosis.debug.attach
diagnosis.bisect
diagnosis.bundle.export
```

---

# 173. Debug Session

Higher privilege.

---

# 174. Secret Access

Still governed separately.

---

# 175. API

Potential:

```text
GET  /v1/jobs/{id}/diagnosis
POST /v1/jobs/{id}/reproduce
POST /v1/jobs/{id}/debug
POST /v1/diagnosis/compare
POST /v1/bisects
GET  /v1/bisects/{id}
```

---

# 176. Reproduction Scheduling

Normal scheduler.

---

# 177. Exact Platform

Hard requirement.

---

# 178. Same Runner

Not required and often undesirable.

---

# 179. Same Runner Class

Prefer if investigating environment-specific issue.

---

# 180. Quarantined Runner

Do not reproduce there unless security/operator forensic workflow.

---

# 181. Security Incident

Part 40 controls forensic handling.

---

# 182. Debug Data Sensitivity

Failure logs/source can be confidential.

---

# 183. Tenant Isolation

All failure clusters default tenant/project scoped.

---

# 184. Cross-Tenant Infrastructure Cluster

Only normalized non-sensitive signature/infra metadata.

---

# 185. Data Lifecycle

Part 46 governs retention.

---

# 186. Failure Bundle Hold

Security/legal incident may extend retention.

---

# 187. Reproduction Artifact Retention

Short-lived by default.

---

# 188. Debug Session Data

Short-lived.

---

# 189. Cost

Bisect/reproduction can consume compute.

---

# 190. Cost Estimate

Optional before large bisect.

---

# 191. Budget

Can cap diagnosis campaigns.

---

# 192. Security-Critical Diagnosis

Policy exception can override cost limits.

---

# 193. Cache

Reproduction may use trusted cache if policy permits.

---

# 194. Exact Reproduction

Policy may disable cache to ensure actual execution.

---

# 195. Reproducibility Verification

Can compare output digest against prior result.

---

# 196. Same Failure + Same Output

Strong evidence.

---

# 197. Nondeterminism Detection

Same derivation -> different output integrates Part 38/FRBS.

---

# 198. Test Intelligence

Part 32 provides:

```text
test identity
flake history
failure signature
quarantine status
```

---

# 199. Benchmark Intelligence

Performance regressions can use bisect.

---

# 200. Performance Bisect

Predicate:

```text
metric exceeds threshold
```

---

# 201. Statistical Predicate

Needs confidence/repeats.

---

# 202. Static Analysis Regression

Can bisect first revision introducing finding.

---

# 203. Dependency Finding

Can correlate dependency diff.

---

# 204. Build Failure

Compiler diagnostic codes useful.

---

# 205. Panic/Crash

Backtrace normalization.

---

# 206. Native Crash

Symbolization may require debug symbols.

---

# 207. Debug Symbols

CAS artifact with retention/security policy.

---

# 208. Symbolization Service

Optional.

---

# 209. No Internet Symbol Lookup by Default

Critical.

---

# 210. Core Dumps

Sensitive; disabled or restricted by security profile.

---

# 211. If Collected

Restricted CAS class and lifecycle.

---

# 212. Platform-Specific Debugging

Linux:

```text
gdb/lldb
strace
perf where allowed
```

---

# 213. Windows

```text
WinDbg/Visual Studio debugger where configured
```

---

# 214. macOS

```text
lldb
```

---

# 215. Android

```text
adb/logcat
native symbols
```

---

# 216. Debug Tool Capability

Explicit.

---

# 217. Privileged Tracing

Requires policy.

---

# 218. eBPF

Optional diagnostic accelerator on Linux.

---

# 219. Not correctness dependency.

---

# 220. Reproduction Fidelity

```rust
pub enum ReproductionFidelity {
    Exact,
    EquivalentClass,
    Partial,
    Unknown,
}
```

---

# 221. Exact

Same declared semantics/platform/toolchain/config.

---

# 222. EquivalentClass

Same compatibility class, not exact host.

---

# 223. Partial

Known differences.

---

# 224. UI Must Show Fidelity

Critical.

---

# 225. Differential Variable Search

Potential future optimization.

---

# 226. Change one dimension at a time:

```text
toolchain
dependency set
config
platform
```

---

# 227. Experimental Diagnosis

Each trial is normal Run/Job.

---

# 228. ExperimentId

```rust
pub struct DiagnosticExperimentId(Ulid);
```

---

# 229. Experiment Result

Evidence, not mutation.

---

# 230. Automated Revert Test

Possible:

```text
apply candidate revert patch in ephemeral source snapshot
run predicate
```

---

# 231. Does Not Modify repository.

---

# 232. If passes

Strong support for cause.

---

# 233. Patch Provenance

Explicit.

---

# 234. Source Snapshot

Derived ephemeral snapshot.

---

# 235. No Auto-Merge

Critical.

---

# 236. AI-Assisted Diagnosis

Optional future.

---

# 237. Guardrails

```text
advisory
evidence citations
no secret exfiltration
no automatic production action
no hidden code changes
```

---

# 238. Hypothesis Generation

Could summarize:

```text
most likely differences
similar historical failures
suspected files/dependencies
```

---

# 239. Confidence

Model confidence never substitutes evidence.

---

# 240. External Model

Source egress policy explicit.

---

# 241. Local Model

Can be preferred high-assurance.

---

# 242. DiagnosisReconciler

Checks:

```text
failed jobs without normalized observation
stuck reproduction
stuck bisect
missing comparison baseline
expired debug sessions
```

---

# 243. Idempotent

---

# 244. HA

Multiple diagnosis workers safe.

---

# 245. Bisect Claims

DB-coordinated.

---

# 246. No Raft Requirement

Ordinary diagnosis coordination via metadata store.

---

# 247. Observability Metrics

```text
diagnosis_failures_classified_total
diagnosis_reproduction_total
diagnosis_reproduction_success_total
diagnosis_bisect_total
diagnosis_bisect_inconclusive_total
diagnosis_debug_sessions_active
```

---

# 248. Labels

Low cardinality:

```text
failure_class
result
fidelity
```

---

# 249. No file/test/tenant IDs in metrics.

---

# 250. Tracing

```text
diagnosis.normalize
diagnosis.cluster
diagnosis.compare
diagnosis.reproduce
diagnosis.bisect
diagnosis.hypothesis
```

---

# 251. Health

Checks:

```text
normalization backlog
symbolizer availability
reproduction worker health
bisect queue
debug session cleanup
```

---

# 252. Doctor

```text
forgeyard diagnosis doctor
```

---

# 253. Doctor Checks

```text
missing failure evidence
unsupported reproduction platform
debug tool availability
symbol files
stuck diagnosis jobs
```

---

# 254. Standalone Mode

Local reproduction/debug directly.

---

# 255. Distributed Mode

Remote reproduction/bisect/fleet integration.

---

# 256. Offline Mode

Reproduction bundle.

---

# 257. DR

Diagnosis metadata derived/rebuildable where evidence retained.

---

# 258. Bisect/Debug State

Operational metadata backed up if long-running.

---

# 259. Failure Signature Reprocessing

Versioned.

---

# 260. New Signature Algorithm

Creates new derived clustering view.

---

# 261. Historical observations immutable.

---

# 262. Failure Cluster Rebuild

Possible.

---

# 263. Testkit

```text
forgeyard-diagnosis-testkit/src/
├── lib.rs
├── failure.rs
├── signature.rs
├── cluster.rs
├── reproduce.rs
├── diff.rs
├── bisect.rs
└── assertions.rs
```

---

# 264. Unit Tests

Failure signature determinism.

---

# 265. Secret Redaction Test

Secret never enters signature.

---

# 266. Cluster Test

Exact vs heuristic confidence.

---

# 267. Infrastructure Classification Test

Spot interruption != workload failure.

---

# 268. Reproduction Test

Same immutable inputs reproduce same failure.

---

# 269. Secretful Reproduction Test

Fresh auth required.

---

# 270. Failed Sandbox Test

Default workflow creates new sandbox.

---

# 271. Environment Diff Test

Toolchain/config/platform differences exact.

---

# 272. Known-Good Test

Mutable selector resolves exact Run.

---

# 273. Bisect Linear Test

Finds first bad.

---

# 274. Bisect Flaky Test

Returns inconclusive/bounded confidence.

---

# 275. Unbuildable Revision Test

Skip, not automatically bad.

---

# 276. Merge DAG Test

Strategy explicit.

---

# 277. Cost Limit Test

Bisect stops at bound.

---

# 278. Revert Experiment Test

Ephemeral source only.

---

# 279. Cross-Tenant Test

No failure detail leakage.

---

# 280. Debug Session Expiry Test

Access revoked.

---

# 281. Quarantined Runner Test

Not selected for normal reproduction.

---

# 282. Cache Disabled Exact-Repro Test

Actual execution occurs.

---

# 283. Nondeterminism Test

Same derivation/different output surfaced.

---

# 284. AI Advisory Test

No automatic state mutation.

---

# 285. DR Test

Derived clusters rebuild from retained observations.

---

# 286. Fuzzing

Fuzz:

```text
log normalizer
stack trace parser
diagnostic parser
symbol metadata
```

---

# 287. Property Tests

Same sanitized evidence -> same signature version output.

---

# 288. Scale Test

Millions of failure observations/clusters.

---

# 289. Failure Injection

```text
reproduction runner disappears
bisect worker crashes
symbolizer fails
CAS evidence missing
```

---

# 290. Implementation Phase 1 — Failure Model/Signature

Core evidence.

---

# 291. Phase 2 — Reproduction

Local/remote clean sandbox.

---

# 292. Phase 3 — Context Diff

Good-vs-bad.

---

# 293. Phase 4 — Failure Clustering

History/search.

---

# 294. Phase 5 — Bisect

First bad revision.

---

# 295. Phase 6 — Debug Sessions

Interactive clean environment.

---

# 296. Phase 7 — Infrastructure Correlation

Fleet/provider incidents.

---

# 297. Phase 8 — Performance/Finding Bisect

Broader predicates.

---

# 298. Phase 9 — Hypothesis Ranking

Rule/statistical.

---

# 299. Phase 10 — Reproduction Bundles/Air-Gap

Enterprise.

---

# 300. Phase 11 — Optional AI Assistance

Advisory only.

---

# 301. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 302. Acceptance Tests

1. Diagnostic state never rewrites canonical Run/Job outcome.
2. Failed attempt remains failed even if retry passes.
3. Failure signatures are sanitized and versioned.
4. Heuristic clustering is never presented as exact causality.
5. Infrastructure and workload failures are classified separately where evidence permits.
6. Unknown remains valid when evidence is insufficient.
7. Reproduction starts from immutable source/toolchain/config/input identity.
8. Failed sandboxes are not reused as default debug environment.
9. Secretful reproduction requires fresh authorization.
10. Secret values are never stored in reproduction bundles.
11. Reproduction fidelity is explicit.
12. Environment/toolchain/config/dependency differences are typed.
13. Known-good selectors resolve to exact historical runs.
14. Candidate differences are not automatically labeled root causes.
15. Bisect runs use normal Forgeyard execution semantics.
16. Bisect predicates are explicit.
17. Unbuildable revisions can be skipped.
18. Flaky bisect can become inconclusive rather than lying.
19. Bisect cost/run count is bounded.
20. Every tested revision maps to exact SourceSnapshotId.
21. Revert experiments use ephemeral snapshots and never mutate VCS automatically.
22. Debug sessions expire.
23. Debug tool privilege is policy-controlled.
24. Quarantined runners are not normal reproduction targets.
25. Cross-tenant diagnostic details remain isolated.
26. Widespread infrastructure clusters may be aggregated without source leakage.
27. Automated/AI hypotheses are advisory only.
28. Hypotheses expose evidence/confidence.
29. Search/analytics are derived.
30. Failure bundle exports are permission-gated/redacted.
31. Signature algorithms can reprocess history without rewriting observations.
32. Reproduction/bisect state survives worker restart.
33. Standalone/distributed share diagnostic semantics.
34. Air-gap reproduction works when required closure is bundled.
35. Forgeyard dogfoods diagnosis/bisect/reproduction on its own CI failures.

---

# 303. Production Readiness Gates

Do not call diagnosis architecture production-ready until:

```text
failure signatures are stable/sanitized
reproduction from immutable context works
secretful reproduction is safe
good-vs-bad context diff is exact
bisect handles flaky/unbuildable revisions honestly
debug sessions are isolated/expiring
cross-tenant isolation passes
failure-cluster reprocessing works
worker crash/restart tests pass
scale/fuzz tests pass
```

---

# 304. Architectural Invariants

1. diagnosis interprets evidence; it does not rewrite truth;
2. failure observations are immutable;
3. signatures are sanitized/versioned;
4. heuristic correlation is not causation;
5. unknown is preferable to false certainty;
6. reproduction uses immutable context;
7. failed sandboxes are not default debug environments;
8. secret values are never reused from failed attempts;
9. reproduction fidelity is explicit;
10. good/bad baselines resolve exactly;
11. candidate diffs are evidence, not automatic root cause;
12. bisect uses normal execution;
13. unbuildable points can be skipped;
14. flaky predicates can yield inconclusive results;
15. bisect campaigns are bounded;
16. debug access is privileged/expiring;
17. debug mutations never become trusted source automatically;
18. source experiments use ephemeral snapshots;
19. AI is optional/advisory only;
20. external AI egress is policy-controlled;
21. cross-tenant details are isolated;
22. infrastructure-wide clusters are privacy-safe aggregates;
23. diagnosis data obeys lifecycle policy;
24. security incidents override normal debug behavior where required;
25. search/analytics are derived;
26. signature/cluster algorithms can be reprocessed;
27. HA uses idempotency/reconciliation;
28. standalone/distributed share semantics;
29. diagnosis never bypasses policy/scheduler;
30. Forgeyard dogfoods its own diagnosis system.

---

# 305. Final Target Architecture

```text
                     Failed Attempt
                          │
                          ▼
                 Failure Observation
                          │
                          ▼
              Signature / Classification
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
           History      Reproduce    Compare
              │           │           │
              └───────────┼───────────┘
                          ▼
                       Bisect
                          │
                          ▼
                Evidence-Backed Hypotheses
                          │
                          ▼
                    Developer Decision
```

---

# 306. Final Architectural Position

Reproduction:

```text
failed Job/Attempt
+
exact SourceSnapshot
+
Job IR
+
toolchain
+
config
+
declared inputs
  ↓
new clean sandbox
  ↓
re-run
  ↓
same / different / pass / inconclusive
```

Differential diagnosis:

```text
known-good exact context
vs
known-bad exact context
  ↓
source/toolchain/config/dependency/platform diffs
  ↓
candidate causes
```

Bisect:

```text
known-good revision
+
known-bad revision
+
predicate
  ↓
bounded normal Forgeyard runs
  ↓
first-bad candidate
  ↓
confidence/evidence
```

The key guarantee is:

> **Forgeyard can make failures reproducible, comparable, and searchable without pretending that correlation is causation. It rebuilds diagnosis from immutable execution evidence, uses clean environments for reproduction, treats bisect as an experiment over exact source states, and exposes uncertainty whenever the available evidence cannot support a definitive root cause.**

---

# 307. Extended Architecture Sequence

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
```
