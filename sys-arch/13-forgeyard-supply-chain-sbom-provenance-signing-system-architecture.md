# 13 — Forgeyard Supply Chain, SBOM, Provenance & Signing System Architecture

**Document type:** Core Software Supply-Chain Security System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** SBOM, VEX, provenance, attestations, signatures, verification, policy evidence, reproducibility evidence, release evidence, in-toto/SLSA-aligned metadata, signing workers, artifact identity binding, dependency/license/vulnerability evidence, and promotion of verified bytes  
**Architecture style:** Evidence-first, digest-bound, immutable, reproducibility-aware, signing-separated, policy-verifiable, provider-neutral  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on the CAS/Data Plane, Pipeline IR, Run/Job, Secrets & Trust, Policy/Authz/Identity, hermetic/reproducible build architecture, Change Proposal system, VCS-neutral snapshots, and the future Release/Packaging systems. It does not redefine package building or deployment; it defines the evidence and trust chain that proves what was built, from what, by whom, where, and under which policy.

---

# 1. Purpose

Forgeyard must make builds verifiable.

A production artifact should not merely exist.

Forgeyard should be able to answer:

```text
what exact source produced this artifact?
which pipeline and JobSpec produced it?
which toolchains/dependencies were used?
which runner/executor produced it?
was the build hermetic?
was it reproduced independently?
which SBOM belongs to it?
which vulnerabilities were known?
which VEX statements apply?
which policy approved it?
who requested signing?
which key signed it?
which exact bytes were promoted?
```

The central rule is:

> **Every supply-chain claim is bound to immutable identities and cryptographic digests, never mutable names.**

A second rule is:

> **Forgeyard builds, verifies, signs, and promotes exact bytes. Signing or promotion must never silently rebuild the artifact.**

A third rule is:

> **Private signing keys remain outside general build runners. General runners produce evidence and unsigned artifacts; restricted signing workers/providers perform signing operations.**

---

# 2. Architectural Position

```text
 SourceSnapshotId
       │
       ▼
  PipelinePlanId
       │
       ▼
   JobSpecId
       │
       ▼
 Hermetic Build
       │
       ▼
   Artifact Digest
       │
       ├──────────────┐
       ▼              ▼
      SBOM        Provenance
       │              │
       ▼              ▼
   Vulnerability     Repro
     Evidence       Evidence
       │              │
       └───────┬──────┘
               ▼
         Policy Verify
               │
               ▼
        Signing Request
               │
               ▼
      Restricted Signer
               │
               ▼
       Signed Artifact
               │
               ▼
        Release Promote
```

---

# 3. Goals

The subsystem MUST:

1. generate artifact-bound SBOMs;
2. support SPDX;
3. support CycloneDX where useful;
4. generate provenance;
5. support SLSA/in-toto-aligned attestations;
6. store evidence in CAS;
7. store semantic evidence metadata in SQL;
8. bind evidence to exact artifact digest;
9. bind provenance to exact source snapshot;
10. bind provenance to exact pipeline plan/job spec;
11. record builder identity;
12. record executor/runner identity where appropriate;
13. record toolchain/dependency identities;
14. record hermeticity/reproducibility level;
15. generate VEX;
16. ingest scanner evidence;
17. evaluate supply-chain policy;
18. verify before signing;
19. keep signing keys out of general runners;
20. support KMS/HSM/provider signing;
21. support package-format signing;
22. support detached signatures;
23. support attestation signatures;
24. support key rotation;
25. support verification of historical releases;
26. support air-gap verification;
27. support release evidence bundles;
28. support audit;
29. support external interoperability;
30. remain independent from any one package ecosystem.

---

# 4. Non-Goals

This subsystem does not:

```text
replace package managers
replace vulnerability databases
replace external certificate authorities
replace artifact packaging
replace release orchestration
replace policy engine
```

It integrates those systems.

---

# 5. Workspace Structure

```text
crates/supply-chain/
├── forgeyard-supply-chain/
├── forgeyard-supply-chain-model/
├── forgeyard-sbom/
├── forgeyard-sbom-spdx/
├── forgeyard-sbom-cyclonedx/
├── forgeyard-vex/
├── forgeyard-provenance/
├── forgeyard-attestation/
├── forgeyard-in-toto/
├── forgeyard-slsa/
├── forgeyard-signature/
├── forgeyard-signature-verify/
├── forgeyard-evidence/
├── forgeyard-evidence-store/
├── forgeyard-evidence-policy/
├── forgeyard-evidence-bundle/
├── forgeyard-repro-evidence/
├── forgeyard-license-evidence/
├── forgeyard-vulnerability-evidence/
├── forgeyard-dependency-evidence/
├── forgeyard-builder-identity/
├── forgeyard-supply-chain-health/
└── forgeyard-supply-chain-testkit/
```

Signing-related application/worker:

```text
apps/forgeyard-signing-worker/
```

---

# 6. Core Evidence Identity

```rust
pub struct EvidenceId(Ulid);
```

Semantic evidence entity.

---

# 7. Evidence Object

```rust
pub struct EvidenceRecord {
    pub id: EvidenceId,
    pub kind: EvidenceKind,
    pub subject: EvidenceSubject,
    pub object: CasObjectRef,
    pub generated_at: Timestamp,
    pub generator: PrincipalId,
}
```

---

# 8. Evidence Subject

```rust
pub enum EvidenceSubject {
    Artifact(ArtifactId),
    CasObject(CasObjectId),
    SourceSnapshot(SourceSnapshotId),
    Release(ReleaseId),
    Run(RunId),
    Job(JobId),
}
```

---

# 9. Evidence Kind

```rust
pub enum EvidenceKind {
    Sbom,
    Vex,
    Provenance,
    Attestation,
    Signature,
    Reproducibility,
    VulnerabilityScan,
    LicenseScan,
    DependencyGraph,
    PolicyDecision,
    BuildManifest,
    Custom(EvidenceKindId),
}
```

---

# 10. Evidence Storage Rule

Actual document bytes:

```text
CAS
```

Metadata:

```text
ForgeyardStore
```

---

# 11. Evidence Immutability

Once an evidence document is generated:

```text
content immutable
```

Corrections create new evidence version/entity.

---

# 12. Subject Digest Binding

Evidence for artifact must include exact digest.

Never:

```text
artifact name only
version string only
release tag only
```

---

# 13. Artifact Subject

```rust
pub struct ArtifactSubject {
    pub artifact: ArtifactId,
    pub object: CasObjectRef,
}
```

---

# 14. Provenance

Provenance states:

```text
this artifact was produced
from these inputs
by this build definition
by this builder
under these conditions
```

---

# 15. Provenance Model

```rust
pub struct BuildProvenance {
    pub subject: Vec<ProvenanceSubject>,
    pub source: SourceSnapshotId,
    pub plan: PipelinePlanId,
    pub job_spec: JobSpecId,
    pub run: RunId,
    pub job: JobId,
    pub builder: BuilderIdentity,
    pub inputs: Vec<ProvenanceInput>,
    pub invocation: ProvenanceInvocation,
    pub environment: ProvenanceEnvironment,
    pub reproducibility: ReproducibilitySummary,
}
```

---

# 16. Provenance Subject

```rust
pub struct ProvenanceSubject {
    pub name: Option<ArtifactName>,
    pub digest: DigestSet,
}
```

---

# 17. Digest Set

```rust
pub struct DigestSet {
    pub blake3: Digest,
    pub sha256: Option<Digest>,
}
```

---

# 18. Source Provenance

Include:

```text
SourceSnapshotId
native VCS revision
repository identity
source provenance
```

---

# 19. VCS Neutrality

Provenance does not assume Git.

Possible source:

```text
Git commit
Mercurial changeset
Jujutsu change
local archive
```

all normalize to:

```text
SourceSnapshotId
```

---

# 20. Build Definition Identity

Use:

```text
PipelineDefinitionId
PipelinePlanId
JobSpecId
```

---

# 21. Builder Identity

```rust
pub struct BuilderIdentity {
    pub forgeyard_version: Version,
    pub builder_class: BuilderClass,
    pub runner: Option<RunnerId>,
    pub agent_session: Option<AgentSessionId>,
    pub executor: ExecutorIdentity,
}
```

---

# 22. Builder Class

```rust
pub enum BuilderClass {
    Standalone,
    DistributedRunner,
    Reproducer,
    SigningWorker,
}
```

---

# 23. Executor Identity

Include:

```text
executor type
sandbox profile
platform
isolation level
```

---

# 24. Provenance Environment

Do not dump all environment variables.

Record only relevant non-secret semantic environment.

---

# 25. Secret Handling in Provenance

Record:

```text
secret reference/purpose if required
```

not value.

---

# 26. Toolchain Evidence

Include immutable:

```text
ToolchainId
toolchain digest
target
```

---

# 27. Dependency Evidence

Include:

```text
lockfile digest
resolved dependency graph
dependency artifacts
```

---

# 28. Hermeticity Evidence

```rust
pub struct HermeticityEvidence {
    pub network: NetworkPolicy,
    pub declared_inputs_only: bool,
    pub managed_toolchains: bool,
    pub host_env_sanitized: bool,
}
```

---

# 29. Reproducibility Evidence

Integrates existing FRBS architecture.

---

# 30. Reproducibility Level

Use existing:

```rust
pub enum ReproducibilityLevel {
    Impure,
    Declared,
    Hermetic,
    DeterministicExpected,
    Reproduced,
    MultiPartyReproduced,
}
```

---

# 31. Reproduction Evidence

```rust
pub struct ReproductionEvidence {
    pub original: CasObjectRef,
    pub reproduction: CasObjectRef,
    pub level: ReproducibilityLevel,
    pub reproducer: BuilderIdentity,
    pub comparison: ReproductionComparison,
}
```

---

# 32. Reproduction Comparison

```rust
pub enum ReproductionComparison {
    BitForBit,
    NormalizedTree,
    Semantic,
    Mismatch(ReproductionMismatchRef),
}
```

---

# 33. Multi-Party Reproduction

Policy may require:

```text
different runner
different site
different trust domain
```

---

# 34. SBOM

SBOM describes components included in a produced artifact.

---

# 35. SBOM Formats

First-class:

```text
SPDX
CycloneDX
```

---

# 36. Internal SBOM Model

Do not make SPDX structs the core domain.

```rust
pub struct SbomDocument {
    pub subject: ArtifactSubject,
    pub components: Vec<SbomComponent>,
    pub relationships: Vec<SbomRelationship>,
    pub generator: SbomGenerator,
}
```

---

# 37. SPDX Adapter

Converts internal SBOM to/from supported SPDX representation.

---

# 38. CycloneDX Adapter

Same.

---

# 39. SBOM Sources

Components can come from:

```text
ecosystem dependency graph
native linker/object analysis
package manifest
binary inspection
container layers
explicit user declaration
```

---

# 40. Ecosystem Integration

Each ecosystem adapter provides dependency evidence.

Examples:

```text
Cargo.lock
go.sum
package-lock/pnpm-lock
uv.lock/poetry.lock
Gradle/Maven resolved graph
pubspec.lock
Package.resolved
```

---

# 41. Native Dependency Evidence

C/C++ and assembly-native subsystem contributes:

```text
linked libraries
object files
runtime dependencies
sysroot/libc
```

---

# 42. Binary Dependency Scan

Post-build binary inspection can validate actual runtime linkage.

---

# 43. Declared vs Observed Dependency

Track both.

---

# 44. Dependency Component

```rust
pub struct SbomComponent {
    pub id: ComponentId,
    pub name: String,
    pub version: Option<String>,
    pub purl: Option<PackageUrl>,
    pub hashes: DigestSet,
    pub licenses: Vec<LicenseExpression>,
    pub origin: ComponentOrigin,
}
```

---

# 45. Component Origin

```rust
pub enum ComponentOrigin {
    EcosystemResolved,
    NativeLinked,
    BinaryObserved,
    UserDeclared,
}
```

---

# 46. SBOM Coverage

Evidence should indicate completeness/coverage.

---

# 47. SBOM Completeness

```rust
pub enum SbomCompleteness {
    Partial,
    DeclaredDependencies,
    ResolvedDependencies,
    ResolvedAndObserved,
}
```

---

# 48. No False Completeness

Forgeyard must not claim complete SBOM when only lockfile data available.

---

# 49. License Evidence

Separate normalized license analysis.

---

# 50. License Expression

Use SPDX license expressions where interoperable.

---

# 51. License Policy

Policy engine can reject:

```text
forbidden license
unknown license
incompatible license
```

---

# 52. License Scan Evidence

```rust
pub struct LicenseEvidence {
    pub components: Vec<ComponentLicenseEvidence>,
    pub policy_digest: PolicyDigest,
}
```

---

# 53. Vulnerability Evidence

Scanner output normalized into internal model.

---

# 54. Vulnerability Finding

```rust
pub struct VulnerabilityFinding {
    pub vulnerability: VulnerabilityId,
    pub component: ComponentId,
    pub severity: VulnerabilitySeverity,
    pub status: VulnerabilityStatus,
    pub source: VulnerabilitySource,
}
```

---

# 55. Scanner Independence

Forgeyard may integrate:

```text
OSV
Trivy
Grype
ecosystem-native scanner
commercial scanner
```

via adapters.

---

# 56. Vulnerability Database Snapshot

Evidence should record:

```text
database/provider
database version/digest/time
```

---

# 57. Reproducible Scan Interpretation

A scan result can change as databases update.

Therefore evidence must include scanner/database context.

---

# 58. VEX

Vulnerability Exploitability eXchange communicates whether a known vulnerability affects the artifact.

---

# 59. VEX Status

Typical internal:

```rust
pub enum VexStatus {
    Affected,
    NotAffected,
    Fixed,
    UnderInvestigation,
}
```

---

# 60. VEX Statement

```rust
pub struct VexStatement {
    pub subject: ArtifactSubject,
    pub vulnerability: VulnerabilityId,
    pub status: VexStatus,
    pub justification: Option<VexJustification>,
    pub evidence: Vec<EvidenceId>,
}
```

---

# 61. VEX Authorization

Manual `NotAffected` claims should require:

```text
security permission
reason
evidence
audit
```

---

# 62. VEX Never Deletes Finding

It qualifies interpretation.

---

# 63. Attestation

Generic signed statement over subject/predicate.

---

# 64. Attestation Model

```rust
pub struct Attestation {
    pub subject: Vec<AttestationSubject>,
    pub predicate_type: AttestationPredicateType,
    pub predicate: CasObjectRef,
    pub signer: SigningIdentityRef,
}
```

---

# 65. in-toto

Use standard in-toto concepts for interoperability.

---

# 66. Statement Envelope

Forgeyard can produce:

```text
in-toto Statement
```

with supported predicates.

---

# 67. SLSA

Forgeyard should support SLSA-aligned provenance/evidence.

Do not hardcode marketing claims.

---

# 68. SLSA Evidence

Record actual capabilities:

```text
build service identity
source integrity
build definition
isolated execution
provenance generation
```

---

# 69. SLSA Claim

Only claim level/requirements when evidence genuinely satisfies current standard/profile.

---

# 70. Standard Version

Record standard version/profile.

---

# 71. Policy Evidence

Supply-chain policy can require:

```text
SBOM
provenance
reproducibility
no critical vulns
approved VEX
license compliance
signature
trusted builder
```

---

# 72. Evidence Requirement

```rust
pub enum EvidenceRequirement {
    Sbom { minimum: SbomCompleteness },
    Provenance,
    Reproducibility { minimum: ReproducibilityLevel },
    VulnerabilityPolicy(PolicyRef),
    LicensePolicy(PolicyRef),
    Signature(SignaturePolicy),
    Attestation(AttestationPredicateType),
}
```

---

# 73. Evidence Verification

```rust
pub struct EvidenceVerificationResult {
    pub subject: CasObjectRef,
    pub satisfied: Vec<EvidenceRequirement>,
    pub failed: Vec<EvidenceViolation>,
    pub policy_digest: PolicyDigest,
}
```

---

# 74. Verification Before Signing

Signing worker/request path must verify required evidence.

---

# 75. Signing Request Input

```rust
pub struct SupplyChainSigningRequest {
    pub subject: CasObjectRef,
    pub artifact: ArtifactId,
    pub evidence_bundle: EvidenceBundleId,
    pub policy_digest: PolicyDigest,
    pub key: SigningKeyRef,
}
```

---

# 76. Signing Key Boundary

Private key remains provider/restricted worker.

---

# 77. Signing Worker Must Not Build

Signing worker receives immutable subject only.

---

# 78. Signing Worker Must Not Resolve Source

It should not fetch arbitrary repo/build code.

---

# 79. Signing Worker Capability

Typed:

```text
verify evidence
sign exact digest
package-specific signing operation
```

---

# 80. General Runner Restriction

General build runner cannot request arbitrary production signing.

Authorization/policy proof required.

---

# 81. Detached Signature

Generic:

```text
signature object
+
subject digest
+
signer identity
```

---

# 82. Embedded Signature

Formats such as:

```text
Windows Authenticode
Apple codesign
APK signing
```

modify package bytes.

---

# 83. Byte Mutation Rule

If signing modifies artifact bytes:

```text
unsigned digest != signed digest
```

Both are separately stored.

---

# 84. Signing Lineage

Metadata links:

```text
UnsignedArtifact
  ↓
SignedArtifact
  ↓
NotarizedArtifact
```

---

# 85. No In-Place Artifact Mutation

Never replace CAS object under same identity.

---

# 86. Apple Signing

Restricted Apple signing worker/environment.

---

# 87. Windows Authenticode

Restricted signing worker/provider.

---

# 88. Android Signing

Restricted Android release signing path.

---

# 89. OCI Signing

Can produce external-standard signatures/attestations.

---

# 90. Package Registry Signing

Ecosystem-specific signing adapters may be integrated.

---

# 91. Signature Model

```rust
pub struct SignatureRecord {
    pub id: SignatureId,
    pub subject: CasObjectRef,
    pub algorithm: SignatureAlgorithm,
    pub signer: SigningIdentityRef,
    pub key_version: SigningKeyVersion,
    pub signature: CasObjectRef,
    pub signed_at: Timestamp,
}
```

---

# 92. Signing Identity

```rust
pub struct SigningIdentityRef {
    pub principal: PrincipalId,
    pub key: SigningKeyRef,
}
```

---

# 93. Signature Verification

```rust
pub struct SignatureVerification {
    pub signature: SignatureId,
    pub valid: bool,
    pub trust: SignatureTrustResult,
}
```

---

# 94. Signature Trust

```rust
pub enum SignatureTrustResult {
    Trusted,
    ValidButUntrusted,
    RevokedAfterSigning,
    RevokedAtSigning,
    Invalid,
    Unknown,
}
```

---

# 95. Historical Verification

Must use:

```text
key validity interval
revocation time
trust root history
```

---

# 96. Key Rotation

New signatures use new version.

Old public verification key retained.

---

# 97. Key Compromise

Policy can mark old signatures:

```text
untrusted retroactively
```

if compromise window demands.

---

# 98. Timestamping

External trusted timestamp service may improve long-term signature validity for some formats.

---

# 99. Timestamp Evidence

Store response/token in CAS.

---

# 100. Notarization

Apple notarization result/evidence stored.

---

# 101. Provenance Signature

Provenance/attestation itself can be signed.

---

# 102. Evidence Bundle

```rust
pub struct EvidenceBundle {
    pub id: EvidenceBundleId,
    pub subject: CasObjectRef,
    pub evidence: Vec<EvidenceId>,
    pub manifest: CasObjectRef,
}
```

---

# 103. Bundle Manifest

Contains exact digests of:

```text
artifact
SBOM
VEX
provenance
repro evidence
license evidence
vuln evidence
signatures
policy decisions
```

---

# 104. Release Evidence Bundle

Release can pin exact bundle.

---

# 105. Evidence Bundle Immutability

Adding new evidence creates a new bundle version/id.

---

# 106. Policy Decision Evidence

Persist high-level policy verification result.

---

# 107. Policy Digest Binding

Every release/signing decision includes exact:

```text
PolicyDigest
```

---

# 108. Change Proposal Evidence

Release can link:

```text
ProposalRevisionId
approval evidence
check runs
integration candidate/result
```

---

# 109. Exact Source Binding

Release artifact provenance binds the exact source snapshot that produced it.

---

# 110. Integration Snapshot

If artifact produced from integration candidate, provenance references exact resulting snapshot.

---

# 111. Build Once

Release path:

```text
build
  ↓
verify
  ↓
sign same bytes
  ↓
promote same signed bytes
```

---

# 112. No Release Rebuild

Do not:

```text
CI build passes
then release server rebuilds separately
```

unless intentionally performing independent reproduction and comparing.

---

# 113. Reproduction Is Not Promotion Build

Reproducer validates.

Promoted artifact remains approved original/signed lineage.

---

# 114. Provenance Builder ID

Use a Forgeyard builder namespace/version.

---

# 115. Builder Trust

Policy may distinguish:

```text
local developer build
general shared runner
trusted release runner
independent reproducer
```

---

# 116. Local Build Provenance

Can still generate provenance.

Marked:

```text
Standalone / Impure / Local
```

appropriately.

---

# 117. No False Trust Upgrade

Local artifact cannot gain enterprise release trust merely by uploading it.

---

# 118. Imported Artifact

Can enter as:

```text
ExternalArtifact
```

with unknown/untrusted provenance.

---

# 119. External Evidence

Forgeyard can ingest external SBOM/provenance/signature.

Must distinguish:

```text
Forgeyard-generated
external
verified external
```

---

# 120. Evidence Origin

```rust
pub enum EvidenceOrigin {
    ForgeyardGenerated,
    ExternalImported,
    ProviderGenerated,
}
```

---

# 121. External Evidence Verification

Validate:

```text
signature
subject digest
schema
trust
```

---

# 122. SBOM Generation Timing

Recommended:

```text
after build/package artifact exists
```

using both resolved dependency graph and final binary/package observation.

---

# 123. Intermediate SBOM

Could be generated earlier for dependency policy.

---

# 124. Final SBOM

Artifact-bound final evidence.

---

# 125. SBOM Drift

If artifact bytes change after signing/notarization:

signed artifact may need its own SBOM subject if content semantics changed materially.

---

# 126. Signature-Only Byte Change

Policy may link SBOM from unsigned artifact to signed artifact through lineage if component payload unchanged.

---

# 127. Explicit Lineage

Do not assume.

---

# 128. Package Manifest Evidence

Packaging subsystem emits:

```text
file manifest
package metadata
target
format
```

---

# 129. Binary Analysis

Optional post-build analysis:

```text
symbols
dynamic dependencies
ABI
debug linkage
```

---

# 130. Security Scan Pipeline

Potential stages:

```text
source scan
dependency scan
binary/package scan
SBOM scan
```

---

# 131. Source Scan

Secret scan/static security separate evidence.

---

# 132. Dependency Scan

Uses SBOM/dependency graph.

---

# 133. Binary Scan

Finds embedded components/runtime libraries.

---

# 134. Container Scan

Layer/package analysis.

---

# 135. Vulnerability Severity

Use normalized model.

---

# 136. Severity Source

Different databases may score differently.

Record source.

---

# 137. Vulnerability Policy

Examples:

```text
deny critical affected
allow fixed
allow NotAffected with approved VEX
warn high
```

---

# 138. Scan Freshness

Release policy may require scan less than N hours old.

---

# 139. Freshness Timer

Evidence metadata includes generated time/database snapshot.

---

# 140. Re-scan Existing Artifact

Can generate new vulnerability evidence without rebuilding artifact.

---

# 141. New Evidence Same Subject

Correct.

---

# 142. Continuous Vulnerability Monitoring

Future optional workflow.

Artifact digest remains same while known vulnerabilities evolve.

---

# 143. VEX Update

New VEX evidence can be added to new evidence bundle.

---

# 144. Release Status

Previously released artifact may become:

```text
security advisory needed
```

without changing bytes.

---

# 145. License Scan Freshness

Generally less time-sensitive but tooling/rules version recorded.

---

# 146. Provenance Generation Authority

Control plane/runner jointly provide data.

---

# 147. Runner-Supplied Evidence

Runner can report observed environment/resource details.

Control plane binds authoritative:

```text
RunId
JobId
PlanId
LeaseId
```

---

# 148. Evidence Tamper Prevention

Generated evidence bytes hashed into CAS.

---

# 149. Evidence Signing

Critical provenance/attestations should be signed.

---

# 150. Builder Attestation Key

Could be:

```text
Forgeyard service signing identity
```

separate from package code-signing key.

---

# 151. Key Separation

Different keys for:

```text
provenance attestation
release/package signing
TLS
CA
```

---

# 152. In-Toto Layout

If used, policy may define expected steps/materials/products.

---

# 153. Forgeyard Mapping

```text
source snapshot = material
toolchain/dependency closure = materials
job output = product
job/run identity = step metadata
```

---

# 154. Attestation Predicate Registry

```rust
pub enum AttestationPredicateType {
    BuildProvenance,
    Sbom,
    Vex,
    Reproducibility,
    VulnerabilityScan,
    LicenseScan,
    Custom(PredicateTypeId),
}
```

---

# 155. Predicate Versioning

Explicit.

---

# 156. Standard Export

Convert to:

```text
SPDX
CycloneDX
in-toto
SLSA provenance
DSSE
```

where appropriate.

---

# 157. DSSE

Useful as signed envelope for attestations.

---

# 158. Internal vs Standard Model

Maintain internal typed model.

Adapters serialize to standards.

---

# 159. JSON Necessity

Many external standards use JSON.

This is an appropriate JSON use despite Forgeyard's internal RON/Postcard preference.

---

# 160. Canonical Signing Representation

When signing JSON-based attestation, use standard-defined canonical/DSSE procedure.

Do not invent ad-hoc JSON signature.

---

# 161. Evidence Parsing

External evidence is untrusted input.

Bound:

```text
document size
component count
nesting
string length
```

---

# 162. SBOM Bomb Defense

Prevent millions of components causing memory exhaustion.

---

# 163. Signature Parser Defense

Use vetted libraries/format parsers.

---

# 164. Supply Chain Policy Engine Integration

Core policy remains in `forgeyard-policy`.

Supply-chain subsystem provides facts/evidence to policy evaluator.

---

# 165. Policy Facts

```rust
pub struct SupplyChainFacts {
    pub sbom: Option<SbomSummary>,
    pub vulnerabilities: VulnerabilitySummary,
    pub licenses: LicenseSummary,
    pub reproducibility: ReproducibilityLevel,
    pub signatures: Vec<SignatureSummary>,
    pub provenance: Option<ProvenanceSummary>,
}
```

---

# 166. No Duplicate Policy Engine

Do not implement a second policy language here.

---

# 167. Evidence Graph

```text
Artifact
 ├── SBOM
 ├── Provenance
 ├── Vuln Scan
 ├── VEX
 ├── License
 ├── Repro
 └── Signature
```

---

# 168. Graph Store

Semantic edges in metadata.

Documents in CAS.

---

# 169. Reverse Query

Given artifact:

```text
find all evidence
```

---

# 170. Forward Query

Given evidence:

```text
subject exact digest
```

---

# 171. Release Gate

Before release promotion:

```text
resolve exact artifact
  ↓
resolve evidence bundle
  ↓
verify evidence requirements
  ↓
verify signatures
  ↓
verify policy
  ↓
promote
```

---

# 172. Signing Gate

Before signing:

```text
unsigned artifact
  ↓
required evidence
  ↓
policy
  ↓
approved SigningRequest
```

---

# 173. Signing Idempotency

Same:

```text
subject digest
key version
signing mode
```

may be idempotent if signing algorithm/provider deterministic or recorded request identity.

---

# 174. Non-Deterministic Signatures

ECDSA/etc may produce different bytes.

Use `SigningRequestId` and provider idempotency where available.

---

# 175. Do Not Assume Signature Byte Determinism

Subject trust does not require identical signature bytes.

---

# 176. Signed Package Determinism

Some packaging/signing formats embed timestamps/nonces.

Reproducibility semantics must distinguish unsigned vs signed stages.

---

# 177. Reproducible Unsigned Build

Strong baseline.

---

# 178. Deterministic Signing

Optional where format/provider supports.

---

# 179. Timestamping Impact

Trusted timestamps intentionally make bytes time-dependent.

Record as signing-stage evidence.

---

# 180. Supply Chain Stage Model

```rust
pub enum ArtifactStage {
    Built,
    Verified,
    Signed,
    Notarized,
    Promoted,
}
```

---

# 181. Stage Is Metadata

Each byte-changing stage creates new artifact object.

---

# 182. Stage Transition

```text
Built Artifact A
  ↓ verification
Verified A
  ↓ signing
Signed Artifact B
  ↓ notarization/stapling
Artifact C
```

---

# 183. Verification Does Not Rewrite Bytes

`Verified` may reference same CAS object.

---

# 184. Signature Changes Bytes

New object.

---

# 185. Promotion Does Not Rewrite Bytes

Release metadata points to exact already-approved object.

---

# 186. Evidence Retention

Release evidence:

```text
long-lived
```

Build-only evidence may follow run retention.

---

# 187. Audit Retention

Signing/release decision long-lived.

---

# 188. Evidence GC Roots

Release pins all required evidence objects.

---

# 189. Artifact Delete

Cannot GC evidence still rooted by release/audit/legal hold.

---

# 190. Historical Verification

Must remain possible after:

```text
runner deleted
user removed
key rotated
provider migrated
```

---

# 191. Historical Identity

Store immutable principal IDs and public verification material/history.

---

# 192. Evidence Bundle Export

Air-gap:

```text
artifact
evidence manifest
SBOM
provenance
VEX
signatures
public verification chain
```

---

# 193. Air-Gap Verification

No external network required if bundle contains trusted verification data/policy snapshot.

---

# 194. Vulnerability DB Air-Gap

Can include scan evidence rather than live rescan.

Optional offline database mirror for re-scan.

---

# 195. Supply Chain CLI

```text
forgeyard sbom generate
forgeyard sbom show
forgeyard sbom export

forgeyard provenance show
forgeyard provenance verify

forgeyard vex list
forgeyard vex create
forgeyard vex verify

forgeyard evidence list
forgeyard evidence verify
forgeyard evidence bundle

forgeyard sign request
forgeyard sign verify

forgeyard supply-chain verify
```

---

# 196. `supply-chain verify`

Given artifact/release:

```text
verify digest
verify evidence bundle
verify provenance
verify signature
evaluate policy
```

---

# 197. Release Verify

Future `forgeyard release verify` calls supply-chain verifier.

---

# 198. UI

Artifact/release supply-chain tabs:

```text
Overview
Provenance
SBOM
Vulnerabilities
VEX
Licenses
Reproducibility
Signatures
Policy
Evidence Graph
```

---

# 199. Provenance UI

Show:

```text
source snapshot
VCS revision
pipeline
run/job
runner
toolchain
sandbox
repro level
```

---

# 200. SBOM UI

Search/filter components.

---

# 201. Vulnerability UI

Show source/database freshness.

---

# 202. VEX UI

Show justification/evidence/approver.

---

# 203. Signature UI

Show:

```text
key identity
key version
trust result
signed at
revocation status
```

---

# 204. Evidence Graph UI

Useful for release audit.

---

# 205. API

Potential:

```text
GET /v1/artifacts/{id}/sbom
GET /v1/artifacts/{id}/provenance
GET /v1/artifacts/{id}/evidence
POST /v1/artifacts/{id}/verify
POST /v1/signing/requests
GET /v1/signatures/{id}
```

---

# 206. Authorization

Permissions:

```text
artifact.read
supplychain.read
vex.create
vex.approve
signing.request
signing.admin
```

---

# 207. Signing Worker Protocol

Already separated from general agent protocol.

---

# 208. Signing Request Authority

Must include:

```text
requester
artifact digest
evidence bundle
policy digest
key ref
expiration
```

---

# 209. Signing Request TTL

Short.

---

# 210. Signing Worker Recheck

Before operation:

```text
request valid
subject digest exists
policy proof valid
key authorized
```

---

# 211. Signing Completion

Returns:

```text
signature
signed object ref if modified
signing evidence
```

---

# 212. Ambiguous Signing Result

If provider times out:

query provider/request status if supported.

Do not blindly duplicate destructive format operation.

---

# 213. Signature Provider Trait

```rust
#[async_trait]
pub trait SigningProvider {
    async fn sign(
        &self,
        request: ProviderSigningRequest,
    ) -> Result<ProviderSigningResult, SigningError>;
}
```

---

# 214. Package Signer Trait

```rust
#[async_trait]
pub trait PackageSigner {
    async fn sign_artifact(
        &self,
        artifact: CasObjectRef,
        context: PackageSigningContext,
    ) -> Result<SignedArtifactResult, SigningError>;
}
```

---

# 215. Detached vs Package Signer

Separate interfaces because embedded package signing semantics differ.

---

# 216. Verification Provider

```rust
pub trait SignatureVerifier {
    fn verify(
        &self,
        subject: &[u8],
        signature: &SignatureRecord,
    ) -> Result<SignatureVerification, VerificationError>;
}
```

Streaming for large subjects.

---

# 217. Key Trust Store

Integrates Trust subsystem.

---

# 218. Certificate Chains

Store public cert chain/evidence as CAS objects if needed.

---

# 219. Transparency Logs

Optional future integration:

```text
Sigstore/Rekor-style transparency
```

where desired.

---

# 220. Transparency Evidence

Store inclusion proof/reference.

---

# 221. Sigstore

Potential interoperability adapter.

Not architectural dependency.

---

# 222. Keyless Signing

Future/useful option via workload OIDC identities.

---

# 223. Keyless Identity

Policy must validate issuer/subject/workflow binding.

---

# 224. Ephemeral Signing Certificate

Store certificate/identity evidence.

---

# 225. Keyless vs Managed Key

Both fit `SigningIdentityRef`.

---

# 226. Artifact Verification at Download

CLI may verify:

```text
digest
signature
```

automatically for Forgeyard releases.

---

# 227. Self-Update

Future Forgeyard agent/daemon self-update must verify signed release/evidence.

---

# 228. Supply-Chain Bootstrapping

First trusted Forgeyard release needs documented root key/bootstrap trust.

---

# 229. Trust Root Distribution

Package/installers include public release verification root(s).

---

# 230. Root Rotation

Release verifier supports overlap.

---

# 231. Compromise Response

Publish revocation/security metadata.

---

# 232. Provenance Privacy

Do not include secret values, private environment, unnecessary personal data.

---

# 233. Runner Identity Exposure

Public provenance may use builder service identity rather than internal hostname.

---

# 234. Internal Provenance

Can retain more operational details.

---

# 235. Public Export Profile

Redacted/standardized.

---

# 236. Evidence Visibility

Tenant/project authz applies.

---

# 237. Public Release Evidence

Can be made public intentionally.

---

# 238. Vulnerability Embargo

Security evidence may have restricted visibility.

---

# 239. VEX Privacy

Could reveal sensitive exploitability reasoning.

Access control needed.

---

# 240. Evidence Event Model

Examples:

```text
SbomGenerated
ProvenanceGenerated
VulnerabilityScanCompleted
VexPublished
ArtifactVerified
SigningRequested
ArtifactSigned
SignatureRevoked
EvidenceBundleCreated
```

---

# 241. Reconciliation

Supply-chain reconciler checks:

```text
artifact missing required evidence
signature metadata/object mismatch
release evidence incomplete
stale vulnerability scan
signing request stuck
```

---

# 242. Signing Request States

```rust
pub enum SigningRequestState {
    Pending,
    Authorized,
    InProgress,
    Succeeded,
    Failed,
    Unknown,
    Cancelled,
}
```

---

# 243. Unknown

Used after ambiguous provider result.

---

# 244. Signing Reconciler

Queries provider or inspects artifact/signature state.

---

# 245. Evidence Generation Failure

Artifact remains built but not verified/releasable.

---

# 246. Supply Chain Status

```rust
pub enum SupplyChainStatus {
    Incomplete,
    Verified,
    Failed,
    Stale,
}
```

---

# 247. Stale Evidence

Example vulnerability scan freshness expired.

---

# 248. Release Gate on Stale

Policy decides.

---

# 249. Evidence Freshness

```rust
pub struct EvidenceFreshness {
    pub generated_at: Timestamp,
    pub valid_until: Option<Timestamp>,
}
```

---

# 250. SBOM Usually Immutable

No short freshness expiry.

---

# 251. Vulnerability Scan Time-Sensitive

Yes.

---

# 252. Provenance Immutable

Yes.

---

# 253. Signature Trust Can Change

Due to revocation/trust changes.

Verification recomputes.

---

# 254. Evidence Verification Cache

Cache by:

```text
subject digest
evidence bundle digest
policy digest
trust epoch
vulnerability policy/database context
```

---

# 255. Invalidate on Trust Change

Signature verification cache includes TrustEpoch.

---

# 256. Invalidate on Policy Change

Policy digest key naturally changes.

---

# 257. Scanner Adapter Trait

```rust
#[async_trait]
pub trait VulnerabilityScanner {
    async fn scan(
        &self,
        subject: ScanSubject,
    ) -> Result<VulnerabilityEvidence, ScanError>;
}
```

---

# 258. License Scanner Trait

Similar.

---

# 259. SBOM Generator Trait

```rust
#[async_trait]
pub trait SbomGenerator {
    async fn generate(
        &self,
        input: SbomGenerationInput,
    ) -> Result<SbomDocument, SbomError>;
}
```

---

# 260. Provenance Generator

Mostly internal deterministic assembler.

---

# 261. Provenance Data Sources

```text
pipeline plan
run/job state
runner/executor metadata
toolchain locks
source snapshot
CAS outputs
repro result
```

---

# 262. Provenance Completeness

If some field unavailable:

```text
explicit Unknown/NotAvailable
```

not omit ambiguously.

---

# 263. Builder Clock

Use control-plane timestamps where authority matters.

---

# 264. Monotonic Phase Times

Optional internal.

---

# 265. Supply Chain Doctor

```text
forgeyard supply-chain doctor
```

Checks:

```text
signing worker
trust roots
scanner availability
SBOM generator
provenance signer
evidence store
```

---

# 266. Health

Separate optional scanner degradation from required release signer availability.

---

# 267. Metrics

```text
sbom_generation_duration
provenance_generation_duration
vulnerability_scan_duration
evidence_verification_duration
signing_requests
signing_failures
signature_verify_failures
evidence_incomplete
repro_verified
```

---

# 268. Scanner Metrics

By scanner type, not component ID.

---

# 269. Signing Metrics

By provider/key class, not secret key ID if sensitive.

---

# 270. Tracing

```text
sbom.generate
provenance.generate
scan.vulnerability
vex.evaluate
evidence.verify
signing.authorize
signing.sign
signature.verify
```

---

# 271. Audit

Mandatory:

```text
manual VEX statement
policy exception
signing request
key selection
release evidence approval
signature revocation
```

---

# 272. Evidence Testkit

```text
forgeyard-supply-chain-testkit/src/
├── lib.rs
├── artifact.rs
├── sbom.rs
├── provenance.rs
├── vulnerability.rs
├── vex.rs
├── signature.rs
├── evidence_bundle.rs
└── assertions.rs
```

---

# 273. SBOM Tests

1. lockfile dependencies captured;
2. native linked dependency captured where supported;
3. final artifact digest bound;
4. stable export;
5. partial completeness labeled.

---

# 274. Provenance Tests

1. exact SourceSnapshotId;
2. exact PipelinePlanId;
3. exact JobSpecId;
4. exact artifact digest;
5. no secret values;
6. hermeticity level correct.

---

# 275. Repro Tests

Bit-for-bit match upgrades evidence.

Mismatch preserves both outputs.

---

# 276. VEX Tests

NotAffected requires justification/policy.

---

# 277. Signature Tests

1. correct digest verifies;
2. modified subject fails;
3. wrong key fails;
4. historical key rotation verifies;
5. revoked-at-signing fails policy.

---

# 278. Signing Isolation Test

General agent cannot access signing key operation.

---

# 279. Signing Lineage Test

Unsigned and signed object IDs differ if bytes changed.

---

# 280. Release Promotion Test

Promoted object equals previously verified signed digest exactly.

---

# 281. No Rebuild Test

Release promotion never invokes build executor.

---

# 282. Air-Gap Test

Evidence bundle verifies offline.

---

# 283. External Evidence Test

Untrusted imported provenance does not become Forgeyard-trusted automatically.

---

# 284. Scanner Freshness Test

Stale vulnerability evidence fails policy when freshness required.

---

# 285. Policy Digest Test

Evidence verification result changes when policy digest changes.

---

# 286. Trust Epoch Test

Signature cache invalidated by root/key revocation.

---

# 287. Fuzzing

Fuzz:

```text
SBOM parser
VEX parser
provenance parser
attestation envelope
signature metadata
evidence bundle manifest
```

---

# 288. External JSON Limits

Bound size/nesting/components.

---

# 289. Failure Injection

```text
scanner unavailable
signing provider timeout
CAS evidence upload fail
trust store unavailable
vulnerability DB unavailable
```

---

# 290. Ambiguous Signing Test

Provider timeout after sign -> reconcile rather than duplicate blindly.

---

# 291. Concurrency Tests

Duplicate signing requests with same request ID produce one semantic operation.

---

# 292. Scale Tests

Large monorepo:

```text
100k+ dependencies/components
```

without pathological memory behavior.

---

# 293. Performance

Stream large SBOM/evidence serialization where useful.

---

# 294. Implementation Phase 1 — Evidence Model

Implement:

```text
EvidenceId
EvidenceKind
EvidenceSubject
EvidenceStore
```

---

# 295. Phase 2 — Provenance

Generate Forgeyard-native provenance from Run/Job/source/toolchain data.

---

# 296. Phase 3 — SBOM

Implement internal SBOM + SPDX export.

---

# 297. Phase 4 — Vulnerability/License Evidence

Normalize scanner results.

---

# 298. Phase 5 — VEX

Manual/automated VEX statements and policy integration.

---

# 299. Phase 6 — Evidence Verification

Supply-chain facts -> policy.

---

# 300. Phase 7 — Restricted Signing

Signing worker/provider path.

---

# 301. Phase 8 — Embedded Package Signing

Windows/Apple/Android/OCI adapters.

---

# 302. Phase 9 — SLSA/in-toto/DSSE Interop

Standard export/verification.

---

# 303. Phase 10 — Reproducibility Evidence

Integrate independent reproduction policy.

---

# 304. Phase 11 — Release Evidence Bundles

Pin artifact/evidence/signature closure.

---

# 305. Phase 12 — Hardening

Air-gap, historical verification, scale, fuzzing, compromise response.

---

# 306. Acceptance Tests

1. Every release artifact can be resolved to exact CAS digest.
2. Provenance references exact SourceSnapshotId.
3. Provenance references exact PipelinePlanId and JobSpecId.
4. Provenance contains no secret values.
5. SBOM subject is exact artifact digest.
6. SBOM completeness is explicit.
7. Vulnerability evidence records scanner/database context.
8. VEX does not erase original vulnerability finding.
9. Manual NotAffected statement requires authorization/evidence.
10. Reproducibility evidence records actual comparison method.
11. Multi-party reproduction can require distinct failure domains.
12. General build runner has no production signing key.
13. Signing request binds exact artifact digest.
14. Signing request binds exact evidence/policy proof.
15. Embedded signing creates new CAS object when bytes change.
16. Unsigned object is preserved.
17. Signed object lineage is preserved.
18. Promotion uses exact verified signed bytes.
19. Release promotion does not rebuild.
20. Signature verification honors historical key validity/revocation.
21. Key rotation preserves old verification material.
22. Evidence documents are immutable.
23. Updating vulnerability scan creates new evidence, not artifact.
24. Stale vulnerability evidence can block release by policy.
25. Evidence bundle includes all required release evidence.
26. Air-gap evidence bundle verifies without network.
27. Imported external evidence remains distinguishable from Forgeyard-generated evidence.
28. Public evidence export can redact internal runner details.
29. Supply-chain policy reuses central Forgeyard policy engine.
30. JSON standards remain adapter-layer concerns.
31. Same evidence model works across all language ecosystems.
32. Same evidence model works standalone/distributed.
33. Signing provider timeout is reconciled safely.
34. Audit captures signing/VEX/exceptions.
35. Forgeyard's own releases are built, verified, signed, and promoted using this exact system.

---

# 307. Production Readiness Gates

Do not call supply-chain subsystem production-ready until:

```text
artifact digest binding stable
provenance generation stable
SBOM generation/export stable
evidence store/CAS integration stable
signature verification stable
restricted signing boundary proven
policy verification integrated
key rotation/history supported
release evidence bundle defined
no-rebuild promotion rule enforced
security/fuzz tests pass
```

VEX, SLSA profiles, keyless signing, transparency logs, and advanced scanner adapters can mature incrementally.

---

# 308. Architectural Invariants

1. all supply-chain claims bind immutable digests;
2. mutable tags/names are never evidence authority;
3. source identity is SourceSnapshotId;
4. build identity includes PipelinePlanId/JobSpecId;
5. artifact bytes are CAS-addressed;
6. evidence bytes are CAS-addressed;
7. evidence metadata stays separate;
8. provenance contains no secret values;
9. SBOM completeness is explicit;
10. vulnerability evidence records scanner/database context;
11. VEX qualifies findings but does not erase them;
12. general runners never receive production signing keys;
13. signing worker cannot run arbitrary build code;
14. signing request binds exact subject digest;
15. byte-changing signing creates new object identity;
16. unsigned/signed/notarized lineage is explicit;
17. verification never mutates artifact bytes;
18. promotion never rebuilds artifact;
19. release uses exact previously verified bytes;
20. signature verification considers trust history;
21. key rotation preserves historical verification;
22. policy engine remains centralized;
23. evidence generation is provider/ecosystem neutral;
24. imported evidence does not become trusted automatically;
25. JSON standards are interoperability adapters;
26. evidence parsers are bounded/untrusted-input safe;
27. release evidence is retained with release;
28. reproducibility evidence integrates FRBS levels;
29. air-gap verification is supported;
30. Forgeyard dogfoods its own supply-chain system.

---

# 309. Final Target Architecture

```text
                     SourceSnapshotId
                           │
                           ▼
                     PipelinePlanId
                           │
                           ▼
                        JobSpecId
                           │
                           ▼
                   Hermetic Execution
                           │
                           ▼
                     Artifact Digest
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
        SBOM          Provenance        Repro Evidence
          │                │                │
          ▼                ▼                ▼
      Vuln/License        SLSA/           Independent
       Evidence          in-toto          Verification
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                      VEX / Policy
                           │
                           ▼
                    Evidence Bundle
                           │
                           ▼
                   Signing Authorization
                           │
                           ▼
                  Restricted Signer/KMS
                           │
                           ▼
                     Signed Artifact
                           │
                           ▼
                      Verification
                           │
                           ▼
                    Release Promotion
```

---

# 310. Final Architectural Position

Build evidence:

```text
SourceSnapshotId
+
PipelinePlanId
+
JobSpecId
+
Toolchain IDs
+
Builder identity
+
Artifact digest
  ↓
Provenance
```

Security evidence:

```text
Artifact digest
+
SBOM
+
Vulnerability scan
+
VEX
+
License evidence
+
Reproducibility evidence
  ↓
Supply-chain facts
  ↓
Central policy engine
```

Signing:

```text
exact artifact digest
+
evidence bundle
+
policy digest
+
SigningKeyRef
  ↓
restricted signing worker/provider
  ↓
signature / signed artifact
```

Release:

```text
verified signed digest
  ↓
promote exact bytes
  ↓
never rebuild
```

The key guarantee is:

> **Forgeyard can prove the lineage and trust of every production artifact from immutable source snapshot to exact promoted bytes, while keeping build execution, evidence generation, signing authority, and release promotion as separate, auditable trust boundaries.**

---

# 311. New-Repository Sequence

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
