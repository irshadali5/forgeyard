# 47 — Forgeyard CI/CD Migration, Import, Compatibility & Legacy-System Interoperability System Architecture

**Document type:** Core CI/CD Migration, Import, Translation, Compatibility, Validation & Legacy Interoperability System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** import from external CI/CD systems, legacy pipeline translation, Forgeyard-version migration, compatibility analysis, semantic gap detection, dual-run validation, staged cutover, credentials/secret migration planning, artifact/history import boundaries, provider migration, pipeline equivalence evidence, rollback, and operator migration tooling  
**Architecture style:** Import-to-canonical-IR, semantics-before-syntax, explicit compatibility levels, evidence-backed equivalence, conservative unsupported-feature handling, staged migration, source-system read-only by default, and no silent behavioral downgrade  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Pipeline IR, VCS-neutral SourceSnapshot, Change Proposal, SCM providers, Workflow Templates, Triggers, Secrets, Artifacts/CAS, Tests, Findings, Release, Deployment, Configuration, Audit, Security, and Developer Experience. This subsystem allows organizations to adopt Forgeyard without rewriting their entire CI/CD estate blindly.

---

# 1. Purpose

A new CI/CD platform rarely starts in an empty organization.

Existing teams may already use:

```text
GitHub Actions
GitLab CI/CD
Jenkins
Buildkite
CircleCI
Azure Pipelines
TeamCity
Drone
Woodpecker
custom shell scripts
Makefiles
Nix
Bazel
old internal CI
older Forgeyard schemas
```

A serious migration system must answer:

```text
what can be imported automatically?
what cannot be translated safely?
what semantics differ?
what secrets are referenced?
what runners/platforms are required?
what external actions/plugins are used?
what schedule/webhook behavior exists?
how can the old and new systems run side by side?
how do we prove the Forgeyard result is equivalent enough to cut over?
how do we roll back if migration fails?
```

The central rule is:

> **Forgeyard imports external CI/CD definitions into normalized intermediate migration models and then into canonical Forgeyard Pipeline IR. External syntax is never executed directly as Forgeyard truth.**

A second rule is:

> **Migration tooling must distinguish syntactic translation from semantic equivalence. If Forgeyard cannot prove or model an external behavior, it reports the gap rather than silently approximating it.**

A third rule is:

> **Cutover should be evidence-driven. Organizations should be able to dual-run old and new pipelines, compare outputs/checks/timing/evidence, and migrate incrementally instead of performing an all-or-nothing rewrite.**

---

# 2. Architectural Position

```text
                   External CI/CD
                         │
                         ▼
                    Source Adapter
                         │
                         ▼
                  Migration Model
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
           Syntax     Semantics   Dependencies
              │          │          │
              └──────────┼──────────┘
                         ▼
                Compatibility Analysis
                         │
                 ┌───────┼────────┐
                 ▼       ▼        ▼
              Exact    Adapted   Unsupported
                 │       │        │
                 └───────┼────────┘
                         ▼
                  Forgeyard Config
                         │
                         ▼
                    Pipeline IR
                         │
                         ▼
                  Dual-Run Validation
                         │
                         ▼
                       Cutover
```

---

# 3. Goals

The subsystem MUST:

1. define migration source identity;
2. support multiple external CI systems;
3. parse source-native configs safely;
4. normalize external pipeline semantics;
5. map jobs/stages/steps;
6. map triggers;
7. map environments;
8. map variables;
9. map secret references;
10. map caches/artifacts;
11. map matrices;
12. map conditions;
13. map schedules;
14. map services/containers;
15. map approvals;
16. map deployment stages;
17. map runner labels/capabilities;
18. identify unsupported semantics;
19. classify compatibility;
20. generate Forgeyard config;
21. preserve migration provenance;
22. support dry-run planning;
23. support dual-run comparison;
24. support phased cutover;
25. support rollback;
26. support history/artifact import where useful;
27. support old-Forgeyard migration;
28. support audit;
29. support CLI/UI;
30. remain conservative and explainable.

---

# 4. Non-Goals

This subsystem does not:

```text
execute GitHub Actions YAML directly
execute Jenkins Groovy directly
guarantee byte-for-byte equivalence for arbitrary legacy CI
migrate secret values automatically without explicit secure flow
replace SCM migration
replace repository migration
```

---

# 5. Workspace Structure

```text
crates/migration/
├── forgeyard-migration/
├── forgeyard-migration-model/
├── forgeyard-migration-source/
├── forgeyard-migration-normalize/
├── forgeyard-migration-compat/
├── forgeyard-migration-translate/
├── forgeyard-migration-validate/
├── forgeyard-migration-dualrun/
├── forgeyard-migration-cutover/
├── forgeyard-migration-report/
├── forgeyard-migration-health/
└── forgeyard-migration-testkit/
```

Source adapters:

```text
crates/migration-adapters/
├── forgeyard-migrate-github-actions/
├── forgeyard-migrate-gitlab-ci/
├── forgeyard-migrate-jenkins/
├── forgeyard-migrate-buildkite/
├── forgeyard-migrate-circleci/
├── forgeyard-migrate-azure-pipelines/
├── forgeyard-migrate-drone/
├── forgeyard-migrate-generic-shell/
└── forgeyard-migrate-forgeyard-legacy/
```

Use modules first; split only where parser/provider dependencies justify.

---

# 6. MigrationProjectId

```rust
pub struct MigrationProjectId(Ulid);
```

One migration effort.

---

# 7. MigrationSourceId

```rust
pub struct MigrationSourceId(Digest);
```

Identifies exact imported source material.

---

# 8. Migration Source

```rust
pub struct MigrationSource {
    pub id: MigrationSourceId,
    pub platform: MigrationPlatform,
    pub config_objects: Vec<CasObjectRef>,
    pub repository: Option<RepositoryId>,
    pub discovered_at: Timestamp,
}
```

---

# 9. Migration Platform

```rust
pub enum MigrationPlatform {
    GitHubActions,
    GitLabCi,
    Jenkins,
    Buildkite,
    CircleCi,
    AzurePipelines,
    Drone,
    Woodpecker,
    GenericShell,
    LegacyForgeyard,
    Custom(MigrationPlatformId),
}
```

---

# 10. Exact Input Capture

Imported configuration files are snapshotted.

---

# 11. No "Current Remote Config" as Final Identity

Critical.

---

# 12. Import Mode

```rust
pub enum MigrationImportMode {
    OfflineFiles,
    RepositoryScan,
    ProviderReadOnly,
}
```

---

# 13. Provider Read-Only

Preferred first.

---

# 14. No Mutation During Discovery

Critical.

---

# 15. Migration Discovery

Detect:

```text
workflow files
runner labels
service definitions
environment names
secret refs
actions/plugins
artifacts
cache
deploy stages
schedules
manual inputs
```

---

# 16. Discovery Result

```rust
pub struct MigrationDiscovery {
    pub source: MigrationSourceId,
    pub workflows: Vec<ExternalWorkflow>,
    pub dependencies: Vec<ExternalCiDependency>,
    pub secrets: Vec<ExternalSecretReference>,
}
```

---

# 17. External Workflow Model

Provider-neutral intermediate representation.

---

# 18. Why Intermediate Model

Do not translate:

```text
GitHub YAML -> Forgeyard config
```

directly.

Use:

```text
GitHub YAML
  ↓
ExternalWorkflow
  ↓
Compatibility Analysis
  ↓
Forgeyard pipeline/template config
```

---

# 19. ExternalWorkflow

```rust
pub struct ExternalWorkflow {
    pub id: ExternalWorkflowId,
    pub name: BoundedString,
    pub triggers: Vec<ExternalTrigger>,
    pub jobs: Vec<ExternalJob>,
    pub semantics: ExternalWorkflowSemantics,
}
```

---

# 20. ExternalJob

Contains:

```text
dependencies
conditions
matrix
environment
runner requirements
steps
artifacts
cache
services
timeout
retry semantics
```

---

# 21. ExternalStep

```rust
pub enum ExternalStep {
    Command(ExternalCommandStep),
    Action(ExternalActionRef),
    Plugin(ExternalPluginRef),
    Script(ExternalScriptRef),
    Unknown(ExternalStepPayload),
}
```

---

# 22. Unknown Is First-Class

Critical.

---

# 23. No Unknown-to-Shell Automatic Fallback

Never.

---

# 24. Semantics Inventory

The adapter must identify platform-specific behavior.

---

# 25. Compatibility Classification

```rust
pub enum MigrationCompatibility {
    Exact,
    Equivalent,
    Adaptable,
    Manual,
    Unsupported,
    Unknown,
}
```

---

# 26. Exact

Forgeyard has directly equivalent semantics.

---

# 27. Equivalent

Implementation differs, externally relevant behavior expected equivalent.

---

# 28. Adaptable

Requires explicit Forgeyard-native restructuring.

---

# 29. Manual

Human must make decision.

---

# 30. Unsupported

Forgeyard intentionally cannot reproduce behavior.

---

# 31. Unknown

Insufficient evidence.

---

# 32. No "Best Effort = Success"

Critical.

---

# 33. Compatibility Finding

```rust
pub struct MigrationFinding {
    pub path: MigrationPath,
    pub compatibility: MigrationCompatibility,
    pub source_semantics: BoundedString,
    pub forgeyard_mapping: Option<ForgeyardMapping>,
    pub rationale: BoundedString,
}
```

---

# 34. MigrationPath

Example:

```text
workflow.build.jobs.test.steps[3]
```

---

# 35. Compatibility Report

```rust
pub struct MigrationCompatibilityReport {
    pub source: MigrationSourceId,
    pub findings: Vec<MigrationFinding>,
    pub summary: CompatibilitySummary,
}
```

---

# 36. Summary

Counts:

```text
Exact
Equivalent
Adaptable
Manual
Unsupported
Unknown
```

---

# 37. Migration Confidence

```rust
pub enum MigrationConfidence {
    High,
    Medium,
    Low,
    Unknown,
}
```

---

# 38. Confidence Is Separate from Compatibility

---

# 39. Example

Equivalent + Low confidence.

---

# 40. Translation Rule Registry

Versioned.

---

# 41. MigrationRuleSetId

```rust
pub struct MigrationRuleSetId(Digest);
```

---

# 42. Why

Migration result depends on adapter/rules version.

---

# 43. Generated Config Identity

Include rule set.

---

# 44. Pipeline Mapping

External:

```text
workflow/stage/job/step
```

maps to Forgeyard:

```text
Pipeline
Job
CommandSpec/Executor
Artifact declaration
```

---

# 45. Dependency DAG

Preserve explicitly.

---

# 46. Implicit Ordering

If source platform has stage ordering, normalize to DAG edges.

---

# 47. Concurrency

Map if semantics compatible.

---

# 48. Job Retry

Normalize to Forgeyard Attempt model.

---

# 49. Step Retry

If external system supports distinct behavior, preserve/adapt explicitly.

---

# 50. Timeout

Exact duration semantics where possible.

---

# 51. Matrix

Map to Pipeline IR matrix expansion.

---

# 52. Dynamic Matrix

If source allows runtime-generated matrices, classify carefully.

---

# 53. Static Matrix

Usually Exact/Equivalent.

---

# 54. Runtime Dynamic Matrix

May require two-stage Forgeyard planning model or manual adaptation.

---

# 55. Conditions

Map into Forgeyard condition language.

---

# 56. Unsupported Expression Function

Manual/unsupported finding.

---

# 57. No Eval Emulation

Critical.

---

# 58. Environment Variables

Classify:

```text
literal
derived
secret reference
provider context
```

---

# 59. Provider Context

Examples:

```text
github.sha
gitlab commit SHA
build number
```

map to typed Forgeyard runtime context.

---

# 60. Secret Detection

Never export values during read-only discovery.

---

# 61. Secret Reference Mapping

```rust
pub struct SecretMigrationMapping {
    pub external_name: BoundedString,
    pub target_ref: SecretRef,
    pub state: SecretMigrationState,
}
```

---

# 62. Secret Migration State

```rust
pub enum SecretMigrationState {
    ReferencedOnly,
    TargetCreated,
    ValueTransferred,
    ManualRequired,
    Unavailable,
}
```

---

# 63. Value Transfer

Explicit high-risk operation.

---

# 64. Prefer Re-Creation/Rotation

Rather than copying old secret blindly.

---

# 65. No Secret Value in Migration Report

Critical.

---

# 66. External Actions

GitHub Actions etc.

---

# 67. Action Classification

```rust
pub enum ExternalActionMigration {
    ReplaceWithForgeyardPrimitive,
    ReplaceWithToolInvocation,
    ReplaceWithPlugin,
    VendorAndSandbox,
    Manual,
    Unsupported,
}
```

---

# 68. Marketplace Action

Not automatically trusted.

---

# 69. Pinning

Resolve exact source/digest where possible.

---

# 70. Mutable `@main`

Security finding.

---

# 71. Action with Docker Entrypoint

Can sometimes map to container step.

---

# 72. Action with Node Runtime

May map to tool invocation/plugin.

---

# 73. Composite Action

Can expand declaratively if semantics known.

---

# 74. JavaScript/Node Action

Do not blindly inline.

---

# 75. Jenkins Shared Libraries

Potentially arbitrary Groovy code.

---

# 76. Baseline

Manual/unsupported unless constrained adapter can analyze safely.

---

# 77. Jenkins Groovy

Never execute in daemon for migration.

---

# 78. Static Parsing

Sandboxed parser if needed.

---

# 79. Dynamic Jenkinsfile

Often impossible to translate exactly statically.

---

# 80. Report Unknown/Manual

Critical honesty.

---

# 81. GitHub Actions Trigger Mapping

Examples:

```text
push -> ScmPush
pull_request -> ChangeProposal
workflow_dispatch -> Manual
schedule -> Schedule
workflow_call -> Template/Reusable Pipeline
```

---

# 82. GitLab Mapping

Examples:

```text
rules
only/except
stages
needs
artifacts
cache
environments
manual jobs
```

---

# 83. Azure Pipeline Mapping

Stages/jobs/steps/pools/variables/environments.

---

# 84. Buildkite

Agents/queues/plugins/steps.

---

# 85. CircleCI

workflows/jobs/executors/orbs.

---

# 86. Orb

Equivalent to reusable template/plugin concept.

---

# 87. Generic Shell CI

Can map commands but likely low hermetic confidence.

---

# 88. Makefile Import

Not pipeline semantics by itself.

---

# 89. Can become command target reference.

---

# 90. Runner Mapping

External labels:

```text
ubuntu-latest
self-hosted
windows
macos
gpu
```

normalize into capability requirements.

---

# 91. `*-latest`

Mutable.

---

# 92. Migration Warning

Resolve to explicit Forgeyard platform/toolchain profile.

---

# 93. No "latest" in protected execution identity

Critical.

---

# 94. Service Containers

Map to DevService/job service dependencies where supported.

---

# 95. Container Image Tags

Resolve exact digest.

---

# 96. Privileged Container

Security finding/manual policy.

---

# 97. Docker Socket

High-risk unsupported by default Forgeyard sandbox.

---

# 98. Migration Report

Explain safer Forgeyard alternative.

---

# 99. Cache Mapping

External cache keys may be string-based.

---

# 100. Forgeyard

Should not preserve unsafe string cache semantics blindly.

---

# 101. Migration

Map intent to Forgeyard derivation/cache policy.

---

# 102. Cache Compatibility

Often Adaptable rather than Exact.

---

# 103. Artifact Mapping

External uploaded artifacts -> Forgeyard declared outputs/CAS.

---

# 104. Artifact Paths

Validate.

---

# 105. Retention

Map to lifecycle policy if explicitly configured.

---

# 106. Test Report Mapping

JUnit etc. to Part 32 ingestion.

---

# 107. Coverage Mapping

Existing formats.

---

# 108. Security Scanner Mapping

Part 37.

---

# 109. Deployment Mapping

External environments/deploy jobs -> Deployment subsystem.

---

# 110. Protected Environment Approval

Map to Part 11/15/16 human workflow.

---

# 111. Do Not Translate Approval as Shell Prompt

Critical.

---

# 112. Release Mapping

Tags/manual release jobs -> Release Candidate/Release pipeline.

---

# 113. Legacy Pattern

```text
build and upload in same script
```

---

# 114. Forgeyard Adaptation

Split:

```text
build
package
verify
sign
publish
```

---

# 115. Compatibility

Adaptable, not Exact.

---

# 116. Trigger Migration

Import:

```text
webhooks
schedules
manual inputs
branch filters
```

into Part 44.

---

# 117. Schedule Timezone

External systems may assume UTC.

---

# 118. Preserve exact documented source semantics.

---

# 119. Concurrency Group

Map to Part 44 if equivalent.

---

# 120. Cancel-In-Progress

Map to supersession/cancel policy.

---

# 121. Workflow Reuse

External reusable workflows -> Part 42 templates where appropriate.

---

# 122. Composite Reuse

Normalize.

---

# 123. Secrets Scope

External repository/org/environment secret scopes may differ.

---

# 124. Forgeyard Secret Mapping

Explicit scope conversion.

---

# 125. Scope Widening

Forbidden silently.

---

# 126. Example

Repo secret cannot become installation-wide secret by convenience.

---

# 127. Environment Protection

Map explicit.

---

# 128. External Deployment Credentials

Prefer new workload-federated credentials rather than copying.

---

# 129. Migration Security Review

Required for:

```text
privileged runners
Docker socket
static cloud keys
SSH keys
production signing
external actions
dynamic scripts
```

---

# 130. Migration Risk

```rust
pub enum MigrationRisk {
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 131. Risk Factors

```text
secret scope
privilege
network
mutable dependencies
production deploy
signing
unparsed dynamic code
```

---

# 132. Risk Report

Separate from compatibility.

---

# 133. Generated Forgeyard Config

Human-readable RON.

---

# 134. Generated Files

Potential:

```text
.forgeyard/pipeline.ron
.forgeyard/triggers.ron
.forgeyard/toolchains.ron
migration-report.md
```

---

# 135. Do Not Overwrite Existing Files

Without explicit user action.

---

# 136. Generation Mode

```rust
pub enum MigrationGenerationMode {
    Preview,
    WriteNewFiles,
    PatchExisting,
}
```

---

# 137. Preview First

Recommended.

---

# 138. Migration Plan

```rust
pub struct MigrationPlan {
    pub id: MigrationPlanId,
    pub source: MigrationSourceId,
    pub rules: MigrationRuleSetId,
    pub compatibility: MigrationCompatibilityReportId,
    pub generated: Vec<GeneratedConfigRef>,
}
```

---

# 139. Plan Is Immutable

---

# 140. Plan Freshness

If source CI config changes, plan stale.

---

# 141. MigrationPlanFreshness

```rust
pub enum MigrationPlanFreshness {
    Current,
    SourceChanged,
    RulesChanged,
    TargetChanged,
    Unknown,
}
```

---

# 142. Stale Plan

Regenerate before cutover.

---

# 143. Dual-Run

Run old CI and Forgeyard for same source revision.

---

# 144. DualRunId

```rust
pub struct DualRunId(Ulid);
```

---

# 145. Dual Run Pair

```rust
pub struct DualRunPair {
    pub source_snapshot: SourceSnapshotId,
    pub external_run: ExternalRunRef,
    pub forgeyard_run: RunId,
}
```

---

# 146. Exact Same Source

Critical.

---

# 147. External Run Source

Verify provider revision maps to same snapshot if possible.

---

# 148. Dual-Run Comparison

Compare:

```text
success/failure
artifacts
tests
coverage
findings
duration
deployment side effects
```

---

# 149. Side Effects

Dangerous.

---

# 150. Dual-Run Mode

Default side effects disabled/shadowed.

---

# 151. Shadow Release

Forgeyard can build/package/verify but not publish.

---

# 152. Shadow Deployment

No production mutation.

---

# 153. External System Remains Authority During Validation

---

# 154. Comparison Level

```rust
pub enum EquivalenceLevel {
    Structural,
    Outcome,
    ArtifactDigest,
    SemanticArtifact,
    FullEvidence,
}
```

---

# 155. Structural

Same logical jobs/stages roughly.

---

# 156. Outcome

Pass/fail equivalence.

---

# 157. ArtifactDigest

Exact bytes.

---

# 158. SemanticArtifact

Normalized equivalent package/tree.

---

# 159. FullEvidence

Tests/findings/provenance policy-aligned.

---

# 160. Not Every Migration Needs Byte-for-Byte

But level must be explicit.

---

# 161. Migration Verification

```rust
pub struct MigrationVerification {
    pub dual_run: DualRunId,
    pub level: EquivalenceLevel,
    pub result: VerificationResult,
    pub differences: Vec<MigrationDifference>,
}
```

---

# 162. Verification Result

```rust
pub enum VerificationResult {
    Equivalent,
    AcceptableDifference,
    Divergent,
    Inconclusive,
}
```

---

# 163. Acceptable Difference

Requires explicit reviewed rationale.

---

# 164. Inconclusive

Never green by default.

---

# 165. Artifact Digest Comparison

Use CAS/digest aliases.

---

# 166. Normalized Tree Comparison

Useful when timestamps/package metadata differ.

---

# 167. Test Comparison

Use Part 32 normalized identities.

---

# 168. Findings Comparison

Part 37.

---

# 169. Benchmark Comparison

Part 33 if relevant.

---

# 170. Cutover

Staged.

---

# 171. Migration Stage

```rust
pub enum MigrationStage {
    Discovery,
    Translation,
    Validation,
    DualRun,
    PartialCutover,
    FullCutover,
    LegacyReadOnly,
    Completed,
}
```

---

# 172. Partial Cutover

Examples:

```text
Forgeyard PR checks
legacy release
```

then:

```text
Forgeyard build/release
legacy disabled
```

---

# 173. Per-Pipeline Cutover

Supported.

---

# 174. Per-Branch Cutover

Possible.

---

# 175. Per-Repository Cutover

Possible.

---

# 176. Cutover Decision

Policy/admin.

---

# 177. MigrationCutoverId

```rust
pub struct MigrationCutoverId(Ulid);
```

---

# 178. Cutover Preconditions

```text
current migration plan
required compatibility threshold
required dual-run evidence
secrets/providers ready
trigger switch ready
rollback path
```

---

# 179. Trigger Cutover

Avoid duplicate production runs.

---

# 180. Sequence

```text
pause legacy trigger
verify paused
activate Forgeyard trigger
observe
```

---

# 181. Ambiguous Provider State

Inspect before enabling duplicate side effects.

---

# 182. No Simultaneous Production Deploy by Both Systems

Critical.

---

# 183. Dual Production Build

Okay if side-effect free.

---

# 184. Single Publication Authority

At any cutover stage.

---

# 185. Rollback

If Forgeyard cutover fails:

```text
pause Forgeyard trigger
verify no active protected side effect
restore legacy trigger
reconcile
```

---

# 186. Rollback Record

Audited.

---

# 187. Legacy Read-Only

After cutover, preserve old system history while preventing new runs.

---

# 188. Decommission

Separate operational project.

---

# 189. History Import

Optional.

---

# 190. Import Scope

Could import:

```text
run summaries
artifact metadata
test summaries
log links
```

---

# 191. Historical External Run

```rust
pub struct ImportedExternalRun {
    pub source_platform: MigrationPlatform,
    pub external_id: BoundedString,
    pub source_snapshot: Option<SourceSnapshotId>,
    pub outcome: ExternalRunOutcome,
}
```

---

# 192. Imported Run Is Not Native Forgeyard Run

Critical.

---

# 193. Do Not Fabricate Native Provenance

---

# 194. Historical Artifact Import

Can import bytes into CAS.

---

# 195. Trust State

ExternalImported.

---

# 196. No automatic ReleaseTrusted status.

---

# 197. Historical Test Import

Can normalize for analytics, but mark ExternalImported.

---

# 198. Log Import

Optional/costly.

---

# 199. Provider Links

May retain external URL/ref while source system alive.

---

# 200. Legacy Forgeyard Migration

Different case.

---

# 201. Old Schema

Can migrate directly if semantics known.

---

# 202. ForgeyardMigrationVersion

```rust
pub struct ForgeyardMigrationVersion(u16);
```

---

# 203. Schema Migration

Pure deterministic transformations where possible.

---

# 204. Old Pipeline IR

Never reused blindly if schema semantics changed.

---

# 205. Recompile From Source Config

Preferred.

---

# 206. Database Migration

Part 25.

---

# 207. Config Migration

Part 39.

---

# 208. Template Migration

Part 42.

---

# 209. Migration Explain

```text
forgeyard migrate explain
```

---

# 210. CLI

```text
forgeyard migrate detect
forgeyard migrate inspect
forgeyard migrate plan
forgeyard migrate generate
forgeyard migrate validate
forgeyard migrate dual-run
forgeyard migrate compare
forgeyard migrate cutover
forgeyard migrate rollback
```

---

# 211. Source-Specific Convenience

```text
forgeyard migrate github-actions
```

can route to common engine.

---

# 212. UI

Pages:

```text
Migration Projects
Discovery
Compatibility
Generated Pipeline
Dual-Run
Differences
Cutover
```

---

# 213. Compatibility UI

Color/status:

```text
Exact
Equivalent
Adaptable
Manual
Unsupported
Unknown
```

---

# 214. Avoid Fake Percentage Score Alone

Critical.

---

# 215. Summary Score

Can exist only with full details.

---

# 216. Secret Migration UI

Shows names/refs/status only.

---

# 217. Never secret values.

---

# 218. Dual-Run UI

Side-by-side:

```text
legacy
Forgeyard
differences
```

---

# 219. Cutover UI

Explicit preconditions.

---

# 220. API

Potential:

```text
POST /v1/migrations
POST /v1/migrations/{id}/discover
POST /v1/migrations/{id}/plan
POST /v1/migrations/{id}/generate
POST /v1/migrations/{id}/dual-run
POST /v1/migrations/{id}/cutover
POST /v1/migrations/{id}/rollback
```

---

# 221. Permissions

```text
migration.read
migration.create
migration.generate
migration.dualrun
migration.cutover
migration.rollback
migration.secret.manage
migration.history.import
```

---

# 222. Cutover Permission

High risk.

---

# 223. Secret Transfer Permission

Separate.

---

# 224. Provider Credentials

Read-only discovery credentials distinct from mutation credentials.

---

# 225. Least Privilege

Critical.

---

# 226. Audit

Audit:

```text
migration plan approval
secret mapping
cutover
rollback
legacy trigger disable
history import
```

---

# 227. Routine parser findings

Not privileged audit.

---

# 228. Notifications

Examples:

```text
dual-run divergence
migration plan stale
legacy trigger still active
cutover failed
```

---

# 229. Search

Part 31 can index migration projects/findings.

---

# 230. Analytics

Examples:

```text
migration compatibility by source
manual gap categories
dual-run divergence
cutover success
```

---

# 231. No Vendor Scorecard Without Context

---

# 232. Observability Metrics

```text
migration_discovery_total
migration_findings_total
migration_translation_failures_total
migration_dualrun_divergence_total
migration_cutover_failures_total
```

---

# 233. Labels

Low cardinality:

```text
source_platform
compatibility
result
```

---

# 234. Tracing

```text
migration.discover
migration.parse
migration.normalize
migration.compat
migration.generate
migration.dualrun
migration.cutover
```

---

# 235. Health

Checks:

```text
adapter version
provider read access
parser failures
stale plans
cutover drift
```

---

# 236. Doctor

```text
forgeyard migrate doctor
```

---

# 237. Doctor Checks

```text
source config accessible
provider token scope
unsupported dynamic features
secret mappings incomplete
runner capability gaps
trigger cutover readiness
```

---

# 238. Migration Security

Source configs are untrusted.

---

# 239. Parse Safely

Bound sizes/depth.

---

# 240. Jenkins/Groovy

Never execute.

---

# 241. YAML Bombs

Bound aliases/nesting.

---

# 242. XML

Disable external entities.

---

# 243. External URLs

Do not fetch arbitrarily during parse.

---

# 244. Action/Plugin Resolution

Go through dependency/plugin governance.

---

# 245. Secrets

Never logged.

---

# 246. Provider Tokens

SecretRef.

---

# 247. Temporary Migration Credentials

Expire.

---

# 248. Source System Mutation

Disabled until explicit cutover stage.

---

# 249. Supply Chain

Imported actions/plugins become dependency/provenance subjects.

---

# 250. Artifact Trust

Imported artifacts classified separately.

---

# 251. Standalone Mode

Can migrate from local files/repositories.

---

# 252. Distributed Mode

Provider-connected migration projects.

---

# 253. Air-Gap

Offline config import supported.

---

# 254. No live provider required.

---

# 255. Migration Bundle

```rust
pub struct MigrationBundle {
    pub source: MigrationSourceId,
    pub configs: Vec<CasObjectRef>,
    pub report: MigrationCompatibilityReportId,
    pub generated: Vec<CasObjectRef>,
}
```

---

# 256. Bundle Export

For review/air-gap.

---

# 257. DR

Migration project metadata is operational, not execution authority after cutover.

---

# 258. If Lost

Can rediscover from source/Forgeyard config.

---

# 259. Dual-Run Evidence

Retain according to migration/compliance policy.

---

# 260. Migration Source Update

Re-scan.

---

# 261. Incremental Migration

Only changed workflows reanalyzed.

---

# 262. Rules Upgrade

Can reprocess old source with new adapter.

---

# 263. Reprocessing

Creates new report/plan, not overwrite.

---

# 264. Adapter Version

```rust
pub struct MigrationAdapterVersion(SemVer);
```

---

# 265. Compatibility Semantics Version

```rust
pub struct MigrationSemanticsVersion(u16);
```

---

# 266. Why

Classification logic changes over time.

---

# 267. Testkit

```text
forgeyard-migration-testkit/src/
├── lib.rs
├── source.rs
├── normalize.rs
├── compat.rs
├── generate.rs
├── dualrun.rs
├── cutover.rs
└── assertions.rs
```

---

# 268. Fixture Corpus

Realistic sample configs from each source platform.

---

# 269. Golden Tests

Source config -> expected intermediate model/report.

---

# 270. Round-Trip Is Not Required

Forgeyard model is richer/different.

---

# 271. GitHub Actions Tests

Matrix, reusable workflow, actions, concurrency, schedules.

---

# 272. GitLab Tests

needs/stages/rules/artifacts/manual environments.

---

# 273. Jenkins Tests

Static declarative subset + dynamic unsupported cases.

---

# 274. Buildkite Tests

queues/plugins.

---

# 275. Azure Tests

stages/pools/environments.

---

# 276. Secret Scope Test

No widening.

---

# 277. Mutable Action Test

Warning/manual hardening.

---

# 278. Docker Socket Test

High-risk/unsupported default.

---

# 279. Unknown Feature Test

Never silently dropped.

---

# 280. Dual-Run Exact Source Test

SourceSnapshot IDs match.

---

# 281. Artifact Comparison Test

Digest/normalized tree.

---

# 282. Inconclusive Test

Does not auto-approve cutover.

---

# 283. Trigger Cutover Test

No duplicate production side effect.

---

# 284. Rollback Test

Legacy trigger restored safely.

---

# 285. Imported History Test

Never becomes native provenance.

---

# 286. Provider Timeout Test

Cutover external effects reconciled.

---

# 287. Tenant Isolation Test

Migration project scoped.

---

# 288. Parser Fuzzing

YAML/JSON/XML/Groovy AST input boundaries.

---

# 289. Failure Injection

```text
provider API outage
source config changes mid-plan
secret mapping unavailable
dual-run legacy job missing
cutover webhook mutation ambiguous
```

---

# 290. Scale Test

Thousands of repositories/workflows.

---

# 291. Implementation Phase 1 — Migration Model/Generic Shell

Core abstractions.

---

# 292. Phase 2 — GitHub Actions

Highest priority.

---

# 293. Phase 3 — GitLab CI

Second major provider.

---

# 294. Phase 4 — Compatibility Report/Generation

Usability.

---

# 295. Phase 5 — Dual-Run Comparison

Evidence.

---

# 296. Phase 6 — Cutover/Rollback

Operational safety.

---

# 297. Phase 7 — Jenkins

Legacy enterprise.

---

# 298. Phase 8 — Buildkite/CircleCI/Azure

Broader adoption.

---

# 299. Phase 9 — History/Artifact Import

Optional.

---

# 300. Phase 10 — Legacy Forgeyard Migration

Self-evolution.

---

# 301. Phase 11 — Air-Gap/Scale

Enterprise.

---

# 302. Phase 12 — Fuzz/Security/Compatibility Hardening

Production readiness.

---

# 303. Acceptance Tests

1. External CI config never executes directly as Forgeyard pipeline truth.
2. Every imported source config has immutable MigrationSourceId.
3. Source adapters normalize into provider-neutral migration models.
4. Unknown source semantics remain Unknown/Manual/Unsupported rather than disappearing.
5. Compatibility and confidence are separate.
6. Migration rules/adapter versions are recorded.
7. Generated Forgeyard config compiles through normal Pipeline IR.
8. External runner labels map to explicit Forgeyard capabilities.
9. Mutable `latest` runner/image/action references are surfaced.
10. Secret values are never included in migration reports.
11. Secret-scope widening is never automatic.
12. External marketplace actions do not become trusted plugins automatically.
13. Dynamic Jenkins/Groovy logic is not executed by Forgeyard migration services.
14. Unsafe Docker-socket/privileged behaviors are highlighted explicitly.
15. External cache semantics are not blindly copied when unsafe.
16. External approvals map to real Forgeyard policy/human workflow.
17. Generated release/deploy flows do not collapse build/sign/publish into unsafe hidden scripts.
18. Migration plan becomes stale if source config changes.
19. Dual-run compares the same exact source snapshot.
20. Dual-run protected side effects are shadowed/disabled by default.
21. Equivalence level is explicit.
22. Inconclusive verification cannot silently approve cutover.
23. Cutover has exactly one production side-effect authority at a time.
24. Trigger cutover avoids duplicate releases/deployments.
25. Rollback can restore legacy authority safely.
26. Imported historical runs remain clearly ExternalImported.
27. Imported artifacts do not automatically gain trusted provenance.
28. Provider discovery uses read-only credentials where possible.
29. Source system mutation is disabled before explicit cutover.
30. Migration parsers are hardened against hostile config input.
31. Tenant migration projects are isolated.
32. Standalone supports offline/file-based migration.
33. Distributed mode supports provider-connected migration.
34. Legacy Forgeyard schema migration remains explicit/versioned.
35. Forgeyard dogfoods its migration engine when evolving its own pipeline/config schemas.

---

# 304. Production Readiness Gates

Do not call migration architecture production-ready until:

```text
provider-neutral migration model is stable
GitHub/GitLab adapters have broad fixture coverage
unknown/unsupported semantics are never silently dropped
secret-scope mapping is safe
generated config compiles through canonical IR
dual-run exact-source comparison works
cutover/rollback side-effect authority is enforced
parser fuzzing passes
provider timeout/reconciliation tests pass
large-repository migration tests pass
```

---

# 305. Architectural Invariants

1. importers translate into canonical Forgeyard models, never execute foreign CI directly;
2. syntax translation is not semantic-equivalence proof;
3. unknown semantics remain explicit;
4. migration input is immutable/versioned;
5. migration rules are versioned;
6. generated pipelines pass normal Pipeline IR validation;
7. no external action/plugin becomes trusted automatically;
8. secret values never enter migration reports;
9. secret scope is never widened silently;
10. dynamic source-language code is not executed during migration;
11. mutable external refs are surfaced/resolved explicitly;
12. privileged behavior is highlighted, not normalized away;
13. old CI approvals map to real policy/human workflows;
14. dual-run uses the same exact source identity;
15. protected side effects are shadowed during dual-run by default;
16. equivalence level is explicit;
17. inconclusive comparison cannot auto-cut over;
18. migration plans have freshness;
19. production cutover has one side-effect authority at a time;
20. rollback is explicit/audited;
21. imported history never fabricates native Forgeyard provenance;
22. imported artifacts have separate trust state;
23. provider discovery is read-only by default;
24. source-system mutation starts only at cutover;
25. parsing is hostile-input safe;
26. tenant migration data is isolated;
27. air-gap/file migration is supported;
28. legacy Forgeyard migration is versioned;
29. migration can be incremental/staged;
30. Forgeyard dogfoods its own migration mechanisms.

---

# 306. Final Target Architecture

```text
                 Existing CI/CD
                      │
                      ▼
               Read-Only Discovery
                      │
                      ▼
               Migration Source
                      │
                      ▼
              Normalized Workflow
                      │
                      ▼
           Compatibility Analysis
                      │
              ┌───────┼────────┐
              ▼       ▼        ▼
            Exact   Adapted   Manual
              │       │        │
              └───────┼────────┘
                      ▼
              Forgeyard Config
                      │
                      ▼
                 Pipeline IR
                      │
                      ▼
                Dual-Run Evidence
                      │
                      ▼
                    Cutover
```

---

# 307. Final Architectural Position

Translation:

```text
external syntax
+
external semantics
+
adapter/rules version
  ↓
provider-neutral migration model
  ↓
compatibility findings
  ↓
generated Forgeyard config
```

Validation:

```text
legacy run
+
Forgeyard run
+
same SourceSnapshotId
  ↓
outcome/artifact/test/finding comparison
  ↓
Equivalent / AcceptableDifference / Divergent / Inconclusive
```

Cutover:

```text
validated migration
  ↓
pause legacy protected triggers
  ↓
verify authority removed
  ↓
activate Forgeyard triggers
  ↓
observe
  ↓
keep rollback path until stable
```

The key guarantee is:

> **Forgeyard can help teams migrate from existing CI/CD systems without pretending that YAML-to-RON translation is enough. Every migration preserves source semantics where possible, exposes unsupported or risky behavior where not, validates the new pipeline against the same immutable source, and cuts over only after production side-effect authority is explicitly transferred.**

---

# 308. Extended Architecture Sequence

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
```
