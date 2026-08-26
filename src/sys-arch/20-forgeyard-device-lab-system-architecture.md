# 20 — Forgeyard Device Lab System Architecture

**Document type:** Core Physical/Virtual Device Execution & Test Lab System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Physical devices, emulators, simulators, device discovery, capability reporting, DeviceLease semantics, scheduler integration, test execution, installation, launch, log/screenshot capture, reset/sanitization, quarantine, remote device farms, device topology, health, and reconciliation  
**Architecture style:** Device resources are schedulable capabilities attached to runners/device agents; Job/Attempt/Lease remains authoritative; DeviceLease adds exclusive scarce-resource ownership; physical/virtual devices are never a second execution authority  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on Scheduler, Runner/Agent, Sandbox/Executor, Transport/QUIC, Events/Reconciliation, Deployment, Dioxus UI, and mobile platform architecture. It supports Android and Apple device testing first, with an extensible model for other device classes.

---

# 1. Purpose

Forgeyard needs a production-grade device lab for workflows that cannot be validated adequately using normal host processes alone.

Examples:

```text
Android phones/tablets
Android emulators
iOS devices
iOS simulators
macOS-attached devices
embedded boards
edge devices
special hardware test fixtures
```

Typical tasks include:

```text
install application
launch application
run integration/UI tests
collect logs
capture screenshots/video
measure startup/performance
exercise Bluetooth/Wi-Fi/device APIs
validate camera/sensor/storage behavior
uninstall/reset application
return device to a known clean state
```

The central rule is:

> **A device is a schedulable scarce resource, not an execution authority. Jobs remain controlled by Forgeyard's normal Run → Job → Attempt → Lease model.**

A second rule is:

> **Every exclusive physical/virtual device assignment is represented by a `DeviceLease` tied to the authoritative `JobLease`.**

A third rule is:

> **A device is never returned to the available pool until reset/sanitization and health validation succeed.**

---

# 2. Architectural Position

```text
                    Eligible Job
                        │
                        ▼
                    Scheduler
                        │
            ┌───────────┴───────────┐
            ▼                       ▼
        Runner Match            Device Match
            │                       │
            └───────────┬───────────┘
                        ▼
              JobLease + DeviceLease
                        │
                        ▼
                  Device Agent/Runner
                        │
                        ▼
                 Device Executor
                        │
                        ▼
             Physical / Virtual Device
                        │
                        ▼
              Test Results / Artifacts
```

---

# 3. Goals

The device subsystem MUST:

1. define stable `DeviceId`;
2. support physical devices;
3. support emulators;
4. support simulators;
5. support device capability discovery;
6. support exclusive leasing;
7. tie DeviceLease to JobLease;
8. prevent concurrent use;
9. support scheduler hard constraints;
10. support device pools;
11. support Android;
12. support iOS;
13. support device health;
14. support reset/sanitization;
15. support quarantine;
16. support install/uninstall;
17. support launch/terminate;
18. support logs;
19. support screenshots;
20. support test artifacts;
21. support device reconnect;
22. support remote labs;
23. support lab topology/failure domains;
24. support emulator lifecycle;
25. support simulator lifecycle;
26. support USB/network connectivity;
27. support idempotent cleanup;
28. support reconciliation;
29. support observability;
30. remain subordinate to Run/Job authority.

---

# 4. Non-Goals

The device lab does not:

```text
replace scheduler
replace runner
replace deployment subsystem
provide arbitrary remote device shell as admin feature
become a generic MDM platform
```

---

# 5. Workspace Structure

```text
crates/device/
├── forgeyard-device/
├── forgeyard-device-model/
├── forgeyard-device-capability/
├── forgeyard-device-discovery/
├── forgeyard-device-store-api/
├── forgeyard-device-lease/
├── forgeyard-device-pool/
├── forgeyard-device-health/
├── forgeyard-device-reset/
├── forgeyard-device-artifact/
├── forgeyard-device-session/
├── forgeyard-device-executor/
├── forgeyard-device-android/
├── forgeyard-device-apple/
├── forgeyard-device-emulator/
├── forgeyard-device-simulator/
├── forgeyard-device-remote/
├── forgeyard-device-reconcile/
├── forgeyard-device-metrics/
└── forgeyard-device-testkit/
```

Device agent application:

```text
apps/forgeyard-device-agent/
```

---

# 6. DeviceId

```rust
pub struct DeviceId(Ulid);
```

Stable Forgeyard identity for a registered device.

---

# 7. Device Kind

```rust
pub enum DeviceKind {
    Physical,
    Emulator,
    Simulator,
    VirtualDevice,
    Embedded,
    Custom(DeviceKindId),
}
```

---

# 8. Device Platform

```rust
pub enum DevicePlatform {
    Android,
    Ios,
    Ipados,
    Tvos,
    Watchos,
    Embedded,
    Custom(DevicePlatformId),
}
```

---

# 9. Device Record

```rust
pub struct Device {
    pub id: DeviceId,
    pub platform: DevicePlatform,
    pub kind: DeviceKind,
    pub pool: DevicePoolId,
    pub state: DeviceState,
    pub capabilities: DeviceCapabilities,
    pub attached_to: DeviceHostRef,
}
```

---

# 10. Device State

```rust
pub enum DeviceState {
    Discovering,
    Available,
    Reserved,
    Preparing,
    InUse,
    Resetting,
    Degraded,
    Quarantined,
    Offline,
    Retired,
}
```

---

# 11. Available

May be leased.

---

# 12. Reserved

DeviceLease exists but job has not begun device actions.

---

# 13. Preparing

Reset/boot/install prerequisites.

---

# 14. InUse

Active attempt owns device.

---

# 15. Resetting

Not schedulable.

---

# 16. Degraded

Health issue; optionally schedulable only by explicit policy.

Recommended default:

```text
not schedulable
```

---

# 17. Quarantined

Automatically or manually removed from pool.

---

# 18. Offline

Not connected/reachable.

---

# 19. Retired

Permanently disabled.

---

# 20. DeviceHost

A device is managed by:

```text
forgeyard-agent
or
forgeyard-device-agent
```

---

# 21. DeviceHostRef

```rust
pub struct DeviceHostRef {
    pub runner: RunnerId,
    pub session: AgentSessionId,
}
```

---

# 22. Stable Device Identity

Avoid using only:

```text
adb serial
USB bus address
simulator UUID
```

as Forgeyard identity.

Retain native IDs as external identifiers.

---

# 23. Native Device Identity

```rust
pub struct DeviceExternalIdentity {
    pub platform: DevicePlatform,
    pub native_id: BoundedString,
}
```

---

# 24. Device Discovery

Device host periodically discovers attached/available devices.

---

# 25. Android Discovery

Potential sources:

```text
adb devices
emulator inventory
udev/system
```

---

# 26. Apple Discovery

Potential sources:

```text
xcrun simctl
devicectl
platform tooling
```

exact implementation adapter-local.

---

# 27. Discovery Does Not Authorize

Discovery reports candidate device.

Control plane registers/associates it.

---

# 28. Capability Discovery

Device capability report includes:

```text
platform
OS version
architecture/ABI
model
screen
GPU class if relevant
hardware features
emulator/simulator
connection type
```

---

# 29. DeviceCapabilities

```rust
pub struct DeviceCapabilities {
    pub platform: DevicePlatform,
    pub os_version: DeviceOsVersion,
    pub architectures: BTreeSet<DeviceArchitecture>,
    pub features: BTreeSet<DeviceFeature>,
    pub form_factor: DeviceFormFactor,
}
```

---

# 30. Device Feature

Examples:

```text
camera
Bluetooth
NFC
GPS
biometrics
accelerometer
hardware AV1
hardware H264
hardware H265
```

---

# 31. Feature Verification

Capabilities should be probed where practical.

Do not rely only on marketing/model-name database.

---

# 32. Device Form Factor

```rust
pub enum DeviceFormFactor {
    Phone,
    Tablet,
    Desktop,
    Tv,
    Watch,
    Embedded,
    Other,
}
```

---

# 33. Connection Type

```rust
pub enum DeviceConnection {
    Usb,
    Network,
    LocalVirtual,
    RemoteFarm,
}
```

---

# 34. Device Pool

```rust
pub struct DevicePool {
    pub id: DevicePoolId,
    pub name: DevicePoolName,
    pub platform: DevicePlatform,
    pub trust: DeviceTrustClass,
}
```

---

# 35. Pool Purpose

Examples:

```text
android-general
android-release-test
ios-simulators
physical-ios
bluetooth-hardware
performance-lab
```

---

# 36. Device Trust

```rust
pub enum DeviceTrustClass {
    GeneralTest,
    TrustedInternal,
    SensitiveRelease,
}
```

---

# 37. Device Trust Assignment

Admin/server-controlled.

Device cannot self-promote.

---

# 38. Job Requirement

Pipeline/JobSpec can declare:

```rust
pub struct DeviceRequirement {
    pub platform: DevicePlatform,
    pub kind: DeviceKindRequirement,
    pub min_os: Option<DeviceOsVersion>,
    pub architecture: Option<DeviceArchitecture>,
    pub features: BTreeSet<DeviceFeature>,
    pub pool: Option<DevicePoolSelector>,
}
```

---

# 39. Hard Scheduling Constraint

If device requirement unsatisfied:

```text
job cannot lease runner/device
```

---

# 40. Runner + Device Co-Placement

Some device work requires device physically attached to selected runner.

Scheduler must match jointly.

---

# 41. Joint Placement

```text
Eligible Job
  ↓
runner candidates
  ↓
device candidates attached to runners
  ↓
hard filters
  ↓
score
```

---

# 42. DeviceLease

```rust
pub struct DeviceLease {
    pub id: DeviceLeaseId,
    pub device: DeviceId,
    pub job_lease: LeaseId,
    pub attempt: JobAttemptId,
    pub runner: RunnerId,
    pub agent_session: AgentSessionId,
    pub epoch: DeviceLeaseEpoch,
    pub expires_at: Timestamp,
}
```

---

# 43. DeviceLeaseId

```rust
pub struct DeviceLeaseId(Ulid);
```

---

# 44. DeviceLease Epoch

Prevents stale reuse.

---

# 45. Atomic Lease

JobLease + DeviceLease + resource reservation should be created atomically where feasible.

---

# 46. Lease Invariant

One exclusive DeviceLease per physical device by default.

---

# 47. Shared Device

Only future explicit special cases.

Do not default to shared access.

---

# 48. Lease Expiry

Device lease expires with job lease or earlier.

---

# 49. Lease Renewal

Tied to authoritative JobLease renewal.

---

# 50. Stale DeviceLease

Cannot continue device control.

---

# 51. Device Session

```rust
pub struct DeviceSessionId(Ulid);
```

Represents one prepared ownership session for attempt.

---

# 52. Device Session Lifecycle

```text
Reserved
  ↓
Prepare
  ↓
DeviceSession
  ↓
Execute
  ↓
Collect
  ↓
Reset
  ↓
Release
```

---

# 53. Prepare

May include:

```text
wake
boot
unlock test profile
reset app state
verify storage
verify connectivity
```

---

# 54. No User Personal Data

Dedicated test devices only.

---

# 55. Physical Device Policy

Production lab devices should not be normal employee/user phones.

---

# 56. Sanitization

Mandatory after each attempt according to class.

---

# 57. Reset Levels

```rust
pub enum DeviceResetLevel {
    AppOnly,
    Workspace,
    UserProfile,
    FullFactory,
    SnapshotRestore,
}
```

---

# 58. Default Android Physical Reset

Potential:

```text
uninstall app
clear test files
stop processes
clear log buffers as appropriate
restore network state
```

Full factory reset only when needed.

---

# 59. Emulator Reset

Prefer:

```text
immutable snapshot
  ↓
restore
```

---

# 60. Simulator Reset

Delete/recreate or reset state from controlled baseline.

---

# 61. Reset Verification

After reset run health probe.

---

# 62. Reset Failure

Device becomes:

```text
Quarantined
```

not Available.

---

# 63. Quarantine

```rust
pub struct DeviceQuarantine {
    pub device: DeviceId,
    pub reason: QuarantineReason,
    pub since: Timestamp,
}
```

---

# 64. Quarantine Reasons

```text
reset failed
device disconnected repeatedly
storage failure
test contamination
battery/thermal issue
ADB instability
simulator corruption
manual
```

---

# 65. Automatic Quarantine

After configurable repeated infrastructure failures.

---

# 66. User Test Failure

Must not quarantine device merely because test assertion failed.

---

# 67. Failure Classification

Distinguish:

```text
TestFailure
DeviceInfrastructure
DeviceDisconnected
InstallFailure
LaunchFailure
ResetFailure
```

---

# 68. Device Health

```rust
pub struct DeviceHealth {
    pub status: HealthStatus,
    pub battery: Option<Percent>,
    pub temperature: Option<Temperature>,
    pub storage_free: ByteSize,
    pub connection: DeviceConnectionHealth,
}
```

---

# 69. Physical Health

Potential checks:

```text
battery
thermal
USB stability
storage
screen responsiveness
```

---

# 70. Virtual Health

```text
boot success
snapshot validity
disk
emulator process
```

---

# 71. Battery Policy

Avoid scheduling long tests below configured threshold unless powered.

---

# 72. Thermal Policy

Avoid performance benchmarks on thermally throttled device.

---

# 73. Performance Lab

Dedicated controlled devices.

---

# 74. Device Clock

Can affect app tests.

Reset/sync according to test requirements.

---

# 75. Network State

Must be explicit.

---

# 76. Network Profiles

```rust
pub enum DeviceNetworkProfile {
    HostDefault,
    Offline,
    Wifi,
    CellularSimulated,
    Restricted,
    Custom(DeviceNetworkProfileId),
}
```

---

# 77. Emulator Network Simulation

Optional:

```text
latency
packet loss
bandwidth
```

---

# 78. Physical Network Control

Capability-dependent.

Do not claim full network simulation everywhere.

---

# 79. Bluetooth

Physical device tests may require Bluetooth hardware.

---

# 80. Bluetooth Capability

Hard device requirement.

---

# 81. External Peripheral

Future device fixture abstraction.

---

# 82. Device Fixture

Examples:

```text
Bluetooth peer
USB accessory
IoT board
camera fixture
```

---

# 83. Fixture Lease

Could use same scarce-resource lease model.

---

# 84. Baseline

Device Lab v1 focuses on one primary device lease.

---

# 85. Device Executor

```rust
#[async_trait]
pub trait DeviceExecutor {
    async fn prepare(
        &self,
        request: DevicePrepareRequest,
    ) -> Result<PreparedDeviceSession, DeviceError>;

    async fn execute(
        &self,
        session: &PreparedDeviceSession,
        action: DeviceAction,
    ) -> Result<DeviceActionResult, DeviceError>;

    async fn cleanup(
        &self,
        session: PreparedDeviceSession,
    ) -> Result<DeviceCleanupResult, DeviceError>;
}
```

---

# 86. DeviceAction

```rust
pub enum DeviceAction {
    Install(DeviceArtifactRef),
    Uninstall(ApplicationId),
    Launch(LaunchSpec),
    Stop(ApplicationId),
    RunTest(DeviceTestSpec),
    CaptureScreenshot,
    CaptureLogs,
    Reboot,
}
```

---

# 87. Arbitrary Shell

Not a general DeviceAction.

---

# 88. Platform-Specific Extension

Typed adapter-specific action may exist but policy-controlled.

---

# 89. Android Executor

Uses controlled Android tooling.

---

# 90. Android Inputs

```text
APK/AAB-derived test artifact
package name
instrumentation/test runner
device requirements
```

---

# 91. APK Install

Verify exact artifact digest before install.

---

# 92. AAB

AAB normally needs generation of device-specific APK set before install.

That transformation is explicit and evidence-linked.

---

# 93. Android App Identity

```rust
pub struct AndroidPackageName(BoundedString);
```

---

# 94. Android Launch

Explicit activity/intent.

---

# 95. Android Test Types

```text
instrumentation
UI test
integration test
benchmark
smoke
```

---

# 96. Logcat

Capture filtered logs.

---

# 97. Screenshot

Stored as artifact.

---

# 98. Screen Recording

Optional artifact.

---

# 99. Android Tombstones/Crash

Sensitive diagnostic artifact policy.

---

# 100. ADB Server Isolation

Avoid one uncontrolled global ADB server if possible.

---

# 101. ADB Ownership

Device agent owns host ADB interaction.

---

# 102. ADB Keys

Protected host credential.

---

# 103. ADB Over Network

Allowed only explicit trusted lab network.

---

# 104. Android Emulator

Immutable AVD definition.

---

# 105. Emulator Template

```rust
pub struct EmulatorTemplateId(Digest);
```

---

# 106. Emulator Template Inputs

```text
system image digest/version
device profile
CPU arch
storage
snapshot
```

---

# 107. Emulator Instance

Ephemeral per lease where capacity allows.

---

# 108. Emulator Snapshot

Restore baseline before/after use.

---

# 109. Emulator Scheduler

Device requirement can instantiate emulator rather than wait for pre-running device.

---

# 110. Provision-On-Demand

Optional.

---

# 111. Emulator Capacity

Host CPU/memory/GPU limits included.

---

# 112. Nested Virtualization

Capability requirement.

---

# 113. iOS Simulator

Runs on real macOS host.

---

# 114. iOS Simulator Template

OS/runtime/device model.

---

# 115. Simulator Instance

Ephemeral per attempt preferred.

---

# 116. Simulator Reset

Erase/delete/recreate.

---

# 117. Physical iOS Device

Requires real macOS host/tooling.

---

# 118. Apple Device Pairing

Host-managed secure pairing state.

---

# 119. Provisioning

Development/test signing/profile as required.

---

# 120. Production Signing

Still separate restricted signing subsystem.

---

# 121. iOS Install

Exact signed development/test artifact.

---

# 122. iOS Logs

Collect via Apple tooling.

---

# 123. XCTest/UI Test

Mapped into device test action.

---

# 124. Apple Screenshot

Artifact.

---

# 125. Apple Device Trust

Dedicated lab devices.

---

# 126. Physical Device Unlock

Avoid requiring manual operator unlock in normal automated lab.

Use managed lab setup within platform rules.

---

# 127. Device Secrets

Test account credentials via SecretRef.

---

# 128. Secret Injection

Never persist on device longer than needed.

---

# 129. Test Accounts

Dedicated non-production where possible.

---

# 130. Production Credentials

Never put on untrusted proposal device tests by default.

---

# 131. App Data Cleanup

Mandatory.

---

# 132. Keychain/Credential Storage Cleanup

Platform-specific reset.

---

# 133. Device Files

Declared test directories only.

---

# 134. Screenshot Privacy

May contain sensitive screen content.

Artifact classification.

---

# 135. Device Log Privacy

Same.

---

# 136. Artifact Types

```rust
pub enum DeviceArtifactKind {
    Screenshot,
    ScreenRecording,
    DeviceLog,
    TestReport,
    CrashReport,
    PerformanceTrace,
    DiagnosticBundle,
}
```

---

# 137. Device Artifact Storage

CAS bytes + metadata.

---

# 138. Retention

Per project/security policy.

---

# 139. Device Test Result

```rust
pub struct DeviceTestResult {
    pub outcome: DeviceTestOutcome,
    pub artifacts: Vec<DeviceArtifactRef>,
    pub duration: Duration,
}
```

---

# 140. DeviceTestOutcome

```rust
pub enum DeviceTestOutcome {
    Passed,
    Failed,
    InfrastructureFailure,
    DeviceLost,
    TimedOut,
    Cancelled,
}
```

---

# 141. Mapping to Job FailureClass

`Failed` -> TestFailure.

Infrastructure issues -> Infrastructure/Device class.

---

# 142. Device Disconnect

Active attempt:

```text
device status lost
  ↓
grace if reconnect possible
  ↓
DeviceLost
```

---

# 143. Reconnect Grace

Short and configurable.

---

# 144. Device Reappears

Must match same native identity/session/lease.

---

# 145. USB Re-enumeration

Native path may change.

Stable external serial identity helps.

---

# 146. Stale Device Action

Rejected if DeviceLease mismatch.

---

# 147. Device Agent Restart

New AgentSessionId.

Old DeviceLease cannot silently continue.

---

# 148. Orphan Device Cleanup

On restart:

```text
discover devices
  ↓
compare persisted lease/control state
  ↓
reset unknown/orphan sessions
```

---

# 149. Local Device State

Minimal:

```text
DeviceId
native identity
lease/session
phase
```

not authority.

---

# 150. Device Registration

Agent sends device inventory/capability digest.

---

# 151. Inventory Update

On:

```text
attach
detach
OS update
emulator image change
health change
```

---

# 152. Heartbeat

Include compact device summary.

---

# 153. No Huge Device Detail Every Heartbeat

Send digest/change update.

---

# 154. Scheduler Device Snapshot

Contains:

```text
available/in-use
capability summary
pool
host
health
```

---

# 155. Device Scarcity

Scheduler score preserves rare devices.

---

# 156. Scarcity Examples

```text
latest iPhone physical
old Android API
Bluetooth-capable hardware
specific GPU/codec
```

---

# 157. Anti-Affinity Retry

Infrastructure retry prefers different device when possible.

---

# 158. Test Retry on Same Device

May be useful for reproducibility diagnostics.

Policy decides.

---

# 159. Flaky Device Score

Track infrastructure reliability.

---

# 160. Device Reliability

```rust
pub struct DeviceReliability {
    pub recent_success_rate: Ratio,
    pub disconnect_rate: Ratio,
    pub reset_failure_rate: Ratio,
}
```

---

# 161. Scheduler Score

Reliable devices preferred.

---

# 162. Flaky Test vs Flaky Device

Keep separate metrics.

---

# 163. Quarantine Threshold

Based on infrastructure failures, not test assertions.

---

# 164. Manual Quarantine

Admin.

---

# 165. Manual Return to Service

Requires doctor/reset pass.

---

# 166. Device Doctor

```text
forgeyard device doctor
```

---

# 167. Doctor Checks

```text
connection
battery
storage
ADB/Apple tooling
install/uninstall
launch
screenshot
reset
```

---

# 168. Deep Doctor

Can run sacrificial test app.

---

# 169. Doctor Never Uses Production App Secret

Synthetic fixture.

---

# 170. Device Lab Topology

```rust
pub struct DeviceLabSite {
    pub id: DeviceLabSiteId,
    pub region: RegionId,
    pub failure_domain: FailureDomainId,
}
```

---

# 171. Site

Physical lab/location.

---

# 172. Rack/Host

Optional lower-level topology.

---

# 173. Failure Domain

Useful for independent reproduction/device testing.

---

# 174. Remote Device Lab

Device agent connects outbound to control plane.

---

# 175. NAT

Same advantage as agents.

---

# 176. Remote Farm Adapter

Future integration with external device clouds.

---

# 177. External Device Cloud

Could include providers like generic mobile testing farms.

Keep adapter-neutral.

---

# 178. External Farm Semantics

Map:

```text
device selection
session
upload
test
artifacts
```

to DeviceExecutor abstraction.

---

# 179. External Farm Identity

Still use Forgeyard `DeviceSession`/provider session refs.

---

# 180. External Farm Availability

Provider reports matching capabilities.

---

# 181. External Provider Cost

Soft scheduler/selection dimension.

---

# 182. Local Device First

Policy can prefer owned lab then external farm.

---

# 183. Device Session Limits

Per provider/account.

---

# 184. Cost Guard

Quota/policy.

---

# 185. Device Pool Quota

Tenant/project limits.

---

# 186. Reservation

Device resources count toward scheduler quota.

---

# 187. Long Device Tests

Queue wait can be significant.

---

# 188. Queue Timeout

Normal Job queue timeout applies.

---

# 189. Device Unavailable Diagnostic

Examples:

```text
NO_MATCHING_DEVICE
DEVICE_POOL_OFFLINE
DEVICE_QUOTA_BLOCKED
DEVICE_HEALTH_UNAVAILABLE
```

---

# 190. UI Scheduling Explain

Show why no device matches.

---

# 191. Device Lab UI

Pages:

```text
Devices
Pools
Hosts
Sessions
Health
Quarantine
Templates
```

---

# 192. Device List

Columns/cards:

```text
model
platform
OS
kind
pool
state
host
health
current job
```

---

# 193. Device Detail

Tabs:

```text
Overview
Capabilities
Current Lease
History
Health
Doctor
Artifacts
```

---

# 194. Device Session Page

Shows:

```text
JobId
AttemptId
DeviceLeaseId
start
actions
artifacts
cleanup
```

---

# 195. Quarantine UI

Reason/history/release action.

---

# 196. Emulator Templates UI

Manage allowed templates/images.

---

# 197. Device Action UI

No arbitrary shell.

---

# 198. Admin Actions

```text
quarantine
retire
doctor
reset
```

---

# 199. Reset Confirmation

Potentially destructive to device state.

---

# 200. Mobile Device Lab UI

Status/admin basics.

---

# 201. Device API

Potential:

```text
GET  /v1/devices
GET  /v1/devices/{id}
GET  /v1/device-pools
POST /v1/devices/{id}/quarantine
POST /v1/devices/{id}/reset
POST /v1/devices/{id}/doctor
```

---

# 202. No Public Lease Creation

Scheduler/service owns normal DeviceLease.

---

# 203. Manual Reservation

Could exist for debugging, explicit admin/dev workflow.

---

# 204. Manual Reservation Scope

Short-lived, audited, no production trust by default.

---

# 205. Interactive Device Debug

Future feature.

---

# 206. Interactive Debug Isolation

Separate from CI attempt.

---

# 207. Never Reuse Production Device Session

Debug gets own DeviceLease.

---

# 208. Artifact Install Source

CAS/release artifact.

---

# 209. Local Upload

Development mode may upload local artifact into CAS first.

---

# 210. Device App Version Verification

After install query app package/version.

---

# 211. Binary Digest Verification

Where platform allows direct file verification.

---

# 212. Installation Evidence

Store app version/package ID/install result.

---

# 213. Launch Readiness

Wait for process/activity.

---

# 214. Test Readiness

Optional app-specific readiness probe.

---

# 215. Screenshot Timing

Explicit action/test framework.

---

# 216. Video Capture

Can affect performance.

Mark test evidence accordingly.

---

# 217. Performance Benchmarking

Dedicated profile.

---

# 218. Performance Controls

```text
battery/power
temperature
background apps
network
screen brightness
```

where controllable.

---

# 219. Performance Evidence

Record environment/device state.

---

# 220. Benchmark Reproducibility

Device physical variance acknowledged.

Use statistical runs, not deterministic claim.

---

# 221. Hardware Codec Testing

Capability-aware device selection.

---

# 222. Example

```text
requires Android AV1 hardware decoder
```

hard requirement.

---

# 223. Camera/Sensor Testing

Physical only unless simulator support adequate.

---

# 224. Physical Requirement

```rust
DeviceKindRequirement::Physical
```

---

# 225. Emulator Requirement

Explicit.

---

# 226. Device Matrix

Pipeline can expand:

```text
Android 12 physical
Android 14 emulator
Android 16 physical
iOS current simulator
```

bounded by matrix rules.

---

# 227. Matrix Scheduling

Each is normal Job.

---

# 228. Device Parallelism

Depends pool capacity.

---

# 229. Fail-Fast

Pipeline policy can cancel remaining matrix after critical failures.

---

# 230. Device Result Aggregation

Normal run/job aggregate.

---

# 231. Device Deployment vs Device Test

Test lab uses temporary app/session.

Deployment subsystem manages long-lived desired device state.

---

# 232. Device Fleet Deployment

Uses Device Lab primitives but different lifecycle.

---

# 233. Shared Device Model

`DeviceId`, health, capabilities can be reused.

---

# 234. Deployment DeviceLease

Long-lived deployment may not use same short test lease semantics.

Keep separate deployment desired state.

---

# 235. Device Lab Security Boundary

Untrusted test application must not access:

```text
device agent credential
other device sessions
host secrets
production accounts
```

---

# 236. Host Isolation

Host tools run within runner sandbox where feasible.

---

# 237. USB Passthrough

Only device executor/controlled helper.

---

# 238. Docker Socket

Never needed.

---

# 239. ADB Credential Exposure

Workload should not receive raw host ADB private key.

---

# 240. Device Command Proxy

Workload/test framework requests typed actions through executor.

---

# 241. Test Framework Escape

If arbitrary platform test tool execution required, run in sandboxed host process with device-scoped access.

---

# 242. Device Access Handle

Only current leased device visible.

---

# 243. Multiple Connected Devices

ADB/tool invocation always pins exact native device ID.

---

# 244. No Default "first device"

Critical.

---

# 245. Apple Tooling

Always target exact device/simulator ID.

---

# 246. Device Filesystem

Avoid broad pull/push outside declared test locations.

---

# 247. Screen Lock/PIN

Dedicated lab policy.

---

# 248. Biometric Tests

Use simulator/test APIs where available; physical controlled fixture if needed.

---

# 249. Account Cleanup

Remove test account tokens/session after attempt.

---

# 250. Network Credentials

Lab Wi-Fi credentials provisioned by host/device management, not workload.

---

# 251. VPN

Explicit lab capability if required.

---

# 252. Device Reboot

Allowed only current lease.

---

# 253. Firmware Update

Administrative maintenance, not normal job action.

---

# 254. OS Update

Maintenance workflow.

---

# 255. Device Maintenance State

Could use:

```text
Quarantined/Retired
```

or future Maintenance state.

---

# 256. Recommended Add

```rust
DeviceState::Maintenance
```

if operational needs justify.

---

# 257. Device Inventory Audit

Record registration/retirement/trust changes.

---

# 258. Device Lease Audit

Operational history.

---

# 259. Sensitive Test Artifact Audit

Downloads permission-controlled.

---

# 260. Events

```text
DeviceDiscovered
DeviceRegistered
DeviceAvailable
DeviceLeased
DeviceLost
DeviceResetStarted
DeviceResetSucceeded
DeviceQuarantined
DeviceRecovered
```

---

# 261. Reconciliation

Device reconciler checks:

```text
DeviceLease expired
device marked InUse but no JobLease
JobLease active but device missing
reset stuck
quarantined device still schedulable
host session changed
```

---

# 262. Lease Reconcile

If JobLease terminal but DeviceLease active:

```text
cleanup/reset
```

---

# 263. Host Restart Reconcile

Old sessions reset.

---

# 264. Device Offline Reconcile

Mark unavailable and fail/lost current attempt according to grace.

---

# 265. Emulator Leak Reconcile

Kill/delete orphan emulator instances.

---

# 266. Simulator Leak Reconcile

Delete orphan simulator.

---

# 267. Artifact Upload Reconcile

Retry diagnostic artifact upload without rerunning test if possible.

---

# 268. Health Reconcile

Quarantined device periodically checked/manual.

---

# 269. Device Metrics

```text
device_available
device_in_use
device_quarantined
device_disconnects
device_reset_failures
device_queue_wait
device_session_duration
device_test_infra_failure
```

---

# 270. Labels

Low-cardinality:

```text
platform
kind
pool class
OS major
```

---

# 271. No DeviceId Metric Label

Use traces/logs.

---

# 272. Tracing

```text
device.reserve
device.prepare
device.install
device.test
device.collect
device.reset
device.release
```

---

# 273. Logs

Agent/device host operational logs separate from device app logs.

---

# 274. Health

Device lab health summary:

```text
host availability
pool capacity
quarantined count
tooling health
```

---

# 275. Doctor

Device-specific.

---

# 276. SLO Examples

```text
device lease acquisition latency
reset success rate
device infra failure rate
```

---

# 277. Backpressure

Device pools naturally bounded by number of devices.

---

# 278. Queue Fairness

Normal scheduler fairness.

---

# 279. Reserved Device Classes

Sensitive-release devices only authorized jobs.

---

# 280. Device Pool Authorization

Policy can restrict project/tenant access.

---

# 281. Multi-Tenancy

Cross-tenant physical device reuse requires strict reset/sanitization.

---

# 282. Hostile Tenant Risk

Dedicated pools may be required.

---

# 283. High-Assurance Policy

```text
one tenant per device pool
```

or strong factory/snapshot reset.

---

# 284. Device Forensics

Do not retain app/data unless diagnostic policy.

---

# 285. Failure Diagnostics

Can retain:

```text
screenshot
logs
crash report
```

with retention/classification.

---

# 286. Full Device Image

Avoid as normal diagnostic artifact.

---

# 287. Test Data

Synthetic.

---

# 288. Device Templates

Emulator/simulator templates are immutable descriptors.

---

# 289. Template Version

```rust
pub struct DeviceTemplateVersion(Digest);
```

---

# 290. Template Change

New version.

---

# 291. Managed System Images

Pin version/digest.

---

# 292. Mutable "latest"

Not strict-mode authority.

---

# 293. Image Download

Resolve/fetch phase.

---

# 294. Test Realization

Can be network-denied except app-under-test requirements.

---

# 295. Emulator Host GPU

Capability requirement.

---

# 296. Headless Mode

Default CI.

---

# 297. Visible Mode

Developer/debug.

---

# 298. Display Capture

Virtual frame buffer/recording.

---

# 299. Android Hardware Acceleration

Host capability.

---

# 300. Apple Simulator Runtime

macOS/Xcode capability.

---

# 301. Real Device Availability

Scheduler diagnostics clear.

---

# 302. External Farm Adapter Cost

Soft score.

---

# 303. External Farm Credentials

SecretRef/workload identity.

---

# 304. External Farm Security

Upload only exact artifact/test package.

---

# 305. External Farm Data Retention

Provider policy recorded/configured.

---

# 306. External Farm Cleanup

Session close/delete.

---

# 307. Compliance

Some tenants may disallow external device clouds.

Policy constraint.

---

# 308. Device Capability Registry

Typed known capabilities.

---

# 309. Custom Capabilities

Namespaced.

---

# 310. No Trust in Free-Form Labels

Labels cannot grant privileged access.

---

# 311. Device Labels

Supplementary:

```text
lab-bench-3
codec-test
```

not core trust.

---

# 312. Scheduler Explain

Show device filter reasons.

---

# 313. Resource Reservation

Host CPU/memory + DeviceLease both reserved.

---

# 314. Emulator Reservation

Consumes substantial host resources.

---

# 315. Physical Device Reservation

May consume little host compute but exclusive device.

---

# 316. Test Services

Host-side mock/server can run in job sandbox and device connects over isolated network.

---

# 317. Device-to-Host Networking

Explicit ephemeral endpoint.

---

# 318. Port Allocation

Typed/managed.

---

# 319. No Accidental Host LAN Exposure

Firewall/network namespace where possible.

---

# 320. TLS for Test Service

Optional depending test.

---

# 321. Localhost Semantics

Device localhost != host localhost.

Architecture must explicitly expose host service endpoint.

---

# 322. Reverse Port Forward

Android adb reverse can be controlled adapter action.

---

# 323. Apple Equivalent

Adapter-specific.

---

# 324. Device Timeouts

```text
boot timeout
install timeout
launch timeout
test timeout
reset timeout
```

---

# 325. Timeout Mapping

Infrastructure vs workload.

---

# 326. Cancellation

Agent receives job cancellation.

---

# 327. Cancellation Sequence

```text
stop test
stop app
collect bounded diagnostics
reset
release device
```

---

# 328. Hard Kill

If host test process stuck, kill host process tree.

---

# 329. Device App Termination

Platform-specific.

---

# 330. Cleanup Despite Cancellation

Mandatory best effort.

---

# 331. Device Release Ack

Control plane only marks Available after reset success.

---

# 332. Cleanup Failure

Quarantine.

---

# 333. Device History

Append:

```text
leases
tests
infra failures
quarantine
maintenance
```

---

# 334. Historical Device Data

Useful for reliability.

---

# 335. Privacy

No test-account secrets in history.

---

# 336. Device Store

Logical entities:

```text
devices
device_external_ids
device_capabilities
device_pools
device_leases
device_sessions
device_health
device_quarantines
device_templates
device_history
```

---

# 337. CAS

Stores:

```text
screenshots
videos
logs
reports
device diagnostics
```

---

# 338. Device Control Protocol

Separate typed message family from agent control.

---

# 339. Wire Messages

Examples:

```text
DeviceInventoryUpdate
DeviceLeaseAssigned
DevicePrepare
DeviceAction
DeviceCleanup
DeviceHealthUpdate
```

---

# 340. No Arbitrary Command Message

Critical.

---

# 341. Device Agent Registration

Role:

```text
PeerRole::DeviceAgent
```

---

# 342. Device Agent Permissions

Can manage registered devices only.

---

# 343. Device Agent Cannot Schedule

No authority.

---

# 344. Device Agent Cannot Mark Job Success

Reports action/test result under current lease.

---

# 345. Result Authority

Run/Job service validates:

```text
JobLease
DeviceLease
AttemptId
AgentSessionId
DeviceId
```

---

# 346. Device Artifacts

Output refs verified before job completion.

---

# 347. Test Report Standardization

Adapters can normalize:

```text
JUnit-like test cases
platform-native reports
```

---

# 348. Internal Test Model

```rust
pub struct TestCaseResult {
    pub name: TestCaseName,
    pub outcome: TestCaseOutcome,
    pub duration: Duration,
}
```

---

# 349. Platform Native Reports

Stored alongside normalized summary.

---

# 350. Screenshot Association

Can bind to failing test case.

---

# 351. Flaky Test Detection

Separate future analysis subsystem.

---

# 352. Device Retry Policy

Infrastructure failure retry may select new device.

---

# 353. Test Failure Retry

Only pipeline policy.

---

# 354. Same-Device Reproduction

Optional diagnostic command.

---

# 355. Device Reservation UI

Admin/debug only.

---

# 356. Manual Device Session

Never overlaps scheduled lease.

---

# 357. Idle Timeout

Manual reservation auto-expires.

---

# 358. Audit

Manual access audited.

---

# 359. Implementation Phase 1 — Core Device Model

Implement:

```text
DeviceId
DeviceState
DeviceCapabilities
DevicePool
DeviceLease
```

---

# 360. Phase 2 — Scheduler Integration

Joint runner/device matching.

---

# 361. Phase 3 — Android Physical

Discovery, lease, install, test, logs, reset.

---

# 362. Phase 4 — Android Emulator

Template lifecycle/snapshot.

---

# 363. Phase 5 — Device Health/Quarantine

Reliability and reset policy.

---

# 364. Phase 6 — Apple Simulator

macOS host integration.

---

# 365. Phase 7 — Physical Apple Device

Install/test/log/cleanup.

---

# 366. Phase 8 — Device Artifacts/UI

Screenshots/logs/reports.

---

# 367. Phase 9 — Remote Labs

Device agent sites/farms.

---

# 368. Phase 10 — External Device Cloud

Optional adapters.

---

# 369. Phase 11 — Performance/Hardware Tests

Codec/Bluetooth/sensor capability-aware tests.

---

# 370. Phase 12 — Hardening

Failure injection, quarantine tuning, multi-tenant sanitization, scale.

---

# 371. Acceptance Tests

1. Device has stable Forgeyard DeviceId.
2. Native adb/simulator identifier is not sole business identity.
3. Device capabilities are typed and versioned.
4. Job requiring device cannot schedule without matching device.
5. Scheduler matches runner and attached device jointly.
6. DeviceLease is tied to JobLease.
7. One physical device cannot have two exclusive leases.
8. Stale DeviceLease cannot control device.
9. New AgentSession cannot inherit old DeviceLease silently.
10. Device is not Available until cleanup/reset succeeds.
11. Reset failure quarantines device.
12. Test assertion failure does not automatically quarantine device.
13. Device infrastructure failures are distinct from test failures.
14. Android install uses exact artifact digest.
15. All adb commands target exact leased device.
16. No default "first connected device" behavior exists.
17. Emulator template is immutable/versioned.
18. Emulator orphan is reconciled/cleaned.
19. iOS simulator runs only on compatible macOS host.
20. Physical Apple device uses exact leased device identity.
21. Production signing keys are never exposed through Device Lab.
22. Device screenshots/logs are CAS artifacts with retention policy.
23. Secret test credentials are removed after session.
24. Device disconnect can transition attempt to infrastructure lost/failure.
25. Retry after device infrastructure failure can prefer another device.
26. Quarantined device cannot be scheduled.
27. Manual re-enable requires health/reset success.
28. External farm adapter cannot bypass Forgeyard job/device lease semantics.
29. Device pool access is policy-controlled.
30. Cross-tenant reuse follows explicit sanitization policy.
31. Cancellation still performs cleanup/reset.
32. Device agent cannot schedule jobs or declare authoritative success.
33. Device events/reconciliation recover from missed updates.
34. Same device model works standalone/distributed.
35. Forgeyard's Android/iOS client tests use this subsystem.

---

# 372. Production Readiness Gates

Do not call Device Lab production-ready until:

```text
DeviceLease authority proven
scheduler joint placement works
Android physical lifecycle stable
reset/quarantine tested
device artifact capture stable
agent restart/orphan cleanup works
device disconnect handling tested
multi-device targeting cannot select wrong device
secrets/data cleanup verified
health/doctor available
```

Apple physical devices, external farms, advanced hardware capabilities, and performance labs can mature incrementally.

---

# 373. Architectural Invariants

1. Device Lab does not replace Run/Job authority;
2. DeviceLease is subordinate to JobLease;
3. exclusive device use is default;
4. stale leases cannot control devices;
5. device host/session identity is validated;
6. device cannot self-promote trust;
7. scheduler jointly matches runner + device;
8. rare devices are scarcity-aware resources;
9. no "first device" implicit targeting;
10. exact native device identity is always selected;
11. device is unavailable during reset;
12. reset failure quarantines;
13. test failure is distinct from infrastructure failure;
14. device data/secrets are cleaned after attempt;
15. production signing keys never enter general device workflows;
16. device artifacts are immutable CAS objects;
17. screenshots/logs are potentially sensitive;
18. emulator/simulator templates are immutable;
19. orphan virtual devices are reconciled;
20. host restarts do not silently preserve old lease authority;
21. external device farms remain adapters;
22. external farm credentials are scoped SecretRefs;
23. physical lab multi-tenancy requires explicit sanitization policy;
24. deployment lifecycle remains separate from test session lifecycle;
25. device agent cannot schedule or authorize jobs;
26. arbitrary remote shell is not a Device Lab feature;
27. capability reports are typed;
28. health/quarantine state is authoritative server metadata;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its Device Lab.

---

# 374. Final Target Architecture

```text
                       JobSpec
                         │
               DeviceRequirement
                         │
                         ▼
                      Scheduler
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
           Runner                 Device
              │                     │
              └──────────┬──────────┘
                         ▼
              JobLease + DeviceLease
                         │
                         ▼
                Device Agent/Runner
                         │
                         ▼
                   DeviceExecutor
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         Physical     Emulator     Simulator
            │            │            │
            └────────────┼────────────┘
                         ▼
                  Test / Artifacts
                         │
                         ▼
                  Cleanup / Reset
                         │
                ┌────────┴────────┐
                ▼                 ▼
             Available        Quarantined
```

---

# 375. Final Architectural Position

Placement:

```text
Job requirements
+
runner capabilities
+
device capabilities
+
device pool/trust
+
resource availability
  ↓
atomic JobLease + DeviceLease
```

Execution:

```text
JobAttempt
+
JobLease
+
DeviceLease
+
AgentSessionId
  ↓
prepare exact device
  ↓
install exact artifact
  ↓
run typed test actions
  ↓
collect immutable artifacts
```

Cleanup:

```text
stop workload
  ↓
remove app/test data/secrets
  ↓
reset device
  ↓
health verify
  ↓
Available
```

Failure cleanup:

```text
reset/health fails
  ↓
Quarantined
```

The key guarantee is:

> **Forgeyard treats devices as scarce, stateful, potentially unreliable hardware resources that must be leased, validated, sanitized, and reconciled with the same rigor as compute runners—without ever allowing device-specific tooling to bypass the normal Job/Attempt/Lease security model.**

---

# 376. New-Repository Sequence

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
