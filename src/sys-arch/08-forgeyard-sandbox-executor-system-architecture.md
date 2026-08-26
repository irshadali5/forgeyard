# 08 — Forgeyard Sandbox & Executor System Architecture

**Document type:** Core Execution Isolation System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Sandbox lifecycle, executor abstraction, process/container/VM execution, Linux/Windows/macOS isolation, filesystem/network/process/resource controls, privileged helper boundaries, cancellation, timeout, cleanup, device exposure, confidential execution, and execution security  
**Architecture style:** Platform-neutral executor contract with platform-specific isolation backends, explicit capability negotiation, deny-by-default sandbox policy, and defense-in-depth resource/security enforcement  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on `07-forgeyard-runner-agent-system-architecture.md`. It consumes `ExecutionRequest`, `AttemptContext`, `JobSpecId`, resource reservations, network policy, workspace paths, and secret references from upstream systems. It provides controlled execution handles/results back to the runner agent.

---

# 1. Purpose

Forgeyard executes untrusted or semi-trusted user workloads.

This is the most security-sensitive runtime boundary in the entire platform.

The executor/sandbox subsystem must answer:

```text
how do we run this job
without letting it own the host?
```

The central rule is:

> **The Forgeyard agent is trusted; the workload is not. The sandbox/executor boundary must enforce that separation.**

A second rule is:

> **Isolation strength is an explicit capability of the selected executor. Forgeyard must never silently claim stronger isolation than the platform/backend actually provides.**

A third rule is:

> **Execution semantics are portable, but isolation mechanisms are platform-specific.**

---

# 2. Architectural Position

```text
                Forgeyard Agent
                      │
                      ▼
               ExecutionRequest
                      │
                      ▼
                  Executor API
                      │
       ┌──────────────┼───────────────┐
       ▼              ▼               ▼
    Process        Container          VM
       │              │               │
       └──────────────┼───────────────┘
                      ▼
                Sandbox Backend
          ┌───────────┼───────────┐
          ▼           ▼           ▼
        Linux       Windows      macOS
          │           │           │
          ▼           ▼           ▼
      untrusted workload execution
```

---

# 3. Goals

The subsystem MUST:

1. provide stable executor traits;
2. provide stable sandbox policy models;
3. isolate workload filesystem;
4. isolate processes;
5. isolate credentials;
6. isolate network according to policy;
7. enforce resource limits;
8. enforce process-tree cleanup;
9. enforce timeout/cancellation;
10. expose stdout/stderr streams;
11. expose structured exit status;
12. support Linux;
13. support Windows;
14. support macOS;
15. support Android-related host tooling where required;
16. support process execution;
17. support container execution;
18. support stronger VM isolation later;
19. support confidential execution later;
20. expose isolation capability honestly;
21. support secure privileged helper patterns;
22. support device attachment;
23. support readonly toolchain/source mounts;
24. support writable build/output paths;
25. prevent agent credential exposure;
26. remain independent from scheduler/business policy;
27. support deterministic/hermetic mode;
28. support developer/impure mode separately;
29. expose health/doctor capabilities;
30. be fuzzed/tested as a security boundary.

---

# 4. Non-Goals

This subsystem does not:

```text
schedule jobs
compile PipelineIr
store metadata
resolve VCS
approve secrets
perform release promotion
```

It executes already-authorized planned work.

---

# 5. Workspace Structure

```text
crates/sandbox/
├── forgeyard-sandbox/
├── forgeyard-sandbox-model/
├── forgeyard-sandbox-policy/
├── forgeyard-sandbox-linux/
├── forgeyard-sandbox-windows/
├── forgeyard-sandbox-apple/
├── forgeyard-sandbox-container/
├── forgeyard-sandbox-vm/
├── forgeyard-sandbox-network/
├── forgeyard-sandbox-filesystem/
├── forgeyard-sandbox-resource/
├── forgeyard-sandbox-device/
├── forgeyard-sandbox-helper/
├── forgeyard-sandbox-health/
└── forgeyard-sandbox-testkit/
```

Executor crates:

```text
crates/executor/
├── forgeyard-executor/
├── forgeyard-executor-model/
├── forgeyard-executor-process/
├── forgeyard-executor-container/
├── forgeyard-executor-linux/
├── forgeyard-executor-windows/
├── forgeyard-executor-apple/
├── forgeyard-executor-vm/
├── forgeyard-executor-confidential/
├── forgeyard-executor-cancel/
├── forgeyard-executor-timeout/
├── forgeyard-executor-output/
└── forgeyard-executor-testkit/
```

---

# 6. Executor Contract

```rust
#[async_trait]
pub trait Executor: Send + Sync {
    async fn prepare(
        &self,
        request: &ExecutionRequest,
    ) -> Result<PreparedExecution, ExecutorError>;

    async fn start(
        &self,
        prepared: PreparedExecution,
    ) -> Result<ExecutionHandle, ExecutorError>;
}
```

Execution handle:

```rust
#[async_trait]
pub trait ExecutionHandle: Send {
    async fn wait(&mut self) -> Result<ExecutionResult, ExecutorError>;
    async fn cancel(&mut self, reason: CancellationReason) -> Result<(), ExecutorError>;
}
```

---

# 7. ExecutionRequest

```rust
pub struct ExecutionRequest {
    pub context: AttemptContext,
    pub spec: PlannedJobSpec,
    pub workspace: PreparedWorkspace,
    pub resources: ResourceReservation,
    pub sandbox: SandboxPolicy,
    pub network: NetworkPolicy,
    pub environment: RuntimeEnvironment,
}
```

---

# 8. PreparedExecution

Represents a fully validated local execution plan before workload process starts.

```rust
pub struct PreparedExecution {
    pub sandbox_id: SandboxId,
    pub context: AttemptContext,
    pub command: PreparedCommand,
    pub environment: PreparedEnvironment,
    pub filesystem: FilesystemPlan,
    pub resource_limits: ResourceLimits,
}
```

---

# 9. SandboxId

```rust
pub struct SandboxId(Ulid);
```

Local operational identity.

Not global business identity.

---

# 10. Executor Types

```rust
pub enum ExecutorKind {
    Process,
    Container,
    VirtualMachine,
    Confidential,
}
```

---

# 11. Process Executor

Uses host kernel/process isolation features.

Best for:

```text
fast builds
local development
Linux namespace sandbox
Windows Job Object/token sandbox
macOS constrained execution
```

---

# 12. Container Executor

Uses OCI/container runtime semantics.

Good for:

```text
stronger dependency isolation
portable Linux images
service dependencies
```

But Forgeyard should not require Docker specifically.

---

# 13. VM Executor

Uses:

```text
microVM
QEMU/KVM
Hyper-V
Virtualization.framework
```

for stronger isolation.

---

# 14. Confidential Executor

Future high-assurance mode:

```text
SEV-SNP
TDX
confidential VM
```

Not correctness dependency for general Forgeyard.

---

# 15. Isolation Level

```rust
pub enum IsolationLevel {
    None,
    Process,
    Namespace,
    Container,
    VirtualMachine,
    ConfidentialVirtualMachine,
}
```

---

# 16. Isolation Capability

Runner reports maximum supported isolation.

Scheduler can require minimum level.

---

# 17. No Silent Downgrade

If job requires:

```text
VirtualMachine
```

and runner only supports:

```text
Namespace
```

runner is ineligible.

---

# 18. Sandbox Policy

```rust
pub struct SandboxPolicy {
    pub isolation: IsolationRequirement,
    pub filesystem: FilesystemPolicy,
    pub network: NetworkPolicy,
    pub process: ProcessPolicy,
    pub devices: DevicePolicy,
    pub privileges: PrivilegePolicy,
}
```

---

# 19. Filesystem Policy

```rust
pub struct FilesystemPolicy {
    pub source_read_only: bool,
    pub toolchains_read_only: bool,
    pub allow_host_paths: Vec<ApprovedHostPath>,
    pub writable_paths: Vec<SandboxPath>,
    pub tmp_size: ByteSize,
}
```

---

# 20. Default Filesystem Layout

Inside sandbox:

```text
/work/source      read-only
/work/build       writable
/work/output      writable
/work/tmp         writable
/toolchains/...   read-only
```

---

# 21. Host Path Exposure

Deny by default.

Only explicit approved mounts.

---

# 22. Never Mount Agent State

Do not expose:

```text
agent config
agent credentials
local metadata
control sockets
other workspaces
```

---

# 23. Source Read-Only

Strict/hermetic mode:

```text
source tree read-only
```

---

# 24. Build Directory

Writable scratch/build state.

---

# 25. Output Directory

Writable, captured after execution.

---

# 26. Tmp

Per-attempt.

Never shared system `/tmp` without namespace/isolation.

---

# 27. Home Directory

Synthetic:

```text
/home/forgeyard
```

or platform equivalent.

---

# 28. Synthetic HOME

Prevents leakage of:

```text
SSH keys
npm config
cargo credentials
cloud credentials
```

---

# 29. PATH

Explicit.

Constructed from managed toolchains + approved system runtime.

---

# 30. Environment Sanitization

Default deny inheritance.

Allow explicit variables only.

---

# 31. Secret Injection

Secrets can enter:

```text
environment
file
stdin
platform secret mechanism
```

according to secret architecture.

---

# 32. Secret File

Must be:

```text
sandbox-private
short-lived
permissions-restricted
cleaned
```

---

# 33. Secret Mount

Prefer tmpfs/in-memory where platform supports.

---

# 34. Process Isolation

At minimum:

```text
separate process group/job
child process ownership
kill tree
```

---

# 35. Linux Process Isolation

Potential:

```text
PID namespace
mount namespace
user namespace
network namespace
UTS namespace
IPC namespace
cgroup v2
seccomp
capability drop
no_new_privs
```

---

# 36. Linux Preferred Baseline

For production untrusted workload:

```text
bubblewrap/namespaces
+
cgroup v2
+
seccomp
+
capability drop
+
read-only mounts
```

where available.

---

# 37. Bubblewrap

Useful implementation mechanism.

But architecture should depend on `SandboxBackend`, not bubblewrap itself.

---

# 38. User Namespace

Can improve rootless isolation.

Availability varies by host policy.

---

# 39. Mount Namespace

Mandatory for meaningful filesystem isolation.

---

# 40. PID Namespace

Prevents visibility/control of unrelated host processes.

---

# 41. Network Namespace

Used when network policy requires.

---

# 42. UTS Namespace

Optional hostname isolation.

---

# 43. IPC Namespace

Prevents host IPC exposure.

---

# 44. Cgroup v2

Use for:

```text
CPU
memory
PIDs
IO where supported
```

---

# 45. CPU Limit

```text
cpu.max
```

or equivalent.

---

# 46. Memory Limit

Hard limit where supported.

---

# 47. PIDs Limit

Important defense against fork bombs.

---

# 48. OOM Detection

Map cgroup OOM to typed execution failure.

---

# 49. Seccomp

Deny dangerous/unnecessary syscalls according to profile.

---

# 50. Seccomp Profile

Versioned by Forgeyard.

---

# 51. No Generic "Block Everything"

Profiles must remain compatible with supported toolchains.

---

# 52. Syscall Policy Classes

Potential:

```text
general build
container build
networked test
browser test
```

---

# 53. Linux Capabilities

Drop all by default.

Add only explicit needed capability.

---

# 54. `no_new_privs`

Enable for untrusted workload.

---

# 55. Setuid

Block where feasible.

---

# 56. Proc Filesystem

Mount constrained `/proc` inside namespace.

---

# 57. Sysfs

Do not expose broadly.

---

# 58. Device Nodes

Minimal.

---

# 59. Docker Socket

Never exposed by default.

---

# 60. Nested Containers

If supported, use dedicated executor/pool with explicit policy.

---

# 61. Privileged Container

Not allowed for ordinary untrusted jobs.

---

# 62. Windows Isolation

Use platform-appropriate mechanisms:

```text
Job Objects
restricted tokens
ACLs
AppContainer where practical
Windows Sandbox/Hyper-V for stronger mode
process mitigation policies
```

---

# 63. Windows Job Object

Own process tree and enforce:

```text
CPU
memory
process count
kill-on-close
```

---

# 64. Restricted Token

Remove unnecessary privileges.

---

# 65. Filesystem ACL

Workspace accessible only to execution identity/service boundary.

---

# 66. Windows Network Isolation

Can use:

```text
Windows Firewall rules
AppContainer/network restrictions
VM mode
```

depending isolation requirement.

---

# 67. Windows Strong Isolation

For high assurance:

```text
Hyper-V isolated VM/container
```

---

# 68. macOS Isolation

macOS has different primitives.

Possible:

```text
dedicated user
sandbox-exec where available/appropriate
filesystem permissions
process groups
Virtualization.framework VM for stronger isolation
```

---

# 69. macOS Reality

Do not claim Linux-equivalent namespace isolation if not available.

Report actual isolation capability honestly.

---

# 70. macOS Strong Isolation

Use VM-backed executor for high-risk untrusted workloads when required.

---

# 71. Apple Toolchain Access

Xcode/SDK can be mounted/read-only or accessed from host-managed installation.

---

# 72. Simulators

Simulator resources should be scoped to attempt/device subsystem.

---

# 73. Android Host Tooling

Android build jobs on Linux/Windows/macOS use normal executor plus Android SDK/NDK.

Physical device access is separate device policy.

---

# 74. Device Policy

```rust
pub struct DevicePolicy {
    pub allowed_devices: Vec<DeviceLeaseId>,
    pub usb: UsbAccessPolicy,
    pub gpu: GpuAccessPolicy,
}
```

---

# 75. Device Exposure

Only leased device(s) visible.

---

# 76. USB Isolation

Platform-specific.

Do not expose all USB devices to arbitrary job.

---

# 77. GPU Exposure

Only allocated GPU/partition.

---

# 78. GPU Driver

Host driver may be shared.

Container/VM execution must validate compatibility.

---

# 79. Network Policy

Canonical:

```rust
pub enum NetworkPolicy {
    Deny,
    FetchOnly,
    Restricted(EgressPolicy),
    Allow,
}
```

---

# 80. `Deny`

No network interface except minimal loopback if needed.

---

# 81. `FetchOnly`

Used for dependency resolution/fetch phases.

Can restrict destinations through fetch proxy rather than arbitrary egress.

---

# 82. Restricted Egress

```rust
pub struct EgressPolicy {
    pub destinations: Vec<NetworkDestination>,
    pub dns: DnsPolicy,
}
```

---

# 83. DNS

Network deny means no external DNS.

---

# 84. Fetch Proxy

Preferred for highly controlled hermetic dependency fetching.

---

# 85. Proxy Benefits

```text
allowlists
audit
cache
credential isolation
```

---

# 86. Host Network

Forbidden for ordinary strict jobs.

---

# 87. Service Networking

Job-scoped service network.

---

# 88. Multi-Container Job

Could create isolated per-job virtual network.

---

# 89. Cross-Job Networking

Forbidden by default.

---

# 90. Loopback

Allowed if needed for test services inside same sandbox scope.

---

# 91. Port Exposure

No public host ports by default.

---

# 92. Inbound Network

Deny by default.

---

# 93. Resource Policy

```rust
pub struct ResourceLimits {
    pub cpu: CpuLimit,
    pub memory: MemoryLimit,
    pub disk: DiskLimit,
    pub pids: PidLimit,
    pub wall_time: Duration,
}
```

---

# 94. Scheduler Reservation vs Sandbox Limit

Scheduler reserves capacity.

Sandbox enforces hard runtime limits.

---

# 95. Defense in Depth

If scheduler overcommits by bug:

sandbox still prevents one job from exceeding its limit.

---

# 96. Disk Enforcement

Possible:

```text
filesystem quota
loopback volume
directory accounting + watchdog
VM disk limit
```

platform-specific.

---

# 97. Disk Watchdog

Fallback if true quota unavailable.

---

# 98. PIDs

Always enforce where possible.

---

# 99. File Count

Optional output/workspace safety limit.

---

# 100. Open File Limit

Set per workload.

---

# 101. Core Dump Limit

Disabled or restricted by default.

---

# 102. Ulimit/Rlimit

Unix process executor can enforce:

```text
nofile
core
fsize
nproc where useful
```

---

# 103. Timeout

Executor receives absolute/relative execution deadline.

---

# 104. Timeout Flow

```text
timer expires
  ↓
soft terminate
  ↓
grace
  ↓
hard kill
  ↓
cleanup
  ↓
ExecutionResult::TimedOut
```

---

# 105. Cancellation

Same kill sequence but different reason.

---

# 106. Cancellation Reasons

```rust
pub enum CancellationReason {
    User,
    RunCancelled,
    Superseded,
    FailFast,
    LeaseExpired,
    RunnerDrain,
    Shutdown,
}
```

---

# 107. Timeout vs Cancellation

Never conflate.

---

# 108. Grace Period

Configurable per workload class.

---

# 109. Hard Kill

Mandatory fallback.

---

# 110. Process Tree Cleanup

Must terminate all descendants.

---

# 111. Detached Child Defense

Use process group/job/cgroup ownership.

---

# 112. Linux Cgroup Kill

Useful:

```text
cgroup.kill
```

where available.

---

# 113. Windows Kill-on-Job-Close

Useful process tree guarantee.

---

# 114. VM Kill

Destroy VM.

---

# 115. Cleanup Verification

After termination, check no workload-owned processes remain.

---

# 116. Executor Result

```rust
pub struct ExecutionResult {
    pub outcome: ExecutionOutcome,
    pub exit: Option<ProcessExit>,
    pub usage: ResourceUsage,
    pub started_at: Timestamp,
    pub finished_at: Timestamp,
}
```

---

# 117. ExecutionOutcome

```rust
pub enum ExecutionOutcome {
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    OutOfMemory,
    ResourceExceeded,
    SandboxViolation,
    InfrastructureFailure,
}
```

---

# 118. Exit Code

Workload exit code remains separate from executor outcome.

---

# 119. Signal

Unix signal modeled portably.

---

# 120. Windows Termination

Use typed platform detail extension.

---

# 121. Resource Usage

```rust
pub struct ResourceUsage {
    pub cpu_time: Duration,
    pub peak_memory: ByteSize,
    pub disk_written: ByteSize,
    pub io_read: ByteSize,
    pub io_written: ByteSize,
}
```

Fields optional by platform support.

---

# 122. Usage Is Observational

Never rely on exact resource accounting if platform cannot guarantee.

---

# 123. Sandbox Violation

Examples:

```text
forbidden syscall
path escape
network policy violation
device access violation
```

---

# 124. Violation Handling

Terminate workload.

Report typed security failure.

---

# 125. Security Event

Sandbox violations should be auditable.

---

# 126. Executor Preparation

Before start, verify:

```text
paths
mount plan
network mode
resource limits
command
environment
secret mounts
device leases
```

---

# 127. No TOCTOU Where Avoidable

Resolve/lock critical filesystem objects before executing.

---

# 128. Workspace Ownership

Set ownership/ACL before workload.

---

# 129. Symlink Safety

Do not trust workspace symlinks when creating privileged mounts.

---

# 130. Canonical Path Validation

Use pre-opened file descriptors/handles where practical.

---

# 131. Privileged Helper

Some sandbox operations require privilege.

Create narrow helper boundary:

```text
forgeyard-sandbox-helper
```

---

# 132. Helper Responsibilities

Only:

```text
create namespace
mount approved paths
create cgroup
apply limits
configure network namespace
drop privileges
```

---

# 133. Helper Must Not

```text
interpret arbitrary shell
read project config
connect to metadata DB
resolve secrets
```

---

# 134. Helper Protocol

Small, typed, versioned.

---

# 135. Helper Request

```rust
pub struct SandboxSetupRequest {
    pub sandbox: SandboxId,
    pub uid: ExecutionUser,
    pub mounts: Vec<ValidatedMount>,
    pub resources: ResourceLimits,
    pub network: ValidatedNetworkPolicy,
}
```

---

# 136. Helper Validation

Defensive validation again.

---

# 137. Privileged Helper Authentication

Only local Forgeyard agent can invoke.

---

# 138. Unix Socket

Could use protected Unix socket or direct child process IPC.

---

# 139. Windows Privileged Service

Equivalent restricted service/helper if needed.

---

# 140. Helper Attack Surface

Keep extremely small.

---

# 141. Unsafe Rust

Sandbox platform crates may need unsafe/FFI.

Core model crates should remain `forbid(unsafe_code)`.

---

# 142. Unsafe Audit

Every unsafe block:

```text
document invariant
test
review
```

---

# 143. Container Runtime Abstraction

```rust
pub trait ContainerRuntime {
    async fn create(...);
    async fn start(...);
    async fn kill(...);
    async fn remove(...);
}
```

---

# 144. Supported Runtimes

Potential:

```text
containerd
Podman
Docker-compatible engine
native OCI runtime
```

---

# 145. Preferred Direction

Avoid Docker socket coupling as architecture.

Use OCI/container runtime adapters.

---

# 146. Rootless Containers

Preferred where compatible.

---

# 147. Container Image Identity

Use digest-pinned image.

Never mutable tag alone in strict mode.

---

# 148. Image Pull

Resolved/fetched before strict execution where possible.

---

# 149. Image Secrets

Registry credentials stay outside workload.

---

# 150. Image Filesystem

Readonly base + writable ephemeral layer.

---

# 151. Container Capabilities

Drop by default.

---

# 152. Seccomp/AppArmor/SELinux

Can strengthen Linux container isolation.

Platform capability reports support.

---

# 153. SELinux

If host enabled, Forgeyard adapter can apply labels.

---

# 154. AppArmor

Likewise.

---

# 155. Container Escape Risk

Containers share host kernel.

Do not equate with VM isolation.

---

# 156. VM Executor

Stronger boundary.

---

# 157. VM Inputs

Materialize CAS closure into VM through:

```text
virtiofs
read-only disk
CAS proxy
```

---

# 158. VM Outputs

Export through controlled channel.

---

# 159. VM Network

Configured explicitly.

---

# 160. VM Lifecycle

```text
create
boot
execute
collect
destroy
```

---

# 161. MicroVM

Useful later for:

```text
high isolation
fast startup
```

---

# 162. VM Image Identity

Immutable/digest-pinned.

---

# 163. VM Snapshot

Can optimize startup.

Not correctness dependency.

---

# 164. Confidential VM

Adds attestation.

---

# 165. Attestation

Trust subsystem validates.

Scheduler only sees capability.

---

# 166. Confidential Secrets

Secrets released only after attestation if required.

---

# 167. Confidential Output

Still hashed/uploaded through normal CAS semantics.

---

# 168. Hermetic Mode

Strict:

```text
readonly declared inputs
sanitized env
network deny
immutable toolchain
declared outputs
```

---

# 169. Impure Developer Mode

May permit:

```text
host network
host toolchain
extra paths
```

but must be labeled impure.

---

# 170. No Mixing Labels

Result provenance records isolation/hermeticity level.

---

# 171. Execution Profile

```rust
pub enum ExecutionProfile {
    Development,
    StandardCi,
    Release,
    HighAssurance,
}
```

---

# 172. Profile Expansion

Profile maps to sandbox policy defaults.

---

# 173. Release Profile

Suggested:

```text
strict filesystem
network deny after fetch
managed toolchains
no host HOME
no device unless explicit
strong sandbox
```

---

# 174. High-Assurance Profile

May require VM isolation + independent reproduction.

---

# 175. Platform Capability Negotiation

Job policy asks:

```text
minimum isolation level
required network controls
required resource enforcement
```

---

# 176. Runner Eligibility

Scheduler filters based on sandbox capabilities.

---

# 177. Runtime Verification

Agent verifies selected backend still supports requirement.

---

# 178. Executor Selection

```rust
pub trait ExecutorSelector {
    fn select(
        &self,
        requirements: &ExecutionRequirements,
        capabilities: &ExecutorCapabilities,
    ) -> Result<ExecutorKind, ExecutorSelectionError>;
}
```

---

# 179. Selection Rule

Prefer least-cost backend satisfying requirements.

---

# 180. Example

```text
Standard CI + Linux namespace available
→ Process/Namespace

High Assurance
→ VM
```

---

# 181. User Cannot Force Weaker Executor

Pipeline may request stronger isolation.

It cannot override organization minimum to weaker.

---

# 182. Executor Capability

```rust
pub struct ExecutorCapabilities {
    pub isolation_levels: BTreeSet<IsolationLevel>,
    pub network_modes: BTreeSet<NetworkModeCapability>,
    pub resource_controls: ResourceControlCapabilities,
}
```

---

# 183. Filesystem Capability

```text
readonly bind mount
tmpfs
overlay
reflink
quota
```

---

# 184. Network Capability

```text
deny all
restricted egress
job-local service network
```

---

# 185. Process Capability

```text
kill tree
PID isolation
privilege drop
```

---

# 186. Health Check

Sandbox doctor verifies features actually work.

Not just kernel version checks.

---

# 187. Probe Sandbox

Run a tiny self-test.

---

# 188. Linux Doctor

Checks:

```text
user namespace
mount namespace
cgroup v2
seccomp
bubblewrap/helper
network namespace
```

---

# 189. Windows Doctor

Checks:

```text
Job Objects
restricted token
ACL creation
process tree termination
Hyper-V if configured
```

---

# 190. macOS Doctor

Checks actual available sandbox/VM backend.

---

# 191. Capability Downgrade

If kernel/admin disables feature:

agent refreshes capabilities.

---

# 192. Existing Jobs

Continue if current sandbox already established safely.

New incompatible leases rejected.

---

# 193. Sandbox Local State

Agent may store:

```text
SandboxId
workspace
pid/process group
cgroup path
container ID
VM ID
```

for cleanup.

---

# 194. No Business State

No Job result authority stored in sandbox subsystem.

---

# 195. Crash Cleanup

On agent restart:

```text
scan known sandbox handles
terminate/destroy leftovers
```

---

# 196. Container Cleanup

Remove stopped containers/snapshots.

---

# 197. Cgroup Cleanup

Remove after processes gone.

---

# 198. Mount Cleanup

Unmount safely.

---

# 199. Cleanup Ordering

```text
stop process
unmount
remove cgroup/container
delete workspace
```

---

# 200. Cleanup Idempotency

Repeated cleanup succeeds/no-op.

---

# 201. Zombie Reaper

Agent/sandbox should reap child processes.

---

# 202. PID 1 in Namespace

Need proper reaper if agent creates PID namespace.

---

# 203. Signal Forwarding

Forward cancellation to workload root process appropriately.

---

# 204. TTY

CI default:

```text
non-interactive
```

Optional pseudo-TTY for specific tools if declared.

---

# 205. stdin

Default closed/controlled.

---

# 206. Interactive Jobs

Potential future separate feature.

Not default CI behavior.

---

# 207. Shell Selection

From planned `CommandSpec`.

---

# 208. Shell Path

Resolved from managed executor/toolchain environment.

---

# 209. Shell Injection

`Exec` avoids shell quoting.

`Shell` script is explicit code.

---

# 210. Working Directory

Must remain inside sandbox workspace.

---

# 211. Absolute Command

Allowed only if points to sandbox-visible approved executable.

---

# 212. PATH Resolution

Deterministic where strict.

---

# 213. Process Environment Size

Bound.

---

# 214. Argument Size

Bound by platform and Forgeyard validation.

---

# 215. Unicode

Use platform-safe conversions and preserve bytes where needed.

---

# 216. Exit Capture

Capture full platform exit semantics into typed result.

---

# 217. stdout/stderr

Pipe asynchronously.

---

# 218. Pipe Deadlock Prevention

Drain both concurrently.

---

# 219. Huge Output

Backpressure/spool.

---

# 220. Broken Pipe

Should not crash agent.

---

# 221. Structured Test Events

Can flow over side channel/file but not trusted as process authority.

---

# 222. Step Execution

Executor may receive sequence of steps or one action at a time.

---

# 223. Recommended Initial Model

Agent orchestrates step sequence.

Executor executes each command within same sandbox.

---

# 224. Sandbox Lifetime

One sandbox per JobAttempt.

---

# 225. Step Isolation

Same job shares sandbox unless explicitly isolated.

---

# 226. Why Job-Level Sandbox

Allows build steps to share workspace.

---

# 227. Service Processes

Start inside same sandbox/service network.

---

# 228. Service Cleanup

All killed at job end.

---

# 229. Service Health

Agent waits for declared health check before main step.

---

# 230. Service Timeout

Bounded.

---

# 231. Container Services

Can be sidecars in container executor.

---

# 232. Process Services

Can be child processes within namespace.

---

# 233. Filesystem Snapshots

Optional optimization for retry/debug.

Not default due to sensitive data.

---

# 234. Failed Workspace Retention

Default delete after diagnostics captured.

---

# 235. Debug Retention Security

If enabled:

```text
restricted operator access
short TTL
secret cleanup
```

---

# 236. Secret Cleanup

Before any debug snapshot retention.

---

# 237. Network Audit

Restricted network mode can log allowed/blocked destinations.

---

# 238. eBPF

Optional Linux visibility/enforcement enhancement.

Not correctness dependency.

---

# 239. eBPF Uses

```text
network observability
syscall/process telemetry
resource profiling
```

---

# 240. eBPF Security

Do not make eBPF mandatory.

Kernel compatibility varies.

---

# 241. io_uring

I/O optimization only.

Not sandbox guarantee.

---

# 242. Sandbox Error Model

```rust
pub enum SandboxError {
    UnsupportedIsolation,
    InvalidFilesystemPlan,
    MountFailed,
    NamespaceFailed,
    ResourceLimitFailed,
    NetworkSetupFailed,
    PrivilegeDropFailed,
    DeviceAttachFailed,
    CleanupFailed,
    SecurityViolation,
    Internal,
}
```

---

# 243. Executor Error Model

```rust
pub enum ExecutorError {
    Prepare(SandboxError),
    SpawnFailed,
    IoFailed,
    CancelFailed,
    Timeout,
    Lost,
    Unsupported,
    Internal,
}
```

---

# 244. Failure Mapping

Map into Run/Job `FailureClass`.

---

# 245. Preparation Failure

Usually infrastructure.

---

# 246. Sandbox Violation

Security failure, non-retry by default until reviewed.

---

# 247. Spawn Failure

Could be:

```text
toolchain missing
invalid executable
resource problem
```

typed detail required.

---

# 248. OOM

May be workload/resource mis-sizing.

Retry policy configurable.

---

# 249. Timeout

Mapped distinctly.

---

# 250. Executor Crash

Attempt infrastructure failure/lost.

---

# 251. Container Runtime Crash

Infrastructure.

---

# 252. VM Startup Failure

Infrastructure.

---

# 253. Unsupported Policy

Job should ideally have been unschedulable.

Runtime reject if capability changed.

---

# 254. Security Invariants

1. workload cannot access agent credentials;
2. workload cannot access metadata DB;
3. workload cannot access other jobs;
4. workload cannot mutate shared CAS;
5. host filesystem access is deny-by-default;
6. host network access is deny-by-default in strict mode;
7. device access is explicit;
8. privileged helper has minimal protocol;
9. child processes cannot survive job cleanup;
10. isolation level is never overstated.

---

# 255. Hermetic Invariants

1. declared inputs only;
2. readonly source/toolchain;
3. sanitized environment;
4. controlled time/locale where configured;
5. network denied during realization;
6. declared outputs only.

---

# 256. Resource Invariants

1. scheduler reservation exists;
2. sandbox enforces local hard limits where supported;
3. PIDs bounded;
4. memory bounded;
5. disk bounded or monitored;
6. timeout bounded.

---

# 257. Cross-Platform Invariants

1. common contract;
2. platform-specific implementation;
3. no fake parity claims;
4. stronger policy may require stronger backend;
5. unsupported requirements cause scheduling/runtime rejection.

---

# 258. Observability Metrics

```text
sandbox_prepare_duration
sandbox_start_failures
sandbox_cleanup_failures
sandbox_violations
executor_spawn_duration
executor_runtime_duration
executor_cancellations
executor_timeouts
executor_oom
resource_peak_memory
```

---

# 259. Isolation Metrics

Group by:

```text
executor kind
isolation level
platform
```

---

# 260. No High-Cardinality IDs

Use tracing for AttemptId/SandboxId.

---

# 261. Tracing

Spans:

```text
sandbox.prepare
sandbox.mount
sandbox.network
sandbox.resource
executor.spawn
executor.wait
executor.cancel
sandbox.cleanup
```

---

# 262. Security Audit

Record:

```text
sandbox violation
privileged helper failure
unexpected device access
policy downgrade rejection
```

---

# 263. CLI

```text
forgeyard runner sandbox doctor
forgeyard runner sandbox capabilities
forgeyard runner sandbox test
```

---

# 264. `sandbox capabilities`

Shows:

```text
Namespace
Container
VM
network deny
restricted egress
cgroup
seccomp
device isolation
```

---

# 265. `sandbox test`

Runs benign isolation checks.

---

# 266. Testkit

```text
forgeyard-sandbox-testkit/src/
├── lib.rs
├── fake_backend.rs
├── filesystem.rs
├── network.rs
├── resource.rs
├── process_tree.rs
└── assertions.rs
```

Executor:

```text
forgeyard-executor-testkit/src/
├── lib.rs
├── fake_executor.rs
├── command.rs
├── timeout.rs
├── cancel.rs
└── result.rs
```

---

# 267. Unit Tests

Test policy validation and selection.

---

# 268. Integration Tests

Platform-specific real sandbox tests.

---

# 269. Linux Security Tests

Attempt:

```text
read host /etc/shadow
access agent credential
escape workspace
fork bomb
network access under Deny
mount privileged filesystem
```

must fail appropriately.

---

# 270. Windows Security Tests

Attempt:

```text
break Job Object
access agent service files
spawn detached child
write other workspace
```

---

# 271. macOS Security Tests

Verify promised isolation level only.

If strong filesystem/network isolation unavailable, high-assurance job must be rejected or use VM.

---

# 272. Process Cleanup Test

Spawn child/grandchild/background daemon.

After cancellation:

```text
all gone
```

---

# 273. OOM Test

Hit memory limit; typed OOM result.

---

# 274. PIDs Test

Fork bomb constrained.

---

# 275. Disk Test

Exceed workspace quota/watchdog.

---

# 276. Network Test

Denied outbound request fails.

---

# 277. Restricted Egress Test

Allowed host works, disallowed blocked.

---

# 278. Secret Test

Secret visible where injected, absent elsewhere, removed after cleanup.

---

# 279. CAS Mutation Test

Attempt to modify mounted CAS/toolchain source must fail.

---

# 280. Path Traversal Test

Malicious symlink/mount path rejected.

---

# 281. Container Escape Regression Tests

Track known runtime hardening expectations.

---

# 282. Fuzzing

Fuzz:

```text
mount plans
path policy
network policy
helper messages
executor result parsing
```

---

# 283. Privileged Helper Fuzzing

High priority.

---

# 284. Failure Injection

```text
mount failure
cgroup failure
runtime crash
network setup failure
cleanup failure
```

---

# 285. Resource Leak Test

Thousands of jobs leave:

```text
no mounts
no cgroups
no containers
no VMs
no child processes
```

---

# 286. Performance Benchmarks

Measure:

```text
sandbox setup
process spawn
container start
VM start
cleanup
```

---

# 287. Startup Cache

May cache prepared base image/toolchain layers.

Never weaken isolation.

---

# 288. Sandbox Pooling

Future optimization:

```text
warm VM/container pool
```

Dangerous because residual state.

---

# 289. Default

Fresh sandbox per attempt.

---

# 290. Warm Pool Requirements

If added:

```text
secure reset
memory/storage scrubbing
identity rotation
verification
```

---

# 291. Implementation Phase 1 — Core Models

Implement:

```text
SandboxPolicy
IsolationLevel
FilesystemPolicy
NetworkPolicy
ResourceLimits
Executor traits
```

---

# 292. Phase 2 — Process Executor

Cross-platform basic process-tree ownership.

---

# 293. Phase 3 — Linux Production Sandbox

Implement:

```text
namespace
mount
cgroup v2
capability drop
seccomp
```

---

# 294. Phase 4 — Windows Sandbox

Implement:

```text
Job Objects
restricted token
ACL workspace
resource limits
```

---

# 295. Phase 5 — macOS Sandbox

Implement honest baseline + VM-backed stronger mode planning.

---

# 296. Phase 6 — Container Executor

OCI runtime abstraction.

---

# 297. Phase 7 — Network Policy

Deny/restricted/service networks.

---

# 298. Phase 8 — Device Exposure

Leased device-only access.

---

# 299. Phase 9 — Privileged Helper Hardening

Small protocol, fuzzing, privilege separation.

---

# 300. Phase 10 — VM Executor

Add strong isolation.

---

# 301. Phase 11 — Confidential Execution

Optional enterprise/high-assurance feature.

---

# 302. Phase 12 — Hardening

Security testing, leak tests, performance, failure injection.

---

# 303. Acceptance Tests

1. Workload cannot read agent credentials.
2. Workload cannot read another job workspace.
3. Source tree is read-only in strict mode.
4. Toolchain store is read-only.
5. Build/output dirs are writable only as intended.
6. Synthetic HOME prevents host credential leakage.
7. PATH is explicit.
8. Network Deny blocks outbound network.
9. Restricted egress blocks unauthorized destinations.
10. Child processes are killed on cancellation.
11. Fork bomb hits PID limit.
12. Memory limit produces OOM result.
13. Wall-time limit produces TimedOut result.
14. User cancellation produces Cancelled result.
15. Sandbox violation produces typed security result.
16. Linux namespace sandbox reports actual capabilities.
17. Windows Job Object owns process tree.
18. macOS does not claim unsupported namespace isolation.
19. High-assurance job cannot run on weak isolation.
20. Container executor uses digest-pinned image in strict mode.
21. Docker socket is not exposed by default.
22. Privileged container is denied for normal jobs.
23. Device job sees only leased device.
24. GPU job sees only allocated GPU where supported.
25. Secret files are removed after cleanup.
26. Cleanup is idempotent.
27. Agent restart cleanup destroys orphan sandboxes.
28. Hardlink optimization cannot mutate CAS.
29. VM executor destroys VM after attempt.
30. Executor result maps correctly to FailureClass.
31. Same ExecutionRequest contract works across Linux/Windows/macOS.
32. Sandbox doctor detects missing host features.
33. Capability downgrade prevents incompatible new jobs.
34. Forgeyard release builds can require stronger sandbox profile.
35. Forgeyard self-hosting uses the same executor/sandbox stack.

---

# 304. Production Readiness Gates

Do not call sandbox/executor production-ready until:

```text
process-tree cleanup proven
filesystem isolation proven
secret isolation proven
resource enforcement proven
network deny proven
Linux sandbox hardened
Windows baseline hardened
macOS capability honesty enforced
cancel/timeout race tests pass
privileged helper fuzzed
cleanup leak tests pass
metrics/doctor available
```

Container/VM/confidential modes may reach production readiness independently.

---

# 305. Architectural Invariants

1. workload is untrusted;
2. agent is outside workload sandbox;
3. agent credentials never enter sandbox;
4. each attempt gets unique sandbox;
5. source/toolchains readonly in strict mode;
6. host paths deny-by-default;
7. host environment deny-by-default;
8. network deny-by-default for strict realization;
9. device access explicit;
10. scheduler reservation plus sandbox enforcement;
11. process tree fully owned/killable;
12. cancellation distinct from timeout;
13. sandbox capability is never overstated;
14. no silent isolation downgrade;
15. platform-specific guarantees remain explicit;
16. Docker is not architectural dependency;
17. containers are not equivalent to VMs;
18. VM isolation is stronger capability, not default requirement;
19. privileged helper is tiny and typed;
20. unsafe code isolated/reviewed;
21. cleanup is idempotent;
22. no workload process survives completed attempt;
23. secrets are removed before debug retention;
24. eBPF/io_uring are optional optimizations;
25. hermetic/release profiles can demand stronger rules;
26. impure development mode is labeled distinctly;
27. executor does not decide job state authority;
28. sandbox does not write metadata DB;
29. same executor contract works across platforms;
30. Forgeyard dogfoods its own isolation layer.

---

# 306. Final Target Architecture

```text
                      Forgeyard Agent
                            │
                            ▼
                    ExecutionRequest
                            │
                            ▼
                    Executor Selector
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
    Process              Container               VM
       │                    │                    │
       └────────────────────┼────────────────────┘
                            ▼
                      Sandbox Policy
                            │
          ┌─────────────────┼─────────────────┐
          ▼                 ▼                 ▼
      Filesystem         Network          Resources
          │                 │                 │
          └─────────────────┼─────────────────┘
                            ▼
                    Platform Backend
               ┌────────────┼────────────┐
               ▼            ▼            ▼
             Linux       Windows       macOS
               │            │            │
               └────────────┼────────────┘
                            ▼
                    Untrusted Workload
                            │
                            ▼
                  Exit / Usage / Logs
                            │
                            ▼
                       Agent Output
```

---

# 307. Final Architectural Position

Security flow:

```text
validated JobSpec
+
resource reservation
+
sandbox policy
+
network policy
+
workspace
        ↓
executor prepares isolated environment
        ↓
agent verifies capabilities
        ↓
workload starts
        ↓
resource/network/filesystem restrictions enforced
        ↓
all descendant processes owned
        ↓
cancel/timeout/exit
        ↓
hard cleanup
        ↓
typed ExecutionResult
```

The key guarantee is:

> **Forgeyard never treats process execution as “just spawn a command.” Every workload runs inside an explicit, inspectable, capability-matched isolation policy whose real guarantees are enforced by the host platform and reported honestly to the scheduler and control plane.**

---

# 308. New-Repository Sequence

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
