# 34 — Forgeyard Monorepo Intelligence, Dependency Graph, Affected-Change & Incremental Execution System Architecture

**Document type:** Core Monorepo Graph, Dependency Intelligence, Change Impact & Incremental Execution System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** workspace discovery, project/package graph, build-target graph, source ownership, dependency impact, affected-target detection, changed-file analysis, incremental planning, selective builds/tests, graph snapshots, cache-aware execution, cycle handling, graph queries, monorepo scale, and policy-safe optimization  
**Architecture style:** Exact immutable source snapshots, deterministic graph construction, explicit dependency semantics, conservative impact analysis, derived graph intelligence, policy-controlled skipping, strong cache integration, and no silent correctness reduction  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on VCS-neutral source snapshots, Pipeline IR, Scheduler, CAS/cache, Test Intelligence, Benchmarking, Search/Analytics, Change Proposal, RBE, language/ecosystem adapters, and hermetic/reproducible builds. This subsystem adds large-repository intelligence without making source analysis authoritative for security or policy.

---

# 1. Purpose

Modern repositories often contain:

```text
many Rust crates
multiple services
shared libraries
desktop/mobile applications
frontend packages
infrastructure
documentation
test utilities
generated code
native components
```

A large monorepo cannot efficiently run every possible build and test for every small change.

Users need answers such as:

```text
what changed?
which packages own those files?
which targets depend on them?
which tests are affected?
which release artifacts may change?
which jobs can safely be skipped?
which cached results remain reusable?
```

The central rule is:

> **Forgeyard computes affected work from an immutable source snapshot and a versioned dependency graph, but impact analysis is an optimization layer—not an unquestioned correctness authority.**

A second rule is:

> **If Forgeyard cannot prove that a target is unaffected, the conservative answer is that it may be affected.**

A third rule is:

> **Policy decides whether affected-only execution is acceptable for a given workflow. Stable releases may require broader validation than ordinary Change Proposals.**

---

# 2. Architectural Position

```text
                  SourceSnapshotId
                        │
                        ▼
                Workspace Discovery
                        │
                        ▼
                 Dependency Graph
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
         Files       Targets      Tests
            │           │           │
            └───────────┼───────────┘
                        ▼
                  Change Impact
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
         Direct       Reverse      Policy
        Changes       Dependents   Expansion
            │           │           │
            └───────────┼───────────┘
                        ▼
                Incremental Plan
                        │
                        ▼
              Pipeline / Scheduler
```

---

# 3. Goals

The subsystem MUST:

1. discover workspace structure;
2. model packages/components;
3. model build targets;
4. model dependencies;
5. model file ownership;
6. model generated-source relationships;
7. support multiple ecosystems;
8. support monorepos;
9. support polyglot repositories;
10. support graph snapshots;
11. support changed-file detection;
12. support direct impact;
13. support reverse dependency expansion;
14. support test impact;
15. support artifact impact;
16. support pipeline impact;
17. support graph queries;
18. support incremental planning;
19. integrate with cache;
20. support policy expansion;
21. support conservative fallback;
22. support graph invalidation;
23. support cycles;
24. support large graphs;
25. support distributed indexing;
26. support graph diff;
27. support visibility rules;
28. support ownership;
29. support standalone/distributed modes;
30. remain rebuildable/derived.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Cargo/Bazel/CMake/npm/etc.
replace package managers
replace VCS
replace policy engine
replace scheduler
guarantee perfect semantic impact analysis for arbitrary code
```

---

# 5. Workspace Structure

```text
crates/graph/
├── forgeyard-graph/
├── forgeyard-graph-model/
├── forgeyard-graph-discovery/
├── forgeyard-graph-build/
├── forgeyard-graph-query/
├── forgeyard-graph-diff/
├── forgeyard-graph-impact/
├── forgeyard-graph-cache/
├── forgeyard-graph-policy/
├── forgeyard-graph-reconcile/
├── forgeyard-graph-health/
└── forgeyard-graph-testkit/
```

Ecosystem graph adapters:

```text
ecosystems/
├── rust/
│   └── graph/
├── go/
│   └── graph/
├── python/
│   └── graph/
├── javascript-typescript/
│   └── graph/
├── java-kotlin/
│   └── graph/
├── dart-flutter/
│   └── graph/
├── swift/
│   └── graph/
├── c-cpp/
│   └── graph/
└── native/
    └── graph/
```

Use modules first; separate crates only when ecosystem/tool dependencies justify.

---

# 6. Repository Graph Identity

```rust
pub struct RepositoryGraphId(Digest);
```

Content-derived from:

```text
SourceSnapshotId
graph schema version
discovery adapters
configuration
```

---

# 7. Graph Schema Version

```rust
pub struct GraphSchemaVersion(u16);
```

---

# 8. Graph Node

```rust
pub enum GraphNode {
    Workspace(WorkspaceNode),
    Package(PackageNode),
    Target(TargetNode),
    TestSuite(TestSuiteNode),
    Artifact(ArtifactNode),
    File(FileNode),
    GeneratedSource(GeneratedSourceNode),
    Custom(CustomGraphNode),
}
```

---

# 9. Graph Edge

```rust
pub struct GraphEdge {
    pub from: GraphNodeId,
    pub to: GraphNodeId,
    pub kind: DependencyKind,
}
```

---

# 10. Dependency Kind

```rust
pub enum DependencyKind {
    Build,
    Runtime,
    Test,
    Development,
    Tool,
    Codegen,
    Data,
    Configuration,
    GeneratedFrom,
    Produces,
    Owns,
    Custom(DependencyKindId),
}
```

---

# 11. Direction

Define clearly:

```text
A -> B
```

means A depends on B.

Reverse traversal gives affected dependents.

---

# 12. WorkspaceId

```rust
pub struct WorkspaceId(Digest);
```

---

# 13. PackageId

Graph package identity separate from release PackageId.

Use distinct name if collision:

```rust
pub struct SourcePackageId(Digest);
```

---

# 14. BuildTargetId

```rust
pub struct BuildTargetId(Digest);
```

---

# 15. FileNodeId

Derived from repo-relative path + SourceSnapshotId context.

---

# 16. Stable Logical Target

A logical target can have a stable key across snapshots.

---

# 17. TargetLogicalId

```rust
pub struct TargetLogicalId(Digest);
```

Derived from normalized workspace/package/target identity.

---

# 18. Why Two IDs

```text
TargetLogicalId
```

tracks history.

```text
BuildTargetId
```

binds exact graph snapshot/config.

---

# 19. File Ownership

```rust
pub struct FileOwnership {
    pub file: RepoRelativePath,
    pub owners: Vec<GraphNodeId>,
    pub confidence: OwnershipConfidence,
}
```

---

# 20. Ownership Confidence

```rust
pub enum OwnershipConfidence {
    Exact,
    Declared,
    Inferred,
    Unknown,
}
```

---

# 21. Exact

Manifest/build-system mapping.

---

# 22. Declared

Forgeyard config.

---

# 23. Inferred

Heuristic.

---

# 24. Unknown

Conservative impact.

---

# 25. No Silent Heuristic Authority

Inferred ownership cannot be treated as proof for strict release skipping unless policy permits.

---

# 26. Workspace Discovery

Starts from exact `SourceSnapshotId`.

---

# 27. Discovery Inputs

Examples:

```text
Cargo.toml
go.mod
package.json
pyproject.toml
pom.xml
build.gradle
CMakeLists.txt
Package.swift
forgeyard.ron
```

---

# 28. Discovery Adapter

```rust
#[async_trait]
pub trait WorkspaceGraphAdapter {
    async fn discover(
        &self,
        snapshot: SourceSnapshotRef,
        context: GraphDiscoveryContext,
    ) -> Result<GraphFragment, GraphDiscoveryError>;
}
```

---

# 29. Adapter Output

Normalized graph fragment.

---

# 30. No Ecosystem-Specific Types in Core Graph

Critical.

---

# 31. Rust Adapter

Can use Cargo metadata semantics.

---

# 32. Cargo Features

Feature configuration affects graph.

---

# 33. Target-Specific Dependencies

Represent conditions.

---

# 34. Conditional Edge

```rust
pub struct ConditionalDependency {
    pub edge: GraphEdge,
    pub condition: GraphCondition,
}
```

---

# 35. Graph Condition

Examples:

```text
platform
feature
profile
environment class
```

---

# 36. Graph Configuration

```rust
pub struct GraphConfiguration {
    pub target_platforms: Vec<PlatformDescriptor>,
    pub feature_sets: Vec<FeatureSetId>,
    pub profiles: Vec<BuildProfileId>,
}
```

---

# 37. Multiple Configurations

Same source snapshot may produce different dependency graphs.

---

# 38. GraphConfigId

```rust
pub struct GraphConfigId(Digest);
```

---

# 39. C/C++

Static dependency discovery can be harder.

---

# 40. Sources

Potential:

```text
CMake codemodel
compile_commands.json
declared target graph
header include scans
```

---

# 41. Header Dependencies

Can be dynamic/generated.

---

# 42. Conservative C/C++ Mode

Unknown header/codegen dependency expands impact broadly.

---

# 43. Java/JVM

Use Gradle/Maven project graph.

---

# 44. JavaScript/TypeScript

Workspace/package manager graph.

---

# 45. Python

Package/module boundaries can be weak.

Prefer declared project/package graph + optional import analysis.

---

# 46. Import Analysis

Optimization only.

---

# 47. Polyglot Graph

Merge fragments by explicit cross-ecosystem edges.

---

# 48. Cross-Language Edge

Example:

```text
Rust server
  depends on
generated protobuf
  generated from
protocol schema
```

---

# 49. Schema Node

Custom/configuration file can be first-class dependency node.

---

# 50. Generated Source

Critical for correct impact.

---

# 51. Codegen Edge

```text
generated file
  --GeneratedFrom-->
schema
```

or target dependencies.

---

# 52. Generated Output Identity

Prefer generator target rather than committing generated files if workflow.

---

# 53. Build Script

Rust `build.rs` can create broad dependencies.

---

# 54. Dynamic Build Script Inputs

Hermetic system should declare.

---

# 55. Undeclared Dynamic Input

Impact analysis cannot prove safety.

---

# 56. Strict Mode

Unknown dynamic inputs => affected broadly.

---

# 57. Graph Completeness

```rust
pub enum GraphCompleteness {
    Complete,
    DeclaredComplete,
    Partial,
    Unknown,
}
```

---

# 58. Complete

Strongly modeled.

---

# 59. Partial

Some dependency classes unknown.

---

# 60. Impact Decision Must Carry Completeness

Critical.

---

# 61. Change Set

```rust
pub struct SourceChangeSet {
    pub base: SourceSnapshotId,
    pub head: SourceSnapshotId,
    pub changes: Vec<FileChange>,
}
```

---

# 62. File Change

```rust
pub enum FileChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    TypeChanged,
}
```

---

# 63. Rename

Retain old/new path.

---

# 64. Diff Source

VCS adapter or canonical tree diff.

---

# 65. Canonical Tree Diff

VCS-neutral.

---

# 66. Dirty Workspace

Impact can compare working SourceSnapshot to base.

---

# 67. Base Resolution

Exact snapshot.

---

# 68. No Branch String After Resolution

Critical.

---

# 69. Direct Impact

Changed file owners.

---

# 70. DirectImpactSet

```rust
pub struct DirectImpactSet {
    pub changed_files: BTreeSet<RepoRelativePath>,
    pub directly_affected: BTreeSet<GraphNodeId>,
    pub unknown_files: BTreeSet<RepoRelativePath>,
}
```

---

# 71. Unknown File

Policy chooses broad expansion.

---

# 72. Examples

Top-level:

```text
Cargo.lock
rust-toolchain.toml
CI config
shared schema
```

can affect broad graph.

---

# 73. Global Impact Pattern

Configured.

---

# 74. GlobalImpactRule

```rust
pub struct GlobalImpactRule {
    pub path: PathPattern,
    pub scope: ImpactScope,
}
```

---

# 75. Impact Scope

```rust
pub enum ImpactScope {
    Repository,
    Workspace(WorkspaceId),
    Package(SourcePackageId),
    Target(TargetLogicalId),
}
```

---

# 76. Lockfile Change

Usually broad within ecosystem workspace.

---

# 77. Toolchain Change

Potential repository-wide.

---

# 78. Policy File Change

May require policy/check re-evaluation broadly.

---

# 79. Pipeline Config Change

Can affect execution plan independently of source graph.

---

# 80. Architecture Rule

Pipeline/config impact evaluated separately and unioned.

---

# 81. Reverse Dependency Expansion

```text
changed dependency
  ↓
all reverse dependents
```

---

# 82. Affected Set

```rust
pub struct AffectedSet {
    pub graph: RepositoryGraphId,
    pub change: SourceChangeSetId,
    pub nodes: BTreeSet<GraphNodeId>,
    pub reasons: BTreeMap<GraphNodeId, Vec<ImpactReason>>,
    pub confidence: ImpactConfidence,
}
```

---

# 83. Impact Reason

Examples:

```text
DirectFileChange
ReverseDependency
GlobalRule
GeneratedSource
UnknownOwnership
PolicyExpansion
ManualInclude
```

---

# 84. Explainability

Every affected target should be explainable.

---

# 85. `forgeyard affected explain`

Shows dependency chain.

---

# 86. Impact Confidence

```rust
pub enum ImpactConfidence {
    ProvenConservative,
    ConservativeWithUnknowns,
    Heuristic,
    Unknown,
}
```

---

# 87. ProvenConservative

Graph contains enough information and algorithm never excludes possible dependent.

---

# 88. Heuristic

Cannot be used for high-assurance skipping unless policy opts in.

---

# 89. Unaffected Set

Be careful.

---

# 90. Claim

Forgeyard should say:

```text
not selected by current impact analysis
```

rather than absolute "unaffected" when confidence limited.

---

# 91. IncrementalPlanId

```rust
pub struct IncrementalPlanId(Digest);
```

---

# 92. Incremental Plan

```rust
pub struct IncrementalExecutionPlan {
    pub id: IncrementalPlanId,
    pub pipeline_plan: PipelinePlanId,
    pub graph: RepositoryGraphId,
    pub change: SourceChangeSetId,
    pub selected_jobs: BTreeSet<JobIrId>,
    pub skipped_jobs: BTreeMap<JobIrId, SkipReason>,
}
```

---

# 93. Pipeline Mapping

Jobs can declare affected target selectors.

---

# 94. Job Affected Selector

```rust
pub enum AffectedSelector {
    Repository,
    Workspace(WorkspaceSelector),
    Targets(Vec<TargetSelector>),
    Tests(TestSelector),
    Artifacts(ArtifactSelector),
}
```

---

# 95. Example Pipeline

Conceptually:

```ron
job: (
    name: "rust-tests",
    affected: Targets(["//crates/..."]),
)
```

---

# 96. Do Not Make Human Config Overly Complex

Adapters can infer common selectors.

---

# 97. Job Selection

If affected set intersects selector, run.

---

# 98. Required Jobs

Policy can force run regardless impact.

---

# 99. Job Classification

```rust
pub enum IncrementalJobMode {
    Always,
    AffectedOnly,
    AffectedOrPolicy,
    Manual,
}
```

---

# 100. Always

Security/release/global checks.

---

# 101. AffectedOnly

Safe optimization where permitted.

---

# 102. AffectedOrPolicy

Default flexible.

---

# 103. Manual

Explicit.

---

# 104. Policy Expansion

Policy can union additional jobs.

---

# 105. Example

Change Proposal:

```text
affected unit tests
+
always lint/security
```

Stable release:

```text
all Tier-1 test suites
```

---

# 106. No Policy Contraction by Impact Engine

Critical.

Impact engine proposes minimum/selected set; policy may expand.

---

# 107. Required Work Set

```text
FinalWorkSet =
AffectedWork
∪ AlwaysRun
∪ PolicyRequired
∪ UserExplicit
```

---

# 108. Never Subtract PolicyRequired

Critical.

---

# 109. Test Impact

Part 32 integration.

---

# 110. Direct Test Mapping

Tests belong to target/package.

---

# 111. Coverage-Based Test Impact

Future optional.

---

# 112. Historical Test Correlation

Optional heuristic.

---

# 113. Baseline Safe Test Selection

```text
all tests for affected targets
```

---

# 114. More Aggressive Selection

Can use:

```text
test-to-file coverage map
dependency graph
historical mapping
```

but confidence lower.

---

# 115. Mandatory Test Floor

Policy.

---

# 116. No Opaque AI-Only Selection

Critical.

---

# 117. Benchmark Impact

Part 33 integration.

---

# 118. Performance Benchmarks

Run only for affected relevant targets in PR, broader on release/nightly.

---

# 119. Artifact Impact

Build artifact target graph.

---

# 120. Release Impact

Can answer:

```text
which packages/releases potentially changed?
```

---

# 121. But Release Completeness

Stable release may rebuild/repackage all required targets from same source according to policy.

---

# 122. Cache Integration

Impact and cache are distinct.

---

# 123. Affected But Cache Hit

Job selected, then cache may satisfy without execution.

---

# 124. Unaffected But Cache Miss

If policy allows affected-only, job may remain skipped.

---

# 125. Correct Order

```text
impact/policy selects logical work
  ↓
cache checks selected work
  ↓
execute misses
```

---

# 126. Do Not Use Cache to Decide Semantic Impact

Critical.

---

# 127. Derivation Integration

Hermetic derivations already encode inputs.

---

# 128. Fine-Grained Incremental Build

Can use derivation dependencies where ecosystem adapter maps them.

---

# 129. Source Digest Inputs

Target derivation can hash relevant source subset.

---

# 130. Avoid Entire Repo Hash

Where safe.

---

# 131. TargetInputSet

```rust
pub struct TargetInputSet {
    pub target: TargetLogicalId,
    pub files: BTreeSet<RepoRelativePath>,
    pub dependencies: BTreeSet<TargetLogicalId>,
    pub config_inputs: BTreeSet<RepoRelativePath>,
}
```

---

# 132. Input Set Completeness

Versioned/confidence.

---

# 133. Hermetic Advantage

Declared inputs improve impact precision.

---

# 134. Dynamic Inputs

Reduce confidence.

---

# 135. CAS Graph Storage

Repository graph snapshot can be canonical serialized into CAS.

---

# 136. Metadata

Graph ID/source/config/completeness stored in DB.

---

# 137. Graph Size

Large monorepos can have millions of nodes/edges.

---

# 138. In-Memory Representation

Use compact indexes/arenas.

---

# 139. Serialization

Postcard suitable internally.

---

# 140. Query Indexes

Maintain:

```text
node by logical ID
file -> owners
node -> deps
node -> reverse deps
package -> targets
target -> tests
```

---

# 141. BTree vs Hash

Deterministic serialization uses ordered structures.

Runtime indexes can use hash maps.

---

# 142. Graph Construction Parallelism

Rayon appropriate for CPU-heavy parsing/merging.

---

# 143. Tokio

I/O/discovery orchestration.

---

# 144. Incremental Graph Rebuild

Future optimization.

---

# 145. Baseline

Rebuild graph per source snapshot or cache by manifest/input digests.

---

# 146. Graph Fragment Cache

Adapter fragment cache.

---

# 147. Fragment Cache Key

```text
adapter version
manifest digests
relevant config
toolchain metadata
```

---

# 148. Graph Cache Is Derived

Can discard/rebuild.

---

# 149. Graph Snapshot Reuse

If same SourceSnapshotId + GraphConfigId + adapter versions, reuse exact graph.

---

# 150. Graph Diff

```rust
pub struct GraphDiff {
    pub base: RepositoryGraphId,
    pub head: RepositoryGraphId,
    pub added_nodes: Vec<GraphNodeId>,
    pub removed_nodes: Vec<GraphNodeId>,
    pub changed_edges: Vec<GraphEdgeChange>,
}
```

---

# 151. Graph Structural Change

Can itself expand affected scope.

---

# 152. Example

Dependency added:

```text
A now depends on B
```

A affected even if A source file unchanged.

---

# 153. Manifest Change

Directly affects package/target graph.

---

# 154. Deleted Target

Downstream pipeline jobs may become invalid.

---

# 155. Plan Validation

Incremental planner checks selected job references still exist.

---

# 156. Cycles

Some dependency graphs contain cycles.

---

# 157. Strongly Connected Components

Collapse SCCs.

---

# 158. Impact

If one node in SCC changes, all SCC members affected.

---

# 159. Build Scheduling

Underlying build system may handle cycles or reject.

---

# 160. Forgeyard Graph

Must represent without infinite traversal.

---

# 161. GraphCycleId

Optional.

---

# 162. Cycle Diagnostic

Show path.

---

# 163. Invalid Cycle

Adapter may mark error.

---

# 164. Ownership Overlap

One file can affect multiple targets.

---

# 165. Shared Config File

Expected.

---

# 166. Glob Ownership

Declared patterns.

---

# 167. Glob Changes

If ownership rules change, graph rebuild.

---

# 168. Ignore Rules

VCS ignore does not automatically mean impact-ignore.

---

# 169. Forgeyard Ignore

Separate config for non-impact files:

```text
docs-only
examples
generated outputs
```

---

# 170. Ignore Safety

Strict policy.

---

# 171. Docs Change

May still affect docs site/package.

---

# 172. No Broad `.md` Ignore by Default

Critical.

---

# 173. Manual Include

Pipeline/user can force targets.

---

# 174. Manual Exclude

High risk.

---

# 175. Recommendation

Do not allow user arbitrary exclusion from required policy jobs.

---

# 176. Impact Override

If permitted:

```rust
pub struct ImpactOverride {
    pub scope: ResourceScope,
    pub include: BTreeSet<TargetLogicalId>,
    pub exclude: BTreeSet<TargetLogicalId>,
    pub reason: BoundedString,
}
```

---

# 177. Exclude Permission

High privilege.

---

# 178. Audit

Mandatory.

---

# 179. Release

Exclusion probably forbidden by default.

---

# 180. Monorepo Ownership

Can answer:

```text
who owns affected components?
```

---

# 181. Ownership Source

Change Proposal ownership/CODEOWNERS-like system.

---

# 182. Component Owner

Not authorization itself.

---

# 183. Review Routing

Can inform required reviewers.

---

# 184. Policy

May require owners for affected components.

---

# 185. Component Boundary

```rust
pub struct ComponentId(Digest);
```

Higher-level grouping across packages/targets.

---

# 186. Component

Examples:

```text
scheduler
runner
web UI
Android app
```

---

# 187. Component Graph

Derived view.

---

# 188. Architectural Boundary

Can integrate architecture.ron.

---

# 189. Dependency Rule

Forbidden edges can be checked.

---

# 190. Architecture Lint

Part of affected validation.

---

# 191. If changed component adds forbidden dependency

Fail architecture check.

---

# 192. Search Integration

Part 31 indexes:

```text
target
package
component
dependencies
owners
```

---

# 193. Graph Query API

Examples:

```text
deps(target)
rdeps(target)
owners(file)
affected(base, head)
path(A, B)
```

---

# 194. Query Language

Typed, bounded.

---

# 195. No Arbitrary Graph Program

Baseline.

---

# 196. CLI

```text
forgeyard graph build
forgeyard graph show
forgeyard graph deps <target>
forgeyard graph rdeps <target>
forgeyard graph path <a> <b>
forgeyard affected --base <ref>
forgeyard affected explain <target>
forgeyard affected jobs
```

---

# 197. `graph show`

Summary.

---

# 198. `graph deps`

Direct/transitive option.

---

# 199. Depth Limit

Bounded.

---

# 200. `affected`

Shows:

```text
changed files
direct targets
reverse dependents
selected jobs
policy expansions
confidence
```

---

# 201. Dioxus UI

Pages:

```text
Repository Graph
Components
Affected Changes
Dependency Explorer
Impact Explanation
```

---

# 202. Change Proposal UI

Affected components panel.

---

# 203. Pipeline Plan UI

Show:

```text
selected
skipped due to unaffected
forced by policy
cache-hit
```

---

# 204. Skip Transparency

Critical.

---

# 205. User Must Know Why Job Did Not Run

---

# 206. SkipReason

```rust
pub enum SkipReason {
    NotAffected,
    PolicyNotRequired,
    ReplacedByEquivalentEvidence,
    UserExcludedAuthorized,
}
```

---

# 207. Cache Hit Is Not SkipReason

It's execution result path.

---

# 208. Job State

Can use existing Skipped semantics with structured reason.

---

# 209. Incremental Evidence

Store selected/skipped reasoning.

---

# 210. IncrementalPlanningEvidenceId

```rust
pub struct IncrementalPlanningEvidenceId(Digest);
```

---

# 211. Change Proposal Check

Can publish:

```text
Affected: 12/248 targets
Required jobs: 18
Skipped: 43
```

---

# 212. No Marketing "100x Faster" Claims Without Evidence

---

# 213. Analytics

Track:

```text
jobs skipped
compute saved
cache hits
impact expansion ratio
false-negative incidents
```

---

# 214. False Negative

Important.

---

# 215. Definition

Impact analysis skipped work that later proves affected.

---

# 216. Detection

Can use periodic full-validation comparison.

---

# 217. Shadow Validation

Run full suite periodically and compare affected plan.

---

# 218. Impact Confidence Calibration

Excellent production mechanism.

---

# 219. Nightly Full Validation

Recommended.

---

# 220. Release Full Validation

Recommended for critical projects.

---

# 221. False-Negative Record

```rust
pub struct ImpactMiss {
    pub graph: RepositoryGraphId,
    pub change: SourceChangeSetId,
    pub skipped_target: TargetLogicalId,
    pub evidence: ImpactMissEvidence,
}
```

---

# 222. Impact Miss

Triggers:

```text
warning
adapter correction
confidence downgrade
```

---

# 223. No Silent Miss Suppression

Critical.

---

# 224. Graph Adapter Quality Metrics

```text
completeness
unknown ownership rate
impact miss rate
```

---

# 225. Health

Checks:

```text
graph build success
projection lag
unknown-file ratio
adapter errors
impact miss rate
```

---

# 226. Doctor

```text
forgeyard graph doctor
```

---

# 227. Doctor Checks

```text
workspace manifests
adapter availability
graph completeness
cycles
orphan files
unknown ownership
global rules
```

---

# 228. Orphan File

Tracked source file with no owner.

---

# 229. Orphan Policy

Can warn/fail depending repo.

---

# 230. Config

```ron
(
    graph: (
        global_impact: [
            "Cargo.lock",
            "rust-toolchain.toml",
            ".forgeyard/**",
        ],
    ),
)
```

---

# 231. Path Pattern Engine

Bounded, deterministic.

---

# 232. No Arbitrary Regex Needed

Globs sufficient baseline.

---

# 233. Security

Manifest parsers untrusted.

---

# 234. External Build Tool Invocation

If adapter invokes:

```text
cargo metadata
gradle
bazel query
```

run sandboxed/bounded.

---

# 235. Network

Discovery defaults network denied where possible.

---

# 236. Generated Config

Do not execute arbitrary project code in daemon.

---

# 237. Discovery Worker

Can run as sandboxed job for risky ecosystems.

---

# 238. Daemon Never Executes Arbitrary Build Scripts Directly

Existing invariant.

---

# 239. Graph Adapter Trust

Built-in first-party adapters preferred.

---

# 240. Plugin Graph Adapter

Possible trusted external, not sandboxed result accepted blindly.

---

# 241. Host Validates Graph Bounds

---

# 242. Graph Bomb

Malicious repo could generate enormous graph.

---

# 243. Limits

```text
max nodes
max edges
max path length
max manifest count
```

---

# 244. Quotas

Part 27 resource governance.

---

# 245. Tenant Isolation

Graphs belong to tenant/project.

---

# 246. Cross-Tenant Graph

Forbidden.

---

# 247. Public OSS Mirror

Still own tenant/project graph.

---

# 248. Graph Data Sensitivity

Package/target names may reveal proprietary architecture.

---

# 249. Authz

Graph read permission.

---

# 250. Permissions

```text
graph.read
graph.build
impact.read
impact.override
```

---

# 251. API

Potential:

```text
GET  /v1/projects/{id}/graph
GET  /v1/projects/{id}/graph/dependencies
GET  /v1/projects/{id}/affected
GET  /v1/runs/{id}/incremental-plan
POST /v1/projects/{id}/graph/rebuild
```

---

# 252. Large Graph Response

Paginated/streamed/subgraph only.

---

# 253. Graph Export

RON/Postcard internal; JSON external.

---

# 254. Graph Visualization

UI fetches bounded subgraph.

---

# 255. No Entire Million-Node Graph to Browser

Critical.

---

# 256. Graph Rebuild

Async.

---

# 257. Rebuild Result

New RepositoryGraphId.

---

# 258. Graph Cache

Derived; backup optional.

---

# 259. DR

Rebuild from SourceSnapshot and config.

---

# 260. Source Retention

Graph reproduction requires source snapshot retained.

---

# 261. Historical Incremental Plan

Store graph/evidence IDs.

---

# 262. Missing Old Graph

Can still inspect stored IncrementalPlanningEvidence.

---

# 263. Supply Chain

Could include graph/impact evidence in change checks, not release provenance baseline unless desired.

---

# 264. RBE

Bazel already has fine-grained action graph.

---

# 265. RBE Integration

Do not duplicate Bazel internals blindly.

---

# 266. REAPI Action Graph

Can use RBE action-level caching independently.

---

# 267. Forgeyard Monorepo Graph

Higher-level workspace/pipeline planning.

---

# 268. Bazel Adapter

Can import target graph via `query/cquery` semantics if supported.

---

# 269. Cache

Bazel action cache remains lower level.

---

# 270. Scheduler

Selected jobs only become scheduler jobs.

---

# 271. Fairness

Unchanged.

---

# 272. Change Proposal

Exact base/head snapshots.

---

# 273. Merge Queue

Target branch may advance.

---

# 274. Integration Candidate

Recompute affected set against exact integration candidate/base as needed.

---

# 275. Critical

Affected analysis based on old base may be stale after queue rebase.

---

# 276. Merge Queue Rule

Re-plan/re-evaluate when target revision changes.

---

# 277. IncrementalPlan Freshness

```rust
pub enum IncrementalPlanFreshness {
    Current,
    StaleBase,
    StaleGraph,
    StalePolicy,
}
```

---

# 278. Stale Plan

Cannot authorize final merge if policy requires current.

---

# 279. Release Candidate

Usually full required validation.

---

# 280. Nightly

Good place for broad/full tests to validate impact system.

---

# 281. Local Developer Mode

Affected analysis useful:

```text
forgeyard run --affected
```

---

# 282. Standalone

Graph stored locally.

---

# 283. Distributed

Shared graph metadata/CAS.

---

# 284. Local Uncommitted Changes

Canonical working-tree snapshot.

---

# 285. Base

User selects/ref resolved exactly.

---

# 286. Developer Explain

Show why local command runs target.

---

# 287. Incremental Build UX

Potential:

```text
3 files changed
7 targets affected
12 jobs required
8 cache hits
4 executed
```

---

# 288. Observability Metrics

```text
graph_build_duration_seconds
graph_nodes
graph_edges
graph_unknown_files
impact_targets_selected
impact_jobs_skipped
impact_policy_expansions
impact_misses_total
graph_rebuild_failures_total
```

---

# 289. Labels

Low cardinality:

```text
ecosystem
result
confidence
```

---

# 290. No Target IDs in metrics

Use analytics/search.

---

# 291. Tracing

```text
graph.discover
graph.merge
graph.query
impact.compute
incremental.plan
graph.reconcile
```

---

# 292. Event

```text
RepositoryGraphBuilt
RepositoryGraphFailed
ImpactComputed
ImpactMissDetected
```

---

# 293. Audit

Audit only:

```text
impact override
graph admin config change
```

not every normal graph computation.

---

# 294. Notifications

Potential:

```text
impact system degraded
high impact miss rate
required full validation
```

---

# 295. Search/Analytics

Part 31.

---

# 296. Testkit

```text
forgeyard-graph-testkit/src/
├── lib.rs
├── graph.rs
├── fragment.rs
├── diff.rs
├── impact.rs
├── cycles.rs
├── policy.rs
└── assertions.rs
```

---

# 297. Unit Tests

Graph traversal.

---

# 298. Reverse Dependency Test

Direct change expands correctly.

---

# 299. Cycle Test

SCC handled.

---

# 300. Unknown File Test

Conservative expansion.

---

# 301. Lockfile Test

Workspace-wide impact.

---

# 302. Toolchain Test

Configured global impact.

---

# 303. Generated Source Test

Schema change affects generated consumers.

---

# 304. Graph Structural Change Test

Manifest edge change affects dependent target.

---

# 305. Base Change Test

Stale incremental plan rejected.

---

# 306. Merge Queue Test

Target-base update triggers recomputation.

---

# 307. Policy Expansion Test

Policy-required job never removed.

---

# 308. Cache Order Test

Impact selects before cache resolution.

---

# 309. Test Impact Test

Affected target selects associated tests.

---

# 310. Release Full Validation Test

Policy expands beyond PR impact.

---

# 311. Heuristic Test

Heuristic confidence cannot satisfy strict skip policy.

---

# 312. Impact Explain Test

Every selected target has reason chain.

---

# 313. Orphan File Test

Reported.

---

# 314. Path Rename Test

Correct old/new ownership.

---

# 315. Deleted Package Test

Graph/planner handles.

---

# 316. Polyglot Test

Cross-language edge propagates.

---

# 317. Graph Bomb Test

Limits enforced.

---

# 318. Tenant Isolation Test

No cross-tenant graph access.

---

# 319. DR Test

Graph rebuilt from retained source.

---

# 320. Shadow Validation Test

Detect skipped target miss.

---

# 321. Fuzzing

Fuzz graph serializers, path/glob parser, query parser.

---

# 322. Property Tests

Transitive affected closure contains all reverse dependents of direct impacted nodes.

---

# 323. Scale Tests

Millions of nodes/edges.

---

# 324. Large Monorepo Test

Measure:

```text
discovery
graph build
affected query
reverse traversal
```

---

# 325. Implementation Phase 1 — Core Graph Model

Nodes/edges/config/completeness.

---

# 326. Phase 2 — Rust/Cargo Adapter

First-class dogfood.

---

# 327. Phase 3 — Source Diff & Direct Impact

File ownership.

---

# 328. Phase 4 — Reverse Dependency Impact

Affected targets.

---

# 329. Phase 5 — Pipeline Incremental Planning

Selected/skipped jobs.

---

# 330. Phase 6 — Policy Expansion

Strict correctness.

---

# 331. Phase 7 — Test/Benchmark Integration

Affected quality work.

---

# 332. Phase 8 — Polyglot Adapters

Go/JS/Python/JVM/C++ etc.

---

# 333. Phase 9 — UI/CLI/Search

Developer experience.

---

# 334. Phase 10 — Shadow Validation/Impact Miss Intelligence

Trust calibration.

---

# 335. Phase 11 — Large-Monorepo Optimization

Fragment cache/parallelism.

---

# 336. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 337. Acceptance Tests

1. Every graph binds exact SourceSnapshotId.
2. Graph configuration/version is part of identity.
3. Ecosystem-specific types do not leak into core graph model.
4. File ownership confidence is explicit.
5. Unknown ownership causes conservative handling.
6. Global-impact files are configurable and explainable.
7. Dependency edges have explicit semantics.
8. Reverse-dependency traversal drives affected expansion.
9. Generated-source relationships propagate impact.
10. Graph structural changes can affect targets even without source-file edits.
11. Cycles are handled via SCCs without infinite traversal.
12. Affected target reason chains are explainable.
13. Impact confidence is explicit.
14. Heuristic impact cannot silently satisfy strict release skipping.
15. Final work set always includes policy-required jobs.
16. Impact engine never subtracts policy-required work.
17. Impact selection occurs before cache resolution.
18. Cache hits do not define semantic impact.
19. Test selection has a policy-defined mandatory floor.
20. Benchmark selection can be narrower for PR and broader for release.
21. Merge-queue base changes invalidate/recompute affected plans.
22. Stale graph/base/policy marks IncrementalPlan stale.
23. Incremental skip reasons are persisted and visible.
24. Users can explain why a job ran or skipped.
25. Impact overrides are scoped/audited.
26. Daemon never executes arbitrary project build scripts directly for discovery.
27. Malicious repositories cannot create unbounded graph structures.
28. Tenant graph data is isolated.
29. Search/analytics are derived from graph state.
30. Graph can rebuild after DR.
31. Shadow/full validation can detect impact misses.
32. Impact misses reduce trust/confidence rather than being hidden.
33. Standalone/distributed share graph semantics.
34. Polyglot cross-language dependencies can be represented.
35. Forgeyard dogfoods monorepo impact analysis on its own Rust workspace.

---

# 338. Production Readiness Gates

Do not call monorepo incremental execution production-ready until:

```text
exact graph identity is stable
Rust workspace graph is accurate
unknown ownership is conservative
global-impact rules are explicit
reverse-dependency closure is tested
policy expansion cannot be bypassed
stale base/graph/policy invalidates plans
impact explanations are persisted
shadow/full validation measures misses
large-repository limits and fuzz tests pass
```

---

# 339. Architectural Invariants

1. repository graph is derived state;
2. source snapshot is immutable/exact;
3. graph identity includes config/schema/adapter semantics;
4. impact analysis is optimization, not unconditional correctness authority;
5. unknown dependency means conservative expansion;
6. ownership confidence is explicit;
7. heuristic results cannot masquerade as proof;
8. policy can always expand required work;
9. impact engine never removes policy-required work;
10. selected work is determined before cache lookup;
11. cache does not define semantic impact;
12. generated-source relationships are explicit;
13. structural graph changes are first-class;
14. cycles cannot break traversal;
15. skip reasons are durable/explainable;
16. stale base/graph/policy invalidates incremental plan;
17. merge-queue target movement triggers recomputation;
18. test selection retains mandatory policy floor;
19. release validation can be broader than PR validation;
20. impact overrides are privileged/audited;
21. discovery never gives arbitrary repository code daemon authority;
22. graph input/output size is bounded;
23. tenant graphs are isolated;
24. graph can be rebuilt from source/config;
25. shadow validation detects false-negative impact;
26. misses are recorded, not hidden;
27. polyglot graphs share one normalized model;
28. standalone/distributed share semantics;
29. graph/search/analytics never become authz authority;
30. Forgeyard dogfoods affected-change planning on its own monorepo.

---

# 340. Final Target Architecture

```text
                   SourceSnapshotId
                          │
                          ▼
                   Graph Discovery
                          │
                          ▼
                  Repository Graph
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
           Files       Targets        Tests
             │            │            │
             └────────────┼────────────┘
                          ▼
                     Source Diff
                          │
                          ▼
                    Impact Closure
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
         Affected      AlwaysRun    PolicyRequired
             │            │            │
             └────────────┼────────────┘
                          ▼
                  Final Work Set
                          │
                          ▼
                      Cache Check
                          │
                          ▼
                       Execute
```

---

# 341. Final Architectural Position

Impact analysis:

```text
exact base snapshot
+
exact head snapshot
+
versioned repository graph
+
changed files
  ↓
direct ownership
  ↓
reverse dependency closure
  ↓
confidence + reasons
```

Incremental planning:

```text
AffectedWork
∪ AlwaysRun
∪ PolicyRequired
∪ ExplicitIncludes
  ↓
selected logical jobs
  ↓
cache resolution
  ↓
execution
```

Safety calibration:

```text
incremental PR validation
  ↓
periodic/full validation
  ↓
compare results
  ↓
detect ImpactMiss
  ↓
fix adapter/rules
  ↓
adjust confidence
```

The key guarantee is:

> **Forgeyard can scale efficiently to large monorepos without converting change-impact heuristics into hidden correctness assumptions. Every skipped job is explainable, policy can always require broader validation, unknown dependencies expand conservatively, and periodic full validation measures whether the optimization remains trustworthy.**

---

# 342. Extended Architecture Sequence

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
```
