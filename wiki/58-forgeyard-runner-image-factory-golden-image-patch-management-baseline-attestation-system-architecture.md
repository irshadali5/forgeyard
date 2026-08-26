# 58 — Forgeyard Runner Image Factory, Golden Image, Patch Management & Fleet Baseline Attestation System Architecture

**Document type:** Core Runner Image Factory, Golden Image, Patch Management, Fleet Baseline, Host Hardening, Attestation & Image Lifecycle System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** runner base-image construction, golden images, immutable machine templates, host hardening, OS patching, tool prewarming, image provenance, vulnerability remediation, image signing/attestation, rollout/canary, rollback, drift detection, runtime baseline verification, image retirement, emergency patching, and platform-specific runner-image lifecycle across Linux, Windows, macOS, Android/device hosts, and VM/container execution hosts  
**Architecture style:** Reproducible image build, immutable baseline, attested boot/runtime identity, patch-as-new-image, canary-first rollout, declarative fleet binding, drift detection, explicit trust state, and no in-place mutable snowflake runner as trusted baseline  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Runner/Agent, Sandbox/Executor, Runner Fleet Autoscaling, Security/Threat Model, Supply Chain, Update Delivery, Infrastructure-as-Code, Federation, Reliability, Cost/FinOps, Configuration, Artifact Registry, and Operations/DR. This subsystem provides the trusted host baseline beneath all runner execution.

---

# 1. Purpose

Forgeyard can schedule workloads onto many execution environments:

```text
Linux runners
Windows runners
macOS runners
VM-based executors
container hosts
GPU hosts
device-lab hosts
Android build/test hosts
confidential-compute hosts
self-hosted enterprise workers
ephemeral cloud workers
```

Those runners are part of the trust boundary.

A secure CI/CD system cannot assume that a host is safe merely because:

```text
it has the forgeyard-agent installed
it came from a cloud VM template
it passed registration once
it has the right labels
```

Runner hosts can drift because of:

```text
OS package updates
manual administrator changes
installed debugging tools
kernel changes
driver changes
malware
stale certificates
configuration drift
unapproved package installation
image rebuild differences
```

The central rule is:

> **A trusted runner must be attributable to an approved immutable baseline and must continuously prove enough of that baseline to retain its trust class.**

A second rule is:

> **Patching produces a new baseline image. Forgeyard does not treat long-lived in-place patched machines as equivalent to a reproducibly built and attested image unless an explicit lower-trust profile permits it.**

A third rule is:

> **Runner image trust is separate from runner availability. A healthy machine that cannot prove its approved baseline may still be schedulable only for lower-trust work—or quarantined entirely.**

---

# 2. Architectural Position

```text
                  Image Definition
                        │
                        ▼
                  Image Build Plan
                        │
                        ▼
                Reproducible Image Build
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
        Security Scan   Test      Hardening
            │           │           │
            └───────────┼───────────┘
                        ▼
                  Image Attestation
                        │
                        ▼
                  Approved Baseline
                        │
                        ▼
               Canary Fleet Rollout
                        │
                        ▼
                 Production Fleets
                        │
                        ▼
                 Runtime Attestation
                        │
                        ▼
                 Drift / Patch Cycle
```

---

# 3. Goals

The subsystem MUST:

1. define runner image identity;
2. define image-definition identity;
3. define baseline identity;
4. support reproducible image builds;
5. support Linux runner images;
6. support Windows runner images;
7. support macOS runner baselines;
8. support container-host images;
9. support GPU-host images;
10. support device-host baselines;
11. support immutable OS/package inputs;
12. support tool preinstallation without hidden correctness dependence;
13. support image provenance;
14. support image SBOM;
15. support vulnerability scanning;
16. support patch campaigns;
17. support canary rollout;
18. support rollback;
19. support image retirement;
20. support host attestation;
21. support drift detection;
22. support runtime trust downgrade;
23. support image/fleet bindings;
24. support emergency patching;
25. support multi-region replication;
26. support cost-aware prewarming;
27. support audit;
28. support UI/API/CLI;
29. support disaster recovery;
30. remain separate from job/toolchain correctness identity.

---

# 4. Non-Goals

This subsystem does not:

```text
replace application toolchain pinning
replace executor sandboxing
replace endpoint security products
replace OS vendor patching
replace autoscaling
replace package managers
replace device firmware management
```

It governs the runner-host baseline those systems rely upon.

---

# 5. Workspace Structure

```text
crates/runner-image/
├── forgeyard-runner-image/
├── forgeyard-runner-image-model/
├── forgeyard-runner-image-definition/
├── forgeyard-runner-image-build/
├── forgeyard-runner-image-hardening/
├── forgeyard-runner-image-attestation/
├── forgeyard-runner-image-scan/
├── forgeyard-runner-image-rollout/
├── forgeyard-runner-image-drift/
├── forgeyard-runner-image-patch/
├── forgeyard-runner-image-reconcile/
├── forgeyard-runner-image-health/
└── forgeyard-runner-image-testkit/
```

Provider/platform adapters:

```text
crates/runner-image-adapters/
├── forgeyard-image-linux/
├── forgeyard-image-windows/
├── forgeyard-image-macos/
├── forgeyard-image-aws-ami/
├── forgeyard-image-azure-gallery/
├── forgeyard-image-gcp-image/
├── forgeyard-image-libvirt/
├── forgeyard-image-kubernetes-node/
└── forgeyard-image-custom/
```

---

# 6. RunnerImageDefinitionId

```rust
pub struct RunnerImageDefinitionId(Digest);
```

Immutable identity of the normalized desired runner image definition.

---

# 7. RunnerImageId

```rust
pub struct RunnerImageId(Digest);
```

Immutable identity of the built machine image/baseline.

---

# 8. RunnerBaselineId

```rust
pub struct RunnerBaselineId(Digest);
```

Represents the approved operational baseline:

```text
RunnerImageId
+
hardening policy
+
agent version range
+
driver/firmware expectations
+
attestation policy
```

---

# 9. Runner Image Definition

```rust
pub struct RunnerImageDefinition {
    pub id: RunnerImageDefinitionId,
    pub platform: RunnerPlatform,
    pub base: BaseImageRef,
    pub packages: Vec<OsPackageRef>,
    pub hardening: HardeningProfileId,
    pub agent: AgentPackageRef,
    pub prewarm: Vec<PrewarmArtifactRef>,
}
```

---

# 10. Base Image Reference

Must resolve exactly.

Examples:

```text
Linux distribution image digest
Windows image generation
macOS installer/build identity
cloud marketplace image ID + verified publisher/version
```

Mutable aliases such as:

```text
ubuntu-latest
windows-latest
macos-latest
```

are never final baseline identity.

---

# 11. Image Definition Source

Can come from:

```text
RON
repository source
organization golden-image template
Forgeyard built-in template
```

---

# 12. Human Configuration

RON preferred.

Example:

```ron
(
    platform: Linux(
        distro: "arch",
        arch: "x86_64",
    ),
    hardening: "runner-high-assurance-v2",
    agent: "1.8.2",
)
```

---

# 13. Source Binding

Repository-backed definition binds exact:

```text
SourceSnapshotId
```

---

# 14. Image Build Plan

```rust
pub struct RunnerImageBuildPlanId(Digest);
```

Inputs:

```text
RunnerImageDefinitionId
base image digest
OS repository snapshots
package digests
hardening profile
agent artifact
image-builder version
platform-specific build tools
```

---

# 15. Reproducible Build Goal

Machine images are harder to make bit-for-bit reproducible than normal artifacts.

Therefore define:

```rust
pub enum ImageReproducibility {
    BitForBit,
    NormalizedFilesystem,
    EquivalentBaseline,
    ProviderGenerated,
    Unverified,
}
```

---

# 16. Honest Reproducibility

Cloud image formats may contain nondeterministic metadata.

Forgeyard must distinguish:

```text
identical bytes
vs
equivalent verified filesystem/configuration baseline
```

---

# 17. Image Build Worker

Dedicated high-trust worker.

---

# 18. Main Daemon

Never executes image-builder tooling directly.

---

# 19. Image Build Environment

Isolated.

Potential tools:

```text
qemu
libguestfs
packer-like adapters
cloud image builders
Windows image tooling
macOS virtualization tooling
```

But no specific external tool becomes architecture authority.

---

# 20. Base Image Provenance

Every base image records:

```text
publisher
version/build
digest/provider identity
acquired_at
signature/verification
```

---

# 21. Unknown Base Provenance

Cannot become high-trust baseline.

---

# 22. OS Repository Snapshot

For reproducibility, pin package repository state when practical.

Examples:

```text
Arch Linux repository snapshot
Debian snapshot
Ubuntu repository snapshot
Windows update catalog/build
```

---

# 23. Mutable Package Index

Not sufficient for protected baseline reproduction.

---

# 24. Package Manifest

```rust
pub struct RunnerImagePackageManifest {
    pub packages: Vec<ResolvedOsPackage>,
}
```

---

# 25. Resolved Package

Contains:

```text
name
version
architecture
digest/source
```

---

# 26. Agent Installation

Image contains exact Forgeyard agent package/digest.

---

# 27. Agent Auto-Update

Part 41 may update agent separately if policy permits.

---

# 28. Baseline Impact

If agent version is part of baseline contract, runtime update changes baseline observation.

---

# 29. Immutable Infrastructure Preference

For ephemeral runners:

```text
replace image
rather than
patch host in place
```

---

# 30. Long-Lived Physical Hosts

May require in-place patch workflows.

These receive separate trust semantics.

---

# 31. HostMutationClass

```rust
pub enum HostMutationClass {
    ImmutableReplace,
    ControlledPatch,
    ManualManaged,
}
```

---

# 32. High-Assurance Baseline

Prefer `ImmutableReplace`.

---

# 33. Controlled Patch

Allowed for:

```text
bare metal
macOS hardware
device hosts
scarce GPU systems
```

where replacement is expensive or impossible.

---

# 34. Controlled Patch Evidence

Must record exact changes.

---

# 35. ManualManaged

Lower trust by default.

---

# 36. Hardening Profile

```rust
pub struct HardeningProfileId(Digest);
```

---

# 37. Hardening Profile Includes

Examples:

```text
SSH policy
local users
sudo policy
firewall
kernel/sysctl
service allowlist
filesystem mount settings
audit configuration
agent permissions
debugging policy
USB/device restrictions
```

---

# 38. Hardening Policy

Platform-specific adapter + common security facts.

---

# 39. No Universal Hardening Script

Critical.

---

# 40. Linux Hardening

May include:

```text
minimal services
cgroup v2
namespace capability
seccomp support
read-only system areas where possible
no password SSH
limited sudo
host firewall
```

---

# 41. Windows Hardening

May include:

```text
service baseline
Defender/EDR policy integration
WinRM/RDP restrictions
PowerShell logging
local admin restrictions
firewall
code-signing policy
```

---

# 42. macOS Hardening

May include:

```text
FileVault where appropriate
SSH restrictions
SIP expectations
launch daemon baseline
developer tool permissions
keychain separation
```

---

# 43. macOS Reality

Physical Apple hardware may be long-lived.

Therefore runtime drift/patch evidence matters more than immutable VM replacement.

---

# 44. GPU Hosts

Driver/CUDA/ROCm state is baseline-sensitive.

---

# 45. DriverIdentity

```rust
pub struct DriverIdentity {
    pub vendor: DriverVendor,
    pub version: BoundedString,
    pub digest: Option<Digest>,
}
```

---

# 46. GPU Capability

Scheduler sees capability.

Image baseline verifies driver/tool stack.

---

# 47. Device-Lab Hosts

Host baseline distinct from device firmware state.

---

# 48. Android Device Host

Includes:

```text
ADB tools
USB policy
device-agent
SDK platform tools
```

---

# 49. Image Prewarming

May include large non-authoritative assets:

```text
container layers
toolchain packages
dependency mirror slices
SDKs
```

---

# 50. Critical Rule

Prewarmed software does not become correctness input merely because it exists on image.

---

# 51. Toolchain Resolution

Job still resolves exact `ToolchainDescriptorId`.

---

# 52. Prewarm Hit

Optimization only.

---

# 53. Prewarm Miss

Fetch/prepare exact required toolchain.

---

# 54. No Host PATH Leakage

Job environment uses declared toolchain paths.

---

# 55. Image SBOM

Generate SBOM for baseline.

---

# 56. RunnerImageSbomRef

```rust
pub struct RunnerImageSbomRef(EvidenceRef);
```

---

# 57. Image Provenance

```rust
pub struct RunnerImageProvenance {
    pub definition: RunnerImageDefinitionId,
    pub plan: RunnerImageBuildPlanId,
    pub builder: BuilderIdentity,
    pub base: BaseImageRef,
    pub packages: RunnerImagePackageManifest,
}
```

---

# 58. Image Attestation

Signed attestation binds:

```text
RunnerImageId
definition
builder
SBOM
hardening profile
scan evidence
```

---

# 59. AttestationId

```rust
pub struct RunnerImageAttestationId(Digest);
```

---

# 60. Signing

Use Supply Chain signing system.

---

# 61. Image Factory

Not signing authority.

---

# 62. Vulnerability Scan

Image scan uses Part 37 findings.

---

# 63. Vulnerability Finding

Exact image digest subject.

---

# 64. Scan Freshness

Separate from image bytes.

---

# 65. Newly Disclosed CVE

Can make old image operationally unacceptable without image bytes changing.

---

# 66. Baseline Trust State

```rust
pub enum RunnerBaselineTrustState {
    Candidate,
    Approved,
    Canary,
    Production,
    Deprecated,
    Blocked,
    Revoked,
}
```

---

# 67. Approved

Passed required build/hardening/scan/attestation.

---

# 68. Canary

Limited fleet deployment.

---

# 69. Production

Allowed for configured trust classes.

---

# 70. Blocked

Not eligible for new runners.

---

# 71. Revoked

Existing runners should drain/quarantine according policy.

---

# 72. Approval Policy

May require:

```text
SBOM
no forbidden packages
hardening tests
vulnerability thresholds
agent compatibility
executor tests
benchmark smoke
```

---

# 73. No Scan Means Clean

Critical.

---

# 74. Image Test Suite

Examples:

```text
boot
agent enrollment
sandbox isolation
network policy
workspace cleanup
container execution
device access if intended
GPU smoke
reboot
```

---

# 75. Runner Baseline Test

Uses normal Test Results evidence.

---

# 76. Security Smoke Tests

Must include privilege boundaries.

---

# 77. Baseline Candidate

Never production merely because image build succeeded.

---

# 78. Fleet Binding

```rust
pub struct FleetBaselineBinding {
    pub fleet: RunnerFleetId,
    pub baseline: RunnerBaselineId,
    pub rollout: BaselineRolloutPolicy,
}
```

---

# 79. Fleet CapacityClass

Part 43 can reference baseline.

---

# 80. CapacityClassId Inputs

Should include image/baseline identity if correctness/trust relevant.

---

# 81. Autoscaler

Provision only allowed baseline generations.

---

# 82. Provider Image Mapping

```rust
pub struct ProviderImageBinding {
    pub runner_image: RunnerImageId,
    pub provider: InfrastructureProviderId,
    pub provider_image_id: BoundedString,
}
```

---

# 83. Provider Image Identity

Mapping is metadata.

Canonical baseline remains Forgeyard image identity.

---

# 84. Image Distribution

Part 51/52 can replicate image artifacts/metadata.

---

# 85. Cloud AMI/Image

Provider copy must be verified/mapped.

---

# 86. No Implicit Trust in Copied Provider Image

---

# 87. Rollout Policy

```rust
pub struct BaselineRolloutPolicy {
    pub canary_percent: Decimal,
    pub batch_percent: Decimal,
    pub observation_window: Duration,
}
```

---

# 88. Canary First

Default.

---

# 89. Canary Fleet

Runs bounded workload.

---

# 90. Canary Eligibility

Low-risk/self-test/selected project.

---

# 91. High-Security Work

Canary baseline may be excluded until production approval.

---

# 92. Rollout State

```rust
pub enum BaselineRolloutState {
    Planned,
    Canary,
    Expanding,
    Production,
    Paused,
    RolledBack,
    Failed,
}
```

---

# 93. Rollout Evidence

Monitor:

```text
runner registration success
job infrastructure failures
sandbox failures
performance regressions
crashes
drift/attestation
```

---

# 94. Reliability Integration

Part 50 can pause rollout on SLO regression.

---

# 95. Rollout Pause

Does not destroy running jobs blindly.

---

# 96. Drain

Existing runner-agent semantics.

---

# 97. New Provisioning

Stops using bad baseline.

---

# 98. Rollback

For ephemeral fleet:

```text
set previous baseline desired
drain new generation
terminate/recycle
provision previous image
```

---

# 99. Rollback Baseline

Must still be security-acceptable.

---

# 100. Emergency Security Issue

Cannot roll back to known vulnerable image merely for availability without explicit security decision.

---

# 101. Image Version

Human-semantic version optional.

---

# 102. Digest

Identity.

---

# 103. Baseline Alias

Examples:

```text
linux-x86_64-stable
windows-build-stable
macos-arm64-stable
```

Mutable pointer.

---

# 104. Alias Authority

Config/fleet metadata.

---

# 105. Never scheduler correctness identity.

---

# 106. Runtime Attestation

After boot/enrollment, runner reports/measures baseline.

---

# 107. RunnerAttestationId

```rust
pub struct RunnerAttestationId(Ulid);
```

---

# 108. Runtime Attestation Inputs

Depending platform:

```text
image/provider identity
boot measurements
OS build
kernel
agent digest
package baseline hash
driver versions
secure boot/TPM evidence
```

---

# 109. Hardware Attestation

Optional based on platform.

---

# 110. TPM/Secure Boot

Can strengthen trust.

---

# 111. Confidential Compute

Attestation can include TEE measurement.

---

# 112. No TPM Required Baseline

Not all runners support.

---

# 113. Attestation Policy

```rust
pub struct RunnerAttestationPolicy {
    pub required_claims: Vec<AttestationClaimKind>,
    pub max_age: Duration,
}
```

---

# 114. Attestation Freshness

Runner trust expires without refresh when required.

---

# 115. Runtime Trust State

```rust
pub enum RunnerRuntimeTrust {
    Verified,
    Degraded,
    Unverified,
    Quarantined,
}
```

---

# 116. Scheduler Integration

Hard filter:

```text
job required trust <= runner runtime trust/profile
```

---

# 117. Trust Downgrade

Runner can finish existing safe work or drain according policy.

---

# 118. High-Risk Job

Immediate cancellation may be required if security compromise.

---

# 119. Drift Detection

Compare observed host to baseline.

---

# 120. RunnerDriftId

```rust
pub struct RunnerDriftId(Ulid);
```

---

# 121. Drift Classes

```rust
pub enum RunnerDriftClass {
    Package,
    Kernel,
    Driver,
    Agent,
    Service,
    User,
    SecuritySetting,
    Filesystem,
    Unknown,
}
```

---

# 122. Drift Severity

```rust
pub enum RunnerDriftSeverity {
    Informational,
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 123. Drift Detection Sources

```text
agent inventory
package manifest
host configuration probes
TPM/boot measurements
provider metadata
EDR integration
```

---

# 124. Agent Self-Report

Useful but not absolute proof.

---

# 125. Independent Attestation

Higher assurance.

---

# 126. Drift Response

```rust
pub enum RunnerDriftAction {
    Observe,
    Drain,
    Reimage,
    Quarantine,
    ManualReview,
}
```

---

# 127. Manual Package Install

High-assurance ephemeral runner:

```text
drain + replace
```

---

# 128. Debug Session Contamination

Debug access can intentionally mutate environment.

Therefore debug runner should be disposable/reimaged afterward.

---

# 129. Debug-Tainted State

```rust
pub enum RunnerTaint {
    DebugSession,
    ManualAdminAccess,
    SecurityIncident,
    UnknownMutation,
}
```

---

# 130. Tainted Runner

Not returned to high-trust pool without rebuild/re-attestation.

---

# 131. Image Patch Campaign

```rust
pub struct PatchCampaignId(Ulid);
```

---

# 132. Patch Trigger

Examples:

```text
scheduled maintenance
critical CVE
OS vendor update
driver security update
agent compatibility
certificate/root change
```

---

# 133. Patch Campaign

```rust
pub struct PatchCampaign {
    pub id: PatchCampaignId,
    pub from: RunnerBaselineId,
    pub target_definition: RunnerImageDefinitionId,
    pub urgency: PatchUrgency,
}
```

---

# 134. PatchUrgency

```rust
pub enum PatchUrgency {
    Routine,
    Elevated,
    Critical,
    Emergency,
}
```

---

# 135. Patch-As-New-Image

Default process:

```text
update package/base inputs
  ↓
new ImageDefinitionId
  ↓
build
  ↓
scan/test/attest
  ↓
canary
  ↓
rollout
```

---

# 136. No SSH-and-apt-update Fleet Baseline

Critical.

---

# 137. Controlled In-Place Patch

For non-replaceable hosts:

```text
plan exact patch set
  ↓
drain
  ↓
apply
  ↓
reboot if required
  ↓
attest
  ↓
rejoin
```

---

# 138. PatchSetId

```rust
pub struct PatchSetId(Digest);
```

---

# 139. Patch Evidence

Records:

```text
before baseline
package/OS changes
reboot state
after inventory
attestation
```

---

# 140. Critical Patch Deadline

```rust
pub struct PatchDeadline(Timestamp);
```

---

# 141. After Deadline

Baseline may become blocked/revoked.

---

# 142. Grace

Explicit.

---

# 143. No Forever Exception

Critical.

---

# 144. Patch Exception

```rust
pub struct PatchException {
    pub baseline: RunnerBaselineId,
    pub reason: BoundedString,
    pub expires_at: Timestamp,
}
```

---

# 145. Exception

Policy/audit.

---

# 146. Vulnerability Feed

Part 37/36 supplies vulnerability evidence.

---

# 147. Image Vulnerability

Can originate from:

```text
OS package
kernel
driver
agent
prewarmed tool
```

---

# 148. Prewarmed Tool Vulnerability

If not used by protected job, risk may differ.

Policy decides.

---

# 149. Base OS EOL

Image blocked after policy deadline.

---

# 150. EOL State

```rust
pub enum OsSupportState {
    Supported,
    Maintenance,
    EndOfLife,
    Unknown,
}
```

---

# 151. EOL Baseline

No new high-trust runner by default.

---

# 152. Image Retirement

```rust
pub enum RunnerImageLifecycle {
    Candidate,
    Active,
    Deprecated,
    Retiring,
    Retired,
    Revoked,
}
```

---

# 153. Retirement Flow

```text
stop new provisioning
  ↓
drain remaining runners
  ↓
retain image/evidence per policy
  ↓
remove provider copies when safe
```

---

# 154. Provider Image Deletion

Part 46 lifecycle.

---

# 155. Release Evidence

Runner image used for a release may need long-lived provenance record even if provider image is deleted.

---

# 156. Image Retention

Keep:

```text
definition
manifest
attestation
SBOM
build provenance
```

longer than provider image if needed.

---

# 157. Image Factory Self-Hosting

Forgeyard should build its own runner images.

---

# 158. Bootstrap Escape

Need external/manual documented method to create emergency baseline if Forgeyard image factory is broken.

---

# 159. No Circular Deadlock

Critical.

---

# 160. Bootstrap Runner

Lower-level known-good environment.

---

# 161. Bootstrap Image

Minimal.

---

# 162. Self-Build Verification

Production image can be rebuilt/verified by independent builder.

---

# 163. Multi-Party Reproducibility

Useful for high-assurance image.

---

# 164. Image Build Cache

Optimization only.

---

# 165. Image Layers

Can cache:

```text
base
package downloads
tool prewarm
```

---

# 166. Final Image

Always verified.

---

# 167. Artifact Registry

Part 52 can store:

```text
image definition bundle
VM image artifacts
SBOM
attestations
cloud-import artifacts
```

---

# 168. OCI

Container-host/tool images can use OCI.

---

# 169. VM Image Format

Examples:

```text
qcow2
raw
vhdx
provider-native
```

---

# 170. Format

Packaging concern separate from baseline identity.

---

# 171. Cloud Provider Import

Potential flow:

```text
verified raw/qcow artifact
  ↓
provider import
  ↓
provider image ID
  ↓
launch canary
  ↓
runtime attestation
```

---

# 172. Provider Import Unknown Outcome

Inspect provider before retry.

---

# 173. No Duplicate Untracked Images

---

# 174. Federation

Part 51.

Baseline can have regional provider-image copies.

---

# 175. Regional Image Availability

```rust
pub struct RunnerImageSiteAvailability {
    pub image: RunnerImageId,
    pub site: SiteId,
    pub provider_image: Option<ProviderImageBinding>,
    pub state: ImageAvailabilityState,
}
```

---

# 176. Residency

Image itself usually less sensitive, but embedded tool/license/data may impose restrictions.

---

# 177. No Embedded Secrets

Critical.

---

# 178. Image Must Not Contain

```text
cloud credentials
tenant secrets
registry publish tokens
production SSH private keys
```

---

# 179. Enrollment Credential

Single-use bootstrap token may be injected at provision time, not baked.

---

# 180. Runtime Identity

Issued after verified enrollment.

---

# 181. Certificates

Rotated by agent/trust subsystem.

---

# 182. Fleet Autoscaling

Part 43 provisions `CapacityClassId`.

Capacity class references:

```text
platform
resources
trust
RunnerBaselineId
```

---

# 183. Autoscaler Image Rollout

Desired baseline generation changes.

---

# 184. Old Generation

Drain according rollout.

---

# 185. Mixed Baselines

Allowed during rollout.

---

# 186. Scheduler

Can prefer new baseline for canary workloads.

---

# 187. Baseline Constraints

Job may require minimum baseline/security generation.

---

# 188. MinimumRunnerBaselineGeneration

```rust
pub struct RunnerBaselineGeneration(u64);
```

---

# 189. Security Floor

Can reject old baseline even if otherwise healthy.

---

# 190. Cost Integration

Image prewarming can reduce job startup but increase:

```text
image storage
build time
replication
boot time
```

---

# 191. Cost Analysis

Part 45.

---

# 192. No Massive Prewarm By Default

Measure.

---

# 193. Startup Performance

Part 33 benchmarks.

---

# 194. Image Benchmarks

Examples:

```text
boot time
agent registration
workspace creation
container cold start
```

---

# 195. Binary/Image Size

Tracked.

---

# 196. Reliability

Part 50 can define:

```text
runner boot success
registration success
baseline rollout failure rate
drift rate
```

---

# 197. SLO Regression

Can pause rollout.

---

# 198. Security Incident

Part 40.

If baseline compromised:

```text
mark Revoked
  ↓
stop provisioning
  ↓
drain/quarantine affected runners
  ↓
determine compromise window
  ↓
rebuild clean baseline
```

---

# 199. Runner Compromise

Outputs produced during compromise window may be quarantined.

---

# 200. Compromise Window

Bind runs to:

```text
RunnerId
RunnerBaselineId
attestation state
```

---

# 201. Run Provenance

Should include baseline identity.

---

# 202. JobAttempt Execution Context

```rust
pub struct RunnerExecutionBaseline {
    pub runner: RunnerId,
    pub baseline: RunnerBaselineId,
    pub attestation: Option<RunnerAttestationId>,
}
```

---

# 203. Provenance Uses It

High-assurance release can require verified baseline.

---

# 204. Image Baseline vs Sandbox

Both matter.

---

# 205. Strong Sandbox on Bad Host

Not fully trusted.

---

# 206. Trusted Host Without Sandbox

Still insufficient for untrusted code isolation.

---

# 207. Defense in Depth

```text
trusted baseline
+
agent trust
+
sandbox
+
job/toolchain identity
```

---

# 208. Host Observability

Collect:

```text
baseline
boot age
drift state
patch status
attestation freshness
```

---

# 209. No Full Filesystem Telemetry By Default

Privacy/performance.

---

# 210. Drift Probes

Targeted.

---

# 211. Dioxus UI

Pages:

```text
Runner Images
Baselines
Patch Campaigns
Fleet Rollouts
Drift
Attestations
```

---

# 212. Image Detail

Shows:

```text
definition
base OS
package manifest
SBOM
vulnerability status
attestation
provider copies
fleet usage
```

---

# 213. Baseline Detail

Shows:

```text
trust state
hardening profile
agent range
rollout
security generation
```

---

# 214. Patch Campaign View

Shows:

```text
affected fleets
urgency
deadline
canary
rollout progress
exceptions
```

---

# 215. Drift View

Shows affected runners.

---

# 216. CLI

```text
forgeyard runner-image build
forgeyard runner-image show
forgeyard runner-image scan
forgeyard runner-image attest
forgeyard runner-image approve
forgeyard runner-image rollout
forgeyard runner-image rollback
forgeyard runner-image retire
forgeyard runner-image drift
forgeyard runner-image doctor
```

---

# 217. API

Potential:

```text
GET  /v1/runner-images
POST /v1/runner-images/build
POST /v1/runner-images/{id}/approve
POST /v1/runner-baselines/{id}/rollout
POST /v1/runner-baselines/{id}/revoke
GET  /v1/runner-drift
GET  /v1/runner-attestations
```

---

# 218. Permissions

```text
runner_image.read
runner_image.build
runner_image.approve
runner_image.rollout
runner_image.revoke
runner_image.patch.manage
runner_image.exception.manage
```

---

# 219. Approval

Separate from build permission.

---

# 220. Revoke

High privilege/security.

---

# 221. Patch Exception

Audited.

---

# 222. Audit

Audit:

```text
baseline approve
baseline revoke
rollout pause/rollback
patch exception
manual host acceptance
drift override
```

---

# 223. Routine image builds

Operational evidence.

---

# 224. Notifications

Examples:

```text
critical baseline vulnerability
patch deadline approaching
canary regression
runner drift high
attestation expired
baseline revoked
```

---

# 225. Search/Catalog

Part 31/49 can show:

```text
fleet baseline
OS version
security generation
```

---

# 226. Data Lifecycle

Part 46 governs:

```text
image artifact
package manifest
SBOM
attestation
drift evidence
patch records
```

---

# 227. Provider Image Copies

Can be deleted after retirement.

---

# 228. Provenance

Retained per release/security policy.

---

# 229. Image Secrets Scan

Mandatory check.

---

# 230. Image Must Be Scanned For

```text
private keys
tokens
cloud credentials
SSH keys
unexpected certificates
```

---

# 231. Embedded Credential Finding

Blocks baseline.

---

# 232. License Governance

Preinstalled licensed tools may have distribution restrictions.

---

# 233. Image Distribution Policy

Track licenses.

---

# 234. Commercial SDKs

May restrict regional replication.

---

# 235. Apple Tooling

Must obey platform licensing/host constraints.

---

# 236. macOS Image Strategy

Real Apple hardware or permitted virtualization.

---

# 237. Linux Cannot Produce Trusted macOS Host Image Substitute

Existing platform invariant.

---

# 238. Windows Image Strategy

Use real Windows image/build tooling.

---

# 239. Driver Signing

Respect OS platform requirements.

---

# 240. Secure Boot

If required, verify.

---

# 241. Kernel Version

Baseline fact.

---

# 242. Live Kernel Patching

If used on long-lived host, counts as controlled mutation and re-attestation.

---

# 243. Ephemeral Runner Preferred

For cloud Linux/Windows when feasible.

---

# 244. Long-Lived Host Policy

Stricter drift/patch cadence.

---

# 245. Host Age

```rust
pub struct RunnerHostAge(Duration);
```

---

# 246. Maximum Host Age

Policy.

---

# 247. Recycle Threshold

After:

```text
N jobs
N hours
debug session
manual admin access
critical patch
```

---

# 248. RunnerRecyclePolicy

```rust
pub struct RunnerRecyclePolicy {
    pub max_jobs: Option<u64>,
    pub max_age: Option<Duration>,
    pub recycle_after_debug: bool,
}
```

---

# 249. High-Assurance Ephemeral

One-job runner possible.

---

# 250. Cost Tradeoff

Policy/profile.

---

# 251. Recycle Is Not Security Proof

Attestation still matters.

---

# 252. Host Persistent Cache

High-risk cross-job state.

---

# 253. Prefer external CAS/cache.

---

# 254. If host cache exists

Tenant/trust isolation + scrub.

---

# 255. Image Bake vs Runtime Cache

Do not bake frequently changing build cache into baseline.

---

# 256. Image Definition Layering

Logical:

```text
OS base
  ↓
security hardening
  ↓
Forgeyard agent/runtime
  ↓
platform drivers
  ↓
optional prewarm
```

---

# 257. Layer Change

Produces new image identity.

---

# 258. Image Diff

```rust
pub struct RunnerImageDiff {
    pub base_changed: bool,
    pub packages: Vec<PackageChange>,
    pub hardening: Vec<HardeningChange>,
    pub agent_changed: bool,
    pub drivers: Vec<DriverChange>,
}
```

---

# 259. Patch Review

Shows exact diff.

---

# 260. Image Change Risk

```rust
pub enum RunnerImageChangeRisk {
    Low,
    Moderate,
    High,
    Critical,
}
```

---

# 261. Example High

```text
kernel
sandbox dependency
hypervisor
GPU driver
agent privilege
```

---

# 262. Canary Size

Can be larger for low risk, smaller for high risk but observation stricter.

---

# 263. Emergency Patch

May shorten canary but never skip core validation entirely unless explicit break-glass policy.

---

# 264. Break-Glass Image

Explicit security event.

---

# 265. Emergency Flow

```text
critical vulnerability
  ↓
new patched definition
  ↓
fast build + mandatory security smoke
  ↓
small canary
  ↓
rapid rollout
  ↓
post-rollout deeper validation
```

---

# 266. Never Patch by Undocumented Manual Shell Fleet-Wide

Critical.

---

# 267. Image Factory Health

```rust
pub enum RunnerImageFactoryHealth {
    Healthy,
    BuildDegraded,
    ScanDegraded,
    AttestationDegraded,
    RolloutDegraded,
    Unhealthy,
}
```

---

# 268. Scan Service Down

Cannot claim new image approved unless policy explicitly allows alternative evidence.

---

# 269. Signing Down

Image candidate can build but approval/attestation may wait.

---

# 270. Existing Approved Baselines

Continue if still valid.

---

# 271. Doctor

```text
forgeyard runner-image doctor
```

Checks:

```text
unapproved fleet baseline
revoked baseline still active
critical patch overdue
attestation stale
provider image missing
drifted runners
embedded secret findings
unsupported OS
```

---

# 272. Observability Metrics

```text
runner_image_build_total
runner_image_build_failures_total
runner_baseline_rollout_total
runner_baseline_canary_failures_total
runner_drift_total
runner_attestation_expired_total
runner_patch_overdue_total
```

---

# 273. Labels

Low cardinality:

```text
platform
baseline_state
drift_class
result
```

---

# 274. Tracing

```text
runner_image.resolve
runner_image.build
runner_image.scan
runner_image.attest
runner_image.import
runner_image.rollout
runner_image.drift
runner_image.patch
```

---

# 275. HA

Image factory workers idempotent.

---

# 276. Build Intent

Persisted before provider import/build.

---

# 277. Ambiguous Provider Image Import

Inspect before retry.

---

# 278. Rollout Reconciler

Desired baseline vs observed fleet generations.

---

# 279. Drift Reconciler

Runner observations vs baseline.

---

# 280. No Raft Requirement

Normal metadata DB except narrow leadership where needed.

---

# 281. Federation Authority

Baseline alias/rollout config has one mutable authority domain.

---

# 282. Regional Image Copies

Immutable.

---

# 283. Site Offline

Can provision cached approved baseline if trust/config remains valid.

---

# 284. Air-Gap

Runner image bundle can include:

```text
image artifact
manifest
SBOM
attestation
provider import metadata
```

---

# 285. Offline Import

Verify signature/digest before activation.

---

# 286. DR

Image definitions/attestations backed up.

---

# 287. Provider-Native Images

Can be recreated from canonical image artifact/definition where possible.

---

# 288. If ProviderGenerated Only

Retain export/replication or rebuild recipe.

---

# 289. Disaster Recovery Runner Capacity

Need at least one known-good baseline available in recovery site.

---

# 290. Bootstrap Image Availability

DR checklist.

---

# 291. Compatibility

Part 57.

Image baseline includes supported:

```text
agent version
kernel/executor
driver
platform
```

---

# 292. Agent/Daemon Compatibility

Before rollout.

---

# 293. Executor Compatibility

Sandbox tests.

---

# 294. Toolchain Compatibility

Prewarm does not define requirement.

---

# 295. Image OS Upgrade

Major OS change may be high-risk compatibility event.

---

# 296. Test Environment

Part 56 can validate runner image using controlled jobs.

---

# 297. Self-Test Pipeline

Examples:

```text
compile Rust
run container
network-deny test
secret redaction
workspace cleanup
cache access
CAS upload/download
```

---

# 298. Image Rollout Pipeline

Is itself normal Forgeyard governed workflow.

---

# 299. Merge Queue

Image definition changes can require protected merge policy.

---

# 300. Supply-Chain Provenance

Image build has SLSA/in-toto style evidence where appropriate.

---

# 301. Image Rebuild Verification

Can rebuild same definition independently.

---

# 302. Reproducibility Difference

Investigate.

---

# 303. No Auto-Approve Different Baseline

Critical.

---

# 304. Testkit

```text
forgeyard-runner-image-testkit/src/
├── lib.rs
├── definition.rs
├── build.rs
├── scan.rs
├── attestation.rs
├── rollout.rs
├── drift.rs
├── patch.rs
└── assertions.rs
```

---

# 305. Unit Tests

Image definition determinism.

---

# 306. Mutable Alias Test

`latest` resolved to exact base before build.

---

# 307. Package Manifest Test

Exact versions/digests recorded.

---

# 308. Embedded Secret Test

Baseline approval blocked.

---

# 309. Prewarm Test

Job still resolves exact toolchain.

---

# 310. Attestation Test

Wrong image identity cannot enroll as trusted.

---

# 311. Drift Test

Manual package install detected.

---

# 312. Debug Taint Test

Runner not returned to high-trust pool.

---

# 313. Canary Test

Regression pauses rollout.

---

# 314. Rollback Test

New baseline drains; prior acceptable baseline restored.

---

# 315. Revoked Baseline Test

New provisioning blocked.

---

# 316. Critical Patch Deadline Test

Overdue baseline downgraded/blocked.

---

# 317. In-Place Patch Test

Host drains/reboots/re-attests.

---

# 318. Provider Import Timeout Test

Inspect before duplicate import.

---

# 319. Federation Test

Regional copies map to same canonical image.

---

# 320. Tenant Isolation Test

Host cache cannot leak cross-tenant state.

---

# 321. Provenance Test

JobAttempt records baseline identity.

---

# 322. Compatibility Test

Unsupported agent/baseline pair blocked.

---

# 323. DR Test

Recovery site has usable approved baseline.

---

# 324. Fuzzing

Fuzz:

```text
image manifests
attestation claims
package inventory
provider image metadata
```

---

# 325. Adversarial Tests

```text
spoofed image ID
stale attestation
tampered package manifest
manual root access
embedded cloud token
```

---

# 326. Chaos Tests

```text
image build worker crash
scan service outage
provider import timeout
canary runner failures
regional image missing
```

---

# 327. Scale Test

Large fleets/many regional copies.

---

# 328. Implementation Phase 1 — Image/Baseline Model

Core identities and manifest.

---

# 329. Phase 2 — Linux Image Factory

Primary dogfood target.

---

# 330. Phase 3 — Scan/SBOM/Attestation

Trust.

---

# 331. Phase 4 — Fleet Binding/Canary Rollout

Operations.

---

# 332. Phase 5 — Runtime Drift/Trust State

Continuous assurance.

---

# 333. Phase 6 — Patch Campaigns

Lifecycle.

---

# 334. Phase 7 — Windows Images

Production desktop/server runners.

---

# 335. Phase 8 — macOS/Bare-Metal Controlled Patch

Apple hosts.

---

# 336. Phase 9 — GPU/Device Host Baselines

Specialized fleets.

---

# 337. Phase 10 — Federation/Air-Gap

Enterprise.

---

# 338. Phase 11 — Reproducibility/Multi-Builder Verification

High assurance.

---

# 339. Phase 12 — Scale/Chaos/Security Hardening

Production readiness.

---

# 340. Acceptance Tests

1. Every trusted runner maps to an approved RunnerBaselineId.
2. Runner images have immutable canonical identity.
3. Mutable OS/provider aliases resolve to exact identities before build.
4. Base image provenance is recorded.
5. OS package manifests are exact.
6. Image build does not run in main daemon.
7. Image candidate is not trusted merely because build succeeded.
8. SBOM/scan/hardening evidence can be required before approval.
9. Image factory is not signing authority.
10. Embedded secrets block image approval.
11. Prewarmed tools do not replace ToolchainDescriptor correctness.
12. Fleet capacity classes reference baseline identity.
13. Autoscaler provisions only allowed baseline generations.
14. Canary rollout precedes broad production rollout by default.
15. Regression can pause baseline rollout.
16. Rollback uses previously approved and still-security-valid baseline.
17. Critical vulnerability can revoke old baseline.
18. Runtime runner attestation is freshness-bound where required.
19. Scheduler can enforce required runner trust/baseline generation.
20. Drift can downgrade/drain/quarantine runner.
21. Debug/manual-admin taint prevents silent return to high-trust pool.
22. Patch-as-new-image is default for replaceable runners.
23. Controlled in-place patch records exact changes and re-attests.
24. No undocumented fleet-wide shell patching is normal architecture.
25. Long-lived hosts have stricter drift/patch policy.
26. Job provenance records RunnerBaselineId.
27. Provider-native image copies map back to canonical image identity.
28. Regional copies are verified.
29. Air-gap baseline imports verify digest/signature.
30. Provider replica/image is not backup of provenance.
31. Recovery sites retain at least one viable approved baseline.
32. Compatibility matrix is checked before agent/baseline rollout.
33. Standalone/distributed share baseline trust semantics.
34. Forgeyard can rebuild and verify its own runner images.
35. Forgeyard dogfoods the image factory for its own CI fleets.

---

# 341. Production Readiness Gates

Do not call runner-image architecture production-ready until:

```text
canonical image/baseline identity is stable
Linux image factory is dogfooded
embedded-secret scanning works
SBOM/vulnerability evidence is integrated
fleet canary/rollback works
runtime drift and attestation are enforced
critical patch deadlines work
provider image import reconciliation is safe
DR/federation image availability tests pass
adversarial/chaos tests pass
```

---

# 342. Architectural Invariants

1. trusted runner implies approved baseline;
2. image identity is immutable;
3. mutable base aliases resolve before build;
4. base provenance is explicit;
5. package/driver manifests are explicit;
6. image builder runs outside main daemon;
7. image build success is not trust approval;
8. attestation/signing are separate authorities;
9. embedded secrets are forbidden;
10. prewarm is optimization, not correctness;
11. fleet binding references baseline identity;
12. canary precedes broad rollout by default;
13. rollout regressions can pause expansion;
14. rollback cannot knowingly violate security floor;
15. revoked baseline cannot provision new trusted runners;
16. attestation freshness affects trust;
17. drift affects trust;
18. debug/manual mutation taints runner;
19. replaceable runners are patched by image replacement;
20. non-replaceable hosts require controlled patch + re-attestation;
21. patch exceptions expire;
22. job provenance records baseline;
23. provider image IDs do not replace canonical identity;
24. regional copies are verified;
25. image artifact/replica does not replace provenance backup;
26. recovery site needs viable baseline;
27. compatibility gates precede rollout;
28. standalone/distributed share trust semantics;
29. bootstrap escape path prevents circular recovery deadlock;
30. Forgeyard dogfoods its own runner image factory.

---

# 343. Final Target Architecture

```text
                  Runner Image Definition
                           │
                           ▼
                    Exact Build Plan
                           │
                           ▼
                    Image Construction
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
           SBOM          Scan         Hardening
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                      Attestation
                           │
                           ▼
                    Approved Baseline
                           │
                           ▼
                     Canary Rollout
                           │
                           ▼
                   Production Fleets
                           │
                           ▼
                  Runtime Attestation
                           │
                           ▼
                    Drift / Patching
```

Runner trust:

```text
RunnerBaselineId
+
runtime attestation
+
fresh security state
+
no disqualifying drift
  ↓
RunnerRuntimeTrust
  ↓
scheduler eligibility
```

Patch lifecycle:

```text
new CVE / OS update
      ↓
new exact image definition
      ↓
build + scan + test
      ↓
canary
      ↓
fleet rollout
      ↓
retire old baseline
```

The key guarantee is:

> **Forgeyard can scale runner fleets aggressively without turning hosts into untracked snowflakes. Every trusted runner has an approved, provenance-backed baseline; changes produce new baselines or controlled re-attestation; rollout is canary-driven; drift reduces trust; and execution provenance records exactly which host baseline participated in the build.**

---

# 344. Extended Architecture Sequence

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
```
