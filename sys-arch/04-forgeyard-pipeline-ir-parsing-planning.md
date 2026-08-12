# 04 — Forgeyard Pipeline IR, Parsing, Normalization & Planning System Architecture

**Document type:** Core Orchestration System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Pipeline configuration, parser, schema/versioning, validation, normalization, templates, matrices, DAG construction, conditions, environment/secrets contracts, ecosystem integration, capability planning, cache planning, policy injection, deterministic Pipeline IR, diagnostics, import adapters, and executable plan generation  
**Architecture style:** Deterministic compiler-like pipeline from human configuration to canonical executable IR  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on `01-forgeyard-core-domain-foundation.md`, `02-forgeyard-storage-metadata.md`, and `03-forgeyard-cas-artifact-data-plane.md`. It assumes the previously defined hermetic/reproducible build architecture, VCS-neutral source model, Change Proposal architecture, workspace structure, and language ecosystem adapters.

---

# 1. Purpose

Forgeyard needs one canonical way to represent CI/CD workflows independent of:

```text
human syntax
external CI provider
language ecosystem
runner platform
local/distributed mode
```

The pipeline subsystem should behave like a compiler:

```text
human config / imported config
        ↓
parse
        ↓
validate
        ↓
normalize
        ↓
expand templates/matrices
        ↓
construct DAG
        ↓
resolve conditions/capabilities
        ↓
apply policy
        ↓
derive cache/action plans
        ↓
canonical PipelineIr
        ↓
ExecutablePlan
```

The central rule is:

> **Forgeyard executes only canonical validated Pipeline IR, never raw user configuration directly.**

---

# 2. Architectural Position

```text
        Human RON / Imported CI
                 │
                 ▼
              Parser
                 │
                 ▼
             Validator
                 │
                 ▼
            Normalizer
                 │
         ┌───────┼────────┐
         ▼       ▼        ▼
     Templates  Matrix  Conditions
         │       │        │
         └───────┼────────┘
                 ▼
               DAG
                 │
                 ▼
        Ecosystem/Capability Plan
                 │
                 ▼
               Policy
                 │
                 ▼
             PipelineIr
                 │
                 ▼
          ExecutablePlan
```

---

# 3. Goals

The subsystem MUST:

1. support human-authored RON pipeline configuration;
2. support imported external CI systems;
3. produce one canonical `PipelineIr`;
4. be deterministic;
5. support schema versioning;
6. validate types and semantics;
7. normalize aliases/defaults;
8. support reusable templates;
9. support matrix expansion;
10. support stages and DAG dependencies;
11. detect cycles;
12. support conditions;
13. support manual/approval gates;
14. support artifacts;
15. support cache declarations;
16. support service/dependency jobs where appropriate;
17. support environment variables;
18. support secret references;
19. support timeouts;
20. support retries;
21. support resources;
22. support platform/toolchain capabilities;
23. support ecosystem adapters;
24. support hermetic derivations;
25. support change-impact optimization;
26. support policy-injected requirements;
27. support local and distributed execution;
28. support explainability;
29. support rich diagnostics;
30. remain provider-neutral.

---

# 4. Non-Goals

Pipeline parser does not:

```text
execute commands
schedule runners
fetch source
resolve secrets
store artifacts
perform deployments
```

It describes and plans work.

---

# 5. Workspace Structure

```text
crates/pipeline/
├── forgeyard-pipeline/
├── forgeyard-pipeline-model/
├── forgeyard-pipeline-schema/
├── forgeyard-pipeline-parser/
├── forgeyard-pipeline-validate/
├── forgeyard-pipeline-normalize/
├── forgeyard-pipeline-template/
├── forgeyard-pipeline-matrix/
├── forgeyard-pipeline-condition/
├── forgeyard-pipeline-dag/
├── forgeyard-pipeline-ir/
├── forgeyard-pipeline-capability/
├── forgeyard-pipeline-cache/
├── forgeyard-pipeline-policy/
├── forgeyard-pipeline-plan/
├── forgeyard-pipeline-import/
├── forgeyard-pipeline-diagnostic/
├── forgeyard-pipeline-explain/
└── forgeyard-pipeline-testkit/
```

---

# 6. `forgeyard-pipeline-model`

Human-facing parsed model, not canonical execution model.

Tree:

```text
src/
├── lib.rs
├── pipeline.rs
├── stage.rs
├── job.rs
├── step.rs
├── command.rs
├── dependency.rs
├── condition.rs
├── environment.rs
├── secret.rs
├── artifact.rs
├── cache.rs
├── resource.rs
├── timeout.rs
├── retry.rs
├── matrix.rs
├── template.rs
├── trigger.rs
└── error.rs
```

---

# 7. Human Pipeline Example

Illustrative RON:

```ron
(
    schema: 1,

    name: "ci",

    triggers: [
        Push(
            branches: ["main"],
        ),
        ChangeProposal,
    ],

    jobs: {
        "test": (
            ecosystem: Rust,

            steps: [
                Run("cargo test --workspace"),
            ],

            resources: (
                cpu: 4,
                memory: "8GiB",
            ),
        ),
    },
)
```

RON is human syntax.

It is not the internal execution representation.

---

# 8. Schema Version

```rust
pub struct PipelineSchemaVersion(u16);
```

Every pipeline config declares or infers a version according to strict rules.

Recommended:

```text
explicit version required for production configs
```

---

# 9. Parser

```text
forgeyard-pipeline-parser
```

Responsibilities:

```text
RON decoding
source spans
syntax errors
unknown fields
basic shape validation
```

---

# 10. Source Spans

Diagnostics should preserve:

```text
file
line
column
span
```

through parsing/validation where possible.

---

# 11. Parser Output

```rust
pub struct ParsedPipeline {
    pub schema: PipelineSchemaVersion,
    pub source: PipelineSourceLocation,
    pub model: PipelineConfig,
    pub spans: SpanTable,
}
```

---

# 12. Parser Is Not Validator

Syntactically valid:

```ron
cpu: 0
```

may still be semantically invalid.

Parser accepts shape.

Validator rejects semantics.

---

# 13. Schema Layer

```text
forgeyard-pipeline-schema
```

Owns:

```text
schema version
deprecations
field migrations
compatibility
```

---

# 14. Schema Migration

Old config:

```text
v1
```

may normalize/migrate to current intermediate form.

Do not mutate user file silently unless explicit command:

```text
forgeyard pipeline migrate
```

---

# 15. Deprecation

Warnings:

```text
field deprecated
replacement
removal version
```

---

# 16. Validator

```text
forgeyard-pipeline-validate
```

Validation categories:

```text
syntax-adjacent
semantic
graph
resource
security
platform
ecosystem
policy-aware
```

---

# 17. Validation Phases

Recommended:

```text
1. names/IDs
2. references
3. step structure
4. resource constraints
5. template/matrix validity
6. DAG dependencies
7. condition type-check
8. environment/secrets
9. ecosystem declarations
10. policy prerequisites
```

---

# 18. Validation Result

```rust
pub struct ValidationResult<T> {
    pub value: Option<T>,
    pub diagnostics: Vec<Diagnostic>,
}
```

Support multiple diagnostics per run rather than fail on first issue.

---

# 19. Diagnostic Severity

```rust
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
    Help,
}
```

---

# 20. Diagnostic Code

Stable machine-readable:

```text
FYPIPE001
FYPIPE002
...
```

Useful for docs/tests/UI.

---

# 21. Name Types

```rust
pub struct PipelineName(BoundedString);
pub struct JobName(BoundedString);
pub struct StepName(BoundedString);
pub struct TemplateName(BoundedString);
```

No arbitrary map keys without validation.

---

# 22. Stable Internal IDs

After normalization:

```rust
pub struct PipelineNodeId(Digest);
pub struct JobNodeId(Digest);
pub struct StepNodeId(Digest);
```

Can derive from canonical path/name + schema where appropriate.

---

# 23. User Names vs Internal IDs

User renaming a job may intentionally change identity.

Internal IDs should remain deterministic for same normalized config.

---

# 24. Normalizer

```text
forgeyard-pipeline-normalize
```

Responsibilities:

```text
defaults
aliases
canonical ordering
explicit values
schema migration application
normalized selectors
normalized resources
```

---

# 25. Normalization Rule

After normalization:

```text
no implicit defaults remain
```

Example:

```text
timeout omitted
```

becomes:

```text
timeout = project/default resolved value
```

where appropriate.

---

# 26. Deterministic Normalization

Same semantic input:

```text
same canonical normalized model
```

regardless of:

```text
map iteration
field order
equivalent aliases
```

---

# 27. Templates

```text
forgeyard-pipeline-template
```

Supports reusable job/step definitions.

---

# 28. Template Definition

```rust
pub struct PipelineTemplate {
    pub name: TemplateName,
    pub parameters: Vec<TemplateParameter>,
    pub body: TemplateBody,
}
```

---

# 29. Template Parameter Types

```rust
pub enum TemplateParameterType {
    String,
    Bool,
    Integer,
    Duration,
    Path,
    Platform,
    Ecosystem,
    List(Box<TemplateParameterType>),
}
```

Avoid untyped string substitution.

---

# 30. Template Expansion

```text
template
+
typed args
  ↓
expanded normalized nodes
```

---

# 31. Template Hygiene

Avoid accidental variable capture.

Template-local names should be scoped.

---

# 32. Recursive Templates

Default:

```text
forbidden
```

or bounded explicitly.

Prevent infinite expansion.

---

# 33. Matrix Expansion

```text
forgeyard-pipeline-matrix
```

Example:

```ron
matrix: (
    os: ["linux", "windows"],
    rust: ["stable", "beta"],
)
```

produces combinations.

---

# 34. Matrix Product

```text
linux/stable
linux/beta
windows/stable
windows/beta
```

---

# 35. Matrix Include/Exclude

Support:

```text
exclude combinations
include special combinations
```

---

# 36. Matrix Explosion Protection

Bound:

```text
max expanded jobs
```

Policy/configurable.

---

# 37. Matrix Identity

Each expanded job gets deterministic dimension identity:

```text
job:test[os=linux,rust=stable]
```

canonical ordering.

---

# 38. Stages

Stages may provide UX grouping:

```text
build
test
package
release
```

But execution authority is DAG dependencies.

---

# 39. DAG

```text
forgeyard-pipeline-dag
```

Canonical graph:

```rust
pub struct PipelineDag {
    pub nodes: BTreeMap<JobNodeId, JobIr>,
    pub edges: Vec<JobDependency>,
}
```

---

# 40. Dependency Edge

```rust
pub struct JobDependency {
    pub from: JobNodeId,
    pub to: JobNodeId,
    pub kind: DependencyKind,
}
```

---

# 41. Dependency Kinds

```rust
pub enum DependencyKind {
    Required,
    Optional,
    Artifact,
    Service,
    Approval,
}
```

Use only semantically justified kinds.

---

# 42. Cycle Detection

Must detect:

```text
direct cycles
indirect cycles
template-induced cycles
matrix-expanded cycles
```

---

# 43. Cycle Diagnostic

Show useful path:

```text
A -> B -> C -> A
```

not merely:

```text
cycle exists
```

---

# 44. Topological Order

Computed after validation.

Not necessarily execution order because scheduler runs independent nodes concurrently.

---

# 45. Conditions

```text
forgeyard-pipeline-condition
```

Conditions should be typed expressions.

---

# 46. Condition Inputs

Examples:

```text
event kind
branch/ref
change paths
previous job result
matrix values
project labels
manual input
policy result
```

---

# 47. Avoid Shell Conditions

Bad:

```text
if: "some arbitrary shell command"
```

Use typed condition AST.

---

# 48. Condition AST

```rust
pub enum ConditionExpr {
    Bool(bool),
    Eq(ValueExpr, ValueExpr),
    Not(Box<ConditionExpr>),
    All(Vec<ConditionExpr>),
    Any(Vec<ConditionExpr>),
    ChangedPaths(PathPredicate),
    EventIs(EventKind),
    JobSucceeded(JobNodeId),
}
```

---

# 49. Condition Type Checking

Reject:

```text
compare Duration to String
```

at plan time.

---

# 50. Runtime Conditions

Some conditions depend on prior job outcomes.

Keep them in IR as typed predicates.

---

# 51. Plan-Time Conditions

Some can be evaluated early:

```text
event kind
changed paths
matrix dimension
```

and prune graph.

---

# 52. Change Impact

Pipeline integrates with:

```text
forgeyard-change-impact
```

to skip unaffected jobs safely.

---

# 53. Safe-Superset Rule

If impact uncertain:

```text
run more
```

never skip necessary work.

---

# 54. Triggers

```rust
pub enum PipelineTrigger {
    Manual,
    Push,
    ChangeProposal,
    Tag,
    Schedule,
    Api,
    ProviderEvent,
}
```

Scheduling system for cron/time is separate infrastructure.

---

# 55. Trigger Normalization

Provider-specific events normalize before pipeline selection.

---

# 56. Source Binding

Pipeline run always binds to exact:

```text
SourceSnapshotId
```

not mutable branch.

---

# 57. Pipeline Source Model

```rust
pub struct PipelineSourceIr {
    pub snapshot: SourceSnapshotId,
    pub provenance: SourceProvenanceId,
}
```

---

# 58. Environment Model

```text
forgeyard-pipeline-model/environment.rs
```

Separate:

```text
public environment
secret references
runtime-injected system values
```

---

# 59. Environment Entry

```rust
pub struct EnvironmentEntry {
    pub key: EnvironmentKey,
    pub value: EnvironmentValue,
}
```

---

# 60. Environment Values

```rust
pub enum EnvironmentValue {
    Literal(BoundedString),
    Template(TemplateValue),
    Secret(SecretRef),
    System(SystemValue),
}
```

---

# 61. Secret References

Pipeline contains:

```text
SecretRef
```

never secret bytes.

---

# 62. Secret Resolution Time

Late:

```text
runner preparing job
```

after authorization/policy.

---

# 63. Secret Cache Key Rule

Secret values must not enter public/shared cache keys.

If build output depends on secret:

```text
cache disabled
or
secret-dependent isolated policy
```

---

# 64. Environment Canonicalization

Canonical key ordering.

Duplicate keys rejected unless override semantics explicit.

---

# 65. Environment Layers

Potential precedence:

```text
system
project
pipeline
job
step
```

Resolve deterministically.

---

# 66. Protected Environment Keys

Forgeyard may reserve:

```text
FORGEYARD_*
```

and deny user override of critical internal variables.

---

# 67. Steps

Canonical step types:

```rust
pub enum StepConfig {
    Run(RunStep),
    Ecosystem(EcosystemStep),
    UploadArtifact(UploadArtifactStep),
    DownloadArtifact(DownloadArtifactStep),
    Cache(CacheStep),
    Approval(ApprovalStep),
}
```

Keep generic extensibility controlled.

---

# 68. Raw Run Step

```rust
pub struct RunStep {
    pub command: CommandSpec,
    pub working_directory: Option<CanonicalRepoPath>,
    pub environment: EnvironmentSpec,
}
```

---

# 69. Command Representation

Prefer:

```rust
pub enum CommandSpec {
    Exec {
        program: BoundedString,
        args: Vec<BoundedString>,
    },
    Shell {
        script: BoundedString,
        shell: ShellKind,
    },
}
```

---

# 70. Exec Preferred

`Exec` avoids shell quoting ambiguities.

Shell step remains necessary for general CI.

---

# 71. Shell Identity

Shell type/version is part of execution environment:

```text
bash
sh
PowerShell
cmd
```

---

# 72. Working Directory

Canonical repo/workspace relative path.

No arbitrary host absolute path in normal job config.

---

# 73. Timeouts

```rust
pub struct TimeoutSpec {
    pub job: Option<Duration>,
    pub step: Option<Duration>,
}
```

---

# 74. Retry Policy

Pipeline retry:

```rust
pub struct JobRetryPolicy {
    pub max_attempts: NonZeroU16,
    pub retry_on: RetryPredicate,
}
```

---

# 75. Retry Classes

Differentiate:

```text
infrastructure
timeout
test failure
compile failure
```

Do not retry deterministic failures by default.

---

# 76. Resources

```rust
pub struct ResourceRequest {
    pub cpu: CpuRequest,
    pub memory: MemoryRequest,
    pub disk: DiskRequest,
    pub gpu: Option<GpuRequest>,
}
```

---

# 77. Resource Request vs Limit

Can distinguish:

```text
requested
hard limit
```

depending executor/platform support.

---

# 78. Resource Validation

Reject:

```text
0 CPU
negative/overflow memory
GPU without capability model
```

---

# 79. Platform Requirement

```rust
pub struct PlatformRequirement {
    pub os: OsRequirement,
    pub architecture: ArchitectureRequirement,
    pub sdk: Vec<SdkRequirement>,
}
```

---

# 80. Capability Planning

```text
forgeyard-pipeline-capability
```

Converts job semantics into scheduler requirements.

---

# 81. Capability Sources

Requirements derive from:

```text
explicit pipeline
ecosystem adapter
platform target
native dependencies
device requirement
signing requirement
sandbox requirement
```

---

# 82. Capability Set

```rust
pub struct CapabilityRequirementSet {
    pub required: BTreeSet<CapabilityRequirement>,
    pub preferred: BTreeSet<CapabilityPreference>,
}
```

---

# 83. Required vs Preferred

Required:

```text
must have Xcode 18
```

Preferred:

```text
input CAS locality
prewarmed Rust toolchain
```

Scheduler uses only required for eligibility.

---

# 84. Ecosystem Integration

Pipeline doesn't know Cargo/Gradle details itself.

It delegates to:

```text
EcosystemAdapter
```

---

# 85. Ecosystem Step

Example:

```rust
pub struct EcosystemStep {
    pub ecosystem: EcosystemKind,
    pub operation: EcosystemOperation,
}
```

---

# 86. Ecosystem Operations

Generic examples:

```rust
pub enum EcosystemOperation {
    Resolve,
    Build,
    Test,
    Lint,
    Package,
    Publish,
    Custom(EcosystemOperationId),
}
```

Each adapter interprets its supported semantics.

---

# 87. Ecosystem Planning

```text
pipeline job
  ↓
detect/selected ecosystem
  ↓
adapter build/test/package plan
  ↓
Derivation/action nodes
```

---

# 88. Mixed Ecosystems

One job/project may use:

```text
Rust + C++
Python + Rust
Node + native addon
Flutter + Kotlin/Swift/C++
```

Planner composes adapter/native requests.

---

# 89. Generic Fallback

Project can use explicit commands without specialized adapter.

Still gets:

```text
sandbox
resources
CAS
logs
artifacts
```

---

# 90. Hermetic Integration

Pipeline planner produces:

```text
declared inputs
toolchain requirements
environment
commands
outputs
network policy
```

which hermetic subsystem converts to derivation/action identity.

---

# 91. Network Policy

```rust
pub enum NetworkPolicy {
    Deny,
    FetchOnly,
    AllowRestricted,
    Allow,
}
```

Release builds should default strict.

---

# 92. Resolve vs Build Boundary

Recommended:

```text
resolve/fetch phase may use network
build realization phase network denied
```

where ecosystem supports it.

---

# 93. Output Declaration

```rust
pub struct OutputDeclaration {
    pub name: ArtifactName,
    pub path: CanonicalWorkspacePath,
    pub kind: OutputKind,
}
```

---

# 94. Output Kinds

```rust
pub enum OutputKind {
    File,
    Directory,
    TestReport,
    Coverage,
    Package,
    Custom(OutputKindId),
}
```

---

# 95. Artifact Flow

Declared outputs become:

```text
runner capture
  ↓
CAS
  ↓
artifact metadata
```

---

# 96. Cache Declaration

Pipeline cache spec is semantic.

```rust
pub struct CacheSpec {
    pub scope: CacheScope,
    pub paths: Vec<CanonicalWorkspacePath>,
    pub policy: CachePolicy,
}
```

---

# 97. Action Cache vs Mutable Cache

Distinguish:

```text
action cache
mutable workspace/tool cache
```

Hermetic/release planning prefers immutable action cache.

---

# 98. Pipeline Cache Planner

```text
forgeyard-pipeline-cache
```

Produces:

```text
cache read plan
cache write plan
cache trust requirements
```

---

# 99. Cache Key Inputs

Include:

```text
source snapshot
toolchain
commands
declared environment subset
dependency lock
platform
sandbox semantics
```

as defined by hermetic architecture.

---

# 100. Cache Safety

Pipeline config cannot arbitrarily omit correctness inputs from action cache identity.

---

# 101. Services

Some CI jobs need databases/services.

Model carefully:

```rust
pub struct ServiceRequirement {
    pub image_or_tool: ServiceSource,
    pub ports: Vec<ServicePort>,
    pub health: ServiceHealthCheck,
}
```

---

# 102. Service Isolation

Service runs inside job sandbox/network namespace where possible.

---

# 103. Service Secrets

Late secret injection.

---

# 104. Service Lifetime

```text
job-scoped
```

by default.

---

# 105. Container Dependency

Do not require Docker architecture-wide.

Service executor can use:

```text
container
process
VM
```

based on platform/executor capabilities.

---

# 106. Approval/Gate Nodes

Pipeline may include manual/policy gates.

---

# 107. Approval Step

Should integrate with Change Proposal/release approval systems rather than invent duplicate approval semantics.

---

# 108. Pipeline Gate

Generic:

```rust
pub enum GateRequirement {
    ManualApproval(ApprovalPolicyRef),
    Policy(PolicyRef),
    EnvironmentApproval(EnvironmentId),
}
```

---

# 109. Policy Integration

```text
forgeyard-pipeline-policy
```

Policy can:

```text
reject
inject required checks
tighten resource/network rules
require approvals
require reproducibility
```

---

# 110. Policy Cannot Weaken User Safety?

Repository policy may tighten.

User pipeline must not override organization protections.

---

# 111. Policy Application Order

Recommended:

```text
parse
normalize
expand
construct semantic graph
derive capabilities
apply policy
final validation
canonical IR
```

---

# 112. Policy Digest

Pipeline plan records:

```text
policy bundle digest
```

---

# 113. Plan Identity

```rust
pub struct PipelinePlanId(Digest);
```

Derived from:

```text
PipelineIr
source snapshot
policy digest
relevant project configuration
```

---

# 114. Pipeline IR

```text
forgeyard-pipeline-ir
```

This is the canonical execution-oriented representation.

---

# 115. `PipelineIr`

```rust
pub struct PipelineIr {
    pub schema: PipelineIrVersion,
    pub pipeline_id: PipelineDefinitionId,
    pub source: PipelineSourceIr,
    pub jobs: BTreeMap<JobNodeId, JobIr>,
    pub edges: Vec<JobDependency>,
    pub policy_digest: PolicyDigest,
}
```

---

# 116. `JobIr`

```rust
pub struct JobIr {
    pub id: JobNodeId,
    pub name: JobName,
    pub steps: Vec<StepIr>,
    pub capabilities: CapabilityRequirementSet,
    pub resources: ResourceRequest,
    pub environment: ResolvedEnvironmentSpec,
    pub timeout: TimeoutSpec,
    pub retry: JobRetryPolicy,
    pub condition: ConditionExpr,
    pub outputs: Vec<OutputDeclaration>,
    pub cache: CachePlan,
}
```

---

# 117. `StepIr`

```rust
pub enum StepIr {
    Command(CommandActionIr),
    Ecosystem(EcosystemActionIr),
    Artifact(ArtifactActionIr),
    Gate(GateIr),
}
```

---

# 118. IR Characteristics

Canonical IR must be:

```text
validated
explicit
versioned
deterministic
provider-neutral
serializable for storage/diagnostics
```

---

# 119. No Raw Templates in IR

Templates are fully expanded before canonical IR.

---

# 120. No Matrix Definitions in IR

Matrix dimensions are fully expanded.

IR can retain origin metadata for UI/explain.

---

# 121. Origin Metadata

```rust
pub struct IrOrigin {
    pub source_span: Option<SourceSpan>,
    pub template: Option<TemplateName>,
    pub matrix: Option<MatrixCoordinate>,
}
```

Not part of semantic execution identity unless explicitly intended.

---

# 122. IR Version

```rust
pub struct PipelineIrVersion(u16);
```

Independent from human config schema.

---

# 123. IR Persistence

Store canonical IR in metadata or CAS depending size.

Recommended:

```text
small summary metadata DB
full canonical IR serialized to CAS if large
```

---

# 124. Executable Plan

```text
forgeyard-pipeline-plan
```

Transforms `PipelineIr` into runtime jobs/actions.

---

# 125. `ExecutablePlan`

```rust
pub struct ExecutablePlan {
    pub id: PipelinePlanId,
    pub jobs: Vec<PlannedJob>,
    pub dag: PlannedDag,
}
```

---

# 126. Planned Job

Includes:

```text
job IR ref
action/derivation refs
input CAS closure
capabilities
resources
policy
```

---

# 127. Plan-Time Resolution

Allowed:

```text
ecosystem manifests
dependency locks
toolchain selector -> immutable identity
source change paths
```

depending hermetic resolve stage.

---

# 128. Runtime Resolution

Late only when necessary:

```text
secrets
runner-specific temp path
lease IDs
```

---

# 129. Determinism Boundary

Given same:

```text
source snapshot
pipeline config
policy
toolchain resolution inputs
```

planning should produce same canonical plan identity.

---

# 130. Host Independence

Planner must not silently inspect developer host to choose:

```text
CPU features
installed compiler
HOME config
shell
```

unless explicitly in local-development mode and marked impure.

---

# 131. Impure Development Mode

Can allow:

```text
host toolchain
host environment
```

but resulting plan marked:

```text
Impure
```

and not equivalent to strict release plan.

---

# 132. Imports

```text
forgeyard-pipeline-import
```

External CI import maps to generic parsed/normalized model.

---

# 133. Import Sources

Potential:

```text
GitHub Actions
GitLab CI
Jenkins
CircleCI
Buildkite
Azure Pipelines
generic scripts
```

Adapters can be added later.

---

# 134. Import Philosophy

Do not promise exact semantic equivalence when provider features have no Forgeyard equivalent.

Importer returns warnings.

---

# 135. Import Result

```rust
pub struct ImportResult {
    pub pipeline: PipelineConfig,
    pub diagnostics: Vec<ImportDiagnostic>,
}
```

---

# 136. Unsupported Imported Feature

Example:

```text
provider-specific hosted action with hidden environment
```

Importer may emit:

```text
manual adaptation required
```

---

# 137. Import Provenance

Generated pipeline can record:

```text
source provider
source file
importer version
```

for audit.

---

# 138. Pipeline Explain

```text
forgeyard-pipeline-explain
```

Commands:

```text
forgeyard pipeline explain
forgeyard pipeline explain-job
forgeyard pipeline explain-cache
forgeyard pipeline explain-capabilities
forgeyard pipeline explain-skip
```

---

# 139. Explain Job

Shows:

```text
why job exists
template origin
matrix dimensions
dependencies
condition
capabilities
resources
policy additions
```

---

# 140. Explain Skip

Shows:

```text
condition false
change impact
manual gate
upstream failure
```

---

# 141. Explain Capability

Example:

```text
requires macOS because Swift target=iOS
requires Xcode 18 because toolchain lock
requires device because integration test target=physical-iOS
```

---

# 142. Explain Cache Key

Shows safe components:

```text
source snapshot
toolchain ID
dependency lock
command digest
environment whitelist
platform
```

Do not reveal secret values.

---

# 143. Diagnostics

```text
forgeyard-pipeline-diagnostic
```

Standard diagnostic model:

```rust
pub struct PipelineDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub primary: Option<SourceLabel>,
    pub secondary: Vec<SourceLabel>,
    pub help: Option<String>,
}
```

---

# 144. Error Recovery

Parser may recover enough to show multiple syntax diagnostics.

But never execute partially valid pipeline.

---

# 145. Lints

Separate warnings from hard validation.

Possible lints:

```text
unused template
job never runs
redundant dependency
overly broad secret scope
cache path likely ineffective
huge matrix
```

---

# 146. Pipeline Doctor

```text
forgeyard pipeline doctor
```

Checks:

```text
schema
templates
matrix size
capability availability
secret refs exist
policy requirements
toolchain availability
```

---

# 147. Capability Availability

Doctor can compare requirements against current registered runner fleet.

This is advisory.

Pipeline may still be valid if runner not currently online.

---

# 148. Unknown Future Runner

Validation should not reject a valid capability merely because no current runner has it unless policy requires deployability now.

---

# 149. Pipeline Hash

Human file hash != pipeline semantic identity.

Normalized/canonical config produces:

```text
PipelineDefinitionId
```

---

# 150. PipelineDefinitionId

```rust
pub struct PipelineDefinitionId(Digest);
```

Computed over normalized semantics excluding irrelevant comments/order.

---

# 151. Comments

Comments do not change pipeline identity.

---

# 152. Field Ordering

RON field ordering does not change semantic identity.

---

# 153. Equivalent Defaults

Explicit default and omitted default normalize same.

---

# 154. Policy Changes

Policy digest change may change executable plan even if pipeline config unchanged.

---

# 155. Toolchain Resolution Changes

Floating selectors like:

```text
stable
latest
```

must resolve to immutable identity before strict plan.

Strict release should disallow uncontrolled floating resolution.

---

# 156. Pipeline Lock

Optional:

```text
.forgeyard/lock/pipeline.ron
```

may record resolved template/toolchain/import dependencies if needed.

---

# 157. Remote Templates

If Forgeyard supports remote templates later:

```text
resolve to immutable source snapshot/digest
```

Never pull moving template at execution time.

---

# 158. Local Templates

Included in source snapshot.

---

# 159. Template Security

Imported/remote templates are code-like configuration.

Subject to trust/policy.

---

# 160. Third-Party Actions

Forgeyard should not reproduce GitHub Actions' opaque marketplace action model by default.

Prefer:

```text
explicit source/toolchain/action derivation
```

---

# 161. Reusable Actions

Future Forgeyard plugin/action packages should have:

```text
version
digest
permissions
input/output schema
```

---

# 162. Permissions Model

Pipeline steps may declare/request:

```text
network
secrets
devices
signing
deployment
```

Policy approves/denies.

---

# 163. Least Privilege

Job receives only permissions needed.

---

# 164. Secret Scope

Secret refs can be restricted by:

```text
project
environment
job
protected target
change trust level
```

---

# 165. Fork/External Change

Pipeline planning strips privileged secret/deploy/signing steps unless trusted policy permits.

---

# 166. Untrusted Code

All proposal code treated as untrusted for execution.

---

# 167. Protected Jobs

Examples:

```text
release signing
production deployment
```

may run only after trusted integration state, not arbitrary proposal source.

---

# 168. Manual Inputs

Manual pipeline trigger can define typed inputs:

```rust
pub struct ManualInput {
    pub name: InputName,
    pub ty: InputType,
    pub required: bool,
}
```

---

# 169. Input Types

```text
string
bool
integer
choice
duration
environment
```

---

# 170. Input Validation

Before plan creation.

---

# 171. Schedule Trigger

Schedule definition belongs to automation/scheduler trigger subsystem.

Pipeline receives normalized trigger event/time.

---

# 172. Event Context

```rust
pub struct PipelineEventContext {
    pub kind: EventKind,
    pub actor: ActorRef,
    pub source: PipelineSourceIr,
    pub metadata: EventMetadata,
}
```

---

# 173. Provider Event Data

Keep provider-specific raw payload outside core pipeline.

Normalizer extracts needed generic fields.

---

# 174. Job Dependencies on Artifacts

Prefer explicit:

```text
job B needs artifact X from job A
```

which implies dependency if needed.

---

# 175. Artifact Dependency Validation

Reject references to artifact never produced.

---

# 176. Optional Artifact

Can model optional consumption.

---

# 177. Cross-Pipeline Artifacts

Later capability:

```text
artifact by release/version/reference
```

but explicit and policy controlled.

---

# 178. Workspace Persistence

Jobs should not implicitly share mutable workspace across runners.

Cross-job data goes through:

```text
artifacts
cache
declared state
```

---

# 179. Same-Runner Affinity

Can be optimization preference, not correctness requirement unless explicitly declared service/session semantics.

---

# 180. Stateful Pipeline

Avoid hidden statefulness.

Any state must be explicit:

```text
artifact
deployment environment
external service
```

---

# 181. Parallelism

DAG nodes with satisfied deps can run concurrently.

---

# 182. Max Parallel

Pipeline/project policy may set concurrency cap.

---

# 183. Concurrency Group

```rust
pub struct ConcurrencyGroup {
    pub key: ConcurrencyKey,
    pub policy: ConcurrencyPolicy,
}
```

---

# 184. Concurrency Policies

```rust
pub enum ConcurrencyPolicy {
    Queue,
    CancelPrevious,
    Reject,
}
```

---

# 185. Use Cases

```text
one production deploy at a time
one release per branch
cancel superseded proposal CI
```

---

# 186. Concurrency Key Determinism

Derived from explicit inputs.

No arbitrary shell generation.

---

# 187. Cancellation Semantics

Pipeline plan declares cancellation policy.

Runtime run/job state machine handles execution.

---

# 188. Fail-Fast

```rust
pub enum FailFastPolicy {
    Disabled,
    Stage,
    Pipeline,
}
```

---

# 189. Matrix Fail-Fast

Can cancel sibling matrix jobs if one fails.

Explicit.

---

# 190. Continue-on-Error

Step/job can be non-blocking:

```rust
pub enum FailureImpact {
    Required,
    Allowed,
    Informational,
}
```

---

# 191. Aggregate Status

Pipeline success logic derived from required nodes.

---

# 192. Skipped Jobs

Skipped is distinct from success.

---

# 193. Planned Status

Before runtime:

```text
Planned
Pruned
WaitingForGate
```

Runtime job states defined in next architecture.

---

# 194. Job Output Variables

Avoid arbitrary runtime string exports across jobs.

Use typed output values where possible.

---

# 195. Job Output Model

```rust
pub struct JobOutputValue {
    pub name: OutputValueName,
    pub value: BoundedValue,
}
```

Sensitive outputs flagged/secret-backed.

---

# 196. Output Value Size

Strictly bounded.

Large data uses artifact/CAS.

---

# 197. Expression References

Condition/template expressions can reference typed output values.

---

# 198. Expression Language

Keep small, deterministic, side-effect free.

No network/filesystem/process access.

---

# 199. Expression Functions

Examples:

```text
starts_with
contains
path_matches
```

No arbitrary user-defined code initially.

---

# 200. Expression Determinism

Same inputs -> same output.

---

# 201. Regex

If supported, bounded and safe engine.

Avoid catastrophic backtracking.

---

# 202. Glob Patterns

Use canonical path glob semantics.

---

# 203. Path Filtering

Change triggers can specify:

```text
include
exclude
```

normalized against canonical repo paths.

---

# 204. Trigger Path Race

Trigger event exact revision/snapshot determines changed paths.

Do not re-evaluate moving branch later.

---

# 205. Pipeline Discovery

Default file locations:

```text
.forgeyard/pipeline.ron
.forgeyard/pipelines/*.ron
```

---

# 206. Multiple Pipelines

Project can define:

```text
ci
release
nightly
security
```

---

# 207. Pipeline Registry

Metadata stores discovered/registered definitions by source snapshot/config digest.

---

# 208. Dynamic Pipeline Generation

Do not execute arbitrary code to generate pipeline before trust boundary.

If supported later, treat generator as explicit build phase producing signed/canonical config.

---

# 209. Pipeline Includes

Local includes:

```text
within source snapshot
```

Remote includes:

```text
immutable digest/source snapshot only
```

---

# 210. Include Cycle

Detect and report.

---

# 211. Include Path Escape

Reject:

```text
../../outside
```

relative to allowed project source root.

---

# 212. Config Size Limits

Bound:

```text
pipeline file size
number of jobs
steps/job
template expansion
matrix expansion
condition AST depth
```

---

# 213. Denial-of-Service Protection

Parser/planner must be safe against malicious proposal configs.

---

# 214. Untrusted Pipeline Config

Change Proposal pipeline changes are untrusted.

Policy may restrict:

```text
new secret refs
new privileged jobs
new deployment targets
```

---

# 215. Pipeline Diff

Change Proposal UI can show semantic pipeline diff:

```text
new job
removed job
new secret access
network widened
resource increase
new deployment
new signing action
```

---

# 216. Pipeline Risk Analysis

High-risk pipeline changes can require extra review.

---

# 217. Pipeline Provenance

Run records:

```text
pipeline definition ID
pipeline IR version
plan ID
policy digest
source snapshot
```

---

# 218. Re-run

Re-run should specify whether:

```text
same exact plan
```

or:

```text
re-plan current policy/toolchains
```

These are different operations.

---

# 219. Exact Re-run

Uses persisted:

```text
source snapshot
PipelineIr/Plan
resolved toolchains
```

where retained.

---

# 220. Re-plan

May produce different PlanId.

Clearly labeled.

---

# 221. Pipeline Storage

Metadata:

```text
definition ID
name
source snapshot
plan summary
```

Large canonical IR:

```text
CAS
```

if needed.

---

# 222. Pipeline Plan Retention

Keep for:

```text
run history
release provenance
audit
```

according to policy.

---

# 223. Import Adapter Workspace

Potential:

```text
crates/pipeline/importers/
├── forgeyard-import-github-actions/
├── forgeyard-import-gitlab-ci/
├── forgeyard-import-jenkins/
└── ...
```

Can live under SCM or pipeline import group; choose one consistent location.

---

# 224. Importer Isolation

Provider parser dependencies stay importer-local.

---

# 225. GitHub Actions Import

Map:

```text
jobs
needs
matrix
env
artifacts
cache
```

where semantics are representable.

Opaque actions produce diagnostics/manual adaptation.

---

# 226. GitLab CI Import

Likewise preserve only supported semantics.

---

# 227. Import Report

Generated:

```text
translated exactly
translated approximately
unsupported
security-sensitive
```

---

# 228. CLI

```text
forgeyard pipeline validate
forgeyard pipeline normalize
forgeyard pipeline plan
forgeyard pipeline graph
forgeyard pipeline explain
forgeyard pipeline explain-job
forgeyard pipeline explain-cache
forgeyard pipeline explain-capabilities
forgeyard pipeline migrate
forgeyard pipeline import
forgeyard pipeline doctor
```

---

# 229. `pipeline validate`

Returns all diagnostics.

---

# 230. `pipeline normalize`

Prints canonical human-readable RON representation.

Useful for debugging.

---

# 231. `pipeline plan`

Displays execution DAG and requirements without executing.

---

# 232. `pipeline graph`

Formats:

```text
text
DOT
JSON
```

Public/internal output choice.

---

# 233. Dioxus UI

Pipeline editor/viewer:

```text
Overview
Graph
Jobs
Steps
Matrix
Capabilities
Cache
Artifacts
Policy
Diagnostics
Source
```

---

# 234. Graph UI

Shows:

```text
DAG
parallel branches
gates
artifact edges
status at runtime
```

---

# 235. Pipeline Editor

Can provide schema-aware forms/text editor.

But server validator remains authority.

---

# 236. Diagnostics UI

Click diagnostic -> exact source span.

---

# 237. Explain UI

Job sidebar:

```text
why scheduled
where from
requirements
policy additions
cache identity summary
```

---

# 238. API

Potential:

```text
POST /v1/pipelines/validate
POST /v1/pipelines/plan
GET  /v1/projects/{id}/pipelines
GET  /v1/pipelines/{id}
GET  /v1/pipelines/{id}/graph
```

---

# 239. Validation API Security

Do not resolve secrets.

Do not execute code.

May restrict expensive import/template fetch.

---

# 240. Planning API

Requires exact source snapshot and policy context.

---

# 241. Caching Parsed Pipelines

Cache by:

```text
pipeline source blob digest
schema version
```

---

# 242. Caching Normalized IR

Cache by:

```text
definition digest
```

---

# 243. Planning Cache

Can cache if all planning inputs immutable.

---

# 244. Plan Cache Invalidation

Invalidate on:

```text
policy digest
toolchain resolution
ecosystem adapter semantic version
platform schema
```

where relevant.

---

# 245. Adapter Version Identity

Ecosystem planner implementation version may affect plan semantics.

Record Forgeyard version/adapter semantic version.

---

# 246. Compiler-Like Architecture

Pipeline compiler stages:

```text
Lex/Parse
Semantic Validate
Normalize
Expand
Type-check Expressions
Build Graph
Lower to IR
Optimize
Policy Rewrite
Plan
```

---

# 247. Optimization Passes

Allowed only if semantics preserved.

Examples:

```text
prune false plan-time conditions
deduplicate identical fetch actions
reuse common toolchain inputs
```

---

# 248. Optimization Correctness

Never let optimizer change required work.

---

# 249. Optimization Explainability

Record:

```text
job pruned due to condition
action reused
```

---

# 250. Predictive Optimization

Predictive cache/prefetch occurs after correctness plan.

---

# 251. Plan Fingerprint

```text
PipelinePlanId
```

allows reproducible/explainable run planning.

---

# 252. Security Boundaries

Threats:

```text
malicious config
matrix explosion
expression DoS
path escape
secret escalation
privileged job injection
unsupported import hiding behavior
```

---

# 253. Parser Resource Limits

Bound parse depth/input bytes.

---

# 254. Template Resource Limits

Bound recursion/expansion.

---

# 255. Expression Resource Limits

Bound AST depth/evaluation steps.

---

# 256. Secret Escalation

Policy checks newly requested secret refs.

---

# 257. Privileged Capability Escalation

Change from:

```text
linux build
```

to:

```text
signing worker
production deployment
```

must require stronger policy.

---

# 258. Import Security

Imported external CI can contain arbitrary scripts/actions.

Importer marks risky/opaque semantics.

---

# 259. No Implicit Host Secrets

Pipeline must not inherit:

```text
developer shell environment
credential helpers
HOME secrets
```

in strict mode.

---

# 260. Logging

Parser/planner logs:

```text
definition ID
plan ID
job count
matrix expansion count
diagnostic count
```

No secret values.

---

# 261. Metrics

Examples:

```text
pipeline_parse_latency
pipeline_validate_latency
pipeline_plan_latency
pipeline_jobs
pipeline_matrix_expanded_jobs
pipeline_validation_errors
pipeline_import_warnings
```

---

# 262. Tracing

Spans:

```text
pipeline.parse
pipeline.validate
pipeline.normalize
pipeline.expand
pipeline.dag
pipeline.capabilities
pipeline.policy
pipeline.plan
```

---

# 263. Testkit

```text
forgeyard-pipeline-testkit/src/
├── lib.rs
├── fixture.rs
├── parser.rs
├── normalize.rs
├── dag.rs
├── matrix.rs
├── condition.rs
├── policy.rs
└── assertions.rs
```

---

# 264. Unit Tests

Test:

```text
name validation
defaulting
condition type system
matrix combination
template substitution
DAG cycle detection
```

---

# 265. Golden Tests

Good for:

```text
diagnostics
normalized RON
import reports
```

Use carefully to avoid brittle giant snapshots.

---

# 266. Property Tests

Properties:

```text
normalization idempotent
field-order independent
DAG topo order valid
equivalent defaults same definition ID
```

---

# 267. Fuzzing

Targets:

```text
RON parser
condition parser
template expansion
matrix config
external importer inputs
```

---

# 268. Cycle Fuzz

Generate random dependency graphs and verify cycle detector.

---

# 269. Matrix Fuzz

Ensure expansion bound respected.

---

# 270. Security Tests

1. path include escape rejected;
2. secret refs do not reveal values;
3. malicious expression bounded;
4. template recursion bounded;
5. matrix explosion rejected;
6. privileged capability requires policy.

---

# 271. Integration Tests

Use fixture projects:

```text
Rust
C++
Go
Python
JVM
Flutter
Swift
mixed
```

to verify ecosystem handoff.

---

# 272. Standalone Test

Plan and execute pipeline entirely local.

---

# 273. Distributed Test

Same PipelineIr sent to scheduler/runners without semantic change.

---

# 274. Cross-Mode Invariant

Same strict inputs:

```text
same PipelineIr / PlanId
```

whether standalone or distributed.

Execution placement can differ.

---

# 275. Plan Compatibility

Agent does not need full human pipeline parser.

Agent receives planned job/action messages.

---

# 276. Daemon Responsibility

Daemon/service compiles pipeline into plan.

---

# 277. CLI Local Responsibility

Standalone CLI may invoke same pipeline compiler library locally.

---

# 278. Agent Simplicity

Agent knows:

```text
job spec
inputs
capabilities
execution
```

not templates/matrix/import logic.

---

# 279. Implementation Phase 1 — Models / Parser

Implement:

```text
schema
PipelineConfig
JobConfig
StepConfig
RON parser
diagnostics
```

---

# 280. Phase 2 — Validation / Normalize

Implement:

```text
names
references
defaults
canonical ordering
resource validation
```

---

# 281. Phase 3 — DAG

Implement:

```text
dependencies
cycle detection
topological analysis
```

---

# 282. Phase 4 — Templates / Matrix

Add bounded typed expansion.

---

# 283. Phase 5 — Conditions

Implement deterministic typed expression AST.

---

# 284. Phase 6 — Pipeline IR

Define versioned canonical `PipelineIr`.

---

# 285. Phase 7 — Ecosystem / Capability Planning

Integrate `EcosystemAdapter`, platform/native capabilities.

---

# 286. Phase 8 — Cache / Hermetic Lowering

Produce derivation/action/cache plans.

---

# 287. Phase 9 — Policy

Apply organization/project/change/release policy.

---

# 288. Phase 10 — Explain / UI/API

Add explainability and public planning surfaces.

---

# 289. Phase 11 — Importers

Start with highest-value external CI providers.

---

# 290. Phase 12 — Hardening

Add:

```text
fuzzing
DoS limits
large monorepo performance
compatibility tests
```

---

# 291. Acceptance Tests

1. Same semantic RON with different field order yields same definition ID.
2. Omitted default and explicit default normalize identically.
3. Invalid job reference produces source-span diagnostic.
4. DAG cycle produces full cycle path.
5. Template expansion is deterministic.
6. Recursive template is rejected/bounded.
7. Matrix expansion is deterministic.
8. Matrix explosion is rejected.
9. Condition type mismatch fails planning.
10. Plan-time false condition prunes safely.
11. Change-impact uncertainty expands work.
12. Secret value never appears in IR.
13. Secret ref remains late-bound.
14. Raw absolute host path is rejected in strict job.
15. Ecosystem adapter contributes required toolchain capabilities.
16. Native dependency contributes native capability requirement.
17. Swift/iOS plan requires Apple/Xcode capability.
18. Rust+native mixed project creates correct adapter handoff.
19. Cache identity includes required semantic inputs.
20. Policy can tighten network permission.
21. User config cannot weaken protected policy.
22. External CI importer reports unsupported semantics.
23. Strict plan does not inspect host toolchain silently.
24. Same strict pipeline produces same PlanId in standalone/distributed modes.
25. Agent receives planned job without needing parser/template logic.
26. Pipeline config size limits are enforced.
27. Include path traversal is rejected.
28. Untrusted proposal pipeline cannot silently gain privileged secrets.
29. Exact rerun can use persisted plan identity.
30. Re-plan after policy change yields new plan identity.

---

# 292. Production Readiness Gates

Do not call pipeline compiler production-ready until:

```text
schema versioning stable
parser diagnostics usable
normalization deterministic
DAG correctness proven
matrix/template bounds enforced
condition language typed
PipelineIr versioned
ecosystem capability planning works
policy integration works
secret values excluded
strict host independence works
plan explainability exists
fuzz/security tests pass
```

---

# 293. Architectural Invariants

1. Raw pipeline config is never executed.
2. Pipeline parsing and execution are separate.
3. Canonical IR is provider-neutral.
4. Human schema version is independent from IR version.
5. Normalization is deterministic.
6. Templates are expanded before IR.
7. Matrices are expanded before IR.
8. DAG cycles are rejected.
9. Conditions are side-effect free.
10. Condition evaluation is bounded.
11. Secret values never enter PipelineIr.
12. Secret refs are late-bound.
13. Platform/toolchain capabilities are explicit.
14. Ecosystem semantics live in ecosystem adapters.
15. Native requirements flow through native APIs.
16. Pipeline planner does not silently depend on host state in strict mode.
17. Policy can tighten but protected policy cannot be weakened by pipeline.
18. Action/cache correctness inputs cannot be omitted by user config.
19. Same immutable inputs yield same semantic plan.
20. Agent does not parse human pipeline configuration.
21. Cross-job mutable filesystem state is not implicit.
22. Large cross-job data uses artifacts/CAS.
23. Imported provider semantics are never overclaimed.
24. Pipeline config is treated as untrusted input.
25. Planning is explainable.
26. Planner failures provide typed diagnostics.
27. Optimizations preserve semantics.
28. Standalone/distributed modes share the same PipelineIr.
29. Pipeline history records definition/plan/policy/source identities.
30. Forgeyard's own `.forgeyard/pipeline.ron` must dogfood the same compiler.

---

# 294. Final Target Architecture

```text
                 .forgeyard/pipeline.ron
                        │
                        ▼
                     Parser
                        │
                        ▼
                    Validator
                        │
                        ▼
                   Normalizer
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       Templates      Matrix       Conditions
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                       DAG
                        │
                        ▼
             Ecosystem / Native Plan
                        │
                        ▼
               Capability Derivation
                        │
                        ▼
                Cache / Hermetic Plan
                        │
                        ▼
                      Policy
                        │
                        ▼
                   PipelineIr
                        │
                        ▼
                  ExecutablePlan
                        │
                        ▼
                Run / Job subsystem
```

---

# 295. Final Architectural Position

Pipeline compilation:

```text
Human config
+
SourceSnapshotId
+
Project configuration
+
Policy bundle
+
Ecosystem/toolchain resolution
        ↓
Validated normalized semantic graph
        ↓
PipelineIr
        ↓
ExecutablePlan
```

Runtime then consumes:

```text
ExecutablePlan
```

not the human file.

The key guarantee is:

> **Forgeyard treats pipeline configuration as source code: parsed, type-checked, normalized, lowered, policy-checked, and compiled into a deterministic execution plan before any job is allowed to run.**

---

# 296. New-Repository Sequence

The implementation sequence is now:

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
