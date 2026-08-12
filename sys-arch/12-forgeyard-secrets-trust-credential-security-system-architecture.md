# 12 — Forgeyard Secrets, Trust & Credential Security System Architecture

**Document type:** Core Security System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Secret references, secret providers, late resolution, runtime delivery, credential lifecycle, encryption boundaries, mTLS trust, certificates, workload credentials, runner trust, key handling, rotation, revocation, redaction, audit, break-glass interaction, and secure recovery  
**Architecture style:** References-not-values, late binding, least privilege, short-lived credentials, provider-neutral secret storage, explicit trust roots, revocation-first identity, and strict separation between secret material and normal Forgeyard metadata/CAS/logging  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on `11-forgeyard-policy-authorization-identity-system-architecture.md` and integrates with the Runner/Agent, Transport/QUIC, Sandbox/Executor, Events/Reconciliation, CAS, Release, Deployment, and Supply-Chain architectures. It does not redefine identity/RBAC policy or artifact-signing provenance; it defines how sensitive material and cryptographic trust are managed and delivered safely.

---

# 1. Purpose

Forgeyard must handle credentials and sensitive material required by CI/CD:

```text
repository credentials
package registry credentials
cloud deployment credentials
OIDC client secrets
database credentials
runner certificates
API tokens
code-signing credentials
notarization credentials
Android keystores
Apple signing material
SSH keys
webhook secrets
encryption keys
short-lived workload credentials
```

These values must never become ordinary pipeline configuration or general metadata.

The central rule is:

> **Forgeyard stores and moves secret references whenever possible, and resolves secret values only at the narrowest possible execution boundary, for the shortest possible lifetime.**

A second rule is:

> **Secret possession is not equivalent to secret-use authorization. A workload may be allowed to use a secret without being allowed to view, export, administer, or persist its value.**

A third rule is:

> **Trust is explicit: trust roots, certificates, signing identities, runner enrollment, workload credentials, and provider identities are all versioned, scoped, revocable, and auditable.**

---

# 2. Architectural Position

```text
                  Pipeline / Policy
                       │
                       ▼
                    SecretRef
                       │
                       ▼
               Authorization Check
                       │
                       ▼
                Secret Provider API
                       │
       ┌───────────────┼────────────────┐
       ▼               ▼                ▼
   Local Secure     Vault/KMS       Cloud Secret
      Store            │              Manager
       │               │                │
       └───────────────┼────────────────┘
                       ▼
                 Short-Lived Value
                       │
                       ▼
              Agent / Restricted Worker
                       │
                       ▼
                 Sandbox Injection
                       │
                       ▼
                    Workload
                       │
                       ▼
                   Zeroize
```

Trust plane:

```text
Trust Root
   ↓
Certificate / Credential
   ↓
Principal / Runner / Workload Identity
   ↓
Policy + Scope
   ↓
Authorized Capability
```

---

# 3. Goals

The subsystem MUST:

1. define stable typed `SecretRef`;
2. keep secret values out of pipeline IR;
3. keep secret values out of normal metadata;
4. keep secret values out of CAS;
5. resolve secrets late;
6. support standalone/local providers;
7. support external secret managers;
8. support cloud KMS/secret stores;
9. support short-lived workload credentials;
10. support rotation;
11. support revocation;
12. support secret versioning;
13. support secret-use authorization;
14. support secret administration separately;
15. support secret redaction;
16. support lease/job binding;
17. support runner trust;
18. support mTLS certificate trust;
19. support certificate enrollment/renewal;
20. support provider credentials;
21. support code-signing boundaries;
22. support break-glass with stronger controls;
23. support audit;
24. support secret expiration;
25. support reconciliation;
26. support zeroization;
27. support secure local persistence where unavoidable;
28. support air-gapped mode;
29. support recovery without plaintext exports by default;
30. remain provider-neutral.

---

# 4. Non-Goals

This subsystem is not:

```text
general password manager UI
user identity provider
RBAC engine
artifact signing/provenance engine
public certificate authority product
```

It provides Forgeyard's internal secret and trust capabilities.

---

# 5. Workspace Structure

```text
crates/secrets/
├── forgeyard-secrets/
├── forgeyard-secrets-model/
├── forgeyard-secrets-provider/
├── forgeyard-secrets-local/
├── forgeyard-secrets-env/
├── forgeyard-secrets-file/
├── forgeyard-secrets-vault/
├── forgeyard-secrets-aws/
├── forgeyard-secrets-gcp/
├── forgeyard-secrets-azure/
├── forgeyard-secrets-kubernetes/
├── forgeyard-secrets-delivery/
├── forgeyard-secrets-redaction/
├── forgeyard-secrets-rotation/
├── forgeyard-secrets-audit/
├── forgeyard-secrets-health/
└── forgeyard-secrets-testkit/
```

Trust:

```text
crates/trust/
├── forgeyard-trust/
├── forgeyard-trust-model/
├── forgeyard-trust-root/
├── forgeyard-trust-ca/
├── forgeyard-trust-certificate/
├── forgeyard-trust-enrollment/
├── forgeyard-trust-mtls/
├── forgeyard-trust-workload/
├── forgeyard-trust-runner/
├── forgeyard-trust-signing/
├── forgeyard-trust-revocation/
├── forgeyard-trust-attestation/
├── forgeyard-trust-health/
└── forgeyard-trust-testkit/
```

---

# 6. SecretRef

```rust
pub struct SecretRef {
    pub provider: SecretProviderId,
    pub path: SecretPath,
    pub version: SecretVersionSelector,
}
```

No raw value.

---

# 7. SecretId

Forgeyard may also assign internal semantic identity:

```rust
pub struct SecretId(Ulid);
```

Metadata can map:

```text
SecretId
  ↓
provider/path/version policy
```

---

# 8. Secret Path

Validated opaque path.

Do not let arbitrary provider syntax leak across all domain crates.

---

# 9. Secret Version Selector

```rust
pub enum SecretVersionSelector {
    LatestAllowed,
    Exact(SecretVersionId),
}
```

---

# 10. Strict Release Rule

Release/signing/deployment-critical workflows should prefer:

```text
Exact secret version
```

or exact credential generation context when reproducibility/audit requires it.

---

# 11. Secret Metadata

Safe metadata may include:

```text
SecretId
provider
display name
scope
created_at
rotated_at
expiry
classification
```

Never value.

---

# 12. Secret Value Type

```rust
pub struct SecretValue(Zeroizing<Vec<u8>>);
```

---

# 13. Debug

`Debug` must redact.

Example:

```text
SecretValue([REDACTED])
```

---

# 14. Display

Do not implement normal `Display` for secret value.

---

# 15. Clone

Avoid or tightly control cloning of secret value.

---

# 16. Serialization

Do not implement general-purpose `Serialize` for raw secret value unless a very narrow encrypted wire type requires it.

---

# 17. Secret Provider Trait

```rust
#[async_trait]
pub trait SecretProvider: Send + Sync {
    async fn resolve(
        &self,
        request: ResolveSecretRequest,
    ) -> Result<ResolvedSecret, SecretError>;
}
```

---

# 18. Resolve Request

```rust
pub struct ResolveSecretRequest {
    pub secret: SecretRef,
    pub principal: PrincipalId,
    pub workload: Option<WorkloadIdentityBinding>,
    pub purpose: SecretPurpose,
    pub context: SecretAccessContext,
}
```

---

# 19. Secret Purpose

```rust
pub enum SecretPurpose {
    SourceFetch,
    RegistryRead,
    RegistryPublish,
    Deployment,
    Signing,
    Notarization,
    Database,
    Webhook,
    InternalService,
    Custom(SecretPurposeId),
}
```

---

# 20. Resolved Secret

```rust
pub struct ResolvedSecret {
    pub value: SecretValue,
    pub version: SecretVersionId,
    pub expires_at: Option<Timestamp>,
    pub lease: SecretLeaseId,
}
```

---

# 21. Secret Lease

```rust
pub struct SecretLeaseId(Ulid);
```

Represents one authorized resolution/use instance.

---

# 22. Secret Lease Binding

Bind to:

```text
principal/workload
job/attempt if applicable
purpose
secret version
expiry
```

---

# 23. Secret Use vs Read

Permissions:

```text
secret.use
secret.read
secret.admin
```

Recommended:

```text
normal workloads only require secret.use
```

---

# 24. Secret Read

Human plaintext viewing should be rare and optionally disabled entirely for some secret classes.

---

# 25. Non-Exportable Secrets

```rust
pub enum SecretExportPolicy {
    Exportable,
    UseOnly,
    ProviderOperationOnly,
}
```

---

# 26. ProviderOperationOnly

Best for:

```text
KMS signing key
HSM key
cloud workload identity
```

Forgeyard never receives raw private key material.

---

# 27. Secret Provider Categories

```text
LocalEncrypted
Environment
File
HashiCorpVault
AWSSecretsManager
AWSKms
GCPSecretManager
GCPKms
AzureKeyVault
KubernetesSecret
ExternalCustom
```

---

# 28. Environment Provider

Development/bootstrap only.

Not recommended for production sensitive values.

---

# 29. File Provider

Useful:

```text
air-gapped
local standalone
system-managed credential files
```

Must validate permissions.

---

# 30. Local Secure Provider

Mode 1 needs zero-external-service secret storage.

---

# 31. Local Secret Store Requirements

```text
encrypted at rest
master-key protection
atomic writes
versioning
restricted permissions
backup semantics
```

---

# 32. Local Master Key

Do not store encrypted secret DB and master key together unprotected.

---

# 33. Local Master Key Sources

Potential:

```text
OS keyring
TPM
user passphrase-derived wrapping key
hardware-backed keystore
```

---

# 34. Linux Local Key

Potential integration:

```text
kernel keyring
Secret Service
TPM2
```

Exact adapter chosen later.

---

# 35. Windows Local Key

Potential:

```text
DPAPI
CNG/TPM-backed key
```

---

# 36. macOS Local Key

Potential:

```text
Keychain
Secure Enclave where appropriate
```

---

# 37. Standalone Fallback

If secure OS keystore unavailable:

```text
operator-supplied passphrase
+
memory-hard KDF
```

with clear warning.

---

# 38. No Home-Grown Cryptography

Use vetted AEAD/KDF primitives and audited libraries.

---

# 39. Envelope Encryption

For local metadata-secret storage:

```text
data encryption key (DEK)
  ↓
encrypt secret
  ↓
key encryption key (KEK) wraps DEK
```

---

# 40. KEK

Protected by:

```text
OS keystore
KMS
HSM
```

---

# 41. Rotation

KEK rotation can rewrap DEKs without re-encrypting every secret payload where design permits.

---

# 42. Secret Ciphertext Record

```rust
pub struct EncryptedSecretRecord {
    pub version: EncryptionSchemaVersion,
    pub ciphertext: Vec<u8>,
    pub nonce: Nonce,
    pub wrapped_dek: WrappedKey,
    pub aad: SecretAad,
}
```

---

# 43. AAD

Bind ciphertext to:

```text
SecretId
version
provider/store
tenant
```

to prevent record swapping.

---

# 44. Secret Encryption Schema Version

Explicitly versioned.

---

# 45. External Secret Manager

Preferred distributed enterprise mode where already available.

---

# 46. Provider Credentials

Forgeyard needs credentials to access provider.

Use:

```text
workload identity
instance identity
short-lived cloud role
```

instead of static secrets where possible.

---

# 47. Credential Bootstrap Problem

Root credentials still exist somewhere.

Minimize and hardware/provider protect.

---

# 48. Workload Federation

Preferred for:

```text
AWS role assumption
GCP workload identity federation
Azure federated identity
```

where available.

---

# 49. Dynamic Credentials

Best pattern:

```text
job identity
  ↓
authorized federation
  ↓
short-lived cloud credential
```

instead of storing static access key.

---

# 50. Dynamic Secret Provider

```rust
pub trait DynamicCredentialProvider {
    async fn issue(
        &self,
        request: CredentialIssueRequest,
    ) -> Result<IssuedCredential, CredentialError>;
}
```

---

# 51. Issued Credential

```rust
pub struct IssuedCredential {
    pub material: SecretValue,
    pub expires_at: Timestamp,
    pub scope: CredentialScope,
}
```

---

# 52. Credential TTL

Short as practical.

---

# 53. Runner Delivery

Secrets reach only the runner currently holding authoritative attempt lease.

---

# 54. Secret Delivery Authorization

Validate:

```text
JobId
AttemptId
LeaseId
AgentSessionId
SecretRef
workload identity
policy
source trust
```

---

# 55. Delivery Wire

```rust
pub struct SecretDeliveryEnvelope {
    pub secret_lease: SecretLeaseId,
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub expires_at: Timestamp,
    pub payload: ProtectedSecretPayload,
}
```

---

# 56. ProtectedSecretPayload

Even inside TLS, use narrow type with:

```text
redacted Debug
no logging
bounded size
zeroizing decode buffer
```

---

# 57. Double Encryption

Application-level envelope encryption over mTLS is optional.

Use if threat model requires protection from transport terminator/proxy or for store-and-forward.

---

# 58. Default

mTLS + process isolation + no persistence is sufficient for native direct agent transport in many deployments.

---

# 59. Secret Delivery Channel

Can use dedicated logical control stream/class.

---

# 60. No Secret in JobSpec CAS

JobSpec contains:

```text
SecretRef
```

not values.

---

# 61. No Secret in Pipeline IR

Same.

---

# 62. No Secret in Action Cache Key

Secret values never become shared cache key material.

---

# 63. Secret-Dependent Build

If secret materially changes output:

```text
cache disabled
isolated cache
or
non-secret deterministic tokenization scheme
```

Policy chooses.

---

# 64. Secret Injection Methods

```rust
pub enum SecretInjection {
    Environment,
    File,
    Stdin,
    ProviderProxy,
    PlatformCredential,
}
```

---

# 65. Environment Injection

Convenient but leaks to:

```text
child process
crash dumps
debuggers
process inspection
```

Use only when tool requires.

---

# 66. File Injection

Better for many tools.

Mount private tmpfs/read-only file.

---

# 67. Stdin Injection

Good for one-time command if supported.

---

# 68. Provider Proxy

Best when workload does not need raw credential.

Example:

```text
credential helper/proxy signs request
```

---

# 69. Platform Credential

Example:

```text
Windows credential handle
macOS Keychain identity
HSM handle
```

---

# 70. Secret Lifetime

Lifecycle:

```text
authorize
  ↓
resolve/issue
  ↓
deliver
  ↓
inject
  ↓
use
  ↓
remove
  ↓
zeroize
```

---

# 71. Secret TTL Enforcement

Agent tracks expiry.

Expired credential removed even if job continues.

---

# 72. Refresh

Long jobs may refresh short-lived credential if policy/provider supports.

---

# 73. Refresh Authorization

Re-check current lease/workload authority.

---

# 74. Revocation During Job

If critical secret revoked:

```text
stop refreshing
possibly cancel job
```

policy decides.

---

# 75. Secret Redaction

```text
forgeyard-secrets-redaction
```

Protect:

```text
logs
diagnostics
errors
traces
UI
```

---

# 76. Exact-Value Redaction

Agent can register delivered secret values with local redactor.

---

# 77. Encoding Variants

Potentially redact:

```text
raw
URL-encoded
base64
```

only within reasonable bounds.

---

# 78. Redaction Is Defense in Depth

Never assume redaction is perfect.

Main defense:

```text
do not print secrets
```

---

# 79. Derived Secrets

Tools can transform values, so generic redaction cannot guarantee removal.

---

# 80. Secret Log Policy

Treat job logs as sensitive.

---

# 81. Secret Size Limits

Bound secret payload sizes.

Large binary signing material should usually be provider-managed rather than injected.

---

# 82. Secret File Permission

Unix example:

```text
0600
```

owned by sandbox execution identity.

---

# 83. Windows ACL

Execution identity only.

---

# 84. Secret File Cleanup

Remove before workspace retention/debug.

---

# 85. Memory Zeroization

Use zeroizing types for:

```text
raw values
private key bytes
tokens
passwords
```

---

# 86. Zeroization Limits

Rust/compiler/OS may make absolute memory erasure guarantees difficult.

Use best-effort secure primitives and minimize copies.

---

# 87. Swap

High-assurance workers may require:

```text
mlock
encrypted swap
no swap
```

where practical.

---

# 88. Core Dumps

Disable for secret-handling/signing workers.

---

# 89. Crash Reports

Must not include secret buffers.

---

# 90. Trust Root

```rust
pub struct TrustRootId(Ulid);
```

Represents trusted CA/public key/root configuration.

---

# 91. Trust Domains

Separate trust roots may exist for:

```text
agent mTLS
signing verification
provider webhook signing
workload identity
artifact signatures
```

---

# 92. Do Not Reuse One Root Everywhere

Compromise blast radius.

---

# 93. Internal CA

Forgeyard may run or integrate with CA for runner certificates.

---

# 94. CA Architecture

```text
offline/root CA
  ↓
intermediate CA
  ↓
runner/service certs
```

for production.

---

# 95. Root Private Key

Ideally offline/HSM-managed.

---

# 96. Intermediate Key

Online restricted service/HSM where required.

---

# 97. Standalone CA

Mode 1 may use self-contained local CA.

---

# 98. Certificate Identity

Certificate SAN/extension maps to:

```text
RunnerId
service principal
role/purpose
```

---

# 99. Certificate TTL

Short enough to limit compromise.

---

# 100. Automatic Renewal

Agent renews before expiry.

---

# 101. Renewal Authorization

Current credential + runner enrollment state.

---

# 102. Revocation

Mechanisms:

```text
server-side principal/credential denylist
certificate serial revocation
short cert lifetime
```

---

# 103. Immediate Revocation

Control plane state is authority.

Even valid cert can be rejected if RunnerId revoked.

---

# 104. CRL/OCSP

Optional depending CA architecture.

Short-lived cert + server-side state can simplify internal trust.

---

# 105. Enrollment Token

```rust
pub struct EnrollmentTokenId(Ulid);
```

One-time or limited-use.

---

# 106. Enrollment Token Properties

```text
short TTL
runner/pool scope
single use
hashed at rest
audited
```

---

# 107. Enrollment Flow

```text
admin creates token
  ↓
runner presents over authenticated server TLS
  ↓
token consumed
  ↓
RunnerId bound
  ↓
client cert issued
```

---

# 108. Re-enrollment

Explicit operator flow.

---

# 109. Runner Trust Assignment

During/after enrollment:

```text
General
Trusted
SigningRestricted
Confidential
```

authorized admin only.

---

# 110. Device Agent Trust

Separate agent purpose/certificate extension.

---

# 111. Signing Worker Trust

Separate enrollment path.

---

# 112. mTLS Rotation

Daemon trusts:

```text
current CA
+
next CA during rotation window
```

---

# 113. Trust Root Rotation

Phases:

```text
introduce new root
issue dual/new certs
accept both
migrate clients
remove old root
```

---

# 114. Root Rotation Event

Audited.

---

# 115. Certificate Pinning

Generally pin CA/identity policy rather than individual cert to enable rotation.

---

# 116. Daemon Server Certificate

Managed via secret/trust provider.

---

# 117. Public API TLS

Can use separate public PKI.

---

# 118. Internal vs Public PKI

Prefer separation.

---

# 119. Webhook Secrets

SecretRef scoped to provider/repository.

---

# 120. Webhook Rotation

Support dual active secrets during provider transition if provider supports.

---

# 121. Repository Credentials

Prefer:

```text
provider app installation token
short-lived OAuth token
deploy key scoped read-only
```

over user PAT.

---

# 122. Git SSH Credentials

Use per-project/service key if necessary.

---

# 123. SSH Host Trust

Manage `known_hosts`/host CA separately from client private key.

---

# 124. Package Registry Credentials

Scope:

```text
read
publish
package namespace
```

---

# 125. Publish Credentials

Only release/publish jobs.

Never proposal builds.

---

# 126. Deployment Credentials

Prefer workload federation.

---

# 127. Database Credentials

Daemon DB credential comes from secret provider.

Agent never receives.

---

# 128. OIDC Client Secret

Daemon-only.

---

# 129. CAS Cloud Credentials

Prefer instance/workload role or scoped temporary credentials.

---

# 130. Direct-to-Object-Store Credential

Use presigned/scoped token.

Runner should not receive broad bucket admin key.

---

# 131. Signing Keys

Highest-sensitivity class.

---

# 132. Signing Architecture Principle

Private signing key should preferably remain:

```text
HSM/KMS/OS signing store
```

and Forgeyard sends digest/sign request.

---

# 133. Signing Key Ref

```rust
pub struct SigningKeyRef {
    pub provider: SigningProviderId,
    pub key: SigningKeyId,
    pub version: SigningKeyVersion,
}
```

---

# 134. No Raw Key CAS

Never.

---

# 135. No Raw Key General Agent

General runner must not receive production signing private key.

---

# 136. Signing Worker

Restricted process/application.

Accepts typed sign requests only.

---

# 137. Signing Request

```rust
pub struct SigningRequest {
    pub request_id: SigningRequestId,
    pub subject: CasObjectRef,
    pub key: SigningKeyRef,
    pub policy_proof: SigningPolicyProof,
    pub requester: PrincipalId,
}
```

---

# 138. Policy Proof

Demonstrates:

```text
artifact approved
subject digest exact
requester authorized
required checks satisfied
```

---

# 139. Signing Output

```text
signature/attestation
```

plus signed artifact if format mutates bytes.

---

# 140. Signing Key Rotation

Old signatures remain verifiable.

Verification trust retains historical public keys.

---

# 141. Signing Key Revocation

Future signatures denied.

Historical verification policy marks revoked-at-time semantics explicitly.

---

# 142. Verification Time Semantics

Need distinguish:

```text
key valid when signed
key revoked later
key compromised retroactively
```

Supply-chain architecture handles final interpretation.

---

# 143. Android Keystore

Release Android signing should use:

```text
restricted signing worker/provider
```

not ordinary build runner if production-grade.

---

# 144. Apple Signing

Same principle.

Certificates/profiles/private keys handled by restricted Apple signing environment.

---

# 145. Notarization Credential

Restricted release workflow.

---

# 146. Windows Code Signing

Prefer HSM/KMS-backed certificate key where possible.

---

# 147. Secret Classification

```rust
pub enum SecretClass {
    General,
    Deployment,
    RegistryPublish,
    Infrastructure,
    Identity,
    Signing,
    RootOfTrust,
}
```

---

# 148. Classification Effects

Controls:

```text
who may administer
where resolved
which runner class
audit level
exportability
rotation policy
```

---

# 149. RootOfTrust

Most restricted.

---

# 150. Secret Scope

```rust
pub enum SecretScope {
    System,
    Tenant(TenantId),
    Organization(OrganizationId),
    Project(ProjectId),
    Repository(RepositoryId),
    Environment(EnvironmentId),
}
```

---

# 151. Environment Scope

Production secrets separate from staging.

---

# 152. Source Trust Restriction

```text
Fork
ExternalContribution
```

cannot access privileged secrets by default.

---

# 153. Branch Name Is Not Enough

Use exact:

```text
SourceSnapshotId
proposal trust
protected target state
```

---

# 154. Secret Policy Input

```rust
pub struct SecretPolicyInput {
    pub secret: SecretId,
    pub principal: PrincipalId,
    pub run: Option<RunId>,
    pub job: Option<JobId>,
    pub attempt: Option<JobAttemptId>,
    pub source: Option<SourceSnapshotId>,
    pub source_trust: Option<SourceTrust>,
    pub purpose: SecretPurpose,
}
```

---

# 155. Secret Access Decision

Policy + permission.

---

# 156. Human Plaintext Reveal

Separate explicit action:

```text
secret.read
```

Potential MFA/step-up.

---

# 157. Clipboard UI

Avoid exposing secret by default.

---

# 158. Secret Creation

UI/CLI sends new value over TLS to daemon/provider.

Do not echo back.

---

# 159. Secret Update

Creates new version.

---

# 160. Versioning

```rust
pub struct SecretVersionId(Ulid);
```

---

# 161. Old Version Retention

Provider/policy controls.

---

# 162. Rollback

Can reactivate prior version if safe.

---

# 163. Rotation Policy

```rust
pub struct SecretRotationPolicy {
    pub interval: Option<Duration>,
    pub before_expiry: Option<Duration>,
    pub automatic: bool,
}
```

---

# 164. Automatic Rotation

Only if provider supports safe generation/update.

---

# 165. Rotation State

```rust
pub enum RotationState {
    Scheduled,
    Generating,
    Activating,
    Verifying,
    Completed,
    Failed,
}
```

---

# 166. Dual-Version Window

Needed for some integrations.

---

# 167. Activation

New consumers use current version.

Existing jobs may retain their leased version if still valid.

---

# 168. Forced Rotation

Compromise response invalidates old immediately.

---

# 169. Rotation Reconciliation

Check provider/current metadata consistency.

---

# 170. Credential Expiry Reconciliation

Alert before expiry.

---

# 171. Certificate Expiry Reconciliation

Same.

---

# 172. Secret Health

Do not "health check" by revealing value.

Check:

```text
provider reachable
metadata readable
credential issue succeeds
key operation succeeds
```

---

# 173. Signing Key Health

Perform non-sensitive sign/verify probe where appropriate.

---

# 174. Audit

Every sensitive operation records:

```text
secret created
version added
secret used
secret read/revealed
secret rotated
secret revoked
policy changed
runner certificate issued
trust root changed
signing request
```

---

# 175. Secret Use Audit

Record reference, purpose, workload, result.

Never value.

---

# 176. High-Volume Use

May aggregate low-risk usage, but production secret/signing use should remain attributable.

---

# 177. Audit Actor

Human/service/workload/system typed actor.

---

# 178. Break-Glass Secret Use

Requires:

```text
break-glass grant
secret permission
reason
strong auth
audit
```

---

# 179. Break-Glass Read

Even stronger, optionally disabled.

---

# 180. Emergency Root Credential

If one exists, store offline/out-of-band.

Do not make it normal Forgeyard secret.

---

# 181. Recovery

Secret metadata backup does not imply plaintext secret backup.

---

# 182. External Provider Backup

Use provider's replication/recovery.

---

# 183. Local Secret Store Backup

Backup encrypted ciphertext + metadata.

---

# 184. Master Key Backup

Separate protected procedure.

---

# 185. Restore

```text
restore ciphertext
  ↓
restore/unlock KEK
  ↓
verify metadata/AAD
  ↓
test resolve
```

---

# 186. No Plaintext Export Default

Backup/export never emits plaintext unless explicit dangerous admin operation.

---

# 187. Air-Gapped Mode

Use:

```text
local encrypted provider
file/provider-backed HSM
offline CA
```

---

# 188. Air-Gap Secret Bundle

If required:

```text
encrypted to target trust root/operator key
```

not normal CAS bundle.

---

# 189. CAS Separation

Secret bundles use dedicated secure transfer path.

---

# 190. Secret Migration

Provider A -> Provider B:

```text
read authorized version
  ↓
write new provider
  ↓
verify
  ↓
update SecretRef binding
  ↓
revoke old
```

Audited.

---

# 191. Non-Exportable Migration

Requires provider-native key migration/rotation, not plaintext extraction.

---

# 192. Trust Attestation

Optional high-assurance executor trust.

---

# 193. Attestation Inputs

```text
hardware identity
VM measurement
boot measurement
executor image
```

---

# 194. Attestation Decision

Trust subsystem produces:

```rust
pub struct AttestationDecision {
    pub trusted: bool,
    pub measurement: AttestationMeasurement,
    pub policy_digest: PolicyDigest,
}
```

---

# 195. Secret Release by Attestation

High-security secret may require successful attestation before resolution/delivery.

---

# 196. Confidential Workload

```text
attested VM
  ↓
workload identity
  ↓
secret release
```

---

# 197. Attestation Freshness

Bounded TTL.

---

# 198. Attestation Is Not General Requirement

Optional for high-assurance deployments.

---

# 199. Trust Store

Stores:

```text
trusted roots
public keys
certificate metadata
revocations
attestation policies
```

No unnecessary private keys.

---

# 200. Public Key Retention

Retain historical verification keys as policy requires.

---

# 201. Trust Root Metadata

```rust
pub struct TrustRoot {
    pub id: TrustRootId,
    pub purpose: TrustPurpose,
    pub public_material: PublicTrustMaterial,
    pub state: TrustRootState,
}
```

---

# 202. Trust Purpose

```rust
pub enum TrustPurpose {
    RunnerMtls,
    InternalServiceMtls,
    ArtifactVerification,
    SigningVerification,
    ProviderWebhook,
    WorkloadAttestation,
}
```

---

# 203. Trust Root State

```rust
pub enum TrustRootState {
    Pending,
    Active,
    Retiring,
    Revoked,
}
```

---

# 204. Root Rotation

`Pending -> Active -> Retiring`.

---

# 205. Revocation Event

Immediate.

---

# 206. Trust Decision Cache

Short-lived.

Include root/policy epoch.

---

# 207. Trust Epoch

```rust
pub struct TrustEpoch(u64);
```

Increment on root/revocation changes.

---

# 208. Transport Integration

mTLS verifier consumes current trust store.

---

# 209. Session Revocation

Active QUIC session fenced/closed when runner credential revoked.

---

# 210. Runner Certificate Renewal

Agent proves existing identity or re-enrolls.

---

# 211. Key Algorithm Agility

Represent algorithm explicitly.

---

# 212. Cryptographic Algorithm Model

```rust
pub enum SignatureAlgorithm {
    Ed25519,
    EcdsaP256Sha256,
    RsaPssSha256,
    ProviderNative(SignatureAlgorithmId),
}
```

Actual allowed algorithms depend on purpose/interoperability.

---

# 213. No Single Algorithm Everywhere

Different ecosystems/signing standards require different algorithms.

---

# 214. Hash Algorithm

Use:

```text
BLAKE3 internally
SHA-256 where interoperability/signature format requires
```

---

# 215. Key Generation

Prefer provider/HSM generation for non-exportable keys.

---

# 216. Private Key Import

High-risk, audited, potentially disabled.

---

# 217. Public Key Export

Generally safe according to policy.

---

# 218. Certificate Request

CSR generated by key holder where possible.

---

# 219. Private Key Never Crosses CA

Good PKI practice.

---

# 220. TLS Private Key

Daemon/server should use secret/provider handle.

---

# 221. HSM/KMS Abstraction

```rust
#[async_trait]
pub trait KeyOperationProvider {
    async fn sign(
        &self,
        request: KeySignRequest,
    ) -> Result<KeySignature, KeyOperationError>;
}
```

---

# 222. Key Operation Purpose Binding

Prevent using TLS key for artifact signing.

---

# 223. Key Usage

```rust
pub enum KeyUsage {
    TlsServer,
    TlsClient,
    ArtifactSigning,
    PackageSigning,
    CertificateSigning,
    Attestation,
}
```

---

# 224. Key Policy

Provider verifies expected usage.

---

# 225. Secret Event Model

Examples:

```text
SecretCreated
SecretVersionActivated
SecretRotated
SecretRevoked
SecretAccessDenied
CredentialIssued
CredentialExpired
```

---

# 226. Trust Event Model

```text
RunnerEnrolled
CertificateIssued
CertificateRenewed
CertificateRevoked
TrustRootActivated
TrustRootRetired
```

---

# 227. Events Exclude Values

Always.

---

# 228. Reconciliation

Secret reconciler checks:

```text
metadata/provider version drift
expired secret
rotation overdue
provider unreachable
```

---

# 229. Trust Reconciler

Checks:

```text
cert expiry
revoked active session
root rotation progress
runner enrollment inconsistency
```

---

# 230. Signing Reconciler

Checks ambiguous provider signing operation if relevant.

---

# 231. Dynamic Credential Reconciler

Usually credentials expire naturally.

No long-lived storage.

---

# 232. Secret Provider Failure

Workload preparation fails safely.

---

# 233. Provider Outage

Do not substitute stale secret silently unless policy explicitly allows cached lease.

---

# 234. Secret Cache

Default:

```text
no long-lived plaintext cache
```

---

# 235. Short In-Memory Cache

May cache within one job/lease for TTL.

---

# 236. Cached Secret Binding

Must include:

```text
SecretVersionId
workload/lease
expiry
```

---

# 237. Cache Eviction

Zeroize.

---

# 238. No Disk Cache

Plaintext secret disk cache forbidden.

---

# 239. Error Model

```rust
pub enum SecretError {
    NotFound,
    AccessDenied,
    VersionUnavailable,
    Expired,
    Revoked,
    ProviderUnavailable,
    ProviderRejected,
    DeliveryFailed,
    InvalidScope,
    Internal,
}
```

---

# 240. Trust Error Model

```rust
pub enum TrustError {
    UnknownIdentity,
    RevokedIdentity,
    ExpiredCertificate,
    UntrustedIssuer,
    InvalidAttestation,
    EnrollmentDenied,
    RotationRequired,
    Internal,
}
```

---

# 241. Retry

Provider unavailable:

```text
bounded retry
```

Access denied/revoked:

```text
no retry until state changes
```

---

# 242. Secret Delivery Failure

Do not log payload.

---

# 243. Timing Side Channels

Exact provider existence may be sensitive.

Public errors should avoid revealing unauthorized secret names.

---

# 244. Secret Enumeration

Users only list secrets they have permission to see metadata for.

---

# 245. Metadata Visibility

Some users can know secret exists without read/use.

Separate permission if needed.

---

# 246. UI Secret Value

Default UI shows:

```text
name
scope
provider
version
last rotation
```

not value.

---

# 247. Secret Reveal UI

Optional high-risk action.

---

# 248. Secret Creation UI

Value input write-only.

---

# 249. Clipboard Protection

Best effort only.

Warn if human plaintext reveal is enabled.

---

# 250. CLI

```text
forgeyard secret list
forgeyard secret show
forgeyard secret create
forgeyard secret update
forgeyard secret rotate
forgeyard secret revoke
forgeyard secret test
forgeyard secret migrate

forgeyard trust roots
forgeyard trust certs
forgeyard trust enroll
forgeyard trust revoke
forgeyard trust rotate
forgeyard trust doctor
```

---

# 251. `secret show`

Never prints plaintext by default.

---

# 252. Explicit Reveal

If supported:

```text
forgeyard secret reveal --dangerous
```

requires:

```text
secret.read
step-up
audit
```

---

# 253. Shell History

CLI should discourage plaintext as command argument.

Use stdin/interactive prompt.

---

# 254. Secret Creation CLI

Read from:

```text
stdin
file descriptor
prompt
```

not positional arg.

---

# 255. Environment Import

Development helper:

```text
forgeyard secret create --from-env NAME
```

but value never printed.

---

# 256. Provider Doctor

Check connectivity/configuration without dumping secret.

---

# 257. Trust Doctor

Check:

```text
root state
certificate expiry
renewal path
revocation propagation
```

---

# 258. Metrics

Secret metrics:

```text
secret_resolve_requests
secret_resolve_denied
secret_provider_errors
secret_rotation_due
secret_rotation_failures
credential_issue_count
credential_refresh_failures
```

---

# 259. Trust Metrics

```text
runner_cert_expiring
cert_renew_failures
trust_revocations
enrollment_failures
attestation_failures
```

---

# 260. Metrics Privacy

Never label with secret path/value.

Use provider/class/scope type.

---

# 261. Tracing

Spans:

```text
secret.authorize
secret.resolve
secret.deliver
secret.rotate
trust.enroll
trust.verify
trust.renew
trust.revoke
key.sign
```

---

# 262. Trace Redaction

Do not attach values/tokens.

---

# 263. Health

Secrets:

```text
provider reachable
KMS operation works
local keystore unlocked
```

Trust:

```text
root valid
server cert valid
issuer available
```

---

# 264. Production Fail-Closed

If secret/trust decision uncertain:

```text
deny privileged operation
```

---

# 265. Read-Only Degraded Mode

Existing non-secret reads/builds can continue.

---

# 266. Bootstrap Secret

Initial DB/provider credentials may need environment/file bootstrap.

---

# 267. Bootstrap Migration

After startup, move to configured secure provider.

---

# 268. Bootstrap Cleanup

Remove one-time bootstrap material.

---

# 269. Circular Dependency Avoidance

Secret provider may be needed to connect DB, but metadata may be needed to resolve secret config.

Solve with:

```text
bootstrap configuration
  ↓
secret provider
  ↓
DB
  ↓
full Forgeyard config
```

---

# 270. Bootstrap Layer

Small, static, local.

---

# 271. Trust Bootstrap

Daemon server identity similarly configured through protected bootstrap files/provider.

---

# 272. Secret Provider Registry

Constructed in app bootstrap.

No global mutable registry.

---

# 273. Provider Dependency Direction

Domain -> SecretProvider trait.

Adapter -> AWS/Vault/etc.

---

# 274. No Cloud SDK in Core

Cloud SDK dependencies only provider adapters.

---

# 275. KMS Signing Adapter

Separate from generic string secret provider where semantics differ.

---

# 276. Provider Capabilities

```rust
pub struct SecretProviderCapabilities {
    pub versioning: bool,
    pub dynamic_credentials: bool,
    pub non_exportable_keys: bool,
    pub rotation: bool,
    pub leases: bool,
}
```

---

# 277. Capability-Aware Behavior

Do not pretend all providers support same semantics.

---

# 278. Provider Selection

Config/policy.

---

# 279. Secret Store Migration Compatibility

Semantic `SecretId` can remain stable while provider binding changes.

---

# 280. Secret Reference Indirection

Recommended:

```text
pipeline SecretRef logical name
  ↓
Forgeyard SecretId
  ↓
provider binding
```

This makes provider migration easier.

---

# 281. Pipeline Secret Name

Example:

```text
deploy.production.aws
```

logical identifier.

---

# 282. Environment Binding

Different environment maps same logical secret purpose to different provider secret.

---

# 283. Secret Template

Do not allow arbitrary interpolation that can accidentally expose values.

---

# 284. Composite Credentials

Provider adapter can resolve structured fields as one secret object.

---

# 285. JSON Credentials

If external format requires JSON, keep as opaque secret bytes.

Do not spread fields into metadata.

---

# 286. Binary Secret

Supported.

---

# 287. UTF-8 Assumption

Do not assume every secret is UTF-8.

---

# 288. Secret Testkit

```text
forgeyard-secrets-testkit/src/
├── lib.rs
├── fake_provider.rs
├── value.rs
├── lease.rs
├── delivery.rs
├── rotation.rs
├── redaction.rs
└── assertions.rs
```

---

# 289. Trust Testkit

```text
forgeyard-trust-testkit/src/
├── lib.rs
├── ca.rs
├── certificate.rs
├── enrollment.rs
├── revocation.rs
├── attestation.rs
└── assertions.rs
```

---

# 290. Unit Tests

Test:

```text
SecretRef parsing
scope
version selection
redacted Debug
zeroizing wrappers
```

---

# 291. Provider Conformance Tests

Every provider:

1. resolve;
2. not found;
3. denied;
4. version behavior;
5. rotation if supported;
6. outage mapping.

---

# 292. Delivery Tests

1. correct lease gets value;
2. stale lease denied;
3. wrong AgentSession denied;
4. expired secret denied;
5. source-trust policy denied.

---

# 293. Redaction Tests

Delivered secret does not appear in:

```text
Debug
error
log
trace
```

for controlled test paths.

---

# 294. Persistence Tests

Raw secret never appears in:

```text
metadata DB
CAS
outbox
local runner state
```

---

# 295. Cache Tests

Secret removed/zeroized after TTL/job.

---

# 296. Rotation Tests

Old/new dual window then old revoked.

---

# 297. Certificate Tests

```text
issue
renew
expire
revoke
root rotate
```

---

# 298. Session Fencing Test

Revoked runner cert/identity closes active session.

---

# 299. Signing Tests

General agent cannot resolve signing key.

Signing worker gets provider operation capability only.

---

# 300. Break-Glass Tests

No bypass without:

```text
permission
step-up
reason
audit
```

---

# 301. Air-Gap Tests

Local encrypted provider restores with separately protected master key.

---

# 302. Security Fuzzing

Fuzz:

```text
SecretRef parser
encrypted record decoder
certificate metadata decoder
enrollment messages
redaction matcher
```

Cryptographic primitive parsing delegated to vetted libraries.

---

# 303. Failure Injection

```text
provider outage
KMS timeout
cert issuer outage
disk permission failure
rotation halfway failure
revocation during job
```

---

# 304. Concurrency Tests

Two rotations/update operations do not create ambiguous active versions.

---

# 305. Performance Tests

Measure:

```text
secret resolve
dynamic credential issue
mTLS verification
certificate renewal
redaction overhead
```

---

# 306. Large Scale

Test:

```text
many tenants
many secrets
many concurrent short-lived credentials
many runners renewing certs
```

---

# 307. Rotation Storm

Jitter renewals/rotations to avoid thundering herd.

---

# 308. Certificate Renewal Window

Randomized within safe range.

---

# 309. Secret Rotation Window

Same.

---

# 310. Reconciliation

Stagger workers.

---

# 311. Implementation Phase 1 — Secret Model

Implement:

```text
SecretId
SecretRef
SecretVersionId
SecretValue
provider trait
permissions integration
```

---

# 312. Phase 2 — Local Secure Provider

Required for standalone.

---

# 313. Phase 3 — Runtime Delivery

Lease/attempt/session-bound secret delivery.

---

# 314. Phase 4 — Redaction / Cleanup

Agent injection and zeroization.

---

# 315. Phase 5 — Runner mTLS Trust

Enrollment, cert issuance, validation, revocation.

---

# 316. Phase 6 — External Secret Provider

Implement first enterprise provider adapter.

---

# 317. Phase 7 — Dynamic Workload Credentials

Cloud workload federation.

---

# 318. Phase 8 — Rotation / Reconciliation

Secrets/certs.

---

# 319. Phase 9 — Signing Key Operations

Restricted provider/worker path.

---

# 320. Phase 10 — Air-Gap / Backup

Secure local recovery.

---

# 321. Phase 11 — Attestation

Optional high-assurance.

---

# 322. Phase 12 — Hardening

Threat model, fuzzing, incident response, scale.

---

# 323. Acceptance Tests

1. Pipeline IR contains SecretRef, never value.
2. JobSpec contains SecretRef, never value.
3. CAS never receives secret value through normal secret path.
4. Metadata DB stores secret metadata only.
5. Raw secret Debug output is redacted.
6. General API errors never include secret value.
7. Secret resolution requires `secret.use`.
8. Secret plaintext read requires separate permission.
9. Stale job lease cannot receive secret.
10. Wrong AgentSession cannot receive secret.
11. Revoked workload cannot refresh secret.
12. Fork/untrusted proposal cannot receive privileged secret by default.
13. General runner cannot receive production signing private key.
14. Signing worker uses non-exportable key operation where provider supports it.
15. Secret file removed before workspace retention.
16. In-memory secret is zeroized best-effort after use.
17. Local secret store is encrypted at rest.
18. Local store master key is protected separately.
19. Provider migration does not require changing pipeline logical secret names.
20. Dynamic cloud credential expires automatically.
21. Runner certificate maps to exact RunnerId.
22. Runner restart does not inherit another identity.
23. Revoked runner identity is rejected even if cert cryptographically valid.
24. Certificate renewal preserves RunnerId.
25. Root rotation supports overlap then retirement.
26. Secret rotation supports safe activation/rollback.
27. Secret-use audit contains metadata but no value.
28. Break-glass secret access is explicit and audited.
29. Encrypted backup does not export plaintext by default.
30. Air-gapped restore requires separately protected key material.
31. Provider outage fails protected secret operation closed.
32. Logs/redaction tests detect no controlled secret leakage.
33. Same SecretRef/provider contract works in standalone/distributed modes.
34. Cloud-specific SDKs stay outside core secret crates.
35. Forgeyard self-hosting uses the same secret/trust infrastructure.

---

# 324. Production Readiness Gates

Do not call secrets/trust production-ready until:

```text
secret values excluded from IR/metadata/CAS
local encrypted provider tested
secret-use authz integrated
lease-bound delivery tested
redaction/zeroization implemented
runner mTLS enrollment stable
certificate rotation/revocation tested
backup/restore tested
signing key path separated
provider outage behavior fail-closed
audit coverage verified
```

Dynamic cloud identity, HSM/KMS, attestation, and advanced enterprise providers can reach readiness incrementally.

---

# 325. Architectural Invariants

1. secret references are normal data; secret values are not;
2. secret values never enter PipelineIr;
3. secret values never enter normal JobSpec persistence;
4. secret values never enter CAS;
5. secret values never enter ordinary event payloads;
6. secret values never enter ordinary runner local state;
7. secret values are resolved late;
8. secret use is distinct from secret read/admin;
9. secret delivery binds exact workload/lease/session;
10. expired/revoked secret cannot be refreshed;
11. secret values are short-lived in memory;
12. plaintext disk caching is forbidden;
13. local secret storage is encrypted;
14. master keys are protected separately;
15. provider-specific SDKs remain adapter-local;
16. workload federation preferred over static cloud keys;
17. runner trust cannot be self-asserted;
18. mTLS credentials are revocable;
19. RunnerId and certificate identity are explicitly bound;
20. root rotation supports overlap and retirement;
21. signing private keys never go to general runners;
22. non-exportable provider operations are preferred;
23. break-glass remains explicit/expiring/audited;
24. secret redaction is defense in depth, not primary containment;
25. logs are treated as potentially sensitive;
26. trust decisions fail closed on uncertainty for protected actions;
27. provider migration does not alter logical secret identity;
28. backup does not imply plaintext export;
29. high-assurance attestation is optional capability, not universal dependency;
30. Forgeyard dogfoods its own secrets/trust system.

---

# 326. Final Target Architecture

```text
                     Pipeline / Policy
                           │
                           ▼
                       SecretRef
                           │
                           ▼
                   Authorization
                           │
                           ▼
                  Secret Provider
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
       Local Encrypted   Vault/KMS    Cloud IAM
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                 SecretLease / Credential
                           │
                           ▼
            JobId + AttemptId + LeaseId
                   + AgentSessionId
                           │
                           ▼
                    Forgeyard Agent
                           │
                    sandbox injection
                           │
                           ▼
                        Workload
                           │
                           ▼
                  remove + zeroize
```

Trust:

```text
Trust Root
   ↓
Certificate / Public Key
   ↓
Runner / Service / Workload Principal
   ↓
Policy + Scope + Revocation State
   ↓
Authorized Trust Decision
```

---

# 327. Final Architectural Position

Secret access:

```text
SecretRef
+
Principal/WorkloadIdentity
+
Job/Attempt/Lease
+
SourceTrust
+
Purpose
+
PolicyDigest
  ↓
authorize
  ↓
resolve exact version
  ↓
short-lived SecretLease
  ↓
deliver
  ↓
inject
  ↓
zeroize
```

Runner trust:

```text
enrollment
  ↓
RunnerId
  ↓
client certificate
  ↓
mTLS
  ↓
server-side revocation/trust state
```

Signing:

```text
approved artifact digest
+
policy proof
+
SigningKeyRef
  ↓
restricted signing worker/provider
  ↓
signature/signed artifact
```

The key guarantee is:

> **Forgeyard can use credentials, signing keys, cloud identities, and sensitive tokens without turning them into ordinary build data. Secret material remains narrowly scoped, short-lived, provider-controlled where possible, revocable, auditable, and inaccessible to components that only need a reference or a permitted operation.**

---

# 328. New-Repository Sequence

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
