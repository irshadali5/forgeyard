# 49 — Forgeyard Service Catalog, Component Ownership, Environment Inventory & Developer Portal System Architecture

**Document type:** Core Software Catalog, Component Ownership, Environment Inventory, Developer Portal & Organizational Metadata System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** software/service/component catalog, ownership, repository bindings, runtime environment inventory, dependency/service relationships, deployment inventory, golden-path adoption, operational metadata, documentation links, health/maturity scorecards, team contacts, discoverability, onboarding, and developer portal integration  
**Architecture style:** Derived organizational view over canonical Forgeyard resources, typed component identities, explicit ownership provenance, immutable resource references, policy-aware metadata, rebuildable indexes, and no second source of truth for execution or deployment state  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Core Domain, VCS-neutral Source, Change Proposal, Deployment, Release, Search/Analytics, Workflow Templates/Golden Paths, Monorepo Intelligence, Identity/Authz, Notifications, Audit, Configuration, Multi-Tenancy, and Developer Experience. This subsystem gives organizations a coherent “what software do we have, who owns it, where does it run, and how healthy/standardized is it?” layer.

---

# 1. Purpose

Large organizations quickly accumulate:

```text
services
libraries
CLIs
mobile apps
desktop apps
workers
internal tools
shared packages
infrastructure components
databases
deployment environments
repositories
owners
dependencies
```

Without a catalog, developers ask:

```text
who owns this service?
where is it deployed?
which repository builds it?
which team should approve a change?
what depends on it?
which golden path does it use?
is it production-critical?
what release is running in staging?
what documentation/runbook exists?
```

The central rule is:

> **The Forgeyard catalog is an organizational discovery and relationship layer over canonical resources. It never becomes the execution truth for builds, releases, deployments, policies, or identities.**

A second rule is:

> **Ownership must be explicit, scoped, explainable, and provenance-aware. A catalog entry cannot silently invent authority that is not supported by identity, repository ownership, project policy, or organization configuration.**

A third rule is:

> **Environment inventory and deployment status are projections from the Deployment subsystem. The catalog may summarize where software runs, but Deployment remains authoritative for desired/observed runtime state.**

---

# 2. Architectural Position

```text
               Canonical Forgeyard Domains
      ┌──────────────┼──────────────┐
      ▼              ▼              ▼
   Projects       Releases       Deployments
      │              │              │
      └──────────────┼──────────────┘
                     ▼
               Catalog Ingestion
                     │
                     ▼
               Relationship Graph
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
      Ownership   Environments  Dependencies
         │           │           │
         └───────────┼───────────┘
                     ▼
                Developer Portal
```

---

# 3. Goals

The subsystem MUST:

1. define software component identity;
2. support services;
3. support libraries;
4. support CLIs;
5. support desktop/mobile apps;
6. support workers/jobs;
7. support infrastructure components;
8. support repository bindings;
9. support ownership;
10. support team ownership;
11. support environment inventory;
12. support deployment inventory;
13. support dependencies/relationships;
14. support golden-path adoption metadata;
15. support documentation/runbook links;
16. support lifecycle/maturity metadata;
17. support discoverability/search;
18. support scorecards;
19. support operational contacts;
20. support security/compliance metadata projections;
21. support onboarding;
22. support tenant/org isolation;
23. support audit;
24. support API/UI/CLI;
25. support rebuildable projections;
26. support catalog drift detection;
27. support standalone mode;
28. support distributed mode;
29. remain derived where possible;
30. never replace canonical domain authority.

---

# 4. Non-Goals

This subsystem does not:

```text
replace project metadata
replace deployment state
replace policy
replace repository ownership authority
replace incident management
replace documentation hosting
replace CMDB/accounting systems
```

It may integrate with those systems.

---

# 5. Workspace Structure

```text
crates/catalog/
├── forgeyard-catalog/
├── forgeyard-catalog-model/
├── forgeyard-catalog-ingest/
├── forgeyard-catalog-ownership/
├── forgeyard-catalog-relations/
├── forgeyard-catalog-environment/
├── forgeyard-catalog-scorecard/
├── forgeyard-catalog-discovery/
├── forgeyard-catalog-reconcile/
├── forgeyard-catalog-health/
└── forgeyard-catalog-testkit/
```

Portal integration:

```text
crates/portal/
├── forgeyard-portal/
├── forgeyard-portal-model/
└── forgeyard-portal-api/
```

Use modules first; split only where runtime/dependency boundaries justify.

---

# 6. SoftwareComponentId

```rust
pub struct SoftwareComponentId(Ulid);
```

Stable logical component identity.

---

# 7. Component Kind

```rust
pub enum SoftwareComponentKind {
    Service,
    Library,
    Cli,
    DesktopApp,
    MobileApp,
    WebApp,
    Worker,
    ScheduledJob,
    Package,
    Infrastructure,
    Database,
    DataPipeline,
    DeviceSoftware,
    Custom(ComponentKindId),
}
```

---

# 8. Component

```rust
pub struct SoftwareComponent {
    pub id: SoftwareComponentId,
    pub tenant: TenantId,
    pub organization: Option<OrganizationId>,
    pub name: ComponentName,
    pub kind: SoftwareComponentKind,
    pub lifecycle: ComponentLifecycle,
}
```

---

# 9. Component Lifecycle

```rust
pub enum ComponentLifecycle {
    Experimental,
    Active,
    Deprecated,
    Maintenance,
    Retired,
}
```

---

# 10. Lifecycle Is Organizational Metadata

It does not replace release/deployment state.

---

# 11. Component Identity vs ProjectId

Not always one-to-one.

Examples:

```text
one monorepo project -> many services/libraries
one component -> multiple repositories
shared library -> one component reused by many projects
```

---

# 12. Project Binding

```rust
pub struct ComponentProjectBinding {
    pub component: SoftwareComponentId,
    pub project: ProjectId,
    pub role: ComponentProjectRole,
}
```

---

# 13. Component Project Role

```rust
pub enum ComponentProjectRole {
    Primary,
    Source,
    Build,
    Deployment,
    Documentation,
}
```

---

# 14. Repository Binding

```rust
pub struct ComponentRepositoryBinding {
    pub component: SoftwareComponentId,
    pub repository: RepositoryId,
    pub path: Option<RepoRelativePath>,
}
```

---

# 15. Monorepo Component

Repository + path.

---

# 16. Path Is Context

Monorepo graph/ownership remains Part 34 authority for source impact.

---

# 17. Component Descriptor

Human-authored optional RON.

Example:

```ron
(
    name: "forgeyard-daemon",
    kind: Service,
    lifecycle: Active,
    owners: ["team/platform"],
    docs: ["docs://forgeyard-daemon"],
)
```

---

# 18. Descriptor Location

Could be:

```text
.forgeyard/component.ron
```

or organization catalog config.

---

# 19. Descriptor Is Declarative Metadata

Not executable.

---

# 20. Repository Descriptor Trust

Cannot grant permissions.

---

# 21. Ownership

Core concept.

---

# 22. OwnershipBindingId

```rust
pub struct OwnershipBindingId(Ulid);
```

---

# 23. Ownership Binding

```rust
pub struct OwnershipBinding {
    pub id: OwnershipBindingId,
    pub component: SoftwareComponentId,
    pub owner: OwnershipSubject,
    pub kind: OwnershipKind,
    pub source: OwnershipSource,
}
```

---

# 24. Ownership Subject

```rust
pub enum OwnershipSubject {
    Team(OrganizationUnitId),
    Principal(PrincipalId),
    ServiceAccount(PrincipalId),
}
```

---

# 25. Ownership Kind

```rust
pub enum OwnershipKind {
    Primary,
    Technical,
    Operational,
    Security,
    Product,
    Documentation,
}
```

---

# 26. Ownership Source

```rust
pub enum OwnershipSource {
    OrganizationCatalog,
    RepositoryDeclaration,
    CodeOwnership,
    ManualAdmin,
    ImportedExternal,
}
```

---

# 27. Ownership Confidence

```rust
pub enum OwnershipConfidence {
    Explicit,
    Derived,
    Imported,
    Unknown,
}
```

---

# 28. Ownership Does Not Automatically Grant Permission

Critical.

---

# 29. Authz

Part 11 remains authority.

---

# 30. Ownership May Influence

```text
review routing
notifications
approver suggestions
on-call routing
```

---

# 31. Ownership May Be Policy Input

Only if policy explicitly uses it.

---

# 32. No Implicit Permission From "Owner" Label

Critical.

---

# 33. Team Identity

Use organization/directory groups.

---

# 34. External Team

Imported mapping must resolve to Forgeyard OrganizationUnitId where possible.

---

# 35. Ownership Drift

Example:

```text
catalog owner != CODEOWNERS-derived team
```

---

# 36. Drift Is Diagnostic

Do not silently pick one.

---

# 37. OwnershipResolution

```rust
pub struct OwnershipResolution {
    pub explicit: Vec<OwnershipBinding>,
    pub derived: Vec<OwnershipBinding>,
    pub conflicts: Vec<OwnershipConflict>,
}
```

---

# 38. Conflict

First-class.

---

# 39. Operational Contact

```rust
pub struct OperationalContact {
    pub component: SoftwareComponentId,
    pub team: Option<OrganizationUnitId>,
    pub escalation: Option<NotificationRouteId>,
}
```

---

# 40. Contact Is Routing Metadata

Not security authority.

---

# 41. Component Relationship

```rust
pub struct ComponentRelation {
    pub from: SoftwareComponentId,
    pub to: SoftwareComponentId,
    pub kind: ComponentRelationKind,
    pub source: RelationSource,
}
```

---

# 42. Relation Kind

```rust
pub enum ComponentRelationKind {
    DependsOn,
    Calls,
    PublishesTo,
    ConsumesFrom,
    StoresIn,
    BuiltFrom,
    DeploysWith,
    Owns,
    ProvidesApiTo,
    Custom(ComponentRelationKindId),
}
```

---

# 43. Relation Source

```rust
pub enum RelationSource {
    Declared,
    MonorepoGraph,
    DeploymentConfig,
    ObservedTelemetry,
    Imported,
    Unknown,
}
```

---

# 44. Declared vs Observed

Keep separate.

---

# 45. Runtime Observed Calls

Optional derived telemetry.

---

# 46. Do Not Treat Telemetry as Complete Dependency Truth

Critical.

---

# 47. Dependency Confidence

```rust
pub enum RelationConfidence {
    Exact,
    Declared,
    Observed,
    Inferred,
    Unknown,
}
```

---

# 48. Component Graph

Derived relationship graph.

---

# 49. Graph Uses Part 34 Where Applicable

Source/build dependencies can project upward into component relationships.

---

# 50. Environment Inventory

Represent runtime environments.

---

# 51. EnvironmentId

Reuse canonical deployment environment identity.

---

# 52. Environment Descriptor

```rust
pub struct CatalogEnvironmentView {
    pub environment: EnvironmentId,
    pub name: EnvironmentName,
    pub classification: EnvironmentClass,
    pub region: Option<RegionRef>,
}
```

---

# 53. Environment Class

```rust
pub enum EnvironmentClass {
    Development,
    Test,
    Staging,
    Production,
    DisasterRecovery,
    Preview,
    Custom(EnvironmentClassId),
}
```

---

# 54. Catalog Environment Is View

Deployment subsystem remains authority.

---

# 55. Deployment Inventory

For component:

```text
environment
ReleaseId
DeploymentRevisionId
health
deployed_at
```

---

# 56. CatalogDeploymentView

```rust
pub struct CatalogDeploymentView {
    pub component: SoftwareComponentId,
    pub environment: EnvironmentId,
    pub release: ReleaseId,
    pub deployment: DeploymentId,
    pub health: DeploymentHealth,
}
```

---

# 57. Never Store "current prod version" Manually

Critical.

Derive from deployment authority.

---

# 58. Multiple Regions

Many deployments per environment.

---

# 59. Multi-Instance

Catalog aggregates carefully.

---

# 60. Deployment Drift

Shown from Part 16.

---

# 61. Release Inventory

Component release history.

---

# 62. Build Source

Link:

```text
ReleaseId
  ↓
Artifact/Package
  ↓
SourceSnapshotId
```

---

# 63. Developer Portal

Primary user experience.

---

# 64. Portal Home

Can show:

```text
My Components
My Teams
Recent Changes
Deployments
Incidents/Alerts
Golden Path Updates
```

---

# 65. "My Components"

Derived from explicit/derived ownership.

---

# 66. No Hidden Behavioral Ranking

Critical.

---

# 67. Component Page

Shows:

```text
description
kind/lifecycle
owners
repositories
dependencies
latest releases
environments
deployment health
docs/runbooks
golden path
scorecards
```

---

# 68. Documentation Links

```rust
pub struct DocumentationRef {
    pub kind: DocumentationKind,
    pub uri: SafeUri,
}
```

---

# 69. Documentation Kind

```text
Overview
Architecture
Runbook
API
Operations
Security
ADR
```

---

# 70. Safe URI

Allowlisted schemes/domains where needed.

---

# 71. Documentation Hosting

Not core responsibility.

---

# 72. Runbook Link

Operational.

---

# 73. API Metadata

Optional OpenAPI/protobuf links.

---

# 74. API Contract Identity

Could point to CAS artifact.

---

# 75. Golden Path Adoption

Part 42.

---

# 76. GoldenPathAdoption

```rust
pub struct GoldenPathAdoption {
    pub component: SoftwareComponentId,
    pub path: GoldenPathId,
    pub template: TemplateRef,
    pub status: GoldenPathAdoptionState,
}
```

---

# 77. Adoption State

```rust
pub enum GoldenPathAdoptionState {
    Current,
    UpgradeAvailable,
    NonCompliant,
    Exempted,
    Unknown,
}
```

---

# 78. Compliance

Derived from template/policy evaluation.

---

# 79. Scorecards

Need careful architecture.

---

# 80. ScorecardId

```rust
pub struct ScorecardId(Digest);
```

---

# 81. Scorecard

A set of explainable checks.

---

# 82. ScorecardCheck

```rust
pub struct ScorecardCheck {
    pub id: ScorecardCheckId,
    pub title: BoundedString,
    pub evaluator: ScorecardEvaluator,
    pub severity: ScorecardSeverity,
}
```

---

# 83. Scorecard Evaluators

Can check:

```text
has owner
has runbook
uses current golden path
has recent successful release
has SBOM
has vulnerability policy
has production rollback strategy
```

---

# 84. Scorecard Is Advisory by Default

Critical.

---

# 85. Policy Enforcement

If organization wants mandatory control, Part 11 policy should enforce it.

---

# 86. Do Not Make Scorecard Itself Policy Authority

Critical.

---

# 87. Scorecard Result

```rust
pub enum ScorecardResult {
    Pass,
    Warning,
    Fail,
    Unknown,
    NotApplicable,
}
```

---

# 88. Unknown Is First-Class

---

# 89. No Single "Engineering Quality Score" Baseline

Critical.

---

# 90. If Aggregated Score Exists

Presentation only, transparent formula.

---

# 91. Maturity Model

Optional.

---

# 92. MaturityLevel

```rust
pub enum MaturityLevel {
    Experimental,
    Emerging,
    Standard,
    Critical,
}
```

---

# 93. Maturity Is Organizational Metadata

Not automatic trust.

---

# 94. Criticality

Separate from maturity.

---

# 95. CriticalityClass

```rust
pub enum CriticalityClass {
    Low,
    Medium,
    High,
    MissionCritical,
}
```

---

# 96. Criticality Can Influence Policy

Explicitly.

---

# 97. No Criticality Self-Upgrade by Project

Admin/org governance.

---

# 98. Catalog Metadata Source

Can come from:

```text
repository descriptor
organization registry
deployment projection
release projection
monorepo graph
telemetry
external import
```

---

# 99. Canonical vs Derived Fields

Must be marked.

---

# 100. CatalogFieldProvenance

```rust
pub struct CatalogFieldProvenance {
    pub field: CatalogFieldPath,
    pub source: CatalogSource,
    pub observed_at: Timestamp,
}
```

---

# 101. User Can Explain Field

`forgeyard catalog explain`.

---

# 102. CatalogSource

```rust
pub enum CatalogSource {
    Repository,
    Organization,
    Project,
    Deployment,
    Release,
    MonorepoGraph,
    Telemetry,
    Imported,
}
```

---

# 103. Conflict Resolution

Deterministic per field.

---

# 104. Example

Description:

```text
organization override > repository descriptor
```

if configured.

---

# 105. Ownership

Do not silently override conflicts; surface.

---

# 106. Environment State

Always deployment projection.

---

# 107. Catalog Reconciliation

Periodically rebuild/update projections.

---

# 108. Event Fast Path

Release/deployment/project changes update catalog quickly.

---

# 109. Reconciliation Slow Path

Correct drift.

---

# 110. Catalog Is Rebuildable

As much as possible.

---

# 111. Non-Derived Metadata

Examples:

```text
description
business domain
criticality
docs links
```

stored authoritatively in catalog metadata.

---

# 112. Keep Narrow

Avoid duplicating core resource fields.

---

# 113. CatalogDescriptorVersion

```rust
pub struct CatalogDescriptorVersion(u16);
```

---

# 114. RON Schema

Versioned.

---

# 115. Repository Descriptor Validation

No arbitrary HTML/JS.

---

# 116. Safe Markdown

Sanitize.

---

# 117. Labels/Tags

Typed/bounded.

---

# 118. Tag Taxonomy

Organization-controlled optional.

---

# 119. Freeform Tags

Can exist but not used for security decisions.

---

# 120. Business Domain

```rust
pub struct BusinessDomainId(BoundedString);
```

---

# 121. Domain Hierarchy

Optional.

---

# 122. Component-to-Domain Binding

Organizational.

---

# 123. Search

Part 31 first-class.

---

# 124. Search Fields

```text
component name
owner
team
repository
language/ecosystem
environment
criticality
golden path
tag
```

---

# 125. Search Authorization

Critical.

---

# 126. Private Components

Not visible cross-tenant/org unless permitted.

---

# 127. Portal Navigation

Could be:

```text
Catalog
Teams
Environments
Dependencies
Scorecards
```

---

# 128. Dependency Graph UI

Subgraph for component.

---

# 129. Blast Radius

Can combine component graph + deployment inventory.

---

# 130. Change Impact

Part 34 can project affected components.

---

# 131. Change Proposal UI

Show impacted components/owners.

---

# 132. Notifications

Routing can use ownership.

---

# 133. Examples

```text
release failed -> technical owner
production deploy degraded -> operational owner
security finding -> security owner
```

---

# 134. Ownership Routing Fallback

Project/team admin.

---

# 135. No Notification Drop Because Owner Missing

Critical.

---

# 136. Missing Ownership

Scorecard/diagnostic.

---

# 137. Onboarding

`forgeyard catalog init`

can create descriptor from existing project.

---

# 138. Auto-Discovery

Advisory.

---

# 139. Detect

```text
Cargo workspace
package manifests
deployment configs
existing services
```

---

# 140. User Reviews

No silent component creation unless configured.

---

# 141. Monorepo Discovery

Use Part 34 graph.

---

# 142. Component Boundary

Can be declared.

---

# 143. Inferred Boundary

Confidence explicit.

---

# 144. External Catalog Import

Potential:

```text
Backstage
CMDB
service inventory
```

---

# 145. Imported Ownership

`Imported` confidence.

---

# 146. No External Catalog Becomes Authz Authority

Critical.

---

# 147. Catalog Federation

Optional future.

---

# 148. External IDs

Preserve mapping.

---

# 149. CatalogImportId

```rust
pub struct CatalogImportId(Ulid);
```

---

# 150. Imported Data

Normalized.

---

# 151. Drift

Can compare external vs Forgeyard.

---

# 152. API

Potential:

```text
GET  /v1/catalog/components
GET  /v1/catalog/components/{id}
POST /v1/catalog/components
PATCH /v1/catalog/components/{id}
GET  /v1/catalog/components/{id}/deployments
GET  /v1/catalog/components/{id}/relations
GET  /v1/catalog/scorecards
```

---

# 153. Permissions

```text
catalog.read
catalog.manage
catalog.ownership.manage
catalog.criticality.manage
catalog.scorecard.manage
catalog.import
```

---

# 154. Ownership Manage

Does not grant runtime permissions.

---

# 155. Criticality Manage

Restricted.

---

# 156. Audit

Audit:

```text
owner change
criticality change
component lifecycle change
catalog import
scorecard definition change
```

---

# 157. Derived deployment updates

Operational projection events, not privileged audit.

---

# 158. CLI

```text
forgeyard catalog list
forgeyard catalog show <component>
forgeyard catalog explain <component>
forgeyard catalog graph <component>
forgeyard catalog owner set
forgeyard catalog scorecard
forgeyard catalog doctor
```

---

# 159. Machine Output

JSON/RON.

---

# 160. Dioxus UI

Native portal.

---

# 161. Mobile UI

Read-heavy, concise.

---

# 162. Desktop

Graph/scorecard detail.

---

# 163. Accessibility

Same Part 19 standards.

---

# 164. Scorecard Definition

RON.

---

# 165. Scorecard Evaluation

Derived worker.

---

# 166. Re-evaluation Trigger

On:

```text
release
deployment
policy
template
owner
finding
```

---

# 167. Scorecard Freshness

```rust
pub enum ScorecardFreshness {
    Fresh,
    Stale,
    Unknown,
}
```

---

# 168. Stale Result

Shown as stale, not current.

---

# 169. Catalog Health

Checks:

```text
missing owner
orphan component
broken repository binding
deployment projection lag
relation cycles where invalid
stale scorecards
```

---

# 170. Doctor

```text
forgeyard catalog doctor
```

---

# 171. Doctor Checks

```text
components without owners
components without source binding
retired but still deployed
prod component without runbook
critical component without rollback plan
```

---

# 172. Doctor Is Diagnostic

Policy decides enforcement.

---

# 173. Observability Metrics

```text
catalog_components_total
catalog_components_missing_owner
catalog_projection_lag_seconds
catalog_scorecard_evaluations_total
catalog_reconcile_failures_total
```

---

# 174. Labels

Low-cardinality:

```text
kind
lifecycle
result
```

---

# 175. No component IDs in metrics.

---

# 176. Tracing

```text
catalog.ingest
catalog.resolve_owner
catalog.project_deployment
catalog.relate
catalog.scorecard
catalog.reconcile
```

---

# 177. Multi-Tenancy

Every component belongs to tenant/org scope.

---

# 178. Cross-Tenant Relation

Forbidden unless explicit public/shared component model.

---

# 179. Shared Component

Can be installation-public if policy.

---

# 180. Visibility

Separate from ownership.

---

# 181. ComponentVisibility

```rust
pub enum ComponentVisibility {
    Private,
    Organization,
    Installation,
    Public,
}
```

---

# 182. Public

Only metadata explicitly allowed.

---

# 183. Source Code

Never exposed merely because catalog entry is public.

---

# 184. Data Lifecycle

Part 46.

---

# 185. Retired Component

Catalog metadata may remain long after execution artifacts expire.

---

# 186. Ownership History

Retain for audit.

---

# 187. Search Index

Derived/rebuildable.

---

# 188. DR

Catalog authoritative metadata backed up.

---

# 189. Derived relationships/projections can rebuild.

---

# 190. Standalone Mode

Simple project/component catalog.

---

# 191. Distributed Mode

Organization-wide portal.

---

# 192. Performance

Catalog read-heavy.

---

# 193. Read Models

Use Part 31 indexing/read models.

---

# 194. Graph Query Limits

Bound traversal.

---

# 195. No unbounded transitive graph query.

---

# 196. Blast Radius Query

Depth/edge type limits.

---

# 197. Catalog Change Concurrency

Optimistic versioning.

---

# 198. ComponentVersion

```rust
pub struct CatalogComponentVersion(u64);
```

---

# 199. Update

ETag/If-Match via API.

---

# 200. Ownership Conflict Resolution

Human/admin.

---

# 201. Auto-Derived Owner

Never silently overwrites explicit owner.

---

# 202. Repository Rename

Binding tracks RepositoryId, not name only.

---

# 203. Project Rename

ProjectId stable.

---

# 204. Environment Rename

EnvironmentId stable.

---

# 205. Component Rename

Stable SoftwareComponentId.

---

# 206. Component Merge

Explicit migration.

---

# 207. Component Split

Explicit migration.

---

# 208. Catalog Alias

Can preserve old name.

---

# 209. Alias Does Not Duplicate identity.

---

# 210. External ID Mapping

For CMDB/backstage integration.

---

# 211. API Contract Catalog

Optional.

---

# 212. ContractRef

```rust
pub struct ContractRef {
    pub artifact: CasObjectRef,
    pub kind: ContractKind,
}
```

---

# 213. Contract Kind

```text
OpenAPI
AsyncAPI
Protobuf
GraphQLSchema
Custom
```

---

# 214. Contract Difference

Can integrate Change Proposal semantic evidence.

---

# 215. Component Dependency

May be derived from contract/deployment config.

---

# 216. Security View

Component page can summarize:

```text
critical findings
SBOM status
last provenance
security owner
```

---

# 217. Findings Authority

Part 37.

---

# 218. Security Status

Projection only.

---

# 219. Compliance View

Part 28 evidence projection.

---

# 220. Cost View

Part 45 project/component cost allocation.

---

# 221. Cost Authority

Part 45.

---

# 222. Reliability View

Part 17/16.

---

# 223. SLO

If future SLO subsystem exists, catalog links only.

---

# 224. Portal Extensibility

Plugins may add read-only panels.

---

# 225. Plugin Panel

Must not bypass authz.

---

# 226. Plugin Cannot inject arbitrary unsafe HTML.

---

# 227. Template/Golden Path Portal

Show:

```text
current template
upgrade available
org standard
exceptions
```

---

# 228. Migration View

Part 47 can map legacy service inventory.

---

# 229. Catalog Import During Migration

Useful.

---

# 230. Imported component trust

Explicit.

---

# 231. Catalog Completeness

```rust
pub enum CatalogCompleteness {
    Complete,
    Partial,
    Unknown,
}
```

---

# 232. Large Organizations

Do not imply complete inventory unless coverage known.

---

# 233. Discovery Coverage

Can show:

```text
repositories scanned
projects represented
components declared
```

---

# 234. No Fake "100%" Without denominator.

---

# 235. Scorecard Example

```text
Ownership          PASS
Runbook            WARNING
Golden Path        PASS
SBOM               PASS
Rollback Strategy  UNKNOWN
```

---

# 236. Scorecard Evidence

Each result links exact source/resource.

---

# 237. ScorecardEvaluationId

```rust
pub struct ScorecardEvaluationId(Digest);
```

---

# 238. Evaluation Inputs

```text
component version
relevant projections
scorecard definition version
```

---

# 239. Reprocessing

Possible.

---

# 240. Historical Scorecards

Optional/derived.

---

# 241. Team Page

Shows:

```text
owned components
critical components
pending upgrades
recent deployments
```

---

# 242. Team Membership

Directory/identity authority.

---

# 243. Catalog Does Not Store separate authoritative membership.

---

# 244. Environment Page

Shows all deployed components.

---

# 245. Production Inventory

High-value.

---

# 246. Runtime State

Deployment projection.

---

# 247. Disaster Recovery Inventory

Can show DR-capable components.

---

# 248. Readiness

Scorecard/policy-derived.

---

# 249. Component Retire Workflow

```text
mark Deprecated
  ↓
remove production deployments
  ↓
archive docs/artifacts per policy
  ↓
mark Retired
```

---

# 250. Retired But Deployed

Doctor warning/fail according policy.

---

# 251. Deletion

Part 46 governs actual metadata deletion.

---

# 252. Catalog Tombstone

Preserve old component ID/name if deleted.

---

# 253. Testkit

```text
forgeyard-catalog-testkit/src/
├── lib.rs
├── component.rs
├── ownership.rs
├── relation.rs
├── environment.rs
├── scorecard.rs
├── reconcile.rs
└── assertions.rs
```

---

# 254. Unit Tests

Component identity/lifecycle.

---

# 255. Ownership Test

Owner label does not grant authz.

---

# 256. Ownership Conflict Test

Conflict surfaced.

---

# 257. Repository Binding Test

Rename retains binding.

---

# 258. Monorepo Path Test

Component maps correct subtree.

---

# 259. Deployment Projection Test

Current prod release derived from Deployment.

---

# 260. Manual Prod Version Test

Rejected/not allowed as authority.

---

# 261. Scorecard Test

Unknown remains Unknown.

---

# 262. Scorecard Policy Test

Advisory unless policy consumes.

---

# 263. Tenant Isolation Test

No cross-tenant catalog visibility.

---

# 264. Search Authorization Test

Private component hidden.

---

# 265. Telemetry Relation Test

Observed call does not become Exact.

---

# 266. Derived Owner Test

Does not overwrite explicit owner.

---

# 267. Retired Deployment Test

Doctor reports.

---

# 268. Import Test

External catalog cannot grant permissions.

---

# 269. DR Test

Authoritative metadata restores; projections rebuild.

---

# 270. Fuzzing

Fuzz descriptor parser/import payloads.

---

# 271. Property Tests

Derived projection never changes canonical deployment truth.

---

# 272. Scale Test

Hundreds of thousands of components/relations.

---

# 273. Graph Bomb Test

Traversal limits enforced.

---

# 274. Failure Injection

```text
search unavailable
deployment events delayed
directory unavailable
catalog worker crash
```

---

# 275. Implementation Phase 1 — Component/Ownership Model

Core catalog.

---

# 276. Phase 2 — Repository/Project Bindings

Source discovery.

---

# 277. Phase 3 — Deployment/Release Inventory

Portal value.

---

# 278. Phase 4 — Component Relations

Graph.

---

# 279. Phase 5 — Search/Portal UX

Discoverability.

---

# 280. Phase 6 — Scorecards

Standards/readiness.

---

# 281. Phase 7 — Golden Path Integration

Part 42.

---

# 282. Phase 8 — Notifications/Ownership Routing

Operations.

---

# 283. Phase 9 — External Catalog Import

Enterprise migration.

---

# 284. Phase 10 — API Contract Inventory

Developer platform.

---

# 285. Phase 11 — Cost/Security/Compliance Projections

Unified portal.

---

# 286. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 287. Acceptance Tests

1. Catalog component identity is stable across repository/project renames.
2. Catalog never becomes execution authority.
3. Deployment inventory is derived from Deployment subsystem.
4. Release inventory is derived from Release subsystem.
5. Ownership is explicit/provenance-aware.
6. Ownership metadata does not automatically grant permissions.
7. Ownership conflicts are surfaced rather than silently resolved.
8. External/imported ownership cannot bypass Forgeyard identity/authz.
9. Monorepo components can bind repository+path.
10. Source/build dependency relationships can project from Part 34.
11. Telemetry-observed relations remain marked Observed.
12. Inferred relations never masquerade as Exact.
13. Environment views use stable EnvironmentId.
14. “Current production release” is never manually authoritative in catalog.
15. Scorecards are advisory unless central policy explicitly consumes them.
16. Unknown scorecard evidence remains Unknown.
17. Golden-path adoption is derived from exact TemplateRef/policy state.
18. Repository descriptors cannot grant privileged capabilities.
19. Private components remain tenant/org isolated.
20. Search respects catalog authorization.
21. Team membership remains identity/directory authority.
22. Documentation links are safe/validated.
23. Missing ownership produces diagnostics but does not drop notifications silently.
24. Catalog field provenance is explainable.
25. Derived projections are rebuildable.
26. Catalog metadata updates use optimistic concurrency.
27. Retired-but-deployed components are detected.
28. External catalog imports preserve source/provenance.
29. Cost/security/compliance summaries remain projections of canonical subsystems.
30. Plugin portal panels cannot bypass authz or inject unsafe content.
31. Graph queries are bounded.
32. Standalone supports a simple local catalog.
33. Distributed mode supports organization-wide catalogs.
34. DR restores authoritative catalog metadata and rebuilds projections.
35. Forgeyard dogfoods the catalog to describe Forgeyard’s own services, agents, workers, UI, and release infrastructure.

---

# 288. Production Readiness Gates

Do not call catalog architecture production-ready until:

```text
component/ownership identities are stable
deployment/release projections are correct
ownership conflicts are visible
search authorization passes
tenant isolation passes
scorecard advisory-vs-policy separation is enforced
graph traversal is bounded
repository/monorepo bindings are reliable
DR/rebuild tests pass
portal performance scales
```

---

# 289. Architectural Invariants

1. catalog is organizational discovery, not execution truth;
2. canonical domain systems remain authoritative;
3. component identity is stable;
4. ownership is explicit/provenance-aware;
5. ownership metadata does not imply authz;
6. ownership conflicts are first-class;
7. deployment inventory is derived;
8. release inventory is derived;
9. current environment state is never manually authoritative;
10. repository descriptors are non-privileged metadata;
11. relations carry source/confidence;
12. observed/inferred relations never masquerade as exact;
13. scorecards are advisory unless policy consumes them;
14. unknown evidence remains unknown;
15. golden-path state is derived from exact template/policy identity;
16. private catalog data is tenant/org isolated;
17. search respects authorization;
18. team membership remains identity-system authority;
19. catalog fields expose provenance;
20. derived projections are rebuildable;
21. cost/security/compliance are projections only;
22. portal extensions obey authz/sanitization;
23. graph queries are bounded;
24. component lifecycle does not replace deployment/release state;
25. catalog import does not grant permissions;
26. missing owner never silently drops critical routing;
27. component renames preserve identity;
28. standalone/distributed share semantics;
29. DR restores authoritative metadata and rebuilds derived views;
30. Forgeyard dogfoods its own catalog.

---

# 290. Final Target Architecture

```text
                 Canonical Forgeyard Data
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       Projects       Releases     Deployments
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                  Catalog Projection
                        │
             ┌──────────┼──────────┐
             ▼          ▼          ▼
          Ownership   Relations  Environments
             │          │          │
             └──────────┼──────────┘
                        ▼
                  Developer Portal
```

---

# 291. Final Architectural Position

Component identity:

```text
SoftwareComponentId
+
repository/project bindings
+
ownership metadata
+
relationship metadata
  ↓
catalog entry
```

Environment inventory:

```text
Component
+
Deployment subsystem projections
  ↓
where it runs
+
which exact ReleaseId
+
health/drift
```

Scorecard:

```text
component
+
canonical subsystem evidence
+
scorecard definition
  ↓
Pass / Warning / Fail / Unknown / N/A
```

The key guarantee is:

> **Forgeyard can provide an organization-wide developer portal and software catalog without inventing a second truth layer. The catalog tells people what software exists, who is responsible for it, what depends on it, which standards it follows, and where it runs—but every operational fact remains grounded in the canonical Forgeyard subsystem that actually owns that truth.**

---

# 292. Extended Architecture Sequence

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
49 Service Catalog / Component Ownership / Environment Inventory / Developer Portal
```
