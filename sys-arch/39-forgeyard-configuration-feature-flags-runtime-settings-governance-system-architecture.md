# 39 — Forgeyard Configuration, Feature Flags, Runtime Settings & Dynamic Configuration Governance System Architecture

**Document type:** Core Configuration, Runtime Settings, Feature Flags, Dynamic Reload & Configuration Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** layered configuration, RON manifests, runtime settings, feature flags, staged rollout, kill switches, tenant/project overrides, dynamic reload, immutable configuration snapshots, schema validation, secret references, config provenance, drift detection, rollout safety, rollback, compatibility, audit, and policy integration  
**Architecture style:** Strongly typed, schema-versioned, layered, deterministic, immutable-snapshot based, auditable, policy-aware, secret-safe, rollbackable, and explicit about static vs dynamic configuration boundaries  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Core Config, Policy/Authz, Secrets, API/Axum, Dioxus UI, HA/Coordination, Operations/Upgrade/DR, Multi-Tenancy, Entitlements, Developer Experience, Plugins, Notifications, Audit, and Self-Hosting. This subsystem turns configuration into a governed first-class control-plane domain instead of scattered environment variables and ad-hoc files.

---

# 1. Purpose

Forgeyard contains many kinds of configuration:

```text
server ports
database settings
CAS backends
runner pools
sandbox defaults
policy bundles
notification routes
SCM integrations
feature availability
rollout toggles
limits
timeouts
retention
UI preferences
plugin enablement
deployment defaults
```

Without a dedicated architecture, configuration tends to become:

```text
environment variables
command-line flags
database rows
hidden defaults
feature booleans
provider-specific settings
```

spread across the codebase.

That creates drift, poor explainability, unsafe rollout, and hard-to-reproduce incidents.

The central rule is:

> **Forgeyard behavior is governed by validated, typed, versioned configuration snapshots. Runtime code consumes effective configuration, never an uncontrolled mixture of environment variables, raw database values, and provider settings.**

A second rule is:

> **Static configuration and dynamically reloadable configuration are different categories. Forgeyard must never pretend a setting can be changed safely at runtime when the subsystem requires restart or migration.**

A third rule is:

> **Feature flags are controlled rollout instruments, not a substitute for authorization, policy, entitlement, schema migration, or permanent product architecture.**

---

# 2. Architectural Position

```text
                 Configuration Sources
      ┌──────────────┼──────────────┐
      ▼              ▼              ▼
   Defaults       RON Files       DB Overrides
      │              │              │
      └──────────────┼──────────────┘
                     ▼
               Parse / Validate
                     │
                     ▼
               Layer Resolution
                     │
                     ▼
             Config Snapshot
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
      Runtime     Feature     Policy/
      Settings      Flags     Entitlement
          │          │          │
          └──────────┼──────────┘
                     ▼
              Application Services
```

---

# 3. Goals

The subsystem MUST:

1. define typed configuration;
2. define configuration identity;
3. support layered sources;
4. support RON config;
5. support environment-specific values;
6. support tenant/org/project overrides;
7. support runtime configuration;
8. distinguish static/dynamic settings;
9. support feature flags;
10. support staged rollout;
11. support kill switches;
12. support validation;
13. support schema versioning;
14. support config migration;
15. support immutable snapshots;
16. support rollback;
17. support config explainability;
18. support drift detection;
19. support secret references;
20. support safe reload;
21. support HA propagation;
22. support audit;
23. support notification;
24. support policy integration;
25. support entitlement integration;
26. support plugin config;
27. support CLI/UI administration;
28. support standalone mode;
29. support distributed mode;
30. remain deterministic and testable.

---

# 4. Non-Goals

This subsystem does not:

```text
replace policy
replace secrets management
replace entitlement
replace deployment configuration
replace application-level user preferences
permit arbitrary code execution from configuration
```

---

# 5. Workspace Structure

```text
crates/config/
├── forgeyard-config/
├── forgeyard-config-model/
├── forgeyard-config-schema/
├── forgeyard-config-loader/
├── forgeyard-config-layer/
├── forgeyard-config-validate/
├── forgeyard-config-snapshot/
├── forgeyard-config-runtime/
├── forgeyard-config-drift/
├── forgeyard-config-migrate/
├── forgeyard-config-health/
└── forgeyard-config-testkit/
```

Feature flags:

```text
crates/feature/
├── forgeyard-feature/
├── forgeyard-feature-model/
├── forgeyard-feature-evaluate/
├── forgeyard-feature-rollout/
├── forgeyard-feature-killswitch/
└── forgeyard-feature-testkit/
```

Use modules first; split only where dependency/security/runtime boundaries justify.

---

# 6. Configuration Domains

Separate:

```text
SystemConfig
TenantConfig
ProjectConfig
RunnerConfig
PluginConfig
UiConfig
RuntimeConfig
FeatureConfig
```

---

# 7. No Giant Global Config Struct

Critical.

Prefer composable typed domains.

---

# 8. Configuration Identity

```rust
pub struct ConfigSnapshotId(Digest);
```

Content-derived from canonical effective config.

---

# 9. Configuration Schema Version

```rust
pub struct ConfigSchemaVersion(u16);
```

---

# 10. Config Source

```rust
pub enum ConfigSource {
    BuiltInDefault,
    SystemFile,
    EnvironmentOverride,
    DatabaseOverride,
    TenantOverride,
    OrganizationOverride,
    ProjectOverride,
    RuntimeAdminOverride,
}
```

---

# 11. Source Precedence

Explicit.

Recommended baseline:

```text
built-in defaults
  ↓
system configuration
  ↓
deployment/environment configuration
  ↓
tenant/org/project configuration
  ↓
explicit runtime override
```

---

# 12. CLI Flags

Application startup override only for approved settings.

---

# 13. Environment Variables

Compatibility/bootstrap source only.

---

# 14. No Arbitrary Env Overlay

Critical.

Environment variables map only to explicitly declared config keys.

---

# 15. Why

Prevent hidden environment drift.

---

# 16. RON

Primary human-authored configuration format.

---

# 17. JSON

Public API transport only where appropriate.

---

# 18. Postcard

Internal snapshot/wire format where suitable.

---

# 19. Raw / Parsed / Validated

Reuse existing core model.

```text
RawConfig
  ↓ parse
ParsedConfig
  ↓ validate
ValidatedConfig
  ↓ layer
EffectiveConfig
```

---

# 20. Canonical Effective Config

Immutable snapshot.

---

# 21. ConfigSnapshot

```rust
pub struct ConfigSnapshot {
    pub id: ConfigSnapshotId,
    pub schema: ConfigSchemaVersion,
    pub created_at: Timestamp,
    pub source_refs: Vec<ConfigSourceRef>,
    pub effective: EffectiveConfig,
}
```

---

# 22. Config Source Ref

Safe metadata only.

---

# 23. Secret Values

Never included in ConfigSnapshot.

---

# 24. SecretRef

Configuration stores references.

---

# 25. Example

```ron
database: (
    url: Secret("system/database/url"),
)
```

---

# 26. Secret Resolution

Late, at consuming boundary.

---

# 27. Static Setting

Requires process/service restart.

---

# 28. Dynamic Setting

Can reload safely at runtime.

---

# 29. Reloadability

```rust
pub enum Reloadability {
    Static,
    RestartComponent(ComponentKind),
    Dynamic,
}
```

---

# 30. Every Config Field

Should declare reloadability metadata.

---

# 31. Why

Avoid unsafe hot reload assumptions.

---

# 32. ConfigFieldDescriptor

```rust
pub struct ConfigFieldDescriptor {
    pub path: ConfigFieldPath,
    pub reloadability: Reloadability,
    pub sensitivity: ConfigSensitivity,
    pub scope: ConfigScope,
}
```

---

# 33. Sensitivity

```rust
pub enum ConfigSensitivity {
    Public,
    Internal,
    Sensitive,
    SecretRefOnly,
}
```

---

# 34. Scope

```rust
pub enum ConfigScope {
    System,
    Tenant,
    Organization,
    Project,
    RunnerPool,
    Environment,
}
```

---

# 35. Not Every Field Is Overrideable at Every Scope

Critical.

---

# 36. Example

Tenant cannot override:

```text
system CA trust
database backend
cluster coordination
security baseline
```

---

# 37. Override Permission Matrix

Schema defines allowed child scopes.

---

# 38. Config Layer

```rust
pub struct ConfigLayer {
    pub source: ConfigSource,
    pub scope: ConfigScopeRef,
    pub values: ConfigPatch,
}
```

---

# 39. Config Patch

Typed partial structure.

---

# 40. No Generic JSON Merge Patch in Core

Strong typing preferred.

---

# 41. Layer Resolution

Deterministic.

---

# 42. Config Explain

For any field:

```text
effective value
source layer
overridden layers
reloadability
```

---

# 43. CLI

```text
forgeyard config explain <path>
```

---

# 44. Redaction

Sensitive fields show refs/status, not values.

---

# 45. Configuration Validation

Multiple levels:

```text
syntax
schema
cross-field
capability
policy
environment
```

---

# 46. Syntax

RON parser.

---

# 47. Schema

Type validation.

---

# 48. Cross-Field

Example:

```text
HA enabled but one voter configured
```

invalid/warn.

---

# 49. Capability

Example:

```text
sandbox.seccomp=true
```

on unsupported host.

---

# 50. Policy

Example:

```text
external plugin network enabled
```

may violate policy.

---

# 51. Environment

Example:

```text
object store bucket unreachable
```

runtime preflight, not pure schema validation.

---

# 52. Validation Result

```rust
pub struct ConfigValidationResult {
    pub errors: Vec<ConfigDiagnostic>,
    pub warnings: Vec<ConfigDiagnostic>,
}
```

---

# 53. Diagnostics

Span-aware where human file source exists.

---

# 54. Config Change

```rust
pub struct ConfigChange {
    pub base: ConfigSnapshotId,
    pub patch: ConfigPatch,
    pub actor: PrincipalId,
}
```

---

# 55. Validate Before Activate

Critical.

---

# 56. Config Activation

```text
propose
  ↓
parse/validate
  ↓
policy/authz
  ↓
create immutable snapshot
  ↓
activate pointer
```

---

# 57. ActiveConfigRef

Mutable pointer to immutable snapshot.

---

# 58. Rollback

Point active ref to previous compatible snapshot.

---

# 59. No In-Place Snapshot Mutation

Critical.

---

# 60. Config History

Persist snapshots + activation events.

---

# 61. Rollback Safety

Static fields may still require restart.

---

# 62. Rollback Plan

```rust
pub struct ConfigRollbackPlan {
    pub target: ConfigSnapshotId,
    pub restart_components: Vec<ComponentKind>,
    pub compatibility: ConfigCompatibility,
}
```

---

# 63. ConfigCompatibility

```rust
pub enum ConfigCompatibility {
    SafeDynamic,
    RequiresRestart,
    RequiresMigration,
    Unsafe,
}
```

---

# 64. Migration

Some config changes may require data/schema changes.

---

# 65. Config Does Not Perform DB Migration

Coordinates with migration subsystem.

---

# 66. Feature Flags

Separate from general config.

---

# 67. FeatureFlagId

```rust
pub struct FeatureFlagId(BoundedString);
```

---

# 68. Feature Flag

```rust
pub struct FeatureFlag {
    pub id: FeatureFlagId,
    pub state: FeatureFlagState,
    pub rollout: FeatureRollout,
    pub expires_at: Option<Timestamp>,
}
```

---

# 69. Feature Flag State

```rust
pub enum FeatureFlagState {
    Disabled,
    Enabled,
    Rollout,
    Killed,
}
```

---

# 70. Killed

Emergency hard-off.

---

# 71. Flag vs Entitlement

Important:

```text
Feature Flag:
is functionality operationally enabled?

Entitlement:
is customer commercially allowed?

Authz:
may this principal use it?
```

---

# 72. Effective Feature Availability

```text
implemented
AND
flag enabled
AND
entitled if commercial
AND
authorized
AND
policy permits
```

---

# 73. Feature Flag Is Not Security Boundary Alone

Critical.

---

# 74. Feature Rollout

```rust
pub enum FeatureRollout {
    All,
    Percentage(RolloutPercentage),
    Tenants(BTreeSet<TenantId>),
    Projects(BTreeSet<ProjectId>),
    Principals(BTreeSet<PrincipalId>),
    Cohort(FeatureCohortId),
}
```

---

# 75. Percentage Rollout

Deterministic.

---

# 76. Stable Bucketing

Hash:

```text
flag ID
subject stable ID
rollout seed
```

---

# 77. No Random Per Request

Critical.

---

# 78. Rollout Seed

Versioned.

---

# 79. Percentage Meaning

Same subject consistently in/out.

---

# 80. Cohort

Explicit stored membership or deterministic rule.

---

# 81. Avoid Sensitive Profiling

Feature cohorts based on product/admin criteria, not opaque personal profiling.

---

# 82. Flag Scope

System/tenant/project.

---

# 83. Child Override

Only if schema permits.

---

# 84. Kill Switch

Emergency disable path.

---

# 85. KillSwitchId

```rust
pub struct KillSwitchId(BoundedString);
```

---

# 86. Use Cases

```text
disable compromised plugin type
disable new release publisher
disable faulty scheduler optimization
disable remote cache writes
disable external dependency resolution
```

---

# 87. Kill Switch Characteristics

```text
fast
auditable
safe default
limited scope
```

---

# 88. Kill Switch Should Not Require Code Deploy

---

# 89. Kill Switch Does Not Replace Incident Response

It is containment tool.

---

# 90. Kill Switch Evaluation

Prefer local in-memory snapshot for fast path.

---

# 91. Dynamic Reload

Config snapshot distributed to services.

---

# 92. Config Distribution

Mode 2:

```text
Postgres authoritative snapshot
  ↓
config-change event
  ↓
daemon/service reload
  ↓
ack status
```

---

# 93. Event Fast Path

Reconciliation slow path.

---

# 94. No Exactly-Once Reload

Idempotent snapshot application.

---

# 95. ComponentConfigGeneration

```rust
pub struct ComponentConfigGeneration(u64);
```

---

# 96. Each Component

Tracks applied ConfigSnapshotId.

---

# 97. Configuration Drift

```text
active snapshot = A
component reports B
```

drift.

---

# 98. Drift State

```rust
pub enum ConfigDriftState {
    InSync,
    Applying,
    Drifted,
    Unknown,
}
```

---

# 99. Drift Reconciler

Reapply or request restart.

---

# 100. Dynamic Reload Failure

Do not claim success.

---

# 101. Config Activation State

```rust
pub enum ConfigActivationState {
    Proposed,
    Validated,
    Activating,
    Active,
    PartiallyApplied,
    Failed,
    RolledBack,
}
```

---

# 102. PartiallyApplied

First-class.

---

# 103. Protected Config

Can require all critical components applied before activation considered complete.

---

# 104. Rollout Strategy

Config changes can be staged.

---

# 105. ConfigRollout

```rust
pub enum ConfigRollout {
    Immediate,
    Rolling,
    Canary(Vec<ComponentInstanceId>),
}
```

---

# 106. Static Restart Rollout

Integrates HA rolling upgrade semantics.

---

# 107. Example

Change runner-agent heartbeat:

```text
dynamic
```

Change QUIC bind port:

```text
restart component
```

Change DB backend:

```text
migration/redeploy
```

---

# 108. No Universal Hot Reload

Critical.

---

# 109. Validation Preflight

For external resources:

```text
test connection
check credentials
verify permissions
```

---

# 110. Preflight Is Side-Effect Safe

Avoid mutation.

---

# 111. Config Plan

```rust
pub struct ConfigChangePlan {
    pub base: ConfigSnapshotId,
    pub candidate: ConfigSnapshotId,
    pub diff: ConfigDiff,
    pub required_actions: Vec<ConfigApplyAction>,
}
```

---

# 112. Apply Action

```text
DynamicReload
RestartDaemon
RestartAgent
RunMigration
ReconfigureProvider
```

---

# 113. Config Diff

Typed.

---

# 114. UI/CLI

Show safe before/after.

---

# 115. SecretRef Diff

Shows ref changed, never secret.

---

# 116. Config Approval

High-risk config may require approval.

---

# 117. Examples

```text
trust root
external network
signing provider
production deployment provider
cross-tenant cache
```

---

# 118. Approval

Reuse policy/human workflow.

---

# 119. Config Policy Digest

Activation records PolicyDigest.

---

# 120. Audit

Every protected config activation audited.

---

# 121. Audit Fields

```text
actor
base snapshot
new snapshot
scope
diff digest
reason
result
```

---

# 122. Runtime Overrides

Use sparingly.

---

# 123. RuntimeOverrideId

```rust
pub struct RuntimeOverrideId(Ulid);
```

---

# 124. Override

```rust
pub struct RuntimeOverride {
    pub id: RuntimeOverrideId,
    pub scope: ConfigScopeRef,
    pub patch: ConfigPatch,
    pub reason: BoundedString,
    pub expires_at: Option<Timestamp>,
}
```

---

# 125. Expiry

Strongly recommended.

---

# 126. Temporary Incident Override

Good use.

---

# 127. Permanent Config

Move into normal config source.

---

# 128. Override Expiry

Durable timer.

---

# 129. Expired Override

Triggers new effective snapshot.

---

# 130. No Hidden "Forever" Emergency Override

Critical.

---

# 131. Environment Variables

Startup bootstrap examples:

```text
FORGEYARD_CONFIG
FORGEYARD_DATA_DIR
FORGEYARD_LOG_LEVEL
```

---

# 132. Sensitive Env

Avoid.

---

# 133. Secret Providers

Configured via SecretRef/bootstrap mechanism.

---

# 134. Bootstrap Secret Problem

Some initial credentials may need OS secret/keyring/file permission protected mechanism.

---

# 135. Bootstrap Config

Small and explicitly documented.

---

# 136. Config Bootstrap Layer

Cannot depend on DB if DB connection itself is configured there.

---

# 137. Bootstrap vs Runtime Config

Separate.

---

# 138. BootstrapConfig

```rust
pub struct BootstrapConfig {
    pub mode: ForgeyardMode,
    pub data_dir: PathBuf,
    pub metadata_backend: MetadataBootstrap,
    pub initial_config_source: ConfigSourceLocation,
}
```

---

# 139. Minimal

Only enough to start config system.

---

# 140. Standalone Mode

System file/local DB.

---

# 141. Distributed Mode

Bootstrap DB/CAS/trust endpoints, then authoritative runtime config from metadata store.

---

# 142. Configuration Store

Part 2 metadata.

---

# 143. Snapshot Bytes

Can be CAS-backed.

---

# 144. Active Pointer

DB.

---

# 145. Tenant Config

Scoped rows/snapshots.

---

# 146. Effective Tenant Config

Base system snapshot + tenant layer.

---

# 147. Project Config

Further layer.

---

# 148. Resolution Cost

Cache effective snapshots.

---

# 149. EffectiveConfigCache

Key:

```text
system snapshot
tenant layer version
org layer version
project layer version
```

---

# 150. Not Build Cache

Separate internal config cache.

---

# 151. Cache Invalidation

Version-based.

---

# 152. Configuration Access

Services receive typed domain config.

---

# 153. No `get("foo.bar")` Throughout Domain

Critical.

---

# 154. Adapter Config

Provider-specific details stay in adapter crate.

---

# 155. Example

S3 config types in CAS S3 adapter.

---

# 156. Core Config

Only normalized capability-level fields.

---

# 157. Plugin Config

Plugin manifest declares schema.

---

# 158. Plugin Config Isolation

Plugin receives only its namespace.

---

# 159. Plugin Cannot Read Full system config.

---

# 160. Plugin Secret Fields

SecretRef/host-mediated.

---

# 161. Plugin Config Schema

Versioned with plugin version/API.

---

# 162. Unknown Plugin Field

Validation error/warning per schema policy.

---

# 163. Feature Flag Lifecycle

Flags should not live forever.

---

# 164. FeatureFlagMetadata

```rust
pub struct FeatureFlagMetadata {
    pub owner: PrincipalOrTeamRef,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub cleanup_issue: Option<ExternalTrackingRef>,
}
```

---

# 165. Expiry

For rollout flags.

---

# 166. Permanent Capability Switch

May become normal config instead.

---

# 167. Flag Debt

Analytics can report old flags.

---

# 168. Kill Switch

May be permanent operational control.

---

# 169. Flag Evaluation

```rust
pub trait FeatureEvaluator {
    fn evaluate(
        &self,
        flag: FeatureFlagId,
        context: FeatureContext,
    ) -> FeatureDecision;
}
```

---

# 170. FeatureDecision

```rust
pub enum FeatureDecision {
    Enabled(FeatureDecisionReason),
    Disabled(FeatureDecisionReason),
    Killed,
}
```

---

# 171. Decision Reason

Explainable.

---

# 172. UI

Can show admin why subject sees feature.

---

# 173. No Secret Feature Flag Data

---

# 174. Feature Flag Performance

In-memory immutable snapshot.

---

# 175. No DB Query Per Request

Critical.

---

# 176. Snapshot Refresh

Event/reconcile.

---

# 177. Flag Change Propagation

Seconds-level expected; not instantaneous global linearizability unless explicitly built.

---

# 178. Kill Switch Critical Path

Can use coordination epoch/fast invalidation.

---

# 179. High-Risk Kill Switch

May require immediate control-plane broadcast plus local poll fallback.

---

# 180. HA

Any daemon can read authoritative snapshot.

---

# 181. Active Snapshot

DB authority.

---

# 182. Raft

Not needed for ordinary config.

---

# 183. Raft Use

Only if specific cluster-coordination setting must be serialized with leadership/membership.

---

# 184. No Config in Raft Log by Default

Critical.

---

# 185. Configuration Consistency Model

Eventual component application with explicit desired vs observed snapshot.

---

# 186. Desired

Active ConfigSnapshotId.

---

# 187. Observed

Component reports applied ID.

---

# 188. Control Plane

Can block sensitive workflow if critical component drifted.

---

# 189. Example

Signing configuration changed but signing worker not updated.

Release signing may pause.

---

# 190. Policy

Can define required config convergence.

---

# 191. Config Health

```text
active snapshot
applied components
drifted components
failed reload
expired overrides
expired flags
```

---

# 192. Doctor

```text
forgeyard config doctor
```

---

# 193. Doctor Checks

```text
schema compatibility
active snapshot integrity
component drift
invalid overrides
feature flag expiry
secret ref resolution readiness
```

---

# 194. Config Lint

```text
forgeyard config check
```

---

# 195. Config Plan

```text
forgeyard config plan
```

---

# 196. Config Apply

```text
forgeyard config apply
```

---

# 197. Config History

```text
forgeyard config history
```

---

# 198. Config Rollback

```text
forgeyard config rollback <snapshot>
```

---

# 199. Feature CLI

```text
forgeyard feature list
forgeyard feature explain <flag>
forgeyard feature rollout <flag>
forgeyard feature kill <flag>
```

---

# 200. High-Risk CLI

Requires explicit confirmation/authz.

---

# 201. JSON/RON Output

Supported.

---

# 202. Dioxus UI

Pages:

```text
Configuration
Effective Config
Config History
Runtime Overrides
Feature Flags
Kill Switches
Config Drift
```

---

# 203. Config Editor

Typed forms.

---

# 204. Raw RON Editor

Advanced mode only.

---

# 205. Validation Before Save

---

# 206. Diff View

Before activation.

---

# 207. Sensitive Fields

Masked/ref-only.

---

# 208. Feature Rollout UI

Shows:

```text
state
cohort
percentage
owner
expiry
```

---

# 209. Kill Switch UI

Prominent, restricted.

---

# 210. Configuration Approval UI

Reuse human workflow.

---

# 211. API

Potential:

```text
GET  /v1/config/effective
GET  /v1/config/history
POST /v1/config/validate
POST /v1/config/plan
POST /v1/config/apply
POST /v1/config/rollback
GET  /v1/features
POST /v1/features/{id}/rollout
POST /v1/features/{id}/kill
```

---

# 212. Permissions

```text
config.read
config.manage
config.apply
config.rollback
config.runtime_override
feature.read
feature.manage
feature.kill
```

---

# 213. Scope-Aware Permissions

Project admin cannot edit system config.

---

# 214. Config Read

Sensitive fields redacted.

---

# 215. Snapshot Export

Safe sanitized export.

---

# 216. Full Internal Snapshot

Higher permission.

---

# 217. No Secret Values Even Full Snapshot

Secret refs only.

---

# 218. Configuration Drift Types

```rust
pub enum ConfigDriftKind {
    SnapshotMismatch,
    RestartPending,
    UnsupportedField,
    FailedReload,
    ExternalProviderDrift,
}
```

---

# 219. External Provider Drift

Example:

```text
configured bucket policy vs actual
```

if adapter can inspect.

---

# 220. External Drift

Diagnostic only unless policy.

---

# 221. Config Reconciler

Checks desired/observed.

---

# 222. Retry

Dynamic reload idempotently.

---

# 223. Restart Required

Creates operational task/status, not automatic unsafe restart unless rollout policy.

---

# 224. Static Config Apply

Mode 2 rolling restart.

---

# 225. Standalone Static Change

Prompt restart/app restart.

---

# 226. Runtime Override Reconciliation

Expire and recompute.

---

# 227. Feature Flag Reconciliation

Expired rollout disables/defaults per declared policy.

---

# 228. Flag Expiry Behavior

Explicit:

```rust
pub enum FlagExpiryBehavior {
    Disable,
    Enable,
    KeepCurrentAndWarn,
}
```

---

# 229. Recommended

Rollout flag -> Disable or stable target state explicitly.

---

# 230. No Ambiguous Expiry

---

# 231. Config Drift Notification

Notify admins for critical drift.

---

# 232. Kill Switch Notification

Immediate for security/production.

---

# 233. Audit Integration

Critical.

---

# 234. Search

Part 31 may index safe config metadata/history, not values.

---

# 235. Analytics

Examples:

```text
flag age
override age
drift incidents
failed reload
```

---

# 236. No Tenant Sensitive Values in analytics.

---

# 237. Observability Metrics

```text
config_activation_total
config_activation_failures_total
config_drift_components
config_reload_failures_total
feature_flag_evaluations_total
feature_killswitch_active
runtime_overrides_active
```

---

# 238. Labels

Low-cardinality:

```text
component
result
scope_kind
```

---

# 239. No flag ID metrics if huge cardinality; analytics instead.

---

# 240. Tracing

```text
config.load
config.validate
config.resolve
config.activate
config.reload
feature.evaluate
feature.rollout
```

---

# 241. Config Storage Security

Config metadata may contain internal infrastructure names.

---

# 242. At-Rest Encryption

Deployment-level.

---

# 243. Config Export

Permission gated.

---

# 244. Source-Controlled Config

Repository `.forgeyard/*.ron`.

---

# 245. Runtime System Config

Metadata DB.

---

# 246. Separation

Repository cannot overwrite system operational config.

---

# 247. Repository Config Trust

Untrusted project source.

---

# 248. Pipeline Config

Parsed/validated as source input.

---

# 249. System Config

Trusted admin plane.

---

# 250. Never Let Repository Config Enable Privileged Host Capability

Critical.

---

# 251. Example

Project may request:

```text
GPU
network access
```

but cannot grant itself.

Policy/scheduler decide.

---

# 252. Config Request vs Config Grant

Separate.

---

# 253. Runner Config

Agent reports capabilities; admin config controls assigned trust/pool.

---

# 254. Runner Local Config

Bootstrap endpoints/certs.

---

# 255. Runner Cannot Self-Set Trust Class

Existing invariant.

---

# 256. Configuration as Evidence

Critical workflows can record ConfigSnapshotId.

---

# 257. Run

Store relevant config snapshot/reference.

---

# 258. Release

Store policy/config snapshots where semantics affect release.

---

# 259. Deployment

Runtime config exact revision/digest already required.

---

# 260. Cache

Config semantics version/key relevant where build behavior changes.

---

# 261. Reproducibility

Human config resolved to exact snapshot.

---

# 262. Config Change and Cache

Changing correctness-relevant config invalidates derivation/cache through ConfigDigest.

---

# 263. ConfigDigest

```rust
pub struct ConfigDigest(Digest);
```

---

# 264. Only Relevant Projection

Job cache key includes relevant config digest, not entire global config.

---

# 265. Avoid Unnecessary Cache Busting

Critical.

---

# 266. Config Dependency Declaration

Job/service declares config paths affecting semantics.

---

# 267. Unknown Config Dependency

Conservative: broader config digest.

---

# 268. Plugin Config Changes

Can invalidate plugin behavior/evidence.

---

# 269. Feature Flag and Cache

Runtime feature flags should generally not affect build cache unless explicitly part of build semantics.

---

# 270. Build Feature Flag

If it changes build output, it must enter derivation key.

---

# 271. UI-only Flag

Must not bust build cache.

---

# 272. Config Migration

Version N -> N+1.

---

# 273. Migration Function

Pure where possible.

---

# 274. Migration Output

New validated config snapshot.

---

# 275. No Destructive Silent Migration

---

# 276. Deprecated Fields

Warnings with replacement.

---

# 277. Removal

Major schema version.

---

# 278. N/N-1 Compatibility

For rolling upgrades.

---

# 279. Older Component

May not understand new field.

---

# 280. Activation Precheck

Ensure all required component versions understand snapshot.

---

# 281. Feature Activation After Upgrade

Same pattern as protocol feature activation.

---

# 282. Config Capability Matrix

```rust
pub struct ConfigCapabilityMatrix {
    pub component: ComponentKind,
    pub supported_schema: VersionRange,
    pub supported_fields: BTreeSet<ConfigFieldPath>,
}
```

---

# 283. Rolling Upgrade Safety

Do not activate unsupported config mid-rollout.

---

# 284. Config Rollback After Upgrade

Only if compatible.

---

# 285. DR

Config snapshots/history backed up.

---

# 286. Bootstrap Config

Backed up separately.

---

# 287. Secret Values

Still in secret provider.

---

# 288. Restore

Restore config snapshots then active pointer.

---

# 289. Drift After Restore

Reconcile components.

---

# 290. Air-Gap

RON config + signed/verified snapshot export.

---

# 291. Config Bundle

```rust
pub struct ConfigBundle {
    pub snapshot: ConfigSnapshotId,
    pub schema: ConfigSchemaVersion,
    pub manifest: CasObjectRef,
    pub signature: Option<SignatureRef>,
}
```

---

# 292. Import

Validate schema/policy before activate.

---

# 293. Signing

Optional high-assurance.

---

# 294. Testkit

```text
forgeyard-config-testkit/src/
├── lib.rs
├── layers.rs
├── validate.rs
├── snapshot.rs
├── reload.rs
├── drift.rs
├── migration.rs
└── assertions.rs
```

Feature:

```text
forgeyard-feature-testkit/
```

---

# 295. Unit Tests

Layer precedence.

---

# 296. Protected Field Test

Tenant cannot override system field.

---

# 297. Secret Test

Secret value never appears in snapshot/export.

---

# 298. Dynamic Reload Test

Supported field updates without restart.

---

# 299. Static Field Test

Does not falsely report dynamic success.

---

# 300. Partial Apply Test

State becomes PartiallyApplied.

---

# 301. Drift Test

Component mismatch detected.

---

# 302. Reconcile Test

Dynamic component catches up.

---

# 303. Rollback Test

Restores previous immutable snapshot.

---

# 304. Migration Test

N->N+1 deterministic.

---

# 305. Old Component Compatibility Test

Unsupported field blocks activation.

---

# 306. Percentage Rollout Test

Stable subject bucketing.

---

# 307. Kill Switch Test

Immediate local evaluation after propagation.

---

# 308. Flag Expiry Test

Declared expiry behavior.

---

# 309. Runtime Override Expiry Test

Effective config recomputed.

---

# 310. Cache Config Digest Test

Only relevant config changes bust cache.

---

# 311. Repository Privilege Test

Project config cannot grant privileged host capability.

---

# 312. Tenant Isolation Test

Tenant config cannot affect another tenant.

---

# 313. Audit Test

Protected activation recorded.

---

# 314. DR Test

History/active snapshot restored.

---

# 315. Fuzzing

Fuzz:

```text
RON parser boundary
config patch parser
migration decoders
feature flag evaluator inputs
```

---

# 316. Property Tests

Layer resolution deterministic.

---

# 317. Failure Injection

```text
DB restart
reload worker crash
component unavailable
invalid secret ref
partial rollout
```

---

# 318. Scale Test

Many tenants/project overrides.

---

# 319. Implementation Phase 1 — Typed Config Domains

System/runtime/tenant/project.

---

# 320. Phase 2 — Layer Resolution/Explain

Core usability.

---

# 321. Phase 3 — Immutable Snapshots/History

Governance.

---

# 322. Phase 4 — Dynamic Reload/Drift

Runtime.

---

# 323. Phase 5 — Feature Flags

Staged rollout.

---

# 324. Phase 6 — Kill Switches

Incident control.

---

# 325. Phase 7 — Config Plan/Rollback

Operations.

---

# 326. Phase 8 — Tenant/Project Overrides

Multi-tenancy.

---

# 327. Phase 9 — Plugin Config Schemas

Extensions.

---

# 328. Phase 10 — Upgrade Compatibility Matrix

HA.

---

# 329. Phase 11 — Signed/Air-Gap Config Bundles

Enterprise.

---

# 330. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 331. Acceptance Tests

1. Forgeyard uses typed config domains rather than one giant untyped map.
2. RON is primary human-authored config format.
3. Environment variables map only to explicitly supported keys.
4. Effective config is represented by immutable ConfigSnapshotId.
5. Secret plaintext never appears in config snapshots.
6. Each field declares scope/reloadability/sensitivity.
7. Tenant/project layers cannot override forbidden system fields.
8. Layer resolution is deterministic.
9. Config explain shows effective source without leaking secrets.
10. Candidate config is validated before activation.
11. Active snapshot pointer changes atomically.
12. Historical snapshots are never mutated.
13. Dynamic settings reload idempotently.
14. Static settings do not pretend to hot reload.
15. Partial application is explicit.
16. Component desired/observed snapshot drift is measurable.
17. Reconciliation repairs dynamic config drift.
18. Feature flags are not authorization or entitlement.
19. Percentage rollout uses deterministic stable bucketing.
20. Kill switches can disable risky functionality without code deploy.
21. Kill-switch changes are audited/authorized.
22. Temporary runtime overrides can expire automatically.
23. Repository config cannot grant itself privileged capabilities.
24. Build-affecting config enters derivation/cache semantics.
25. UI-only config does not unnecessarily invalidate build cache.
26. Config schema migration is versioned/tested.
27. Rolling upgrade prechecks component config compatibility.
28. Unsupported config is not activated during mixed-version rollout.
29. Config history/active snapshot survive backup/restore.
30. Air-gap config bundles are validated before activation.
31. Plugin receives only its config namespace.
32. Tenant config is isolated.
33. Standalone/distributed share config semantics.
34. CLI/UI expose plan/diff/history/rollback.
35. Forgeyard dogfoods this configuration system for its own runtime.

---

# 332. Production Readiness Gates

Do not call configuration governance production-ready until:

```text
typed schema/layering is stable
secret leakage tests pass
protected scope override rules pass
snapshot/history/rollback work
dynamic/static reload distinction is enforced
component drift detection works
feature rollout bucketing is deterministic
kill switch path is tested
mixed-version config compatibility is validated
backup/restore and air-gap config tests pass
```

---

# 333. Architectural Invariants

1. effective behavior comes from typed validated config;
2. config snapshots are immutable;
3. active pointer is mutable, snapshot content is not;
4. secrets are references only;
5. layer precedence is deterministic;
6. override scope is explicitly constrained;
7. environment variables are allowlisted inputs, not universal config;
8. static and dynamic settings are distinct;
9. partial reload is never called fully applied;
10. desired/observed config drift is first-class;
11. feature flags are rollout controls, not security authority;
12. entitlement/authz/policy remain separate;
13. percentage rollout is stable/deterministic;
14. kill switches are fast, scoped, authorized, and audited;
15. temporary overrides should expire;
16. repository config cannot self-grant privilege;
17. config compatibility is checked during rolling upgrades;
18. build-affecting config participates in derivation/cache identity;
19. irrelevant config does not unnecessarily bust cache;
20. config migration is versioned;
21. plugin config is namespaced;
22. secret fields never appear in exports;
23. rollback points to previous immutable snapshot;
24. config restore reconciles component drift;
25. ordinary config does not live in Raft;
26. dynamic propagation is idempotent/reconciled;
27. standalone/distributed share semantics;
28. config history is auditable;
29. high-risk config can require approval;
30. Forgeyard dogfoods its own configuration governance.

---

# 334. Final Target Architecture

```text
                   Config Sources
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         Defaults       RON       DB Overrides
            │            │            │
            └────────────┼────────────┘
                         ▼
                 Parse / Validate
                         │
                         ▼
                 Layer Resolution
                         │
                         ▼
                 ConfigSnapshotId
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
       Dynamic Config Feature Flags Static Config
            │            │            │
            ▼            ▼            ▼
         Reload       Evaluate      Restart Plan
            │            │            │
            └────────────┼────────────┘
                         ▼
                  Desired State
                         │
                         ▼
                 Drift/Reconcile
```

---

# 335. Final Architectural Position

Configuration change:

```text
base snapshot
+
typed patch
  ↓
validation
  ↓
policy/authz
  ↓
immutable candidate snapshot
  ↓
apply plan
  ↓
dynamic reload / restart / migration
  ↓
observed component convergence
```

Feature rollout:

```text
FeatureFlagId
+
stable subject ID
+
rollout rule
+
seed
  ↓
deterministic decision
  ↓
entitlement/authz/policy still apply
```

Kill switch:

```text
risk detected
  ↓
authorized kill-switch activation
  ↓
new immutable config/feature snapshot
  ↓
fast propagation
  ↓
reconciliation + audit
```

The key guarantee is:

> **Forgeyard can change runtime behavior safely without making production configuration mysterious or mutable in place. Every effective setting can be explained, every protected change has a versioned snapshot and audit trail, dynamic reload is used only where genuinely safe, and feature flags never replace the permanent security and policy architecture.**

---

# 336. Extended Architecture Sequence

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
```
