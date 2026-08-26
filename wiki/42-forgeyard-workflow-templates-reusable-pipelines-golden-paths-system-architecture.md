# 42 — Forgeyard Workflow Templates, Reusable Pipelines, Organization Standards & Golden Paths System Architecture

**Document type:** Core Workflow Reuse, Pipeline Template, Organization Standardization & Golden Path System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** reusable pipeline modules, organization templates, typed template parameters and outputs, versioning, template registries, inheritance/composition, centrally managed golden paths, mandatory stages, policy bindings, template trust/signing, upgrade/migration workflows, template provenance, project adoption, local overrides, rollout, and backward compatibility  
**Architecture style:** Typed composition, explicit versioning, immutable template identities, Pipeline IR normalization, policy-controlled inheritance, organization governance, reproducible expansion, and no hidden executable logic outside canonical Forgeyard planning  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Pipeline IR, Configuration Governance, Policy/Authz, Monorepo Intelligence, Developer Experience, Static Analysis, Tests, Benchmarks, Release, Deployment, Supply Chain, and Search/Analytics. This subsystem turns repeated CI/CD patterns into governed reusable building blocks without sacrificing transparency.

---

# 1. Purpose

As Forgeyard grows across many projects, teams will repeat patterns such as:

```text
Rust build/test
security scan
SBOM generation
package signing
release publishing
deployment rollout
mobile build/test
monorepo affected checks
```

Copy-pasted pipeline files create:

```text
drift
security inconsistency
slow rollout of fixes
duplicated maintenance
hard-to-audit variation
```

Organizations need reusable workflows and opinionated defaults.

The central rule is:

> **Reusable workflows compile into the same canonical Pipeline IR as hand-written pipelines. Templates are a source-level composition mechanism, not a second execution system.**

A second rule is:

> **A template reference resolves to an immutable version before planning. Mutable names such as `stable` or `latest` are never retained as execution identity.**

A third rule is:

> **Golden paths may enforce organization requirements, but project authors must still be able to inspect exactly what jobs, permissions, secrets, tools, and policies the template expands into.**

---

# 2. Architectural Position

```text
                 Template Sources
      ┌──────────────┼──────────────┐
      ▼              ▼              ▼
  Built-In       Organization     Project
  Templates       Registry        Templates
      │              │              │
      └──────────────┼──────────────┘
                     ▼
              Template Resolver
                     │
                     ▼
             Typed Composition
                     │
                     ▼
              Expanded Pipeline
                     │
                     ▼
                Pipeline IR
                     │
                     ▼
              Policy / Planning
                     │
                     ▼
                Execution
```

---

# 3. Goals

The subsystem MUST:

1. define template identity;
2. define template versions;
3. support reusable jobs;
4. support reusable stages;
5. support reusable pipeline fragments;
6. support typed parameters;
7. support typed outputs;
8. support template defaults;
9. support organization templates;
10. support project-local templates;
11. support template registries;
12. support immutable resolution;
13. support signed trusted templates;
14. support template provenance;
15. support inheritance/composition;
16. prevent unsafe arbitrary override;
17. support mandatory organization controls;
18. support golden paths;
19. support template upgrades;
20. support compatibility checks;
21. support migration tooling;
22. support local developer preview;
23. support template diff/explain;
24. support policy integration;
25. support search/discovery;
26. support tenant isolation;
27. support air-gap bundles;
28. support standalone mode;
29. support distributed mode;
30. remain deterministic and transparent.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Pipeline IR
create a general-purpose programming language
allow arbitrary runtime code in template expansion
replace policy
hide privileged execution
```

---

# 5. Workspace Structure

```text
crates/template/
├── forgeyard-template/
├── forgeyard-template-model/
├── forgeyard-template-parser/
├── forgeyard-template-resolve/
├── forgeyard-template-expand/
├── forgeyard-template-validate/
├── forgeyard-template-registry/
├── forgeyard-template-trust/
├── forgeyard-template-migrate/
├── forgeyard-template-policy/
├── forgeyard-template-health/
└── forgeyard-template-testkit/
```

Organization standards:

```text
crates/golden-path/
├── forgeyard-golden-path/
├── forgeyard-golden-path-model/
├── forgeyard-golden-path-evaluate/
└── forgeyard-golden-path-testkit/
```

Use modules first; split crates only where real dependency/runtime boundaries justify.

---

# 6. TemplateId

```rust
pub struct TemplateId(Digest);
```

Content-derived immutable identity.

---

# 7. TemplateLogicalId

```rust
pub struct TemplateLogicalId(BoundedString);
```

Stable human identity across versions.

Examples:

```text
org/rust-service
org/android-app
builtin/rust-library
```

---

# 8. TemplateVersion

```rust
pub struct TemplateVersion(SemVer);
```

---

# 9. Exact Template Ref

```rust
pub struct TemplateRef {
    pub logical: TemplateLogicalId,
    pub version: TemplateVersion,
    pub digest: TemplateId,
}
```

---

# 10. Mutable Template Selector

Allowed only before resolution.

```rust
pub enum TemplateSelector {
    Exact(TemplateRef),
    VersionReq(TemplateLogicalId, VersionRequirement),
    Channel(TemplateLogicalId, TemplateChannel),
}
```

---

# 11. Resolution

Before planning:

```text
selector
  ↓
exact TemplateRef
```

---

# 12. No Mutable Template Identity in Run

Critical.

---

# 13. Template Kind

```rust
pub enum TemplateKind {
    Job,
    Stage,
    PipelineFragment,
    FullPipeline,
    GoldenPath,
}
```

---

# 14. Template Definition

```rust
pub struct WorkflowTemplate {
    pub logical_id: TemplateLogicalId,
    pub version: TemplateVersion,
    pub kind: TemplateKind,
    pub parameters: Vec<TemplateParameterDef>,
    pub outputs: Vec<TemplateOutputDef>,
    pub body: TemplateBody,
}
```

---

# 15. Template Parameters

Strongly typed.

---

# 16. Parameter Types

```rust
pub enum TemplateParameterType {
    Bool,
    Integer,
    String,
    Enum(Vec<BoundedString>),
    Duration,
    Platform,
    TargetSelector,
    SecretRef,
    ArtifactRef,
    List(Box<TemplateParameterType>),
}
```

---

# 17. No Arbitrary Untyped Map

Critical.

---

# 18. Parameter Validation

At expansion time.

---

# 19. Defaults

Explicit.

---

# 20. Required Parameter

No default.

---

# 21. Secret Parameter

Must be `SecretRef`, never plaintext.

---

# 22. Template Outputs

Typed references.

Examples:

```text
artifact
package set
test summary
deployment target
```

---

# 23. Output Identity

Must map to normal Pipeline IR output constructs.

---

# 24. Template Body

Source-level declarative structure.

---

# 25. No Embedded Arbitrary Rust/Lua/JavaScript

Critical.

---

# 26. Expression Language

Reuse existing safe condition/expression language.

---

# 27. Template Expansion

Pure/deterministic.

---

# 28. Expansion Inputs

```text
exact TemplateRef
parameter values
project config
platform/profile
```

---

# 29. Expansion Result

```rust
pub struct TemplateExpansion {
    pub template: TemplateRef,
    pub invocation: TemplateInvocationId,
    pub expanded: PipelineFragment,
    pub digest: Digest,
}
```

---

# 30. TemplateInvocationId

Content-derived.

---

# 31. Pipeline Compiler

Consumes expanded fragment.

---

# 32. Canonical End State

All workflows become:

```text
PipelineIr
```

---

# 33. Provenance

Pipeline plan records template invocations.

---

# 34. Template Provenance

```rust
pub struct TemplateProvenance {
    pub template: TemplateRef,
    pub parameters_digest: Digest,
    pub expansion_digest: Digest,
}
```

---

# 35. Why

A user can trace:

```text
which template?
which version?
which parameters?
```

---

# 36. Template Registry

Stores metadata and immutable template objects.

---

# 37. Registry Scope

```rust
pub enum TemplateRegistryScope {
    BuiltIn,
    Installation,
    Tenant,
    Organization,
    Project,
}
```

---

# 38. Tenant Isolation

Private templates stay scoped.

---

# 39. Built-In Templates

Versioned with Forgeyard release.

---

# 40. Organization Templates

Managed centrally.

---

# 41. Project Templates

Repository-local.

---

# 42. Template Search

Part 31 integration.

---

# 43. Registry APIs

Potential:

```text
publish
resolve
list
fetch
deprecate
yank
```

---

# 44. Publish

Creates immutable version.

---

# 45. Update

New version.

---

# 46. No In-Place Mutation

Critical.

---

# 47. Template Channel

Examples:

```text
stable
beta
legacy
```

---

# 48. Channel Is Mutable Alias

Resolved before plan.

---

# 49. Deprecation

Marks version discouraged.

---

# 50. Yank

Prevents new resolution where policy says.

---

# 51. Historical Run

Still references exact old template.

---

# 52. Template Trust

```rust
pub enum TemplateTrustClass {
    BuiltInTrusted,
    OrganizationTrusted,
    ProjectTrusted,
    ExternalUntrusted,
}
```

---

# 53. External Template

Must not become privileged automatically.

---

# 54. Signing

Organization template packages can be signed.

---

# 55. Signature

Proves publisher/trust provenance.

---

# 56. Signature Does Not Bypass Policy

Critical.

---

# 57. Template Package

```rust
pub struct TemplatePackage {
    pub manifest: WorkflowTemplate,
    pub documentation: Option<CasObjectRef>,
    pub signature: Option<SignatureRef>,
}
```

---

# 58. Template Dependencies

Templates may reuse other templates.

---

# 59. Dependency Model

```rust
pub struct TemplateDependency {
    pub selector: TemplateSelector,
}
```

---

# 60. Resolution

All nested dependencies resolved exactly.

---

# 61. Template Dependency Graph

Must be acyclic.

---

# 62. Cycle

Validation error.

---

# 63. Maximum Depth

Bounded.

---

# 64. Expansion Size

Bound:

```text
jobs
steps
nested invocations
parameters
```

---

# 65. Template Bomb

Malicious expansion cannot create unbounded IR.

---

# 66. Composition

Prefer explicit composition over inheritance.

---

# 67. Why

Deep inheritance chains become hard to reason about.

---

# 68. Baseline Composition

```text
include template
pass parameters
bind outputs
```

---

# 69. Golden Path

Organization-managed full workflow standard.

---

# 70. GoldenPathId

```rust
pub struct GoldenPathId(Digest);
```

---

# 71. Golden Path Purpose

Examples:

```text
standard Rust service
mobile application
production service
library crate
```

---

# 72. Golden Path Contents

Potential:

```text
build
test
coverage
static analysis
SBOM
package
release
deployment
```

---

# 73. Golden Path Is Not Mandatory by Definition

Policy determines whether project must adopt.

---

# 74. Organization Standard

```rust
pub struct OrganizationWorkflowStandard {
    pub organization: OrganizationId,
    pub required_templates: Vec<TemplateRequirement>,
    pub mandatory_jobs: Vec<RequiredJobSpec>,
}
```

---

# 75. Mandatory Controls

Examples:

```text
security scan
license check
SBOM
release signing
```

---

# 76. Project Cannot Remove Mandatory Control

Critical.

---

# 77. Policy Expansion

Same principle as Part 34.

---

# 78. Final Pipeline

```text
ProjectPipeline
∪ TemplateExpansion
∪ OrganizationRequired
∪ PolicyRequired
```

---

# 79. Never Subtract PolicyRequired

Critical.

---

# 80. Template Override

Need explicit model.

---

# 81. Override Types

```rust
pub enum TemplateOverrideCapability {
    ParameterOnly,
    Extend,
    ReplaceOptionalSection,
    Disabled,
}
```

---

# 82. Default

ParameterOnly.

---

# 83. Extend

Add jobs/steps around template.

---

# 84. Replace Optional Section

Only if template author marks slot overrideable.

---

# 85. Protected Section

Cannot be replaced.

---

# 86. Template Slot

```rust
pub struct TemplateSlot {
    pub id: TemplateSlotId,
    pub mode: SlotMode,
}
```

---

# 87. Slot Mode

```rust
pub enum SlotMode {
    Append,
    Replace,
    Protected,
}
```

---

# 88. Protected

Organization security stage.

---

# 89. Local Project Freedom

Use parameters/extension slots.

---

# 90. No Arbitrary Mutation of Expanded IR

Critical.

---

# 91. Template Policy

Can enforce:

```text
allowed registries
minimum trusted template class
version ranges
mandatory template
forbidden template
```

---

# 92. Template Version Policy

Example:

```text
org/rust-service >= 4,<5
```

---

# 93. Security Urgent Minimum

Can require minimum safe version.

---

# 94. Template Upgrade

Projects need migration path.

---

# 95. Upgrade Candidate

```rust
pub struct TemplateUpgradeCandidate {
    pub current: TemplateRef,
    pub proposed: TemplateRef,
    pub compatibility: TemplateCompatibility,
    pub migration: Option<TemplateMigrationPlan>,
}
```

---

# 96. Compatibility

```rust
pub enum TemplateCompatibility {
    Compatible,
    RequiresParameterChange,
    RequiresProjectChange,
    Breaking,
}
```

---

# 97. SemVer

Useful but not sufficient.

---

# 98. Template Compatibility Checker

Compares:

```text
parameters
outputs
slots
required capabilities
job semantics
```

---

# 99. Template Migration

Can provide declarative suggestions.

---

# 100. No Hidden Source Mutation

Migration produces patch/proposal.

---

# 101. `forgeyard template upgrade`

Shows diff.

---

# 102. Template Diff

```text
added jobs
removed jobs
changed permissions
changed secrets
changed network
changed toolchains
changed outputs
```

---

# 103. Security-Sensitive Diff

Highlight strongly.

---

# 104. Template Review

Change Proposal can show template version change separately.

---

# 105. Auto Upgrade

Not baseline for breaking changes.

---

# 106. Minor Safe Upgrade

Policy may allow automated proposal.

---

# 107. Never Auto-Merge by Default

---

# 108. Template Lock

Repository can pin resolved refs.

---

# 109. `forgeyard.template.lock`

Optional machine-generated lock.

---

# 110. Purpose

Prevent channel/version-range drift.

---

# 111. Pipeline Plan

Still stores exact refs regardless lock.

---

# 112. Template Resolution Cache

Safe derived cache.

---

# 113. Key

```text
selector
registry snapshot
policy
```

---

# 114. Registry Snapshot

Immutable version/index snapshot.

---

# 115. Offline Development

Pinned templates available locally/CAS.

---

# 116. Air-Gap Bundle

Can include template packages.

---

# 117. TemplateOfflineBundle

```rust
pub struct TemplateOfflineBundle {
    pub templates: Vec<TemplatePackageRef>,
    pub manifest: CasObjectRef,
    pub signature: Option<SignatureRef>,
}
```

---

# 118. Import

Verify digests/signatures/policy.

---

# 119. Built-In Golden Paths

Forgeyard can ship examples.

---

# 120. Example Rust Golden Path

Conceptually:

```text
format
clippy
unit test
integration test
coverage
SAST
SBOM
package
reproducibility
```

---

# 121. Project Type Detection

Developer Experience can recommend template.

---

# 122. Recommendation Is Advisory

No silent application.

---

# 123. `forgeyard init`

Can offer:

```text
Use organization Rust service golden path?
```

---

# 124. Template Parameter Example

```ron
use: (
    template: "org/rust-service@4.2.0",
    params: (
        package: "forgeyard-daemon",
        coverage_min: 0.85,
    ),
)
```

---

# 125. Canonical Parsing

RON.

---

# 126. Template Inputs

Cannot read arbitrary filesystem outside declared project/config context.

---

# 127. No Dynamic Network During Expansion

Critical.

---

# 128. Registry Resolution

Happens before pure expansion.

---

# 129. No Time/Randomness

Expansion deterministic.

---

# 130. No Host Environment

Unless explicit declared parameter.

---

# 131. Template Secrets

SecretRef only.

---

# 132. Template Cannot Reveal Secret

---

# 133. Template Privileged Capability Request

Still subject to scheduler/policy.

---

# 134. Example

Template requests:

```text
network: restricted
signing: required
```

It cannot grant.

---

# 135. Organization Trust

Trusted template may be allowed to request certain approved capabilities.

---

# 136. But Execution Authority Still Central

Critical.

---

# 137. Template Source

Can be:

```text
repository
internal registry
built-in
```

---

# 138. External Git URL

Not baseline direct runtime import.

---

# 139. Better

Import/publish into trusted template registry.

---

# 140. Why

Immutable versioning, scanning, availability.

---

# 141. Template Supply Chain

Treat template package like code.

---

# 142. Static Analysis

Parse/validate.

---

# 143. Security Checks

Detect requests for:

```text
secrets
network
privileged executor
signing
production deploy
```

---

# 144. Template Risk Profile

```rust
pub struct TemplateRiskProfile {
    pub secrets: BTreeSet<SecretRefPattern>,
    pub network: NetworkRequirement,
    pub privileged_capabilities: BTreeSet<CapabilityKind>,
}
```

---

# 145. Publish Review

Organization registry can require review for high-risk changes.

---

# 146. Template Provenance

Publisher identity + source + signature.

---

# 147. Template SBOM

Usually unnecessary as pure declarative data, but plugin/tool references inside template remain dependency evidence.

---

# 148. Tool Version Pins

Template can reference toolchain descriptors.

---

# 149. Dependency Closure

Resolved normally.

---

# 150. Template and Monorepo

Template can target logical component selectors.

---

# 151. Affected Mode

Parameter:

```text
affected_only: true
```

may configure supported template behavior.

---

# 152. Policy Floor

Cannot reduce required full validation.

---

# 153. Test Template

Can expose outputs:

```text
test_summary
coverage
```

---

# 154. Analysis Template

Outputs findings/evidence.

---

# 155. Release Template

Protected.

---

# 156. Release Golden Path

Can enforce:

```text
verification
SBOM
sign
release
```

---

# 157. No Hidden Production Deploy

Template expansion must visibly show deployment job.

---

# 158. Pipeline Explain

```text
forgeyard plan explain
```

includes template origin.

---

# 159. Template Explain

```text
forgeyard template explain <ref>
```

shows:

```text
parameters
outputs
required capabilities
secrets
network
expanded jobs
```

---

# 160. Template Diff CLI

```text
forgeyard template diff <old> <new>
```

---

# 161. Template List/Search

```text
forgeyard template list
forgeyard template search rust
```

---

# 162. Template Publish

```text
forgeyard template publish
```

---

# 163. Publish Permission

```text
template.publish
```

---

# 164. Template Admin

```text
template.manage
```

---

# 165. Golden Path Admin

```text
golden_path.manage
```

---

# 166. Project Use

```text
template.use
```

---

# 167. Template Trust Admin

Separate high privilege.

---

# 168. API

Potential:

```text
GET  /v1/templates
GET  /v1/templates/{id}
POST /v1/templates
GET  /v1/templates/{id}/versions
POST /v1/templates/resolve
POST /v1/templates/expand
GET  /v1/golden-paths
```

---

# 169. Expansion API

Developer/admin preview only.

Server planner performs canonical expansion.

---

# 170. Dioxus UI

Pages:

```text
Templates
Golden Paths
Template Versions
Upgrade Candidates
Organization Standards
```

---

# 171. Template Detail

Shows:

```text
version
publisher
trust
parameters
outputs
jobs
capabilities
usage
```

---

# 172. Upgrade View

Diff.

---

# 173. Project Pipeline UI

Badges:

```text
from template
organization required
project-defined
policy-added
```

---

# 174. Transparency

Critical.

---

# 175. Template Usage Analytics

Examples:

```text
projects by version
deprecated usage
upgrade lag
golden-path adoption
```

---

# 176. No Developer Performance Tracking

---

# 177. Search

Part 31.

---

# 178. Notification

Examples:

```text
template deprecated
security minimum version required
golden path update available
```

---

# 179. Audit

Audit:

```text
template publish/yank
trust change
organization mandatory requirement
golden path activation
```

---

# 180. Project Template Invocation

Not necessarily high-level audit beyond Run/Pipeline plan.

---

# 181. Template Registry Storage

Metadata DB + CAS package bytes.

---

# 182. Immutable Package

CAS.

---

# 183. Registry Index

DB.

---

# 184. Registry Outage

Pinned/local template can still compile.

---

# 185. Distributed HA

Registry metadata Postgres.

---

# 186. Standalone

Built-in/project-local templates.

---

# 187. Template Availability

No external online dependency required for pinned templates.

---

# 188. Template Cache

Derived.

---

# 189. DR

Registry metadata backed up; packages in CAS.

---

# 190. Golden Path Config

Authoritative org metadata.

---

# 191. Historical Runs

Exact expansion retained enough to inspect even if template removed.

---

# 192. Store Expansion Digest

---

# 193. Optional Expanded Source

CAS for audit/debug.

---

# 194. Template Removal

Can yank/deprecate, not erase historical identities.

---

# 195. Compatibility With Pipeline IR Schema

Template version declares supported PipelineSchemaVersion range.

---

# 196. TemplateCompatibilityMatrix

```rust
pub struct TemplateCompatibilityMatrix {
    pub template: TemplateRef,
    pub pipeline_schema: VersionRange,
    pub forgeyard_version: VersionRange,
}
```

---

# 197. Upgrade Forgeyard

Template registry checks compatibility.

---

# 198. Old Template

Can continue if supported.

---

# 199. Unsupported Template

Planner rejects with migration guidance.

---

# 200. Template Normalization

All parameter values canonicalized.

---

# 201. Map Ordering

Deterministic.

---

# 202. String Paths

Validated.

---

# 203. Enum Unknown

Reject.

---

# 204. Template Expansion Error

```rust
pub enum TemplateExpansionError {
    UnknownTemplate,
    UnsupportedVersion,
    ParameterMissing,
    ParameterInvalid,
    DependencyCycle,
    ExpansionTooLarge,
    PolicyDenied,
    IncompatiblePipelineSchema,
}
```

---

# 205. Diagnostics

Span-aware invocation source.

---

# 206. Nested Template Error

Show chain.

---

# 207. Template Cycle

Exact cycle path.

---

# 208. Organization Standard Evaluation

```rust
pub struct GoldenPathEvaluation {
    pub project: ProjectId,
    pub required: Vec<TemplateRequirement>,
    pub present: Vec<TemplateRef>,
    pub missing: Vec<TemplateRequirement>,
}
```

---

# 209. Policy Decision

Missing mandatory standard can fail validation.

---

# 210. Exceptions

Use PolicyException, not template-specific bypass.

---

# 211. Exception

Scoped/reasoned/expiring.

---

# 212. Break-Glass

Still audited.

---

# 213. Template Slot Security

A project cannot insert arbitrary step inside protected signing stage.

---

# 214. Protected Slot

No extension.

---

# 215. Append Slot

Project additions allowed after/before.

---

# 216. Output Binding

Cannot spoof output from skipped required job.

---

# 217. Pipeline Compiler

Validates data dependency.

---

# 218. Template Scope Isolation

Tenant A cannot resolve Tenant B private template.

---

# 219. Public Template Registry

Future optional.

---

# 220. External Marketplace

Not baseline.

---

# 221. If Added

Use package/signature/trust model similar to plugins.

---

# 222. Built-In Template Versioning

Do not silently change behavior under same Forgeyard release/template ID.

---

# 223. Security Fix

Publish new template version/minimum policy.

---

# 224. Organization Emergency

Set minimum allowed template version.

---

# 225. Existing Run

Unaffected historical truth.

---

# 226. New Run

Resolves current policy.

---

# 227. Cached Pipeline Plan

Template exact refs part of plan key.

---

# 228. Template Change

Changes PipelinePlanId.

---

# 229. Cache Correctness

Expanded semantics enter derivation/job keys normally.

---

# 230. No Template-Specific Cache Shortcut

---

# 231. Testkit

```text
forgeyard-template-testkit/src/
├── lib.rs
├── template.rs
├── parameters.rs
├── resolve.rs
├── expand.rs
├── trust.rs
├── upgrade.rs
└── assertions.rs
```

Golden paths:

```text
forgeyard-golden-path-testkit/
```

---

# 232. Unit Tests

Parameter typing/defaults.

---

# 233. Deterministic Expansion Test

Same exact inputs -> same expansion digest.

---

# 234. Mutable Selector Test

Resolved before plan.

---

# 235. Cycle Test

Rejected.

---

# 236. Expansion Bomb Test

Bound enforced.

---

# 237. Secret Test

Plaintext secret parameter impossible/rejected.

---

# 238. Protected Slot Test

Project cannot override.

---

# 239. Mandatory Job Test

Policy-required job remains.

---

# 240. Trust Test

External untrusted template cannot request privileged capability without policy.

---

# 241. Tenant Isolation Test

Private template inaccessible cross-tenant.

---

# 242. Version Upgrade Test

Compatibility classified.

---

# 243. Breaking Upgrade Test

Requires migration.

---

# 244. Golden Path Missing Test

Policy evaluation fails/warns as configured.

---

# 245. Project Exception Test

Uses normal PolicyException.

---

# 246. Historical Run Test

Exact old template/expansion inspectable.

---

# 247. Registry Outage Test

Pinned template still available.

---

# 248. Air-Gap Test

Bundle resolves offline.

---

# 249. Forgeyard Upgrade Compatibility Test

Unsupported template blocked clearly.

---

# 250. Fuzzing

Fuzz:

```text
template parser
parameter decoder
nested expansion
version requirement parser
```

---

# 251. Property Tests

Expansion deterministic/canonical.

---

# 252. Scale Test

Large org template registry.

---

# 253. Large Expansion Test

Thousands of jobs bounded.

---

# 254. Failure Injection

```text
registry DB unavailable
CAS package missing
signature invalid
nested dependency missing
```

---

# 255. Implementation Phase 1 — Template Model/Expansion

Job/stage fragments.

---

# 256. Phase 2 — Typed Parameters/Outputs

Core composition.

---

# 257. Phase 3 — Registry/Immutable Versions

Organization reuse.

---

# 258. Phase 4 — Golden Paths/Mandatory Standards

Governance.

---

# 259. Phase 5 — Trust/Signing

Enterprise.

---

# 260. Phase 6 — Upgrade/Diff/Migration

Long-term maintainability.

---

# 261. Phase 7 — Developer Init/Preview

DX.

---

# 262. Phase 8 — Search/UI/Analytics

Discoverability.

---

# 263. Phase 9 — Air-Gap/Offline

Enterprise.

---

# 264. Phase 10 — Polyglot Golden Paths

Rust/Go/JS/Python/JVM/mobile.

---

# 265. Phase 11 — Compatibility/Upgrade Hardening

Rolling evolution.

---

# 266. Phase 12 — Scale/Fuzz/DR

Production readiness.

---

# 267. Acceptance Tests

1. Templates compile into canonical Pipeline IR.
2. Templates never create a second execution engine.
3. Every template invocation resolves to immutable TemplateRef before planning.
4. Template parameters are strongly typed.
5. Secret values cannot be passed as plaintext parameters.
6. Template expansion is deterministic.
7. Template expansion has no network/time/randomness dependency.
8. Nested template dependencies resolve exactly.
9. Template dependency cycles are rejected.
10. Expansion size/depth are bounded.
11. Organization-required stages cannot be removed by project config.
12. Policy-required jobs are never subtracted by template overrides.
13. Protected template slots cannot be replaced.
14. Template trust does not bypass authz/policy.
15. External templates cannot self-grant privileged capabilities.
16. Every expanded job can be traced to project/template/policy origin.
17. PipelinePlan records exact template provenance.
18. Template version changes alter plan identity.
19. Mutable channels/version ranges do not survive into execution identity.
20. Template upgrades show security/capability diffs.
21. Breaking upgrades require explicit migration.
22. Historical runs remain inspectable after template deprecation/yank.
23. Tenant-private templates are isolated.
24. Pinned templates remain usable during registry outage.
25. Air-gapped template bundles verify offline.
26. Repository templates cannot override organization security minima.
27. Golden-path exceptions use normal policy exception semantics.
28. Forgeyard-version/template-schema compatibility is checked.
29. Cached plan/job semantics include expanded template behavior.
30. Template registry metadata and CAS objects survive DR.
31. UI exposes template expansion rather than hiding it.
32. Search/analytics are derived.
33. Standalone/distributed share template semantics.
34. Organization golden paths can evolve without silently changing existing runs.
35. Forgeyard dogfoods reusable templates/golden paths for its own CI pipeline.

---

# 268. Production Readiness Gates

Do not call workflow templates production-ready until:

```text
typed parameter/expansion model is stable
immutable version resolution works
nested cycles/size bounds are enforced
template provenance reaches PipelinePlan
protected organization stages cannot be overridden
trust/policy capability checks pass
upgrade/diff tooling is reliable
registry outage/offline path works
tenant isolation passes
DR/compatibility tests pass
```

---

# 269. Architectural Invariants

1. templates are source-level composition only;
2. Pipeline IR remains execution truth;
3. template refs resolve immutably before planning;
4. parameters/outputs are typed;
5. secret plaintext is forbidden;
6. expansion is deterministic;
7. expansion does not perform arbitrary network/code execution;
8. nesting is bounded/acyclic;
9. template trust is explicit;
10. trust never bypasses policy;
11. project overrides cannot remove protected controls;
12. organization/policy requirements only expand mandatory work;
13. every expanded job has provenance;
14. template changes alter plan identity;
15. mutable channels are never execution identity;
16. upgrades are diffable/migratable;
17. historical runs preserve exact template identity;
18. yanking affects new resolution, not history;
19. tenant-private templates remain isolated;
20. registry outage does not break pinned templates;
21. golden paths are inspectable, not hidden;
22. project freedom exists through typed parameters/slots;
23. security-sensitive template changes are highlighted;
24. template packages are immutable;
25. template schema/Forgeyard compatibility is explicit;
26. exceptions use central policy mechanisms;
27. cache semantics derive from expanded IR;
28. standalone/distributed share semantics;
29. template registry is recoverable;
30. Forgeyard dogfoods its own workflow templates and golden paths.

---

# 270. Final Target Architecture

```text
                    Template Registry
                           │
                           ▼
                    Exact TemplateRef
                           │
                           ▼
                   Typed Invocation
                           │
                           ▼
                Deterministic Expansion
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
         Project Jobs   Golden Path   Policy Jobs
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                      Pipeline IR
                           │
                           ▼
                     Normal Planning
                           │
                           ▼
                        Execution
```

---

# 271. Final Architectural Position

Template invocation:

```text
TemplateSelector
+
typed parameters
  ↓
resolve exact version/digest
  ↓
TemplateRef
  ↓
deterministic expansion
  ↓
PipelineFragment
```

Organization governance:

```text
ProjectPipeline
+
TemplateExpansion
+
GoldenPathRequired
+
PolicyRequired
  ↓
Final Pipeline IR
```

Upgrade:

```text
current exact template
+
candidate exact template
  ↓
parameter/output/capability/job diff
  ↓
compatibility classification
  ↓
migration patch/proposal
```

The key guarantee is:

> **Forgeyard can standardize CI/CD across many projects without hiding what actually runs. Reusable templates and golden paths reduce duplication and let organizations enforce secure defaults, but every invocation resolves to immutable content, expands deterministically into ordinary Pipeline IR, and remains fully inspectable before execution.**

---

# 272. Extended Architecture Sequence

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
```
