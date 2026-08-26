# 26 — Forgeyard Self-Hosting, Bootstrap & Release-of-Forgeyard System Architecture

**Document type:** Core Self-Hosting, Bootstrap, Dogfooding & Forgeyard Release System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** bootstrap trust, first-build strategy, self-hosted build/test/package/release/deploy pipelines, standalone and distributed installation, binary provenance, release signing, upgrade channels, bootstrap-to-self-host transition, reproducibility verification, disaster/bootstrap recovery, and dogfooding of all Forgeyard subsystems  
**Architecture style:** Stage-based bootstrapping with minimal trusted seed, immutable release artifacts, reproducible self-builds, exact-digest promotion, explicit trust establishment, gradual transition from external bootstrap tooling to full Forgeyard self-hosting, and no circular dependency on an already-running Forgeyard instance  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** This closes the numbered architecture series. It composes Core Domain, Storage, CAS, Pipeline IR, Run/Job, Scheduler, Runner, Sandbox, Transport, Events/Reconciliation, Policy/Authz, Secrets/Trust, Supply Chain, Packaging, Release, Deployment, Observability, API, Dioxus UI, Device Lab, SCM, HA/Raft, RBE, Plugins, and Operations/DR into Forgeyard's own build/release lifecycle.

---

# 1. Purpose

Forgeyard must be able to build and release itself.

But a self-hosted CI/CD platform has an obvious bootstrap problem:

```text
Forgeyard builds Forgeyard
        ↑
but Forgeyard must exist first
```

The architecture therefore needs an explicit bootstrap chain.

The central rule is:

> **Forgeyard never depends on an already-running Forgeyard instance to create the first trusted Forgeyard binary.**

A second rule is:

> **Once bootstrap is complete, Forgeyard must dogfood its own pipeline, runner, packaging, signing, release, deployment, observability, upgrade, and recovery systems.**

A third rule is:

> **The initial trusted seed must be as small, inspectable, reproducible, and independently verifiable as practical.**

---

# 2. Bootstrap Stages

Forgeyard self-hosting is divided into stages:

```text
Stage 0 — Trusted external toolchain seed
Stage 1 — Bootstrap Forgeyard build
Stage 2 — First runnable Forgeyard
Stage 3 — Self-hosted Forgeyard build
Stage 4 — Reproducibility verification
Stage 5 — Signed release
Stage 6 — Self-deployment
Stage 7 — Continuous self-hosting
```

---

# 3. Stage 0 — Trusted Seed

Stage 0 consists of external tools that must exist before Forgeyard itself.

Typical:

```text
Rust toolchain
Cargo
system linker
platform SDK/tooling
bootstrap scripts/xtask
source checkout
```

---

# 4. Trusted Seed Goal

Minimize:

```text
number of tools
mutable dependencies
network fetches
unverified binaries
```

---

# 5. Rust Toolchain

Pinned by:

```text
rust-toolchain.toml
```

---

# 6. Cargo Lock

Pinned:

```text
Cargo.lock
```

---

# 7. Dependency Integrity

Use locked crates + checksum verification.

---

# 8. Vendoring

High-assurance/offline bootstrap may use:

```text
vendor/
```

---

# 9. Stage-0 Source

Exact VCS revision / source snapshot.

---

# 10. Bootstrap Source Verification

Before build:

```text
verify commit/tag/signature if used
verify vendored dependency hashes
verify bootstrap toolchain identity
```

---

# 11. Bootstrap Script

Prefer Rust `xtask` over complex shell logic.

---

# 12. Shell

Small platform bootstrap wrappers acceptable.

---

# 13. `xtask`

Responsibilities:

```text
environment check
toolchain verification
build orchestration
artifact layout
test invocation
package invocation
bootstrap manifest generation
```

---

# 14. No Hidden Bootstrap Logic

Bootstrap commands should be documented/reproducible.

---

# 15. Bootstrap Workspace

Recommended:

```text
xtask/
tools/bootstrap/
scripts/bootstrap-linux.sh
scripts/bootstrap-windows.ps1
scripts/bootstrap-macos.sh
```

---

# 16. Bootstrap Manifest

```rust
pub struct BootstrapManifest {
    pub source: SourceRevisionDescriptor,
    pub rust_toolchain: ToolchainIdentity,
    pub cargo_lock_digest: Digest,
    pub target: BuildTarget,
    pub commands: Vec<BootstrapCommandDigest>,
}
```

---

# 17. Bootstrap Manifest Storage

Generated artifact.

---

# 18. Stage 1 — Bootstrap Build

Build the minimum Forgeyard executable(s).

---

# 19. Minimum Bootstrap Target

Recommended first target:

```text
forgeyard single-binary standalone mode
```

---

# 20. Why Standalone First

It requires:

```text
no external PostgreSQL
no cluster
no external CAS
no HA
```

---

# 21. Standalone Bootstrap Composition

Single installation includes:

```text
daemon
local scheduler
local runner
embedded Stoolap
local CAS
UI/CLI as selected
```

---

# 22. Minimal Bootstrap Feature Set

Required:

```text
project load
pipeline parse
local run
runner/executor
local CAS
artifact handling
package/release basics
doctor
```

---

# 23. Optional Features Disabled

At first bootstrap:

```text
HA
external plugins
SCM write integrations
cloud deployment providers
external RBE
```

---

# 24. Bootstrap Feature Profile

Cargo feature/profile explicit.

---

# 25. Bootstrap Binary Identity

Record:

```text
source revision
toolchain
target
binary digest
bootstrap manifest digest
```

---

# 26. Stage 1 Output

```text
forgeyard-bootstrap binary
```

or normal `forgeyard` binary marked bootstrap provenance.

---

# 27. Stage 2 — First Running Forgeyard

Start standalone Forgeyard locally.

---

# 28. Initial Bootstrap Project

Repository itself imported as project.

---

# 29. Bootstrap Pipeline

Forgeyard repository includes:

```text
.forgeyard/pipeline.ron
```

---

# 30. Bootstrap Pipeline Goal

Use newly-built Forgeyard to build Forgeyard again.

---

# 31. Stage 3 — Self-Hosted Build

Now:

```text
Bootstrap Forgeyard
   ↓
runs Forgeyard pipeline
   ↓
builds Forgeyard
```

---

# 32. Self-Hosted Output

Call:

```text
Stage3Artifact
```

conceptually.

---

# 33. Comparison

Compare Stage1 bootstrap binary vs Stage3 self-built output if build conditions equivalent.

---

# 34. Full Bit-for-Bit Equality

Desirable where reproducibility allows.

---

# 35. If Not Bit-for-Bit

Use existing normalized/semantic reproducibility levels.

---

# 36. Reproducibility Claim

Never overstate.

---

# 37. Stage 4 — Independent Rebuild

Run second build:

```text
different runner
different host
different site if possible
```

---

# 38. Reproducibility Verification

Use existing FRBS model:

```text
BitForBit
NormalizedTree
Semantic
```

---

# 39. Multi-Party Reproduction

High-assurance release may require independent verifier.

---

# 40. Self-Hosting Trust Chain

```text
Stage0 toolchain
  ↓
Stage1 Forgeyard
  ↓
Stage3 self-built Forgeyard
  ↓
independent reproduce
  ↓
release candidate
```

---

# 41. Stage 5 — Signing

Unsigned release artifact first.

---

# 42. Signing Boundary

Use restricted signing worker/provider.

---

# 43. Forgeyard Signing Keys

Never available to general build runner.

---

# 44. Signed Outputs

Examples:

```text
Linux binaries/packages
Windows MSI/MSIX
macOS app/pkg/dmg
Android APK/AAB
OCI images
checksums
release manifest
```

---

# 45. Signing Lineage

Each byte-changing signing stage creates new CAS object.

---

# 46. Notarization

macOS as applicable.

---

# 47. Stage 6 — Release

Use Part 15 release subsystem.

---

# 48. Forgeyard Release Candidate

Contains exact package/evidence digests.

---

# 49. Release Approval

Production Forgeyard release may require:

```text
reproducibility gate
SBOM
provenance
vulnerability scan
license policy
signatures
manual approval
```

---

# 50. Exact-Byte Promotion

No rebuild.

---

# 51. Release Channels

Recommended:

```text
Nightly
Beta
Stable
LTS
```

---

# 52. Channel Semantics

Nightly:

```text
automation allowed
lower approval threshold
```

Stable:

```text
full evidence/approval
```

---

# 53. LTS

Optional later.

---

# 54. Release Package Set

Potential:

```text
Linux x86_64
Linux arm64
Windows x64
macOS arm64/x64/universal
container image
Android if shipped
```

---

# 55. Real Platform Requirement

Windows/macOS production artifacts built/tested on actual supported platforms.

---

# 56. Linux Cannot Replace macOS

Critical.

---

# 57. Cross-Compile

Allowed only where platform/package semantics support.

---

# 58. Forgeyard Distribution Site

Release subsystem publishes:

```text
binary/package
checksums
signatures
SBOM
provenance
release notes
```

---

# 59. OCI

Publish Forgeyard server/agent image by digest.

---

# 60. Package Managers

Potential:

```text
deb/rpm
Arch package
Homebrew-like external formula
winget-like metadata
```

publisher adapters later.

---

# 61. No Unverified Curl Pipe Shell Baseline

Installer may exist, but must verify signed/checksummed artifacts.

---

# 62. Bootstrap Installer

Minimal installer verifies:

```text
release manifest
signature/trust root
artifact digest
```

---

# 63. Install Trust Root

How does first installer trust Forgeyard?

---

# 64. Bootstrap Trust Root

Distribution includes pinned public release root/key.

---

# 65. Trust Root Rotation

Signed transition:

```text
old root signs new root
```

with overlap.

---

# 66. Offline Verification

Users can verify downloaded release without running Forgeyard.

---

# 67. Verification Tool

Could provide small standalone:

```text
forgeyard-verify
```

or documented generic signature/checksum method.

---

# 68. Avoid Circular Verify Dependency

First release must be verifiable with standard external tools.

---

# 69. Stage 7 — Self-Deployment

Forgeyard deploys its own control plane.

---

# 70. Self-Deployment Flow

```text
Forgeyard release
  ↓
DeploymentPlan
  ↓
staging Forgeyard cluster
  ↓
health
  ↓
production Forgeyard cluster
```

---

# 71. Self-Deployment Artifact

Exact released bytes.

---

# 72. No Rebuild Between Staging/Production

Critical.

---

# 73. Upgrade Self

Forgeyard deploys new Forgeyard to existing Forgeyard cluster.

---

# 74. Bootstrap Upgrade Paradox

Old Forgeyard orchestrates new Forgeyard rollout.

This is acceptable because old version remains authority until new nodes healthy.

---

# 75. Rolling Upgrade

Use Part 25:

```text
expand schema
upgrade follower
validate
transfer leadership
upgrade old leader
contract later
```

---

# 76. Self-Upgrade Safety

Do not let new binary control migration until compatibility preflight passes.

---

# 77. Upgrade Release Pin

Use exact `ReleaseId`.

---

# 78. No `latest`

Never.

---

# 79. Upgrade Plan

Records:

```text
current Forgeyard version
target Forgeyard ReleaseId
DB migration plan
coordination compatibility
agent compatibility
plugin compatibility
```

---

# 80. Agent Version Compatibility

Forgeyard release defines:

```text
minimum supported agent
maximum/tested range
```

---

# 81. Agent Auto-Upgrade

Optional later.

---

# 82. Agent Upgrade Safety

Drain jobs first.

---

# 83. Runner Upgrade

```text
drain
finish current work
upgrade exact package
restart
reconnect
health
```

---

# 84. Signing Worker Upgrade

Separate high-assurance rollout.

---

# 85. Device Agent Upgrade

Drain device sessions.

---

# 86. CLI Upgrade

Independent client compatibility matrix.

---

# 87. UI Upgrade

Same API major.

---

# 88. Standalone Upgrade

```text
backup
verify
install exact new package
migrate embedded DB
restart
doctor
```

---

# 89. Standalone Rollback

Only if local DB schema backward-compatible.

---

# 90. Self-Hosted Repository

Forgeyard repository itself uses:

```text
Git canonical VCS
```

---

# 91. SCM Provider

GitHub/GitLab etc optional provider hosting.

---

# 92. Self-Hosted Change Proposal

Initially provider PR/MR.

Later Forgeyard can manage policy/check/queue.

---

# 93. Self-Hosted Checks

Forgeyard publishes its own CI checks.

---

# 94. No Single Bootstrap CI Dependency

Critical release can be reproduced outside Forgeyard using documented bootstrap instructions.

---

# 95. Escape Hatch

If Forgeyard production is completely unavailable:

```text
Stage0 external bootstrap path
```

still exists.

---

# 96. Disaster Rebootstrap

Fresh environment can recreate Forgeyard from:

```text
source
toolchain
bootstrap instructions
or signed prior release
```

---

# 97. Preferred DR Bootstrap

Use last known-good signed Forgeyard release.

---

# 98. Source Bootstrap

Fallback if no release artifact available.

---

# 99. Bootstrap Tiers

```rust
pub enum BootstrapPath {
    SignedRelease,
    SourceBootstrap,
    AirGapBundle,
}
```

---

# 100. Signed Release Bootstrap

Fastest/preferred.

---

# 101. Source Bootstrap

Highest independence.

---

# 102. Air-Gap Bundle

High assurance/offline.

---

# 103. Air-Gap Bootstrap Bundle

Contains:

```text
Forgeyard binaries/packages
verification metadata
public trust roots
SBOM/provenance
configuration templates
optional vendored source/toolchain refs
```

---

# 104. No Private Secrets

Bundle contains no production private credentials.

---

# 105. First-Run Wizard

Standalone:

```text
choose data directory
initialize local store
initialize local CAS
create local admin identity
run doctor
```

---

# 106. First-Run Security

Generate random local secret material.

---

# 107. Default Bind

Localhost only unless operator configures network exposure.

---

# 108. Distributed Bootstrap

More complex.

---

# 109. Distributed Bootstrap Order

```text
1. provision PostgreSQL/Neon
2. provision durable CAS
3. establish trust root
4. start first daemon
5. initialize cluster identity
6. add second/third daemon as learners/voters
7. configure OIDC
8. enroll runners
9. run doctor
10. import Forgeyard project
```

---

# 110. First Admin

Bootstrap principal.

---

# 111. Bootstrap Admin Credential

Short-lived setup path.

---

# 112. Post-Bootstrap

Require normal OIDC/local identity and retire bootstrap credential.

---

# 113. Bootstrap Token Expiry

Mandatory.

---

# 114. Cluster Bootstrap Token

Separate from admin login.

---

# 115. Runner Enrollment

One-time enrollment tokens.

---

# 116. Signing Worker Enrollment

Higher trust path.

---

# 117. Trust Establishment

```text
root trust
  ↓
daemon certs
  ↓
runner/service certs
```

---

# 118. Initial CA

Generated/imported explicitly.

---

# 119. Production CA

Prefer offline root + online intermediate.

---

# 120. Standalone CA

Simpler local trust.

---

# 121. Bootstrap Secrets

Production external provider credentials entered only after trusted control plane available.

---

# 122. No Secret in Bootstrap Script

Critical.

---

# 123. Bootstrap Config

Templates only.

---

# 124. Bootstrap Config Example

```ron
(
    mode: Standalone,
    data_dir: "~/.local/share/forgeyard",
    listen: "127.0.0.1:8080",
)
```

---

# 125. Distributed Config Example

```ron
(
    mode: Distributed,
    database: Neon(...),
    cas: S3Compatible(...),
    cluster: (...),
)
```

Secret refs, not values.

---

# 126. Bootstrap Doctor

Mandatory before declaring installation ready.

---

# 127. Doctor Checks

Standalone:

```text
store
CAS
runner
sandbox
toolchain
UI/API
```

Distributed:

```text
DB
CAS
cluster quorum
trust
OIDC
scheduler
agent transport
```

---

# 128. Bootstrap Health Gate

Installation not production-ready until doctor passes.

---

# 129. Bootstrap Metadata

Persist:

```rust
pub struct InstallationMetadata {
    pub installation_id: InstallationId,
    pub bootstrap_path: BootstrapPath,
    pub initial_version: ForgeyardVersion,
    pub created_at: Timestamp,
}
```

---

# 130. InstallationId

```rust
pub struct InstallationId(Ulid);
```

---

# 131. Self-Hosting Pipeline

Repository:

```text
.forgeyard/
├── pipeline.ron
├── package.ron
├── release.ron
├── deploy.ron
└── policy.ron
```

---

# 132. Pipeline Stages

Recommended:

```text
validate
format
lint
unit
integration
security
build
reproducibility
package
SBOM/provenance
sign
release-candidate
deploy-staging
health
promote
deploy-production
```

---

# 133. Format

```text
cargo fmt --check
```

---

# 134. Lint

```text
cargo clippy
```

---

# 135. Dependency Policy

```text
cargo deny
license checks
vulnerability scan
```

---

# 136. Unit Tests

Workspace.

---

# 137. Integration Tests

DB/CAS/transport.

---

# 138. Distributed Tests

3-node/agent scenarios.

---

# 139. Fuzzing

Critical protocol/parsers.

---

# 140. Miri

Selected unsafe/core crates.

---

# 141. Property Tests

State machines/epochs/digests.

---

# 142. Architecture Check

Machine-enforced dependency rules.

---

# 143. Architecture Tool

```text
forgeyard-architecture-check
```

---

# 144. Workspace Dependency Enforcement

Fails CI on forbidden edges.

---

# 145. Feature Matrix

Build/test:

```text
standalone
distributed
minimal
full
```

---

# 146. Platform Matrix

At least:

```text
Linux x86_64
Linux arm64
Windows x64
macOS arm64
```

as product promises.

---

# 147. Android

If Forgeyard mobile UI/app ships.

---

# 148. Web

If web UI supported.

---

# 149. Cross-Compilation

Used only where trustworthy.

---

# 150. Native Test Hosts

Windows/macOS tests on native hosts.

---

# 151. Device Lab

Forgeyard mobile client/device components tested by own Device Lab.

---

# 152. RBE Dogfood

Forgeyard may build portions through own RBE adapter.

---

# 153. But Core Native Path

Also test normal Forgeyard pipeline execution.

---

# 154. Plugin Dogfood

At least scanner/notification sample plugin.

---

# 155. SCM Dogfood

Forgeyard repository binding uses own SCM adapter.

---

# 156. Release Dogfood

Forgeyard release generated only through Release subsystem after bootstrap maturation.

---

# 157. Deployment Dogfood

Production control plane upgrade uses Deployment subsystem.

---

# 158. Observability Dogfood

Forgeyard monitors Forgeyard.

---

# 159. Backup Dogfood

Forgeyard backups/restores its own metadata/CAS.

---

# 160. DR Dogfood

Periodic fresh-environment reconstruction.

---

# 161. Bootstrapping Trust Problem

How do users trust first binary?

---

# 162. Release Verification Assets

Publish:

```text
SHA-256
BLAKE3
signature
public key/root
SBOM
provenance
reproducibility evidence
```

---

# 163. Reproducible Build Evidence

Users can independently reproduce.

---

# 164. Rebuilder Documentation

Exact:

```text
toolchain
source revision
commands
target
```

---

# 165. Binary Transparency

Optional future transparency log.

---

# 166. Transparency Log

Could publish signed release digest log.

---

# 167. Not Required Initially

But architecture compatible.

---

# 168. Reproducible Release Rule

Unsigned Forgeyard binaries should aim deterministic.

---

# 169. Signed Artifacts

Signing may introduce expected non-determinism.

---

# 170. Compare Unsigned

Primary reproducibility target.

---

# 171. Build Host Trust

Production release can require trusted runner class.

---

# 172. Multi-Party Build

Different site/trust domain for Stable.

---

# 173. Release Candidate Completeness

Must include required platform artifacts.

---

# 174. Missing macOS

Stable release incomplete if macOS promised.

---

# 175. Release Policy

Example:

```text
all Tier-1 targets built
all tests pass
critical vulnerabilities none
license policy pass
SBOM/provenance present
reproducibility >= Reproduced
signatures valid
2 approvals
```

---

# 176. Release Notes

Generated from exact Change Proposals/integrated revisions.

---

# 177. Changelog

Versioned.

---

# 178. Migration Notes

Release includes:

```text
DB migration compatibility
agent min version
plugin API changes
operator actions
```

---

# 179. Upgrade Notes Machine-Readable

Could provide:

```text
upgrade-manifest.ron
```

---

# 180. Upgrade Manifest

```rust
pub struct ForgeyardUpgradeManifest {
    pub version: ForgeyardVersion,
    pub min_db_schema: SchemaVersion,
    pub max_db_schema: SchemaVersion,
    pub agent_compat: VersionRange,
    pub plugin_api: PluginApiVersion,
    pub coordination_schema: CoordinationSchemaVersion,
}
```

---

# 181. Upgrade Planner

Consumes manifest.

---

# 182. Bootstrap Release Manifest

Public machine-readable JSON/RON.

---

# 183. Download Site

Select platform/arch.

---

# 184. Installer Metadata

Exact digest.

---

# 185. Update Feed

Signed.

---

# 186. Self-Update

Optional.

---

# 187. CLI Self-Update

Can check signed update feed.

---

# 188. Server Self-Update

Do not auto-update production cluster blindly.

Requires upgrade plan/policy.

---

# 189. Standalone Self-Update

Can offer guided update.

---

# 190. Update Verification

Signature + digest.

---

# 191. Rollback

Keep previous package until new version healthy.

---

# 192. Binary Rollback Root

Previous release retained.

---

# 193. Config Migration

Versioned.

---

# 194. Config Forward Compatibility

Unknown safe fields tolerated where possible.

---

# 195. Config Backup

Before migration.

---

# 196. Plugin Compatibility

Upgrade preflight.

---

# 197. Incompatible Required Plugin

Blocks production upgrade.

---

# 198. Optional Plugin

Can disable.

---

# 199. Agent Compatibility Enforcement

Daemon rejects too-old protocol cleanly.

---

# 200. Upgrade Order for Agents

Can lag server within N/N-1.

---

# 201. Bootstrap from Source

Detailed command shape:

```text
cargo xtask bootstrap --target <target>
```

---

# 202. Bootstrap Output Directory

```text
target/bootstrap/
```

---

# 203. Bootstrap Metadata

```text
bootstrap-manifest.ron
checksums
binary
```

---

# 204. Bootstrap Test

At minimum:

```text
forgeyard doctor
forgeyard --version
local smoke pipeline
```

---

# 205. Stage-3 Self Build Command

```text
forgeyard run .forgeyard/pipeline.ron
```

conceptually.

---

# 206. Self-Build Recursion

Do not recursively launch infinite self-build.

---

# 207. Bootstrap Marker

Pipeline context:

```rust
pub enum SelfHostStage {
    Bootstrap,
    SelfHosted,
    Reproduce,
    Release,
}
```

---

# 208. Stage Policy

Certain pipeline steps only at release stage.

---

# 209. Bootstrap Build Must Not Sign

No production signing in Stage1.

---

# 210. Self-Hosted Release Signs Only After Verification

Critical.

---

# 211. Build Once

Release artifact built once before signing/promote.

---

# 212. No Release Rebuild

Existing invariant.

---

# 213. Self-Hosting Circular Secrets

Signing credentials not required to build Forgeyard.

---

# 214. Release Signing

Only final release stage.

---

# 215. Bootstrap Without SCM Provider

Source can be local checkout/tarball.

---

# 216. Bootstrap Without Internet

With vendored dependencies/toolchain availability.

---

# 217. Offline Toolchain Bundle

Optional.

---

# 218. Offline Source Bundle

Signed source archive + vendored crates.

---

# 219. Reproducibility Bundle

Contains exact inputs required.

---

# 220. Bootstrap Failure Modes

```text
toolchain mismatch
dependency missing
platform SDK missing
build failure
test failure
binary mismatch
```

---

# 221. Bootstrap Diagnostics

`xtask doctor-bootstrap`.

---

# 222. Bootstrap Error

Actionable.

---

# 223. Release Failure Modes

```text
repro mismatch
signer unavailable
platform artifact missing
policy failure
publication unknown
```

---

# 224. Self-Deployment Failure

Use deployment rollback/previous healthy release.

---

# 225. Production Control Plane Rollback

Check DB migration compatibility first.

---

# 226. Operator Recovery

If self-upgrade breaks API:

```text
use prior signed binary
maintenance mode
restore/forward-fix schema
```

---

# 227. Full Self-Host Disaster

If running Forgeyard cannot repair itself:

```text
external bootstrap path
```

---

# 228. This Must Be Tested

Critical.

---

# 229. Fire Drill

Periodic:

```text
delete fresh test Forgeyard installation
rebuild from published release/source
restore backup
run self-host pipeline
```

---

# 230. Self-Host Independence Test

Production Forgeyard outage must not make release source/toolchain permanently inaccessible.

---

# 231. Source Mirror

Keep independent source mirror/export.

---

# 232. Release Artifact Mirror

Independent backup.

---

# 233. Toolchain Mirror

Optional high-assurance.

---

# 234. Dependency Mirror

Optional offline.

---

# 235. Bootstrap Docs

Versioned in repository.

---

# 236. `BOOTSTRAP.md`

Recommended.

---

# 237. `SELF_HOSTING.md`

Recommended.

---

# 238. `RELEASES.md`

Already top-level.

---

# 239. `UPGRADE.md`

Recommended.

---

# 240. `RECOVERY.md`

Recommended.

---

# 241. Self-Hosting Workspace

Potential:

```text
self-host/
├── bootstrap.ron
├── pipeline.ron
├── release.ron
├── deploy.ron
├── policies/
└── reproducibility/
```

---

# 242. Keep `.forgeyard/` Canonical

Avoid duplicating if possible.

---

# 243. Bootstrap Config

Can reference `.forgeyard/` after Stage1.

---

# 244. Release Pipeline Identity

Store `PipelinePlanId`.

---

# 245. Self-Host Provenance

Every Forgeyard release records:

```text
SourceSnapshotId
PipelinePlanId
RunId
JobIds
builder identity
toolchain
reproduction results
PackageSet
EvidenceBundle
ReleaseId
```

---

# 246. Release-of-Forgeyard Evidence Bundle

Contains:

```text
SBOM
provenance
reproducibility
vulnerability scan
license report
signatures
notarization
package validation
```

---

# 247. Supply Chain Self-Reference

Forgeyard proves the chain used to produce Forgeyard.

---

# 248. Trust Bootstrapping Honesty

The chain ultimately depends on Stage0 toolchain unless fully bootstrapped compiler chain exists.

---

# 249. No False "Trustless Build"

Critical.

---

# 250. Diverse Double Compilation

Future optional high-assurance research.

---

# 251. Not Baseline

But architecture does not block it.

---

# 252. Rust Compiler Trust

Documented external trust dependency.

---

# 253. Third-Party Dependencies

SBOM records.

---

# 254. License Compliance

Release gate.

---

# 255. Vulnerability Findings

Release gate per severity/policy.

---

# 256. Emergency Security Release

Can reduce soak but not skip artifact identity/signature/provenance.

---

# 257. Emergency Self-Deployment

Break-glass release/deploy path.

---

# 258. Audit

Every production Forgeyard release/upgrade.

---

# 259. Release Manager Identity

Recorded.

---

# 260. Automated Nightly

Service principal.

---

# 261. Stable Approval

Human/independent according to policy.

---

# 262. Rollout Rings for Forgeyard

Example:

```text
internal dev
staging
small production canary
full production
```

---

# 263. Dogfood Ring

Forgeyard maintainers use new version before Stable public release.

---

# 264. Public Release Can Follow Internal Soak

Same bytes if policy allows.

---

# 265. No Rebuild After Soak

Critical.

---

# 266. Internal Build vs Public Artifact

Use same signed artifact where possible.

---

# 267. Release Metadata

Can change visibility/channel without changing artifact.

---

# 268. Self-Hosted HA Upgrade

Use 3-node architecture.

---

# 269. Quorum Safety

Upgrade one voter at a time.

---

# 270. Leadership Transfer

Before leader upgrade.

---

# 271. Cluster Feature Activation

Only after all voters upgraded.

---

# 272. Migration Contract

Later.

---

# 273. Runner Pools

Keep enough capacity during upgrade.

---

# 274. Scheduler

Old/new compatible.

---

# 275. API

Dioxus UI/CLI remain usable.

---

# 276. Agent Connections

Reconnect across daemon restarts.

---

# 277. Event/Reconcile

Repairs missed transitions.

---

# 278. Self-Monitoring

Observe:

```text
upgrade errors
leader churn
agent reconnect
DB latency
CAS errors
release/deploy health
```

---

# 279. Upgrade Health Gate

Must pass before next node.

---

# 280. Automated Pause

On degradation.

---

# 281. Rollback Gate

Check DB compatibility.

---

# 282. Self-Hosted Backup Before Upgrade

Mandatory for risky version.

---

# 283. DR Release

Last known-good Forgeyard release retained offsite.

---

# 284. Golden Recovery Version

Optional designated LTS/golden release.

---

# 285. Golden Version Requirements

Known:

```text
restore current supported backup format
connect to current DB schema range
rebuild coordination
```

---

# 286. Recovery Tooling

May be shipped separately/static.

---

# 287. `forgeyard-recovery`

Optional small recovery utility.

---

# 288. Recovery Utility Scope

```text
verify release
inspect backup
restore config/bootstrap
```

not full CI system.

---

# 289. Minimal Trusted Recovery Tools

Keep small.

---

# 290. Upgrade Compatibility Tests

Each release tests:

```text
N-1 → N
mixed N/N-1
agent N-1
plugin compatibility
DB expand-contract
rollback before contract
```

---

# 291. Bootstrap Compatibility Test

Fresh source build from clean supported host.

---

# 292. Offline Bootstrap Test

High-assurance release.

---

# 293. Reproduction Test

Independent build.

---

# 294. Installation Test

Fresh:

```text
Linux
Windows
macOS
```

as supported.

---

# 295. Uninstall Test

Packages.

---

# 296. Upgrade Test

Previous stable -> candidate.

---

# 297. DR Test

Restore previous backup using candidate recovery tooling.

---

# 298. Self-Hosting Test

Candidate Forgeyard builds its own source.

---

# 299. RBE Test

Candidate RBE API can build sample Bazel project.

---

# 300. Device Test

Mobile/device components.

---

# 301. Plugin Test

Sample sandbox plugin.

---

# 302. SCM Test

Provider integration sandbox/test repo.

---

# 303. Release Test

Dry-run to non-production destination.

---

# 304. Deployment Test

Staging.

---

# 305. Full Dogfood Gate

Stable candidate must complete all required dogfood flows.

---

# 306. Self-Host Testkit

```text
crates/self-host/
├── forgeyard-self-host-model/
├── forgeyard-self-host-bootstrap/
├── forgeyard-self-host-verify/
├── forgeyard-self-host-release/
├── forgeyard-self-host-upgrade/
└── forgeyard-self-host-testkit/
```

Module-first; only split if justified.

---

# 307. Bootstrap Testkit

Simulates clean host.

---

# 308. Repro Testkit

Compares stage outputs.

---

# 309. Upgrade Testkit

N-1/N cluster.

---

# 310. Disaster Rebootstrap Testkit

Fresh control plane from last release.

---

# 311. Failure Injection

```text
Stage1 build failure
Stage3 mismatch
signer unavailable
publication timeout
upgrade leader crash
migration interruption
CAS missing package
```

---

# 312. Chaos Upgrade Test

Kill node mid-upgrade.

---

# 313. Recovery Invariant

At least one known-good Forgeyard release + recovery documentation remains independently accessible.

---

# 314. Binary Retention

Keep:

```text
current
previous stable
golden recovery
```

---

# 315. Source Retention

Release source snapshot pinned.

---

# 316. Toolchain Metadata Retention

Pinned.

---

# 317. Repro Metadata Retention

Pinned.

---

# 318. Bootstrap Security

No downloading unpinned installer dependencies at runtime.

---

# 319. Installer Network

Fetch only exact signed release metadata/artifacts.

---

# 320. Mirror Support

Users can choose trusted mirror.

---

# 321. Mirror Verification

Digest/signature independent of mirror.

---

# 322. CDN Compromise

Cannot substitute valid artifact without signing key.

---

# 323. Signing Key Compromise

Part 12/25 rotation/revocation.

---

# 324. Release Root Compromise

Emergency root-rotation procedure.

---

# 325. Bootstrap Trust Update

Installer/update feed trusts rotated root chain.

---

# 326. Self-Hosted Build Secrets

Normal build should need none.

---

# 327. Integration Tests

May use ephemeral test secrets only.

---

# 328. Release Publishing Secrets

Only release worker.

---

# 329. Deployment Secrets

Only deployment worker.

---

# 330. Separation of Duties

Self-host pipeline should visibly enforce.

---

# 331. Self-Host Permissions

Examples:

```text
forgeyard.release.approve
forgeyard.release.promote
forgeyard.deploy.production
forgeyard.sign.production
```

---

# 332. No Single Broad "admin" for Automation

Use narrow service principals.

---

# 333. Production Signing

Separate identity.

---

# 334. Release Automation

Cannot access signing private key directly.

---

# 335. Deployment Automation

Cannot create release.

---

# 336. Bootstrap Developer

Cannot automatically promote stable.

---

# 337. Audit Trail

Complete.

---

# 338. Public Build Transparency

Publish machine-readable release evidence index.

---

# 339. User Verification Flow

```text
download release manifest
verify signature
download package
verify digest
optionally inspect SBOM/provenance
install
```

---

# 340. Self-Hosted Verification Flow

Forgeyard itself can verify candidate through Supply Chain subsystem.

---

# 341. Independent Verification Flow

Does not require Forgeyard.

---

# 342. Important Independence Principle

At least one verification path must remain external.

---

# 343. Governance

Release policy may require maintainer quorum.

---

# 344. Release Freeze

Optional during incidents.

---

# 345. Security Advisory Release

Embargoed candidate.

---

# 346. Embargo Access

Restricted.

---

# 347. Public Disclosure

Release publication event.

---

# 348. Self-Hosted Documentation Build

Docs site can be built/deployed by Forgeyard.

---

# 349. Web/Download Site

Dogfood Deployment subsystem.

---

# 350. Package Repository

Dogfood Release publisher.

---

# 351. Status Page

Could consume observability, externalized.

---

# 352. Bootstrap Status Independence

Status page should not be sole place for recovery docs.

---

# 353. Documentation Mirror

Independent static copy.

---

# 354. Acceptance Tests

1. Forgeyard can be built from source without an existing Forgeyard instance.
2. Bootstrap toolchain/version is pinned.
3. Cargo dependencies are lockfile/checksum controlled.
4. Offline/vendored bootstrap is possible for high-assurance path.
5. Stage1 produces a runnable standalone Forgeyard.
6. Stage1 binary can execute Forgeyard's own pipeline.
7. Stage3 self-hosted build succeeds.
8. Stage3 output is reproducibility-compared against independent build.
9. Reproducibility level is reported honestly.
10. Production signing occurs only after verification.
11. General runners never receive production signing keys.
12. Forgeyard release uses exact package/evidence digests.
13. Stable release never rebuilds after approval/soak.
14. Staging and production deploy exact same release bytes.
15. Forgeyard can upgrade itself through its Deployment subsystem.
16. HA upgrade changes one voter at a time.
17. Leadership transfers before leader upgrade.
18. DB expand-contract preserves N/N-1 compatibility.
19. Unsafe binary rollback is blocked after incompatible schema contract.
20. Standalone upgrade performs verified backup first.
21. Fresh distributed cluster can bootstrap from signed release.
22. Total Forgeyard outage can recover from external bootstrap/release path.
23. Bootstrap admin credentials are retired after normal identity setup.
24. Bootstrap tokens are short-lived.
25. Release root/public verification path does not require running Forgeyard.
26. CDN/mirror cannot substitute different bytes without failing digest/signature verification.
27. Forgeyard's own SCM integration uses Forgeyard SCM subsystem.
28. Forgeyard's own CI uses Forgeyard Run/Job/Scheduler/Runner.
29. Forgeyard's own packages use Packaging subsystem.
30. Forgeyard's own releases use Release subsystem.
31. Forgeyard's own production upgrades use Deployment subsystem.
32. Forgeyard monitors itself using Observability/Doctor.
33. Forgeyard periodically restores its own backups in DR drills.
34. At least one known-good signed recovery release remains independently retained.
35. Forgeyard can rebuild and recover Forgeyard even when production Forgeyard is unavailable.

---

# 355. Production Readiness Gates

Forgeyard is not truly self-hosting-ready until:

```text
clean source bootstrap works
Stage1 -> Stage3 self-build works
independent reproducibility verification works
production signing separation works
multi-platform package set works
stable release pipeline works
staging/production self-deploy works
rolling self-upgrade works
previous-version compatibility tests pass
fresh-environment rebootstrap succeeds
backup/restore of Forgeyard's own production state succeeds
```

---

# 356. Architectural Invariants

1. first Forgeyard build never requires existing Forgeyard;
2. Stage0 trusted seed is minimized and documented;
3. toolchain/dependencies are pinned;
4. bootstrap metadata records exact inputs;
5. first runnable target is standalone-capable;
6. Forgeyard builds Forgeyard after bootstrap;
7. self-built output is independently reproduced/verified;
8. reproducibility claims are honest;
9. production signing is separate from build;
10. release approval binds exact Forgeyard candidate digest;
11. stable release never rebuilds;
12. staging/production deploy the same bytes;
13. self-upgrade consumes exact ReleaseId;
14. HA self-upgrade preserves quorum;
15. schema upgrades use expand-contract;
16. agent/plugin/API compatibility is checked before upgrade;
17. previous known-good Forgeyard release is retained;
18. production outage does not remove external bootstrap path;
19. release verification can be performed without Forgeyard;
20. bootstrap credentials are temporary;
21. root trust rotation is explicit and signed;
22. mirrors/CDNs are not trust authorities;
23. self-host build normally needs no production secrets;
24. release/deploy/signing identities remain separated;
25. Forgeyard dogfoods every major subsystem;
26. recovery tooling remains independently accessible;
27. source/release/toolchain metadata are retained;
28. disaster rebootstrap is tested periodically;
29. standalone/distributed share release trust principles;
30. Forgeyard never becomes operationally dependent on itself in a circular way that prevents recovery.

---

# 357. Final Target Architecture

Bootstrap:

```text
             Trusted External Rust Toolchain
                          │
                          ▼
                    Stage1 Build
                          │
                          ▼
                Runnable Forgeyard
                          │
                          ▼
                 Self-Hosted Pipeline
                          │
                          ▼
                    Stage3 Build
                          │
                          ▼
              Independent Reproduction
                          │
                          ▼
                   Verified Candidate
                          │
                          ▼
                       Signing
                          │
                          ▼
                       Release
```

Self-deployment:

```text
Forgeyard ReleaseId
       │
       ▼
Staging Forgeyard
       │
     health
       │
       ▼
Production Canary
       │
     health
       │
       ▼
Full Production
```

Recovery escape path:

```text
Production Forgeyard unavailable
          │
          ├──► last known-good signed Forgeyard release
          │
          ├──► air-gap recovery bundle
          │
          └──► source bootstrap via Stage0 toolchain
                          │
                          ▼
                    Fresh Forgeyard
                          │
                          ▼
                     Restore state
                          │
                          ▼
                     Reconcile
```

---

# 358. Final Architectural Position

Self-host trust chain:

```text
external Rust/toolchain seed
+
exact Forgeyard source
+
Cargo.lock/dependency integrity
  ↓
bootstrap binary
  ↓
self-hosted Forgeyard build
  ↓
independent reproduction
  ↓
SBOM/provenance/security verification
  ↓
restricted signing
  ↓
immutable ReleaseId
  ↓
staging
  ↓
production
```

Upgrade chain:

```text
current Forgeyard
  ↓
verified target Forgeyard ReleaseId
  ↓
backup/preflight
  ↓
expand migration
  ↓
rolling nodes
  ↓
health
  ↓
feature activation
  ↓
contract later
```

Recovery chain:

```text
known-good release/source
  ↓
fresh bootstrap
  ↓
restore PostgreSQL/CAS/secrets
  ↓
rebuild/fence coordination
  ↓
doctor
  ↓
reconciliation
  ↓
production resume
```

The key guarantee is:

> **Forgeyard can fully dogfood its own CI/CD lifecycle without making its existence circular. A small external bootstrap path can always recreate a trusted Forgeyard; once running, Forgeyard builds, verifies, packages, signs, releases, deploys, observes, upgrades, backs up, and recovers itself using the same architecture it provides to users.**

---

# 359. Completion of the Numbered Architecture Series

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

This completes the planned numbered subsystem architecture set.
