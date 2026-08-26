# 64 — Forgeyard Remote Development Environments, Cloud Workspaces, Codespaces-Style Sessions & Developer Workspace Orchestration System Architecture

**Document type:** Core Remote Development Environment, Cloud Workspace, Developer Session, Prebuild, Workspace Persistence, IDE/Terminal Access & Developer Environment Orchestration System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** ephemeral remote developer workspaces, local/remote parity, workspace templates, exact source/toolchain/environment identity, prebuilds, persistent developer volumes, workspace suspension/resume, secure terminal/IDE access, port forwarding, private network access, developer secrets, per-workspace credentials, environment drift, cost/TTL governance, workspace snapshots, recovery, and promotion boundaries between development and CI/CD  
**Architecture style:** Reproducible developer environments, explicit workspace identity, disposable compute with bounded persistent state, zero-trust access, exact source snapshots, prebuild acceleration, dev/CI parity, no hidden release authority, and no undeclared dependency on mutable workstation state  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Developer Experience/Local Dev, Runner/Agent, Sandbox/Executor, Toolchains, Test Environments, Infrastructure-as-Code, Network Connectivity, Secrets/Trust, Runner Image Factory, Configuration, Multi-Tenancy/Quotas, Cost/FinOps, Identity/Authz, Audit, Observability, and Data Lifecycle. This subsystem gives Forgeyard a first-class remote development workspace model without compromising CI determinism or supply-chain trust.

---

## 1. Purpose

Developers increasingly work in environments that are not their physical laptops:

```text
remote Linux workspaces
cloud VMs
ephemeral containers
browser IDEs
SSH/terminal sessions
remote Dioxus/desktop IDE clients
GPU development workspaces
device-development hosts
on-prem development nodes
air-gapped engineering environments
```

A remote workspace can improve:

```text
onboarding
toolchain consistency
performance
large monorepo access
private-resource access
GPU availability
cross-platform development
reproducibility
```

But unmanaged remote environments also create risks:

```text
persistent snowflake machines
developer secrets in disks
shared SSH keys
stale source trees
untracked package installation
hidden build dependencies
long-lived production credentials
unbounded compute cost
workspace data leakage
```

The central rule is:

> **A developer workspace is a user-facing mutable working environment, but its reproducible base, source identity, toolchains, permissions, network reachability, and persistent state must be explicit.**

A second rule is:

> **Remote workspace success never substitutes for CI validation. A developer can experiment freely inside a workspace, but release/build authority remains with normal Forgeyard pipelines and evidence.**

A third rule is:

> **Persistent developer convenience state must be separated from authoritative build inputs. Personal caches, editor state, and writable home directories are never silently incorporated into derivation identity.**

---

## 2. Architectural Position

```text
                  Project / Source
                        │
                        ▼
                Workspace Template
                        │
                        ▼
                 Workspace Prebuild
                        │
                        ▼
                  Workspace Instance
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
           IDE       Terminal    Port Forward
            │           │           │
            └───────────┼───────────┘
                        ▼
                Developer Changes
                        │
                        ▼
                 Source Snapshot
                        │
                        ▼
                    Normal CI
```

---

## 3. Goals

The subsystem MUST:

1. define workspace identity;
2. define workspace template identity;
3. define developer environment identity;
4. support local and remote workspace parity;
5. support cloud and on-prem providers;
6. support ephemeral workspaces;
7. support persistent developer volumes;
8. support exact toolchain setup;
9. support source checkout/snapshot identity;
10. support prebuilds;
11. support suspension/resume;
12. support workspace recreation;
13. support secure IDE access;
14. support terminal access;
15. support port forwarding;
16. support private network resources;
17. support scoped developer secrets;
18. support short-lived credentials;
19. support per-user authorization;
20. support quotas and TTL;
21. support cost accounting;
22. support drift detection;
23. support workspace snapshot/recovery;
24. support multi-region placement;
25. support air-gapped operation;
26. support audit;
27. support UI/API/CLI;
28. support HA;
29. preserve CI/release boundaries;
30. eliminate hidden dependency on developer machine state.

---

## 4. Non-Goals

This subsystem does not replace:

```text
source control
CI pipelines
build runners
release systems
IDE products
desktop OS management
VDI products
endpoint management
```

It orchestrates developer workspaces around Forgeyard’s reproducible environment model.

---

## 5. Workspace Structure

```text
crates/dev-workspace/
├── forgeyard-dev-workspace/
├── forgeyard-dev-workspace-model/
├── forgeyard-dev-workspace-template/
├── forgeyard-dev-workspace-provision/
├── forgeyard-dev-workspace-prebuild/
├── forgeyard-dev-workspace-session/
├── forgeyard-dev-workspace-volume/
├── forgeyard-dev-workspace-access/
├── forgeyard-dev-workspace-drift/
├── forgeyard-dev-workspace-reconcile/
├── forgeyard-dev-workspace-health/
└── forgeyard-dev-workspace-testkit/
```

Adapters:

```text
crates/dev-workspace-adapters/
├── forgeyard-dev-workspace-local/
├── forgeyard-dev-workspace-kubernetes/
├── forgeyard-dev-workspace-aws/
├── forgeyard-dev-workspace-azure/
├── forgeyard-dev-workspace-gcp/
├── forgeyard-dev-workspace-libvirt/
├── forgeyard-dev-workspace-ssh/
└── forgeyard-dev-workspace-custom/
```

---

## 6. DeveloperWorkspaceId

```rust
pub struct DeveloperWorkspaceId(Ulid);
```

Represents one live or historical developer workspace instance.

---

## 7. WorkspaceTemplateId

```rust
pub struct WorkspaceTemplateId(Digest);
```

Immutable normalized definition of the desired developer environment.

---

## 8. WorkspaceTemplate

```rust
pub struct WorkspaceTemplate {
    pub id: WorkspaceTemplateId,
    pub project: ProjectId,
    pub platform: WorkspacePlatform,
    pub toolchains: Vec<ToolchainDescriptorId>,
    pub services: Vec<WorkspaceServiceRef>,
    pub network: WorkspaceNetworkPolicyId,
    pub resources: WorkspaceResourceSpec,
}
```

---

## 9. Workspace Platform

```rust
pub enum WorkspacePlatform {
    Linux,
    Windows,
    MacOS,
    Custom(PlatformId),
}
```

Remote Linux is expected to be the baseline.

Windows/macOS use real platform hosts where required.

---

## 10. Source Identity

Workspace attaches to:

```text
RepositoryId
RevisionId
SourceSnapshotId
```

Branch name is navigation metadata, not canonical source identity.

---

## 11. Dirty Working Tree

Allowed.

---

## 12. DirtyWorkspaceSnapshotId

```rust
pub struct DirtyWorkspaceSnapshotId(Digest);
```

When needed, Forgeyard can snapshot the working tree explicitly.

---

## 13. CI From Workspace

If developer runs CI from dirty workspace:

```text
capture explicit dirty SourceSnapshotId
  ↓
CI runs against immutable snapshot
```

No CI runner reads developer filesystem live.

---

## 14. Critical Boundary

```text
workspace mutable filesystem
    !=
CI source input
```

CI consumes explicit immutable snapshot.

---

## 15. Workspace State

```rust
pub enum DeveloperWorkspaceState {
    Requested,
    Provisioning,
    Starting,
    Ready,
    Active,
    Suspended,
    Resuming,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}
```

---

## 16. Unknown

Provider outcome uncertain.

Inspect before duplicate provision/start/stop.

---

## 17. Workspace Ownership

```rust
pub struct WorkspaceOwner {
    pub principal: PrincipalId,
    pub tenant: TenantId,
    pub project: ProjectId,
}
```

---

## 18. Shared Workspace

Not baseline.

---

## 19. Pairing/Shared Session

Can be explicitly enabled with separate session grants.

---

## 20. Workspace Template Source

Possible:

```text
forgeyard.ron
organization golden path
project-specific template
CLI-generated template
```

---

## 21. Example RON

```ron
(
    platform: Linux,
    resources: (
        cpu: 8,
        memory_gib: 16,
    ),
    toolchains: [
        "rust-stable-pinned",
        "node-pinned",
    ],
    services: [
        "postgres-dev",
    ],
    network_policy: "developer-standard",
)
```

---

## 22. Toolchains

Use existing immutable `ToolchainDescriptorId`.

---

## 23. Developer Convenience Tooling

Can include:

```text
rust-analyzer
debugger
editor server
language servers
git
shell utilities
```

---

## 24. Tooling Identity

Pinned where reproducibility matters.

---

## 25. User Customization

Allowed in mutable user layer.

---

## 26. User Layer

Examples:

```text
editor plugins
shell aliases
dotfiles
temporary packages
```

---

## 27. User Layer Is Not Workspace Base

Critical.

---

## 28. Layering

```text
Approved Workspace Base
        ↓
Project Template
        ↓
Prebuild
        ↓
User Mutable Layer
```

---

## 29. WorkspaceBaseId

```rust
pub struct WorkspaceBaseId(Digest);
```

Can map to Part 58 runner-image baseline or container image.

---

## 30. Base Image

Exact digest.

---

## 31. No `latest`

Critical.

---

## 32. Workspace Prebuild

Prebuild prepares reusable project state before developer starts.

---

## 33. WorkspacePrebuildId

```rust
pub struct WorkspacePrebuildId(Digest);
```

---

## 34. Prebuild Inputs

```text
WorkspaceTemplateId
SourceSnapshotId
toolchains
dependency lockfiles
setup scripts
```

---

## 35. Prebuild Output

Can include:

```text
fetched dependencies
compiled index
language-server index
generated code
local dev services image
```

---

## 36. Prebuild Is Optimization

Critical.

Workspace can initialize without it.

---

## 37. Prebuild Trust

Does not become release artifact.

---

## 38. Prebuild Scripts

Run in restricted environment.

---

## 39. Setup Script Identity

```rust
pub struct WorkspaceSetupScriptId(Digest);
```

---

## 40. Network During Prebuild

Declared.

---

## 41. Secret Use During Prebuild

Avoid where possible.

---

## 42. Secret-Bearing Prebuild

Cannot be globally shared/cached.

---

## 43. Prebuild Cache Scope

Tenant/project/trust-aware.

---

## 44. No Cross-Tenant Prebuild Reuse If Sensitive

Critical.

---

## 45. Developer Home Volume

```rust
pub struct DeveloperVolumeId(Ulid);
```

---

## 46. Volume Types

```rust
pub enum DeveloperVolumeKind {
    Home,
    WorkspacePersistent,
    Cache,
    Scratch,
}
```

---

## 47. Home

May persist across workspace recreation.

---

## 48. WorkspacePersistent

Project-specific writable state.

---

## 49. Cache

Disposable.

---

## 50. Scratch

Ephemeral.

---

## 51. Persistent Volume Encryption

Required for remote hosted workspaces.

---

## 52. Volume Ownership

Tenant + user + project scope.

---

## 53. No Cross-User Mount

Baseline.

---

## 54. Workspace Filesystem

Recommended logical layout:

```text
/workspace          project checkout
/home/developer     user state
/forgeyard/tools    immutable toolchains
/forgeyard/cache    disposable cache
/tmp                ephemeral
```

---

## 55. Project Checkout

Writable.

---

## 56. Toolchain Directory

Read-only where possible.

---

## 57. Workspace Service Dependencies

Examples:

```text
Postgres
Stoolap
Redis-compatible
local object store
test mail
message broker
```

---

## 58. Reuse Part 56

For reproducible dev/test service environments.

---

## 59. Dev Services

Can be longer-lived than test services, but identity remains explicit.

---

## 60. WorkspaceServiceRef

```rust
pub struct WorkspaceServiceRef {
    pub service: TestEnvironmentSpecId,
    pub persistence: WorkspaceServicePersistence,
}
```

---

## 61. Persistence

```rust
pub enum WorkspaceServicePersistence {
    Ephemeral,
    Persistent,
    SnapshotOnSuspend,
}
```

---

## 62. Developer Database

May contain local dev data.

Not production data baseline.

---

## 63. Production Data

Forbidden by default.

---

## 64. Masked Production-Derived Data

Part 56 policy applies.

---

## 65. Workspace Provisioning

Can use:

```text
local process/container
VM
Kubernetes pod
cloud VM
bare-metal host
```

---

## 66. Provider Abstraction

```rust
#[async_trait]
pub trait DeveloperWorkspaceProvider {
    async fn provision(
        &self,
        request: WorkspaceProvisionRequest,
    ) -> Result<WorkspaceProviderHandle, WorkspaceError>;

    async fn start(
        &self,
        handle: &WorkspaceProviderHandle,
    ) -> Result<(), WorkspaceError>;

    async fn suspend(
        &self,
        handle: &WorkspaceProviderHandle,
    ) -> Result<(), WorkspaceError>;

    async fn destroy(
        &self,
        handle: &WorkspaceProviderHandle,
    ) -> Result<(), WorkspaceError>;
}
```

---

## 67. Provider Intent

Persist before effect.

---

## 68. Provider Timeout

`Unknown`.

Inspect before retry.

---

## 69. No Duplicate VM Provisioning Blindly

Critical.

---

## 70. Workspace Resource Spec

```rust
pub struct WorkspaceResourceSpec {
    pub cpu: CpuUnits,
    pub memory: MemoryBytes,
    pub disk: StorageBytes,
    pub gpu: Option<GpuRequirement>,
}
```

---

## 71. Resource Class

Can reference Part 43 capacity class.

---

## 72. GPU Workspace

Explicit and quota-governed.

---

## 73. Cost Guard

Part 45.

---

## 74. Workspace TTL

```rust
pub struct WorkspaceTtl {
    pub idle_suspend_after: Duration,
    pub delete_after: Option<Duration>,
}
```

---

## 75. Idle Detection

Based on:

```text
active IDE connection
terminal session
foreground task
explicit keepalive
```

---

## 76. No CPU-Only Idle Heuristic

Critical.

Build/indexing may be active without UI.

---

## 77. Suspend

Stops compute, keeps allowed persistent state.

---

## 78. Resume

Rehydrates workspace from exact base/template + persistent state.

---

## 79. Resume Freshness

Can detect template/base update.

---

## 80. WorkspaceBaseFreshness

```rust
pub enum WorkspaceBaseFreshness {
    Current,
    TemplateChanged,
    BaseChanged,
    ToolchainChanged,
    SecurityUpdateRequired,
    Unknown,
}
```

---

## 81. Security Update Required

Can force rebuild/recreate.

---

## 82. Recreate

Preserve persistent volume but rebuild compute/base.

---

## 83. No Long-Lived Snowflake VM Requirement

Critical.

---

## 84. Workspace Drift

```rust
pub enum WorkspaceDriftClass {
    BasePackage,
    Toolchain,
    Service,
    SecuritySetting,
    UserCustomization,
    Unknown,
}
```

---

## 85. User Customization Drift

Usually allowed.

---

## 86. Base/Security Drift

May require recreate.

---

## 87. Workspace Baseline Trust

Lower than protected CI runner by default.

---

## 88. Developer Workspaces May Execute Untrusted Experimental Code

Therefore do not reuse them as high-trust signing/release workers.

---

## 89. Critical Separation

```text
developer workspace
    !=
trusted release runner
```

---

## 90. Workspace Agent

Dedicated `forgeyard-workspace-agent`.

---

## 91. Workspace Agent Responsibilities

```text
session lifecycle
port forwarding
file/snapshot integration
resource heartbeat
workspace metadata
```

---

## 92. Workspace Agent Does Not

```text
approve release
sign artifact
modify policy
```

---

## 93. Access Session

```rust
pub struct WorkspaceSessionId(Ulid);
```

---

## 94. WorkspaceSession

```rust
pub struct WorkspaceSession {
    pub id: WorkspaceSessionId,
    pub workspace: DeveloperWorkspaceId,
    pub principal: PrincipalId,
    pub method: WorkspaceAccessMethod,
    pub expires_at: Timestamp,
}
```

---

## 95. Access Methods

```rust
pub enum WorkspaceAccessMethod {
    Terminal,
    SshCompatible,
    IdeRemote,
    WebTerminal,
    FileSync,
    PortForward,
}
```

---

## 96. SSH-Compatible

Can be supported without permanent SSH key.

---

## 97. Short-Lived Session Certificate

Preferred.

---

## 98. No Shared Static SSH Key

Critical.

---

## 99. Session Authorization

Normal identity/authz.

---

## 100. MFA/Step-Up

Can be required for sensitive workspace/network profile.

---

## 101. Session Expiry

Bounded.

---

## 102. Session Revocation

Immediate where possible.

---

## 103. Workspace Console

Dioxus desktop/web client can launch terminal/session.

---

## 104. IDE Integration

Protocol adapters can support:

```text
VS Code Remote
JetBrains Gateway-like integration
SSH-based editor
Dioxus-native workspace UI
```

Forgeyard core remains IDE-neutral.

---

## 105. File Sync

Optional.

---

## 106. Remote-First

Repository checkout occurs in workspace.

---

## 107. Local File Sync

If supported, exact conflict semantics required.

---

## 108. No Hidden Two-Way Merge

Critical.

---

## 109. Source Control Remains Authority

Use VCS commits/source snapshots.

---

## 110. Port Forwarding

Part 59 network capability model.

---

## 111. WorkspacePortForwardId

```rust
pub struct WorkspacePortForwardId(Ulid);
```

---

## 112. Port Forward Grant

Bound to:

```text
workspace
principal
local/remote port
expiry
service
```

---

## 113. Public Exposure

Disabled by default.

---

## 114. Private Preview

Preferred.

---

## 115. Public Port

Explicit permission/policy.

---

## 116. No Wildcard All-Ports

Critical.

---

## 117. Network Policy

Developer workspace can have broader access than build sandbox, but still explicit.

---

## 118. WorkspaceNetworkPolicy

Examples:

```text
public development
private VPC development
air-gapped
restricted source-only
```

---

## 119. Private Resource Access

Part 59 connectors/tunnels.

---

## 120. Network Access Does Not Grant Service Authorization

Existing invariant.

---

## 121. Developer Secrets

Use `SecretRef`.

---

## 122. Workspace Secret Policy

```text
developer/dev/*
project/dev/*
preview/*
```

---

## 123. Production Secrets

Forbidden by default.

---

## 124. Production Access

Requires explicit privileged workspace profile.

---

## 125. Privileged Developer Workspace

High-risk, short-lived, audited.

---

## 126. Secret Delivery

Late-bound to process/session.

---

## 127. No Secret Baked Into Workspace Image

Critical.

---

## 128. Secret Persistence

Avoid writing plaintext into persistent volume.

---

## 129. Shell History

Secret redaction/usage guidance.

---

## 130. Environment Variables

Short-lived process scope.

---

## 131. Credential Broker

Can issue:

```text
SCM token
registry token
cloud dev token
database dev credential
```

---

## 132. Production Credential

Separate policy/step-up.

---

## 133. Cloud Workload Identity

Preferred over static API keys.

---

## 134. SCM Authentication

User delegated identity.

---

## 135. Git Push

Developer action, audited by SCM provider.

---

## 136. Workspace Does Not Push Automatically Without User Action

Baseline.

---

## 137. Source Snapshot Creation

```text
workspace tree
  ↓
canonical snapshot
  ↓
SourceSnapshotId
```

---

## 138. Run Local CI

```text
forgeyard run
```

Can execute locally or remote.

---

## 139. Remote CI From Workspace

Captures exact snapshot first.

---

## 140. No Live Mount From Workspace Into Runner

Critical.

---

## 141. Local Dev Environment

Part 35 parity.

---

## 142. Same Template

Can materialize:

```text
local developer environment
remote cloud workspace
```

---

## 143. Capability Differences

Explicit.

---

## 144. Example

Local macOS cannot emulate Linux kernel behavior perfectly.

---

## 145. Remote Linux Workspace

Can improve parity with Linux CI.

---

## 146. Dev Container Compatibility

Optional importer.

---

## 147. `.devcontainer`

Can be mapped to `WorkspaceTemplate`.

---

## 148. Nix/DevShell

Can interoperate with hermetic environment model.

---

## 149. No Hard Dependency

---

## 150. Prebuild Trigger

Can run on:

```text
default branch update
workspace template change
toolchain lock change
manual request
```

---

## 151. Prebuild Pipeline

Normal Forgeyard job.

---

## 152. Prebuild Artifact

Not a release artifact.

---

## 153. Prebuild Promotion

None.

---

## 154. Prebuild Selection

Closest exact source/template match.

---

## 155. Incremental Prebuild

May reuse CAS/cache.

---

## 156. Cache Correctness

Part 38.

---

## 157. Workspace Resume

Persistent working tree may be old.

---

## 158. VCS Status

Show divergence.

---

## 159. No Automatic Rebase/Merge

Critical.

---

## 160. Workspace Snapshot

```rust
pub struct DeveloperWorkspaceSnapshotId(Digest);
```

---

## 161. Snapshot Contents

Potential:

```text
working tree
selected persistent service snapshots
workspace metadata
```

---

## 162. Exclusions

```text
secrets
ephemeral credentials
system caches
```

---

## 163. Snapshot Use

Recovery/migration.

---

## 164. Snapshot Is Not CI Source Until Canonicalized

Critical.

---

## 165. Workspace Migration

Move between hosts/regions.

---

## 166. Migration Flow

```text
suspend
  ↓
snapshot persistent state
  ↓
verify
  ↓
provision destination
  ↓
restore
  ↓
resume
```

---

## 167. Region Placement

Part 51.

---

## 168. Placement Constraints

```text
data residency
source residency
private resource location
GPU availability
cost
latency
```

---

## 169. Hard Filters First

Existing scheduling rule.

---

## 170. Workspace Scheduler

Can reuse generic placement framework but separate from job scheduler authority.

---

## 171. Developer Preference

Soft score.

---

## 172. Cost

Part 45.

Track:

```text
compute hours
persistent storage
GPU time
network egress
prebuild cost
```

---

## 173. Workspace Budget

Per user/project/tenant.

---

## 174. Budget Limit

Can prevent new optional workspace or suspend idle one.

---

## 175. Do Not Delete Active Unsaved Work Due Budget Automatically

Critical.

---

## 176. Low Balance / Quota

Notify and suspend, preserving workspace where policy permits.

---

## 177. Deletion

Requires retention/TTL policy.

---

## 178. Persistent Volume Retention

Longer than compute.

---

## 179. Workspace Deletion State

```rust
pub enum WorkspaceDeletionState {
    Requested,
    Snapshotting,
    DestroyingCompute,
    DeletingVolumes,
    Completed,
    Failed,
}
```

---

## 180. User Confirmation

May be required when dirty/unpushed work exists.

---

## 181. Dirty Work Detection

VCS-aware.

---

## 182. Automated TTL Deletion

Can preserve recovery snapshot for grace period.

---

## 183. Recovery Window

Policy-defined.

---

## 184. No Permanent Hidden Backup

Lifecycle explicit.

---

## 185. Multi-Tenancy

Workspace resources tenant-scoped.

---

## 186. Cross-Tenant Host

Requires strong VM/container isolation.

---

## 187. High-Assurance Tenant

Dedicated host/project/account optional.

---

## 188. Quotas

Part 27.

Examples:

```text
max active workspaces
max CPUs
max GPUs
max persistent storage
max prebuilds
```

---

## 189. Fairness

Workspace capacity should not starve CI runners if shared infrastructure.

---

## 190. Capacity Partition

Recommended.

---

## 191. Priority

CI/release can have higher protected resource priority.

---

## 192. No Workspace GPU Hoarding

Lease/idle policies.

---

## 193. Suspension

Can release compute.

---

## 194. Persistent Background Task

If user intentionally runs long task, workspace not idle.

---

## 195. Long-Running Work

Better moved to normal Forgeyard job if reproducibility/evidence matters.

---

## 196. "Promote to Job"

Feature.

---

## 197. Promote Command

```text
forgeyard workspace run-as-job
```

Captures source + command/environment into normal JobSpec.

---

## 198. No Hidden Transfer of Process State

Critical.

---

## 199. Workspace Security Boundary

Remote workspace contains user code and credentials.

---

## 200. Threats

```text
workspace escape
cross-user access
secret persistence
stale credentials
public port exposure
malicious editor extension
supply-chain plugin
privilege escalation
persistent malware
```

---

## 201. Isolation

VM strongest baseline for untrusted multi-tenant remote workspaces.

---

## 202. Container

Allowed with appropriate trust/risk.

---

## 203. Host User Namespace

Lower assurance.

---

## 204. WorkspaceIsolationClass

```rust
pub enum WorkspaceIsolationClass {
    Vm,
    Container,
    Process,
    DedicatedHost,
}
```

---

## 205. Policy

Maps tenant/project to minimum isolation.

---

## 206. Privileged Container

Forbidden baseline.

---

## 207. Docker Socket

Not exposed baseline.

---

## 208. Nested Build Containers

Use rootless/container sandbox where supported.

---

## 209. Kernel Build/VM Dev

May require dedicated host/VM profile.

---

## 210. Workspace Root

Developer may have root inside disposable VM/container.

---

## 211. Host Root

Never.

---

## 212. Root Inside Workspace

Does not imply production credential access.

---

## 213. Editor Extensions

Untrusted user software.

---

## 214. Extension Marketplace

Optional.

---

## 215. Enterprise Policy

May allowlist extensions.

---

## 216. Workspace Agent Privilege

Minimal.

---

## 217. Control Plane Credential

Agent mTLS identity only.

---

## 218. No DB Credential

Critical.

---

## 219. Workspace Access Logging

Record:

```text
session start
session end
port-forward grant
privileged profile activation
```

---

## 220. Command Logging

Not baseline.

Developer privacy.

---

## 221. Audit

High-risk events only.

---

## 222. No Keystroke Recording

Critical.

---

## 223. Terminal Content

Not captured by default.

---

## 224. Enterprise Session Recording

Could be optional separate policy for privileged workspace, but explicit and visible.

---

## 225. Privacy

Developer workspace is more personal than build runner.

---

## 226. File Indexing

Tenant-scoped.

---

## 227. AI Assistance

Part 55 can be available inside workspace.

---

## 228. AI Context

Normal project policy.

---

## 229. AI Does Not Gain Workspace Shell Automatically

Use capability broker.

---

## 230. IDE Assistant

Can draft commands/code.

---

## 231. No Auto-Execute Privileged Command

Existing invariant.

---

## 232. Observability

Operational metrics:

```text
dev_workspaces_active
dev_workspace_start_seconds
dev_workspace_suspend_total
dev_workspace_resume_total
dev_workspace_failures_total
dev_workspace_idle_seconds
dev_workspace_prebuild_hit_total
```

---

## 233. Labels

Low cardinality:

```text
provider
platform
state
isolation_class
```

---

## 234. No user identity in aggregate metric labels.

---

## 235. Tracing

```text
workspace.provision
workspace.start
workspace.session
workspace.suspend
workspace.resume
workspace.snapshot
workspace.destroy
workspace.prebuild
```

---

## 236. Health

```rust
pub enum DeveloperWorkspaceSubsystemHealth {
    Healthy,
    ProvisioningDegraded,
    SessionDegraded,
    StorageDegraded,
    ProviderDegraded,
    Unhealthy,
}
```

---

## 237. Doctor

```text
forgeyard workspace doctor
```

Checks:

```text
orphan workspaces
stale sessions
volume attachment errors
security update required
public ports
expired credentials
prebuild backlog
```

---

## 238. Dioxus UI

Pages:

```text
Workspaces
Workspace Templates
Prebuilds
Developer Volumes
Workspace Sessions
Workspace Costs
```

---

## 239. Workspace Card

Shows:

```text
project
branch/revision
state
platform
resources
idle time
cost
```

---

## 240. Workspace Detail

Shows:

```text
exact base/template
source status
services
ports
sessions
persistent volumes
network policy
freshness
```

---

## 241. Create Workspace UX

Select:

```text
project
revision
template
region
resource class
```

---

## 242. Defaults

Organization/project templates.

---

## 243. "Open in IDE"

Generates bounded session grant.

---

## 244. CLI

```text
forgeyard workspace create
forgeyard workspace list
forgeyard workspace open
forgeyard workspace ssh
forgeyard workspace port-forward
forgeyard workspace suspend
forgeyard workspace resume
forgeyard workspace snapshot
forgeyard workspace recreate
forgeyard workspace delete
forgeyard workspace doctor
```

---

## 245. API

Potential:

```text
POST /v1/workspaces
GET  /v1/workspaces
GET  /v1/workspaces/{id}
POST /v1/workspaces/{id}/sessions
POST /v1/workspaces/{id}/suspend
POST /v1/workspaces/{id}/resume
POST /v1/workspaces/{id}/snapshot
DELETE /v1/workspaces/{id}
```

---

## 246. Permissions

```text
workspace.read
workspace.create
workspace.use
workspace.port_forward
workspace.public_port
workspace.privileged_profile
workspace.admin
```

---

## 247. Privileged Profile

High risk.

---

## 248. Public Port

Separate permission.

---

## 249. Workspace Admin

Cannot read arbitrary developer files by default.

---

## 250. Admin Operations

Can:

```text
suspend
recreate
quarantine
delete according policy
```

---

## 251. Support Access

Requires explicit temporary grant.

---

## 252. No Silent Admin Shell

Critical.

---

## 253. Audit

Audit:

```text
workspace privileged profile
public port exposure
support access
admin quarantine
persistent volume export
production resource grant
```

---

## 254. Routine Session

Operational event.

---

## 255. Data Lifecycle

Part 46 governs:

```text
workspace metadata
persistent volumes
snapshots
session metadata
prebuild artifacts
```

---

## 256. Source Worktree

Developer-owned data.

---

## 257. Workspace Deletion

Should warn about unpushed commits/dirty work.

---

## 258. Snapshot Retention

Explicit.

---

## 259. Legal Hold

Can apply where enterprise policy requires.

---

## 260. Secret Deletion

Secrets external; workspace only holds ephemeral delivery.

---

## 261. Backup

Persistent developer volumes can be backed up if product promises it.

---

## 262. Backup Policy

Explicit.

---

## 263. Do Not Imply Backup If None

Critical.

---

## 264. DR

Workspace compute is disposable.

Persistent volume/snapshot recovery depends on configured durability.

---

## 265. After Region Failure

Recreate workspace elsewhere from:

```text
VCS
WorkspaceTemplateId
persistent volume backup/snapshot
```

---

## 266. Dirty Unsynced Work

May be lost if persistent volume lacked durability.

---

## 267. Honest RPO

Critical.

---

## 268. Federation

Workspace placement respects:

```text
residency
private-resource location
source policy
site trust
cost
```

---

## 269. Disconnected Site

Local remote workspace can operate against local source/cache.

---

## 270. Reconnect

Normal VCS/source synchronization.

---

## 271. Air-Gap

Workspace uses local:

```text
SCM
registry mirror
toolchains
dependency mirror
AI model if any
```

---

## 272. No Public Network

Machine enforced.

---

## 273. Runner Image Factory

Part 58 can provide workspace base image.

---

## 274. Different Trust Profile

Workspace image can include developer tools excluded from release runner.

---

## 275. Network

Part 59.

Per-workspace network policy.

---

## 276. Concurrency

Part 60 can manage scarce workspace resources.

---

## 277. Incident Management

Part 61.

Workspace subsystem outage can become incident.

---

## 278. Progressive Delivery

No direct authority.

---

## 279. Database Migration

Developer can test migration in dev DB, but production migration remains Part 63.

---

## 280. Service Catalog

Part 49 can offer "Create Workspace" for component.

---

## 281. Golden Paths

Part 42 can provide standard workspace template.

---

## 282. Compatibility

Part 57 can gate workspace base/toolchain compatibility.

---

## 283. Update Delivery

Workspace agent/base updates through normal controlled mechanisms.

---

## 284. Prebuild Reconciliation

Desired prebuild vs available cache/artifact.

---

## 285. Workspace Reconciler

Checks:

```text
provider state
session state
volume attachments
TTL/idle
base freshness
security revocation
```

---

## 286. Unknown Provider State

No blind duplicate action.

---

## 287. HA

Multiple controllers safe.

---

## 288. Workspace Ownership Lease

Control operation lease.

---

## 289. No One Global Workspace Lock

---

## 290. Security Quarantine

Workspace can be quarantined on:

```text
malware finding
credential compromise
policy violation
host drift
```

---

## 291. Quarantine

Revokes sessions/network, preserves evidence/state where policy requires.

---

## 292. User Notification

Explicit.

---

## 293. Workspace Rebuild

Preferred remediation.

---

## 294. Testkit

```text
forgeyard-dev-workspace-testkit/src/
├── lib.rs
├── template.rs
├── provision.rs
├── prebuild.rs
├── session.rs
├── volume.rs
├── network.rs
├── snapshot.rs
└── assertions.rs
```

---

## 295. Core Tests

### Identity
- template identity deterministic;
- mutable base alias resolves exact image.

### Source
- dirty workspace run captures immutable SourceSnapshotId;
- runner never reads live workspace filesystem.

### Prebuild
- prebuild cache miss still works;
- secret-bearing prebuild is not globally reusable.

### Sessions
- expired session denied;
- static shared SSH key absent;
- public port requires explicit permission.

### Secrets
- production secret unavailable in standard dev workspace;
- no secret baked into image/snapshot.

### Volumes
- cross-user mount denied;
- delete warns on dirty/unpushed work.

### Drift
- base/security drift triggers recreate requirement;
- user shell customization does not corrupt template identity.

### Cost/TTL
- idle suspend preserves data;
- budget pressure never silently deletes active dirty work.

### Federation
- residency blocks disallowed region;
- air-gap blocks public egress.

### Security
- workspace cannot act as release/signing runner.

---

## 296. Chaos Tests

Inject:

```text
cloud VM provision timeout
volume attach failure
workspace-agent crash
region outage
session gateway outage
prebuild service outage
```

Expected:

```text
workspace state remains explicit
no duplicate instance creation
persistent data durability follows declared policy
sessions recover or fail cleanly
```

---

## 297. Scale Tests

Test:

```text
thousands of active workspaces
large prebuild fanout
large monorepo checkout/index
high concurrent session count
large persistent volume fleet
```

---

## 298. Implementation Phases

### Phase 1 — Workspace Model & Local Provider
Establish canonical semantics.

### Phase 2 — Remote Linux Workspace
Cloud/on-prem VM/container.

### Phase 3 — Sessions & Port Forwarding
Secure access.

### Phase 4 — Persistent Volumes & Suspend/Resume
Developer continuity.

### Phase 5 — Prebuild System
Startup acceleration.

### Phase 6 — Dev Services/TestEnv Integration
Local parity.

### Phase 7 — Cost/TTL/Quota
Hosted operation.

### Phase 8 — Private Network/Secrets
Enterprise development.

### Phase 9 — Federation/Air-Gap
Distributed organizations.

### Phase 10 — Windows/macOS/Specialized Workspaces
Platform expansion.

### Phase 11 — Dioxus UI/IDE Integrations
Developer experience.

### Phase 12 — Security/Chaos/Scale Hardening
Production readiness.

---

## 299. Acceptance Tests

1. Every workspace has immutable WorkspaceTemplateId.
2. Workspace source revision resolves to exact SourceSnapshotId.
3. Dirty workspace CI captures explicit immutable snapshot.
4. CI never builds directly from live mutable workspace filesystem.
5. Workspace base image is exact/digest-bound.
6. User customization is separate from reproducible base identity.
7. Prebuilds are optional acceleration only.
8. Prebuild failure does not make workspace fundamentally unusable.
9. Secret-bearing prebuilds are not shared broadly.
10. Developer persistent volumes are tenant/user/project scoped.
11. Cross-user/cross-tenant mounts are denied by default.
12. Production data is not present by default.
13. Production secrets are denied in standard workspace profile.
14. Credentials are late-bound and short-lived.
15. No static shared SSH key is required.
16. Port forwards are explicit and expiring.
17. Public port exposure requires separate permission.
18. Workspace network reachability does not imply service authorization.
19. Workspace compute can suspend independently from persistent state.
20. Security/base drift can require workspace recreation.
21. Remote workspace is never automatically trusted as release/signing runner.
22. Long-running reproducible work can be promoted into a normal JobSpec.
23. Budget/TTL policy cannot silently delete active dirty work.
24. Workspace deletion warns/protects unpushed work according policy.
25. Workspace snapshots exclude secrets by design.
26. DR behavior reflects actual persistent-volume durability/RPO.
27. Federation placement obeys residency.
28. Air-gapped workspaces run without public egress.
29. Admin/support access is explicit and audited.
30. Forgeyard dogfoods remote workspaces for its own development.

---

## 300. Production Readiness Gates

Do not call remote workspaces production-ready until:

```text
workspace/source identity is stable
dirty-source snapshotting works
session credentials are short-lived
persistent volume isolation is proven
production-secret denial is enforced
public port controls are machine-enforced
prebuild isolation/cache rules pass
suspend/resume and provider Unknown reconciliation work
DR/RPO behavior is documented and tested
security/chaos/scale tests pass
```

---

## 301. Architectural Invariants

1. workspace base is reproducible;
2. user mutable layer is separate;
3. branch name is not source truth;
4. dirty CI uses explicit snapshot;
5. CI never consumes live workspace filesystem;
6. prebuild is acceleration only;
7. persistent state is explicit;
8. caches are disposable;
9. secrets are not baked into workspace image/snapshot;
10. standard workspace lacks production secrets;
11. session credentials are short-lived;
12. no static shared SSH key baseline;
13. port forwards are scoped/expiring;
14. network reachability is not authorization;
15. workspace can suspend independently from persistent state;
16. base/security drift affects freshness;
17. developer workspace is not trusted release runner;
18. workspace process state is never "promoted" into CI;
19. reproducible long work becomes a normal JobSpec;
20. user/admin access boundaries are explicit;
21. no silent session/keystroke recording baseline;
22. public exposure requires explicit permission;
23. tenant/user volume isolation is enforced;
24. quota/cost policies preserve dirty work safely;
25. data lifecycle governs snapshots/volumes;
26. DR promises match actual persistence durability;
27. federation obeys residency;
28. air-gap blocks public egress;
29. provider effects are reconciled after ambiguity;
30. Forgeyard dogfoods its own remote workspace system.

---

## 302. Final Target Architecture

```text
                  WorkspaceTemplateId
                          │
                          ▼
                    Workspace Base
                          │
                          ▼
                     Prebuild Layer
                          │
                          ▼
                 DeveloperWorkspaceId
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Terminal       IDE       Port Forward
             │            │            │
             └────────────┼────────────┘
                          ▼
                   Mutable Worktree
                          │
                          ▼
                 Explicit SourceSnapshotId
                          │
                          ▼
                     Normal CI
```

State separation:

```text
Immutable base
    +
Project/template layer
    +
Persistent developer volume
    +
Disposable cache
    +
Ephemeral credentials
```

CI boundary:

```text
developer workspace
      ↓
capture exact source snapshot
      ↓
normal Forgeyard pipeline
      ↓
trusted runners/evidence
      ↓
release
```

The key guarantee is:

> **Forgeyard can offer fast, persistent, cloud-hosted development environments without weakening its local-first and reproducible CI architecture. Developers keep mutable, personalized workspaces for productivity, while every transition into CI/release crosses an explicit immutable snapshot boundary and re-enters the normal governed Forgeyard pipeline.**

---

## 303. Extended Architecture Sequence

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
12 Secrets / Trust
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
26 Self-Hosting / Bootstrap / Release Forgeyard
27 Multi-Tenancy / Quotas / Resource Governance
28 Audit / Compliance / Security Governance
29 Notifications / Alerting / Human Workflow
30 Entitlements / Licensing / Subscription
31 Search / Indexing / Operational Analytics
32 Test Results / Quality / Coverage / Flaky Intelligence
33 Benchmark / Performance / Load / Capacity
34 Monorepo / Dependency Graph / Affected Work
35 Developer Experience / Local Dev
36 Dependency / Registry / Mirror Governance
37 Static Analysis / Security Findings
38 Cache / Remote Cache / Correctness
39 Configuration / Feature Flags / Runtime
40 Security / Threat Model / Incident Response
41 Release Distribution / Update Delivery
42 Workflow Templates / Golden Paths
43 Runner Fleet Autoscaling / Provisioning
44 Pipeline Triggers / Schedules / Dispatch
45 Cost Accounting / FinOps
46 Data Lifecycle / Retention / Privacy
47 CI/CD Migration / Compatibility
48 Failure Diagnosis / Reproduction / Bisect
49 Service Catalog / Ownership / Developer Portal
50 Reliability / SLO / Error Budget / Resilience
51 Multi-Region Federation / Edge / Disconnected
52 Artifact Registry / OCI / Package Distribution
53 Infrastructure-as-Code / Preview Environments / Drift
54 Merge Queue / Speculative Integration / Batch Validation
55 AI-Assisted CI Optimization / Engineering Copilot
56 Test Data / Fixtures / Ephemeral Databases / Service Virtualization
57 API / ABI / Schema / Protocol Compatibility / Contract Evolution
58 Runner Image Factory / Golden Images / Patch Management / Baseline Attestation
59 Network Connectivity / Private Resource Access / Egress / Tunneling / Zero-Trust Connectivity
60 Workflow Concurrency / Distributed Locks / Idempotency / Reservations / Exclusive Coordination
61 Incident Management / On-Call / Escalation / Response Coordination / Postmortem
62 Environment Promotion / Progressive Delivery / Feature Rollout / Canary Analysis / Automated Rollback
63 Database Schema Migration / Online Backfill / Data Transformation / Zero-Downtime Change Orchestration
64 Remote Development Environments / Cloud Workspaces / Developer Workspace Orchestration
```
