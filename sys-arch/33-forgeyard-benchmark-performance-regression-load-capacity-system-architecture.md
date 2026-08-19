# 33 — Forgeyard Benchmarking, Performance Regression, Load-Test & Capacity Intelligence System Architecture

**Document type:** Core Benchmark, Performance Evidence, Regression Detection, Load/Stress/Soak Testing & Capacity Intelligence System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** microbenchmarks, macrobenchmarks, application benchmarks, load tests, stress tests, soak tests, performance baselines, statistical regression analysis, hardware normalization, environment control, binary-size/resource regressions, performance budgets, capacity tests, benchmark history, and policy/release integration  
**Architecture style:** Evidence-first, hardware-aware, statistically explicit, reproducibility-conscious, immutable benchmark observations, exact source/artifact binding, explainable regression decisions, no single noisy sample authority, and no hidden performance-policy bypass  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Run/Job/Attempt, Scheduler, Runner, Sandbox/Executor, Test Intelligence, Search/Analytics, Supply Chain, Change Proposal, Release, Deployment, Device Lab, Observability, Policy, and Multi-Tenancy. This subsystem makes performance a first-class quality-evidence domain without creating a second execution engine.

---

# 1. Purpose

CI/CD quality is not only:

```text
does it compile?
do tests pass?
```

Production software also needs answers to:

```text
did latency regress?
did throughput improve?
did memory use increase?
did CPU cost change?
did binary size grow?
did startup time regress?
does this release survive sustained load?
what is the saturation point?
did Android performance change on real hardware?
is the new result real or just noise?
```

The central rule is:

> **Performance results are immutable evidence bound to exact code, artifacts, hardware class, environment, and benchmark definition.**

A second rule is:

> **A benchmark comparison is meaningful only when Forgeyard can explain the baseline, environment comparability, sample count, statistical method, and uncertainty.**

A third rule is:

> **A single faster/slower measurement never silently becomes merge or release authority. Policy evaluates explicit performance facts and confidence.**

---

# 2. Architectural Position

```text
                    Benchmark Job
                         │
                         ▼
                   Raw Measurements
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
          Latency     Throughput    Resources
            │            │            │
            └────────────┼────────────┘
                         ▼
                Normalized Evidence
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
         Baseline     Statistics   History
             │           │           │
             └───────────┼───────────┘
                         ▼
               Regression Facts
                         │
                         ▼
                Policy / Quality Gate
```

---

# 3. Goals

The subsystem MUST:

1. define benchmark identity;
2. define benchmark suites;
3. support repeated samples;
4. support warmup;
5. support microbenchmarks;
6. support macrobenchmarks;
7. support load tests;
8. support stress tests;
9. support soak tests;
10. support startup-time measurements;
11. support latency percentiles;
12. support throughput;
13. support CPU measurements;
14. support memory measurements;
15. support allocation measurements;
16. support disk/network measurements;
17. support binary-size regression;
18. support package-size regression;
19. support baseline selection;
20. support statistical comparison;
21. support hardware-class matching;
22. support environment fingerprints;
23. support noise detection;
24. support performance budgets;
25. support Change Proposal gates;
26. support Release gates;
27. support capacity intelligence;
28. support device benchmarks;
29. support history/search/analytics;
30. remain evidence-first.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Criterion/iai-callgrind/hyperfine/JMH/pytest-benchmark/etc.
replace load generators
replace scheduler
replace profiler
replace observability backend
replace policy engine
guarantee scientific reproducibility on arbitrary shared hardware
```

---

# 5. Workspace Structure

```text
crates/benchmark/
├── forgeyard-benchmark/
├── forgeyard-benchmark-model/
├── forgeyard-benchmark-definition/
├── forgeyard-benchmark-ingest/
├── forgeyard-benchmark-normalize/
├── forgeyard-benchmark-baseline/
├── forgeyard-benchmark-statistics/
├── forgeyard-benchmark-regression/
├── forgeyard-benchmark-environment/
├── forgeyard-benchmark-history/
├── forgeyard-benchmark-capacity/
├── forgeyard-benchmark-quality/
├── forgeyard-benchmark-search/
├── forgeyard-benchmark-health/
└── forgeyard-benchmark-testkit/
```

Load testing:

```text
crates/loadtest/
├── forgeyard-loadtest/
├── forgeyard-loadtest-model/
├── forgeyard-loadtest-plan/
├── forgeyard-loadtest-runner/
├── forgeyard-loadtest-result/
├── forgeyard-loadtest-analysis/
└── forgeyard-loadtest-testkit/
```

Use modules first; split only where runtime/security/dependency boundaries justify.

---

# 6. BenchmarkDefinitionId

```rust
pub struct BenchmarkDefinitionId(Digest);
```

Content-derived benchmark definition identity.

---

# 7. BenchmarkSuiteId

```rust
pub struct BenchmarkSuiteId(Digest);
```

---

# 8. BenchmarkCaseId

```rust
pub struct BenchmarkCaseId(Digest);
```

---

# 9. Benchmark Definition

```rust
pub struct BenchmarkDefinition {
    pub id: BenchmarkDefinitionId,
    pub suite: BenchmarkSuiteId,
    pub case: BenchmarkCaseId,
    pub metric: BenchmarkMetric,
    pub direction: OptimizationDirection,
    pub sampling: SamplingPlan,
    pub environment: BenchmarkEnvironmentRequirement,
}
```

---

# 10. Optimization Direction

```rust
pub enum OptimizationDirection {
    LowerIsBetter,
    HigherIsBetter,
    TargetRange,
    InformationalOnly,
}
```

---

# 11. Benchmark Metric

```rust
pub enum BenchmarkMetric {
    Latency,
    Throughput,
    CpuTime,
    WallTime,
    MemoryPeak,
    MemoryAverage,
    Allocations,
    BinarySize,
    PackageSize,
    StartupTime,
    DiskThroughput,
    NetworkThroughput,
    FramesPerSecond,
    Energy,
    Custom(BenchmarkMetricId),
}
```

---

# 12. Unit

Every metric carries explicit unit.

Examples:

```text
nanoseconds
milliseconds
requests/second
bytes
bytes/second
joules
frames/second
```

---

# 13. No Unitless Numeric Result

Critical.

---

# 14. Benchmark Observation

```rust
pub struct BenchmarkObservation {
    pub id: BenchmarkObservationId,
    pub definition: BenchmarkDefinitionId,
    pub subject: BenchmarkSubject,
    pub environment: BenchmarkEnvironmentFingerprint,
    pub sample_set: BenchmarkSampleSetRef,
    pub summary: BenchmarkSummary,
    pub created_at: Timestamp,
}
```

---

# 15. BenchmarkObservationId

```rust
pub struct BenchmarkObservationId(Ulid);
```

---

# 16. Benchmark Subject

```rust
pub enum BenchmarkSubject {
    SourceSnapshot(SourceSnapshotId),
    Artifact(ArtifactId),
    Package(PackageId),
    ReleaseCandidate(ReleaseCandidateId),
    DeploymentRevision(DeploymentRevisionId),
}
```

---

# 17. Exact Binding

No branch-name baseline authority.

---

# 18. Raw Samples

Stored in CAS or normalized metadata depending size.

---

# 19. Sample Set

```rust
pub struct BenchmarkSampleSet {
    pub warmup_samples: Vec<Measurement>,
    pub measured_samples: Vec<Measurement>,
}
```

---

# 20. Warmup

Explicitly separate.

---

# 21. Sampling Plan

```rust
pub struct SamplingPlan {
    pub warmup_iterations: u32,
    pub measurement_iterations: u32,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
}
```

---

# 22. No Hidden Adaptive Sampling

If adapter/framework adapts dynamically, record the actual method/version.

---

# 23. Benchmark Framework

Retain:

```text
framework name
version
adapter version
command/toolchain
```

---

# 24. Raw Tool Output

CAS.

---

# 25. Normalizer Version

Recorded.

---

# 26. Reprocessing

Parser/analysis upgrades create new interpretation revision.

---

# 27. BenchmarkEnvironmentFingerprint

```rust
pub struct BenchmarkEnvironmentFingerprint(Digest);
```

---

# 28. Environment Inputs

Potential:

```text
OS/kernel
CPU model/class
CPU topology
memory class
GPU/device
power profile
governor
runner image/toolchain
executor/sandbox
virtualization
NUMA
device model
thermal state class
```

---

# 29. Hardware Class

```rust
pub struct BenchmarkHardwareClassId(Digest);
```

---

# 30. Why Hardware Matters

Comparing laptop and server CPU directly is usually meaningless.

---

# 31. Dedicated Benchmark Runner

Recommended for protected gates.

---

# 32. Shared Runner

Can produce informational data only unless noise controls satisfy policy.

---

# 33. Runner Capability

```text
benchmark.trusted
benchmark.hardware_class=<id>
```

---

# 34. Benchmark Runner Configuration

Prefer:

```text
fixed CPU governor
bounded background load
stable thermal conditions
known memory config
no overcommit
```

where practical.

---

# 35. Hardware Drift

If BIOS/CPU/kernel changes materially, environment fingerprint changes.

---

# 36. Baseline Compatibility

Policy decides if environments are comparable.

---

# 37. Strict Mode

Require exact hardware class + benchmark environment profile.

---

# 38. Relaxed Mode

Allow calibrated normalization.

---

# 39. Calibration

Optional.

---

# 40. Calibration Benchmark

Known stable workload executed to estimate host performance factor.

---

# 41. Calibration Risk

Normalization can hide real hardware-specific regressions.

---

# 42. Recommended Baseline

Direct same-class comparisons for protected gates.

---

# 43. Benchmark Profile

```rust
pub struct BenchmarkProfileId(Digest);
```

Defines:

```text
runner class
sandbox
resources
network
warmup
sampling
noise limits
```

---

# 44. CPU Affinity

Optional but useful.

---

# 45. NUMA Affinity

For high-end benchmarks.

---

# 46. Frequency Scaling

Record/enforce profile where possible.

---

# 47. Thermal State

Device benchmarks should record coarse thermal state.

---

# 48. Battery State

Mobile performance may require:

```text
charging
battery percentage range
power saver off
```

---

# 49. Device Benchmarks

Part 20 DeviceLease + benchmark evidence.

---

# 50. Android

Can measure:

```text
startup
frame time
memory
codec throughput
battery/energy where APIs permit
```

---

# 51. Apple Device

Likewise through platform-specific adapters.

---

# 52. Benchmark Execution

Still ordinary Forgeyard Job/Attempt.

---

# 53. No Separate Benchmark Executor

Use normal sandbox/executor plus benchmark profile.

---

# 54. Benchmark Job Retry

New JobAttempt.

---

# 55. Original Observation

Never overwritten.

---

# 56. Failed Benchmark Tool

Outcome distinct from performance regression.

---

# 57. BenchmarkRunOutcome

```rust
pub enum BenchmarkRunOutcome {
    Completed,
    ToolFailed,
    TimedOut,
    InfrastructureError,
    InvalidEvidence,
}
```

---

# 58. Invalid Evidence

Examples:

```text
insufficient samples
environment drift
malformed report
```

---

# 59. Summary

```rust
pub struct BenchmarkSummary {
    pub count: u64,
    pub mean: Measurement,
    pub median: Measurement,
    pub p50: Measurement,
    pub p95: Option<Measurement>,
    pub p99: Option<Measurement>,
    pub stddev: Option<Measurement>,
    pub min: Measurement,
    pub max: Measurement,
}
```

---

# 60. Percentile Validity

Only meaningful with sufficient sample count.

---

# 61. No Fake p99 From 5 Samples

Critical.

---

# 62. Summary Validity

Expose which statistics are supported.

---

# 63. Latency Distribution

Raw/histogram evidence where needed.

---

# 64. HDR Histogram

Possible adapter representation.

---

# 65. Throughput

Measure with duration/concurrency context.

---

# 66. Resource Utilization

Record:

```text
CPU
memory
IO
network
```

with observation interval.

---

# 67. Binary Size

Simple deterministic metric.

---

# 68. BinarySizeObservation

Bind exact ArtifactId/CAS digest.

---

# 69. Package Size

Bind exact PackageId.

---

# 70. Section Size

Optional:

```text
text
rodata
debug
```

---

# 71. Strip State

Must be explicit.

---

# 72. Compression

Package size comparison requires same package format/compression semantics.

---

# 73. Baseline

```rust
pub struct BenchmarkBaseline {
    pub id: BenchmarkBaselineId,
    pub definition: BenchmarkDefinitionId,
    pub subject: BenchmarkSubject,
    pub observation: BenchmarkObservationId,
    pub selection: BaselineSelectionReason,
}
```

---

# 74. Baseline Selection Strategies

```rust
pub enum BaselineSelection {
    ExactSubject(BenchmarkSubject),
    TargetBranchLatestGreen,
    LastRelease(ChannelId),
    RollingWindow,
    Explicit(BenchmarkObservationId),
}
```

---

# 75. Mutable Reference Resolution

`TargetBranchLatestGreen` resolves to exact SourceSnapshotId before comparison.

---

# 76. Baseline Snapshot

Comparison stores exact resolved observation.

---

# 77. No Moving Baseline in Historical Decision

Critical.

---

# 78. Baseline Missing

Regression result `Incomplete`.

---

# 79. Comparison

```rust
pub struct BenchmarkComparison {
    pub id: BenchmarkComparisonId,
    pub candidate: BenchmarkObservationId,
    pub baseline: BenchmarkObservationId,
    pub method: StatisticalMethod,
    pub result: RegressionResult,
}
```

---

# 80. Statistical Method

Baseline options:

```rust
pub enum StatisticalMethod {
    RelativeThreshold,
    ConfidenceInterval,
    MannWhitneyU,
    BootstrapDifference,
    FrameworkNative,
}
```

---

# 81. Initial Recommendation

Use simple:

```text
relative threshold
+
minimum absolute delta
+
sample/noise validity
```

then add robust statistics.

---

# 82. Why

Explainability.

---

# 83. Relative Change

```text
(candidate - baseline) / baseline
```

with metric direction.

---

# 84. Absolute Delta

Prevents tiny values creating misleading large percentages.

---

# 85. Regression Threshold

```rust
pub struct RegressionThreshold {
    pub relative: Option<Ratio>,
    pub absolute: Option<Measurement>,
}
```

---

# 86. Example

```text
latency regression if:
> 5%
AND
> 2 ms
```

---

# 87. Improvement

Can classify separately.

---

# 88. RegressionClassification

```rust
pub enum RegressionClassification {
    SignificantRegression,
    PossibleRegression,
    NoMeaningfulChange,
    Improvement,
    Inconclusive,
    IncompatibleEnvironment,
    InsufficientData,
}
```

---

# 89. Inconclusive

First-class.

---

# 90. Never Convert Inconclusive to Pass Silently

Policy chooses.

---

# 91. Noise

BenchmarkNoiseAssessment.

---

# 92. Noise Inputs

```text
coefficient of variation
outliers
host background load
thermal throttling
calibration drift
```

---

# 93. Noisy Observation

Can be rejected/inconclusive.

---

# 94. Outlier Handling

Method must be explicit/versioned.

---

# 95. Never Delete Raw Samples

Critical.

---

# 96. Winsorization/Filtering

If used, preserve method parameters.

---

# 97. Statistical Version

```rust
pub struct BenchmarkAnalysisVersion(u16);
```

---

# 98. Analysis Reprocessing

Old raw observations can be re-analyzed.

---

# 99. Historical Decision

Original comparison remains immutable.

---

# 100. Performance Budget

```rust
pub struct PerformanceBudget {
    pub id: PerformanceBudgetId,
    pub scope: PerformanceBudgetScope,
    pub metric: BenchmarkMetric,
    pub threshold: PerformanceThreshold,
}
```

---

# 101. Budget Scope

```text
benchmark case
suite
artifact
package
platform
release channel
```

---

# 102. Absolute Budget

Example:

```text
startup p95 <= 500ms
```

---

# 103. Regression Budget

Example:

```text
no >5% latency regression
```

---

# 104. Size Budget

Example:

```text
APK <= 40 MiB
```

---

# 105. Memory Budget

Example:

```text
peak RSS <= 512 MiB
```

---

# 106. Quality Fact

```rust
pub enum PerformanceFact {
    BenchmarkResult(BenchmarkObservationRef),
    Regression(BenchmarkComparisonRef),
    Budget(PerformanceBudgetResult),
    LoadTest(LoadTestResultRef),
    Capacity(CapacityResultRef),
}
```

---

# 107. Policy

Part 11 evaluates.

---

# 108. No Second Performance Policy Engine

Critical.

---

# 109. Change Proposal

Can require:

```text
no significant regression
binary size budget
benchmark suite complete
```

---

# 110. Release

Can require stricter:

```text
load/soak test pass
capacity floor
no inconclusive critical benchmark
```

---

# 111. Performance Quality Decision

Can reuse Part 32 `QualityDecision`.

---

# 112. Exact Binding

Candidate benchmark binds exact ProposalRevision/SourceSnapshot/Artifact.

---

# 113. Old Benchmark

Cannot satisfy new candidate unless exact subject equivalence proven.

---

# 114. Load Testing

Different from microbenchmark.

---

# 115. LoadTestPlanId

```rust
pub struct LoadTestPlanId(Digest);
```

---

# 116. Load Test Plan

```rust
pub struct LoadTestPlan {
    pub target: LoadTarget,
    pub workload: WorkloadModel,
    pub phases: Vec<LoadPhase>,
    pub success_criteria: Vec<LoadCriterion>,
}
```

---

# 117. Load Target

```rust
pub enum LoadTarget {
    Deployment(DeploymentRevisionId),
    ServiceEndpoint(ManagedEndpointRef),
    LocalArtifact(ArtifactId),
}
```

---

# 118. External Target

Restricted.

---

# 119. Safety

Forgeyard must prevent accidental load tests against arbitrary third-party hosts.

---

# 120. Target Authorization

Only configured/owned targets.

---

# 121. Load Generator Network

Explicit policy.

---

# 122. Workload Model

```rust
pub enum WorkloadModel {
    ConstantRate,
    ConstantConcurrency,
    RampRate,
    RampConcurrency,
    TraceReplay,
}
```

---

# 123. Load Phase

```rust
pub struct LoadPhase {
    pub duration: Duration,
    pub target_rate: Option<RequestRate>,
    pub concurrency: Option<u32>,
}
```

---

# 124. Warmup Phase

Explicit.

---

# 125. Steady State

Explicit.

---

# 126. Cooldown

Optional.

---

# 127. Success Criteria

Examples:

```text
p95 < 200 ms
error rate < 1%
throughput >= 5000 rps
CPU < 80%
```

---

# 128. Load Generator

Normal runner jobs.

---

# 129. Distributed Load Generation

Multiple generator jobs coordinated by plan.

---

# 130. LoadCoordinatorId

```rust
pub struct LoadCoordinatorId(Ulid);
```

---

# 131. Synchronization

Use scheduled start barrier/time window.

---

# 132. No Precise Nanosecond Synchronization Claim

Clock accuracy documented.

---

# 133. Generator Identity

Each shard/runner recorded.

---

# 134. Result Merge

Time-bucket/histogram merge.

---

# 135. Generator Failure

Load result can be Incomplete.

---

# 136. Target Telemetry

Can correlate server-side metrics/traces.

---

# 137. But Observability Dependency

Load result should distinguish:

```text
client-side measurement
server-side supporting telemetry
```

---

# 138. Target Telemetry Missing

May make resource criteria Incomplete, not fabricate.

---

# 139. Stress Test

Goal:

```text
find failure/saturation point
```

---

# 140. Stress Plan

Gradually increase load.

---

# 141. Saturation Point

```rust
pub struct CapacityPoint {
    pub throughput: Measurement,
    pub concurrency: Option<u32>,
    pub limiting_resource: Option<ResourceKind>,
}
```

---

# 142. Capacity Result

Derived evidence.

---

# 143. Stop Conditions

Critical:

```text
error threshold
latency threshold
resource threshold
target protection signal
max configured load
```

---

# 144. Automatic Kill Switch

Mandatory for production-adjacent stress tests.

---

# 145. Soak Test

Sustained duration.

---

# 146. Goals

Detect:

```text
memory leak
resource leak
latency drift
connection exhaustion
thermal effects
```

---

# 147. Soak Duration

Could be hours/days.

---

# 148. Durable Execution

Normal long-running Job semantics.

---

# 149. Reconnect

Logs/results resilient.

---

# 150. Soak Intermediate Checkpoints

Persist summaries periodically.

---

# 151. Process Crash

Final result incomplete/failed.

---

# 152. Load-Test Environment

Must distinguish:

```text
ephemeral staging
production canary
dedicated benchmark environment
```

---

# 153. Production Load Test

High privilege.

---

# 154. Permission

```text
benchmark.load.production
```

---

# 155. Policy

Requires explicit target and maximum load.

---

# 156. Audit

Production load/stress execution audited.

---

# 157. Notifications

Notify operators before/after production stress test.

---

# 158. Deployment Integration

Benchmark can target exact DeploymentRevisionId.

---

# 159. Canary Performance Gate

Deployment may require latency/resource criteria before advancing.

---

# 160. Caution

Runtime deployment health metrics are not identical to controlled benchmark evidence.

---

# 161. Capacity Intelligence

Historical load results can answer:

```text
how many requests can this configuration sustain?
```

---

# 162. CapacityProfileId

```rust
pub struct CapacityProfileId(Digest);
```

---

# 163. Capacity Profile Inputs

```text
release/artifact
deployment config
replica count
instance class
DB class
load workload
```

---

# 164. Capacity Result

```rust
pub struct CapacityResult {
    pub profile: CapacityProfileId,
    pub max_sustainable_rate: Option<RequestRate>,
    pub saturation_point: Option<CapacityPoint>,
    pub limiting_resources: Vec<ResourceLimitObservation>,
}
```

---

# 165. No Universal Capacity Claim

Bound to exact profile/environment.

---

# 166. Autoscaling Integration

Capacity facts can inform autoscaler configuration.

---

# 167. Autoscaler Authority

Separate.

---

# 168. Capacity Recommendation

Advisory unless policy explicitly uses it.

---

# 169. Performance Baseline Repository

Metadata DB.

---

# 170. Raw Sample Storage

CAS.

---

# 171. Observation Tables

```text
benchmark_definitions
benchmark_observations
benchmark_comparisons
benchmark_baselines
performance_budgets
load_test_plans
load_test_results
capacity_results
```

---

# 172. Immutable Observations

Critical.

---

# 173. Baseline Pointer

Mutable selection pointer may exist.

---

# 174. Historical Comparison

Stores exact baseline observation.

---

# 175. Baseline Promotion

Example:

```text
new stable release becomes baseline
```

---

# 176. Baseline Promotion Audit

If manually selected.

---

# 177. Automatic Baseline

Policy-driven.

---

# 178. Baseline Poisoning

A bad release should not silently become benchmark baseline unless release policy allows.

---

# 179. Recommended

Stable-channel baseline only after release success/soak.

---

# 180. Branch Baseline

Target branch latest accepted change.

---

# 181. Benchmark Search

Part 31.

---

# 182. Queries

```text
benchmark:startup platform:linux regressions:true
binary-size:>100MiB
```

---

# 183. Analytics

Historical trends.

---

# 184. Benchmark History

Graphs:

```text
median
p95
relative delta
```

---

# 185. No Visual Smoothing That Hides Regressions

Raw points accessible.

---

# 186. UI Pages

```text
Benchmarks
Regressions
Performance Budgets
Load Tests
Capacity
Binary Size
```

---

# 187. Run Benchmark Tab

Shows:

```text
candidate
baseline
delta
confidence/noise
environment match
```

---

# 188. Regression Detail

Explain:

```text
candidate median
baseline median
relative delta
absolute delta
sample count
method
noise assessment
```

---

# 189. Environment Mismatch

Prominent.

---

# 190. Inconclusive

Never displayed as green Pass by default.

---

# 191. Binary Size UI

Breakdown by artifact/platform.

---

# 192. Package Size Trend

Release history.

---

# 193. Load Test UI

Live:

```text
current phase
request rate
latency
errors
generator health
target health
```

---

# 194. Live View Is Not Final Evidence

Final persisted result after completion.

---

# 195. Capacity UI

Shows exact configuration context.

---

# 196. Comparison API

Potential:

```text
GET /v1/benchmarks
GET /v1/benchmarks/{id}/history
GET /v1/benchmark-comparisons/{id}
GET /v1/performance-regressions
POST /v1/load-tests
GET /v1/load-tests/{id}
GET /v1/capacity-results
```

---

# 197. Permissions

```text
benchmark.read
benchmark.run
benchmark.manage_baseline
benchmark.budget.manage
loadtest.run
loadtest.production
capacity.read
performance.override
```

---

# 198. Performance Override

If policy allows.

---

# 199. Override

Does not modify benchmark evidence.

---

# 200. Override Record

Exact comparison/budget result + reason + principal + scope.

---

# 201. Audit

Mandatory.

---

# 202. Benchmark Report Adapters

Potential:

```text
Criterion JSON
hyperfine JSON
JMH JSON
pytest-benchmark
Google Benchmark JSON
custom RON/Postcard
```

---

# 203. Adapter Principle

Framework-native parser -> normalized metrics.

---

# 204. Unknown Metric

Preserve raw evidence; normalized Custom metric if configured.

---

# 205. Report Security

Untrusted input.

---

# 206. JSON/XML Limits

Bounded.

---

# 207. Numeric Validation

Reject:

```text
NaN where unsupported
infinity
negative duration
overflow
```

---

# 208. Unit Conversion

Typed.

---

# 209. Floating Point

Use carefully for statistical values.

---

# 210. Base Measurements

Can store integer nanos/bytes/counts where possible.

---

# 211. Ratios

Explicit rational/decimal.

---

# 212. Deterministic Serialization

Needed for evidence digests.

---

# 213. Benchmark Command Output

Never trust stdout regex as baseline architecture.

---

# 214. Adapter Can Parse Text

Only if bounded and versioned.

---

# 215. Perf Counters

Linux `perf`/hardware counters optional.

---

# 216. Privilege

Performance counters can require kernel permissions.

---

# 217. Benchmark Runner Capability

Explicit.

---

# 218. eBPF

Optional diagnostics, not benchmark correctness dependency.

---

# 219. Profiling

Benchmark regression can trigger profile capture.

---

# 220. Profile Artifact

CAS.

---

# 221. Flamegraph

Derived visualization artifact.

---

# 222. Profiling Is Diagnostic

Does not replace benchmark metric.

---

# 223. Automatic Profile on Regression

Optional policy.

---

# 224. Resource Impact

Bound.

---

# 225. Benchmark Caching

Generally dangerous.

---

# 226. Do Not Cache Benchmark Result Based on build cache alone

Performance must actually run.

---

# 227. Reuse Evidence

Only explicit exact observation reuse, usually not for fresh release gate.

---

# 228. Benchmark Frequency

Can be:

```text
every PR
nightly
release candidate
manual
```

---

# 229. Expensive Benchmark

Nightly/release.

---

# 230. Freshness

Performance evidence can expire.

---

# 231. PerformanceEvidenceFreshness

```rust
pub enum PerformanceEvidenceFreshness {
    Fresh,
    Stale,
    Unknown,
}
```

---

# 232. Release Policy

May require observation within certain time/source/environment.

---

# 233. Baseline Freshness

Also.

---

# 234. Resource Governance

Part 27 quotas apply.

---

# 235. Benchmark Priority

Cannot bypass tenant fairness.

---

# 236. Dedicated Benchmark Pool Reservation

Explicit.

---

# 237. Load Test Cost

Usage accounted normally.

---

# 238. Entitlement

Part 30 may gate advanced benchmark analytics, but security/correctness unaffected.

---

# 239. Multi-Tenancy

Benchmark evidence tenant scoped.

---

# 240. Shared Hardware

Results isolated logically.

---

# 241. Cross-Tenant Performance Leakage

Do not expose other tenant workload/host data.

---

# 242. Dedicated Host

Recommended for sensitive/strict benchmark.

---

# 243. Host Utilization Metadata

Store coarse safe facts.

---

# 244. No Other Tenant Names

Critical.

---

# 245. Device Lab

Device benchmark reservation may require:

```text
model
OS version
thermal profile
battery range
```

---

# 246. Repetition on Device

Noise generally higher.

---

# 247. Statistical Threshold

Device-specific policy.

---

# 248. Benchmark Failure Cluster

Can correlate runtime crashes, but separate from Part 32 test failures.

---

# 249. Performance Regression Notification

Notify owners/reviewers.

---

# 250. SCM Check

Publish:

```text
Performance
Binary Size
Load Test
```

---

# 251. Provider Summary

Bound exact revision.

---

# 252. Annotation

Top regressions only.

---

# 253. Release Evidence Bundle

Performance evidence can join supply-chain/quality evidence.

---

# 254. Performance Evidence Kind

Extend Part 13 evidence taxonomy.

---

# 255. Benchmark Provenance

Records:

```text
benchmark definition
tool version
hardware class
runner
executor
toolchain
artifact/source subject
samples
analysis version
```

---

# 256. Performance Reproducibility

Not same as byte reproducibility.

---

# 257. Repeatability

Can classify benchmark repeatability.

---

# 258. Repeatability Class

```rust
pub enum PerformanceRepeatability {
    High,
    Moderate,
    Low,
    Unknown,
}
```

---

# 259. Decision Policy

Can require High/Moderate for protected gates.

---

# 260. Benchmark Environment Drift

Detected before comparing.

---

# 261. Kernel Update

May invalidate baseline comparability.

---

# 262. Compiler Update

Can legitimately alter performance.

Still regression relative to chosen baseline, but context explicit.

---

# 263. Toolchain Change

Comparison stores.

---

# 264. Source Change + Toolchain Change

Policy may mark comparison confounded.

---

# 265. Confounded Comparison

```rust
pub enum ComparisonValidity {
    Valid,
    Confounded,
    Invalid,
}
```

---

# 266. Confounded

Policy can require rerun with matching toolchain.

---

# 267. Time Series

Benchmark trends use exact observations.

---

# 268. Missing Data

Gap, not interpolated as measured.

---

# 269. Search/Analytics Derived

Can rebuild from observations.

---

# 270. DR

Metadata backup + raw samples CAS.

---

# 271. Baseline/Performance Budgets

Authoritative metadata, must backup.

---

# 272. Rebuild

Trend projections rebuildable.

---

# 273. Load Test Results

Raw histograms/summaries in CAS.

---

# 274. Long Soak Logs

Retention policy.

---

# 275. HA

Ingestion/analysis workers idempotent.

---

# 276. Duplicate Analysis

Comparison identity prevents duplicate semantic records.

---

# 277. BenchmarkComparisonId

Can be content-derived from candidate + baseline + analysis version.

---

# 278. Analysis Failure

Quality `Incomplete`, not Pass.

---

# 279. Service Outage

CI jobs can complete; performance gate waits/degrades according to policy.

---

# 280. Testkit

```text
forgeyard-benchmark-testkit/src/
├── lib.rs
├── definition.rs
├── observation.rs
├── baseline.rs
├── statistics.rs
├── regression.rs
├── environment.rs
└── assertions.rs
```

Load tests:

```text
forgeyard-loadtest-testkit/src/
```

---

# 281. Unit Tests

Unit conversion/statistical thresholds.

---

# 282. Environment Match Test

Different hardware class rejected in strict mode.

---

# 283. Noisy Sample Test

Inconclusive.

---

# 284. Single-Sample Test

Cannot claim p99/significance.

---

# 285. Baseline Resolution Test

Mutable target resolved to exact observation.

---

# 286. Baseline History Test

Historical comparison does not move after new stable release.

---

# 287. Threshold Test

Relative + absolute threshold.

---

# 288. Improvement Direction Test

Higher-is-better handled correctly.

---

# 289. Binary Size Test

Exact artifact.

---

# 290. Retry Test

Old benchmark observation retained.

---

# 291. Parser Security Test

Malformed/oversized report rejected.

---

# 292. NaN/Infinity Test

Rejected/handled explicitly.

---

# 293. Confounded Toolchain Test

Comparison flagged.

---

# 294. Change Proposal Test

Old benchmark cannot satisfy new source revision.

---

# 295. Release Test

Critical benchmark Inconclusive blocks if policy requires.

---

# 296. Load Target Auth Test

Cannot load-test arbitrary internet target.

---

# 297. Stress Kill-Switch Test

Stops at configured safety threshold.

---

# 298. Distributed Generator Failure Test

Result Incomplete.

---

# 299. Soak Restart Test

Checkpoints survive reconnect where supported.

---

# 300. Device Thermal Test

Out-of-profile observation invalid/inconclusive.

---

# 301. Tenant Isolation Test

No cross-tenant evidence/trend.

---

# 302. HA Worker Test

Duplicate analysis safe.

---

# 303. DR Test

Baseline/history restored.

---

# 304. Fuzzing

Fuzz benchmark report parsers and numeric/unit conversion.

---

# 305. Failure Injection

```text
runner lost
CAS sample missing
analysis worker crash
load generator timeout
target telemetry unavailable
```

---

# 306. Load Scale Test

Large distributed load tests.

---

# 307. History Scale Test

Millions of benchmark observations.

---

# 308. Implementation Phase 1 — Benchmark Model/Ingestion

Definitions, observations, raw samples.

---

# 309. Phase 2 — Baselines/Comparisons

Relative/absolute threshold.

---

# 310. Phase 3 — Environment Fingerprinting

Trusted benchmark runners.

---

# 311. Phase 4 — Performance Budgets

Policy facts.

---

# 312. Phase 5 — Binary/Package Size

Easy deterministic wins.

---

# 313. Phase 6 — Statistical Analysis

Confidence/noise/inconclusive.

---

# 314. Phase 7 — Change/Release Integration

SCM/quality gates.

---

# 315. Phase 8 — Load/Stress Tests

Controlled targets.

---

# 316. Phase 9 — Soak/Capacity

Long-running resilience.

---

# 317. Phase 10 — Device Performance

Real mobile hardware.

---

# 318. Phase 11 — Search/Analytics/Notifications

UX/operations.

---

# 319. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 320. Acceptance Tests

1. Benchmarks run as normal Forgeyard Jobs.
2. Benchmark observations are immutable.
3. Raw samples are preserved.
4. Every metric has explicit unit and optimization direction.
5. Benchmark subject is exact SourceSnapshot/Artifact/Package/Release/Deployment identity.
6. Hardware/environment fingerprint is recorded.
7. Strict gates compare compatible hardware classes.
8. Mutable baseline names are resolved to exact observations before comparison.
9. Historical comparison baseline never silently moves.
10. Single noisy measurement cannot silently produce a protected Pass/Fail.
11. Insufficient data is explicit.
12. Inconclusive is explicit.
13. Environment mismatch is explicit.
14. Raw outliers are never deleted from evidence.
15. Statistical/normalization version is recorded.
16. Parser upgrades create new analysis rather than rewrite observations.
17. Relative and absolute thresholds can be combined.
18. Binary/package size gates bind exact immutable artifact/package.
19. Performance override never mutates evidence.
20. Change Proposal benchmark evidence binds exact ProposalRevision/source.
21. Old observations cannot satisfy new candidate unless exact equivalence is proven.
22. Release policy may require stricter benchmark/load/soak evidence than PR policy.
23. Load tests can target only authorized/configured endpoints.
24. Production load/stress tests require explicit high-risk permission/policy.
25. Stress tests have hard kill switches.
26. Distributed generator failure yields Incomplete rather than false success.
27. Capacity claims are bound to exact deployment/workload profile.
28. Device benchmark observations include device/environment context.
29. Tenant A cannot access Tenant B performance evidence.
30. Search/analytics are derived, not performance authority.
31. Benchmark analysis is idempotent/reconcilable.
32. Performance evidence can be included in release evidence bundles.
33. Standalone/distributed share benchmark semantics.
34. Benchmark system outage does not corrupt execution state.
35. Forgeyard dogfoods benchmark/performance gates for its own releases.

---

# 321. Production Readiness Gates

Do not call performance intelligence production-ready until:

```text
immutable observation model is stable
environment fingerprinting is reliable
trusted benchmark-runner profile exists
baseline resolution is exact
noise/inconclusive handling is explicit
relative+absolute regression rules are tested
binary/package size gates work
change/release subject binding is exact
load target authorization/kill switches pass
tenant isolation and DR tests pass
```

---

# 322. Architectural Invariants

1. benchmark execution is ordinary Forgeyard execution;
2. performance observations are immutable;
3. raw samples are retained;
4. every metric has explicit unit;
5. benchmark subject identity is exact;
6. hardware/environment context is mandatory for meaningful comparisons;
7. baseline selection resolves to immutable observation;
8. historical baselines never move;
9. one noisy sample is never enough for hidden authority;
10. Inconclusive is first-class;
11. incompatible environment is first-class;
12. raw samples are not rewritten by outlier processing;
13. statistical method/version is recorded;
14. parser/analysis upgrades create new interpretations;
15. policy remains central decision authority;
16. performance override never edits evidence;
17. binary/package size binds exact artifacts;
18. protected gates reject stale subject evidence;
19. load tests target only authorized systems;
20. production stress tests require explicit permission/policy;
21. kill switches are mandatory for stress tests;
22. load generator failures cannot produce false pass;
23. capacity claims are profile-specific;
24. device performance records hardware/thermal context;
25. tenant evidence is isolated;
26. benchmark results are not cached as if execution occurred when it did not;
27. search/analytics are derived;
28. HA analysis is idempotent/reconcilable;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its performance regression system.

---

# 323. Final Target Architecture

```text
                     Benchmark Job
                          │
                          ▼
                     Raw Samples
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
         Environment    Summary      Resources
             │            │            │
             └────────────┼────────────┘
                          ▼
                       Baseline
                          │
                          ▼
                  Statistical Compare
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
         Regression   Inconclusive   Improvement
             │            │            │
             └────────────┼────────────┘
                          ▼
                    Performance Fact
                          │
                          ▼
                     Policy Gate
```

---

# 324. Final Architectural Position

Micro/macro benchmark:

```text
exact subject
+
benchmark definition
+
trusted environment
+
raw repeated samples
  ↓
normalized observation
  ↓
exact baseline
  ↓
statistical comparison
  ↓
performance fact
```

Load/capacity testing:

```text
authorized target
+
workload plan
+
generator set
+
safety limits
  ↓
load/stress/soak execution
  ↓
latency/error/resource evidence
  ↓
capacity result
```

Merge/release:

```text
exact candidate
+
performance evidence
+
comparison validity
+
PerformanceBudget
+
PolicyDigest
  ↓
Pass / Warning / Fail / Incomplete / Stale
```

The key guarantee is:

> **Forgeyard treats performance as measured evidence, not intuition or a single noisy number. Every regression claim is tied to exact code/artifacts, benchmark definitions, comparable environments, raw samples, and an explicit analysis method, allowing performance gates to be strict without pretending inherently noisy measurements are perfectly deterministic.**

---

# 325. Extended Architecture Sequence

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
```
