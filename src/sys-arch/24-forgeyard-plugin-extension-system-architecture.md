# 24 — Forgeyard Plugin & Extension System Architecture

**Document type:** Core Extensibility, Plugin & Capability Extension System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Plugin manifests, extension points, capability declarations, trust tiers, sandboxed execution, compatibility/versioning, provider adapters, executor/package/deploy integrations, notification extensions, policy-fact providers, doctor checks, UI extensions, lifecycle/install/update/remove, observability, and extension security boundaries  
**Architecture style:** Extension-by-capability, least privilege, sandbox-first for third-party code, typed host contracts, explicit trust levels, stable manifests, no direct database access, no bypass of policy/authz/scheduler/secret boundaries, and first-party preference for correctness-critical integrations  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on Core Domain, API/Axum, Dioxus UI, Policy/Authz/Identity, Secrets/Trust, Scheduler, Runner, Sandbox/Executor, Packaging, Release, Deployment, Observability/Doctor, SCM integrations, and RBE. It provides extension surfaces without weakening any of those invariants.

---

# 1. Purpose

Forgeyard needs to evolve without turning its core workspace into an ever-growing monolith.

Future users may want to add:

```text
new SCM provider
new package format
new deployment target
new notification provider
new artifact publisher
new vulnerability scanner
new license scanner
new executor
new doctor check
new UI panel
new policy fact source
new device provider
new secret provider
new cache/CAS backend
```

But extensibility is dangerous if plugins can:

```text
open the database directly
read arbitrary secrets
run arbitrary in-process unsafe code
grant themselves permissions
schedule jobs
mark releases successful
rewrite CAS objects
```

The central rule is:

> **Plugins may add capabilities only through explicit typed host contracts. They never gain authority merely because they are installed.**

A second rule is:

> **Correctness-critical and security-critical extension points default to first-party Rust crates or isolated trusted processes, not arbitrary third-party in-process dynamic libraries.**

A third rule is:

> **Third-party extensions are sandboxed and capability-scoped. The host owns identity, policy, secrets, storage, scheduling, and lifecycle.**

---

# 2. Architectural Position

```text
                    Forgeyard Core
                         │
                         ▼
                 Extension Registry
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
          Built-In    Trusted      Sandboxed
          Adapter     Plugin       Plugin
             │           │           │
             └───────────┼───────────┘
                         ▼
                 Typed Host Contracts
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
           SCM        Package      Deploy
         Scanner      Notify       UI/Doctor
```

---

# 3. Goals

The subsystem MUST:

1. define plugin identity;
2. define extension-point identity;
3. define manifests;
4. define compatibility/version ranges;
5. define capability declarations;
6. define trust tiers;
7. support built-in extensions;
8. support trusted external extensions;
9. support sandboxed external extensions;
10. prevent direct DB access;
11. prevent direct unrestricted CAS access;
12. prevent arbitrary secret reads;
13. prevent authz bypass;
14. prevent scheduler bypass;
15. prevent release/deployment state forgery;
16. support install;
17. support update;
18. support disable;
19. support uninstall;
20. support health/doctor;
21. support observability;
22. support resource quotas;
23. support crash isolation;
24. support timeouts;
25. support extension-specific configuration;
26. support provider credentials by `SecretRef`;
27. support UI extension descriptors;
28. support first-party static registration;
29. support external-process/IPC plugins;
30. remain evolvable without unstable Rust ABI coupling.

---

# 4. Non-Goals

The plugin system is not:

```text
an arbitrary code execution marketplace
a way to load random `.so`/`.dll` into forgeyard-daemon
a replacement for Cargo workspace crates
a bypass around policy
a scripting engine for all Forgeyard logic
```

---

# 5. Extension Categories

Primary extension categories:

```rust
pub enum ExtensionKind {
    ScmProvider,
    VcsAdapter,
    SecretProvider,
    CasBackend,
    Executor,
    PackageAdapter,
    DeploymentProvider,
    ArtifactPublisher,
    VulnerabilityScanner,
    LicenseScanner,
    NotificationProvider,
    DeviceProvider,
    DoctorCheck,
    PolicyFactProvider,
    UiExtension,
    Custom(ExtensionKindId),
}
```

---

# 6. First-Party vs Third-Party

First-party extension:

```text
compiled in workspace
reviewed with Forgeyard
same release train
```

Third-party extension:

```text
separately shipped
versioned independently
sandboxed by default
```

---

# 7. PluginId

```rust
pub struct PluginId(BoundedString);
```

Canonical namespaced ID.

Example:

```text
com.example.forgeyard.deploy.nomad
```

---

# 8. Plugin Version

```rust
pub struct PluginVersion(SemVer);
```

---

# 9. Plugin Manifest

Human-readable:

```text
plugin.ron
```

---

# 10. Manifest Model

```rust
pub struct PluginManifest {
    pub id: PluginId,
    pub version: PluginVersion,
    pub name: BoundedString,
    pub vendor: BoundedString,
    pub api: PluginApiRequirement,
    pub extensions: Vec<ExtensionDeclaration>,
    pub permissions: Vec<PluginPermissionRequest>,
    pub resources: PluginResourceLimits,
    pub entrypoint: PluginEntrypoint,
}
```

---

# 11. Manifest Signature

Optional for dev.

Required for trusted marketplace/enterprise policy if configured.

---

# 12. Manifest Digest

```rust
pub struct PluginManifestDigest(Digest);
```

---

# 13. Installed Plugin Identity

```rust
pub struct InstalledPluginId(Ulid);
```

Separate installation identity.

---

# 14. Plugin API Version

```rust
pub struct PluginApiVersion {
    pub major: u16,
    pub minor: u16,
}
```

---

# 15. Compatibility

Plugin declares:

```text
min Forgeyard plugin API
max supported major
```

---

# 16. Breaking Plugin API

New major.

---

# 17. No Rust ABI Promise

Critical.

Do not expose unstable Rust trait-object ABI across independently compiled dynamic libraries.

---

# 18. Recommended External Plugin Boundary

```text
process isolation
+
versioned IPC
```

---

# 19. IPC Encoding

Preferred:

```text
Postcard for Forgeyard-native local IPC
```

with explicit schema version.

JSON optional for language-neutral plugin SDK later.

---

# 20. Transport

Possible:

```text
Unix domain socket
Windows named pipe
loopback QUIC
stdio framed protocol
```

---

# 21. Recommended Initial External Transport

Local socket/named pipe with framed Postcard protocol.

---

# 22. External Plugin Process

Started/managed by Forgeyard plugin supervisor.

---

# 23. Plugin Supervisor

Responsibilities:

```text
start
stop
health
restart
resource limits
IPC handshake
version negotiation
permission enforcement
```

---

# 24. Plugin Process Identity

```rust
pub struct PluginProcessId(Ulid);
```

---

# 25. Plugin Session

```rust
pub struct PluginSessionId(Ulid);
```

New per plugin process incarnation.

---

# 26. Session Fencing

Requests/responses bind current session.

---

# 27. Trust Tier

```rust
pub enum PluginTrustTier {
    BuiltIn,
    TrustedExternal,
    SandboxedExternal,
    Disabled,
}
```

---

# 28. BuiltIn

Static Rust adapter.

---

# 29. TrustedExternal

Enterprise/operator-approved process with broader capabilities.

Still no direct database authority.

---

# 30. SandboxedExternal

Default third-party.

---

# 31. Disabled

Installed but not active.

---

# 32. Plugin Permissions

Examples:

```rust
pub enum PluginPermission {
    Network(NetworkPermission),
    SecretUse(SecretPurposeSelector),
    ArtifactRead(ArtifactScope),
    ArtifactWrite(ArtifactScope),
    ProviderOperation(ProviderOperationScope),
    UiContribution,
    DoctorCheck,
    NotificationSend,
}
```

---

# 33. No `DatabaseAccess` Permission

There should not be a generic direct DB permission.

---

# 34. No `BypassAuthz`

Never.

---

# 35. No `ScheduleJob`

External plugin does not schedule directly.

It can request a typed service operation if extension point permits.

---

# 36. No `MarkSuccess`

Plugin returns result/evidence.

Core service decides state transition.

---

# 37. Capability Token

Host can issue short-lived typed capability to plugin.

---

# 38. Plugin Capability Grant

```rust
pub struct PluginCapabilityGrant {
    pub plugin: InstalledPluginId,
    pub session: PluginSessionId,
    pub capability: PluginCapability,
    pub expires_at: Timestamp,
}
```

---

# 39. Capability Context

Bound to:

```text
tenant
project
operation
resource
request ID
```

---

# 40. Secret Access

Plugin gets:

```text
SecretRef-mediated operation
```

not unrestricted secret store.

---

# 41. Preferred Secret Model

Host resolves/proxies only exact authorized secret purpose.

---

# 42. Provider Token

For SCM/deploy plugin:

```text
short-lived scoped token
```

if possible.

---

# 43. Non-Exportable Key

Plugin receives operation handle, not private key.

---

# 44. Plugin Configuration

Stored as typed/versioned plugin config blob.

---

# 45. Config Secrets

Use SecretRef.

---

# 46. Config Validation

Plugin may provide schema/validator.

Host also bounds size/types.

---

# 47. Config Version

```rust
pub struct PluginConfigVersion(u32);
```

---

# 48. Config Migration

Plugin provides explicit migration path.

---

# 49. Plugin State

Avoid arbitrary hidden local state for critical semantics.

---

# 50. Durable Plugin State

Preferred host-mediated key/value state.

---

# 51. PluginState API

```rust
pub trait PluginStateStore {
    async fn get(...);
    async fn put(...);
    async fn delete(...);
}
```

---

# 52. Namespaced

Per installed plugin + tenant/project.

---

# 53. Bounded

Quota.

---

# 54. No SQL Schema Control

Third-party plugin cannot create arbitrary tables.

---

# 55. First-Party Adapter Exception

Built-in workspace crates can have normal reviewed migrations if architecture requires.

---

# 56. Artifact Access

Plugin requests artifact by ref.

Host authorizes and streams.

---

# 57. No CAS Root Directory Access

Never give filesystem path to entire CAS.

---

# 58. Artifact Write

Plugin uploads through scoped writer.

Host computes/verifies digest.

---

# 59. Network Access

Sandboxed external plugin has deny-by-default network.

---

# 60. Network Allowlist

Manifest/operator policy may permit:

```text
api.vendor.example
registry.example
```

---

# 61. DNS

Controlled.

---

# 62. Loopback

Restrict to plugin IPC/explicit endpoints.

---

# 63. Metadata Service

Block cloud metadata endpoints by default.

---

# 64. Filesystem

Sandbox exposes:

```text
read-only plugin package
temp dir
scoped working dir
```

---

# 65. No Host Home

Default denied.

---

# 66. No Docker Socket

Denied.

---

# 67. No SSH Agent

Denied.

---

# 68. Process Spawn

Sandbox policy.

---

# 69. Native Libraries

Contained within plugin sandbox/process.

---

# 70. Plugin Runtime Limits

```rust
pub struct PluginResourceLimits {
    pub memory: ByteSize,
    pub cpu: CpuLimit,
    pub processes: u32,
    pub temp_storage: ByteSize,
    pub request_timeout: Duration,
}
```

---

# 71. Kill on Limit

Plugin request fails; core remains healthy.

---

# 72. Crash Isolation

Plugin panic/process crash does not crash daemon.

---

# 73. Restart Policy

```rust
pub enum PluginRestartPolicy {
    Never,
    OnFailure,
    Always,
}
```

---

# 74. Backoff

Exponential/jittered.

---

# 75. Crash Loop

Disable/quarantine plugin after threshold.

---

# 76. Plugin Health

```rust
pub enum PluginHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Disabled,
}
```

---

# 77. Health Probe

Handshake + extension-specific optional probe.

---

# 78. Doctor

Plugin can expose typed doctor checks.

---

# 79. Plugin Doctor Check

Returns:

```text
Pass
Warn
Fail
Skipped
```

---

# 80. Doctor Security

No arbitrary shell output/action.

---

# 81. Extension Registry

Host-side registry.

---

# 82. Static Registry

Built-in extension registration at bootstrap.

---

# 83. Dynamic Registry

External active plugin sessions.

---

# 84. Registration

Plugin handshake declares manifest extensions.

Host matches installed/approved manifest.

---

# 85. No Self-Declared Extra Capability

Runtime declaration cannot exceed installed manifest.

---

# 86. Extension Point ID

```rust
pub struct ExtensionPointId(BoundedString);
```

---

# 87. Known Extension Point Examples

```text
scm.provider
secret.provider
cas.backend
executor
package.adapter
deploy.provider
scanner.vulnerability
notification.provider
doctor.check
ui.panel
```

---

# 88. Extension Descriptor

```rust
pub struct ExtensionDescriptor {
    pub point: ExtensionPointId,
    pub implementation: ExtensionImplementationId,
    pub capabilities: ExtensionCapabilities,
}
```

---

# 89. Selection

Core service chooses extension by configured provider/format/kind.

---

# 90. Plugin Cannot Hijack Existing Extension

Explicit configuration/priority.

---

# 91. Duplicate ID

Installation rejected.

---

# 92. Extension Priority

Avoid arbitrary numeric override initially.

Explicit selection is safer.

---

# 93. SCM Provider Extension

Implements normalized SCM provider contract.

---

# 94. VCS Extension

More security-sensitive.

Recommended:

```text
built-in/trusted only initially
```

because source materialization defines source identity.

---

# 95. Secret Provider Extension

High sensitivity.

Recommended:

```text
built-in or trusted external
```

---

# 96. CAS Backend Extension

High correctness sensitivity.

Recommended first-party/trusted only.

---

# 97. Executor Extension

Highest risk.

---

# 98. Executor Plugin Policy

Third-party arbitrary executor cannot run inside daemon.

At minimum external isolated process + runner-side explicit installation/trust.

---

# 99. Executor Registration

Runner declares installed executor capability.

---

# 100. Scheduler

Only schedules to runner if executor capability approved.

---

# 101. Executor Trust

Admin/policy controlled.

---

# 102. Package Adapter Extension

Moderate risk.

Can run as normal sandboxed packaging job/process.

---

# 103. Deployment Provider Extension

High privilege due to infrastructure credentials.

Must use scoped secrets/workload identity and typed deployment operations.

---

# 104. Artifact Publisher Extension

Similar release/publish restrictions.

---

# 105. Vulnerability Scanner Extension

Good sandboxed plugin candidate.

---

# 106. License Scanner Extension

Good sandboxed candidate.

---

# 107. Notification Provider Extension

Good sandboxed candidate.

---

# 108. Policy Fact Provider

Can provide facts/evidence only.

---

# 109. Cannot Decide Authorization

Critical.

---

# 110. Policy Fact

```rust
pub struct PluginPolicyFact {
    pub key: PolicyFactKey,
    pub value: PolicyFactValue,
    pub provenance: PluginFactProvenance,
}
```

---

# 111. Fact Provenance

Includes plugin ID/version/result digest.

---

# 112. Policy Engine

May consume fact according to configured trust.

---

# 113. UI Extension

Must be constrained.

---

# 114. UI Extension Types

Recommended initial:

```text
navigation link
entity detail panel
dashboard card
settings page
action descriptor
```

---

# 115. No Arbitrary Core DOM Replacement

Third-party UI extension cannot replace:

```text
login
policy
secret reveal
release approval
deployment rollback
```

core security surfaces.

---

# 116. UI Extension Descriptor

```rust
pub struct UiExtensionDescriptor {
    pub location: UiExtensionLocation,
    pub title: BoundedString,
    pub data_source: UiExtensionDataSource,
}
```

---

# 117. UI Code Execution

Avoid loading arbitrary native/web code into Dioxus process initially.

---

# 118. Safer Initial UI Plugin Model

Declarative panels/forms driven by descriptors + plugin API.

---

# 119. Later Rich UI

Could use sandboxed web content/isolated component protocol, but high complexity.

---

# 120. UI Data Access

Through scoped plugin API endpoints.

---

# 121. No Direct API Token Exposure

UI plugin never receives session bearer token if avoidable.

---

# 122. Host Proxy

Host UI/API proxies authorized plugin requests.

---

# 123. UI Action

Descriptor names typed backend plugin operation.

---

# 124. Server Authz

Still authoritative.

---

# 125. Notification Plugin

```rust
pub trait NotificationProvider {
    async fn send(
        &self,
        request: NotificationRequest,
    ) -> Result<NotificationResult, NotificationError>;
}
```

External plugin contract mirrors normalized request.

---

# 126. Notification Secret

Provider token via scoped SecretRef.

---

# 127. Retry

Notification delivery uses normal external-effect semantics.

---

# 128. Idempotency

Notification request ID.

---

# 129. Vulnerability Scanner Plugin

Input:

```text
artifact/SBOM refs
scanner profile
```

Output:

```text
normalized findings
raw evidence CAS ref
```

---

# 130. Scanner Cannot Mutate Artifact

Read only.

---

# 131. Deployment Plugin

Input:

```text
DeploymentPlan subset
target
desired action
credential capability
```

Output:

```text
normalized observed state/result
```

---

# 132. Deployment Plugin Cannot Mark Deployment Healthy

Core evaluates health/policy.

---

# 133. SCM Plugin

Returns normalized provider state.

---

# 134. SCM Plugin Cannot Mint Forgeyard Principal

Core identity mapper.

---

# 135. Package Plugin

Input immutable artifacts.

Output package bytes/evidence.

---

# 136. Package Plugin Cannot Publish

Unless separately granted publisher extension.

---

# 137. Separation of Extension Capabilities

One plugin package may declare multiple extensions, but permissions remain per extension/operation.

---

# 138. Least Privilege

Do not give plugin all permissions because one extension needs them.

---

# 139. Request Context

```rust
pub struct PluginRequestContext {
    pub request_id: PluginRequestId,
    pub installation: InstalledPluginId,
    pub session: PluginSessionId,
    pub tenant: TenantId,
    pub principal: Option<PrincipalId>,
    pub deadline: Timestamp,
}
```

---

# 140. PluginRequestId

```rust
pub struct PluginRequestId(Ulid);
```

---

# 141. Request Deadline

Mandatory.

---

# 142. Cancellation

Host can cancel request.

---

# 143. Plugin Response

```rust
pub struct PluginResponse<T> {
    pub request_id: PluginRequestId,
    pub result: Result<T, PluginRemoteError>,
}
```

---

# 144. Remote Error

Typed stable code/message.

---

# 145. No Rust Panic Across Boundary

Process boundary.

---

# 146. Protocol Envelope

```rust
pub struct PluginEnvelope<T> {
    pub protocol: PluginApiVersion,
    pub session: PluginSessionId,
    pub message: T,
}
```

---

# 147. Handshake

```text
host hello
plugin hello
manifest digest
API version negotiate
session issue
extension register
health
```

---

# 148. Version Negotiation

Major must match.

Minor feature negotiation.

---

# 149. Feature Bits

```rust
pub struct PluginProtocolFeatures(BTreeSet<PluginProtocolFeature>);
```

---

# 150. Unknown Feature

Ignored/rejected according to required flag.

---

# 151. Rolling Forgeyard Upgrade

External plugins remain compatible across declared plugin API range.

---

# 152. Plugin Upgrade

Can run:

```text
old version
drain
stop
install new
handshake
health
activate
```

---

# 153. Zero-Downtime Plugin Upgrade

Possible for stateless extensions by running old/new concurrently then switch registry.

---

# 154. State Migration

For stateful plugin, explicit.

---

# 155. Plugin Install

```text
verify package
verify manifest
verify signature if required
check compatibility
review requested permissions
store package
register disabled
operator enable
```

---

# 156. Plugin Package

Immutable artifact.

---

# 157. PluginPackageId

```rust
pub struct PluginPackageId(Digest);
```

---

# 158. Package Contents

```text
manifest.ron
binary
license
SBOM
signature
optional schemas/assets
```

---

# 159. Plugin Supply Chain

Treat plugin as software supply-chain artifact.

---

# 160. Plugin Verification

Can require:

```text
signature
SBOM
provenance
approved publisher
```

---

# 161. Marketplace

Future optional.

Not required for core.

---

# 162. Enterprise Allowlist

```text
allowed plugin IDs/vendors/signing roots
```

---

# 163. Plugin Install Policy

Central policy engine.

---

# 164. Trust Promotion

Sandboxed -> TrustedExternal requires explicit admin/policy.

---

# 165. Never Auto-Trust by Popularity

Critical.

---

# 166. Plugin Disable

Stops new calls.

---

# 167. In-Flight Requests

Drain/cancel according to timeout.

---

# 168. Plugin Uninstall

Requires:

```text
disable
verify no active dependency
remove package
retain audit/config history
```

---

# 169. Orphaned Configuration

Mark inactive.

---

# 170. Plugin State Retention

Policy-controlled.

---

# 171. Plugin Removal Does Not Rewrite Historical Evidence

Historical evidence records plugin ID/version.

---

# 172. Missing Historical Plugin

Evidence remains readable.

---

# 173. Historical Verification

Plugin binary/package digest may be retained if needed.

---

# 174. Extension Output Provenance

Every output includes:

```text
PluginId
PluginVersion
PluginPackageId
extension point
request ID
```

---

# 175. Cache Semantics

Plugin version affecting output must be part of cache/derivation key.

---

# 176. Package Adapter Example

`PackageSpecId`/derivation includes plugin implementation version/digest.

---

# 177. Scanner Example

Evidence records scanner plugin version/database version.

---

# 178. Deploy Provider Example

Deployment plan may bind provider extension implementation version if behavior affects semantics.

---

# 179. Plugin Update Drift

If active deployment provider changes version, existing DeploymentPlan remains bound to previous semantic adapter version or requires re-plan.

---

# 180. Plugin Capability Version

Each extension contract can have independent schema version.

---

# 181. Extension API

```rust
pub struct ExtensionApiVersion {
    pub point: ExtensionPointId,
    pub major: u16,
    pub minor: u16,
}
```

---

# 182. Global vs Point Version

Global handshake + per-extension contracts.

---

# 183. Breaking One Extension

Does not necessarily break all plugin APIs.

---

# 184. Configuration Schema

Plugin can expose machine-readable schema.

---

# 185. RON Schema

Forgeyard-native.

---

# 186. JSON Schema

Optional for UI/editor integration.

---

# 187. UI Settings Form

Host generates from constrained schema where possible.

---

# 188. Secret Field

Schema marks:

```text
SecretRef
```

not plaintext persisted.

---

# 189. Validation

Host basic + plugin semantic validator.

---

# 190. Plugin Endpoint

No arbitrary public HTTP listener by default.

---

# 191. Host-Mediated Webhook

Provider plugin can register route descriptor.

Axum receives/limits/authenticates framing then passes raw payload to plugin verifier/normalizer.

---

# 192. Webhook Security

Plugin returns verification result; host still checks binding/dedup/limits.

---

# 193. Plugin Cannot Bind Arbitrary Host Route

Routes under controlled namespace.

---

# 194. Route Example

```text
/webhooks/plugins/{plugin-id}/{binding}
```

---

# 195. Public Plugin API

Optional under:

```text
/v1/plugins/{plugin-id}/...
```

only declared typed operations.

---

# 196. No Raw Reverse Proxy

Do not expose plugin HTTP server transparently.

---

# 197. Plugin Outbound HTTP

Host could provide mediated HTTP client for stronger sandboxing.

---

# 198. Mediated HTTP

Allows:

```text
allowlist
timeouts
TLS trust
metrics
secret header injection
```

---

# 199. Recommended for Sandboxed Plugins

Prefer host-mediated outbound HTTP over unrestricted sockets where feasible.

---

# 200. PluginHttpRequest

```rust
pub struct PluginHttpRequest {
    pub destination: AllowedEndpointId,
    pub method: HttpMethod,
    pub path: RelativeHttpPath,
    pub headers: SafeHeaders,
    pub body: BoundedBytes,
}
```

---

# 201. Secret Header

Host inserts provider auth from SecretRef.

---

# 202. Plugin Never Sees Token

Excellent for many API integrations.

---

# 203. Response Limits

Bound.

---

# 204. Redirect Policy

Host-controlled.

---

# 205. TLS Validation

Host-controlled.

---

# 206. This Pattern

Strongly reduces credential exfiltration for SCM/notification plugins.

---

# 207. Direct Network Mode

Only trusted plugin when mediated model insufficient.

---

# 208. Sandboxing Linux

Potential:

```text
namespaces
seccomp
cgroup v2
Landlock/bwrap
no_new_privs
```

---

# 209. Windows

Restricted token/Job Object/AppContainer/Firewall where practical.

---

# 210. macOS

Sandbox/limited process profile; stronger isolation via VM where needed.

---

# 211. Cross-Platform Consistency

Plugin trust tier can require external container/VM on weaker host.

---

# 212. WASM

Potential future plugin runtime.

---

# 213. WASM Advantages

```text
portable
capability-based host calls
memory isolation
easy resource limits
```

---

# 214. WASM Limitations

```text
ecosystem/tooling
native SDK access
performance
system integration
```

---

# 215. Recommended Architecture

Keep plugin contract runtime-neutral so WASM can be added later.

---

# 216. Do Not Make WASM Mandatory Initially

External process is pragmatic.

---

# 217. Dynamic Libraries

Not recommended for third-party plugins.

---

# 218. Why

```text
Rust ABI unstable
daemon crash risk
memory unsafety from foreign code
dependency conflicts
```

---

# 219. Built-In Crates

Use normal Rust traits.

---

# 220. BuiltIn Extension Trait

Example:

```rust
#[async_trait]
pub trait VulnerabilityScanner: Send + Sync {
    async fn scan(&self, request: ScanRequest) -> Result<ScanResult, ScanError>;
}
```

---

# 221. External Adapter

Host proxy implements same logical trait by IPC.

---

# 222. Uniform Service

Core does not care whether implementation built-in/external once approved.

---

# 223. But Trust Metadata

Core can inspect implementation trust tier.

---

# 224. Policy

May require:

```text
BuiltIn or TrustedExternal
```

for production deployment/signing contexts.

---

# 225. Extension Trust Requirement

```rust
pub struct ExtensionTrustRequirement {
    pub minimum: PluginTrustTierRequirement,
}
```

---

# 226. Signing Extension

Private-key signing should remain core Secrets/Trust/Signing provider interfaces.

Third-party signer only trusted external/built-in.

---

# 227. CA Extension

Same.

---

# 228. Identity Provider Extension

High-risk; not baseline plugin point initially.

---

# 229. Policy Engine Extension

Do not allow plugin to replace central policy evaluator.

---

# 230. Policy Fact Extension Only

Safer.

---

# 231. Authz Extension

Not baseline.

---

# 232. Audit Sink Extension

Potential later, but audit persistence remains core.

---

# 233. Telemetry Exporter Extension

Reasonable plugin point.

---

# 234. Telemetry Plugin

Receives sanitized structured telemetry, not secrets.

---

# 235. UI Plugin Rendering

Declarative first.

---

# 236. Example Dashboard Card

```rust
pub struct UiCardDescriptor {
    pub title: BoundedString,
    pub endpoint: PluginUiDataEndpoint,
    pub renderer: UiRendererKind,
}
```

---

# 237. Renderer Kinds

```text
KeyValue
Table
StatusList
MarkdownSanitized
Chart
```

---

# 238. Markdown

Sanitized.

---

# 239. No arbitrary HTML/JS

Initial invariant.

---

# 240. Extension Localization

Plugin provides strings/resources.

---

# 241. Accessibility

Host components enforce baseline.

---

# 242. Plugin Navigation

Namespace under plugin area unless approved core placement.

---

# 243. UI Action Permission

Descriptor includes required Forgeyard permission.

Server still checks.

---

# 244. Plugin Health UI

Show:

```text
version
trust tier
permissions
health
last restart
```

---

# 245. Admin Plugin Page

Tabs:

```text
Overview
Extensions
Permissions
Configuration
Health
Logs
Updates
Audit
```

---

# 246. Plugin Logs

Separate target, sanitized.

---

# 247. Resource Metrics

```text
CPU
memory
request latency
restart count
errors
```

---

# 248. Metrics Labels

Plugin ID can be high-cardinality if marketplace huge.

Use bounded/admin metrics or top-N; traces/logs for exact ID.

---

# 249. Core Metrics

```text
plugin_active
plugin_request_duration
plugin_request_failures
plugin_restart_total
plugin_quarantined
```

---

# 250. Tracing

```text
plugin.call
plugin.handshake
plugin.start
plugin.stop
plugin.health
plugin.http
```

---

# 251. Correlation

PluginRequestId + original RequestId/RunId/etc in logs/traces.

---

# 252. No Metric Labels With User Resource IDs

Same observability rules.

---

# 253. Audit

Audit:

```text
install
enable
disable
permission change
trust-tier change
update
uninstall
```

---

# 254. Plugin Operation Audit

High-risk extension actions may also be audited.

---

# 255. Plugin Marketplace Metadata

Future:

```text
publisher
signature
license
SBOM
provenance
compatibility
```

---

# 256. Licensing

Plugin may have independent license.

Forgeyard UI should expose license info.

---

# 257. Policy

Enterprise can forbid non-approved licenses.

---

# 258. Plugin Dependency

Avoid plugin-to-plugin dependencies initially.

---

# 259. Why

Complex upgrade graph.

---

# 260. Shared SDK

Use versioned Forgeyard plugin SDK.

---

# 261. SDK

Provides:

```text
manifest types
IPC protocol
client/server stubs
test harness
```

---

# 262. SDK Crate

```text
crates/plugin/forgeyard-plugin-sdk/
```

---

# 263. Workspace Structure

Recommended logical crates:

```text
crates/plugin/
├── forgeyard-plugin/
├── forgeyard-plugin-model/
├── forgeyard-plugin-api/
├── forgeyard-plugin-sdk/
├── forgeyard-plugin-manifest/
├── forgeyard-plugin-registry/
├── forgeyard-plugin-supervisor/
├── forgeyard-plugin-ipc/
├── forgeyard-plugin-sandbox/
├── forgeyard-plugin-permission/
├── forgeyard-plugin-state/
├── forgeyard-plugin-http/
├── forgeyard-plugin-ui/
├── forgeyard-plugin-health/
├── forgeyard-plugin-install/
├── forgeyard-plugin-update/
└── forgeyard-plugin-testkit/
```

Apply module-first rule; physical crates only where dependency/security/runtime boundaries justify.

---

# 264. Plugin Registry Record

```rust
pub struct InstalledPlugin {
    pub id: InstalledPluginId,
    pub package: PluginPackageId,
    pub manifest: PluginManifestDigest,
    pub trust: PluginTrustTier,
    pub state: InstalledPluginState,
}
```

---

# 265. Installed State

```rust
pub enum InstalledPluginState {
    Installed,
    Enabled,
    Disabled,
    Updating,
    Quarantined,
    Failed,
}
```

---

# 266. Quarantined Plugin

No new calls.

---

# 267. Quarantine Reasons

```text
crash loop
protocol violation
permission violation
signature invalid
resource abuse
health failure
```

---

# 268. Protocol Violation

Examples:

```text
wrong session
oversized message
unexpected response
```

---

# 269. Permission Violation

Host denies and records.

Repeated malicious behavior can quarantine.

---

# 270. Plugin Request Queue

Bounded per plugin.

---

# 271. Concurrency Limit

Manifest/operator policy.

---

# 272. Backpressure

Caller gets resource exhausted/retry if queue full.

---

# 273. Critical Core Path

Never wait unbounded for optional plugin.

---

# 274. Optional Extension Failure

Degrade feature only.

---

# 275. Required Extension Failure

Block exact operation safely.

---

# 276. Example

Vulnerability scanner unavailable:

```text
build can succeed
release requiring scan cannot pass
```

---

# 277. Deployment Provider Unavailable

Deployment blocked/degraded.

---

# 278. SCM Provider Unavailable

Source sync/check publication degraded.

---

# 279. Plugin Timeout

Typed error.

---

# 280. Retry Semantics

Extension-specific.

---

# 281. External Side Effect

Plugin request includes idempotency key.

---

# 282. Unknown Outcome

Plugin must support inspect/reconcile if extension has side effects.

---

# 283. Deployment Plugin Example

```text
apply timed out
  ↓
Unknown
  ↓
inspect
```

---

# 284. SCM Merge Plugin Example

Same.

---

# 285. Notification Plugin

Duplicate okay only with idempotency/dedup semantics.

---

# 286. Plugin Reconciliation

Core subsystem owns desired-state reconciliation.

Plugin supplies inspect/apply primitives.

---

# 287. Plugin Must Not Own Global Reconcile Loop

Avoid hidden authority.

---

# 288. Plugin Background Tasks

Allowed only declared bounded maintenance tasks.

---

# 289. Background Task Authority

Cannot mutate core state directly.

---

# 290. Host Callback

Plugin can emit:

```text
health update
provider event
fact/evidence
```

through typed channel.

---

# 291. Callback Validation

Host validates installation/session/scope.

---

# 292. Event Subscription

Plugin may subscribe only to declared safe event projection.

---

# 293. No Full Internal Event Bus Access

Critical.

---

# 294. Event Projection

Example:

```text
RunCompleted
ReleaseReleased
DeploymentFailed
```

sanitized DTO.

---

# 295. Notification Trigger

Core notification service decides which event invokes plugin.

---

# 296. Plugin Data Retention

Host-managed state quotas.

---

# 297. Plugin Temp Data

Deleted on restart/cleanup.

---

# 298. Plugin Package Storage

CAS/library artifact.

---

# 299. Executable Materialization

Verified into controlled plugin directory.

---

# 300. File Permissions

Read-only package, executable entrypoint.

---

# 301. Upgrade Rollback

Keep previous package until new version healthy.

---

# 302. Activation Switch

Atomic registry pointer.

---

# 303. Failed Upgrade

Return to previous healthy plugin version.

---

# 304. Schema Migration Failure

Do not activate new plugin.

---

# 305. Downgrade

Only if manifest/state compatibility allows.

---

# 306. Plugin Dependency on Forgeyard API

Checked pre-activation.

---

# 307. Safe Mode

Daemon can start with external plugins disabled.

---

# 308. Recovery Use

```text
forgeyard --safe-mode
```

or config.

---

# 309. Plugin Cannot Prevent Daemon Startup

Unless explicitly marked required core integration; even then admin recovery path.

---

# 310. Required Plugin

Example enterprise mandatory secret provider may be required for certain operations, but control plane should still expose diagnostics.

---

# 311. Bootstrap Dependency

Avoid needing third-party plugin to unlock plugin registry itself.

---

# 312. Secret Provider Bootstrap

Core local/provider support remains available.

---

# 313. Upgrade Bootstrap

Plugin supervisor should be simple.

---

# 314. Plugin Discovery

Explicit installed registry.

---

# 315. No Scanning Arbitrary PATH

Avoid accidental code loading.

---

# 316. Installation Source

```text
local file
trusted registry URL
Forgeyard artifact
```

---

# 317. Download

Use normal artifact/download verification.

---

# 318. Checksum

Verify.

---

# 319. Signature

Verify if policy requires.

---

# 320. SBOM/Provenance

Can be required.

---

# 321. Plugin Policy Facts

```text
publisher trust
signature status
license
vulnerabilities
```

feed install policy.

---

# 322. Security Review

High-trust plugin requires manual approval.

---

# 323. Trust Tier Policy Example

```text
SandboxedExternal:
  scanners/notifications/UI only

TrustedExternal:
  SCM/deploy providers

BuiltIn:
  CAS, VCS, signing, coordination
```

---

# 324. This Is Default, Not Absolute

Enterprise policy can tighten.

---

# 325. Executor Plugin Default

BuiltIn/TrustedExternal only.

---

# 326. CAS Plugin Default

BuiltIn/TrustedExternal.

---

# 327. Secret Provider Default

BuiltIn/TrustedExternal.

---

# 328. UI Plugin Default

Sandboxed declarative.

---

# 329. Scanner Default

Sandboxed.

---

# 330. Notification Default

Sandboxed.

---

# 331. Plugin Capability Matrix

Host computes allowed extension/trust combination.

---

# 332. Reject Unsafe Combination

Example:

```text
SandboxedExternal + CoordinationBackend
```

not allowed.

---

# 333. Coordination Backend

Not a plugin point initially.

---

# 334. Policy Evaluator

Not a plugin point initially.

---

# 335. Identity Core

Not a plugin point initially.

---

# 336. Scheduler Core

Not a plugin point initially.

---

# 337. Reason

These define central system authority.

---

# 338. Testkit

```text
forgeyard-plugin-testkit/src/
├── lib.rs
├── manifest.rs
├── fake_plugin.rs
├── supervisor.rs
├── protocol.rs
├── permission.rs
├── sandbox.rs
├── state.rs
└── assertions.rs
```

---

# 339. Unit Tests

Test:

```text
manifest validation
version negotiation
permission resolution
state machine
```

---

# 340. Protocol Tests

Wrong version/session rejected.

---

# 341. Oversize Message Test

Rejected/quarantine if malicious.

---

# 342. Crash Test

Plugin crash does not crash daemon.

---

# 343. Crash Loop Test

Plugin quarantined.

---

# 344. Timeout Test

Request deadline enforced.

---

# 345. Memory Limit Test

Plugin killed safely.

---

# 346. CPU Abuse Test

Cgroup/Job Object limits.

---

# 347. Filesystem Escape Test

Cannot access host secrets/home/CAS root.

---

# 348. Network Test

Sandboxed plugin cannot reach undeclared endpoint.

---

# 349. Metadata Service Test

Blocked.

---

# 350. Secret Test

Plugin cannot enumerate/read arbitrary secrets.

---

# 351. Scoped Secret Test

Exact authorized provider operation works.

---

# 352. Mediated HTTP Test

Plugin performs API call without seeing credential token.

---

# 353. Authz Test

UI/plugin action denied server-side if principal lacks permission.

---

# 354. DB Access Test

No direct database socket/filesystem available to sandboxed plugin unless explicitly network path accidentally exposed; sandbox/network policy blocks.

---

# 355. CAS Test

Plugin only gets authorized artifact streams.

---

# 356. Scheduler Test

Plugin cannot create JobLease.

---

# 357. Release Test

Plugin cannot mark release Released.

---

# 358. Deployment Test

Provider plugin result still requires core deployment state transition.

---

# 359. Plugin Upgrade Test

Old -> new with health switch.

---

# 360. Failed Upgrade Test

Rollback previous version.

---

# 361. Config Migration Test

Explicit and idempotent.

---

# 362. Historical Evidence Test

Old plugin version remains identifiable after uninstall.

---

# 363. Signature Test

Tampered package rejected.

---

# 364. Compatibility Test

Unsupported plugin API major rejected.

---

# 365. Fuzzing

Fuzz:

```text
manifest parser
IPC envelope
extension payload decoders
config schema
```

---

# 366. Failure Injection

```text
plugin crash mid-request
IPC disconnect
disk full
network timeout
host restart
upgrade failure
```

---

# 367. Scale Tests

```text
many installed plugins
high scanner requests
many notification events
```

---

# 368. Supervisor Startup Scale

Parallel bounded startup.

---

# 369. Implementation Phase 1 — Built-In Extension Registry

Formalize static extension interfaces.

---

# 370. Phase 2 — Manifest/Registry

Plugin package/install metadata.

---

# 371. Phase 3 — External Process IPC

Versioned handshake/request-response.

---

# 372. Phase 4 — Supervisor/Sandbox

Process lifecycle/resource limits.

---

# 373. Phase 5 — Permissions/Host Services

Artifact, secret, network mediation.

---

# 374. Phase 6 — First Sandboxed Plugin Point

Recommended:

```text
VulnerabilityScanner
```

---

# 375. Phase 7 — Notification Provider

External side-effect semantics.

---

# 376. Phase 8 — Declarative UI Extensions

Safe panels/cards/forms.

---

# 377. Phase 9 — SCM/Deployment Trusted Plugins

Higher trust.

---

# 378. Phase 10 — Update/Rollback/Supply Chain

Signed plugin packages.

---

# 379. Phase 11 — Enterprise Policy

Allowlist/trust tiers.

---

# 380. Phase 12 — WASM Runtime Optional

Only if benefits justify.

---

# 381. Acceptance Tests

1. Third-party plugin cannot load arbitrary in-process Rust ABI code by default.
2. Built-in extensions use normal Rust traits.
3. External plugins use versioned process/IPC boundary.
4. Plugin manifest is immutable/digest-addressed.
5. Runtime capabilities cannot exceed installed manifest.
6. Plugin permissions cannot include authz bypass.
7. Sandboxed plugin has no direct DB access.
8. Sandboxed plugin has no CAS root filesystem access.
9. Plugin cannot enumerate arbitrary secrets.
10. Exact scoped SecretRef/provider operation can be granted.
11. Provider credentials can be injected by host-mediated HTTP without exposing token to plugin.
12. Plugin cannot schedule JobLease directly.
13. Plugin cannot mark release/deployment/job success authoritatively.
14. Policy fact plugin supplies facts only, not decisions.
15. UI plugin cannot replace core security surfaces.
16. Initial UI extensions are declarative and sanitized.
17. Plugin crash does not crash daemon.
18. Resource limits are enforced.
19. Crash-looping plugin is quarantined.
20. Optional plugin outage degrades only its feature.
21. Required plugin outage blocks only dependent operation safely.
22. Plugin requests have deadlines and bounded queues.
23. State-changing plugin operations are idempotent/reconciled.
24. Plugin version affecting outputs is recorded in provenance/cache identity.
25. Plugin update can roll back to prior healthy version.
26. Historical evidence retains plugin ID/version/package digest.
27. Tampered/unsigned plugin is rejected according to policy.
28. Enterprise allowlist can restrict vendors/licenses/trust tiers.
29. Core authority systems such as policy evaluator, scheduler core, and coordination are not third-party plugin points initially.
30. Plugin install/update/remove actions are audited.
31. Safe mode can start Forgeyard with external plugins disabled.
32. Standalone/distributed share extension semantics.
33. External plugins cannot create arbitrary public routes/listeners.
34. Provider-specific SDKs remain isolated inside extension implementation.
35. Forgeyard can dogfood external scanner/notification plugins without weakening core invariants.

---

# 382. Production Readiness Gates

Do not call plugin system production-ready until:

```text
manifest/versioning stable
external IPC boundary stable
supervisor crash isolation proven
sandbox/resource limits proven
permission mediation complete
secret/artifact/network host services safe
first sandboxed plugin conformance passes
upgrade/rollback tested
audit/health/doctor available
safe-mode recovery tested
supply-chain verification for plugin packages integrated
```

---

# 383. Architectural Invariants

1. installed plugin does not imply authority;
2. external plugin is sandboxed by default;
3. third-party dynamic libraries are not loaded into daemon by default;
4. Rust ABI is not an external compatibility contract;
5. plugin APIs are versioned;
6. plugin permissions are explicit and least privilege;
7. plugin cannot bypass authz;
8. plugin cannot directly mutate business database;
9. plugin cannot create scheduler leases;
10. plugin cannot forge release/deployment/job terminal state;
11. secret access is scoped and mediated;
12. CAS access is artifact-scoped;
13. network is deny-by-default for sandboxed plugins;
14. plugin resource use is bounded;
15. plugin crashes are isolated;
16. crash loops lead to quarantine;
17. optional plugin failure does not crash core;
18. external side effects use idempotency/reconciliation;
19. plugin version/digest is part of provenance where behavior matters;
20. plugin config secrets are SecretRefs;
21. UI extensions are constrained and sanitized;
22. policy plugins supply facts, not authorization decisions;
23. correctness-critical extension points default to built-in/trusted;
24. plugin lifecycle is audited;
25. plugin packages are immutable and verifiable;
26. upgrade can roll back;
27. historical evidence remains valid after plugin removal;
28. safe mode can disable external plugins;
29. standalone/distributed share plugin semantics;
30. Forgeyard dogfoods its extension system without weakening core security.

---

# 384. Final Target Architecture

```text
                      Forgeyard Core
                           │
                           ▼
                    Extension Registry
              ┌────────────┼────────────┐
              ▼            ▼            ▼
           Built-In    Trusted       Sandboxed
           Rust Trait  Process        Process
              │            │            │
              └────────────┼────────────┘
                           ▼
                    Typed Host API
              ┌────────────┼────────────┐
              ▼            ▼            ▼
          Artifact      Secret       Network
           Access      Capability   Mediation
              │            │            │
              └────────────┼────────────┘
                           ▼
                  Core Domain Services
```

---

# 385. Final Architectural Position

Plugin installation:

```text
plugin package
  ↓
digest/signature/SBOM/provenance verify
  ↓
manifest compatibility check
  ↓
permission review
  ↓
install disabled
  ↓
sandboxed handshake
  ↓
health
  ↓
activate extension
```

Runtime call:

```text
core service request
  ↓
selected approved extension
  ↓
typed scoped capability grant
  ↓
sandboxed plugin request
  ↓
normalized result/evidence
  ↓
core validates
  ↓
core performs authoritative state transition
```

High-risk boundary:

```text
plugin result ≠ authority
```

The key guarantee is:

> **Forgeyard can be extended deeply without handing extensions the keys to the platform. Plugins contribute capabilities, evidence, provider operations, and UI surfaces through explicit contracts, while Forgeyard retains sole authority over identity, policy, secrets, scheduling, state transitions, releases, deployments, and durable business truth.**

---

# 386. New-Repository Sequence

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
