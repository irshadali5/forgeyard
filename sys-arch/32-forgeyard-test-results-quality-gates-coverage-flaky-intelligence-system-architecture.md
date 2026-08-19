# 32 — Forgeyard Test Results, Quality Gates, Coverage & Flaky-Test Intelligence System Architecture

**Document type:** Core Test Intelligence, Test Evidence, Coverage & Quality-Gate System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** normalized test-result ingestion, test-suite/test-case identity, JUnit/native report adapters, retries, flaky-test detection, test quarantine, test sharding, coverage ingestion, diff coverage, failure clustering, test-history analytics, quality-gate facts, test artifacts, and release/change-policy integration  
**Architecture style:** Test evidence is immutable and normalized; test execution remains ordinary Forgeyard jobs; quality decisions are deterministic policy decisions over exact evidence; retries never rewrite history; flaky-test intelligence is derived, explainable, and never allowed to silently turn a failing required test into success  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Pipeline IR, Run/Job/Attempt, Scheduler, Runner, CAS, Supply Chain, Change Proposal, Release, Device Lab, Observability, Search/Analytics, Policy/Authz, Audit, and Notifications. It formalizes testing as a first-class evidence subsystem without creating a second execution engine.

---

# 1. Purpose

CI/CD ultimately exists to answer questions such as:

```text
did the software build?
did tests pass?
which tests failed?
did the failure reproduce?
is the test flaky?
did coverage regress?
which lines changed without coverage?
is this failure new?
which platform/device is affected?
is the evidence sufficient to merge/release?
```

Without a dedicated test subsystem, these facts become scattered across:

```text
job exit codes
logs
JUnit XML
coverage files
screenshots
ad-hoc check summaries
```

The central rule is:

> **A Job executes tests, but a test result is structured evidence. Forgeyard preserves both the raw test report and a normalized immutable representation.**

A second rule is:

> **Retries create additional test observations; they never overwrite the original failure.**

A third rule is:

> **Flaky-test intelligence can explain, quarantine, or alter scheduling policy, but it cannot silently convert a required failing test into a passing policy decision.**

---

# 2. Architectural Position

```text
                     Test Job / Device Job
                             │
                             ▼
                      Raw Test Reports
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
           JUnit          Coverage       Screenshots/
          Native           Data           Diagnostics
              │              │              │
              └──────────────┼──────────────┘
                             ▼
                     Test Normalizer
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         Test Evidence   Coverage Fact   Failure Facts
              │              │              │
              └──────────────┼──────────────┘
                             ▼
                    Test Intelligence
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         Flaky Analysis   History       Quality Facts
                             │
                             ▼
                      Policy / Checks
```

---

# 3. Goals

The subsystem MUST:

1. define stable test identities;
2. define suites and cases;
3. ingest common test reports;
4. preserve raw reports;
5. normalize results;
6. support test retries;
7. distinguish execution retry from test-case retry;
8. support test history;
9. support flaky-test detection;
10. support quarantine;
11. support failure clustering;
12. support test sharding;
13. support coverage reports;
14. support line/function/branch coverage;
15. support diff coverage;
16. support coverage gates;
17. support platform/device dimensions;
18. support test artifacts;
19. support test duration analytics;
20. support quality-gate facts;
21. support Change Proposal checks;
22. support Release gates;
23. support search;
24. support notifications;
25. support audit for quarantine/admin actions;
26. support tenant isolation;
27. support HA;
28. support reprocessing;
29. support provider/framework adapters;
30. remain evidence-first.

---

# 4. Non-Goals

This subsystem does not:

```text
replace test frameworks
replace cargo test/pytest/JUnit/XCTest/etc.
replace job execution
replace scheduler
replace policy engine
declare merge/release authority itself
```

---

# 5. Workspace Structure

```text
crates/test/
├── forgeyard-test/
├── forgeyard-test-model/
├── forgeyard-test-report/
├── forgeyard-test-normalize/
├── forgeyard-test-ingest/
├── forgeyard-test-history/
├── forgeyard-test-flaky/
├── forgeyard-test-quarantine/
├── forgeyard-test-shard/
├── forgeyard-test-failure/
├── forgeyard-test-quality/
├── forgeyard-test-search/
├── forgeyard-test-health/
└── forgeyard-test-testkit/
```

Coverage:

```text
crates/coverage/
├── forgeyard-coverage/
├── forgeyard-coverage-model/
├── forgeyard-coverage-ingest/
├── forgeyard-coverage-normalize/
├── forgeyard-coverage-diff/
├── forgeyard-coverage-gate/
└── forgeyard-coverage-testkit/
```

Adapters:

```text
crates/test-adapters/
├── forgeyard-test-junit/
├── forgeyard-test-rust/
├── forgeyard-test-xctest/
├── forgeyard-test-android/
├── forgeyard-test-go/
├── forgeyard-test-python/
├── forgeyard-test-jvm/
└── ...
```

Use modules first; split only where dependency/runtime boundaries justify.

---

# 6. TestSuiteId

```rust
pub struct TestSuiteId(Digest);
```

---

# 7. TestCaseId

```rust
pub struct TestCaseId(Digest);
```

Stable logical identity.

---

# 8. Test Identity

Derived from normalized:

```text
project
test framework
suite namespace
case name/path/symbol
optional parameter identity
```

---

# 9. Native Test ID

Retain framework-native identifier separately.

---

# 10. Why Stable Identity Matters

Allows:

```text
history
flaky detection
duration prediction
failure clustering
quarantine
ownership
```

---

# 11. Test Case Name

Display metadata only.

---

# 12. Parameterized Tests

Parameter identity explicit.

---

# 13. Test Case Version

Test source can change.

---

# 14. TestDefinitionFingerprint

```rust
pub struct TestDefinitionFingerprint(Digest);
```

Can include:

```text
source file
symbol
test body/source digest where available
framework metadata
```

---

# 15. Why Fingerprint

Distinguish:

```text
same logical test after code change
```

from completely different test reusing a name.

---

# 16. TestObservationId

```rust
pub struct TestObservationId(Ulid);
```

One result for one test case in one attempt.

---

# 17. Test Observation

```rust
pub struct TestObservation {
    pub id: TestObservationId,
    pub test_case: TestCaseId,
    pub definition: Option<TestDefinitionFingerprint>,
    pub run: RunId,
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub execution: TestExecutionContext,
    pub outcome: TestOutcome,
    pub duration: Duration,
    pub evidence: TestEvidenceRefs,
}
```

---

# 18. Test Outcome

```rust
pub enum TestOutcome {
    Passed,
    Failed,
    Skipped,
    Ignored,
    TimedOut,
    Crashed,
    InfrastructureError,
    Unknown,
}
```

---

# 19. InfrastructureError

Distinct from test assertion failure.

---

# 20. Skipped vs Ignored

Keep framework semantics where possible.

---

# 21. Test Execution Context

```rust
pub struct TestExecutionContext {
    pub platform: PlatformDescriptor,
    pub runner: Option<RunnerId>,
    pub device: Option<DeviceId>,
    pub shard: Option<TestShardId>,
    pub retry_index: u16,
}
```

---

# 22. Test Evidence Refs

```rust
pub struct TestEvidenceRefs {
    pub raw_report: Option<CasObjectRef>,
    pub stdout: Option<CasObjectRef>,
    pub stderr: Option<CasObjectRef>,
    pub screenshots: Vec<CasObjectRef>,
    pub crash_reports: Vec<CasObjectRef>,
    pub attachments: Vec<CasObjectRef>,
}
```

---

# 23. Raw Report Preservation

Always retain according to policy.

---

# 24. Normalized Representation

Stored as metadata + canonical evidence object.

---

# 25. Raw Report Is Evidence

Not trusted blindly.

---

# 26. Parser

Framework adapter validates and normalizes.

---

# 27. Malformed Report

Test job may still be considered failed/invalid evidence according to policy.

---

# 28. Report Parser Limits

Bound:

```text
file size
XML depth
test count
attachment count
string length
```

---

# 29. XML Security

Disable external entities/DTD expansion.

---

# 30. JSON Reports

Bound depth/size.

---

# 31. JUnit Adapter

Maps:

```text
testsuite
testcase
failure
error
skipped
time
```

to Forgeyard model.

---

# 32. JUnit Is Not Semantically Uniform

Different frameworks emit variants.

Adapter must handle documented profiles.

---

# 33. Native Reports

Can preserve richer metadata.

---

# 34. Rust Testing

Adapters may consume:

```text
cargo test structured output where available
nextest-like reports
JUnit output
```

---

# 35. Device Testing

Part 20 results map to same test model.

---

# 36. XCTest

Maps suite/case/device artifacts.

---

# 37. Android Instrumentation

Same.

---

# 38. Test Ingestion Flow

```text
test process
  ↓
declared test report output
  ↓
runner uploads CAS
  ↓
job completion references report
  ↓
test ingest service
  ↓
normalized test evidence
```

---

# 39. Declared Report

Pipeline step should declare:

```text
format
path
required/optional
```

---

# 40. Pipeline IR Addition

```rust
pub struct TestReportDeclaration {
    pub format: TestReportFormat,
    pub path: RelativePath,
    pub required: bool,
}
```

---

# 41. Coverage Declaration

Similarly.

---

# 42. Test Job Success

Job exit code and report evidence both matter.

---

# 43. Example

Process exit 0 but required report missing:

```text
InvalidTestEvidence
```

policy can fail check.

---

# 44. Process Exit Nonzero With Valid Report

Normalized failed tests still preserved.

---

# 45. Test Summary

```rust
pub struct TestSummary {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub errors: u64,
    pub duration: Duration,
}
```

---

# 46. Job Test Result

Aggregates observations for exact job attempt.

---

# 47. Retry Semantics

Two kinds:

```text
JobAttempt retry
TestCase retry within same job
```

Keep distinct.

---

# 48. Job Retry

Creates new JobAttempt.

---

# 49. Test Retry

Creates additional TestObservation with same AttemptId and higher retry index.

---

# 50. Original Failure

Never deleted.

---

# 51. Final Retry Result

Presentation can show:

```text
Passed after 2 retries
```

but underlying evidence includes all failures.

---

# 52. Retry Policy

Explicit.

---

# 53. Test Retry Config

```rust
pub struct TestRetryPolicy {
    pub max_retries: u16,
    pub retry_on: BTreeSet<TestOutcomeClass>,
}
```

---

# 54. Retry Does Not Make Test Non-Flaky

A pass-after-failure is strong flaky signal.

---

# 55. Required Test Gate

Policy may define:

```text
all observations pass
final observation passes
no retry allowed
```

depending trust profile.

---

# 56. Recommended Protected Release

Do not treat pass-after-failure exactly same as clean pass.

---

# 57. Test Stability State

```rust
pub enum TestStability {
    Stable,
    SuspectedFlaky,
    Flaky,
    Quarantined,
    Unknown,
}
```

---

# 58. Flaky Definition

A test producing inconsistent results under sufficiently similar conditions.

---

# 59. Flaky Detection Inputs

```text
recent outcomes
retry transitions
platform
definition fingerprint
source snapshot
runner/device class
failure signature
```

---

# 60. Avoid Naive Global Flaky Label

A test can be flaky only on:

```text
Windows
Android device model
specific toolchain
```

---

# 61. Stability Scope

```rust
pub struct TestStabilityScope {
    pub test: TestCaseId,
    pub platform: Option<PlatformClass>,
    pub device_class: Option<DeviceClass>,
}
```

---

# 62. Flaky Score

```rust
pub struct FlakyScore {
    pub probability: Ratio,
    pub sample_count: u32,
    pub confidence: ConfidenceLevel,
}
```

---

# 63. Explainability

Store inputs/summary.

---

# 64. No ML Requirement

Start deterministic/statistical.

---

# 65. Baseline Flaky Rules

Examples:

```text
fail then pass on retry
pass/fail oscillation in recent equivalent runs
same source snapshot produces different outcomes
```

---

# 66. Infrastructure Failure Exclusion

Do not mark test flaky because runner disconnected.

---

# 67. Environmental Failure

Classify separately.

---

# 68. FailureSignatureId

```rust
pub struct FailureSignatureId(Digest);
```

---

# 69. Failure Signature

Derived from normalized safe failure metadata.

---

# 70. Inputs

Potential:

```text
assertion type
top stack frames
error code
test ID
crash signal
```

---

# 71. No Raw Secret Text

Before signature normalize/redact.

---

# 72. Failure Clustering

Group likely same root failure.

---

# 73. Deterministic Baseline

Hash normalized failure signature.

---

# 74. Similarity Later

Optional fuzzy clustering.

---

# 75. Failure Cluster

```rust
pub struct FailureCluster {
    pub id: FailureClusterId,
    pub signature: FailureSignatureId,
    pub first_seen: Timestamp,
    pub last_seen: Timestamp,
    pub count: u64,
}
```

---

# 76. New Failure

Can highlight first-seen on current Change Proposal.

---

# 77. Existing Known Failure

Still failure unless quarantine/policy says otherwise.

---

# 78. Test Quarantine

Administrative/policy object.

---

# 79. TestQuarantineId

```rust
pub struct TestQuarantineId(Ulid);
```

---

# 80. Quarantine Record

```rust
pub struct TestQuarantine {
    pub id: TestQuarantineId,
    pub test: TestCaseId,
    pub scope: TestStabilityScope,
    pub reason: BoundedString,
    pub created_by: PrincipalId,
    pub expires_at: Timestamp,
    pub tracking_ref: Option<ExternalTrackingRef>,
}
```

---

# 81. Quarantine Must Expire

Strong recommendation.

---

# 82. Permanent Quarantine

Discouraged; explicit renewal required.

---

# 83. Quarantine Permission

```text
test.quarantine
```

---

# 84. Quarantine Audit

Mandatory.

---

# 85. Quarantine Does Not Change Observation

Test still records Failed.

---

# 86. Quality Evaluation

Can classify:

```text
failed-but-quarantined
```

---

# 87. Quarantine Policy

A policy may allow merge despite quarantined failure.

---

# 88. Release Policy

May be stricter than Change Proposal policy.

---

# 89. Example

```text
PR:
quarantined flaky failure allowed with warning

Stable release:
no quarantined failures allowed
```

---

# 90. Quarantine Expiry

Durable timer/reevaluation.

---

# 91. Owner

Optional test ownership mapping.

---

# 92. Test Owner

Could come from:

```text
CODEOWNERS-like source ownership
test metadata
explicit mapping
```

---

# 93. Failure Notification

Notify owner when:

```text
new flaky
quarantine expires
repeated failure
```

---

# 94. Test History

Query by TestCaseId.

---

# 95. Test History Record

Derived from observations.

---

# 96. History Dimensions

```text
source snapshot
branch/change
platform
runner/device class
duration
outcome
failure signature
```

---

# 97. History Retention

Configurable.

---

# 98. Raw Evidence Retention

May be shorter than normalized summary.

---

# 99. Test Search

Part 31 indexes:

```text
test ID
suite
name
stability
failure cluster
```

---

# 100. Test Analytics

Examples:

```text
pass rate
flake rate
p50/p95 duration
failure recurrence
```

---

# 101. Duration History

Used for sharding.

---

# 102. Test Sharding

Goal:

```text
split suite across N jobs with balanced predicted duration
```

---

# 103. ShardPlanId

```rust
pub struct TestShardPlanId(Digest);
```

---

# 104. Shard Plan

```rust
pub struct TestShardPlan {
    pub suite: TestSuiteId,
    pub shard_count: NonZeroU16,
    pub assignments: Vec<TestShardAssignment>,
    pub history_snapshot: TestHistorySnapshotId,
}
```

---

# 105. Deterministic Plan

Same inputs/history snapshot -> same assignments.

---

# 106. History Snapshot

Immutable statistics snapshot.

---

# 107. Unknown Tests

Assign with fallback duration.

---

# 108. Sharding Strategy

Baseline:

```text
longest-processing-time greedy bin packing
```

simple and deterministic.

---

# 109. No Hidden Adaptive Scheduler

Shard plan compiled before execution.

---

# 110. Dynamic Work Stealing

Future option, but complicates reproducibility.

---

# 111. Shard Identity

Each shard is ordinary Job.

---

# 112. Fail-Fast

Pipeline can cancel remaining shards.

---

# 113. Re-run Failed Tests

Explicit separate job/test retry semantics.

---

# 114. Shard Cache

Plan can cache by test inventory + history snapshot.

---

# 115. Test Discovery

How know test list before execution?

Options:

```text
static manifest
framework discovery command
historical inventory
```

---

# 116. Discovery Job

Can run before sharded jobs.

---

# 117. TestInventoryId

```rust
pub struct TestInventoryId(Digest);
```

---

# 118. Test Inventory

Immutable list of TestCaseIds.

---

# 119. Discovery Evidence

Stored in CAS.

---

# 120. Discovery Network

Normal sandbox policy.

---

# 121. Coverage

First-class evidence.

---

# 122. CoverageReportId

```rust
pub struct CoverageReportId(Ulid);
```

---

# 123. Coverage Kinds

```rust
pub enum CoverageKind {
    Line,
    Branch,
    Function,
    Region,
}
```

---

# 124. Coverage Summary

```rust
pub struct CoverageSummary {
    pub lines: Option<CoverageRate>,
    pub branches: Option<CoverageRate>,
    pub functions: Option<CoverageRate>,
}
```

---

# 125. CoverageRate

```rust
pub struct CoverageRate {
    pub covered: u64,
    pub total: u64,
}
```

---

# 126. Do Not Store Only Percentage

Counts matter.

---

# 127. Raw Coverage Formats

Potential:

```text
LCOV
Cobertura XML
JaCoCo XML
LLVM coverage
Go coverprofile
Istanbul/NYC JSON
```

---

# 128. Raw Report

CAS.

---

# 129. Normalized Coverage Model

Path-based/source-snapshot-bound.

---

# 130. Coverage File Record

```rust
pub struct FileCoverage {
    pub source_path: RepoRelativePath,
    pub line_hits: BTreeMap<LineNumber, HitCount>,
    pub branch: Option<BranchCoverage>,
    pub function: Option<FunctionCoverage>,
}
```

---

# 131. SourceSnapshot Binding

Critical.

Coverage belongs to exact source snapshot.

---

# 132. Path Normalization

Strip controlled workspace prefix.

---

# 133. Reject Path Escape

Critical.

---

# 134. Generated Files

Can mark/exclude explicitly.

---

# 135. Coverage Merge

Multiple shards/platform jobs can merge only if compatible.

---

# 136. Merge Inputs

Must share:

```text
SourceSnapshotId
coverage schema/tool semantics
source mapping
```

---

# 137. Duplicate Hit Semantics

Format-specific normalization.

---

# 138. CoverageMergeId

Content-derived.

---

# 139. Overall Coverage

Aggregate.

---

# 140. Diff Coverage

Coverage over lines changed between exact base and proposal/source.

---

# 141. Diff Coverage Inputs

```text
base SourceSnapshotId
head SourceSnapshotId
change diff
head coverage report
```

---

# 142. Diff Identity

Exact.

---

# 143. DiffCoverageResult

```rust
pub struct DiffCoverageResult {
    pub changed_executable_lines: u64,
    pub covered_changed_lines: u64,
    pub uncovered_lines: Vec<SourceLineRef>,
}
```

---

# 144. Renames

Use VCS-neutral tree/diff semantics.

---

# 145. New Files

Included.

---

# 146. Deleted Lines

Not coverage-required.

---

# 147. Non-Executable Lines

Excluded according to coverage mapping.

---

# 148. Coverage Gate

Examples:

```text
overall lines >= 80%
diff lines >= 90%
no regression > 1%
```

---

# 149. Quality Fact

Coverage subsystem emits normalized fact.

---

# 150. Policy Engine Decides

Critical.

---

# 151. QualityFact

```rust
pub enum QualityFact {
    TestSummary(TestSummaryFact),
    FlakyTests(FlakyTestFact),
    Coverage(CoverageFact),
    DiffCoverage(DiffCoverageFact),
    FailureClusters(FailureClusterFact),
}
```

---

# 152. Quality Evaluation

```rust
pub struct QualityEvaluation {
    pub subject: QualitySubject,
    pub facts: Vec<QualityFactRef>,
    pub policy_digest: PolicyDigest,
    pub decision: QualityDecision,
}
```

---

# 153. Quality Subject

```text
Run
ChangeProposalRevision
ReleaseCandidate
PackageSet
```

---

# 154. Decision

```rust
pub enum QualityDecision {
    Pass,
    PassWithWarnings,
    Fail,
    Incomplete,
    Stale,
}
```

---

# 155. Incomplete

Required report missing.

---

# 156. Stale

Evidence belongs to wrong source revision/candidate.

---

# 157. Exact Evidence Binding

Never apply test evidence from old ProposalRevision to new source.

---

# 158. Change Proposal

Checks bind exact ProposalRevisionId + SourceSnapshotId.

---

# 159. Release

Release gate binds exact ReleaseCandidateId/PackageSet/source.

---

# 160. Reuse Test Evidence

Only if evidence subject equivalence is proven.

---

# 161. Example

Same exact SourceSnapshotId + PipelinePlanId + relevant test environment may reuse.

---

# 162. Do Not Reuse Branch-Based

Critical.

---

# 163. Test Result Publication

SCM check summary.

---

# 164. Summary

```text
1,203 passed
3 failed
2 flaky/quarantined
coverage 87.4%
diff coverage 93.1%
```

---

# 165. Provider Annotation Limit

Only top failures; link to Forgeyard for full details.

---

# 166. Dioxus UI

Test pages:

```text
Tests
Suites
Failures
Flaky Tests
Quarantines
Coverage
Quality Gates
```

---

# 167. Run Test Tab

Shows:

```text
summary
failed tests
slow tests
flaky signals
artifacts
coverage
```

---

# 168. Test Detail

Tabs:

```text
History
Failures
Platforms
Duration
Quarantine
Evidence
```

---

# 169. Failure Detail

Show:

```text
message
safe stack
signature
first/last seen
affected platforms
attachments
```

---

# 170. Stack Trace

Sanitized/escaped.

---

# 171. Source Link

Exact source snapshot/revision.

---

# 172. Coverage UI

File tree + line coverage.

---

# 173. Source Rendering

Exact source snapshot.

---

# 174. Coverage Overlay

Derived.

---

# 175. Diff Coverage UI

Highlight changed uncovered lines.

---

# 176. Accessibility

Coverage status not color-only.

---

# 177. Flaky Dashboard

Sort by:

```text
flake probability
recent occurrences
affected runs
duration cost
```

---

# 178. Quarantine Dashboard

Shows expiry/owner/reason.

---

# 179. Quarantine Action

High impact; authz/audit.

---

# 180. Test Retry UI

Can request re-run through normal Run/Job API.

---

# 181. UI Does Not Fabricate Retry

Server creates explicit new execution.

---

# 182. Search Integration

Queries:

```text
test:foo stability:flaky
failure:signature platform:windows
coverage:<80%
```

---

# 183. Notifications

Examples:

```text
new flaky test
quarantine expiring
coverage gate failed
new failure cluster
```

---

# 184. Audit

Audit:

```text
quarantine create/update/remove
quality policy change
manual test override
```

---

# 185. Manual Override

If allowed, explicit PolicyException.

---

# 186. Never Edit Test Result

Override does not mutate evidence.

---

# 187. Override Record

Binds exact:

```text
quality subject
failed fact
reason
principal
expiry/scope
```

---

# 188. Release Override

Stricter permission.

---

# 189. Test Ownership

Could integrate source ownership.

---

# 190. Automatic Owner Notification

Optional.

---

# 191. Test Cost

Duration × resources.

---

# 192. Slow Test

Derived threshold.

---

# 193. Slow-Test Analytics

Can recommend sharding.

---

# 194. Test Impact Analysis

Future.

---

# 195. Goal

Run subset of tests likely affected by change.

---

# 196. Baseline

Do not make correctness depend on impact analysis initially.

---

# 197. Safe Use

Optimization for non-required tests or combined with mandatory baseline suite.

---

# 198. Impact Evidence

Potential dependencies:

```text
source graph
historical test-file correlations
coverage maps
```

---

# 199. No Opaque AI Skip

Critical.

---

# 200. Required Tests

Policy enumerates mandatory suite regardless prediction.

---

# 201. Test Selection Plan

If introduced:

```rust
pub struct TestSelectionPlan {
    pub inventory: TestInventoryId,
    pub selected: BTreeSet<TestCaseId>,
    pub mandatory: BTreeSet<TestCaseId>,
    pub rationale: TestSelectionRationale,
}
```

---

# 202. Selection Plan Immutable

Evidence.

---

# 203. Quality Gate Policy

Examples:

```text
required suites
allowed skips
max flaky quarantines
coverage thresholds
retry tolerance
platform matrix
```

---

# 204. Policy Remains Part 11

No second policy language.

---

# 205. Quality Rule Input

Typed facts.

---

# 206. Example Rule

```text
required suite "unit" == Pass
AND
diff_coverage >= 0.90
AND
unquarantined_failures == 0
```

---

# 207. Required Suite Identity

Stable TestSuiteId/profile, not display string only.

---

# 208. Test Profile

```rust
pub struct TestProfileId(Digest);
```

Defines expected test/report semantics.

---

# 209. Test Environment Identity

Include:

```text
toolchain
platform
executor
environment profile
```

where quality comparison depends.

---

# 210. Historical Comparison

Compare like-for-like.

---

# 211. Performance Regression of Tests

Can flag dramatic suite-duration changes.

---

# 212. Not Benchmark Authority

Dedicated benchmarks could be later subsystem.

---

# 213. Test Result Storage

Metadata:

```text
test_cases
test_observations
test_summaries
failure_signatures
failure_clusters
quarantines
test_inventories
shard_plans
coverage_reports
coverage_summaries
quality_evaluations
```

---

# 214. CAS

Stores:

```text
raw reports
normalized report blobs
coverage maps
screenshots
crash reports
attachments
```

---

# 215. Indexing

Part 31 derived.

---

# 216. Observation Immutability

Once committed, no update to outcome.

---

# 217. Parser Bug

Reprocessing creates new normalized interpretation version.

---

# 218. NormalizerVersion

```rust
pub struct TestNormalizerVersion(u16);
```

---

# 219. Reprocessing

Original raw report retained.

---

# 220. NormalizedRevision

```rust
pub struct TestNormalizationRevisionId(Digest);
```

---

# 221. Current Interpretation

Pointer/view can change to newer normalization.

---

# 222. Historical Transparency

Keep parser version.

---

# 223. Coverage Normalizer Version

Same principle.

---

# 224. Quality Re-Evaluation

New policy can evaluate old immutable evidence.

---

# 225. Historical Policy Decision

Preserve original decision + PolicyDigest.

---

# 226. New Decision

Separate reevaluation record.

---

# 227. Test Evidence Freshness

Evidence may be stale when:

```text
source changed
test definition changed
environment requirement changed
```

---

# 228. Quality Cache

Key includes:

```text
subject
evidence bundle
PolicyDigest
normalizer version
```

---

# 229. Test Ingestion Idempotency

Key:

```text
AttemptId + report declaration + report digest
```

---

# 230. Duplicate Upload

No duplicate observations.

---

# 231. Partial Report

Explicit.

---

# 232. Sharded Aggregation

Wait for required shard set.

---

# 233. Missing Shard

Quality `Incomplete`.

---

# 234. Shard Retry

New attempt, preserves old shard evidence.

---

# 235. Aggregate Selection

Use accepted/latest valid attempt according to Run/Job state machine.

---

# 236. Do Not Mix Attempts Accidentally

Critical.

---

# 237. Device Matrix

Same exact aggregation rules.

---

# 238. Test Artifact Retention

Failures may retain longer than passes.

---

# 239. Screenshots

Potentially sensitive.

---

# 240. Crash Dumps

Potentially highly sensitive.

---

# 241. Classification

Use artifact classification.

---

# 242. Tenant Isolation

Every observation/evidence inherits tenant via Project/Run.

---

# 243. Cross-Tenant Test History

Forbidden.

---

# 244. Shared OSS Cache

Does not imply shared test history.

---

# 245. API

Potential:

```text
GET  /v1/runs/{id}/tests
GET  /v1/tests/{test_case_id}
GET  /v1/tests/{test_case_id}/history
GET  /v1/runs/{id}/coverage
GET  /v1/change-proposals/{id}/quality
GET  /v1/releases/{id}/quality
GET  /v1/flaky-tests
POST /v1/tests/{id}/quarantine
DELETE /v1/test-quarantines/{id}
```

---

# 246. Test Evidence Download

Normal artifact authz.

---

# 247. Permissions

```text
test.read
test.retry
test.quarantine
test.override
coverage.read
quality.read
quality.override
```

---

# 248. Retry Permission

Creates new run/job action.

---

# 249. Quarantine Permission

Scoped.

---

# 250. Quality Override

High privilege.

---

# 251. Test Parser Security

Reports are untrusted input.

---

# 252. XML XXE

Disabled.

---

# 253. Zip Bomb

If report archive supported, bound decompression.

---

# 254. Attachment Path

Never trust raw path for filesystem.

---

# 255. HTML Failure Messages

Render as text/sanitized.

---

# 256. ANSI

Strip/escape in structured display.

---

# 257. Stack URLs

Do not auto-fetch arbitrary URLs.

---

# 258. Coverage Path

Repo-relative validated.

---

# 259. Malicious Report Count

Bound max test cases per report.

---

# 260. Oversize Failure Text

Truncate safely, raw report in CAS subject to max.

---

# 261. Test Command

Still sandboxed as normal job.

---

# 262. Parser Runs

Can run in control-plane safe parser process/crate with strict limits.

---

# 263. Very Complex Formats

Sandbox parser worker optional.

---

# 264. Plugin Test Adapter

Part 24 possible.

---

# 265. Third-Party Parser

Sandboxed.

---

# 266. Plugin Output

Normalized intermediate, host validates bounds.

---

# 267. Quality Fact Provenance

Records parser/plugin version.

---

# 268. Observability Metrics

```text
test_observations_total
test_failures_total
test_flaky_detected_total
test_quarantined_total
test_ingest_failures_total
test_ingest_latency_seconds
coverage_reports_total
quality_gate_failures_total
test_shard_imbalance_ratio
```

---

# 269. Metric Labels

Low-cardinality:

```text
outcome
framework
platform_class
quality_result
```

---

# 270. No TestCaseId Metric Label

Use analytics/search.

---

# 271. Tracing

```text
test.ingest
test.normalize
test.aggregate
test.flaky.evaluate
test.shard.plan
coverage.ingest
coverage.diff
quality.evaluate
```

---

# 272. Health

Checks:

```text
ingestion backlog
parser errors
quality evaluator
history projector
```

---

# 273. Doctor

```text
forgeyard test doctor
```

---

# 274. Doctor Checks

```text
report parser registry
coverage adapters
stuck ingestion
quarantine expiry timers
history lag
```

---

# 275. CLI

```text
forgeyard test list
forgeyard test failures
forgeyard test history <id>
forgeyard test flaky
forgeyard test quarantine
forgeyard coverage show
forgeyard coverage diff
forgeyard quality explain
```

---

# 276. `quality explain`

Shows:

```text
required facts
actual values
policy digest
failed rules
```

---

# 277. No Hidden Quality Score

Prefer explicit facts/rules.

---

# 278. Optional Composite Score

Presentation only.

---

# 279. Quality Decision Authority

Central policy/check service.

---

# 280. SCM Provider Check

One or multiple checks:

```text
Tests
Coverage
Quality Gate
```

---

# 281. Check Identity

Trusted Forgeyard origin.

---

# 282. Release Candidate

Can require quality evidence bundle.

---

# 283. Supply Chain Integration

Test/coverage evidence can join EvidenceBundle.

---

# 284. Provenance

Build provenance can reference test evidence performed on artifact/source.

---

# 285. Test Evidence vs Artifact Evidence

Some tests validate source/build output.

Bind exact subjects.

---

# 286. Binary Test

May bind package/artifact digest.

---

# 287. Device Install Test

Bind exact APK/IPA artifact digest + DeviceId/class.

---

# 288. Release Smoke Test

Bind exact package/release candidate.

---

# 289. EvidenceSubject

Reuse Part 13 model if possible.

---

# 290. TestEvidenceKind

Extend evidence taxonomy rather than parallel evidence system.

---

# 291. Quality Bundle

```rust
pub struct QualityEvidenceBundle {
    pub id: QualityEvidenceBundleId,
    pub subject: EvidenceSubject,
    pub test_summaries: Vec<TestSummaryRef>,
    pub coverage: Vec<CoverageReportRef>,
    pub flaky_snapshot: FlakySnapshotId,
}
```

---

# 292. Flaky Snapshot

Exact state at evaluation time.

---

# 293. Why Snapshot

A test can be quarantined later; historical decision should show what was known then.

---

# 294. Quarantine Change

Does not retroactively rewrite previous quality decision.

---

# 295. Re-evaluation

Explicit new record.

---

# 296. Test Baseline

For regression:

```text
target branch/source snapshot historical baseline
```

---

# 297. Baseline Identity

Exact snapshot or rolling reference resolved at evaluation.

---

# 298. Coverage Regression

Compare same metric/tool semantics.

---

# 299. Baseline Missing

Quality fact `Unknown/Incomplete`.

---

# 300. No Fake 0%/100%

Critical.

---

# 301. Failure First-Seen

Compare against configured history horizon.

---

# 302. New vs Existing Failure

Presentation/policy fact.

---

# 303. Known Flaky Failure

Still evidence.

---

# 304. Auto-Quarantine

Not baseline.

---

# 305. Recommendation

Detection may suggest quarantine, but human/policy approval creates quarantine.

---

# 306. If Auto-Quarantine Later

Require strict policy, TTL, audit, max scope.

---

# 307. Auto-Unquarantine

Possible after sustained clean passes, but explicit policy.

---

# 308. Baseline

Expiry + human renewal.

---

# 309. Test Ownership Assignment

Can be derived, but owner mapping is not security authority.

---

# 310. Notifications to Owner

Best effort.

---

# 311. Test Inventory Drift

If a required test disappears unexpectedly:

```text
quality warning/failure
```

---

# 312. Removed Test

Could be legitimate source change.

---

# 313. Required Test Manifest

Policy can pin required named tests/suites.

---

# 314. Silent Test Deletion Defense

Compare inventory against policy/baseline.

---

# 315. Test Count Regression

Optional quality fact.

---

# 316. Example

```text
suite dropped from 1200 to 300 tests
```

suspicious.

---

# 317. Quality Gate Can Require Minimum Inventory

Explicit.

---

# 318. Skipped Test Increase

Quality fact.

---

# 319. Timeout Rate

Quality fact.

---

# 320. Flaky Budget

Policy may allow max N quarantined flakes.

---

# 321. Retry Budget

Policy may cap retries.

---

# 322. Test Debt

Derived analytics:

```text
quarantined tests
age
failure frequency
```

---

# 323. No Commercial Coupling

Part 30 may gate advanced analytics, but core correctness remains.

---

# 324. Search/Analytics Integration

Part 31 stores derived projections.

---

# 325. DR

Test normalized metadata in PostgreSQL backup.

---

# 326. Raw reports in CAS backup according to retention.

---

# 327. Rebuild

Test history analytics can rebuild from observations.

---

# 328. Flaky State

Derived/rebuildable from retained observations + quarantine records.

---

# 329. Quarantine

Authoritative metadata; must backup.

---

# 330. Coverage Summary

Can recompute from normalized/raw coverage if retained.

---

# 331. Quality Decisions

Historical evidence; retain.

---

# 332. HA

Ingest workers can run multiple replicas.

---

# 333. Claim Work

Idempotent job/report key.

---

# 334. Duplicate Parser

Same normalized revision id prevents duplicates.

---

# 335. Ingest Outage

Job completion remains authoritative, but quality evaluation is `Incomplete` until reports processed.

---

# 336. Protected Merge/Release

Waits for quality evidence when required.

---

# 337. Test Ingest Lag

Visible.

---

# 338. Testkit

```text
forgeyard-test-testkit/src/
├── lib.rs
├── report.rs
├── observation.rs
├── flaky.rs
├── quarantine.rs
├── shard.rs
├── failure.rs
└── assertions.rs
```

Coverage:

```text
forgeyard-coverage-testkit/src/
```

---

# 339. Unit Tests

Test identity normalization.

---

# 340. JUnit Variant Tests

Multiple emitters.

---

# 341. XXE Test

External entity rejected.

---

# 342. Oversize Report Test

Bounded.

---

# 343. Retry History Test

Failure remains after retry pass.

---

# 344. Flaky Detection Test

Fail/pass oscillation flagged.

---

# 345. Infrastructure Exclusion Test

Runner loss does not mark test flaky.

---

# 346. Platform-Specific Flake Test

Windows flake does not automatically mark Linux.

---

# 347. Quarantine Test

Observation remains failed.

---

# 348. Quarantine Expiry Test

No longer suppresses quality rule after expiry.

---

# 349. Quality Snapshot Test

Later quarantine change does not mutate old decision.

---

# 350. Shard Determinism Test

Same inventory/history snapshot -> same plan.

---

# 351. Shard Balance Test

Historical duration improves balance.

---

# 352. Coverage Path Escape Test

Rejected.

---

# 353. Diff Coverage Test

Changed executable lines computed correctly.

---

# 354. Source Snapshot Mismatch Test

Coverage rejected as stale for proposal.

---

# 355. Missing Report Test

Required evidence -> Incomplete.

---

# 356. Duplicate Ingest Test

No duplicate observations.

---

# 357. Parser Reprocess Test

New normalization revision preserves old interpretation.

---

# 358. Cross-Tenant Test

No test history leakage.

---

# 359. Secret Leakage Test

Failure/stack/report projection redacted.

---

# 360. Release Gate Test

Old proposal test result cannot satisfy new candidate.

---

# 361. Device Evidence Test

Exact installed artifact/device context preserved.

---

# 362. Load Test

Millions of test observations.

---

# 363. Flaky Analytics Scale

Large history bounded.

---

# 364. Fuzzing

Fuzz:

```text
JUnit/XML parsers
coverage parsers
failure normalizer
```

---

# 365. Failure Injection

```text
CAS report missing
parser crash
DB restart
duplicate event
ingest worker loss
```

---

# 366. Implementation Phase 1 — Test Model/Ingestion

TestSuite/TestCase/TestObservation/JUnit.

---

# 367. Phase 2 — Run UI/Test Summaries

First-class results.

---

# 368. Phase 3 — History/Failure Signatures

Recurring failure intelligence.

---

# 369. Phase 4 — Flaky Detection

Deterministic/statistical.

---

# 370. Phase 5 — Quarantine

TTL/audit/policy.

---

# 371. Phase 6 — Coverage

LCOV/Cobertura/LLVM adapters.

---

# 372. Phase 7 — Diff Coverage

Change Proposal integration.

---

# 373. Phase 8 — Quality Facts/Gates

Policy integration.

---

# 374. Phase 9 — Sharding

Historical balancing.

---

# 375. Phase 10 — Device/Platform Test Unification

Android/iOS.

---

# 376. Phase 11 — Search/Analytics/Notifications

Operational polish.

---

# 377. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 378. Acceptance Tests

1. Test execution remains a normal Forgeyard Job.
2. Test reports are structured immutable evidence.
3. Raw reports are preserved in CAS according to retention.
4. Test identity is stable and framework-neutral.
5. Parameterized test identity is explicit.
6. Test observations are immutable.
7. Job retries create new Attempt-linked observations.
8. Test-case retries preserve original failures.
9. Pass-after-retry is visible and contributes to flaky detection.
10. Infrastructure failures are not misclassified as test flakes.
11. Flaky analysis can be platform/device scoped.
12. Quarantine never rewrites a failed observation.
13. Quarantine is scoped, reasoned, expiring, and audited.
14. Protected release policy can be stricter than PR policy.
15. Quality decisions bind exact source/artifact/release subject.
16. Old evidence cannot satisfy a changed ProposalRevision.
17. Missing required reports produce Incomplete, not false Pass.
18. Parser errors do not silently discard failures.
19. JUnit/XML parsing is protected against XXE/entity expansion.
20. Coverage is bound to exact SourceSnapshotId.
21. Diff coverage uses exact base/head snapshots.
22. Coverage path traversal is rejected.
23. Coverage stores counts, not percentage only.
24. Shard plans are deterministic for a fixed history snapshot.
25. Sharding jobs remain ordinary scheduler Jobs.
26. Test inventory disappearance can be detected by policy.
27. Failure signatures are generated only from sanitized data.
28. Test/coverage evidence is tenant isolated.
29. Quality override never edits evidence and is separately audited.
30. SCM test status publishes exact revision-bound results.
31. Release evidence can include test/coverage quality bundle.
32. Ingestion can recover from duplicate/dropped events.
33. Parser upgrades create new interpretation revisions instead of rewriting history.
34. Standalone/distributed share test evidence semantics.
35. Forgeyard dogfoods the test-intelligence system for its own CI/release.

---

# 379. Production Readiness Gates

Do not call test intelligence production-ready until:

```text
normalized test model is stable
JUnit/native ingestion is bounded and secure
retry history preserves failures
test/infra failure classification is reliable
flaky detection is explainable
quarantine TTL/audit works
coverage source binding is exact
diff coverage is verified
quality decisions bind immutable subjects
tenant/secret leakage tests pass
ingestion/reconciliation survives worker failure
```

---

# 380. Architectural Invariants

1. test execution is not a separate execution engine;
2. test evidence is immutable;
3. raw reports are retained as evidence where policy requires;
4. retries never erase failures;
5. infrastructure failures are distinct from assertion failures;
6. flaky status is derived, not an edit to evidence;
7. quarantine never turns a failed observation into Passed;
8. quarantine is explicit, scoped, expiring, and audited;
9. policy decides whether quarantined failures are acceptable;
10. quality decisions bind exact immutable subjects;
11. stale test evidence cannot satisfy new source revisions;
12. required missing evidence yields Incomplete;
13. coverage binds exact SourceSnapshotId;
14. diff coverage binds exact base/head snapshots;
15. path normalization prevents traversal;
16. coverage percentages derive from covered/total counts;
17. shard plans are deterministic for fixed inputs;
18. sharded tests remain normal Jobs;
19. failure signatures are sanitized;
20. test evidence is tenant scoped;
21. quality overrides never mutate evidence;
22. parser/normalizer versions are recorded;
23. reprocessing creates new normalization revision;
24. historical policy decisions remain immutable;
25. search/analytics are derived from test evidence;
26. SCM checks do not become quality authority;
27. Release gates use exact evidence bundle;
28. ingestion is idempotent/reconcilable;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its own test intelligence and quality system.

---

# 381. Final Target Architecture

```text
                     Test Execution
                          │
                          ▼
                    Raw Reports
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
           Tests       Coverage     Diagnostics
             │            │            │
             └────────────┼────────────┘
                          ▼
                  Normalized Evidence
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          History       Flaky       Failure
                       Analysis     Clusters
             │            │            │
             └────────────┼────────────┘
                          ▼
                    Quality Facts
                          │
                          ▼
                  Policy Evaluation
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
         Change Gate   Release Gate    UI/SCM
```

---

# 382. Final Architectural Position

Test retry:

```text
failure observation
  ↓
retry
  ↓
pass observation
  ↓
both preserved
  ↓
flaky signal
```

Coverage:

```text
exact SourceSnapshotId
+
normalized coverage
+
exact base/head diff
  ↓
diff coverage fact
  ↓
policy
```

Quality:

```text
exact subject
+
immutable test evidence
+
coverage evidence
+
flaky/quarantine snapshot
+
PolicyDigest
  ↓
Pass / Warning / Fail / Incomplete / Stale
```

The key guarantee is:

> **Forgeyard treats testing as evidence rather than a single green/red exit code. Every failure, retry, flaky signal, quarantine, coverage fact, and quality decision remains traceable to exact immutable execution and source identities, so teams can improve test reliability without weakening the correctness of merge and release gates.**

---

# 383. Extended Architecture Sequence

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
```
