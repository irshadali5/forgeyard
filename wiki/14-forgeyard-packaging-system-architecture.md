# 14 — Forgeyard Packaging System Architecture

**Document type:** Core Distribution & Packaging System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Package planning, package manifests, platform/ecosystem adapters, deterministic packaging, installer/archive generation, container image packaging, repository metadata, signing handoff, artifact lineage, package validation, reproducibility, publishing preparation, and package-oriented evidence  
**Architecture style:** Build-output-to-package transformation with immutable inputs, deterministic manifests, adapter-isolated format logic, strict artifact lineage, and signing/release separation  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on `03` CAS/Data Plane, `04` Pipeline IR, hermetic/reproducible build architecture, `12` Secrets & Trust, `13` Supply Chain/SBOM/Provenance/Signing, and all language/platform ecosystem architecture documents. It produces package artifacts that later flow into `15 — Release` and `16 — Deployment`.

---

# 1. Purpose

Forgeyard needs a packaging subsystem that converts already-built outputs into artifacts users and deployment systems can consume.

Examples:

```text
.tar.zst
.zip
.deb
.rpm
.pkg
.dmg
.msi
.msix
.exe installer
.AppImage
.Flatpak bundle/repo artifact
Snap package
APK
AAB
IPA
XCArchive-derived release object
OCI image
Helm/chart-like bundle
generic server bundle
static-site bundle
```

Packaging is not "run whatever release script happens to exist."

It must be modeled explicitly.

The central rule is:

> **Packaging consumes immutable build outputs and produces new immutable package artifacts. It must not silently rebuild application code.**

A second rule is:

> **Every package format is an adapter behind one normalized package model.**

A third rule is:

> **If packaging or signing changes bytes, Forgeyard creates a new artifact identity and preserves lineage from the original build output.**

---

# 2. Architectural Position

```text
            Verified Build Outputs
                     │
                     ▼
                Package Plan
                     │
                     ▼
             Package Manifest
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
     Linux        Windows       Apple
     Adapter        Adapter      Adapter
        │            │            │
        └────────────┼────────────┘
                     ▼
               Package Artifact
                     │
          ┌──────────┼──────────┐
          ▼          ▼          ▼
       Validate     SBOM      Provenance
          │          │          │
          └──────────┼──────────┘
                     ▼
               Signing Handoff
                     │
                     ▼
             Signed Package Artifact
                     │
                     ▼
                  Release
```

---

# 3. Goals

The packaging subsystem MUST:

1. consume immutable build outputs;
2. define normalized package metadata;
3. define normalized package contents;
4. define deterministic package manifests;
5. support multiple target formats;
6. preserve file metadata explicitly;
7. normalize timestamps where format permits;
8. normalize ordering;
9. support package-specific reproducibility;
10. support Linux packaging;
11. support Windows packaging;
12. support macOS packaging;
13. support Android packaging;
14. support iOS packaging handoff;
15. support OCI/container images;
16. support generic archives;
17. support installer generation;
18. support package validation;
19. support package SBOM/provenance;
20. support signing handoff;
21. keep signing keys out of packaging workers;
22. support artifact lineage;
23. support package repository preparation;
24. support publishing metadata generation;
25. support dry-run package planning;
26. support package verification;
27. support cross-platform runner requirements;
28. support package adapter capability discovery;
29. remain independent from release promotion;
30. remain independent from deployment orchestration.

---

# 4. Non-Goals

Packaging does not:

```text
compile source
resolve arbitrary mutable dependencies
approve releases
deploy packages
hold production signing keys
decide release channels
```

---

# 5. Workspace Structure

```text
crates/package/
├── forgeyard-package/
├── forgeyard-package-model/
├── forgeyard-package-plan/
├── forgeyard-package-manifest/
├── forgeyard-package-layout/
├── forgeyard-package-metadata/
├── forgeyard-package-validate/
├── forgeyard-package-repro/
├── forgeyard-package-signing/
├── forgeyard-package-publish-model/
├── forgeyard-package-testkit/
│
├── forgeyard-package-archive/
├── forgeyard-package-deb/
├── forgeyard-package-rpm/
├── forgeyard-package-appimage/
├── forgeyard-package-flatpak/
├── forgeyard-package-snap/
├── forgeyard-package-msi/
├── forgeyard-package-msix/
├── forgeyard-package-windows-exe/
├── forgeyard-package-pkg/
├── forgeyard-package-dmg/
├── forgeyard-package-android/
├── forgeyard-package-apple-mobile/
├── forgeyard-package-oci/
└── forgeyard-package-server-bundle/
```

Potential ecosystem helpers remain in ecosystem crates, not package core.

---

# 6. PackageId

```rust
pub struct PackageId(Ulid);
```

Semantic package entity.

---

# 7. PackageSpecId

Canonical content/semantic plan digest:

```rust
pub struct PackageSpecId(Digest);
```

---

# 8. Package Artifact

Package bytes are CAS objects.

```rust
pub struct PackageArtifact {
    pub package: PackageId,
    pub spec: PackageSpecId,
    pub object: CasObjectRef,
    pub format: PackageFormat,
}
```

---

# 9. Package Format

```rust
pub enum PackageFormat {
    Tar,
    TarZstd,
    Zip,
    Deb,
    Rpm,
    AppImage,
    Flatpak,
    Snap,
    Msi,
    Msix,
    WindowsExeInstaller,
    Pkg,
    Dmg,
    Apk,
    Aab,
    Ipa,
    OciImage,
    ServerBundle,
    Custom(PackageFormatId),
}
```

---

# 10. Package Plan

```rust
pub struct PackagePlan {
    pub id: PackageSpecId,
    pub subject: PackageSubject,
    pub metadata: PackageMetadata,
    pub files: PackageLayout,
    pub format: PackageFormat,
    pub reproducibility: PackageReproducibilityPolicy,
    pub signing: PackageSigningRequirement,
}
```

---

# 11. Package Subject

```rust
pub struct PackageSubject {
    pub build_artifacts: Vec<ArtifactId>,
    pub build_objects: Vec<CasObjectRef>,
    pub source: SourceSnapshotId,
    pub plan: PipelinePlanId,
}
```

---

# 12. Package Metadata

```rust
pub struct PackageMetadata {
    pub name: PackageName,
    pub version: PackageVersion,
    pub architecture: PackageArchitecture,
    pub platform: PackagePlatform,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub maintainer: Option<String>,
}
```

---

# 13. Version

Package version is explicit input.

Do not derive from local Git implicitly.

---

# 14. Version Source

May come from:

```text
project manifest
release candidate
pipeline parameter
version file
```

but must be resolved before packaging.

---

# 15. Package Layout

```rust
pub struct PackageLayout {
    pub entries: Vec<PackageEntry>,
}
```

---

# 16. PackageEntry

```rust
pub struct PackageEntry {
    pub source: PackageSource,
    pub destination: PackagePath,
    pub mode: FileMode,
    pub ownership: PackageOwnership,
    pub kind: PackageEntryKind,
}
```

---

# 17. Package Source

```rust
pub enum PackageSource {
    Artifact(ArtifactId),
    Cas(CasObjectRef),
    Generated(GeneratedPackageFile),
}
```

---

# 18. Generated Package File

Examples:

```text
desktop file
systemd unit
manifest
plist
metadata/control file
license notice
```

Generated deterministically from package plan.

---

# 19. No Arbitrary Host File

Strict packaging cannot include:

```text
/home/user/random-file
```

unless materialized as explicit immutable input.

---

# 20. Package Path

Validated target-relative path.

---

# 21. Path Safety

Reject:

```text
..
absolute paths
drive escape
invalid target path
```

---

# 22. File Ownership

Package ownership is semantic:

```text
root:root
app user
current user
```

not inherited blindly from build host.

---

# 23. File Mode

Explicit.

---

# 24. Timestamp Normalization

Use controlled timestamp where format permits.

---

# 25. Ordering

Canonical file order.

---

# 26. Compression Parameters

Pinned/explicit for reproducibility.

---

# 27. Archive Packaging

Generic archive adapter supports:

```text
tar
tar.zst
zip
```

---

# 28. Archive Manifest

Can include:

```text
manifest.ron
checksums
LICENSE
README
SBOM
provenance
```

depending release policy.

---

# 29. Server Bundle

For daemon/server deployments:

```text
binary
config templates
migration binaries
systemd/service files
licenses
SBOM/provenance refs
```

---

# 30. Linux Package Architecture

Linux package formats should consume one normalized Linux layout.

---

# 31. Linux Layout

Example:

```text
/usr/bin/forgeyard
/usr/lib/forgeyard/...
/usr/share/doc/forgeyard/...
/usr/lib/systemd/system/forgeyard.service
```

format adapter maps appropriately.

---

# 32. DEB Adapter

Responsibilities:

```text
control metadata
dependencies
filesystem layout
maintainer scripts only when necessary
deterministic archive construction
```

---

# 33. Maintainer Scripts

Avoid where declarative packaging suffices.

Scripts are high risk and harder to make portable/reproducible.

---

# 34. RPM Adapter

Same normalized package intent.

---

# 35. Dependency Metadata

System package dependencies should be explicit.

---

# 36. Auto Dependency Detection

May assist but must produce inspectable package plan.

---

# 37. AppImage

Build from exact application layout.

---

# 38. Flatpak

Flatpak build semantics may involve external runtime/app manifests.

Treat manifest/runtime as explicit inputs.

---

# 39. Snap

Same principle.

---

# 40. Windows Packaging

Native Windows packages should be built on Windows where platform tooling/validation/signing requires it.

---

# 41. MSI

Adapter handles:

```text
ProductCode/UpgradeCode semantics
components/features
install paths
uninstall behavior
service registration
```

---

# 42. Stable Upgrade Identity

MSI identity fields must follow version/update strategy.

---

# 43. MSIX

Adapter handles:

```text
manifest
capabilities
identity
package layout
```

---

# 44. Windows EXE Installer

Could use a selected installer backend.

Forgeyard core models installer semantics, adapter executes backend.

---

# 45. No Installer Tool Lock-In

Do not bake one vendor/tool into core.

---

# 46. Windows Signing

Unsigned package first.

Restricted signing phase produces signed package.

---

# 47. Apple Packaging

macOS packages require real Apple tooling where necessary.

---

# 48. `.pkg`

Adapter constructs installer package.

---

# 49. `.dmg`

Distribution image around signed/notarized app/package.

---

# 50. macOS App Bundle

Packaging may normalize:

```text
.app structure
Info.plist
resources
embedded frameworks
```

---

# 51. Apple Signing Sequence

Typical:

```text
build components
  ↓
assemble app bundle
  ↓
codesign nested components/app
  ↓
package/dmg
  ↓
sign package where required
  ↓
notarize/staple
```

Each byte-changing stage gets new artifact identity.

---

# 52. Apple Packaging vs Signing Boundary

Package adapter prepares signing operations.

Restricted signing worker/provider performs key operations.

---

# 53. Android Packaging

Android application build may already produce APK/AAB through Android toolchain.

Forgeyard packaging system normalizes:

```text
artifact metadata
validation
alignment/signing stages
distribution metadata
```

---

# 54. Android APK

Unsigned APK -> signing/alignment sequence according to Android toolchain semantics.

---

# 55. Android AAB

Bundle artifact prepared for signing/publishing.

---

# 56. Android Release Key

Never general runner secret.

---

# 57. iOS Packaging

Requires Apple runner/toolchain.

---

# 58. IPA

Derived from exact signed archive/app bundle.

---

# 59. Provisioning Profiles

Sensitive release configuration handled through secret/trust subsystem.

---

# 60. OCI Image Packaging

OCI image is package artifact.

---

# 61. OCI Model

```rust
pub struct OciImagePlan {
    pub config: OciConfig,
    pub layers: Vec<OciLayerPlan>,
    pub annotations: BTreeMap<String, String>,
}
```

---

# 62. OCI Layers

Construct deterministically from immutable inputs.

---

# 63. OCI Base Image

Resolved by digest.

Never mutable tag only in strict/release mode.

---

# 64. OCI Metadata

Avoid unnecessary timestamps/random data.

---

# 65. Container Entrypoint

Explicit.

---

# 66. OCI SBOM

Can attach as evidence/attestation rather than bake into filesystem unless desired.

---

# 67. OCI Signing

Separate signing/attestation stage.

---

# 68. Multi-Architecture OCI

Manifest/index references exact platform image digests.

---

# 69. Package Manifest

Every generated package has Forgeyard-native manifest.

```rust
pub struct ForgeyardPackageManifest {
    pub spec: PackageSpecId,
    pub format: PackageFormat,
    pub subject: PackageSubject,
    pub entries: Vec<ManifestEntry>,
    pub generated_files: Vec<GeneratedFileRecord>,
    pub toolchain: PackageToolchainIdentity,
}
```

---

# 70. Package Manifest Storage

Manifest bytes in CAS.

Metadata references it.

---

# 71. Manifest Is Not Necessarily Embedded

Could be embedded if format/release policy desires.

---

# 72. Package Toolchain Identity

Include:

```text
dpkg tooling
rpm tooling
WiX/MSIX backend
Apple packaging tools
Android build tools
OCI builder
compression library
```

---

# 73. Toolchain Pinning

Release package toolchain must be immutable/pinned.

---

# 74. Packaging Derivation

Packaging itself can be represented as hermetic derivation:

```text
package output =
f(
  build artifacts,
  package plan,
  packaging toolchain,
  controlled environment
)
```

---

# 75. PackageDerivationId

```rust
pub struct PackageDerivationId(Digest);
```

---

# 76. Package Reproducibility

```rust
pub struct PackageReproducibilityPolicy {
    pub expected: ReproducibilityExpectation,
    pub normalized_fields: Vec<PackageNormalizedField>,
}
```

---

# 77. Reproducible Package

Same inputs/toolchain -> same unsigned package bytes where format permits.

---

# 78. Format Constraints

Some formats/signing stages introduce unavoidable/non-deterministic fields.

Mark explicitly.

---

# 79. Unsigned Reproducibility

Prefer validating unsigned package reproducibility independently from signing.

---

# 80. Package Reproduction

Can run package derivation on independent runner and compare.

---

# 81. Package Normalized Comparison

If format inherently contains non-semantic variation, normalized-tree/semantic verification may apply.

---

# 82. No False Bit-for-Bit Claim

Use existing reproducibility levels.

---

# 83. Package Builder

Packaging should execute via normal Runner/Sandbox/Executor.

No special privileged daemon build path.

---

# 84. Package Worker

General runner is allowed if package step does not require sensitive signing.

---

# 85. Signing Worker

Only for signing/key operations.

---

# 86. Package Step JobSpec

Contains:

```text
immutable input refs
package plan
toolchain
output declaration
network policy
```

---

# 87. Network

Packaging should generally be network-denied once dependencies/tooling are resolved.

---

# 88. Package Validation

After package generation, adapter validates package.

---

# 89. Validation Categories

```text
format integrity
metadata
file layout
dependency metadata
signature state
installability static checks
policy
```

---

# 90. Install Test

Strong validation can install package in clean disposable environment.

---

# 91. Linux Install Test

Fresh container/VM matching distro.

---

# 92. Windows Install Test

Windows VM.

---

# 93. macOS Install Test

macOS VM/runner where licensing/platform permits.

---

# 94. Android Install Test

Emulator/physical device through device lab.

---

# 95. iOS Install Test

Simulator/device as supported.

---

# 96. Package Smoke Test

Install -> launch/version -> uninstall.

---

# 97. Installer Side Effects

Test in isolated disposable environment.

---

# 98. Package Validation Evidence

Stored as evidence.

---

# 99. Package SBOM

Final package artifact gets exact subject-bound SBOM.

---

# 100. Package Provenance

Provenance includes packaging transformation.

---

# 101. Build vs Package Provenance

Possible chain:

```text
BuildArtifact provenance
  ↓
Packaging provenance
  ↓
PackageArtifact
```

---

# 102. Transformation Provenance

Records:

```text
input artifact digests
PackageSpecId
PackageDerivationId
packaging toolchain
output digest
```

---

# 103. Signing Provenance

Separate stage from package derivation.

---

# 104. Package Lineage

```rust
pub struct PackageLineage {
    pub inputs: Vec<CasObjectRef>,
    pub unsigned: CasObjectRef,
    pub signed: Option<CasObjectRef>,
    pub notarized: Option<CasObjectRef>,
}
```

---

# 105. No In-Place Mutation

Every byte-changing stage creates new CAS object.

---

# 106. Metadata-Only State

Verification/promotion can change metadata state without changing object.

---

# 107. Package State

```rust
pub enum PackageState {
    Planned,
    Building,
    Built,
    Validating,
    Verified,
    AwaitingSignature,
    Signed,
    Notarized,
    Failed,
}
```

---

# 108. Package State Is Not Release State

Release subsystem later decides candidate/promotion/channel.

---

# 109. Package Attempt

Packaging is executed as normal JobAttempt.

Package entity links resulting job/run.

---

# 110. Package Service

```rust
#[async_trait]
pub trait PackageService {
    async fn plan_package(...);
    async fn register_result(...);
    async fn verify_package(...);
}
```

---

# 111. Package Adapter Trait

```rust
#[async_trait]
pub trait PackageAdapter: Send + Sync {
    fn format(&self) -> PackageFormat;

    fn requirements(
        &self,
        plan: &PackagePlan,
    ) -> PackageCapabilityRequirements;

    async fn build(
        &self,
        context: PackageBuildContext,
    ) -> Result<PackageBuildResult, PackageError>;

    async fn validate(
        &self,
        artifact: &PackageArtifact,
    ) -> Result<PackageValidationResult, PackageError>;
}
```

---

# 112. Adapter Requirements

Returns:

```text
OS
architecture
packaging toolchain
SDK
sandbox
```

---

# 113. Capability Flow

Package plan -> adapter requirements -> scheduler.

---

# 114. Cross-Build Packaging

Generic archive can package cross-built outputs.

Native installers may require target OS.

---

# 115. Do Not Fake Native Validation

A Windows MSI created elsewhere without Windows validation cannot automatically be claimed production-ready.

---

# 116. Build/Host/Target Types

Reuse existing strong Build/Host/Target architecture.

---

# 117. Package Target

```rust
pub struct PackageTarget {
    pub os: TargetOs,
    pub arch: TargetArch,
    pub format: PackageFormat,
}
```

---

# 118. Distribution Target

Linux distro package may include:

```text
debian-family/version
rpm-family/version
```

---

# 119. ABI Compatibility

Package metadata can state minimum OS/runtime requirements.

---

# 120. glibc/musl

Linux artifact/package must preserve actual libc requirement.

---

# 121. Universal Binaries

Apple multi-arch package can contain universal app.

---

# 122. Windows Architecture

x64/arm64/etc explicit.

---

# 123. Android ABI

APK/AAB semantics explicit.

---

# 124. Package Metadata Sources

Project-level package config.

Example:

```text
.forgeyard/package.ron
```

---

# 125. Human Package Config

Illustrative:

```ron
(
    packages: [
        (
            name: "forgeyard-cli-linux",
            format: TarZstd,
            target: (
                os: Linux,
                arch: X86_64,
            ),
            files: [
                (
                    artifact: "forgeyard-cli",
                    to: "bin/forgeyard",
                    mode: "0755",
                ),
            ],
        ),
    ],
)
```

---

# 126. Package Config Parser

Could live in package model/parser or reuse general config infrastructure.

---

# 127. Package Config Validation

Check:

```text
referenced output exists
destination collision
format metadata complete
target compatible
signing requirement compatible
```

---

# 128. Destination Collision

Two files to same path:

```text
error
```

unless explicit override semantics.

---

# 129. Duplicate Metadata

Reject conflicting values.

---

# 130. Package Description

Non-semantic text can be versioned but still affects package bytes if embedded.

Therefore part of PackageSpecId.

---

# 131. Maintainer Metadata

Same.

---

# 132. License Files

Explicit package entries.

---

# 133. Generated Notices

Generated from dependency/license evidence.

---

# 134. Third-Party Notices

Could generate deterministic NOTICE artifact.

---

# 135. Desktop Integration

Linux:

```text
.desktop
icons
MIME associations
```

Windows:

```text
shortcuts
file associations
service registration
```

macOS:

```text
Info.plist
bundle identifiers
icons
```

---

# 136. Bundle Identifier

Stable semantic ID.

---

# 137. Application Identifier

Use typed:

```rust
pub struct ApplicationId(BoundedString);
```

---

# 138. Package Identifier

Format-specific mapping from stable Forgeyard application/package IDs.

---

# 139. Upgrade Semantics

Packaging architecture must model:

```text
install
upgrade
downgrade
uninstall
```

---

# 140. Upgrade Compatibility

Package adapter validates version identity rules.

---

# 141. Config Files

System package can mark config files appropriately.

---

# 142. User Data

Installer/package must not treat user data as package-owned artifact.

---

# 143. Uninstall

Should not delete user data by default unless product semantics require.

---

# 144. Service Installation

Server package can install:

```text
systemd unit
Windows service
launchd plist
```

---

# 145. Service Credentials

Never baked into package.

---

# 146. Configuration

Package includes templates/defaults, not production secrets.

---

# 147. Post-Install Configuration

Handled by user/operator/deployment subsystem.

---

# 148. Migrations

Package may include migration binary.

Deployment/release decides when to execute migration.

---

# 149. No Auto DB Migration in Installer by Default

Dangerous for server packages.

---

# 150. Package Repository

Packaging prepares repository-ready objects/metadata.

Publishing is later action.

---

# 151. Linux Repository Metadata

Potential:

```text
APT metadata
RPM repository metadata
```

---

# 152. Repository Metadata Signing

Restricted signing operation.

---

# 153. Repository Index Identity

Index bytes are separate artifact.

---

# 154. Repository Update

Produces new immutable index snapshot.

---

# 155. Package Registry Metadata

Container/package registry publish descriptors prepared.

---

# 156. Publishing Model

```rust
pub struct PackagePublishDescriptor {
    pub package: PackageId,
    pub object: CasObjectRef,
    pub destination: PublishDestination,
    pub coordinates: PackageCoordinates,
}
```

---

# 157. Publish Destination

```rust
pub enum PublishDestination {
    AptRepository,
    RpmRepository,
    OciRegistry,
    GenericHttp,
    PackageRegistry,
    AppStore,
    PlayStore,
    Custom(PublishDestinationId),
}
```

---

# 158. Release Subsystem Owns Publish Decision

Packaging only prepares descriptor/artifact.

---

# 159. App Stores

Android/iOS store upload is release/publish workflow.

---

# 160. Package Validation Before Publish

Required according to policy.

---

# 161. Package Scan

Security scan final package.

---

# 162. Malware Scan

Optional enterprise evidence.

---

# 163. Binary Signature Check

After signing.

---

# 164. Package Format Validation

Use native tooling where available.

---

# 165. Installer Simulation

Optional.

---

# 166. Package Metadata DB

Store:

```text
PackageId
PackageSpecId
format
target
state
artifact refs
lineage
validation refs
```

---

# 167. Package Bytes

CAS only.

---

# 168. Large Package

Stream.

---

# 169. Package Cache

Packaging derivation can be action-cached.

---

# 170. Cache Key

Includes:

```text
input artifact digests
package spec
toolchain
format adapter semantic version
```

---

# 171. Package Cache Trust

Release may require trusted/reproduced package even if cache hit.

---

# 172. Cache Hit

Still validate evidence/signature policy.

---

# 173. Package Spec Canonicalization

Deterministic RON/model normalization.

---

# 174. PackageSpecId

Digest of canonical semantic package plan.

---

# 175. Adapter Version

Part of derivation identity if behavior affects bytes.

---

# 176. Compression Library Version

Same if output bytes depend on it.

---

# 177. Packaging Environment

Controlled:

```text
TZ
locale
umask
hostname
user
timestamps
```

---

# 178. SOURCE_DATE_EPOCH

Use where compatible.

---

# 179. Package Timestamp Source

Source snapshot timestamp or explicit release epoch according to reproducibility architecture.

---

# 180. Random IDs

Avoid if format permits.

If format requires GUID-like IDs:

derive deterministically from semantic package identity where safe.

---

# 181. MSI GUIDs

Must respect Windows Installer upgrade/component semantics.

Can derive stable UUIDv5-like identifiers from package/component identity where appropriate.

---

# 182. Randomness

If required for security/signing, separate from deterministic unsigned packaging.

---

# 183. Temporary Files

Per attempt.

---

# 184. Host Tool Leakage

Strict package jobs only use managed toolchain.

---

# 185. Package Adapter Error

```rust
pub enum PackageError {
    InvalidPlan,
    UnsupportedTarget,
    MissingInput,
    ToolchainUnavailable,
    BuildFailed,
    ValidationFailed,
    ReproMismatch,
    SigningRequired,
    Internal,
}
```

---

# 186. Failure Class Mapping

Packaging build failure is workload/package failure unless infrastructure-specific.

---

# 187. Adapter Diagnostic

Provide format-specific actionable diagnostics.

---

# 188. Package Explain

```text
forgeyard package explain
```

Shows:

```text
inputs
layout
target
toolchain
signing requirement
repro policy
```

---

# 189. CLI

```text
forgeyard package plan
forgeyard package build
forgeyard package validate
forgeyard package inspect
forgeyard package reproduce
forgeyard package lineage
forgeyard package list
```

---

# 190. `package plan`

No execution.

---

# 191. `package build`

Creates Job/Run or local execution according to mode.

---

# 192. `package validate`

Validates package object.

---

# 193. `package inspect`

Shows normalized package metadata/file list.

---

# 194. `package lineage`

Shows:

```text
build outputs
unsigned package
signed package
notarized package
```

---

# 195. Dioxus UI

Package page:

```text
Overview
Inputs
Layout
Metadata
Validation
SBOM
Provenance
Signatures
Lineage
Publish Targets
```

---

# 196. Package Matrix

One product release may include:

```text
Linux x86_64 tar
Linux arm64 tar
Windows x64 MSI
Windows arm64 MSIX
macOS universal DMG
Android APK/AAB
```

---

# 197. Package Group

```rust
pub struct PackageSetId(Ulid);
```

Groups related target packages.

---

# 198. Package Set

```rust
pub struct PackageSet {
    pub id: PackageSetId,
    pub version: PackageVersion,
    pub packages: Vec<PackageId>,
}
```

---

# 199. Release Candidate Uses Package Set

Release subsystem can promote a package set.

---

# 200. Partial Package Failure

One target package can fail while others succeed.

Release policy decides whether full set required.

---

# 201. Cross-Target Version Consistency

Package set enforces same logical version where desired.

---

# 202. Cross-Target Source Consistency

All package artifacts in one release should bind same SourceSnapshotId unless explicitly multi-source.

---

# 203. Cross-Target Plan Consistency

Release records exact plan/spec for each target.

---

# 204. Package Naming

Human names not identities.

---

# 205. Output Filename

Generated deterministic from:

```text
name
version
target
format
```

---

# 206. Filename Collision

Reject.

---

# 207. Sanitization

Platform-safe.

---

# 208. Content Type

Package metadata records MIME/media type.

---

# 209. Checksum File

Can generate:

```text
SHA256SUMS
BLAKE3SUMS
```

as release/package artifacts.

---

# 210. Checksum Signing

Can be signed separately.

---

# 211. Package Index Manifest

For download site, generate machine-readable manifest.

---

# 212. Public Manifest

JSON may be appropriate for interoperability/site.

Internal canonical manifest can be RON/Postcard/CAS.

---

# 213. Download Metadata

Includes:

```text
platform
arch
format
digest
size
signature
```

---

# 214. Package Size

Recorded from CAS.

---

# 215. Delta Updates

Future optional.

---

# 216. Delta Package

Derived from:

```text
old exact package
new exact package
```

---

# 217. Delta Is New Artifact

With provenance linking both.

---

# 218. Delta Validation

Reconstruct new package and verify exact digest.

---

# 219. Auto-Update Feeds

Release subsystem later publishes update feed.

Packaging can create feed entries/manifests.

---

# 220. Feed Signing

Restricted signing.

---

# 221. Supply Chain Integration

Each package should have:

```text
SBOM
provenance
validation evidence
signature evidence
```

as policy requires.

---

# 222. Evidence Bundle

Package-level evidence bundle becomes release input.

---

# 223. Package Provenance Chain

```text
SourceSnapshot
  ↓
BuildArtifact
  ↓
PackageArtifact
  ↓
SignedPackage
  ↓
Release
```

---

# 224. VEX

VEX can target final package digest.

---

# 225. License Notice

Package can embed notices generated from evidence.

---

# 226. Package Policy Facts

```rust
pub struct PackagePolicyFacts {
    pub format: PackageFormat,
    pub target: PackageTarget,
    pub validated: bool,
    pub reproducibility: ReproducibilityLevel,
    pub signed: bool,
    pub sbom: bool,
}
```

---

# 227. Policy Examples

```text
Windows production package must be signed
macOS release must be notarized
Android release must use production signing key
Linux tar must have SBOM
all release packages must be reproduced
```

---

# 228. Policy Engine Reuse

No separate package policy language.

---

# 229. Signing Requirement

```rust
pub enum PackageSigningRequirement {
    None,
    Optional,
    Required(SigningProfileId),
}
```

---

# 230. Signing Profile

Defines:

```text
key class
format signer
timestamping
notarization
```

---

# 231. Package Adapter Does Not Resolve Key Value

Only SigningKeyRef/profile.

---

# 232. Restricted Signing Request

Generated after package validation/evidence.

---

# 233. Notarization Request

Separate external side effect.

---

# 234. Notarization State

```text
Pending
Submitted
Accepted
Rejected
Unknown
```

---

# 235. Ambiguous Notarization

Reconcile provider state before retry.

---

# 236. Package Publish Credentials

Resolved only by release/publish worker.

---

# 237. No Publish Credential in Packaging Job

Unless packaging itself absolutely requires provider operation, which should be separated.

---

# 238. Package Reconciler

Checks:

```text
Built but no artifact ref
Verified state without validation evidence
Signed state missing signed object
notarization unknown
```

---

# 239. Event Model

Examples:

```text
PackagePlanned
PackageBuilt
PackageValidationSucceeded
PackageValidationFailed
PackageSigningRequested
PackageSigned
PackageNotarized
```

---

# 240. Event Payload

Digest/IDs only, no keys/secrets.

---

# 241. Observability

Metrics:

```text
package_build_duration
package_validation_duration
package_size
package_repro_match
package_repro_mismatch
package_signing_wait
package_failures
```

---

# 242. Format Metrics

Low-cardinality by format/platform.

---

# 243. Tracing

```text
package.plan
package.build
package.validate
package.reproduce
package.sign.request
package.notarize
```

---

# 244. Doctor

```text
forgeyard package doctor
```

Checks:

```text
packaging toolchains
platform capability
signing integration availability
validation tooling
```

---

# 245. Platform Doctor

Examples:

```text
dpkg/rpm tooling
Windows installer backend
Xcode packaging tools
Android SDK/build-tools
OCI builder
```

---

# 246. Testkit

```text
forgeyard-package-testkit/src/
├── lib.rs
├── plan.rs
├── layout.rs
├── artifact.rs
├── validator.rs
├── reproducibility.rs
├── lineage.rs
└── assertions.rs
```

---

# 247. Unit Tests

Test:

```text
PackageSpecId
layout collisions
path safety
metadata normalization
```

---

# 248. Adapter Conformance

Every adapter should support standardized tests:

```text
requirements
build
inspect
validate
lineage
```

---

# 249. Archive Tests

Bit-for-bit reproducible archive from same inputs.

---

# 250. DEB Tests

Validate package metadata/files.

---

# 251. RPM Tests

Same.

---

# 252. Windows Tests

Install/uninstall in clean VM.

---

# 253. macOS Tests

Bundle/package validation on real macOS.

---

# 254. Android Tests

APK/AAB validation and install test where applicable.

---

# 255. OCI Tests

Manifest/config/layer digest validation.

---

# 256. Signing Isolation Tests

Packaging runner cannot access private production key.

---

# 257. Package Rebuild Guard Test

Packaging stage cannot invoke source build action not declared as packaging tool transformation.

---

# 258. Input Identity Test

Changing input artifact digest changes PackageDerivationId.

---

# 259. Metadata Identity Test

Changing embedded package description/version changes PackageSpecId.

---

# 260. Toolchain Identity Test

Changing packaging toolchain changes derivation identity.

---

# 261. Timestamp Test

Controlled timestamps produce deterministic unsigned package where supported.

---

# 262. File Order Test

Host directory traversal order does not change package digest.

---

# 263. Ownership Test

Host UID/GID does not leak into package unexpectedly.

---

# 264. Permission Test

Package modes follow manifest.

---

# 265. Symlink Test

Safe and deterministic according to format.

---

# 266. Path Traversal Test

Malicious destination rejected.

---

# 267. Huge Package Test

Stream packaging without loading all files into RAM.

---

# 268. Repro Test

Independent runner produces identical/accepted normalized package.

---

# 269. Signed Lineage Test

Signing modifies bytes -> new object.

---

# 270. Promotion Guard Test

Release later uses signed object directly.

---

# 271. Fuzzing

Fuzz:

```text
package manifest parser
layout
archive path handling
package metadata decoder
adapter inspection parsers
```

---

# 272. Failure Injection

```text
disk full
packaging tool crash
CAS read fail
validation tool fail
signer unavailable
notarization timeout
```

---

# 273. Scale Tests

Many package targets.

---

# 274. Large File Count

Hundreds of thousands of files.

---

# 275. Memory Bound

Streaming manifest/package builder.

---

# 276. Build Artifact Closure

Package inputs can reference directory tree CAS closure.

---

# 277. Sparse Files

Handle according to format capability.

---

# 278. Extended Attributes

Explicit policy.

---

# 279. macOS xattrs

Important for bundle/signing semantics.

---

# 280. Linux capabilities/xattrs

Package adapter can represent when needed.

---

# 281. ACLs

Only where target format supports and explicitly declared.

---

# 282. Device Nodes

Forbidden for normal application packages unless special privileged package policy.

---

# 283. Setuid Bits

Denied by default.

---

# 284. Privileged Package Files

Require explicit security policy.

---

# 285. Installer Scripts

Treated as executable code.

Need explicit source/digest/review.

---

# 286. Script Environment

Package managers control runtime environment; document carefully.

---

# 287. Safe Default

Prefer declarative package metadata over scripts.

---

# 288. Package Template

Reusable package definitions can use typed templates from pipeline/config model.

---

# 289. Template Inputs

Typed.

---

# 290. Template Resolution

Before PackageSpecId.

---

# 291. Remote Package Template

Immutable source/digest only.

---

# 292. Package Config Version

```rust
pub struct PackageSchemaVersion(u16);
```

---

# 293. Package IR Version

Could be independent:

```rust
pub struct PackageIrVersion(u16);
```

---

# 294. Raw Config vs Canonical Plan

Same compiler-like approach as Pipeline.

---

# 295. Package Parse Flow

```text
RON
  ↓
parse
  ↓
validate
  ↓
normalize
  ↓
PackagePlan
  ↓
PackageSpecId
```

---

# 296. Dry Run

Plan outputs:

```text
target
runner requirements
input artifacts
expected generated files
signing requirements
```

---

# 297. Package DAG

One release can have dependency graph:

```text
build
  ↓
package unsigned
  ↓
validate
  ↓
sign
  ↓
notarize
```

---

# 298. Use Normal Pipeline/Run System

Do not implement separate package scheduler.

---

# 299. Package Nodes

Pipeline package steps lower into normal jobs.

---

# 300. Package State Metadata

Convenience/read model around normal Run/Job results.

---

# 301. Release Candidate Inputs

Future release subsystem consumes only package artifacts meeting policy.

---

# 302. Package Readiness

```rust
pub enum PackageReadiness {
    Incomplete,
    Built,
    Verified,
    ReleaseReady,
}
```

---

# 303. `ReleaseReady`

Means required package-level evidence/signing complete.

It does not mean promoted.

---

# 304. Production Readiness Gates

Do not call packaging production-ready until:

```text
normalized package model stable
PackageSpecId deterministic
path/layout safety tested
archive adapter reproducible
Linux package adapters validated
Windows packaging validated on Windows
Apple packaging validated on macOS
Android package flow validated
OCI packaging validated
package provenance integrated
signing handoff separated
install/smoke test path exists
lineage immutable
```

---

# 305. Implementation Phase 1 — Core Package Model

Implement:

```text
PackageId
PackageFormat
PackagePlan
PackageLayout
PackageSpecId
```

---

# 306. Phase 2 — Generic Archives

```text
tar
tar.zst
zip
```

---

# 307. Phase 3 — Package Derivation/Reproducibility

Hermetic packaging and comparison.

---

# 308. Phase 4 — Linux

DEB/RPM first.

AppImage/Flatpak/Snap later.

---

# 309. Phase 5 — Windows

MSI/MSIX/installer adapters.

---

# 310. Phase 6 — Apple

macOS app/pkg/dmg + signing handoff.

---

# 311. Phase 7 — Android/iOS

Mobile package metadata and signing handoff.

---

# 312. Phase 8 — OCI

Image/index packaging.

---

# 313. Phase 9 — Validation / Install Tests

Disposable environments/device lab.

---

# 314. Phase 10 — Evidence

Final package SBOM/provenance/validation.

---

# 315. Phase 11 — Publishing Descriptors

Prepare release destinations.

---

# 316. Phase 12 — Hardening

Large packages, fuzzing, cross-version adapters, failure injection.

---

# 317. Acceptance Tests

1. Packaging accepts immutable build artifact refs only.
2. Package plan cannot silently compile source.
3. PackageSpecId is deterministic.
4. Destination path traversal is rejected.
5. Host UID/GID do not leak unintentionally.
6. Host file ordering does not change deterministic archive.
7. Controlled timestamps are applied.
8. Packaging toolchain identity is recorded.
9. Changing package metadata changes PackageSpecId.
10. Changing input artifact changes PackageDerivationId.
11. Archive packaging is bit-for-bit reproducible where expected.
12. DEB validates on supported distro tooling.
13. RPM validates on supported distro tooling.
14. MSI/MSIX production validation uses Windows.
15. Apple production packaging uses macOS tooling where required.
16. Android release packaging keeps signing key outside general runner.
17. OCI base image is digest-pinned in strict mode.
18. Package output bytes are stored in CAS.
19. Package metadata remains in SQL.
20. Package provenance binds input/output digests.
21. Final package SBOM binds exact package digest.
22. Validation evidence binds exact package digest.
23. Signing never mutates an existing CAS object.
24. Signed package becomes new artifact if bytes change.
25. Notarization/stapling byte changes create new object.
26. General packaging worker cannot access production signing key.
27. Package install test runs in disposable environment.
28. Service package does not embed runtime secrets.
29. Installer scripts require explicit declaration.
30. Release publish descriptor has no authority to publish by itself.
31. Same package model works standalone/distributed.
32. Package adapters declare platform capabilities to scheduler.
33. Package cache key includes adapter/toolchain semantics.
34. Evidence bundle can prove complete package lineage.
35. Forgeyard's own Linux/Windows/macOS/mobile/container packages use this subsystem.

---

# 318. Architectural Invariants

1. packaging is a transformation, not an uncontrolled rebuild;
2. inputs are immutable CAS/artifact refs;
3. package plan is canonical and deterministic;
4. PackageSpecId identifies semantic package intent;
5. PackageDerivationId identifies actual packaging derivation;
6. package bytes live in CAS;
7. metadata lives in store;
8. every byte-changing stage creates new object;
9. signing is separate from general packaging;
10. production signing keys never reach general runners;
11. package adapters are format-specific and isolated;
12. common package model remains provider-neutral;
13. native packages are validated on appropriate real platform;
14. packaging toolchain is part of provenance/derivation;
15. timestamps/order/ownership normalized where possible;
16. path traversal is impossible by construction/validation;
17. package scripts are explicit high-risk inputs;
18. package configuration never contains production secrets;
19. package verification does not mutate bytes;
20. release promotion does not rebuild package;
21. publishing authority belongs to release subsystem;
22. final package has digest-bound evidence;
23. package lineage remains immutable;
24. OCI mutable tags never replace digest identity;
25. package sets bind consistent version/source policy;
26. cross-platform capability requirements are explicit;
27. cache hits never bypass verification policy;
28. standalone/distributed share package semantics;
29. package adapters do not leak platform dependencies into core;
30. Forgeyard dogfoods its packaging system.

---

# 319. Final Target Architecture

```text
                    Build Artifact(s)
                          │
                          ▼
                     PackagePlan
                          │
                          ▼
                    PackageSpecId
                          │
                          ▼
                 Packaging Derivation
                          │
                          ▼
                  Unsigned Package
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
        Validate        SBOM        Provenance
            │             │             │
            └─────────────┼─────────────┘
                          ▼
                     Policy Check
                          │
                          ▼
                  Signing Handoff
                          │
                          ▼
                   Signed Package
                          │
                    optional notarize
                          │
                          ▼
                  Release-Ready Artifact
                          │
                          ▼
                      Release
```

---

# 320. Final Architectural Position

Packaging identity:

```text
immutable input artifact digests
+
canonical package plan
+
packaging toolchain identity
+
controlled environment
  ↓
PackageDerivationId
  ↓
package bytes
  ↓
CAS digest
```

Signing lineage:

```text
Unsigned Package A
  ↓
restricted signing
  ↓
Signed Package B
  ↓
optional notarization/stapling
  ↓
Package C
```

Release handoff:

```text
exact release-ready package digest
+
SBOM
+
provenance
+
validation
+
signature evidence
  ↓
Release subsystem
```

The key guarantee is:

> **Forgeyard packaging never obscures where distribution bytes came from. Every installer, archive, mobile bundle, system package, or container image is a deterministic, inspectable transformation of exact immutable build outputs, with explicit toolchains, validation, evidence, signing lineage, and no hidden rebuild step.**

---

# 321. New-Repository Sequence

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
