# 40 — Forgeyard Security Architecture, Threat Model, Hardening & Incident Response System Architecture

**Document type:** End-to-End Security Architecture, Threat Model, Hardening, Detection, Containment, Forensics & Incident Response  
**Project:** Forgeyard CI/CD  
**Subsystem:** cross-cutting security model spanning edge/API, identity, policy, secrets, SCM, control plane, scheduler, runners, sandboxes, plugins, device lab, CAS, metadata DB, dependency sources, cache, signing, release, deployment, audit, configuration, and recovery  
**Architecture style:** Defense-in-depth, zero-trust-oriented boundaries, least privilege, immutable identities/evidence, explicit trust classes, revocable credentials, compartmentalization, fail-closed protected writes, conservative recovery, and evidence-preserving incident response  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** This document does not replace Parts 08, 11, 12, 13, 17, 21, 22, 24, 25, 27, 28, 36, 37, 38, or 39. It unifies their security properties into one end-to-end threat model and response architecture.

---

# 1. Purpose

Forgeyard is unusually security-sensitive because it sits between:

```text
source code
credentials
build infrastructure
package ecosystems
signing keys
release channels
production deployment
```

A compromised CI/CD system can become a software-supply-chain compromise.

The central rule is:

> **No individual untrusted build, user, plugin, runner, external provider, or cache result may become control-plane, signing, release, or deployment authority merely by existing inside Forgeyard.**

A second rule is:

> **Every trust boundary must authenticate, authorize, validate, constrain, and record the operation crossing it. Network location alone is never sufficient trust.**

A third rule is:

> **Compromise must be containable. Forgeyard should prefer short-lived identities, narrow scopes, isolated execution, revocable trust, immutable evidence, and explicit epochs so an incident can be bounded instead of forcing blind global distrust.**

---

# 2. Security Objectives

Forgeyard MUST protect:

1. confidentiality of source code;
2. confidentiality of secrets;
3. integrity of source snapshots;
4. integrity of build inputs;
5. integrity of build outputs;
6. integrity of metadata;
7. integrity of CAS content;
8. integrity of policies;
9. integrity of approvals;
10. integrity of release candidates;
11. integrity of signing operations;
12. integrity of deployments;
13. tenant isolation;
14. runner isolation;
15. control-plane availability;
16. auditability;
17. recovery evidence;
18. revocation capability;
19. provenance trust;
20. operator accountability.

---

# 3. Security Non-Goals

Forgeyard cannot guarantee:

```text
perfect sandboxing against unknown kernel vulnerabilities
perfect secret redaction against malicious transforms
perfect detection of all supply-chain malware
perfect offline-license revocation
perfect prevention of compromised cloud-provider control planes
perfect global time synchronization
```

Security architecture must state real guarantees, not impossible ones.

---

# 4. Threat Actors

```rust
pub enum ThreatActorClass {
    UnauthenticatedExternal,
    AuthenticatedMaliciousUser,
    MaliciousTenant,
    CompromisedHumanAccount,
    MaliciousSourceContributor,
    MaliciousFork,
    CompromisedRunner,
    CompromisedDaemon,
    CompromisedPlugin,
    CompromisedScmProvider,
    CompromisedDependencyRegistry,
    CompromisedCacheBackend,
    CompromisedMetadataDatabase,
    CompromisedCasBackend,
    MaliciousInsider,
    CompromisedSigningKey,
    CompromisedSecretProvider,
    CloudInfrastructureCompromise,
}
```

---

# 5. Primary Assets

```text
SourceSnapshot
PipelinePlan
JobSpec
JobLease
RunnerIdentity
SecretRef / SecretValue
PolicyBundle
ConfigSnapshot
DependencyClosure
CAS objects
Artifact / Package
ReleaseCandidate
ReleaseId
DeploymentPlan
SigningKeyRef
AuditRecord
TrustRoot
```

---

# 6. Trust-Boundary Model

```text
Internet / SCM / Users
        │
        ▼
Edge / API / Webhook
        │
        ▼
Authentication / Authorization / Policy
        │
        ▼
Control Plane / Metadata
        │
        ▼
Scheduler / Lease Authority
        │
        ▼
Agent Transport
        │
        ▼
Runner Host
        │
        ▼
Sandbox / VM
        │
        ▼
Untrusted Build Code
```

Parallel protected boundaries:

```text
Control Plane ──► Secret Provider
Control Plane ──► CAS
Control Plane ──► PostgreSQL
Control Plane ──► Signing Worker/KMS/HSM
Control Plane ──► SCM Provider
Control Plane ──► Deployment Provider
```

---

# 7. Security Principle: Explicit Trust

Use typed trust classes.

```rust
pub enum TrustClass {
    Untrusted,
    Low,
    Standard,
    Privileged,
    HighAssurance,
}
```

---

# 8. Trust Is Provisioned, Not Claimed

A runner cannot promote itself.

A plugin cannot request arbitrary trust.

A workload cannot inherit a user's entire permission set.

---

# 9. Security Principle: Least Privilege

Every identity gets only:

```text
required action
required resource scope
required duration
required environment
```

---

# 10. Security Principle: Short-Lived Credentials

Prefer:

```text
short-lived workload credentials
short-lived runner certificates
OIDC federation
ephemeral deployment credentials
```

over static long-lived secrets.

---

# 11. Security Principle: Immutable Subjects

Protected approvals/signatures bind:

```text
digest
revision
snapshot
candidate
```

not mutable names.

---

# 12. Security Principle: No Ambient Authority

Avoid:

```text
Docker socket
SSH agent
cloud metadata credentials
host home directory
system DBus
unscoped filesystem
```

inside normal build sandboxes.

---

# 13. Security Principle: Fail Closed for Protected Writes

On uncertainty:

```text
release
sign
deploy
trust change
secret reveal
break-glass
```

should fail closed.

---

# 14. Security Principle: Availability Degrades Before Security

Example:

```text
signing service unavailable
```

means signing pauses.

It does not fall back to a weaker key.

---

# 15. Security Zones

```rust
pub enum SecurityZone {
    Edge,
    ControlPlane,
    Metadata,
    Cas,
    RunnerControl,
    RunnerExecution,
    Signing,
    Secrets,
    ProviderIntegration,
    Device,
    Recovery,
}
```

---

# 16. Zone Crossing

Each crossing requires explicit authenticated protocol.

---

# 17. Edge/API Attack Surface

Includes:

```text
REST
SSE
WebSocket
webhook endpoints
RBE gRPC
login/session endpoints
artifact upload/download
admin API
```

---

# 18. Edge Hardening

Require:

```text
TLS
body limits
timeouts
rate limits
CORS rules
CSRF protection
request IDs
safe proxy handling
```

---

# 19. HTTP Request Limits

Bound:

```text
headers
body
multipart parts
JSON depth
query complexity
```

---

# 20. Webhook Security

Flow:

```text
receive raw body
  ↓
verify signature
  ↓
validate timestamp/replay window
  ↓
deduplicate delivery
  ↓
persist
  ↓
normalize asynchronously
```

---

# 21. Webhook Signer Is Not Human Identity

Critical.

---

# 22. API Token Security

Tokens:

```text
scoped
hashed at rest
expiring
revocable
audited
```

---

# 23. Browser Session Security

Use:

```text
Secure
HttpOnly
SameSite
CSRF defense
rotation
```

---

# 24. Authentication Assurance

High-risk actions may require step-up MFA.

---

# 25. Identity Linking

Never link external accounts based only on email equality.

---

# 26. OIDC

Validate:

```text
issuer
audience
nonce/state
signature
expiry
subject
```

---

# 27. SAML

If supported, use mature library and strict metadata/trust configuration.

---

# 28. SCIM

Provisioning is lifecycle input, not auth bypass.

---

# 29. Authorization Confused-Deputy Defense

Service identity must preserve:

```text
requesting actor
effective workload
resource scope
```

---

# 30. Workload Identity

Human starts job:

```text
Human Principal
  ↓
authorized request
  ↓
constrained WorkloadIdentity
```

---

# 31. Workload Cannot Act as Full Human

Critical.

---

# 32. Policy Digest

Protected action records exact `PolicyDigest`.

---

# 33. Config Digest

Protected action may record exact `ConfigSnapshotId`.

---

# 34. Tenant Isolation

Every tenant-owned object has explicit `TenantId`.

---

# 35. Tenant ID from Request Header Is Not Authority

Resolve from authenticated resource binding.

---

# 36. Cross-Tenant Reference

Rejected unless explicit shared-resource type.

---

# 37. PostgreSQL Isolation

Use:

```text
tenant-scoped queries
constraints
optional RLS
separate service roles
```

---

# 38. DB Least Privilege

Application role should not own schema superuser privileges.

---

# 39. Migration Role

Separate.

---

# 40. Backup Role

Separate.

---

# 41. DB TLS

Required for remote DB.

---

# 42. SQL Injection

Parameterized queries only.

---

# 43. Dynamic SQL

Allowlisted identifiers, never raw user strings.

---

# 44. CAS Security

Digest grants identity, not authorization.

---

# 45. Knowing Digest Does Not Grant Read

Critical.

---

# 46. CAS Authorization

Check tenant/resource reference before issuing object access.

---

# 47. Presigned URLs

Short-lived and scoped.

---

# 48. CAS Upload

Verify claimed digest after upload.

---

# 49. CAS Substitution Attack

Different bytes cannot satisfy same digest.

---

# 50. Digest Alias

BLAKE3/SHA-256 aliases must be verified against same bytes.

---

# 51. CAS Corruption

Detect, quarantine, repair.

---

# 52. Cache Poisoning

See Part 38.

Security response:

```text
same key + different output
  ↓
quarantine
  ↓
producer investigation
```

---

# 53. Dependency Supply-Chain Attack

Threats:

```text
typosquat
dependency confusion
maintainer compromise
registry equivocation
malicious build scripts
```

---

# 54. Dependency Controls

Use Part 36:

```text
exact lock
digest
source mapping
quarantine
promotion
sandbox
```

---

# 55. Registry Credentials

Fetcher only.

Never exposed to build process.

---

# 56. Source Trust

```rust
pub enum SourceTrustClass {
    TrustedInternal,
    ExternalContribution,
    Fork,
    Unknown,
}
```

---

# 57. Fork Builds

No privileged secrets by default.

---

# 58. Fork Builds

No production signing.

---

# 59. Fork Builds

Restricted network.

---

# 60. Source Archive Extraction

Protect:

```text
path traversal
symlink escape
device nodes
decompression bombs
```

---

# 61. Command Injection

Execution API uses typed argv.

---

# 62. Shell

Disabled by default.

---

# 63. Shell Command

Explicit:

```rust
CommandSpec::Shell(...)
```

with policy.

---

# 64. No String Concatenation for Shell

Critical.

---

# 65. Environment Injection

Validate env keys.

---

# 66. PATH

Controlled.

---

# 67. Filesystem Attacks

Threats:

```text
symlink race
hardlink escape
path traversal
TOCTOU
mount trick
```

---

# 68. Workspace Materialization

Use safe path joining.

---

# 69. Symlink

Do not follow outside workspace root.

---

# 70. Output Collection

Declared paths only.

---

# 71. Host Ownership

Sandbox user cannot write host-sensitive directories.

---

# 72. Runner Threat Model

Runner executes hostile code.

Therefore runner is not inherently trusted merely because enrolled.

---

# 73. Runner Control vs Execution Plane

Separate as much as possible.

---

# 74. Agent Process

Thin trusted control.

---

# 75. Build Process

Untrusted.

---

# 76. Privileged Helper

Small typed interface.

No shell.

No general filesystem API.

---

# 77. Linux Hardening

Baseline:

```text
namespaces
cgroup v2
seccomp
capability drop
no_new_privs
read-only mounts
private /proc where possible
tmpfs scratch
```

---

# 78. Rootless

Prefer where compatible.

---

# 79. Container Is Not VM

Critical.

---

# 80. Hostile Multi-Tenant Workload

Use VM-capable isolation.

---

# 81. Windows Hardening

Use:

```text
Job Objects
restricted tokens
AppContainer where possible
ACLs
Firewall rules
Hyper-V/VM for stronger boundary
```

---

# 82. macOS Hardening

Use available sandbox/process controls honestly.

For hostile multi-tenant work, VM boundary preferred.

---

# 83. Device Security

Device test data must be wiped/reset before reuse.

---

# 84. Device Quarantine

Mandatory after reset failure.

---

# 85. USB Device

Explicit leased capability.

---

# 86. GPU

Explicit leased capability.

---

# 87. Cloud Metadata

Blocked from untrusted workloads.

---

# 88. Network Policy

```rust
pub enum NetworkSecurityPolicy {
    Deny,
    FetchOnly,
    Restricted,
    Allow,
}
```

---

# 89. Egress Allowlist

For restricted jobs.

---

# 90. DNS Rebinding

Resolve/validate host and IP class carefully for protected fetchers.

---

# 91. SSRF Defense

Apply to:

```text
SCM URLs
webhook destinations
plugin host-mediated HTTP
deployment providers
generic fetchers
notification webhooks
```

---

# 92. Private IP Ranges

Blocked by default in hosted external-URL features.

---

# 93. Redirect Revalidation

Every redirect target rechecked.

---

# 94. Plugin Security

Plugins are not trusted core merely because installed.

---

# 95. Plugin Permission Model

No:

```text
DatabaseAccess
BypassAuthz
ScheduleJob
MarkSuccess
```

---

# 96. Plugin Host-Mediated Network

Preferred.

---

# 97. Plugin Secret Access

Exact SecretRef purpose only.

---

# 98. Plugin Sandbox

External plugin:

```text
filesystem deny
network deny
bounded CPU/memory
restart limits
```

---

# 99. Plugin Crash Loop

Quarantine.

---

# 100. Plugin Supply Chain

Require:

```text
signature
SBOM
provenance
allowlist
```

for enterprise profiles.

---

# 101. RBE Security

External action input is untrusted.

---

# 102. RBE Instance Name

Maps server-side to tenant/project.

---

# 103. RBE Priority

Cannot self-elevate above policy.

---

# 104. RBE Platform Properties

Unknown correctness property -> reject.

---

# 105. RBE CAS

Tenant isolation.

---

# 106. RBE Action Cache

Trust-aware.

---

# 107. Secret Security

Secret values:

```text
never metadata
never CAS
never events
never logs
never UI persistence
```

---

# 108. Late Secret Resolution

Resolve after runner/lease authorization.

---

# 109. SecretLease

Bind:

```text
JobId
AttemptId
LeaseId
RunnerId
purpose
expiry
```

---

# 110. Secret Zeroization

Best effort in memory.

---

# 111. Core Dumps

Disable for secret-bearing processes where practical.

---

# 112. Redaction

Defense-in-depth, not perfect confidentiality.

---

# 113. Malicious Transformation

Can evade redactor.

Do not claim otherwise.

---

# 114. Signing Security

Signing is highest-trust domain.

---

# 115. General Runner Cannot Access Production Signing Key

Critical.

---

# 116. Restricted Signing Worker

Can sign.

Cannot compile arbitrary code.

---

# 117. KMS/HSM

Prefer provider operation over exported private key.

---

# 118. Signing Request

Bind exact:

```text
artifact digest
EvidenceBundleId
PolicyDigest
SigningKeyRef
```

---

# 119. Signing Result

New immutable object if bytes change.

---

# 120. Release Security

Build once → verify → sign → promote same bytes.

---

# 121. No Release Rebuild

Critical supply-chain invariant.

---

# 122. Approval

Binds exact `ReleaseCandidateId`.

---

# 123. Candidate Change

Invalidates approval.

---

# 124. Deployment Security

Deployment consumes exact released artifact.

---

# 125. Cloud Credentials

Prefer federation.

---

# 126. Deployment Provider Scope

Environment-specific.

---

# 127. Production Deployment

Step-up/approval according to policy.

---

# 128. Configuration Security

Part 39.

---

# 129. Repository Config

Cannot grant privileged capabilities.

---

# 130. Runtime Config

Authorized, versioned, audited.

---

# 131. Kill Switch

Security containment mechanism.

---

# 132. Feature Flag

Never used to bypass security enforcement.

---

# 133. Update Security

Forgeyard updater/release feed must be signed.

---

# 134. Mirror/CDN

Not trust authority.

---

# 135. Anti-Rollback

Update policy can enforce minimum trusted version.

---

# 136. Replay/Frozen Feed

Track signed metadata expiry/version where supported.

---

# 137. Root Rotation

Defined explicit process.

---

# 138. CompromiseEpoch

```rust
pub struct CompromiseEpoch(u64);
```

---

# 139. Purpose

Invalidate trust created before/within compromise scope.

---

# 140. TrustEpoch

Existing trust model.

---

# 141. RunnerEpoch

Can quarantine/invalidate stale runner trust.

---

# 142. SigningTrustEpoch

Can mark signatures after compromised key window.

---

# 143. SecurityState

```rust
pub enum SecurityState {
    Trusted,
    Quarantined,
    Compromised,
    Unknown,
}
```

---

# 144. Apply to Metadata

Do not mutate artifact bytes.

---

# 145. Artifact Trust Change

Bytes remain immutable.

Metadata trust state changes.

---

# 146. Security Event

Separate from normal observability log.

---

# 147. SecurityEventId

```rust
pub struct SecurityEventId(Ulid);
```

---

# 148. Security Event Categories

```text
Authentication
Authorization
SecretAccess
TrustChange
SandboxViolation
IntegrityFailure
PolicyBypassAttempt
CachePoisoning
DependencyEquivocation
SigningAnomaly
TenantIsolation
Configuration
```

---

# 149. Security Severity

```rust
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 150. Detection Sources

```text
audit
authn
authz
sandbox
CAS verifier
cache verifier
dependency verifier
scanner
signing worker
provider adapters
configuration drift
```

---

# 151. Detection Rules

Examples:

```text
repeated failed MFA
impossible token usage
runner trust change
unexpected secret.use spike
cache equivocation
CAS digest mismatch
signing key use outside release workflow
```

---

# 152. Rate/Anomaly Detection

Derived analytics.

---

# 153. No Opaque AI Security Authority

Critical.

---

# 154. Detection Confidence

Explicit.

---

# 155. Alert Routing

Part 29.

---

# 156. Audit

Part 28.

---

# 157. IncidentId

```rust
pub struct IncidentId(Ulid);
```

---

# 158. Incident State

```rust
pub enum IncidentState {
    Detected,
    Triaged,
    Containing,
    Eradicating,
    Recovering,
    Monitoring,
    Closed,
}
```

---

# 159. Incident Severity

```rust
pub enum IncidentSeverity {
    Sev4,
    Sev3,
    Sev2,
    Sev1,
}
```

---

# 160. Incident Record

```rust
pub struct SecurityIncident {
    pub id: IncidentId,
    pub severity: IncidentSeverity,
    pub state: IncidentState,
    pub opened_at: Timestamp,
    pub affected_scopes: Vec<SecurityScope>,
    pub evidence: Vec<EvidenceRef>,
}
```

---

# 161. Security Scope

```text
Principal
Runner
Tenant
Project
Plugin
SigningKey
Secret
Release
Cluster
Provider
```

---

# 162. Incident Response Phases

```text
detect
  ↓
triage
  ↓
contain
  ↓
eradicate
  ↓
recover
  ↓
monitor
  ↓
postmortem
```

---

# 163. Triage

Determine:

```text
what happened?
what identities?
what scope?
what time window?
what artifacts/releases?
what credentials?
```

---

# 164. Containment

Examples:

```text
revoke token
quarantine runner
disable plugin
freeze signing
freeze release
suspend tenant
disable provider
kill feature
```

---

# 165. Eradication

Examples:

```text
patch vulnerable software
replace compromised host
rotate credential
remove malicious dependency
rebuild clean image
```

---

# 166. Recovery

Restore from verified trusted state.

---

# 167. Monitoring

Heightened detection after restore.

---

# 168. Postmortem

Record:

```text
root cause
scope
timeline
detection gaps
containment gaps
corrective actions
```

---

# 169. Incident Evidence

Immutable refs.

---

# 170. Forensic Preservation

Preserve:

```text
audit records
logs
runner/session IDs
source snapshots
artifact digests
CAS refs
policy/config digests
provider event IDs
```

---

# 171. Do Not Destroy Compromised Host Evidence Prematurely

Snapshot/collect before rebuild where safe/legal.

---

# 172. Forensic Bundle

```rust
pub struct ForensicBundle {
    pub incident: IncidentId,
    pub manifest: ForensicManifest,
    pub artifacts: Vec<EvidenceRef>,
    pub digest: Digest,
}
```

---

# 173. Bundle Integrity

Sign/hash.

---

# 174. Legal/Privacy

Access highly restricted.

---

# 175. Time Synchronization

Security timeline records server timestamps and known clock uncertainty.

---

# 176. Playbook 1 — Stolen API Token

Contain:

```text
revoke token
revoke sessions if needed
increase principal security epoch
search audit for use
```

---

# 177. Scope

Identify resources accessed.

---

# 178. Recovery

Issue new token only after account assurance.

---

# 179. Playbook 2 — Compromised Runner

Immediate:

```text
quarantine RunnerId
revoke cert/session
stop new leases
invalidate active lease
```

---

# 180. Artifact Investigation

Find outputs produced during compromise window.

---

# 181. Cache Investigation

Quarantine cache entries produced by runner/time range.

---

# 182. Release Investigation

Identify whether any release evidence depends on compromised outputs.

---

# 183. Runner Recovery

Reimage/re-enroll as new trusted identity/session.

---

# 184. Never Simply Re-enable Old Host After "cleanup"

High-assurance recommendation.

---

# 185. Playbook 3 — Compromised Daemon

Treat reachable credentials as compromised.

---

# 186. Immediate

```text
remove node
revoke service cert
advance epochs
freeze signing/release/deploy
```

---

# 187. Rebuild

Use clean host/image.

---

# 188. Verify

```text
DB
CAS
policy
config
audit
```

---

# 189. Rotate

All credentials reachable by daemon.

---

# 190. Playbook 4 — Secret Provider Credential Compromise

Revoke provider credential.

---

# 191. Determine Accessible Secret Scope

Rotate affected secrets.

---

# 192. Workload Credentials

Expire/revoke.

---

# 193. Audit secret access.

---

# 194. Playbook 5 — Signing Key Compromise

Highest severity.

---

# 195. Immediate

```text
freeze signing
freeze release publication
revoke/disable key
activate security notice
```

---

# 196. Identify Window

Which signatures could be compromised.

---

# 197. Mark Affected Signatures/Artifacts

`SecurityState::Compromised` or `Unknown`.

---

# 198. Rotate Key

Via trust-root/delegation process.

---

# 199. Re-signing

Only after verifying clean artifact lineage.

---

# 200. Rebuild

Required if artifact itself may be compromised.

---

# 201. Do Not Blindly Re-sign Existing Bytes

Critical.

---

# 202. Playbook 6 — Root CA Compromise

Extreme severity.

---

# 203. Replace root/intermediates.

---

# 204. Re-enroll runners/services.

---

# 205. Treat existing cert identity assertions as suspect.

---

# 206. Advance trust epoch.

---

# 207. Playbook 7 — Malicious Release Published

Immediate:

```text
yank/freeze channel
stop rollout
rollback deployment
notify consumers
```

---

# 208. Preserve Malicious Release

Do not delete evidence.

---

# 209. Determine Root Cause

```text
source compromise
runner compromise
signing compromise
policy bypass
operator error
```

---

# 210. Clean Release

Use clean source/build/sign path.

---

# 211. Playbook 8 — CAS Corruption/Tampering

Verify affected digest/object.

---

# 212. Quarantine.

---

# 213. Replicate/restore known-good bytes.

---

# 214. Identify consumers.

---

# 215. Digest mismatch

Never reinterpret as same object.

---

# 216. Playbook 9 — Metadata DB Compromise

Freeze protected writes.

---

# 217. Preserve DB image/logs.

---

# 218. Restore/compare from backup.

---

# 219. Verify CAS/release/audit references.

---

# 220. Rotate DB credentials.

---

# 221. Tenant breach assessment.

---

# 222. Playbook 10 — Tenant Isolation Breach

Sev1/Sev2 depending scope.

---

# 223. Suspend affected cross-tenant path.

---

# 224. Preserve access evidence.

---

# 225. Determine exposure.

---

# 226. Rotate credentials if secrets exposed.

---

# 227. Correct isolation defect before re-enable.

---

# 228. Playbook 11 — Dependency Supply-Chain Compromise

Freeze registry/source.

---

# 229. Deny affected hash/version.

---

# 230. Identify all builds/releases containing dependency.

---

# 231. Re-evaluate SBOM/provenance.

---

# 232. Patch/rebuild.

---

# 233. Notify affected users.

---

# 234. Playbook 12 — SCM Provider Token Compromise

Revoke installation/token.

---

# 235. Freeze merge/integration if provider integrity uncertain.

---

# 236. Reconcile refs/proposals from VCS truth.

---

# 237. Rotate webhook secrets.

---

# 238. Playbook 13 — Plugin Compromise

Disable/quarantine plugin.

---

# 239. Revoke plugin credentials.

---

# 240. Identify host-mediated actions performed.

---

# 241. Rebuild plugin state from trusted source.

---

# 242. Review affected outputs.

---

# 243. Playbook 14 — Ransomware / Backup Compromise

Isolate writable systems.

---

# 244. Protect immutable/offline backups.

---

# 245. Restore to clean infrastructure.

---

# 246. Verify backup integrity and key trust.

---

# 247. Rotate credentials.

---

# 248. Playbook 15 — DDoS / Resource Exhaustion

Protect edge.

---

# 249. Rate limit.

---

# 250. Shed non-critical work.

---

# 251. Preserve protected control operations.

---

# 252. Do not disable auth/security checks for throughput.

---

# 253. Security Kill Switches

Recommended set:

```text
freeze-signing
freeze-release
freeze-deployment
disable-external-plugins
disable-external-dependency-fetch
disable-remote-cache-write
disable-scm-submit
disable-new-runner-enrollment
```

---

# 254. Kill Switch Scope

System/tenant/provider as appropriate.

---

# 255. Kill Switch Audit

Mandatory.

---

# 256. Revocation Epochs

Use epochs for fast logical invalidation.

---

# 257. PrincipalSecurityEpoch

Invalidates tokens/sessions.

---

# 258. RunnerTrustEpoch

Invalidates prior runner trust.

---

# 259. SigningTrustEpoch

Marks key trust changes.

---

# 260. PolicyEpoch

Emergency policy update.

---

# 261. Epoch Is Metadata

Not a replacement for certificate/token revocation.

---

# 262. Vulnerability Management

Forgeyard itself needs process.

---

# 263. SECURITY.md

Publish:

```text
contact
reporting process
supported versions
disclosure policy
```

---

# 264. Internal Vulnerability State

```rust
pub enum VulnerabilityState {
    Reported,
    Triaged,
    Confirmed,
    Fixing,
    Fixed,
    Released,
    Disclosed,
    Rejected,
}
```

---

# 265. Severity

Use established scoring where useful.

---

# 266. Exploitability

Separate from severity.

---

# 267. Patch SLA

Policy-driven by severity/exposure.

---

# 268. Coordinated Disclosure

Supported.

---

# 269. VEX

Use when dependency CVE not applicable.

---

# 270. Secure Development Lifecycle

Required activities:

```text
threat modeling
code review
dependency review
SAST
fuzzing
integration tests
release verification
```

---

# 271. Threat Model Review Trigger

When changing:

```text
trust boundary
new network endpoint
secret access
runner privilege
plugin permission
signing flow
deployment authority
```

---

# 272. Security Architecture Decision Record

High-risk changes require ADR/RFC.

---

# 273. Rust Unsafe Policy

Default:

```text
unsafe minimized
unsafe isolated
SAFETY comments required
```

---

# 274. Unsafe Audit

Central inventory.

---

# 275. `unsafe` Is Not Forbidden Absolutely

Platform/FFI/system code may need it.

---

# 276. Miri

For suitable crates.

---

# 277. Sanitizers

Native/FFI code.

---

# 278. Fuzzing

Priority targets:

```text
protocol decoders
archive parsers
SARIF/report parsers
Postcard envelopes
webhook parsers
registry metadata
config parser
```

---

# 279. Rust Benefit

Memory safety reduces classes of defects.

---

# 280. Rust Does Not Prevent

```text
logic bugs
authz bugs
SQL mistakes
SSRF
supply-chain compromise
misconfiguration
```

---

# 281. Security Test Matrix

```text
unit
property
fuzz
integration
tenant-isolation
authz-matrix
sandbox
protocol
incident drill
```

---

# 282. Authz Matrix Tests

Every protected action against:

```text
principal class
scope
tenant
role/permission
source trust
```

---

# 283. Tenant Isolation Tests

Attempt cross-tenant:

```text
API
SSE
CAS
RBE
search
cache
audit
device
plugin
```

---

# 284. SSRF Tests

Cloud metadata, loopback, private ranges, DNS rebinding.

---

# 285. CSRF Tests

Browser mutation endpoints.

---

# 286. Webhook Replay Tests

Duplicate/timestamp.

---

# 287. Sandbox Escape Tests

Known escape patterns.

---

# 288. Path Traversal Tests

Source/CAS/artifacts/archives.

---

# 289. Secret Leakage Tests

Search:

```text
logs
audit
events
CAS metadata
UI
notification
error envelopes
```

---

# 290. Security Chaos

Inject:

```text
cert expiry
secret provider outage
signing unavailable
DB unavailable
CAS corruption
stale policy
```

---

# 291. Expected Result

Protected operation fails safely.

---

# 292. Penetration Testing

Periodic external/internal assessment.

---

# 293. Scope

Edge/API/authz/tenant/sandbox/provider boundaries.

---

# 294. Security Assurance Profiles

```rust
pub enum SecurityAssuranceProfile {
    Standard,
    HostedHostile,
    HighAssurance,
    AirGapped,
}
```

---

# 295. Standard

Trusted team/internal CI.

---

# 296. HostedHostile

Untrusted tenant code.

Requires stronger isolation.

---

# 297. HighAssurance

Strict signing, WORM audit, curated dependencies, restricted plugins.

---

# 298. AirGapped

No external network dependencies.

---

# 299. Profile Is Minimum Baseline

Admin can tighten further.

---

# 300. Profile Cannot Weaken Hard Security Invariants

Critical.

---

# 301. HostedHostile Requirements

At least:

```text
VM-capable isolation
strict tenant cache separation
no host socket exposure
short-lived workload credentials
restricted egress
```

---

# 302. HighAssurance Requirements

Potential:

```text
HSM/KMS signing
two-person release approvals
curated dependencies
immutable audit export
verified backups
```

---

# 303. AirGapped Requirements

```text
offline dependency bundles
offline license/config bundles
local trust roots
no hidden external calls
```

---

# 304. Security Production Gates

Before public production:

```text
threat model reviewed
critical authz tests pass
tenant isolation tests pass
secret leakage tests pass
sandbox profile validated
signing separation validated
backup restore drill passed
incident playbooks exercised
```

---

# 305. Security Readiness State

```rust
pub enum SecurityReadiness {
    Development,
    InternalTesting,
    LimitedProduction,
    ProductionReady,
    HighAssuranceReady,
}
```

---

# 306. No Self-Certification

This is engineering readiness, not external compliance certification.

---

# 307. Monitoring Signals

Examples:

```text
auth failure spike
runner quarantine
secret access anomaly
cache equivocation
CAS corruption
plugin violation
signing freeze
policy bypass attempt
```

---

# 308. Metrics

```text
security_events_total
security_incidents_open
runner_quarantines_total
credential_revocations_total
sandbox_violations_total
integrity_failures_total
security_killswitch_active
```

---

# 309. Labels

Low-cardinality:

```text
event_category
severity
result
```

---

# 310. No Principal/Tenant IDs in metrics.

---

# 311. Tracing

Security-sensitive operations retain correlation IDs.

---

# 312. Logs

Structured/redacted.

---

# 313. SIEM

Part 28 audit/security export.

---

# 314. Security Dashboard

Dioxus:

```text
Security Overview
Open Incidents
Quarantined Runners
Trust/Certificates
Signing Status
Kill Switches
Critical Findings
```

---

# 315. Incident Detail

Shows:

```text
timeline
scope
containment
evidence
affected releases
affected identities
recovery actions
```

---

# 316. Restricted Access

Incident data is highly sensitive.

---

# 317. CLI

```text
forgeyard security status
forgeyard security incident list
forgeyard security incident show
forgeyard security quarantine runner
forgeyard security freeze signing
forgeyard security revoke token
forgeyard security verify trust
```

---

# 318. Dangerous CLI

Requires explicit confirmation/authz/MFA as policy.

---

# 319. API

Potential:

```text
GET  /v1/security/status
GET  /v1/security/incidents
POST /v1/security/incidents
POST /v1/security/runners/{id}/quarantine
POST /v1/security/signing/freeze
POST /v1/security/tokens/{id}/revoke
```

---

# 320. Permissions

```text
security.read
security.incident.manage
security.runner.quarantine
security.signing.freeze
security.credential.revoke
security.forensics.read
security.trust.manage
```

---

# 321. Break-Glass

Part 28.

---

# 322. Break-Glass Does Not Disable Audit

Critical.

---

# 323. Break-Glass Cannot Bypass Cryptographic Integrity

---

# 324. Break-Glass Expiry

Mandatory.

---

# 325. Incident Break-Glass

Can be used if IdP unavailable.

---

# 326. Post-Incident Review

Required.

---

# 327. Security Data Retention

Incident/audit evidence often retained longer.

---

# 328. Legal Hold

Part 28.

---

# 329. Privacy

Collect only necessary forensic data.

---

# 330. Backups

Encrypted.

---

# 331. Recovery Keys

Separate failure domain.

---

# 332. Root Trust

Offline backup.

---

# 333. Recovery Verification

After restore:

```text
verify DB
verify CAS
verify audit chain
verify trust
verify signing
reconcile
```

---

# 334. Clean-Room Recovery

For severe compromise:

```text
new hosts
new credentials
verified binaries
restored trusted data
```

---

# 335. Do Not Recover Onto Suspected Compromised Host

High-assurance rule.

---

# 336. Security Dependencies

Track critical libraries separately.

---

# 337. Update Cadence

Security-sensitive dependencies reviewed aggressively.

---

# 338. Build Toolchain

Pinned.

---

# 339. Compiler Trust

Toolchain provenance.

---

# 340. Reproducible Build

Independent reproduction increases supply-chain confidence.

---

# 341. Multi-Party Reproduction

High-assurance release option.

---

# 342. Binary Transparency

Future optional public release log.

---

# 343. Transparency Log

Could publish:

```text
ReleaseId
artifact digest
signature
provenance digest
```

---

# 344. Not Required Baseline

---

# 345. Attestation Verification

Consumers can verify offline.

---

# 346. Trust Root Distribution

Part of Forgeyard release artifacts.

---

# 347. Security Documentation

Maintain:

```text
SECURITY.md
threat model
trust boundary diagrams
incident playbooks
key rotation runbooks
```

---

# 348. Security Review Checklist

For new subsystem:

```text
new identity?
new secret?
new network edge?
new mutable external state?
new privileged operation?
new cross-tenant path?
new parser?
new file extraction?
```

---

# 349. Security Invariant Registry

Machine-readable high-level invariants can live in `architecture.ron`.

---

# 350. Example

```text
daemon_never_executes_untrusted_shell = true
release_never_rebuilds = true
runner_cannot_self_promote = true
```

---

# 351. Architecture Check

CI enforces dependency boundaries and selected security rules.

---

# 352. Security Regression Test

Any bug that caused incident gets permanent test when feasible.

---

# 353. Incident Drill Schedule

Examples:

```text
runner compromise
signing key compromise
DB restore
tenant isolation
```

---

# 354. Game Days

Production-like non-destructive drills.

---

# 355. Security Testkit

```text
crates/security/
└── forgeyard-security-testkit/
    ├── authz.rs
    ├── tenant.rs
    ├── secrets.rs
    ├── runner.rs
    ├── signing.rs
    ├── cas.rs
    ├── ssrf.rs
    ├── incident.rs
    └── assertions.rs
```

---

# 356. Unit Tests

Trust/epoch/state transitions.

---

# 357. Property Tests

Revoked identity never regains access from stale cache.

---

# 358. Tenant Tests

Cross-tenant matrix.

---

# 359. Runner Tests

Stale cert/session rejected.

---

# 360. Signing Tests

Unsigned/unapproved artifact cannot sign.

---

# 361. Secret Tests

No leak in logs/events/audit.

---

# 362. SSRF Tests

Private addresses blocked.

---

# 363. Cache Poisoning Tests

Untrusted producer cannot trusted-write.

---

# 364. Dependency Equivocation Tests

Quarantine.

---

# 365. Plugin Permission Tests

No hidden escalation.

---

# 366. DAST/Load Target Tests

No arbitrary third-party targets.

---

# 367. Config Tests

Project config cannot grant privilege.

---

# 368. Recovery Tests

Compromised-node clean-room recovery.

---

# 369. Key Rotation Tests

Overlap/revocation.

---

# 370. Incident Playbook Tests

Automation/manual steps simulated.

---

# 371. Fuzzing

Hostile parsers/protocols.

---

# 372. Chaos

Security service outages.

---

# 373. Pen-Test Findings

Tracked in Part 37 finding model where useful.

---

# 374. Implementation Phase 1 — Threat Model & Security Invariants

Document/encode boundaries.

---

# 375. Phase 2 — Edge/Authn/Authz Hardening

Public attack surface.

---

# 376. Phase 3 — Runner/Sandbox Hardening

Hostile code.

---

# 377. Phase 4 — Secrets/Signing Hardening

High-value assets.

---

# 378. Phase 5 — Tenant Isolation Verification

Hosted safety.

---

# 379. Phase 6 — Integrity/Poisoning Detection

CAS/cache/dependency.

---

# 380. Phase 7 — Security Events/Detection

Operational visibility.

---

# 381. Phase 8 — Incident Model/Kill Switches

Containment.

---

# 382. Phase 9 — Forensics/Playbooks

Recovery.

---

# 383. Phase 10 — Assurance Profiles

Deployment modes.

---

# 384. Phase 11 — Security Game Days/Pen Tests

Validation.

---

# 385. Phase 12 — High-Assurance Hardening

HSM/WORM/multi-party reproduction.

---

# 386. Acceptance Tests

1. Every major trust boundary is documented.
2. Network location alone never grants trust.
3. Runner self-reported capabilities cannot self-promote trust.
4. Human identity is not propagated as unrestricted workload authority.
5. Protected actions record immutable subject identities.
6. Tenant isolation is enforced at all storage/protocol surfaces.
7. CAS digest knowledge never implies authorization.
8. Build runners do not receive registry/provider/admin credentials.
9. Untrusted build code does not access Docker socket, SSH agent, cloud metadata, or host home by default.
10. Hostile multi-tenant workloads have VM-capable isolation path.
11. Shell execution is explicit rather than default string execution.
12. Filesystem materialization resists traversal/symlink escape.
13. SSRF defenses apply consistently to outbound URL features.
14. Redirect targets are revalidated.
15. Plugins cannot obtain DB/authz/scheduler authority.
16. RBE cannot bypass tenant/trust/priority constraints.
17. Secret values never enter metadata/CAS/events/audit/logs by design.
18. General runners cannot access production signing private keys.
19. Release signing binds exact artifact/evidence/policy identity.
20. Release never rebuilds after verification.
21. Feature flags/config cannot bypass security policy.
22. Security kill switches exist for signing/release/deploy/plugins/external fetch.
23. Compromise epochs can invalidate stale trust.
24. Runner compromise can quarantine its cache/output lineage.
25. Signing-key compromise has explicit freeze/rotation/investigation procedure.
26. Daemon compromise triggers credential rotation and clean-host rebuild.
27. Tenant breach has explicit containment/investigation process.
28. Dependency compromise can identify all affected releases by provenance/SBOM.
29. Security incidents preserve immutable evidence.
30. Audit remains enabled during break-glass.
31. Security scanner failure cannot produce false green.
32. Critical authz/tenant/sandbox/secret tests run continuously.
33. Rust unsafe use is isolated/reviewed rather than casually spread.
34. Recovery from severe compromise uses verified clean infrastructure.
35. Forgeyard dogfoods all security controls on its own release pipeline.

---

# 387. Production Readiness Gates

Do not call Forgeyard security production-ready until:

```text
threat model is reviewed
trust boundaries are documented
tenant isolation matrix passes
authz matrix passes
secret leakage tests pass
runner/sandbox profile is validated
signing worker separation is proven
dependency/cache poisoning defenses work
kill switches are tested
backup/clean-room recovery drill passes
incident playbooks are exercised
```

---

# 388. Architectural Invariants

1. no untrusted build becomes control-plane authority;
2. no runner self-promotes trust;
3. no plugin becomes policy/authz authority;
4. network location is never enough for trust;
5. every protected action binds immutable identities;
6. tenant isolation is universal;
7. digest identity is not access authorization;
8. privileged credentials are not exposed to build code;
9. secret values stay outside normal metadata/CAS/log/event paths;
10. general runners never hold production signing keys;
11. release signing binds exact verified bytes;
12. release does not rebuild;
13. sandbox strength is honest and explicit;
14. hostile tenants have stronger isolation path;
15. SSRF defenses are cross-cutting;
16. typed argv is preferred to shell;
17. config/feature flags do not bypass security;
18. cache/dependency equivocation is treated as security event;
19. stale/revoked identities are fenced;
20. compromise can quarantine trust metadata without rewriting immutable bytes;
21. security events are separate from ordinary logs;
22. incident response preserves evidence;
23. severe compromise uses clean-room recovery;
24. break-glass never disables audit;
25. security controls degrade availability before integrity;
26. Rust reduces memory-safety risk but does not remove logic/security risk;
27. security architecture changes trigger threat-model review;
28. assurance profiles can tighten but not weaken invariants;
29. incident drills are part of production readiness;
30. Forgeyard dogfoods its own security architecture.

---

# 389. Final Target Architecture

```text
                     Internet / SCM
                          │
                          ▼
                      Edge/API
                          │
                          ▼
                Authn / Authz / Policy
                          │
                          ▼
                    Control Plane
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
     Metadata            CAS          Secrets/Signing
         │                │                │
         └────────────────┼────────────────┘
                          ▼
                     Scheduler
                          │
                          ▼
                        Agent
                          │
                          ▼
                    Runner Host
                          │
                          ▼
                    Sandbox / VM
                          │
                          ▼
                  Untrusted Build Code
```

Security response:

```text
detect
  ↓
triage
  ↓
contain
  ↓
revoke/quarantine/freeze
  ↓
eradicate
  ↓
recover on verified state
  ↓
reconcile
  ↓
monitor
  ↓
postmortem
```

The key guarantee is:

> **Forgeyard is designed so a compromise is bounded by explicit trust scopes, identities, and revocable epochs rather than silently spreading across the CI/CD control plane. Untrusted code may execute, fail, or even compromise its immediate sandbox or runner, but it is never supposed to inherit the authority to modify policy, read unrelated tenant data, obtain production signing keys, or publish trusted releases.**

---

# 390. Extended Architecture Sequence

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
```
