# 41 — Forgeyard Release Distribution, Update Delivery, Installer, Channel & Client Update System Architecture

**Document type:** Core Release Distribution, Installer, Update Feed, Client/Agent Upgrade & Channel Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** signed update feeds, desktop/server/agent/mobile distribution, installers, update channels, staged rollout, compatibility gates, delta updates, mirror/CDN distribution, self-update, administrator-managed update, rollback, anti-rollback, update provenance, update trust, package delivery, air-gapped update bundles, and multi-platform release consumption  
**Architecture style:** Signed immutable release metadata, digest-bound packages, trust separated from transport, staged and reversible rollout, compatibility-aware updates, exact ReleaseId consumption, and no mirror/CDN/package host becoming update authority  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Packaging, Release, Deployment, Supply Chain, Self-Hosting, Security, Configuration, HA/Upgrade, Device Lab, Entitlements, and Developer Experience. This subsystem governs how already-released Forgeyard software reaches users, servers, runners, agents, and devices safely.

---

# 1. Purpose

Forgeyard can build and release itself, but production operation also needs a complete answer to:

```text
how does a user install Forgeyard?
how does forgeyard-agent update?
how does a distributed cluster roll forward?
how are desktop clients updated?
how are Android/mobile builds distributed?
how do air-gapped installations update?
how are release channels represented?
how do we prevent downgrade or freeze attacks?
what happens if a CDN or mirror is compromised?
how do we roll back a bad update?
```

The central rule is:

> **Distribution transports immutable release artifacts; it never decides which artifact is trusted. Trust comes from signed release metadata, exact digests, policy, compatibility, and configured update roots.**

A second rule is:

> **Every update installs an exact already-released artifact. Update delivery must never rebuild, repackage, or silently mutate release bytes.**

A third rule is:

> **Update rollout and update trust are separate concerns. A release can be cryptographically valid but still not be eligible for this installation, channel, component role, platform, or policy state.**

---

# 2. Architectural Position

```text
                   Released Forgeyard Artifact
                             │
                             ▼
                     Release Manifest
                             │
                  ┌──────────┼──────────┐
                  ▼          ▼          ▼
              Signature   Provenance  Compatibility
                  │          │          │
                  └──────────┼──────────┘
                             ▼
                       Update Feed
                             │
          ┌──────────────────┼──────────────────┐
          ▼                  ▼                  ▼
       CDN/Mirror         Air-Gap Bundle      Direct Store
          │                  │                  │
          └──────────────────┼──────────────────┘
                             ▼
                       Update Client
                             │
                  ┌──────────┼──────────┐
                  ▼          ▼          ▼
              Verify      Eligibility   Rollout
                  │          │          │
                  └──────────┼──────────┘
                             ▼
                         Install
                             │
                             ▼
                      Health / Commit
```

---

# 3. Goals

The subsystem MUST:

1. define update feed identity;
2. define release channels;
3. define component update targets;
4. define platform-specific package selection;
5. verify signatures;
6. verify digests;
7. verify release provenance metadata;
8. support staged rollout;
9. support canary update;
10. support rollback;
11. support anti-rollback;
12. support update freeze detection;
13. support delta updates;
14. support full-package fallback;
15. support server update;
16. support daemon update;
17. support agent update;
18. support CLI update;
19. support Dioxus desktop update;
20. support Android/mobile distribution;
21. support air-gapped update bundles;
22. support mirrors/CDNs;
23. support offline verification;
24. support compatibility gates;
25. support cluster rolling updates;
26. support update health;
27. support update audit;
28. support notifications;
29. support policy;
30. remain transport-independent.

---

# 4. Non-Goals

This subsystem does not:

```text
build Forgeyard
package Forgeyard
sign Forgeyard
decide release approval
replace platform app stores
replace OS package managers
```

It consumes already-released artifacts.

---

# 5. Workspace Structure

```text
crates/update/
├── forgeyard-update/
├── forgeyard-update-model/
├── forgeyard-update-feed/
├── forgeyard-update-verify/
├── forgeyard-update-select/
├── forgeyard-update-install/
├── forgeyard-update-rollout/
├── forgeyard-update-delta/
├── forgeyard-update-rollback/
├── forgeyard-update-health/
├── forgeyard-update-airgap/
└── forgeyard-update-testkit/
```

Platform adapters:

```text
crates/update-platform/
├── forgeyard-update-linux/
├── forgeyard-update-windows/
├── forgeyard-update-macos/
├── forgeyard-update-android/
├── forgeyard-update-ios/
└── forgeyard-update-generic/
```

Use modules first; split crates where platform/dependency boundaries justify.

---

# 6. UpdateTargetId

```rust
pub struct UpdateTargetId(Digest);
```

Represents exact release artifact intended for update installation.

---

# 7. Update Component Kind

```rust
pub enum UpdateComponentKind {
    ForgeyardStandalone,
    Daemon,
    Agent,
    Cli,
    Ui,
    Worker,
    SigningWorker,
    DeviceAgent,
    Custom(UpdateComponentKindId),
}
```

---

# 8. Update Target

```rust
pub struct UpdateTarget {
    pub id: UpdateTargetId,
    pub release: ReleaseId,
    pub component: UpdateComponentKind,
    pub platform: PlatformDescriptor,
    pub package: PackageId,
    pub digest: DigestSet,
}
```

---

# 9. Exact Release Binding

No update based only on:

```text
version string
tag
filename
```

---

# 10. Version

Display/ordering metadata.

---

# 11. ReleaseId

Authority for exact release identity.

---

# 12. Update Feed

```rust
pub struct UpdateFeed {
    pub id: UpdateFeedId,
    pub channel: ReleaseChannelId,
    pub generation: UpdateFeedGeneration,
    pub entries: Vec<UpdateFeedEntry>,
    pub signature: SignatureRef,
}
```

---

# 13. UpdateFeedId

Content-derived.

---

# 14. Feed Generation

Monotonic.

---

# 15. Signed Metadata

Update feed metadata is signed.

---

# 16. Transport

May be served through:

```text
HTTPS
CDN
object storage
mirror
air-gap file
```

---

# 17. Transport Is Untrusted

Critical.

---

# 18. Mirror/CDN Compromise

Cannot create trusted release if signature/digest validation is correct.

---

# 19. Update Root

Pinned trust root shipped with Forgeyard.

---

# 20. Root Rotation

Explicit signed/delegated process.

---

# 21. Multiple Keys

Support overlapping rotation.

---

# 22. Historical Key

Preserve for older release verification.

---

# 23. Metadata Expiry

Update metadata should have validity/expiry where appropriate.

---

# 24. Freeze Attack

Attacker serves old valid metadata forever.

---

# 25. Freeze Protection

Track:

```text
feed generation
signed timestamp/expiry
minimum accepted metadata version
```

---

# 26. Rollback Attack

Attacker serves older valid release.

---

# 27. Anti-Rollback

Installation can store:

```text
minimum trusted release generation
minimum security version
```

---

# 28. AntiRollbackPolicy

```rust
pub struct AntiRollbackPolicy {
    pub minimum_generation: Option<u64>,
    pub minimum_version: Option<ReleaseVersion>,
    pub allow_manual_downgrade: bool,
}
```

---

# 29. Manual Downgrade

High-risk explicit operation.

---

# 30. Security Minimum

Cannot be bypassed without break-glass/high privilege.

---

# 31. Release Channel

Examples:

```text
Nightly
Beta
Stable
LTS
```

---

# 32. Channel Is Mutable Pointer

Points to immutable releases.

---

# 33. Channel Move

Audited release operation.

---

# 34. Update Client

Resolves:

```text
configured channel
component
platform
current release
policy
```

---

# 35. Update Eligibility

```rust
pub struct UpdateEligibility {
    pub candidate: UpdateTargetId,
    pub compatible: bool,
    pub policy_allowed: bool,
    pub rollout_allowed: bool,
}
```

---

# 36. Compatibility

Must check:

```text
binary protocol
DB schema
config schema
plugin API
OS version
architecture
runtime requirements
```

---

# 37. Compatibility Matrix

Reuse Part 25/26 version compatibility.

---

# 38. N/N-1

Distributed components support rolling update matrix.

---

# 39. Mixed-Version Cluster

Allowed only where compatibility matrix says.

---

# 40. No Blind Agent Auto-Update

Agent checks daemon protocol compatibility.

---

# 41. Update Policy

Can specify:

```text
automatic
notify-only
admin-approved
maintenance-window
disabled
```

---

# 42. UpdateMode

```rust
pub enum UpdateMode {
    Automatic,
    NotifyOnly,
    Manual,
    ManagedExternally,
    Disabled,
}
```

---

# 43. ManagedExternally

For:

```text
apt/rpm
enterprise software distribution
Kubernetes
MDM
```

---

# 44. Self-Update

Not always appropriate.

---

# 45. CLI Self-Update

Can be optional.

---

# 46. Daemon

Admin-managed or rolling updater.

---

# 47. Agent

Central managed update useful.

---

# 48. Desktop UI

Native updater useful.

---

# 49. Mobile

Use platform app stores or enterprise distribution as appropriate.

---

# 50. Android

Support:

```text
Play Store
signed APK direct distribution
enterprise MDM
```

---

# 51. iOS

App Store/TestFlight/enterprise paths, subject to platform requirements.

---

# 52. Linux

Potential:

```text
tarball
AppImage-like bundle
deb
rpm
system package repository
```

---

# 53. Windows

Potential:

```text
MSI
MSIX
signed executable installer
```

---

# 54. macOS

Potential:

```text
pkg
dmg
signed/notarized app
```

---

# 55. Installer

Must verify package identity before replacing current installation.

---

# 56. Installer Privilege

Minimize.

---

# 57. Privileged Install Helper

Small typed interface.

---

# 58. No General Shell Installer Authority

Critical.

---

# 59. Atomic Install

Use:

```text
stage new version
verify
switch pointer/replace atomically
health check
commit
```

where platform permits.

---

# 60. Side-by-Side Installation

Preferred for daemon/agent when practical.

---

# 61. InstallSlotId

```rust
pub struct InstallSlotId(Ulid);
```

---

# 62. Active Slot

Mutable pointer.

---

# 63. Rollback Slot

Previous known-good.

---

# 64. Update State

```rust
pub enum UpdateState {
    Available,
    Downloading,
    Downloaded,
    Verifying,
    ReadyToInstall,
    Installing,
    HealthChecking,
    Committed,
    RolledBack,
    Failed,
}
```

---

# 65. Download Is Not Install

Separate.

---

# 66. Verify Before Install

Critical.

---

# 67. Install Before Commit

Health check.

---

# 68. Health Check

Component-specific.

---

# 69. Daemon Health

Examples:

```text
process alive
metadata DB reachable
API ready
protocol version valid
```

---

# 70. Agent Health

```text
starts
registers
heartbeat
capability report
```

---

# 71. Desktop Health

App launch/basic state.

---

# 72. Rollback Trigger

```text
process crash
failed readiness
protocol incompatibility
migration failure
```

---

# 73. DB Migration Complication

Cannot always rollback binary after irreversible schema migration.

---

# 74. Expand-Contract

Required.

---

# 75. Update Planner

Checks migration compatibility before install.

---

# 76. RollbackClass

```rust
pub enum RollbackClass {
    Safe,
    SafeBeforeMigrationCommit,
    ManualRecoveryRequired,
    Impossible,
}
```

---

# 77. UI Must Be Honest

Do not show rollback button when unsafe.

---

# 78. Delta Update

Optional optimization.

---

# 79. DeltaPackageId

```rust
pub struct DeltaPackageId(Digest);
```

---

# 80. Delta Identity

Bound:

```text
from exact digest
to exact digest
algorithm/version
```

---

# 81. Delta Apply

```text
verify current bytes
apply delta
rehash result
verify target digest
```

---

# 82. Delta Failure

Fallback full package.

---

# 83. Delta Is Not Trusted by Itself

Final target digest is authority.

---

# 84. Delta Generation

Release/post-release transform from exact package pair.

---

# 85. Delta Provenance

Recorded.

---

# 86. Staged Rollout

```rust
pub enum UpdateRollout {
    All,
    Percentage(RolloutPercentage),
    Cohort(UpdateCohortId),
    InstallationAllowlist(BTreeSet<InstallationId>),
    Region(UpdateRegionSelector),
}
```

---

# 87. Stable Bucketing

Same principle as feature flags.

---

# 88. Rollout Seed

Versioned.

---

# 89. Canary

Small cohort first.

---

# 90. Rollout Health

Observe:

```text
update failure
crash rate
agent reconnect
daemon readiness
```

---

# 91. Pause Rollout

If thresholds exceeded.

---

# 92. Rollout Controller

Advisory/desired-state coordinator.

---

# 93. Rollout Does Not Rewrite Release

Critical.

---

# 94. Bad Release

Channel can move back or release can be yanked.

---

# 95. Installed Client

May need explicit rollback/update-to-known-good.

---

# 96. Yank

Prevents new normal installs.

---

# 97. Existing Installed Release

Not erased.

---

# 98. Security Revocation

Can mark release compromised.

---

# 99. Compromised Release

Client should refuse future install if trust metadata updated.

---

# 100. Already Installed Compromised Release

Requires incident response/update.

---

# 101. Update Feed Freshness

Client tracks last successful check.

---

# 102. Offline Operation

Installed Forgeyard keeps running.

---

# 103. No Mandatory Phone-Home

Especially self-hosted/air-gap.

---

# 104. Offline Update

Import signed update bundle.

---

# 105. UpdateBundleId

```rust
pub struct UpdateBundleId(Digest);
```

---

# 106. Update Bundle

Contains:

```text
release manifest
packages
signatures
provenance
SBOM
feed/channel metadata
compatibility metadata
```

---

# 107. Air-Gap Flow

```text
internet-connected export
  ↓
signed update bundle
  ↓
offline transfer
  ↓
verify root/signatures/digests
  ↓
install
```

---

# 108. No Network Required

Critical.

---

# 109. Bundle Replay

Anti-rollback policy still applies.

---

# 110. Mirror

Mirror may cache:

```text
feed metadata
packages
delta packages
```

---

# 111. Mirror Trust

None beyond transport.

---

# 112. Enterprise Internal Mirror

Can improve availability/privacy.

---

# 113. Mirror Sync

Verifies upstream signatures/digests before publication.

---

# 114. Mirror Cannot Re-sign as Forgeyard Root

Unless explicitly delegated/private distribution architecture.

---

# 115. CDN

Cache immutable packages aggressively.

---

# 116. Immutable URLs

Prefer digest/release IDs.

---

# 117. Mutable Channel URL

Only signed metadata.

---

# 118. HTTP Cache Poisoning

Signature prevents authority compromise, but availability still affected.

---

# 119. Update Download Resume

Range/resume.

---

# 120. Partial Download

Never install.

---

# 121. Download Manifest

Tracks chunks/digest.

---

# 122. CAS Integration

Downloaded package can enter local CAS.

---

# 123. Install Materialization

From verified local CAS.

---

# 124. Package Authenticity

Release signature + digest.

---

# 125. Platform Signature

Additionally verify where relevant:

```text
Windows Authenticode
Apple signing/notarization
Android APK signature
```

---

# 126. Platform Signature vs Forgeyard Release Signature

Both can exist.

---

# 127. No Confusion

Forgeyard release signature proves Forgeyard release lineage.

Platform signature satisfies OS/platform trust requirements.

---

# 128. Update Check

```rust
pub struct UpdateCheckRequest {
    pub installation: InstallationId,
    pub component: UpdateComponentKind,
    pub current_release: ReleaseId,
    pub platform: PlatformDescriptor,
    pub channel: ReleaseChannelId,
}
```

---

# 129. Update Check Response

Returns exact eligible target metadata.

---

# 130. Privacy

Hosted update service does not need project/source data.

---

# 131. Self-Hosted

Can point update client at internal mirror.

---

# 132. Entitlement

Update eligibility may consider support channel/LTS entitlement, but security updates should not be dangerously withheld if contractual model can avoid it.

---

# 133. Security Baseline

Critical security fixes should have safe policy path.

---

# 134. Update Metadata

No secrets.

---

# 135. Agent Update

Central daemon can request:

```text
drain
download
install
restart
re-register
```

---

# 136. Drain First

Critical.

---

# 137. Active Job

Do not interrupt unless emergency.

---

# 138. Update Agent State

```text
Active
Draining
Updating
Restarting
Healthy
Failed
```

---

# 139. Scheduler

Stops new leases when agent draining.

---

# 140. Runner Host Update

Can be separate from agent binary.

---

# 141. OS/Image Update

Out of baseline scope, but provider adapter can coordinate.

---

# 142. Daemon Rolling Update

Mode 2:

```text
validate compatibility
  ↓
update follower/learner
  ↓
health
  ↓
repeat
  ↓
leadership transfer
  ↓
update old leader
```

---

# 143. HA

Never update quorum majority simultaneously.

---

# 144. Three-Voter Baseline

One at a time.

---

# 145. Update Coordinator

Uses cluster role/epoch.

---

# 146. Postgres Migration

Expand first.

---

# 147. Contract Later

After all supported binaries upgraded.

---

# 148. Feature Activation

After compatibility.

---

# 149. Agent Fleet Rollout

Canary pools.

---

# 150. Failure Budget

Pause if too many failed agents.

---

# 151. Retry

Bounded.

---

# 152. Manual Intervention

When rollback unsafe.

---

# 153. Signing Worker Update

Special.

---

# 154. Drain/disable signing first.

---

# 155. Verify new worker.

---

# 156. No Signing During Ambiguous Upgrade

High assurance.

---

# 157. Device Agent Update

Canary by lab/pool.

---

# 158. Desktop App Update

May:

```text
notify
download in background
install on restart
```

---

# 159. User Control

Respect configured update mode.

---

# 160. Mandatory Security Update

Policy may require minimum version.

---

# 161. User Experience

Explain why update required.

---

# 162. CLI Update

```text
forgeyard update check
forgeyard update download
forgeyard update install
forgeyard update rollback
forgeyard update status
```

---

# 163. Admin Fleet CLI

```text
forgeyard update fleet plan
forgeyard update fleet rollout
forgeyard update fleet pause
forgeyard update fleet resume
```

---

# 164. Dioxus UI

Pages:

```text
Updates
Release Channels
Fleet Rollout
Update Health
Air-Gap Bundles
```

---

# 165. Update Detail

Shows:

```text
current release
candidate
channel
signature status
compatibility
rollout eligibility
rollback class
```

---

# 166. Never Show Only Version String

Show release identity/digest where useful.

---

# 167. Update Plan

```rust
pub struct UpdatePlan {
    pub id: UpdatePlanId,
    pub current: ReleaseId,
    pub target: ReleaseId,
    pub components: Vec<ComponentUpdatePlan>,
    pub compatibility: UpdateCompatibility,
    pub rollback: RollbackClass,
}
```

---

# 168. UpdatePlanId

Content-derived.

---

# 169. Fleet Update Plan

Exact set/cohort.

---

# 170. Plan Staleness

If channel moves or compatibility changes, plan can be stale.

---

# 171. UpdatePlanFreshness

```rust
pub enum UpdatePlanFreshness {
    Current,
    StaleFeed,
    StalePolicy,
    StaleCompatibility,
}
```

---

# 172. Re-plan Before Protected Fleet Rollout

Critical.

---

# 173. Update Policy

Part 11 central policy.

---

# 174. Policy Inputs

```text
channel
release trust
security state
component role
platform
maintenance window
rollout health
```

---

# 175. Authz

Admin permission required for managed fleet update.

---

# 176. Automatic Client Update

Acts under installation/system policy, not arbitrary user permission.

---

# 177. Audit

Audit:

```text
channel change
fleet rollout start/pause
manual downgrade
rollback
trust-root change
mandatory update
```

---

# 178. Normal automatic client check

Not necessarily audit every poll.

---

# 179. Notifications

Examples:

```text
update available
mandatory security update
fleet rollout paused
update failure threshold exceeded
```

---

# 180. Security Incident Integration

Can activate:

```text
minimum safe version
release revocation
forced channel migration
```

---

# 181. Update Revocation

```rust
pub struct ReleaseRevocation {
    pub release: ReleaseId,
    pub reason: SecurityRevocationReason,
    pub effective_at: Timestamp,
    pub signature: SignatureRef,
}
```

---

# 182. Revocation Metadata

Signed.

---

# 183. Client Behavior

Do not newly install revoked release.

---

# 184. Installed Revoked Release

Warn/block protected operation according to severity/policy until updated.

---

# 185. No Remote Bricking

Critical.

Even compromised-version handling should preserve safe recovery/export path.

---

# 186. Update Feed Storage

Immutable feed generations.

---

# 187. Channel Head

Mutable pointer to signed feed generation.

---

# 188. Feed History

Retained.

---

# 189. Feed Equivocation

Same generation/different content is critical incident.

---

# 190. Update Metadata Signature

Verify before parsing deeply where format supports.

---

# 191. Parser Limits

Bound:

```text
feed entries
string lengths
package refs
delta refs
```

---

# 192. JSON/RON

External feed may use JSON for interoperability.

Internal canonical representation can use Postcard/RON.

---

# 193. TUF-Like Model

Architecture should align with mature update-framework principles:

```text
root trust
role separation
metadata versions
expiry
targets
```

---

# 194. No Need to Invent Weak Custom Trust

Critical.

---

# 195. Standard Crypto

Use audited libraries.

---

# 196. Key Roles

Potential:

```text
root
targets/release
snapshot/feed
timestamp
```

---

# 197. Root Key

Offline/high assurance.

---

# 198. Online Metadata Key

Shorter-lived/rotatable.

---

# 199. Release Signing Key

Can be separate from update metadata keys.

---

# 200. Compromise Containment

Online feed-key compromise should not equal root compromise.

---

# 201. Package Store

Immutable.

---

# 202. URL

Digest-based.

---

# 203. Package Missing

Update unavailable, not trust failure.

---

# 204. Partial Mirror

Client can try alternate mirror.

---

# 205. Mirror List

Signed/configured.

---

# 206. Mirror Selection

Performance only.

---

# 207. Mirror Priority

Local/internal first.

---

# 208. No Mirror-Supplied Unsigned Redirect Authority

---

# 209. Download TLS

Still required in connected mode.

---

# 210. Signature Protects Integrity; TLS Protects privacy/availability MITM class.

---

# 211. Update Telemetry

Optional.

---

# 212. Privacy

May report:

```text
installation ID pseudonymous
current version
update result
```

only if configured.

---

# 213. No Project Data

Critical.

---

# 214. Air-Gap

No telemetry.

---

# 215. Update Result

```rust
pub struct UpdateResult {
    pub plan: UpdatePlanId,
    pub state: UpdateState,
    pub installed_release: Option<ReleaseId>,
    pub rollback_release: Option<ReleaseId>,
}
```

---

# 216. Health Evidence

Can reference update result.

---

# 217. Failed Update

Preserve logs/diagnostics.

---

# 218. Update Logs

No secrets.

---

# 219. Update Doctor

```text
forgeyard update doctor
```

---

# 220. Doctor Checks

```text
trust root
feed signature
clock/freshness
current release identity
install slots
rollback availability
platform package compatibility
```

---

# 221. Update Health

```text
feed freshness
mirror availability
fleet rollout status
failure rate
stuck agents
```

---

# 222. Metrics

```text
update_check_total
update_available_total
update_download_failures_total
update_install_failures_total
update_rollbacks_total
update_rollout_paused
update_revoked_release_installed
```

---

# 223. Labels

Low cardinality:

```text
component
platform_class
channel
result
```

---

# 224. No Installation IDs in metrics.

---

# 225. Tracing

```text
update.check
update.verify
update.download
update.install
update.health
update.rollback
update.rollout
```

---

# 226. Search/Analytics

Part 31 can provide fleet update analytics.

---

# 227. Fleet Analytics

```text
adoption rate
failure rate
time-to-update
rollback rate
```

---

# 228. No Client Surveillance

Aggregate operations only.

---

# 229. API

Potential:

```text
GET  /v1/updates/check
GET  /v1/updates/status
POST /v1/updates/install
POST /v1/updates/rollback
GET  /v1/admin/update-rollouts
POST /v1/admin/update-rollouts
POST /v1/admin/update-rollouts/{id}/pause
```

---

# 230. Public Update Feed API

Separate unauthenticated signed metadata endpoint possible.

---

# 231. Signed Metadata Is Public

For public release.

---

# 232. Private/LTS Feed

May require auth/entitlement.

---

# 233. Authenticated Feed

Still signed.

---

# 234. Auth Is Access Control, signature is trust.

---

# 235. Entitlement

Can gate private channel access.

---

# 236. Entitlement Never Replaces Signature Verification

Critical.

---

# 237. Configuration

Part 39 configures:

```text
channel
update mode
mirror
maintenance window
rollout policy
```

---

# 238. Repository Config

Cannot change system update trust root.

---

# 239. System Admin Only

Trust root/channel policy.

---

# 240. Update Root Migration

Requires signed transition.

---

# 241. Backup/DR

Current installed release and rollback metadata backed up/configured.

---

# 242. Update Feed

Can be reconstructed from release metadata if signing keys/trust available, but historical feed should be retained.

---

# 243. Air-Gap Bundle

Independent recovery source.

---

# 244. Broken Updater Recovery

Manual installer from signed package remains supported.

---

# 245. Critical Escape Hatch

Forgeyard updater must not be sole path to obtain Forgeyard.

---

# 246. Last Known Good Release

Retain.

---

# 247. Recovery Command

```text
forgeyard update rollback
```

when safe.

---

# 248. Manual Recovery

Document signed package verification.

---

# 249. Installer Bootstrap

Initial install has no existing updater.

---

# 250. Bootstrap Trust

Download package + signature/checksum from release source.

---

# 251. Distribution Website

Provides convenience.

---

# 252. Verification

Can be independent CLI/tool/manual public key verification.

---

# 253. Self-Hosting

Stage 0 bootstrap can install known-good release.

---

# 254. Release-of-Forgeyard Integration

Part 26 output feeds Part 41.

---

# 255. No Circular Dependency

New installation can verify/install without already-running Forgeyard.

---

# 256. Install Metadata

Store:

```text
InstallationId
current ReleaseId
component version
update root version
channel
```

---

# 257. InstallationId

Stable across normal updates.

---

# 258. Reinstall

Can preserve or regenerate according to admin intent.

---

# 259. Clone Detection

Not core updater responsibility.

---

# 260. Testkit

```text
forgeyard-update-testkit/src/
├── lib.rs
├── feed.rs
├── verify.rs
├── selection.rs
├── install.rs
├── delta.rs
├── rollback.rs
├── rollout.rs
└── assertions.rs
```

---

# 261. Unit Tests

Feed signature/generation.

---

# 262. Tampered Feed Test

Rejected.

---

# 263. Tampered Package Test

Digest mismatch.

---

# 264. Old Feed Freeze Test

Detected by generation/expiry.

---

# 265. Rollback Attack Test

Older release rejected by anti-rollback policy.

---

# 266. Manual Downgrade Test

Requires explicit elevated action.

---

# 267. Delta Test

Final target digest verified.

---

# 268. Delta Corruption Test

Fallback full package.

---

# 269. Mirror Compromise Test

Invalid bytes rejected.

---

# 270. Mirror Outage Test

Alternate mirror/full fallback.

---

# 271. Agent Drain Test

No new jobs during update.

---

# 272. Agent Failed Update Test

Rollback/recovery path.

---

# 273. Daemon Rolling Upgrade Test

Quorum preserved.

---

# 274. Mixed-Version Compatibility Test

Unsupported rollout blocked.

---

# 275. Migration Rollback Test

Unsafe rollback clearly refused.

---

# 276. Rollout Canary Test

Stable bucketing.

---

# 277. Failure Budget Test

Rollout pauses.

---

# 278. Revoked Release Test

New install blocked.

---

# 279. Installed Revoked Release Test

Security state surfaced.

---

# 280. Air-Gap Test

Bundle installs without internet.

---

# 281. Feed Key Rotation Test

Old/new overlap.

---

# 282. Root Rotation Test

Signed transition.

---

# 283. Private Feed Entitlement Test

Access controlled but signature still verified.

---

# 284. Tenant Isolation Test

Private channel metadata scoped.

---

# 285. Parser Fuzzing

Feed/manifest/delta metadata.

---

# 286. Failure Injection

```text
power loss during install
disk full
network drop
process crash
health check timeout
```

---

# 287. Atomicity Test

Old version remains recoverable if install fails before commit.

---

# 288. Load Test

Large agent fleet rollout.

---

# 289. Implementation Phase 1 — Signed Update Feed

Release target metadata.

---

# 290. Phase 2 — Linux/CLI/Agent Installer

Dogfood.

---

# 291. Phase 3 — Rollback Slots/Health

Reliability.

---

# 292. Phase 4 — Distributed Agent Fleet Rollout

Drain/update/reconnect.

---

# 293. Phase 5 — Daemon Rolling Upgrade

HA.

---

# 294. Phase 6 — Desktop Update

Dioxus client.

---

# 295. Phase 7 — Windows/macOS Platform Signing

Native distribution.

---

# 296. Phase 8 — Android/Mobile Distribution

Play/direct/enterprise.

---

# 297. Phase 9 — Delta Updates

Optimization.

---

# 298. Phase 10 — Air-Gap/Internal Mirrors

Enterprise.

---

# 299. Phase 11 — Anti-Rollback/Revocation Hardening

Security.

---

# 300. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 301. Acceptance Tests

1. Update installs exact already-released artifacts.
2. Distribution never rebuilds or repackages release bytes.
3. Update feed metadata is signed.
4. Package bytes are digest verified.
5. Mirror/CDN transport is not trust authority.
6. ReleaseId/digest is authoritative over version/filename.
7. Feed generation/expiry protect against freeze attacks.
8. Anti-rollback policy blocks unauthorized downgrade.
9. Manual downgrade is privileged/audited.
10. Platform signatures complement but do not replace Forgeyard release trust.
11. Delta update verifies exact final target digest.
12. Delta failure safely falls back to full package.
13. Agent drains before update.
14. Distributed daemon rollout preserves quorum.
15. Mixed-version compatibility is validated before rollout.
16. Expand-contract migration rules constrain rollback.
17. Unsafe rollback is explicitly refused.
18. Staged rollout uses deterministic cohorts.
19. Fleet rollout pauses on configured failure thresholds.
20. Revoked releases are not newly installed.
21. Installed revoked releases surface a security state and recovery path.
22. Air-gap bundles verify/install without internet.
23. Root/feed key rotation preserves continuity.
24. Private feed entitlement does not replace signature verification.
25. Update trust root cannot be changed by repository/project config.
26. Update logs contain no secrets.
27. Initial install can verify Forgeyard without an already-running Forgeyard instance.
28. Last known-good release remains recoverable.
29. Updater failure cannot permanently strand installation without manual signed-package recovery path.
30. Mirror outage reduces availability, not trust.
31. Failed install leaves previous version recoverable where rollback class allows.
32. Fleet update state is auditable/explainable.
33. Standalone/distributed share update trust semantics.
34. Update analytics preserve user privacy.
35. Forgeyard dogfoods this update system for its own daemon/agent/CLI/UI releases.

---

# 302. Production Readiness Gates

Do not call Forgeyard update delivery production-ready until:

```text
signed feed verification is stable
digest/anti-rollback/freeze protection passes
atomic install/rollback is tested
agent drain/update/reconnect works
daemon rolling upgrade preserves quorum
mixed-version compatibility is enforced
air-gap bundle works
root/feed key rotation is tested
manual recovery path is documented/tested
large fleet rollout failure handling passes
```

---

# 303. Architectural Invariants

1. update delivery consumes immutable released artifacts;
2. distribution never rebuilds;
3. transport is not trust authority;
4. signed metadata defines trusted update targets;
5. exact digest/ReleaseId outranks version string;
6. feed metadata is versioned/generation tracked;
7. freeze and rollback attacks are explicitly defended;
8. platform signatures complement Forgeyard release signatures;
9. delta updates verify final target digest;
10. failed delta falls back safely;
11. installer stages/verifies before activation;
12. rollback capability is explicit/honest;
13. unsafe rollback is never pretended safe;
14. agent drains before self-update;
15. daemon rolling update preserves quorum;
16. mixed-version compatibility is checked;
17. rollout is deterministic and pauseable;
18. revoked releases are not newly installed;
19. installed compromised versions have recovery path;
20. air-gap updates do not require phone-home;
21. mirrors/CDNs cannot forge trust;
22. entitlement controls access, not authenticity;
23. repository config cannot rewrite update trust roots;
24. updater is not the only recovery/install path;
25. initial install can verify independently;
26. update logs/metadata contain no secret values;
27. current/rollback release identities are explicit;
28. standalone/distributed share trust semantics;
29. fleet rollout is auditable;
30. Forgeyard dogfoods its own update delivery system.

---

# 304. Final Target Architecture

```text
                  Immutable Release
                         │
                         ▼
                  Signed Metadata
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
          CDN          Mirror      Air-Gap
            │            │            │
            └────────────┼────────────┘
                         ▼
                    Update Client
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
        Signature      Digest      Eligibility
            │            │            │
            └────────────┼────────────┘
                         ▼
                    Stage Install
                         │
                         ▼
                    Health Check
                         │
              ┌──────────┼──────────┐
              ▼                     ▼
           Commit                Rollback
```

---

# 305. Final Architectural Position

Update selection:

```text
current ReleaseId
+
channel
+
platform/component
+
signed feed generation
+
compatibility/policy
  ↓
exact UpdateTargetId
```

Install:

```text
download immutable package
  ↓
verify signature + digest
  ↓
stage side-by-side
  ↓
health check
  ↓
commit active slot
```

Fleet rollout:

```text
canary cohort
  ↓
observe health
  ↓
expand rollout
  ↓
pause on failure budget
  ↓
complete
```

The key guarantee is:

> **Forgeyard can distribute and update itself across local machines, clusters, agents, desktops, and air-gapped environments without trusting the CDN, mirror, package host, or version string. Every installed update is an exact previously released artifact whose signatures, digests, compatibility, rollout eligibility, and rollback properties are verified before activation.**

---

# 306. Extended Architecture Sequence

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
```
