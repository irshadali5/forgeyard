# 07 — Forgeyard Runner / Agent System Architecture

**Document type:** Core Execution Runtime System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Runner identity, agent lifecycle, registration, capability discovery, lease handling, workspace preparation, CAS materialization, sandbox/executor handoff, heartbeats, logs, output finalization, local persistence, reconnect/recovery, drain/shutdown, device hosting, and platform-specific execution integration  
**Architecture style:** Thin trusted agent coordinating immutable planned work with strict lease authority, local capability discovery, isolated execution, and resumable control-plane communication  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on `05-forgeyard-run-job-state-machine.md` and `06-forgeyard-scheduler-system-architecture.md`. It also consumes the CAS data plane, platform/native capability model, pipeline `JobSpecId`, and transport/protocol architecture.

---

# 1. Purpose

Forgeyard runners execute the actual workload.

That makes them one of the highest-risk and most operationally important components in the system.

The runner/agent must:

```text
register honestly
report capabilities
receive only authorized leased work
fetch exact immutable inputs
create isolated workspace
invoke sandbox/executor
stream logs/status
upload outputs
finalize only current attempt
recover safely from disconnects
```

The central rule is:

> **The agent coordinates execution but is never the authority for job state, policy, or scheduling.**

A second rule is:

> **Every active execution is bound to a specific JobId + AttemptId + LeaseId + JobSpecId + AgentSessionId.**

A third rule is:

> **The agent never executes arbitrary daemon text commands outside a validated planned job specification.**

---

# 2. Architectural Position

```text
                Forgeyard Daemon
                      │
                 QUIC/Postcard
                      │
                      ▼
                Forgeyard Agent
                      │
      ┌───────────────┼────────────────┐
      ▼               ▼                ▼
 Capability      CAS Materialize     Heartbeat
 Discovery            │                │
                      ▼                │
                  Workspace            │
                      │                │
                      ▼                │
              Sandbox / Executor       │
                      │                │
                      ▼                │
                 User Workload         │
                      │                │
                      ▼                │
               Output Capture          │
                      │                │
                      ▼                │
                     CAS               │
                      │                │
                      └───────┬────────┘
                              ▼
                         Completion
```

---

# 3. Goals

The agent MUST:

1. have stable runner identity;
2. have ephemeral agent session identity;
3. authenticate to daemon;
4. negotiate protocol version;
5. report capabilities;
6. refresh capabilities;
7. report liveness;
8. accept/reject leases;
9. validate lease/job/spec identity;
10. fetch CAS inputs;
11. verify inputs;
12. materialize workspace safely;
13. invoke executor abstraction;
14. enforce local resource limits;
15. stream logs;
16. stream significant phase changes;
17. upload outputs;
18. verify output digests;
19. finalize attempts idempotently;
20. support reconnect;
21. support drain;
22. support graceful shutdown;
23. support local state recovery;
24. handle stale leases safely;
25. isolate workloads from agent internals;
26. support Linux/Windows/macOS runners;
27. support device-hosting runners;
28. support signing-restricted workers via separate composition;
29. expose metrics/health;
30. remain independent of UI/business-policy logic.

---

# 4. Non-Goals

The agent does not:

```text
compile pipeline syntax
decide runner placement
evaluate business authorization
become metadata authority
host public REST API
make release approval decisions
```

---

# 5. Workspace Structure

```text
crates/runner/
├── forgeyard-runner/
├── forgeyard-runner-model/
├── forgeyard-runner-identity/
├── forgeyard-runner-registration/
├── forgeyard-runner-capability/
├── forgeyard-runner-session/
├── forgeyard-runner-lease/
├── forgeyard-runner-workspace/
├── forgeyard-runner-materialize/
├── forgeyard-runner-input/
├── forgeyard-runner-output/
├── forgeyard-runner-log/
├── forgeyard-runner-heartbeat/
├── forgeyard-runner-reconnect/
├── forgeyard-runner-drain/
├── forgeyard-runner-local-state/
├── forgeyard-runner-health/
├── forgeyard-runner-metrics/
└── forgeyard-runner-testkit/
```

Application binary:

```text
apps/forgeyard-agent/
```

---

# 6. Runner Identity

Stable identity:

```rust
pub struct RunnerId(Ulid);
```

Represents one provisioned logical runner.

---

# 7. Agent Session Identity

```rust
pub struct AgentSessionId(Ulid);
```

Changes every agent process/session startup unless explicit recovery reuses validated session state.

Recommended initial behavior:

```text
new process = new AgentSessionId
```

---

# 8. Why Separate RunnerId and SessionId

Because:

```text
same machine
same runner enrollment
new process
```

must not silently inherit old leases.

---

# 9. Runner Enrollment

Provisioning flow:

```text
operator creates/enrolls runner
  ↓
runner gets bootstrap credential
  ↓
agent starts
  ↓
mTLS/token enrollment
  ↓
RunnerId assigned/bound
  ↓
long-term runner credential issued
```

Exact identity mechanism belongs to trust/identity subsystem.

---

# 10. Agent Startup

```text
load config
  ↓
load runner identity
  ↓
create AgentSessionId
  ↓
discover local capabilities
  ↓
connect daemon
  ↓
authenticate
  ↓
negotiate protocol
  ↓
register
  ↓
enter active loop
```

---

# 11. Agent Main Loop

Conceptually:

```text
control messages
heartbeats
lease offers
capability refresh
log streams
reconnect
shutdown
```

---

# 12. Agent State

```rust
pub enum AgentState {
    Starting,
    Connecting,
    Registering,
    Online,
    Draining,
    Reconnecting,
    ShuttingDown,
    Offline,
}
```

---

# 13. Registration Request

```rust
pub struct RegisterAgent {
    pub runner: RunnerId,
    pub session: AgentSessionId,
    pub protocol: ProtocolSupport,
    pub version: AgentVersion,
    pub capabilities: RunnerCapabilities,
    pub capability_digest: CapabilityDigest,
}
```

---

# 14. Registration Response

```rust
pub struct RegistrationAccepted {
    pub server_time: Timestamp,
    pub lease_policy: LeasePolicy,
    pub heartbeat_policy: HeartbeatPolicy,
    pub protocol: ProtocolVersion,
}
```

---

# 15. Registration Rejection

Reasons:

```text
authentication failed
runner disabled
protocol incompatible
agent version unsupported
capability schema unsupported
```

---

# 16. Capability Discovery

Agent discovers:

```text
OS
architecture
CPU
memory
disk
GPU
sandbox support
toolchains
SDKs
devices
security capabilities
```

---

# 17. Capability Discovery Crates

Platform adapters provide probes.

Example Linux:

```text
platforms/linux/forgeyard-linux-detect
```

Runner capability layer composes results.

---

# 18. Capability Snapshot

```rust
pub struct RunnerCapabilities {
    pub platform: PlatformCapability,
    pub resources: ResourceCapacity,
    pub toolchains: Vec<ToolchainCapability>,
    pub sandbox: Vec<SandboxCapability>,
    pub devices: DeviceCapabilitySummary,
    pub trust: RunnerTrust,
}
```

---

# 19. Capability Digest

Canonical digest over capability snapshot.

```rust
pub struct CapabilityDigest(Digest);
```

---

# 20. Capability Refresh

Refresh triggers:

```text
startup
toolchain install/remove
device attach/detach
GPU state change
major resource availability change
config change
```

---

# 21. Capability Refresh Is Not Every Heartbeat

Avoid heavy repeated reports.

Heartbeat can carry:

```text
capability_digest
```

and send full snapshot only when changed.

---

# 22. Capability Honesty

General capabilities may be locally probed.

Privileged capabilities:

```text
signing
confidential
trusted network
```

must be provisioned/verified by control plane policy, not self-asserted.

---

# 23. Runner Resource Snapshot

```rust
pub struct LocalResourceSnapshot {
    pub total: ResourceVector,
    pub agent_reserved: ResourceVector,
    pub active_allocations: ResourceVector,
    pub pressure: ResourcePressure,
}
```

---

# 24. Resource Pressure

```rust
pub enum ResourcePressure {
    Normal,
    Cpu,
    Memory,
    Disk,
    Thermal,
    Critical,
}
```

---

# 25. Pressure Response

Agent can reduce advertised allocatable resources or reject new lease.

---

# 26. Heartbeat

```rust
pub struct AgentHeartbeat {
    pub runner: RunnerId,
    pub session: AgentSessionId,
    pub sent_at: Timestamp,
    pub agent_state: AgentState,
    pub active_attempts: Vec<ActiveAttemptSummary>,
    pub capability_digest: CapabilityDigest,
    pub resource_summary: ResourceSummary,
}
```

---

# 27. Heartbeat Frequency

Configurable.

Should balance:

```text
failure detection
network overhead
DB/event load
```

---

# 28. Heartbeat Coalescing

Daemon should not persist every field of every heartbeat as append-only event.

Runner liveness is high-frequency operational state.

---

# 29. Lease Offer

```rust
pub struct LeaseOffer {
    pub lease: JobLease,
    pub spec: PlannedJobSpec,
    pub spec_id: JobSpecId,
    pub input_manifest: CasObjectRef,
}
```

---

# 30. Agent Lease Validation

Before accepting:

```text
runner/session match
protocol compatible
lease not expired
job spec supported
required capabilities still present
local resource availability sufficient
agent not draining
```

---

# 31. Lease Acceptance

```rust
pub struct AcceptLease {
    pub lease: LeaseId,
    pub attempt: JobAttemptId,
    pub runner: RunnerId,
    pub session: AgentSessionId,
    pub spec: JobSpecId,
}
```

---

# 32. Lease Rejection

```rust
pub struct RejectLease {
    pub lease: LeaseId,
    pub reason: LeaseRejectReason,
}
```

---

# 33. Rejection Reasons

```rust
pub enum LeaseRejectReason {
    Draining,
    ResourcePressure,
    CapabilityChanged,
    DiskInsufficient,
    UnsupportedSpec,
    LocalPolicy,
    InternalPreparationFailure,
}
```

---

# 34. Reject Before Work

Prefer rejection before mutating workspace/starting workload.

---

# 35. Accepted Lease Lifecycle

```text
Accepted
  ↓
Preparing
  ↓
Running
  ↓
UploadingOutputs
  ↓
Completed
```

---

# 36. Agent Attempt Context

```rust
pub struct AttemptContext {
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub runner: RunnerId,
    pub session: AgentSessionId,
    pub spec: JobSpecId,
}
```

Every execution-side action carries this context.

---

# 37. Local Attempt Registry

In memory:

```text
AttemptId -> AttemptRuntime
```

Guarded against duplicate lease messages.

---

# 38. Local Persistence

```text
forgeyard-runner-local-state
```

Stores minimal recoverable metadata:

```text
active attempt IDs
lease IDs
spec IDs
workspace paths
sandbox IDs
phase
```

---

# 39. Local State Is Not Authority

Daemon remains truth.

Local state only assists recovery/cleanup.

---

# 40. Local State Path

Example:

```text
/var/lib/forgeyard-agent/state/
```

or platform equivalent.

---

# 41. Atomic Local State

Write:

```text
temp
fsync
rename
```

where durability needed.

---

# 42. Workspace Root

Separate from CAS:

```text
/var/lib/forgeyard-agent/workspaces/
```

---

# 43. Workspace Per Attempt

```text
workspaces/<attempt-id>/
```

or randomized internal path.

---

# 44. Workspace Lifecycle

```text
create temp
  ↓
materialize inputs
  ↓
configure sandbox
  ↓
run
  ↓
capture outputs
  ↓
cleanup
```

---

# 45. Workspace Isolation

No sharing mutable workspace between unrelated attempts.

---

# 46. Input Materialization

```text
input manifest
  ↓
batch local CAS missing check
  ↓
fetch
  ↓
digest verify
  ↓
tree materialize
  ↓
read-only source/input
```

---

# 47. Source Path

Typically:

```text
/work/source
```

inside sandbox.

---

# 48. Build Path

Writable:

```text
/work/build
```

---

# 49. Output Path

Declared output roots only.

---

# 50. Toolchain Paths

Immutable toolchain store mounted read-only where possible.

---

# 51. CAS Safety

Never let workload modify shared CAS file.

---

# 52. Reflink/Copy-on-Write

Allowed optimization.

---

# 53. Hardlink

Only if strict read-only guarantees prevent mutation.

Default avoid for writable inputs.

---

# 54. Materialization Failure

Classify:

```text
InputFetch
CorruptInput
DiskInsufficient
PathIncompatible
```

---

# 55. Corrupt Input

Agent must not continue.

Report integrity error.

---

# 56. Missing Input

Try configured fallback tiers.

If unavailable:

```text
attempt preparation failure
```

---

# 57. Sandbox Handoff

Agent constructs:

```rust
pub struct ExecutionRequest {
    pub context: AttemptContext,
    pub workspace: PreparedWorkspace,
    pub spec: PlannedJobSpec,
    pub resources: ResourceReservation,
    pub environment: RuntimeEnvironment,
}
```

passes to `Executor`.

---

# 58. Agent Does Not Shell-Execute Directly

All workload execution goes through executor abstraction.

---

# 59. Executor Trait Boundary

```rust
#[async_trait]
pub trait Executor {
    async fn execute(
        &self,
        request: ExecutionRequest,
    ) -> Result<ExecutionHandle, ExecutorError>;
}
```

Detailed executor architecture comes next.

---

# 60. Secret Resolution

Agent requests/receives late-bound secret material through secret provider/control plane.

Never store in CAS.

---

# 61. Secret Lifetime

```text
resolve
  ↓
inject
  ↓
use
  ↓
zeroize/remove
```

---

# 62. Secret Persistence

Do not write secrets to local runner state.

---

# 63. Environment Construction

Combine:

```text
validated literal env
system env
late secrets
runtime paths
```

---

# 64. Host Environment Leakage

Strict mode starts from sanitized environment.

No automatic inheritance of:

```text
HOME
SSH_AUTH_SOCK
AWS credentials
developer PATH
```

unless explicitly allowed.

---

# 65. Preparation Phase Event

Agent sends:

```text
AttemptPreparing
```

after lease accepted and before substantial prep.

---

# 66. Running Phase Event

Send only after executor actually starts workload.

---

# 67. Output Phase Event

Send when workload exits and output capture starts.

---

# 68. Significant Phase Idempotency

Duplicate phase messages safe.

---

# 69. Logs

```text
forgeyard-runner-log
```

Captures:

```text
stdout
stderr
structured step events
agent diagnostics
```

---

# 70. Log Sequence

Per attempt:

```rust
pub struct LogSeq(u64);
```

Monotonic.

---

# 71. Log Frame

```rust
pub struct LogFrame {
    pub attempt: JobAttemptId,
    pub seq: LogSeq,
    pub stream: LogStreamKind,
    pub bytes: BoundedBytes,
    pub timestamp: Timestamp,
}
```

---

# 72. Log Streams

```rust
pub enum LogStreamKind {
    Stdout,
    Stderr,
    Agent,
    Structured,
}
```

---

# 73. Log Backpressure

If daemon/network slow:

```text
bounded memory buffer
local spool
CAS chunk flush
```

---

# 74. No Infinite RAM Buffer

Never buffer unbounded logs in memory.

---

# 75. Local Log Spool

Optional:

```text
workspaces/<attempt>/logs/
```

or dedicated spool area.

---

# 76. Log Loss Policy

For production builds:

```text
best effort live stream
durable chunk spool
```

so temporary disconnect does not lose all logs.

---

# 77. Log Redaction

Secret redaction before durable upload where possible.

---

# 78. Redaction Limit

No perfect generic secret detection guarantee.

Treat logs as sensitive.

---

# 79. Log Reconnect

On reconnect, agent reports last locally available sequence.

Daemon requests missing range.

---

# 80. Output Capture

After successful executor exit:

```text
declared outputs only
```

are captured.

---

# 81. Output Traversal

Validate:

```text
path under workspace
symlinks safe
size limits
file count
```

---

# 82. Output Tree

Canonicalize into CAS tree/manifest.

---

# 83. Output Hashing

BLAKE3 internal.

---

# 84. Output Upload

Tiered CAS upload according to durability class.

---

# 85. Output Metadata

Agent returns CAS refs.

Daemon registers artifact semantics.

---

# 86. Agent Does Not Create Release Metadata

Agent only produces execution outputs/evidence.

---

# 87. Output Upload Retry

Can retry without rerunning workload while workspace retained.

---

# 88. Workspace Retention During Finalize

Keep until:

```text
completion acknowledged
```

or finalization TTL expires.

---

# 89. Completion Request

```rust
pub struct CompleteAttemptRequest {
    pub context: AttemptContext,
    pub result: AttemptExecutionResult,
    pub outputs: Vec<CasObjectRef>,
    pub logs: Option<LogStreamRef>,
    pub message_id: MessageId,
}
```

---

# 90. Completion Ack

Daemon returns:

```rust
pub enum CompletionAck {
    Accepted,
    DuplicateAccepted,
    StaleRejected,
    InvalidRejected,
}
```

---

# 91. Stale Completion

Agent MUST stop retrying as authoritative after:

```text
StaleRejected
```

May keep local diagnostics briefly.

---

# 92. Duplicate Accepted

Safe to cleanup.

---

# 93. Unknown Completion Result

Network lost before ack:

retry same MessageId.

---

# 94. Completion Idempotency

Agent must persist completion message ID until acknowledged if local recovery desired.

---

# 95. Cleanup

After accepted terminal result:

```text
remove secrets
destroy sandbox
delete workspace
release local resources
trim spool
```

---

# 96. Cleanup Failure

Report health issue.

Do not change already accepted Job result.

---

# 97. Cleanup Reconciler

Periodic scan removes abandoned workspaces.

---

# 98. Workspace TTL

Unreferenced old workspaces deleted after grace.

---

# 99. Debug Retention

Policy may retain failed workspace locally temporarily.

Disabled by default in sensitive environments.

---

# 100. Drain

```rust
pub enum DrainMode {
    Graceful,
    CancelActive,
}
```

---

# 101. Graceful Drain

```text
stop accepting new leases
finish active attempts
disconnect when empty
```

---

# 102. Cancel Drain

Requests cancellation for active attempts through control plane.

Agent should not independently mark jobs cancelled.

---

# 103. Shutdown

Signal handling:

```text
SIGTERM
Ctrl-C
Windows service stop
```

moves agent to draining/shutdown.

---

# 104. Shutdown Timeout

After grace:

```text
hard terminate local workload
```

according to configured policy.

---

# 105. Agent Restart

Initial recommended behavior:

```text
new session
report old local attempt records
daemon decides stale/lost
cleanup
```

---

# 106. Process Reattachment

Advanced future capability.

Not required initially.

---

# 107. Recovery Handshake

```rust
pub struct RecoverAttempts {
    pub runner: RunnerId,
    pub new_session: AgentSessionId,
    pub previous: Vec<RecoveredAttemptSummary>,
}
```

---

# 108. Daemon Recovery Decision

```rust
pub enum RecoveryDecision {
    AbandonAndCleanup,
    ContinueSupported,
    Stale,
}
```

Initial implementation:

```text
AbandonAndCleanup
```

for restarted process attempts.

---

# 109. Reconnect Without Restart

Same AgentSessionId.

Active attempts can continue if lease valid.

---

# 110. Connection Loss

Agent enters:

```text
Reconnecting
```

---

# 111. During Short Disconnect

Workload may continue according to lease connectivity policy.

---

# 112. Lease Connectivity Policy

```rust
pub enum LeaseConnectivityPolicy {
    Continuous,
    Grace(Duration),
    OfflineAuthorized(Duration),
}
```

---

# 113. Default Distributed Policy

Short grace.

---

# 114. Offline Edge Mode

Can allow longer disconnected execution if explicitly authorized.

---

# 115. Lease Expiry Locally

Agent must stop workload when hard lease deadline exceeded if unable to renew.

---

# 116. Runner Monotonic Timer

Use local monotonic time for enforcement.

---

# 117. Clock Skew

Daemon absolute expiry converted to safe local monotonic deadline at receipt.

---

# 118. Reconnect Resync

Agent sends:

```text
active attempts
phase
last log seq
lease ID
```

---

# 119. Daemon Authority Response

```text
Continue
Cancel
Stale
```

---

# 120. Continue

Agent resumes reporting and renews lease.

---

# 121. Cancel

Agent terminates workload and reports cancellation.

---

# 122. Stale

Agent terminates workload immediately, does not finalize as authoritative.

---

# 123. Runner Health

```rust
pub struct RunnerHealth {
    pub status: RunnerHealthStatus,
    pub checks: Vec<RunnerHealthCheck>,
}
```

---

# 124. Health Status

```rust
pub enum RunnerHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

---

# 125. Health Checks

```text
disk space
CAS access
sandbox availability
executor probe
toolchain store
device daemon
clock sanity
credential validity
```

---

# 126. Unhealthy Runner

Agent remains connected but scheduler should receive no new leases.

---

# 127. Degraded Runner

May accept limited jobs depending reason/policy.

---

# 128. Agent Doctor

```text
forgeyard-agent doctor
```

or:

```text
forgeyard runner doctor
```

Checks all required local dependencies.

---

# 129. Runner CLI

On host:

```text
forgeyard-agent status
forgeyard-agent doctor
forgeyard-agent capabilities
forgeyard-agent drain
forgeyard-agent resume
forgeyard-agent cleanup
```

---

# 130. Remote Admin

Control-plane CLI:

```text
forgeyard runner list
forgeyard runner show
forgeyard runner drain
forgeyard runner disable
```

---

# 131. Capability Output

Human-readable:

```text
Linux x86_64
32 CPU
64 GiB
Rust stable ...
Docker/container executor ...
Android devices ...
```

---

# 132. Local Configuration

```text
config/default/agent.ron
```

Example:

```ron
(
    data_dir: "/var/lib/forgeyard-agent",
    max_parallel_jobs: 4,
    workspace_retention: "30m",
    reconnect: (
        max_backoff: "30s",
    ),
)
```

---

# 133. Runner Labels

Optional.

Used for:

```text
site
ownership
operator hints
```

Not privileged trust.

---

# 134. Runner Pool Binding

Agent can be assigned to pool by server/admin.

Do not trust self-selected restricted pool.

---

# 135. Trust Boundary

Untrusted workload must never access:

```text
agent credentials
daemon mTLS key
runner enrollment token
other job workspace
host secrets
```

---

# 136. Agent Credential Isolation

Keep credentials outside workload-visible mount namespace.

---

# 137. IPC Boundary

Prefer no direct workload-to-agent IPC.

---

# 138. Host Socket Leakage

Do not mount:

```text
Docker socket
SSH agent
system DBus
```

unless explicitly needed and policy-approved.

---

# 139. Privilege

Agent may require host privileges for sandbox setup.

Workload should run with far lower privileges.

---

# 140. Privilege Separation

Potential future:

```text
unprivileged agent
+
small privileged helper
```

for Linux namespace/cgroup setup.

---

# 141. Initial Linux Recommendation

Use narrowly scoped privileged operations and isolate helper logic in platform sandbox crate.

---

# 142. Windows Runner

Platform adapter handles:

```text
Job Objects
ACLs
process tokens
Windows SDK
```

---

# 143. macOS Runner

Platform adapter handles:

```text
sandbox/process controls
Xcode
simulators
signing only if allowed
```

---

# 144. Agent Core Remains Cross-Platform

No large platform-specific `cfg` forest in runner model/service.

---

# 145. Platform Adapter Trait

```rust
pub trait RunnerPlatform {
    fn discover_capabilities(&self) -> Result<PlatformCapabilities, PlatformError>;
    fn prepare_workspace(&self, ...) -> ...;
}
```

---

# 146. Device Hosting

Runner can host physical devices.

Device subsystem reports:

```text
attached
available
leased
health
```

---

# 147. Device Isolation

One physical device lease at a time unless device supports safe parallelism.

---

# 148. Device Runner Workflow

```text
job lease
+
device lease
  ↓
prepare
install/flash
run tests
collect logs
cleanup/reset
```

---

# 149. Device Cleanup

Mandatory before device returns to pool.

---

# 150. Device Failure

Classify separately from job workload.

---

# 151. Signing Worker Distinction

Do not simply enable signing capability on general-purpose agent.

Use:

```text
apps/forgeyard-signing-worker
```

with restricted code path.

---

# 152. Runner Agent vs Signing Worker

Runner:

```text
arbitrary untrusted build code
```

Signing worker:

```text
no arbitrary build code
restricted signing requests only
```

---

# 153. Confidential Runner

Requires special executor/trust integration.

Agent core only consumes capability/policy.

---

# 154. Toolchain Discovery

Agent should prefer immutable managed toolchain registry over scanning arbitrary PATH.

---

# 155. Host Toolchains

Can be reported in impure/dev mode.

Marked distinctly.

---

# 156. Managed Toolchain Capability

```rust
pub struct ToolchainCapability {
    pub id: ToolchainId,
    pub source: ToolchainSource,
    pub verified: bool,
}
```

---

# 157. Toolchain Warm Cache

Agent can report a bounded set of warm toolchains.

Scheduler uses as soft preference.

---

# 158. Auto-Install Toolchain

Agent may materialize required toolchain from CAS.

Does not need preinstalled capability if runner supports generic platform + toolchain materializer.

---

# 159. Capability Semantics

Distinguish:

```text
can materialize toolchain
```

from:

```text
toolchain already warm
```

---

# 160. Input Prefetch

Agent may prefetch leased job inputs before full sandbox creation.

---

# 161. Speculative Prefetch

Only if scheduler/control plane explicitly requests.

Do not waste bandwidth by guessing excessively.

---

# 162. Local CAS

Agent has L1 CAS.

---

# 163. CAS Eviction

Independent local cache policy.

Never delete active attempt required objects while in use.

---

# 164. CAS Pin During Attempt

Pin/local reference count required input closure until attempt cleanup.

---

# 165. Disk Quota Per Attempt

Prevent one job from exhausting host.

---

# 166. Workspace Size Accounting

Track:

```text
input materialized size
working data
outputs
logs
```

---

# 167. Resource Enforcement

Agent delegates hard limits to sandbox/executor.

---

# 168. Soft Usage Monitoring

Agent can monitor actual:

```text
CPU
memory
disk
```

for metrics.

---

# 169. OOM

Sandbox/executor reports:

```text
OutOfMemory
```

mapped to failure class.

---

# 170. Disk Full

Likewise.

---

# 171. Timeout Enforcement

Agent has local timers for:

```text
prepare
run
upload
```

---

# 172. Cancellation Token

Per attempt local token propagated into executor.

---

# 173. Graceful Process Termination

Executor:

```text
soft signal
grace period
hard kill
```

platform-specific.

---

# 174. Child Process Cleanup

Mandatory.

No orphan child processes after attempt.

---

# 175. Background Daemons Started by Job

Sandbox/process group cleanup kills them.

---

# 176. Workspace Escape Detection

Sandbox should prevent.

Agent cleanup also validates no unexpected mounts/processes remain.

---

# 177. Output Capture After Failure

Policy may capture diagnostics even for failed jobs.

---

# 178. Failure Artifacts

Examples:

```text
test report
core dump
screenshots
crash logs
```

subject to size/security policy.

---

# 179. Core Dumps

Disabled by default or restricted due to secrets.

---

# 180. Debug Artifacts

Can be marked sensitive.

---

# 181. Agent Internal Logs

Separate from job logs.

---

# 182. Agent Log Fields

```text
runner
session
attempt
lease
phase
```

No secret values.

---

# 183. Structured Events

Use stable event types for:

```text
lease accepted
workspace prepared
executor started
upload completed
cleanup failed
```

---

# 184. Metrics

```text
agent_active_jobs
agent_cpu_used
agent_memory_used
agent_disk_free
agent_lease_accept
agent_lease_reject
agent_reconnects
agent_input_fetch_bytes
agent_output_upload_bytes
agent_workspace_cleanup_failures
agent_log_spool_bytes
```

---

# 185. Attempt Metrics

```text
prepare_duration
execution_duration
upload_duration
cleanup_duration
```

---

# 186. High Cardinality

Do not label metrics with JobId.

Tracing carries IDs.

---

# 187. Health Endpoint

Local-only/admin:

```text
/health
```

if an HTTP endpoint is used.

Could instead expose IPC/CLI.

---

# 188. Public Exposure

Do not expose runner admin endpoint publicly by default.

---

# 189. Reconnect Backoff

```rust
pub struct ReconnectPolicy {
    pub initial: Duration,
    pub max: Duration,
    pub jitter: JitterPolicy,
}
```

---

# 190. Reconnect Behavior

Retry forever unless:

```text
credential revoked
protocol permanently incompatible
runner disabled
```

---

# 191. Credential Rotation

Agent supports server-issued new credential/cert.

---

# 192. Certificate Expiry

Warn before expiry.

---

# 193. Control-Plane Discovery

Agent config may contain:

```text
daemon endpoint
cluster endpoint
```

HA discovery details later.

---

# 194. Multiple Daemon Endpoints

Agent can fail over among cluster endpoints.

---

# 195. Connection Stickiness

Keep one active control stream; reconnect elsewhere on failure.

---

# 196. Duplicate Connections

Daemon prevents one same session from being simultaneously authoritative on two channels, or treats latest authenticated connection as active.

---

# 197. Session Fencing

Registration can return:

```text
session epoch/token
```

if needed.

---

# 198. Initial Recommendation

Use unique AgentSessionId and server-side active-session record.

---

# 199. Runner Disable

Admin sets runner disabled.

Agent connection may remain only long enough to cancel/drain.

---

# 200. Runner Delete

Logical deletion after active leases cleared.

---

# 201. Runner Re-enrollment

Creates/reuses RunnerId according to admin policy.

---

# 202. Host Reimage

Recommended:

```text
new enrollment credential/session
```

Stable RunnerId optional if inventory identity preserved.

---

# 203. Runner Tags vs Identity

Do not derive identity from hostname.

Hostname can change/collide.

---

# 204. Machine Fingerprint

Can be diagnostic, not sole security identity.

---

# 205. Runner Version

```rust
pub struct AgentVersion {
    pub forgeyard_version: SemVersion,
    pub protocol_support: ProtocolRange,
}
```

---

# 206. N/N-1

Agent and daemon support rolling compatibility as defined in protocol architecture.

---

# 207. Unsupported Version

Agent does not run jobs until upgraded.

---

# 208. Self-Update

Potential future.

Do not make runner auto-update mandatory initially.

---

# 209. Self-Update Security

Would require signed release verification and rollback.

Separate architecture later if needed.

---

# 210. Agent Installation

Platform packages:

```text
systemd service
Windows service
launchd
```

---

# 211. Linux Service

Runs dedicated user where possible.

---

# 212. Windows Service

Dedicated service account.

---

# 213. macOS Service

Dedicated daemon account/process.

---

# 214. Permissions

Grant only sandbox/device capabilities needed.

---

# 215. Rootless Mode

Support where sandbox capabilities allow.

---

# 216. Privileged Mode

Explicitly surfaced in doctor/UI.

---

# 217. Runner Local Policy

Agent may enforce stricter local limits than central job request.

Example:

```text
never allow host networking
```

---

# 218. Local Policy Cannot Weaken Central Policy

It may only tighten.

---

# 219. Lease Validation Against Local Policy

Reject lease if local stricter rule conflicts.

---

# 220. Agent Config Reload

Safe fields can hot-reload:

```text
log level
capacity limits
drain
```

Sensitive structural changes may require restart.

---

# 221. Config Reload Audit

Server/admin change audited if centrally managed.

---

# 222. Runner Capacity Override

Admin can reserve:

```text
2 CPUs for host
8 GiB RAM
```

---

# 223. Effective Allocatable

```text
physical
- agent reserve
- admin reserve
- active reservations
```

---

# 224. Max Parallel Jobs

Hard local guard even if scheduler bug.

---

# 225. Defense in Depth

Agent checks resource reservation and capability again before execution.

---

# 226. Scheduler Bug Safety

If impossible lease received:

```text
reject
```

not blindly execute.

---

# 227. JobSpec Validation

Agent validates schema/protocol + signatures/digest if applicable.

---

# 228. JobSpec Immutable

`JobSpecId` digest must match received bytes.

---

# 229. JobSpec Contents

Includes:

```text
steps/actions
resource limits
input refs
output declarations
network policy
sandbox requirements
timeouts
secret refs
```

---

# 230. No Dynamic Command Mutation

Daemon cannot alter command mid-attempt without new spec/lease.

---

# 231. Runtime Control Messages

Allowed:

```text
cancel
lease renewal
log backfill request
```

Not:

```text
replace shell command
```

---

# 232. Attempt Control Channel

Per attempt logical stream optional over shared QUIC connection.

---

# 233. Multiplexing

QUIC streams can separate:

```text
control
logs
CAS transfer
attempt events
```

---

# 234. Agent Transport Independence

Runner core should depend on transport trait, not Quinn-specific internals everywhere.

---

# 235. Transport Trait

```rust
#[async_trait]
pub trait AgentControlTransport {
    async fn send_event(&self, ...);
    async fn recv_command(&self, ...);
}
```

---

# 236. Internal Backpressure

Bound control channel queues.

---

# 237. Priority

Cancellation/lease renew messages higher priority than verbose logs.

---

# 238. Log Channel Failure

Should not block cancellation/control.

---

# 239. CAS Transfer Channel Failure

Can retry independently.

---

# 240. Agent Event Ordering

Per attempt significant phase events should maintain logical order.

---

# 241. Sequence Number

Optional:

```rust
pub struct AttemptEventSeq(u64);
```

---

# 242. Duplicate Event

Daemon deduplicates.

---

# 243. Local Event Journal

Optional minimal journal for reconnect.

---

# 244. Initial Recommendation

Persist:

```text
completion message
active attempt metadata
log spool
```

not every phase event.

---

# 245. Workspace Cleanup on Startup

Scan old directories.

Compare with local attempt state.

Cleanup unknown stale workspaces after grace.

---

# 246. Active Sandbox Detection

If process still running from previous crash:

initial implementation:

```text
terminate + cleanup
```

unless reattachment supported.

---

# 247. Safety First

Never leave unknown orphan workload running after agent restart.

---

# 248. Host Reboot

All active attempts likely lost.

On startup report/reconcile.

---

# 249. Resume Long Download

CAS subsystem can resume partial objects.

---

# 250. Resume Upload

Likewise.

---

# 251. Attempt Preparation Checkpoint

Could reuse fetched inputs after retry.

Local cache handles naturally.

---

# 252. Workspace Reuse

Do not reuse previous attempt mutable workspace by default.

New attempt gets fresh workspace.

---

# 253. Why Fresh Retry Workspace

Avoid contaminated state.

---

# 254. Cache Reuse

Reuse through CAS/action cache only.

---

# 255. Testkit

```text
forgeyard-runner-testkit/src/
├── lib.rs
├── fake_daemon.rs
├── fake_transport.rs
├── fake_executor.rs
├── fake_cas.rs
├── capability.rs
├── workspace.rs
├── lease.rs
└── assertions.rs
```

---

# 256. Unit Tests

Test:

```text
state
lease validation
capability refresh
workspace paths
cleanup
reconnect backoff
```

---

# 257. Integration Tests

1. register agent;
2. receive lease;
3. fetch input;
4. run fake executor;
5. upload output;
6. complete.

---

# 258. Duplicate Lease Test

Same LeaseId sent twice.

One local attempt only.

---

# 259. Stale Lease Test

Expired/stale lease rejected.

---

# 260. Capability Change Test

Lease requires GPU; GPU removed before accept -> reject.

---

# 261. Disconnect Test

Disconnect during Running, reconnect, resync, continue.

---

# 262. Stale After Reconnect Test

Daemon says Stale -> terminate.

---

# 263. Log Backpressure Test

Network stalled; memory remains bounded; spool used.

---

# 264. Output Retry Test

CAS temporary failure; output upload retried without rerun.

---

# 265. Completion Ack Loss Test

Completion committed but ack lost; same message retried, duplicate accepted.

---

# 266. Agent Crash Test

Crash during Running; restart cleans orphan and control plane marks lost.

---

# 267. Drain Test

Draining agent rejects new lease and completes active attempt.

---

# 268. Shutdown Test

Graceful + forced timeout.

---

# 269. Security Tests

1. workload cannot read agent credential;
2. workload cannot access another workspace;
3. absolute path escape rejected;
4. host env secrets absent;
5. stale lease cannot execute;
6. malformed JobSpec rejected;
7. unauthorized signing capability impossible on general agent.

---

# 270. Cross-Platform Tests

Linux, Windows, macOS agent behavior through same core contracts.

---

# 271. Device Tests

Attach/detach/lease/reset.

---

# 272. Performance Tests

Measure:

```text
registration
input materialization
log throughput
output hashing
concurrent attempt overhead
```

---

# 273. Scale Tests

One agent with many concurrent small jobs.

---

# 274. Resource Leak Test

After 1000 attempts:

```text
no leaked workspaces
no leaked child processes
no leaked resource reservations
```

---

# 275. Fuzzing

Targets:

```text
JobSpec decoder
control messages
local state recovery
log frames
```

---

# 276. Failure Injection

```text
disk full
CAS corrupt
network partition
executor crash
sandbox setup fail
credential expiry
```

---

# 277. Observability Acceptance

Operator can answer:

```text
why runner is not receiving jobs
which attempts are active
what capability changed
why lease rejected
how much disk/CAS used
```

---

# 278. Implementation Phase 1 — Identity / Registration

Implement:

```text
RunnerId
AgentSessionId
registration
heartbeat
basic health
```

---

# 279. Phase 2 — Capability Discovery

Implement platform/resource/toolchain capability reporting.

---

# 280. Phase 3 — Lease Loop

Implement:

```text
receive
validate
accept/reject
dedupe
```

---

# 281. Phase 4 — Workspace / CAS

Implement:

```text
per-attempt workspace
input fetch/materialize
local CAS pinning
```

---

# 282. Phase 5 — Executor Handoff

Integrate generic executor trait.

---

# 283. Phase 6 — Logs / Phase Events

Implement bounded streaming/spool.

---

# 284. Phase 7 — Output / Completion

Implement output capture, CAS upload, idempotent completion.

---

# 285. Phase 8 — Drain / Shutdown

Implement operator lifecycle.

---

# 286. Phase 9 — Reconnect / Recovery

Implement same-session reconnect and restart cleanup.

---

# 287. Phase 10 — Devices / Advanced Platforms

Integrate device-hosting and platform-specific capabilities.

---

# 288. Phase 11 — Hardening

Security, failure injection, high concurrency, protocol compatibility.

---

# 289. Acceptance Tests

1. Agent registers with stable RunnerId and new SessionId.
2. Protocol incompatibility blocks Online state.
3. Capability snapshot is deterministic.
4. Capability change triggers refresh.
5. Draining agent rejects new leases.
6. Expired lease is rejected.
7. Wrong RunnerId/SessionId lease is rejected.
8. Duplicate LeaseId creates one local execution.
9. Input CAS bytes are digest verified.
10. Corrupt input prevents execution.
11. Workspace is unique per attempt.
12. Workload cannot mutate shared CAS.
13. Host environment is sanitized.
14. Executor receives exact JobSpec.
15. Agent cannot alter planned command.
16. Phase transitions are reported idempotently.
17. Logs remain bounded under network backpressure.
18. Reconnect can backfill logs.
19. Output capture is limited to declared roots.
20. Output digest is verified.
21. Completion includes Job/Attempt/Lease/Spec/Session.
22. Lost completion ACK is safely retried.
23. StaleRejected stops authoritative retry.
24. Cleanup happens after accepted completion.
25. Agent crash leaves no permanently trusted state.
26. Restart creates new session and reconciles old attempts.
27. Graceful drain completes active work.
28. Forced shutdown terminates child processes.
29. General runner cannot self-assert signing trust.
30. Device hosting follows separate device lease.
31. Linux/Windows/macOS share same core agent semantics.
32. Same agent core works in standalone and distributed compositions.
33. UI has no direct agent control authority.
34. Metrics expose pressure/rejections/reconnects.
35. Forgeyard self-hosting runners can execute Forgeyard jobs through this agent.

---

# 290. Production Readiness Gates

Do not call runner/agent production-ready until:

```text
registration/auth stable
capability discovery verified
lease validation correct
input integrity enforced
workspace isolation stable
executor boundary enforced
bounded logs implemented
output commit idempotent
reconnect tested
drain/shutdown tested
restart cleanup tested
security tests pass
```

---

# 291. Architectural Invariants

1. RunnerId is stable provisioned identity.
2. AgentSessionId is ephemeral process/session identity.
3. Lease is bound to runner + session.
4. Agent never becomes job-state authority.
5. Agent never becomes scheduler authority.
6. Agent executes only validated immutable JobSpec.
7. JobSpecId must match received spec.
8. Duplicate lease delivery is safe.
9. Stale lease never executes.
10. Shared CAS is immutable to workload.
11. Workspace is unique per attempt.
12. Retry gets fresh workspace.
13. Host environment is sanitized.
14. Secrets are late-bound and never persisted in normal local state.
15. Logs are bounded and recoverable.
16. Output capture follows declarations.
17. CAS upload alone does not finalize job.
18. Completion is idempotent.
19. Stale completion is rejected by control plane.
20. Agent cleanup does not rewrite job outcome.
21. Restart never silently inherits old authority.
22. Reconnect resynchronizes active attempts.
23. Hard lease expiry eventually stops disconnected work.
24. General runners cannot self-grant privileged trust.
25. Platform-specific behavior lives in adapters.
26. Signing worker remains separate from general runner.
27. Device control uses separate device lease.
28. In-memory state is not sole authority.
29. Local persisted state is recovery aid only.
30. Forgeyard should dogfood its own agent runtime.

---

# 292. Final Target Architecture

```text
                    Forgeyard Daemon
                          │
                    Authenticated QUIC
                          │
                          ▼
                    Forgeyard Agent
                          │
          ┌───────────────┼────────────────┐
          ▼               ▼                ▼
      Registration    Capability       Heartbeat
                          │
                          ▼
                     Lease Offer
                          │
                   validate/dedupe
                          │
                          ▼
                      Accept
                          │
                          ▼
                    AttemptContext
                          │
                          ▼
                   Fetch CAS Inputs
                          │
                          ▼
                 Prepare Workspace
                          │
                          ▼
                Sandbox / Executor
                          │
                          ▼
                     Workload
                          │
                logs / phase events
                          │
                          ▼
                   Capture Outputs
                          │
                          ▼
                        CAS
                          │
                          ▼
                 Completion Request
                          │
                          ▼
                  Daemon validates
                          │
                  Accepted / Stale
                          │
                          ▼
                       Cleanup
```

---

# 293. Final Architectural Position

Agent authority:

```text
RunnerId
+
AgentSessionId
+
LeaseId
+
AttemptId
+
JobSpecId
```

Execution:

```text
validated lease
  ↓
verified CAS inputs
  ↓
fresh workspace
  ↓
sandbox/executor
  ↓
bounded logs
  ↓
declared outputs
  ↓
verified CAS upload
```

Finalization:

```text
same authority tuple
+
result
+
outputs
  ↓
daemon
  ↓
accepted / duplicate / stale
```

The key guarantee is:

> **A Forgeyard agent is powerful enough to execute untrusted workloads, but deliberately too weak to invent work, choose its own privileges, alter job semantics, or declare itself successful without the control plane validating its exact lease, attempt, session, specification, and outputs.**

---

# 294. New-Repository Sequence

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
