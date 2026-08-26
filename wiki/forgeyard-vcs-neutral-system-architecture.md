# Forgeyard VCS-Neutral Source Control System & Architecture

**Document type:** Core Forgeyard System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** VCS-neutral source acquisition, revision graph normalization, canonical source snapshots, provenance, change-impact analysis, repository events, and FOSS VCS adapters  
**Implementation direction:** Pure Rust Forgeyard core with adapter boundaries for Git, Mercurial, Fossil, Breezy, Darcs, Jujutsu, Pijul, local source trees, archives, and future SCM systems  
**Status:** Target production architecture  
**Core principle:** Forgeyard builds immutable source snapshots, not “Git commits.”

---

# 1. Purpose

Forgeyard must support repositories without making Git's object model the internal definition of source control.

Git should be the deepest first implementation because of ecosystem reach, but the internal domain model must also be able to represent:

- Mercurial changesets, bookmarks, named branches, tags, phases;
- Fossil immutable artifacts and check-in baselines;
- Breezy revisions, branches, repositories, working trees;
- Jujutsu commits and stable change IDs across rewrites;
- Darcs patch-oriented history;
- Pijul changes/patches and channels;
- local working trees;
- source archives;
- future open-source VCS implementations;
- proprietary adapters later without contaminating Forgeyard core.

Forgeyard's fundamental source identity must therefore be:

```text
Canonical source tree
        ↓
Forgeyard canonicalization
        ↓
BLAKE3 digest
        ↓
SourceSnapshotId
```

rather than:

```text
Git SHA
```

or:

```text
Mercurial node ID
```

or:

```text
Fossil artifact ID
```

A VCS-native revision identifier is provenance and navigation identity.

`SourceSnapshotId` is Forgeyard build-content identity.

---

# 2. Central Architectural Rule

> **Forgeyard never builds “a Git commit,” “a Mercurial changeset,” or “a Pijul channel.” Forgeyard resolves those VCS-native references into a canonical immutable source snapshot and builds that snapshot.**

Therefore:

```text
VCS-native reference
      ↓
VCS adapter
      ↓
ResolvedRevision
      ↓
CanonicalSourceSnapshot
      ↓
SourceSnapshotId
      ↓
Pipeline / Derivation
```

---

# 3. Why VCS Neutrality Matters

VCS neutrality provides:

1. repository migration without changing Forgeyard's build identity;
2. Git/Mercurial/Fossil/Jujutsu support through one pipeline interface;
3. deterministic source identity independent of VCS hash algorithm;
4. source equivalence detection across mirrors;
5. safer reproducibility;
6. support for patch-oriented systems without pretending patches are Git commits;
7. source archives/local trees as legitimate first-class sources;
8. enterprise SCM adapters later;
9. SCM-independent provenance;
10. clean architecture boundaries.

---

# 4. Fundamental Separation

Forgeyard separates four concepts that are often incorrectly collapsed:

```text
Repository identity
Revision identity
Change identity
Source snapshot identity
```

They are not interchangeable.

---

# 5. Repository Identity

A repository identifies a logical source repository.

```rust
pub struct RepositoryId(Digest);
```

Repository identity may derive from normalized configured origin identity plus tenant/project binding rather than directly from remote URL text.

A repository can have multiple remotes/mirrors.

---

# 6. Revision Identity

A revision is a backend-native historical state/reference.

```rust
pub struct RevisionId {
    pub vcs: VcsKind,
    pub native: NativeRevisionId,
}
```

Examples:

```text
Git commit OID
Mercurial changeset/node
Fossil check-in artifact ID
Breezy revision ID
Jujutsu commit ID
Darcs repository context/patch state
Pijul channel state/change context
```

Do not force all backend IDs into SHA-shaped types.

---

# 7. Change Identity

Some VCS systems distinguish logical changes from immutable revisions.

Jujutsu is the clearest example: a commit can be rewritten to a new commit ID while retaining a stable logical change identity.

Forgeyard therefore supports:

```rust
pub struct ChangeId {
    pub vcs: VcsKind,
    pub native: String,
}
```

but it is optional.

```rust
pub enum ChangeIdentity {
    Unsupported,
    Native(ChangeId),
    Derived(Digest),
}
```

---

# 8. Source Snapshot Identity

The actual source tree to build:

```rust
pub struct SourceSnapshotId(Digest);
```

This is independent from repository history representation.

Two different revisions can produce the same snapshot:

```text
Git commit A ─────┐
Mercurial rev B ──┼──> same canonical tree ──> SourceSnapshotId X
Archive C ────────┘
```

---

# 9. Internal Truth Hierarchy

For build correctness:

```text
SourceSnapshotId
    >
RevisionId
    >
branch/bookmark/channel/tag name
```

Names are mutable selectors.

Revision IDs are historical identities.

Source snapshot IDs are build-content identities.

---

# 10. Suggested Workspace

```text
crates/
├── vcs/
│   ├── forgeyard-vcs/
│   ├── forgeyard-vcs-model/
│   ├── forgeyard-vcs-source/
│   ├── forgeyard-vcs-snapshot/
│   ├── forgeyard-vcs-canonical/
│   ├── forgeyard-vcs-graph/
│   ├── forgeyard-vcs-diff/
│   ├── forgeyard-vcs-provenance/
│   ├── forgeyard-vcs-signature/
│   ├── forgeyard-vcs-auth/
│   ├── forgeyard-vcs-cache/
│   ├── forgeyard-vcs-events/
│   ├── forgeyard-vcs-git/
│   ├── forgeyard-vcs-mercurial/
│   ├── forgeyard-vcs-fossil/
│   ├── forgeyard-vcs-breezy/
│   ├── forgeyard-vcs-jujutsu/
│   ├── forgeyard-vcs-darcs/
│   ├── forgeyard-vcs-pijul/
│   ├── forgeyard-vcs-local/
│   └── forgeyard-vcs-archive/
```

Physical crate count may be consolidated later.

Capability boundaries are the architectural requirement.

---

# 11. Dependency Direction

```text
forgeyard-core
      ↑
forgeyard-vcs-model
      ↑
forgeyard-vcs
      ↑
VCS adapters
      ↑
daemon / scheduler / importer / UI
```

Core must never depend on:

```text
libgit-specific types
Mercurial-specific node IDs
Fossil manifest structs
Jujutsu internals
```

---

# 12. VCS Kinds

```rust
pub enum VcsKind {
    Git,
    Mercurial,
    Fossil,
    Breezy,
    Jujutsu,
    Darcs,
    Pijul,
    LocalTree,
    Archive,
    External(String),
}
```

Do not add a variant unless Forgeyard has a defined adapter contract.

---

# 13. Core Backend Trait

```rust
#[async_trait]
pub trait VcsBackend: Send + Sync {
    fn kind(&self) -> VcsKind;

    async fn detect(
        &self,
        location: &SourceLocation,
    ) -> Result<VcsDetection, VcsError>;

    async fn discover(
        &self,
        request: &RepositoryRequest,
    ) -> Result<RepositoryDescriptor, VcsError>;

    async fn resolve_revision(
        &self,
        repo: &RepositoryHandle,
        spec: &RevisionSpec,
    ) -> Result<ResolvedRevision, VcsError>;

    async fn materialize_snapshot(
        &self,
        repo: &RepositoryHandle,
        revision: &ResolvedRevision,
        policy: &SnapshotPolicy,
    ) -> Result<MaterializedSource, VcsError>;

    async fn graph(
        &self,
        repo: &RepositoryHandle,
        query: &GraphQuery,
    ) -> Result<RevisionGraphSlice, VcsError>;

    async fn changed_paths(
        &self,
        repo: &RepositoryHandle,
        from: &ResolvedRevision,
        to: &ResolvedRevision,
    ) -> Result<ChangeSet, VcsError>;

    async fn references(
        &self,
        repo: &RepositoryHandle,
    ) -> Result<Vec<VcsReference>, VcsError>;

    async fn provenance(
        &self,
        repo: &RepositoryHandle,
        revision: &ResolvedRevision,
    ) -> Result<NativeVcsProvenance, VcsError>;
}
```

---

# 14. Capability-Based Backend Design

Not every VCS supports every operation with identical semantics.

Use explicit capabilities:

```rust
bitflags::bitflags! {
    pub struct VcsCapabilities: u64 {
        const REVISION_DAG       = 1 << 0;
        const LOGICAL_CHANGE_ID  = 1 << 1;
        const MOVABLE_REFS       = 1 << 2;
        const TAGS               = 1 << 3;
        const SIGNATURES         = 1 << 4;
        const PATCH_MODEL        = 1 << 5;
        const SHALLOW_FETCH      = 1 << 6;
        const PARTIAL_FETCH      = 1 << 7;
        const SPARSE_WORKTREE    = 1 << 8;
        const SUBREPOSITORIES    = 1 << 9;
        const NATIVE_DIFF        = 1 << 10;
        const SERVER_EVENTS      = 1 << 11;
    }
}
```

Forgeyard MUST NOT emulate unsupported semantics and pretend they are native.

---

# 15. Repository Descriptor

```rust
pub struct RepositoryDescriptor {
    pub id: RepositoryId,
    pub vcs: VcsKind,
    pub canonical_origin: Option<RepositoryOrigin>,
    pub mirrors: Vec<RepositoryOrigin>,
    pub capabilities: VcsCapabilities,
    pub trust: RepositoryTrust,
}
```

---

# 16. Repository Origins

```rust
pub enum RepositoryOrigin {
    Https(SecretSafeUrl),
    Ssh(SshRepositoryRef),
    File(VirtualPath),
    ForgeyardMirror(MirrorId),
    Custom(String),
}
```

Credentials must not be embedded in serialized origin strings.

---

# 17. Revision Specifications

User/pipeline input may request:

```rust
pub enum RevisionSpec {
    Exact(NativeRevisionId),
    Ref(ReferenceName),
    Tag(TagName),
    Branch(BranchName),
    Bookmark(BookmarkName),
    Channel(ChannelName),
    Change(ChangeId),
    Default,
    WorkingCopy,
    BackendNative(String),
}
```

Resolution converts mutable selectors into an immutable `ResolvedRevision`.

---

# 18. ResolvedRevision

```rust
pub struct ResolvedRevision {
    pub repository: RepositoryId,
    pub vcs: VcsKind,
    pub revision_id: NativeRevisionId,
    pub change_id: Option<ChangeId>,
    pub parents: Vec<NativeRevisionId>,
    pub tree_hint: Option<NativeTreeId>,
    pub timestamp: Option<VcsTimestamp>,
    pub author: Option<VcsIdentity>,
    pub committer: Option<VcsIdentity>,
    pub native_metadata: NativeMetadata,
}
```

---

# 19. Native Metadata

Do not use an unstructured JSON dump.

Use a typed extension envelope:

```rust
pub enum NativeMetadata {
    Git(GitRevisionMetadata),
    Mercurial(MercurialRevisionMetadata),
    Fossil(FossilRevisionMetadata),
    Breezy(BreezyRevisionMetadata),
    Jujutsu(JujutsuRevisionMetadata),
    Darcs(DarcsRevisionMetadata),
    Pijul(PijulRevisionMetadata),
    Local(LocalRevisionMetadata),
    Archive(ArchiveRevisionMetadata),
}
```

Serialize internally using versioned Postcard where possible.

Use RON for diagnostics/configuration.

---

# 20. Normalized Revision Graph

Forgeyard can represent a generic DAG where the backend genuinely has revision ancestry.

```rust
pub struct RevisionNode {
    pub revision: RevisionKey,
    pub parents: Vec<RevisionKey>,
    pub change: Option<ChangeId>,
    pub snapshot: Option<SourceSnapshotId>,
}
```

But this is only one normalized view.

Patch-centric systems must retain their patch semantics separately.

---

# 21. No Lowest-Common-Denominator Trap

Bad abstraction:

```text
everything = commit + branch
```

Correct abstraction:

```text
common source-resolution contract
+
common snapshot model
+
optional revision DAG
+
optional logical change identity
+
optional patch/change algebra
+
backend-native extension data
```

---

# 22. Canonical Source Snapshot

```rust
pub struct CanonicalSourceSnapshot {
    pub id: SourceSnapshotId,
    pub root: TreeObjectId,
    pub entries: Vec<CanonicalTreeEntry>,
    pub provenance: SourceProvenanceId,
}
```

---

# 23. Canonical Tree Entry

```rust
pub struct CanonicalTreeEntry {
    pub path: CanonicalRepoPath,
    pub kind: SourceEntryKind,
    pub content: Option<BlobId>,
    pub executable: bool,
    pub symlink_target: Option<Vec<u8>>,
}
```

---

# 24. Source Entry Kinds

```rust
pub enum SourceEntryKind {
    RegularFile,
    Directory,
    Symlink,
    Subrepository,
}
```

Platform-specific exotic filesystem state should be rejected or normalized through explicit policy.

---

# 25. Canonical Path Rules

Forgeyard canonicalization MUST define:

1. path separator `/`;
2. no `.` components;
3. no `..`;
4. no absolute paths;
5. deterministic byte/string normalization policy;
6. duplicate/case-collision detection;
7. deterministic sorting;
8. explicit Unicode policy;
9. symlink target preservation;
10. executable-bit preservation where source semantics expose it.

---

# 26. Case-Collision Safety

Repositories may contain:

```text
Foo.rs
foo.rs
```

which are distinct on some systems but collide on others.

Snapshot creation records collision risk.

Scheduler refuses incompatible targets unless policy allows.

---

# 27. Unicode Paths

Do not silently Unicode-normalize paths in a way that changes repository semantics.

Canonical representation should preserve backend path identity as safely as possible while defining deterministic byte serialization.

---

# 28. File Content Identity

```rust
pub struct BlobId(Digest);
```

Default internal digest:

```text
BLAKE3
```

Optional aliases:

```text
SHA-256
Git blob OID
Fossil artifact hash
backend-native hash
```

---

# 29. Tree Hash

Canonical tree digest:

```text
TreeObjectId =
BLAKE3(
  schema_version
  +
  sorted canonical entries
)
```

---

# 30. Snapshot Hash

```text
SourceSnapshotId =
BLAKE3(
  snapshot_schema
  +
  root_tree_id
  +
  snapshot-policy identity
)
```

Do not include mutable branch/tag names.

Do not include commit message.

Do not include author timestamp unless a build policy explicitly adds it as source metadata input.

---

# 31. Source Metadata vs Source Bytes

Separate:

```text
source tree bytes
```

from:

```text
VCS metadata
```

A build may choose to embed:

```text
revision
version
commit date
```

but those become explicit derivation inputs.

---

# 32. VCS Metadata Injection

If project wants version metadata:

```rust
pub struct SourceMetadataInput {
    pub revision: Option<String>,
    pub change_id: Option<String>,
    pub source_date: Option<Timestamp>,
    pub dirty: DirtyState,
}
```

Forgeyard supplies it deterministically.

Build scripts should not invoke Git/Hg/JJ dynamically.

---

# 33. Dirty Working Tree

Dirty source is not "unversioned."

Forgeyard snapshots it explicitly.

```rust
pub enum DirtyState {
    Clean,
    Modified,
    UntrackedIncluded,
    UntrackedExcluded,
}
```

---

# 34. Dirty Snapshot Identity

```text
clean base revision
+
working-tree modifications
+
selected untracked files
  ↓
canonical snapshot
  ↓
new SourceSnapshotId
```

The base revision remains provenance.

---

# 35. Dirty Release Policy

Recommended:

```text
release:
    dirty source denied

development:
    dirty source allowed
    exact snapshot hashed
```

---

# 36. Ignore Rules

Backend ignore rules determine default working-tree discovery.

But snapshot policy MUST record whether:

```text
ignored files included
untracked files included
generated files included
```

---

# 37. Source Snapshot Policy

```rust
pub struct SnapshotPolicy {
    pub include_untracked: bool,
    pub include_ignored: bool,
    pub nested_repo_policy: NestedRepoPolicy,
    pub symlink_policy: SymlinkPolicy,
    pub large_file_policy: LargeFilePolicy,
}
```

---

# 38. Nested Repositories

Nested repositories are dangerous if silently flattened.

Policy:

```rust
pub enum NestedRepoPolicy {
    Reject,
    SnapshotAsFiles,
    ResolveAsSubrepository,
}
```

---

# 39. Submodules / Subrepositories

Forgeyard uses a generic:

```rust
pub struct SubrepositoryRef {
    pub path: CanonicalRepoPath,
    pub backend: Option<VcsKind>,
    pub origin: RepositoryOrigin,
    pub revision: RevisionSpec,
}
```

Each nested source is independently resolved and snapshot-hashed.

---

# 40. Composite Source Snapshot

For projects with subrepositories:

```text
root snapshot
+
child snapshot IDs
+
mount paths
=
CompositeSourceId
```

---

# 41. Git Submodules

Git adapter maps `.gitmodules` + gitlink entries into `SubrepositoryRef`.

Do not merely execute recursive checkout and hide child provenance.

---

# 42. Mercurial Subrepositories

Mercurial adapter maps supported nested-repository semantics into the same generic model where possible.

Backend-native details remain attached.

---

# 43. Large File Extensions

Systems such as Git LFS-like extensions alter file materialization.

Forgeyard MUST hash final build-visible bytes.

Pointer file identity alone is insufficient if actual content is required.

---

# 44. Materialized Source

```rust
pub struct MaterializedSource {
    pub snapshot: SourceSnapshotId,
    pub path: SandboxSourcePath,
    pub provenance: SourceProvenanceId,
}
```

Materialization is disposable.

Snapshot identity is persistent.

---

# 45. CAS Integration

Source objects use existing Forgeyard CAS:

```text
VCS fetch
  ↓
source blobs
  ↓
canonical trees
  ↓
SourceSnapshot
```

Do not create a second VCS-specific permanent blob store.

---

# 46. VCS Native Cache vs Forgeyard CAS

Separate:

```text
Git object database / hg store / Fossil repo / etc.
        =
fetch/navigation cache

Forgeyard CAS
        =
canonical build source objects
```

---

# 47. Fetch Lifecycle

```text
RepositoryRequest
   ↓
resolve origin/auth
   ↓
backend fetch
   ↓
resolve revision
   ↓
verify source
   ↓
canonicalize tree
   ↓
store snapshot
```

---

# 48. Offline Build Boundary

VCS/network is used before realization.

Build runners receive:

```text
SourceSnapshotId
```

and fetch source objects from Forgeyard CAS.

They do not need Git/Hg/Fossil network access.

---

# 49. Repository Authentication

Use generic secret references:

```rust
pub enum VcsCredentialRef {
    SshKey(SecretRef),
    HttpsToken(SecretRef),
    Basic(SecretRef),
    ClientCertificate(SecretRef),
    BackendSpecific(SecretRef),
}
```

---

# 50. Credentials Never Enter Snapshot Identity

Source bytes may be identical regardless of which credential fetched them.

Credentials are provenance/audit context, not source content.

---

# 51. Host Key / TLS Verification

Repository policy controls:

```text
SSH known-host verification
TLS certificate verification
custom CA
certificate pinning where required
```

No "accept everything" release default.

---

# 52. Repository Trust

```rust
pub enum RepositoryTrust {
    Untrusted,
    Authenticated,
    OrganizationApproved,
    Mirrored,
    Revoked,
}
```

---

# 53. Source Trust

Separate source provenance trust:

```rust
pub enum SourceTrust {
    Unknown,
    HashVerified,
    SignatureVerified,
    PolicyApproved,
    Revoked,
}
```

---

# 54. Signed Revisions / Tags

Different VCS systems have different signing models.

Forgeyard normalizes verification result:

```rust
pub struct SignatureEvidence {
    pub subject: SignatureSubject,
    pub signer: SignerIdentity,
    pub algorithm: SignatureAlgorithm,
    pub status: SignatureStatus,
    pub backend_evidence: NativeSignatureEvidence,
}
```

Do not pretend signatures mean the same thing across all VCS.

---

# 55. Signature Policy

Examples:

```text
release tag must be verified
revision must be signed by approved key
unsigned development commit allowed
```

---

# 56. Provenance Model

```rust
pub struct SourceProvenance {
    pub repository: RepositoryId,
    pub vcs: VcsKind,
    pub revision: Option<NativeRevisionId>,
    pub change_id: Option<ChangeId>,
    pub resolved_from: RevisionSpec,
    pub snapshot: SourceSnapshotId,
    pub dirty: DirtyState,
    pub signatures: Vec<SignatureEvidence>,
    pub nested_sources: Vec<SourceProvenanceId>,
}
```

---

# 57. Provenance Invariant

It must be possible to answer:

```text
Which logical repository?
Which VCS?
Which exact native revision/state?
Which mutable selector was requested?
Which immutable snapshot was built?
Was it dirty?
Were nested repositories present?
Which signature evidence was verified?
```

---

# 58. Source Equivalence

If:

```text
Git rev A -> Snapshot X
Mercurial rev B -> Snapshot X
```

Forgeyard can record:

```text
SourceEquivalent(A, B)
```

without claiming histories are equivalent.

---

# 59. Mirror Migration

Migration flow:

```text
old VCS
  ↓
snapshot X
new VCS
  ↓
snapshot X
```

A migration check can prove source-state equivalence at selected milestones.

---

# 60. History Migration Is Stronger Than Snapshot Migration

Forgeyard distinguishes:

```text
snapshot equivalence
history mapping
semantic change mapping
```

Only snapshot equivalence is universally portable.

---

# 61. Revision Graph API

```rust
pub struct RevisionGraphSlice {
    pub nodes: Vec<RevisionNode>,
    pub roots: Vec<RevisionKey>,
    pub truncated: bool,
}
```

---

# 62. Graph Queries

```rust
pub enum GraphQuery {
    Ancestors { revision: RevisionSpec, limit: usize },
    Descendants { revision: RevisionSpec, limit: usize },
    Between { from: RevisionSpec, to: RevisionSpec },
    Heads,
    BackendNative(String),
}
```

---

# 63. Patch-Oriented History

Darcs and Pijul should not be reduced exclusively to revision DAG edges.

Expose optional:

```rust
pub trait PatchSemanticBackend {
    async fn changes(
        &self,
        query: PatchQuery,
    ) -> Result<Vec<NativePatch>, VcsError>;
}
```

---

# 64. Logical Change Backend

For systems such as Jujutsu:

```rust
pub trait LogicalChangeBackend {
    async fn resolve_change(
        &self,
        id: &ChangeId,
    ) -> Result<Vec<ResolvedRevision>, VcsError>;
}
```

Supports divergent/rewrite scenarios.

---

# 65. Reference Model

```rust
pub enum VcsReferenceKind {
    Branch,
    Bookmark,
    Tag,
    Channel,
    Head,
    NamedBranch,
    BackendNative(String),
}
```

---

# 66. Reference Object

```rust
pub struct VcsReference {
    pub name: String,
    pub kind: VcsReferenceKind,
    pub target: RevisionTarget,
    pub mutable: bool,
}
```

---

# 67. Mutable Ref Rule

Never use:

```text
main
master
trunk
default
stable
bookmark
channel
```

as immutable build identity.

Always resolve before creating derivation.

---

# 68. Reference Race Protection

Flow:

```text
resolve main -> R1
fetch/materialize R1
record R1
build R1
```

If `main` moves to R2 later, active run remains R1.

---

# 69. Webhook/Event Race

Webhook says:

```text
ref main updated to revision R
```

Forgeyard must prefer event-supplied exact revision if trustworthy, then verify ref relationship.

Do not re-resolve `main` hours later and accidentally build a newer revision.

---

# 70. ChangeSet

Generic changed paths:

```rust
pub struct ChangeSet {
    pub entries: Vec<PathChange>,
    pub semantics: ChangeSemantics,
}
```

---

# 71. Path Changes

```rust
pub enum PathChange {
    Added(CanonicalRepoPath),
    Modified(CanonicalRepoPath),
    Deleted(CanonicalRepoPath),
    Renamed {
        from: CanonicalRepoPath,
        to: CanonicalRepoPath,
        confidence: RenameConfidence,
    },
    TypeChanged(CanonicalRepoPath),
}
```

---

# 72. Rename Semantics

Rename detection varies by backend.

Do not treat it as universal truth.

Use:

```rust
pub enum RenameConfidence {
    Native,
    InferredHigh,
    InferredLow,
    Unknown,
}
```

---

# 73. Change Semantics

```rust
pub enum ChangeSemantics {
    NativeDiff,
    SnapshotDiff,
    PatchSemantic,
    Approximate,
}
```

---

# 74. Canonical Snapshot Diff

Forgeyard always has fallback:

```text
Snapshot A
  vs
Snapshot B
```

to determine content-level path differences.

This is VCS-neutral and ideal for affected-build analysis.

---

# 75. Native Diff vs Snapshot Diff

Native VCS diff may preserve:

```text
rename
copy
patch semantics
conflict semantics
```

Snapshot diff preserves:

```text
build-visible before/after content
```

Both can be useful.

---

# 76. Change Impact Analysis

Pipeline:

```text
VCS event/revisions
   ↓
snapshot diff
   ↓
changed paths
   ↓
workspace/project graph
   ↓
reverse dependencies
   ↓
affected derivations
```

---

# 77. Safe-Superset Rule

If VCS adapter cannot determine a precise change:

```text
run broader set
```

Never skip required work because of uncertain history semantics.

---

# 78. Monorepo Integration

For Forgeyard itself:

```text
changed path
  ↓
Rust workspace crate ownership
  ↓
dependency reverse graph
  ↓
affected tests/builds
```

VCS layer only reports source change.

Language ecosystem layer understands build impact.

---

# 79. Ignore/Generated Boundaries

Change-impact analysis uses canonical source snapshot.

Generated files outside source are not VCS changes unless committed/included.

---

# 80. Git Adapter

Suggested crate:

```text
forgeyard-vcs-git
```

Responsibilities:

```text
repository discovery
refs/tags
commit resolution
commit ancestry
tree materialization
worktree snapshot
submodules
signed commit/tag evidence
diff/change extraction
partial/shallow strategies
remote/auth
```

---

# 81. Git Internal Mapping

Conceptually:

```text
Git commit
  ↓
Git tree
  ↓
Forgeyard canonical tree
  ↓
SourceSnapshotId
```

Git commit OID remains provenance.

---

# 82. Git Hash Independence

Forgeyard should support Git repositories regardless of whether repository object identity is SHA-1-era or SHA-256-capable.

Internal CAS identity remains independent.

---

# 83. Git Ref Mapping

```text
refs/heads/* -> Branch
refs/tags/*  -> Tag
other refs   -> BackendNative
```

---

# 84. Git Worktree/Index

For clean revision builds:

```text
do not rely on mutable index
```

For local dirty builds:

```text
base commit
+
index/worktree state according to snapshot policy
```

is canonicalized.

---

# 85. Git Submodules

Resolve each gitlink exactly.

Record child provenance.

---

# 86. Git Partial/Promisor Objects

Git adapter may use partial clone/promisor behavior as a fetch optimization.

Canonical snapshot must verify all build-needed objects are actually materialized.

---

# 87. Git Shallow Fetch

Shallow history is acceptable if operation does not require unavailable ancestry.

Capability response must indicate truncation.

---

# 88. Git Sparse Checkout

Sparse checkout is a materialization optimization.

Source identity must clearly represent whether build source intentionally excludes repository paths.

Default release source snapshot should represent the intended project source policy, not accidental developer sparse state.

---

# 89. Git Signed Objects

Verify through configured Git signing mechanisms/adapters and emit normalized signature evidence.

---

# 90. Mercurial Adapter

Suggested:

```text
forgeyard-vcs-mercurial
```

Responsibilities:

```text
changeset resolution
revsets
bookmarks
named branches
tags
phases
working-directory parents
subrepositories where used
diff/status
signature extension evidence where configured
```

---

# 91. Mercurial Mapping

```text
changeset/node -> RevisionId
bookmark -> Bookmark
named branch -> NamedBranch
tag -> Tag
phase -> NativeMetadata
```

---

# 92. Mercurial Branch Semantics

Mercurial named branches and bookmarks are not the same abstraction.

Do not map both to generic `Branch` and discard semantics.

---

# 93. Mercurial Revsets

Backend-native revision query support belongs behind:

```text
BackendNative
```

or typed Mercurial query API.

Do not force Git rev-list syntax into Mercurial.

---

# 94. Mercurial Phases

Phases are backend-native history/publication metadata.

They can influence policy but not source snapshot identity.

---

# 95. Mercurial Working Copy

Working directory may have one/two parents depending on operation state.

Dirty snapshot provenance records native parent context.

---

# 96. Fossil Adapter

Suggested:

```text
forgeyard-vcs-fossil
```

Fossil's immutable artifact/check-in/baseline model should be preserved.

---

# 97. Fossil Mapping

```text
check-in artifact -> RevisionId
baseline manifest -> source-state mapping
artifact IDs -> native content provenance
branch/tag metadata -> native/reference metadata
```

---

# 98. Fossil Artifacts

Forgeyard may reuse Fossil's immutable artifact IDs as aliases, but final source tree is still canonicalized into Forgeyard CAS.

---

# 99. Fossil Repository Extras

Fossil also contains non-source collaboration artifacts.

Forgeyard VCS source adapter should explicitly scope what is source-control relevant.

Do not inject wiki/ticket state into source snapshot identity.

---

# 100. Fossil Sync

Push/pull/sync semantics belong to backend adapter.

Forgeyard build source acquisition should resolve exact check-in before snapshot.

---

# 101. Breezy Adapter

Suggested:

```text
forgeyard-vcs-breezy
```

Preserve:

```text
working tree
branch
repository
revision
```

as distinct concepts.

---

# 102. Breezy Revision Mapping

Breezy revision IDs become native `RevisionId`.

Branches remain reference/history containers.

---

# 103. Breezy Revision Specifications

Backend can expose expressive revision specifications via backend-native query support.

Forgeyard still resolves them to exact revision before build.

---

# 104. Jujutsu Adapter

Suggested:

```text
forgeyard-vcs-jujutsu
```

Jujutsu requires explicit support for:

```text
commit ID
change ID
rewrites
divergent changes
bookmarks
Git-backed repositories
```

---

# 105. Jujutsu Commit ID

Map to:

```text
RevisionId
```

When using Git backend, native commit identity may correspond to Git commit identity, but Forgeyard still records VCS kind and backend provenance.

---

# 106. Jujutsu Change ID

Map to:

```text
ChangeId
```

This is exactly why Forgeyard separates `ChangeId` from `RevisionId`.

---

# 107. Rewritten Jujutsu Change

```text
Change C
  ↓ rewrite
Commit R1 -> Commit R2

ChangeId stays C
RevisionId changes
Snapshot may or may not change
```

Forgeyard can compare all three levels.

---

# 108. Jujutsu Divergence

A logical change may temporarily have multiple visible revisions.

API must return a set, not pretend one canonical commit always exists.

---

# 109. Jujutsu Git Backend

Do not bypass Jujutsu semantics by silently opening the backing Git repo unless adapter policy explicitly requests a Git-native view.

---

# 110. Darcs Adapter

Suggested:

```text
forgeyard-vcs-darcs
```

Darcs is patch-centric.

Forgeyard should not pretend Darcs history is fundamentally a Git-style commit DAG.

---

# 111. Darcs Mapping

Use:

```text
patch identity -> NativePatch
repository state/context -> ResolvedRevision-like source state
snapshot -> SourceSnapshotId
```

The source snapshot contract remains straightforward even when history model differs.

---

# 112. Darcs Patch Algebra

Patch ordering/commutation semantics remain backend-native.

Forgeyard does not reimplement Darcs patch theory.

---

# 113. Darcs Change Impact

Prefer native patch paths/content when trustworthy.

Fallback to snapshot diff.

---

# 114. Pijul Adapter

Suggested:

```text
forgeyard-vcs-pijul
```

Pijul is change/patch-centric with channels.

---

# 115. Pijul Mapping

```text
change/patch -> NativePatch / ChangeId-like native identity
channel -> Channel reference/state
materialized channel state -> SourceSnapshotId
```

---

# 116. Pijul Channels

Channel name is mutable/evolving repository state.

Never use it directly as build identity.

Resolve/materialize its current state first.

---

# 117. Pijul Conflicts

Pijul can represent conflicts as part of repository state.

Forgeyard snapshot adapter must have an explicit conflict policy:

```rust
pub enum ConflictPolicy {
    Reject,
    MaterializeBackendRepresentation,
    BackendResolvedOnly,
}
```

Release default:

```text
Reject unresolved conflicts
```

---

# 118. Local Tree Adapter

Suggested:

```text
forgeyard-vcs-local
```

This supports:

```text
source directory without VCS
generated source tree
vendored code
temporary project
```

---

# 119. Local Tree Identity

No revision ID exists.

```text
canonical tree -> SourceSnapshotId
```

Provenance records local origin and dirty/local state.

---

# 120. Archive Adapter

Suggested:

```text
forgeyard-vcs-archive
```

Support:

```text
tar
tar.gz
tar.zst
zip
```

as immutable source packages.

---

# 121. Archive Safety

Reject:

```text
path traversal
absolute paths
unsafe symlink escapes
duplicate normalized paths
archive bombs according to policy
```

---

# 122. Archive Identity

Record:

```text
archive byte digest
+
canonical unpacked tree digest
```

Both matter.

---

# 123. Future FOSS VCS Adapter Contract

Additional adapters can implement the same source contract if they can:

1. identify repository/source;
2. resolve requested source state;
3. materialize build-visible tree;
4. return native provenance;
5. optionally expose history/change capabilities.

---

# 124. No VCS Required for Build

Important:

```text
BuildRequest
    requires SourceSnapshotId
```

not:

```text
BuildRequest requires Git checkout
```

This keeps runners small and hermetic.

---

# 125. Repository Importer

External CI/import workflows:

```text
SCM event
  ↓
VCS backend detection
  ↓
exact revision
  ↓
snapshot
  ↓
Pipeline IR
```

---

# 126. SCM Provider vs VCS

Separate:

```text
VCS:
  Git / Mercurial / Fossil / ...

Forge/hosting provider:
  GitHub / GitLab / Codeberg / SourceHut / self-hosted ...
```

Do not conflate Git with GitHub.

---

# 127. Hosting Provider Adapter

```rust
pub trait ScmProvider {
    async fn normalize_event(
        &self,
        request: ProviderEvent,
    ) -> Result<NormalizedScmEvent>;

    async fn repository_descriptor(
        &self,
        remote: ProviderRepository,
    ) -> Result<RepositoryRequest>;
}
```

---

# 128. Normalized SCM Event

```rust
pub enum ScmEvent {
    Push(PushEvent),
    TagCreated(TagEvent),
    MergeRequest(ChangeProposalEvent),
    PullRequest(ChangeProposalEvent),
    BranchDeleted(RefDeleteEvent),
    Manual(ManualSourceEvent),
}
```

Provider terminology maps to normalized CI intent, not VCS history semantics.

---

# 129. Event Idempotency

Each event has:

```text
provider
delivery/event ID
repository
exact target revision when available
```

Deduplicate before creating run.

---

# 130. Webhook Verification

Verify provider signatures/tokens before accepting event.

Persist normalized event and raw hash/audit metadata.

---

# 131. Event-to-Revision Resolution

Best order:

1. trust verified exact revision from event;
2. fetch/verify revision exists;
3. verify ref relationship when relevant;
4. snapshot exact revision;
5. never build a later ref accidentally.

---

# 132. Pull/Merge Request Builds

Change proposal may require:

```text
head revision
base revision
provider-generated merge revision
Forgeyard synthetic merge snapshot
```

These are distinct build modes.

---

# 133. Synthetic Merge

Forgeyard can optionally produce a synthetic integration source:

```text
base snapshot/history
+
head revision
+
backend merge operation
```

but merge semantics belong to backend.

Do not implement one universal textual merge engine and call it VCS-neutral.

---

# 134. Synthetic Merge Identity

```text
base revision
+
head revision
+
VCS/backend version
+
merge policy
  ↓
resulting SourceSnapshotId
```

---

# 135. Merge Conflicts

If unresolved:

```text
integration build fails before pipeline
```

unless conflict testing is explicitly requested.

---

# 136. Source Fetch Cache

Cache key:

```text
RepositoryId
+
backend-native object/revision needs
+
fetch policy
```

This is navigation/fetch acceleration.

---

# 137. Snapshot Cache

```text
(RepositoryId, RevisionId, SnapshotPolicy)
    -> SourceSnapshotId
```

Cache result is verified by canonicalization.

---

# 138. Mirror Architecture

```text
Origin
  ↓
Forgeyard source mirror/cache
  ↓
resolver
  ↓
snapshot CAS
```

Enterprise installations can avoid repeated public SCM access.

---

# 139. Multi-Remote Repositories

```rust
pub struct RepositoryRemotes {
    pub primary: Option<RepositoryOrigin>,
    pub fetch: Vec<RepositoryOrigin>,
    pub mirrors: Vec<RepositoryOrigin>,
}
```

Do not assume one origin.

---

# 140. Mirror Trust

A mirror can be:

```text
performance mirror
organization authority
disaster-recovery mirror
air-gap import source
```

Policy distinguishes them.

---

# 141. Source Availability

Once snapshot enters CAS:

```text
build availability
```

no longer depends on origin being online.

---

# 142. Source Retention

GC roots include:

```text
release provenance
active runs
audit retention
pinned source snapshots
compliance retention
```

---

# 143. Re-fetch Verification

If source is re-fetched:

```text
same revision
  ↓
new canonical snapshot
```

must match stored mapping unless backend allows mutable native revision semantics, which should be considered a serious trust violation.

---

# 144. Revision Mutation Detection

If:

```text
NativeRevisionId R
previously -> Snapshot A
now        -> Snapshot B
```

Forgeyard marks:

```text
RevisionContentViolation
```

and quarantines repository/mirror according to policy.

---

# 145. Ref Mutation Is Normal

If:

```text
main -> R1
later main -> R2
```

this is expected.

Only immutable resolution mapping is protected.

---

# 146. Source Provenance Database

Potential tables:

```text
repositories
repository_origins
repository_mirrors
vcs_revisions
vcs_references
vcs_change_ids
source_snapshots
source_snapshot_entries
revision_snapshot_map
source_provenance
source_signatures
source_events
source_fetches
source_migrations
source_equivalence
```

Bulk file bytes remain in CAS.

---

# 147. Database Boundary

Postgres/Neon in distributed mode stores:

```text
metadata
relationships
provenance
state
```

CAS stores:

```text
source blobs
trees
archives
reports
```

Stoolap can provide same metadata interfaces locally.

---

# 148. ForgeyardStore Integration

VCS subsystem uses generic storage ports:

```rust
pub trait SourceMetadataStore {
    async fn put_revision(...);
    async fn map_revision_snapshot(...);
    async fn put_provenance(...);
    async fn source_equivalent(...);
}
```

No adapter directly writes Postgres.

---

# 149. Protocol Messages

Internal Postcard examples:

```rust
pub enum SourceMessage {
    ResolveSource(ResolveSourceRequest),
    FetchSource(FetchSourceRequest),
    MaterializeSnapshot(MaterializeSnapshotRequest),
    QueryChanges(QueryChangesRequest),
}
```

---

# 150. ResolveSourceRequest

```rust
pub struct ResolveSourceRequest {
    pub repository: RepositoryRequest,
    pub revision: RevisionSpec,
    pub snapshot_policy: SnapshotPolicy,
}
```

---

# 151. ResolveSourceResult

```rust
pub struct ResolveSourceResult {
    pub repository: RepositoryId,
    pub resolved_revision: Option<ResolvedRevision>,
    pub snapshot: SourceSnapshotId,
    pub provenance: SourceProvenanceId,
}
```

---

# 152. REST/API Surface

Potential:

```text
POST /v1/sources/resolve
GET  /v1/repositories/{id}
GET  /v1/repositories/{id}/refs
GET  /v1/repositories/{id}/revisions/{rev}
GET  /v1/snapshots/{id}
GET  /v1/snapshots/{id}/tree
POST /v1/sources/diff
GET  /v1/source-provenance/{id}
```

Public APIs may use JSON.

Internal hot-path protocols use Postcard.

---

# 153. CLI

```text
forgeyard vcs detect
forgeyard vcs resolve
forgeyard vcs fetch
forgeyard vcs refs
forgeyard vcs log
forgeyard vcs diff
forgeyard vcs snapshot
forgeyard vcs provenance
forgeyard vcs verify
forgeyard vcs mirror
forgeyard vcs doctor
forgeyard vcs migrate-check

forgeyard git ...
forgeyard hg ...
forgeyard fossil ...
forgeyard jj ...
```

Backend-specific commands should be namespaced and optional.

---

# 154. `forgeyard vcs snapshot`

Example:

```text
forgeyard vcs snapshot . --include-untracked
```

Output:

```text
backend
base revision
dirty state
file count
SourceSnapshotId
```

---

# 155. `forgeyard vcs explain`

Explain:

```text
requested ref
resolved immutable revision
backend
native change identity
snapshot ID
nested sources
signature state
dirty state
fetch origin
```

---

# 156. Dioxus UI

Views:

```text
Repository
Refs
Revision graph
Logical changes
Patch/change view
Source snapshot
Snapshot tree
Diff
Provenance
Signatures
Nested repositories
Mirrors
Fetch health
Migration equivalence
```

---

# 157. Backend-Specific UI

Only show relevant semantics.

Git:

```text
branches/tags
```

Mercurial:

```text
bookmarks/named branches/phases
```

Jujutsu:

```text
change IDs/divergence/bookmarks
```

Pijul:

```text
changes/channels
```

Do not force same labels everywhere.

---

# 158. Error Model

```rust
pub enum VcsError {
    UnsupportedBackend,
    RepositoryNotFound,
    AuthenticationFailed,
    TransportFailed,
    RevisionNotFound,
    AmbiguousRevision,
    UnsupportedOperation,
    ConflictPresent,
    InvalidPath,
    NestedRepositoryViolation,
    SignatureVerificationFailed,
    RevisionContentViolation,
    SnapshotCanonicalizationFailed,
    SourceUnavailable,
    BackendCorruption,
    PolicyDenied,
}
```

---

# 159. Ambiguous Revision

Some backends allow short IDs or ambiguous selectors.

Strict APIs should require resolution to unique immutable ID.

---

# 160. Backend Corruption

Do not convert repository corruption into "revision missing."

Report distinct integrity failure.

---

# 161. Failure Example

```text
Source resolution failed

repository:
  forgeyard-core

backend:
  Mercurial

requested:
  bookmark:release

reason:
  bookmark resolves to an unavailable changeset in current mirror
```

---

# 162. Revision Content Violation Example

```text
Repository integrity violation

revision:
  native-id:R

previous snapshot:
  blake3:A

new snapshot:
  blake3:B

action:
  repository quarantined
```

---

# 163. Nested Repository Failure

```text
Nested repository detected

path:
  vendor/libfoo

policy:
  Reject

suggestion:
  declare it as a subrepository source or snapshot it explicitly
```

---

# 164. Source Canonicalization Failure

Examples:

```text
case-colliding paths
unsafe symlink
path traversal
invalid backend tree
duplicate canonical path
```

---

# 165. Performance Strategy

Principle:

```text
avoid checkout when tree/object APIs suffice
```

Backend adapter can stream source objects directly into canonical CAS.

---

# 166. Checkoutless Snapshot

Preferred where practical:

```text
native repository objects
  ↓
tree iterator
  ↓
canonical tree builder
```

instead of:

```text
checkout to disk
  ↓
rescan filesystem
```

---

# 167. Filesystem Snapshot Fallback

Use when backend cannot expose safe direct tree APIs.

Then use secure temporary sandbox and deterministic scan.

---

# 168. Parallel Blob Ingestion

Parallelize:

```text
hashing
CAS existence checks
blob uploads
```

with bounded concurrency.

Rayon is suitable for CPU hashing/canonical processing.

Tokio is suitable for network/disk orchestration.

---

# 169. Large Repository Strategy

Support:

```text
lazy blob acquisition
tree-first traversal
CAS deduplication
partial native fetch
incremental snapshot construction
```

without weakening snapshot completeness.

---

# 170. Content Reuse Across Revisions

If unchanged blob already exists in CAS:

```text
reuse by digest
```

No need to duplicate bytes.

---

# 171. Content Reuse Across VCS

Git blob and Mercurial file revision yielding same bytes map to same Forgeyard `BlobId`.

---

# 172. Source Compression

CAS compression is storage transport detail.

Digest is over canonical uncompressed logical content representation according to CAS policy.

---

# 173. Security Threat Model

Threats:

```text
malicious repository
malicious archive
path traversal
symlink escape
credential exfiltration
host-key bypass
mutable mirror
revision substitution
ref race
submodule substitution
large-file pointer substitution
repository corruption
parser vulnerabilities
resource exhaustion
```

---

# 174. Untrusted Repository Parsing

Treat repository input as untrusted.

Prefer:

```text
memory-safe Rust parsers/libraries
subprocess sandboxing for external VCS
bounded input sizes
timeouts
resource limits
```

---

# 175. External VCS Process Model

When using VCS executable:

```text
minimal environment
synthetic HOME
no credential helpers unless explicitly configured
bounded runtime
captured structured output
network only during fetch
```

---

# 176. Credential Helpers

Ambient Git/Hg credential helpers are denied in strict Forgeyard service mode.

Use Forgeyard SecretProvider-bound credentials.

---

# 177. User Local Mode Exception

Developer local mode may optionally allow host VCS auth integrations.

Mark provenance:

```text
auth mode: LocalHost
```

without affecting source snapshot identity.

---

# 178. SSRF Protection

Repository URL fetching must enforce:

```text
protocol allowlist
private-network policy
DNS/IP validation
redirect limits
```

especially in hosted/multi-tenant Forgeyard.

---

# 179. File Protocol Policy

Server mode should normally deny arbitrary:

```text
file://
local path remotes
```

outside approved roots.

---

# 180. Multi-Tenant Isolation

Repository fetch caches and credentials must respect tenant boundaries.

Content-addressed blobs may be globally deduplicated only if policy permits and existence side channels are handled.

---

# 181. Audit Events

Record:

```text
repository registered
origin changed
mirror changed
revision resolved
signature verified/failed
snapshot created
dirty snapshot created
revision-content violation
migration equivalence verified
credential policy failure
```

---

# 182. Metrics

Examples:

```text
vcs_resolve_latency
vcs_fetch_bytes
vcs_fetch_failures
snapshot_build_latency
snapshot_blob_reuse_ratio
snapshot_size
revision_snapshot_cache_hit
source_mirror_hit
signature_verification_failures
```

---

# 183. Tracing

W3C/OTLP spans:

```text
source.resolve
source.fetch
source.verify
source.snapshot
source.diff
source.materialize
```

Include repository ID, not secret URL.

---

# 184. Doctor

```text
forgeyard vcs doctor
```

checks:

```text
backend executable/library
version
network/auth
mirror
CAS
snapshot canonicalization
signature tooling
subrepository support
```

---

# 185. Backend Version Policy

Adapter runtime should capture actual backend implementation version.

Behavioral changes can affect:

```text
merge
diff
revision expression
working-tree interpretation
```

For release source resolution, resolved source bytes remain the final truth.

---

# 186. Adapter Conformance Test Suite

Every backend must pass:

1. exact revision resolve;
2. branch/ref resolve;
3. clean snapshot;
4. dirty snapshot if supported;
5. changed paths;
6. nested source behavior;
7. source equivalence;
8. offline materialization;
9. auth failure classification;
10. corruption classification;
11. ref race stability;
12. canonical path tests.

---

# 187. Cross-Backend Golden Repository Suite

Maintain equivalent test repositories across:

```text
Git
Mercurial
Fossil
Breezy
Jujutsu
Darcs
Pijul
```

for portable snapshot scenarios.

Expected:

```text
same visible files
  ↓
same SourceSnapshotId
```

when semantics intentionally represent same tree.

---

# 188. Backend-Native Test Suite

Separately test unique semantics:

Git:

```text
submodules
signed tags
shallow/partial
```

Mercurial:

```text
bookmarks
named branches
phases
```

Jujutsu:

```text
rewrites
change IDs
divergence
```

Pijul/Darcs:

```text
patch/change state
```

Fossil:

```text
artifact/check-in mapping
```

---

# 189. Property Tests

Canonical snapshot properties:

```text
entry order independent
same bytes -> same BlobId
same canonical tree -> same SourceSnapshotId
materialization roundtrip preserves tree
```

---

# 190. Fuzzing

Fuzz:

```text
path canonicalization
archive extraction
metadata parsing
object/tree decoding adapters
webhook normalization
```

---

# 191. Migration Verification

Command:

```text
forgeyard vcs migrate-check old://repo new://repo --revisions mapping.ron
```

Checks selected revision/source pairs.

---

# 192. Migration Mapping

```rust
pub struct MigrationMapping {
    pub from: NativeRevisionRef,
    pub to: NativeRevisionRef,
    pub expected_snapshot_equal: bool,
}
```

---

# 193. Migration Result

```text
revision mapping
snapshot equality
path differences
metadata differences
history mapping confidence
```

---

# 194. VCS Bridge Policy

Git↔Mercurial/Fossil/etc. bridge tools can be used for migration.

They are not canonical Forgeyard source identity.

Always validate resulting snapshots.

---

# 195. Why Not Dual-Authoritative Repositories

Avoid:

```text
Git canonical
+
Mercurial canonical
```

simultaneously.

Instead:

```text
one organizational source authority
+
optional mirrors
+
snapshot equivalence verification
```

---

# 196. Git as Forgeyard's Own Development VCS

Recommended:

```text
Forgeyard source repo:
    Git

Forgeyard VCS architecture:
    neutral
```

This allows Forgeyard development to benefit from Git ecosystem while product architecture remains independent.

---

# 197. Jujutsu on Forgeyard's Git Repo

Developers may optionally use Jujutsu with a Git-backed repository.

Forgeyard's organizational canonical remote can remain Git.

Local developer tool choice need not become Forgeyard internal architecture.

---

# 198. Build Request

```rust
pub struct BuildSource {
    pub snapshot: SourceSnapshotId,
    pub provenance: SourceProvenanceId,
}
```

No Git-specific field.

---

# 199. Pipeline IR

```rust
pub struct PipelineSource {
    pub repository: Option<RepositoryId>,
    pub resolved_revision: Option<RevisionKey>,
    pub snapshot: SourceSnapshotId,
    pub provenance: SourceProvenanceId,
}
```

---

# 200. Derivation Integration

Derivation includes:

```text
SourceSnapshotId
```

and only explicitly requested VCS metadata:

```text
SourceMetadataInput
```

---

# 201. Cache Key Invariant

Changing commit message without changing source tree:

```text
does not invalidate compile derivation
```

unless build explicitly consumes commit metadata.

Changing source tree:

```text
does invalidate
```

---

# 202. Release Provenance

Release provenance still records:

```text
repository
revision
tag/ref
signature evidence
snapshot
```

even when build cache keys use source snapshot content.

---

# 203. Reproducibility

Reproducer needs:

```text
SourceSnapshotId
```

It does not need original VCS service to still exist if CAS retention is satisfied.

---

# 204. Disaster Recovery

Release source can be reconstructed from Forgeyard CAS/provenance even if original remote is temporarily unavailable, subject to retention policy.

---

# 205. Air-Gap Source Bundle

Bundle:

```text
source snapshot
tree/blob CAS closure
provenance
native VCS identity metadata
signatures
```

No VCS network needed in air-gapped build environment.

---

# 206. Source Bundle Format

Forgeyard-native source bundle can contain:

```text
manifest.ron
snapshot metadata
Postcard graph/provenance
content objects
checksums
```

Transport archive itself is not source identity.

---

# 207. RON Configuration

Example:

```ron
source: Vcs(
    backend: Auto,

    repository: (
        origin: "ssh-secret-ref/repo",
    ),

    revision: Ref("main"),

    snapshot: (
        include_untracked: false,
        include_ignored: false,
        nested_repositories: ResolveAsSubrepository,
    ),
)
```

Actual secret-bearing repository origins should use typed secret references, not literal credentials.

---

# 208. Git Configuration Example

```ron
vcs: Git(
    fetch: (
        partial: Auto,
        shallow: Auto,
    ),

    submodules: Resolve,

    signatures: (
        release_tags: Required,
    ),
)
```

---

# 209. Mercurial Configuration Example

```ron
vcs: Mercurial(
    bookmarks: Enabled,
    named_branches: Preserve,
    phases: Record,
)
```

---

# 210. Jujutsu Configuration Example

```ron
vcs: Jujutsu(
    change_identity: Preserve,
    divergence: RejectForRelease,
    git_backend_metadata: Preserve,
)
```

---

# 211. Patch VCS Configuration Example

```ron
vcs: Pijul(
    channel: "main",
    conflicts: Reject,
    patch_metadata: Preserve,
)
```

---

# 212. Implementation Phase 1 — Neutral Core

Implement:

```text
RepositoryId
VcsKind
RevisionSpec
ResolvedRevision
SourceSnapshotId
canonical path/tree/blob model
SourceProvenance
```

---

# 213. Phase 2 — Local/Archive

Before Git, prove source model with:

```text
local tree adapter
archive adapter
canonical snapshot tests
```

This verifies VCS-neutral core independently.

---

# 214. Phase 3 — Git

Implement deepest production adapter:

```text
refs
commits
trees
worktree
submodules
diff
signatures
partial/shallow fetch
```

---

# 215. Phase 4 — SCM Provider Events

Implement normalized:

```text
push
tag
change proposal
manual
```

with Git hosting providers first.

---

# 216. Phase 5 — Mercurial

Implement:

```text
changesets
bookmarks
named branches
tags
phases
revsets
working copy
```

---

# 217. Phase 6 — Snapshot Diff + Impact

Implement VCS-neutral:

```text
snapshot diff
change paths
monorepo impact interface
```

---

# 218. Phase 7 — Jujutsu

Implement:

```text
commit IDs
change IDs
rewrites
divergence
bookmarks
Git-backed semantics
```

---

# 219. Phase 8 — Fossil + Breezy

Add immutable-artifact/baseline and branch/revision models.

---

# 220. Phase 9 — Darcs + Pijul

Add patch/change-centric extension interfaces.

Do not compromise core model to make them look Git-like.

---

# 221. Phase 10 — Enterprise Source Mirror

Implement:

```text
organization mirrors
source fetch cache
signature policy
air-gap source bundles
revision mutation detection
```

---

# 222. Phase 11 — Migration/Interop

Implement:

```text
source equivalence
revision mapping
migration reports
bridge validation
```

---

# 223. Phase 12 — Hardening

Add:

```text
fuzzing
SSRF controls
parser sandboxing
resource limits
multi-tenant isolation
corruption tests
```

---

# 224. Acceptance Tests

1. Same source tree from Git and Mercurial yields same `SourceSnapshotId`.
2. Commit message-only change does not change source snapshot.
3. Source-byte change changes source snapshot.
4. Git branch movement does not alter already-resolved run.
5. Mercurial bookmark movement does not alter already-resolved run.
6. Jujutsu rewrite preserves ChangeId but changes RevisionId.
7. Jujutsu rewritten commit with identical tree preserves SourceSnapshotId.
8. Local dirty tree gets unique source snapshot.
9. Release policy rejects dirty tree.
10. Git submodule child revision becomes explicit child provenance.
11. Archive path traversal is rejected.
12. Case-collision is detected.
13. Repository credential is absent from source identity/provenance plaintext.
14. Offline runner builds from SourceSnapshotId without VCS.
15. Native revision mapping to different snapshot later is quarantined.
16. Pijul unresolved conflict is rejected for release.
17. Darcs/Pijul adapter can materialize snapshot without pretending patch is Git commit.
18. Fossil check-in materializes deterministic snapshot.
19. Source migration checker proves selected Git/Mercurial states equivalent.
20. Snapshot diff drives affected-build calculation independently of VCS.

---

# 225. Production Readiness Gates

Do not call the subsystem production-ready until:

```text
canonical path/tree semantics frozen/versioned
Git adapter passes full conformance
dirty snapshot semantics are stable
subrepository model works
archive security works
revision/ref races are eliminated
credentials are isolated
signature evidence is normalized
revision mutation is detected
offline build from snapshot works
Mercurial adapter passes conformance
cross-backend snapshot equivalence tests pass
```

Other VCS adapters can mature independently after core reaches production status.

---

# 226. Architectural Invariants

1. Forgeyard builds snapshots, not VCS refs.
2. `SourceSnapshotId` is independent of VCS-native hashes.
3. Mutable ref names never enter immutable source identity.
4. Native revision IDs are provenance, not CAS IDs.
5. Logical change identity is distinct from revision identity.
6. VCS-neutral core does not expose Git-specific types.
7. Backend-native semantics are preserved through typed metadata/capabilities.
8. Patch-centric VCS are not flattened into fake Git commits.
9. Jujutsu change IDs are not treated as commit IDs.
10. Dirty trees are explicit immutable snapshots.
11. Nested repositories are explicit.
12. Build runners do not need VCS network access.
13. VCS credentials never enter source snapshot identity.
14. Same canonical tree can deduplicate across VCS.
15. Native immutable revision mapping to source snapshot cannot silently mutate.
16. Ref movement is expected and safe after exact resolution.
17. Snapshot diff is the universal fallback for change-impact analysis.
18. Uncertain impact expands work rather than skipping work.
19. SCM hosting providers are separate from VCS backends.
20. Webhook exact revisions beat late re-resolution of mutable refs.
21. Source bytes and VCS metadata are separate derivation concepts.
22. Build cache invalidation follows source snapshot plus explicit VCS metadata inputs.
23. Release provenance still records repository/revision/signature history.
24. Migration bridges never become Forgeyard internal source identity.
25. Forgeyard's own repository may use Git while Forgeyard core remains VCS-neutral.

---

# 227. Final Target Architecture

```text
             Git / Mercurial / Fossil / Breezy
             Jujutsu / Darcs / Pijul / Local
                         │
                         ▼
                  VCS Backend Adapter
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
       References     History      Native metadata
            │            │            │
            └────────────┼────────────┘
                         ▼
                  ResolvedRevision
                         │
                         ▼
                Source Materialization
                         │
                         ▼
                Canonical Tree Builder
                         │
                         ▼
                  SourceSnapshotId
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
            CAS      Provenance   Snapshot Diff
             │           │           │
             │           │           ▼
             │           │      Change Impact
             │           │           │
             └───────────┼───────────┘
                         ▼
                    Pipeline IR
                         │
                         ▼
                    Derivations
                         │
                         ▼
                 Forgeyard Scheduler
                         │
                         ▼
                VCS-Free Build Runner
```

---

# 228. Final Architectural Position

The source-resolution formula is:

```text
Repository
+
VCS backend
+
mutable or immutable revision request
+
snapshot policy
        ↓
exact backend-native source state
        ↓
canonical source tree
        ↓
SourceSnapshotId
```

The build formula is:

```text
SourceSnapshotId
+
explicit source metadata inputs
+
toolchains
+
dependency closures
+
build configuration
=
Forgeyard Derivation
```

The provenance formula is:

```text
RepositoryId
+
VcsKind
+
NativeRevisionId
+
optional ChangeId
+
resolved-from ref/tag/bookmark/channel
+
dirty state
+
signature evidence
+
nested source provenance
+
SourceSnapshotId
=
SourceProvenance
```

This creates the desired separation:

```text
VCS answers:
    "Where did this source state come from?"

Forgeyard snapshot answers:
    "Exactly which source bytes are being built?"

Forgeyard derivation answers:
    "Exactly which complete build inputs produced this artifact?"
```

That separation is the foundation that lets Forgeyard deeply support Git today, Mercurial and other FOSS VCS tomorrow, and future source-control models without rewriting the scheduler, pipeline IR, cache, CAS, provenance, or build architecture.

---

# Appendix A — Backend Semantic Mapping

| Forgeyard concept | Git | Mercurial | Fossil | Breezy | Jujutsu | Darcs | Pijul |
|---|---|---|---|---|---|---|---|
| Repository | repository | repository | repository DB | repository | repo/workspace | repository | repository |
| Revision ID | commit OID | changeset/node | check-in artifact | revision ID | commit ID | backend state/context | channel/state context |
| Logical Change ID | optional derived | optional derived | optional derived | optional derived | native change ID | patch identity | change/patch identity |
| Mutable ref | branch/ref | bookmark | branch/tag metadata | branch | bookmark | not Git-like | channel |
| Tag | tag | tag | tag metadata | tag | Git-backed/native metadata | backend native | backend native |
| Revision DAG | yes | yes | check-in ancestry | yes | yes | not primary abstraction | not primary abstraction |
| Patch semantics | diff/patch secondary | diff secondary | artifact/check-in | diff secondary | change+commit | primary | primary |
| Snapshot | tree | manifest state | baseline | revision tree | commit tree | repository state | channel state |

This table is a normalization guide, not a claim that the systems are semantically identical.

---

# Appendix B — Capability Matrix Philosophy

Forgeyard code should prefer:

```rust
if backend.capabilities().contains(VcsCapabilities::LOGICAL_CHANGE_ID) {
    // expose logical change UI/API
}
```

rather than:

```rust
if backend.kind() == VcsKind::Jujutsu {
    // scattered special case
}
```

Backend-specific behavior remains in adapter crates.

---

# Appendix C — Example Build Source

```ron
source: (
    repository: "repo:forgeyard",

    requested_revision: Ref("main"),

    snapshot_policy: (
        include_untracked: false,
        include_ignored: false,
        nested_repo_policy: ResolveAsSubrepository,
    ),

    metadata_inputs: (
        revision: true,
        source_date: false,
    ),
)
```

At run creation this resolves into immutable:

```ron
resolved_source: (
    vcs: Git,
    revision: "native:...",
    snapshot: "blake3:...",
    provenance: "srcprov:...",
)
```

---

# Appendix D — Forgeyard's Own Recommended Use

Forgeyard itself should use:

```text
canonical development VCS:
    Git

internal source model:
    VCS-neutral

developer local alternative:
    Jujutsu-on-Git allowed

product support:
    Git first
    Mercurial next
    Jujutsu next
    Fossil/Breezy
    Darcs/Pijul
```

This gives Forgeyard the mainstream ecosystem advantage of Git without allowing Git implementation details to become architectural dependencies.

---

# Appendix E — Upstream Semantic Principles Used by the Architecture

The architecture intentionally reflects several backend-native differences:

- Git models committed source through commit/tree/object identities and mutable refs.
- Mercurial distinguishes changesets, bookmarks, tags, and named branches; bookmarks are movable references while named branches are persistent history metadata.
- Fossil models immutable artifacts and check-in manifests/baselines.
- Breezy separates working tree, branch, repository, and revision concepts.
- Jujutsu separates commit IDs from stable logical change IDs, and rewrites can retain the change identity.
- Darcs is fundamentally patch-oriented rather than state/commit-DAG-first.
- Pijul uses changes/patches and channels and can model conflicts in repository state.

Forgeyard therefore normalizes source state and provenance, not every VCS's internal theory of history.
