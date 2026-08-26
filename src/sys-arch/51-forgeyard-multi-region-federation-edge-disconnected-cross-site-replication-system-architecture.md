# 51 — Forgeyard Multi-Region Federation, Edge Sites, Disconnected Operation & Cross-Site Replication System Architecture

**Document type:** Core Federation, Multi-Region, Edge-Site, Disconnected Operation, Cross-Site Replication & Geographic Reliability System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** regional sites, federated installations, edge/home-lab/branch sites, disconnected and intermittently connected operation, metadata replication, CAS replication, regional placement, failover/failback, sovereignty/data residency, site trust, cross-site reconciliation, global read models, site-local execution, cross-site artifact movement, and geo-distributed operational governance  
**Architecture style:** Explicit authority domains, single-writer mutable truth, replicated immutable data, site-local execution, conflict-free derived state where possible, durable reconciliation, residency-aware placement, bounded autonomy, and no ambiguous multi-writer control-plane ownership  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan

---

## 1. Purpose

Forgeyard may need to operate across multiple cloud regions, data centers, branch offices, factories, labs, developer edge sites, sovereign jurisdictions, and air-gapped or intermittently connected networks.

The system therefore needs a rigorous answer to the following:

- Which site owns mutable metadata?
- Can a regional site keep building during a WAN outage?
- Which classes of work are permitted while disconnected?
- How are immutable artifacts replicated?
- How are regional read models kept fresh?
- How is failover performed without split-brain?
- How are residency and sovereignty rules enforced?
- How are offline results reconciled after reconnection?
- What happens if an old site returns after failover?
- How are multi-region runner fleets scheduled safely?

The central rule is:

> **Federation distributes execution and data geographically, but every mutable authority domain has exactly one accepted writer epoch at any point in time.**

A second rule is:

> **Immutable content may replicate broadly where policy allows; mutable business/control state replicates according to explicit ownership and consistency rules.**

A third rule is:

> **Disconnected operation is bounded autonomy, not hidden split-brain. A site may continue only the classes of work for which it has explicit authority, immutable inputs, and valid policy/trust state.**

---

## 2. Architectural Position

```text
                    Global Forgeyard Federation
                              │
               ┌──────────────┼──────────────┐
               ▼              ▼              ▼
          Authority Site   Replica Site    Edge Site
               │              │              │
               │         read / CAS copy   delegated work
               │              │              │
               └──────────────┼──────────────┘
                              ▼
                      Durable Reconciliation
                              │
                              ▼
                       Canonical Global View
```

Federation sits above regional HA and below user-facing global discovery/routing:

```text
Global Federation
  ├── Authority ownership / site routing
  ├── Residency / sovereignty
  ├── Cross-site CAS replication
  ├── Disconnected operation
  └── Failover/failback
        ↓
Regional Forgeyard Site
  ├── Postgres/Neon authority or replica
  ├── regional Raft if needed
  ├── scheduler
  ├── agents/runners
  ├── CAS/cache
  └── API/UI edge
```

A federation is **not** one giant WAN-spanning Raft cluster.

---

## 3. Goals

The subsystem MUST:

1. define site and region identity;
2. define federation membership;
3. define authority domains;
4. enforce one mutable writer epoch per authority domain;
5. support primary/secondary regions;
6. support site-local execution;
7. support disconnected/air-gapped operation;
8. support immutable CAS replication;
9. support read-replica metadata;
10. prevent split-brain;
11. support failover and failback;
12. support tenant data residency;
13. support regional runner placement;
14. support cross-site artifact transfer;
15. support release replication without rebuild;
16. support deployment locality;
17. support site-local caches;
18. support event journaling/reconciliation;
19. support global authorized read models;
20. support site trust/enrollment;
21. support WAN partitions;
22. support site quarantine;
23. support tenant relocation;
24. support air-gap import/export bundles;
25. support mixed-version site compatibility;
26. support observability/doctor;
27. support audit;
28. support API/UI/CLI;
29. remain optional for standalone/single-region installations;
30. preserve canonical subsystem authority.

---

## 4. Non-Goals

This subsystem does not:

```text
turn PostgreSQL into eventually-consistent multi-master storage
make all business state CRDT-based
make Raft span unreliable global links by default
allow concurrent release authority in disconnected sites
allow local policy to weaken global security minima
replace CAS
replace DR/backups
replace the scheduler
replace Deployment or Release authority
```

---

## 5. Workspace Structure

```text
crates/federation/
├── forgeyard-federation/
├── forgeyard-federation-model/
├── forgeyard-federation-membership/
├── forgeyard-federation-authority/
├── forgeyard-federation-routing/
├── forgeyard-federation-metadata/
├── forgeyard-federation-cas/
├── forgeyard-federation-disconnected/
├── forgeyard-federation-failover/
├── forgeyard-federation-reconcile/
├── forgeyard-federation-health/
└── forgeyard-federation-testkit/
```

Optional adapters:

```text
crates/federation-adapters/
├── forgeyard-federation-postgres/
├── forgeyard-federation-object-store/
├── forgeyard-federation-iroh/
├── forgeyard-federation-airgap/
└── forgeyard-federation-custom/
```

Use modules before creating new crates unless there is a real dependency/runtime/security boundary.

---

## 6. Site and Region Identity

```rust
pub struct SiteId(Ulid);
pub struct RegionId(BoundedString);
pub struct FederationId(Ulid);
```

A site is a concrete Forgeyard operational location. A region is a logical geographic, legal, or failure-domain grouping.

```rust
pub struct SiteDescriptor {
    pub id: SiteId,
    pub region: RegionId,
    pub kind: SiteKind,
    pub trust: SiteTrustClass,
    pub connectivity: ConnectivityClass,
}
```

```rust
pub enum SiteKind {
    PrimaryRegion,
    SecondaryRegion,
    Edge,
    Branch,
    AirGapped,
    DisasterRecovery,
    Custom(SiteKindId),
}
```

```rust
pub enum ConnectivityClass {
    AlwaysConnected,
    Intermittent,
    OfflineCapable,
    AirGapped,
}
```

---

## 7. Site Trust

```rust
pub enum SiteTrustClass {
    CoreTrusted,
    RegionalTrusted,
    Restricted,
    EdgeLimited,
    Quarantined,
}
```

Trust must never be inferred from IP range, VPN membership, or physical location.

A site may be local but untrusted. A site may be remote but strongly trusted.

---

## 8. Federation Membership

```rust
pub struct FederationMembership {
    pub federation: FederationId,
    pub site: SiteId,
    pub state: FederationMembershipState,
}
```

```rust
pub enum FederationMembershipState {
    Joining,
    Active,
    Draining,
    Suspended,
    Removed,
}
```

Joining requires explicit authorization, certificate enrollment, compatibility checks, and trust assignment.

A site cannot self-join.

---

## 9. Authority Domains

The most important federation concept is the **authority domain**.

```rust
pub struct AuthorityDomainId(Digest);
```

Examples:

```text
tenant metadata
project control plane
release authority
deployment environment
policy/config
runner fleet
site-local device lab
site-local cache
```

Every mutable authority domain has one accepted writer.

```rust
pub struct AuthorityLease {
    pub domain: AuthorityDomainId,
    pub holder: SiteId,
    pub epoch: AuthorityEpoch,
    pub valid_until: Option<Timestamp>,
}
```

```rust
pub struct AuthorityEpoch(u64);
```

All protected mutable writes carry or are validated against the current epoch where applicable.

---

## 10. Single-Writer Mutable Truth

The following must not become optimistic multi-writer state:

```text
Run state
Job state
Attempt state
Release approval
Release publication authority
Deployment state
Policy activation
Config activation
Entitlement authority
security/trust epochs
```

If two sites believe they own the same mutable authority domain, Forgeyard treats it as a critical federation conflict and fails closed.

No last-write-wins behavior is permitted for these domains.

---

## 11. Immutable Multi-Site Data

Immutable content is much easier to replicate safely.

Examples:

```text
CAS blobs
source snapshots
artifact bytes
SBOMs
provenance documents
test evidence
signed bundles
toolchain packages
```

The same digest may exist at many sites.

The digest remains the content identity, independent of site.

---

## 12. Tenant Home Site

```rust
pub struct TenantHomeSite {
    pub tenant: TenantId,
    pub site: SiteId,
}
```

A tenant may be assigned a home site/region for mutable metadata authority and residency.

This is especially important for:

```text
sovereign deployments
regulated industries
customer-controlled regions
air-gapped tenants
```

---

## 13. Residency Policy

```rust
pub struct ResidencyPolicy {
    pub metadata_regions: RegionSet,
    pub artifact_regions: RegionSet,
    pub execution_regions: RegionSet,
}
```

Residency is a **hard scheduling/routing constraint**.

It may not be weakened for:

```text
lower cost
lower latency
cache locality
available capacity
```

If no eligible region exists, work waits or fails explicitly.

---

## 14. Site Selection

Before local scheduling, federation can choose an eligible site.

```rust
pub struct SiteSelectionDecision {
    pub site: SiteId,
    pub reasons: Vec<SiteSelectionReason>,
}
```

Hard filters:

```text
residency
authority
site trust
platform/device availability
connectivity
tenant scope
```

Soft scores:

```text
latency
CAS locality
cost
queue depth
warm capacity
```

The scheduler still decides which runner receives the job.

Federation chooses eligible site scope; scheduler chooses eligible execution capacity.

---

## 15. Metadata Topology

Recommended baseline for a mutable authority domain:

```text
one writable authoritative PostgreSQL/Neon domain
+
regional read replicas
+
rebuildable caches/read models
```

Do not make global active-active multi-master PostgreSQL the baseline.

Regional replicas can serve:

```text
history
catalog
search
analytics
read-only dashboards
```

when freshness requirements allow.

---

## 16. Replica Freshness

```rust
pub enum ReplicaFreshness {
    Current,
    Lagging(Duration),
    Stale,
    Unknown,
}
```

Staleness must be visible to API/UI.

Protected policy decisions, approvals, release decisions, entitlement decisions, and other security-sensitive reads must use authoritative/current state.

---

## 17. Read Consistency

```rust
pub enum ReadConsistency {
    LocalStaleAllowed,
    BoundedStaleness(Duration),
    Authoritative,
}
```

The API may expose consistency selection only where safe.

Examples:

```text
Run history → bounded stale may be acceptable
Release approval → authoritative
policy evaluation → authoritative
catalog search → local stale may be acceptable
```

---

## 18. Regional Read Models

Read replicas should not become accidental business authority.

Read models may be rebuilt from:

```text
authoritative metadata
domain events
CAS manifests
```

If stale, they show stale/unknown rather than pretending current.

---

## 19. Disconnected Authority Grant

Disconnected writes require bounded delegated authority.

```rust
pub struct DisconnectedAuthorityGrant {
    pub site: SiteId,
    pub domains: Vec<AuthorityDomainId>,
    pub epoch: AuthorityEpoch,
    pub valid_until: Timestamp,
}
```

An offline site never receives permanent unbounded write authority.

---

## 20. Disconnected Operation Classes

```rust
pub enum DisconnectedOperationClass {
    ReadOnly,
    LocalBuild,
    LocalTest,
    LocalArtifactCreate,
    LocalReleaseCandidate,
    ProtectedReleaseForbidden,
    DeploymentForbidden,
}
```

A site may have different permission classes for different authority domains.

Example:

```text
build/test       → allowed for 7 days offline
artifact produce → allowed for 7 days
release approve  → forbidden
signing          → 2-hour delegated window only
prod deployment  → forbidden
```

---

## 21. Local Build While Offline

A disconnected site can build/test if it possesses:

```text
exact SourceSnapshot
toolchains
dependency closure
policy bundle
config snapshot
executor capability
local metadata authority
CAS space
```

If required inputs are missing, Forgeyard fails explicitly.

It must not bypass hermetic/network policy merely because the WAN is down.

---

## 22. Offline Run Identity

Local Run/Job/Attempt identities remain globally unique.

Use typed globally unique IDs.

No post-reconnect ID rewriting.

---

## 23. Site Event Journal

Disconnected sites maintain a durable event journal.

The journal stores allowed local authoritative events until reconnection.

Properties:

```text
append-only
sequence numbered
bounded
checksummed
tenant/site scoped
epoch-bound
```

If a journal required for authoritative mutation becomes full, privileged mutations fail closed.

No silent event dropping.

---

## 24. Federation Reconciliation Classes

```rust
pub enum FederationReconcileClass {
    ImmutableMerge,
    AuthorityReplay,
    DerivedRebuild,
    Conflict,
}
```

- **ImmutableMerge** — CAS/evidence by digest.
- **AuthorityReplay** — accepted events from current authority epoch.
- **DerivedRebuild** — search/catalog/analytics.
- **Conflict** — unexpected concurrent ownership or incompatible mutations.

A `Conflict` is not auto-merged.

---

## 25. No Last-Write-Wins

Never use last-write-wins for:

```text
Release
Policy
Deployment
Approval
Run/Job state
security state
```

If an impossible concurrent write appears, preserve both records for investigation and block further protected mutation until resolved.

---

## 26. CAS Replication

```rust
pub enum CasReplicationPolicy {
    OnDemand,
    Pinned,
    RegionReplicated(u8),
    Global,
    AirGapBundleOnly,
}
```

Typical use:

```text
release artifacts     → region replicated
active run inputs     → on demand
cache                 → local/on demand
security evidence     → residency constrained
air-gap dependencies  → bundle only
```

---

## 27. CAS Replica Record

```rust
pub struct CasReplicaRecord {
    pub object: CasObjectId,
    pub site: SiteId,
    pub state: ReplicaState,
}
```

```rust
pub enum ReplicaState {
    Requested,
    Transferring,
    Verified,
    Corrupt,
    Missing,
    Unknown,
}
```

A replica is available only after digest verification.

---

## 28. CAS Transfer Protocols

Adapters may use:

```text
QUIC
S3-compatible object replication
GCS/Azure object copy
Iroh P2P acceleration
offline bundles
```

Iroh is an acceleration/data-movement adapter only.

It does not become the authority for release, metadata, or durability.

---

## 29. Cross-Site Release Replication

```text
ReleaseId
+
signed release metadata
+
artifact/package digests
  ↓
replicate exact bytes
  ↓
verify signatures/digests
  ↓
mark site availability
```

A regional copy is not a new release.

No rebuild occurs during replication.

---

## 30. Release Availability

```rust
pub struct RegionalReleaseAvailability {
    pub release: ReleaseId,
    pub site: SiteId,
    pub artifacts_verified: bool,
}
```

Release trust remains governed by Part 15/13.

---

## 31. Environment Home Site

```rust
pub struct EnvironmentHomeSite {
    pub environment: EnvironmentId,
    pub site: SiteId,
}
```

Deployment authority can be pinned to a site.

Cross-region deployment may still occur through provider adapters, but the environment has one control authority.

---

## 32. Failover Plan

```rust
pub struct FederationFailoverPlan {
    pub domain: AuthorityDomainId,
    pub from: SiteId,
    pub to: SiteId,
    pub next_epoch: AuthorityEpoch,
    pub prerequisites: Vec<FailoverPrerequisite>,
}
```

Failover must validate:

```text
target trust
target compatibility
replica freshness
database recovery state
CAS availability
site certificate state
policy/config availability
old-site fencing capability
```

---

## 33. Failover Is Not a DNS Flip

Correct sequence:

```text
detect/freeze old authority
  ↓
establish authoritative data state
  ↓
advance AuthorityEpoch
  ↓
activate target authority
  ↓
fence stale writers
  ↓
route mutation traffic
  ↓
reconcile
```

---

## 34. Unclean Failover

If the previous region disappears abruptly, some last writes may be unknown.

Forgeyard records:

```text
last confirmed sequence
replication point
possible data-loss window
RPO status
```

It must not claim zero data loss without proof.

---

## 35. Site Fencing

```rust
pub struct SiteFence {
    pub site: SiteId,
    pub minimum_epoch: AuthorityEpoch,
}
```

A recovering old site with stale epoch is prevented from writing.

---

## 36. Failback

Failback is not immediate reversal.

Correct sequence:

```text
rebuild/re-sync former site
  ↓
verify no stale state
  ↓
prepare authority transfer
  ↓
advance epoch again
  ↓
route traffic
```

The old epoch never becomes valid again.

---

## 37. WAN Partition Behavior

A site may be:

```text
healthy locally
but globally partitioned
```

Therefore health and connectivity are separate.

```rust
pub enum SiteHealth {
    Healthy,
    Degraded,
    Partitioned,
    Offline,
    Recovering,
    Quarantined,
    Unknown,
}
```

```rust
pub enum SiteConnectivityState {
    Connected,
    Degraded,
    Partitioned,
    OfflineExpected,
}
```

A planned air-gapped site must not be marked failed simply because it is offline.

---

## 38. Site-Local HA

A regional site may itself run:

```text
Postgres HA
regional Raft coordination
multiple API nodes
multiple schedulers
multiple agents
```

Global federation sits above this.

```text
Global Federation
  ↓
Regional Authority Domain
  ↓
Regional HA
```

Do not stretch low-latency consensus across unreliable global WAN links by default.

---

## 39. Federation + Raft

Raft remains narrow.

Recommended:

```text
regional Raft:
  membership
  leadership
  coordination epoch
  exclusive operations
```

Federation authority epochs govern site-to-site ownership.

The two are related but not the same system.

---

## 40. Federation Routing

Mutation requests go to the authority site.

Reads can use a local site if consistency permits.

If a client hits a non-authoritative site:

```text
proxy securely
or
return authority-routing metadata
```

Avoid clients guessing authority.

---

## 41. Global Idempotency

Cross-site retry must preserve idempotency.

Idempotency keys are scoped to the authority domain and survive failover.

A request retried after failover should resolve to the same semantic operation when possible.

---

## 42. Webhooks

Provider webhook ingress can be regional.

Flow:

```text
regional ingress
  ↓
signature verification
  ↓
replay-window validation
  ↓
dedupe/persist
  ↓
forward normalized event to authority
```

If WAN is unavailable, persist and forward later.

Do not replay raw historical webhooks blindly.

---

## 43. Schedules

Each scheduled trigger has one authority domain/site epoch.

Never allow two sites to independently fire the same authoritative schedule during a partition.

Schedule execution uses authority epoch fencing.

---

## 44. Policy and Configuration

Disconnected sites use exact:

```text
PolicyBundleId
ConfigSnapshotId
```

with explicit expiry/freshness rules.

A site may continue lower-risk operations under last approved policy.

Privileged operations can fail closed when policy/trust freshness expires.

---

## 45. Secret Resolution

Prefer site-local secret providers.

A logical `SecretRef` may resolve through a site-specific provider binding.

Secret values are not replicated globally merely for convenience.

---

## 46. Signing Authority

Signing should be pinned to highly trusted sites.

If the signing site is unavailable:

```text
release waits
or
explicitly delegated signer is used
```

Never fall back to unsigned release.

Root signing keys must not be copied to edge sites.

---

## 47. Air-Gap Site Bundle

```rust
pub struct FederationSiteBundle {
    pub site: SiteId,
    pub config: ConfigSnapshotId,
    pub policy: PolicyBundleId,
    pub dependencies: Vec<CasObjectRef>,
    pub toolchains: Vec<ToolchainDescriptorId>,
}
```

A bundle may also include:

```text
source snapshots
template packages
dependency mirrors
runner image metadata
trust roots
```

The bundle is signed and digest verified.

---

## 48. Air-Gap Return Bundle

A disconnected site can export:

```text
artifacts
test evidence
logs
provenance
Run summaries
security findings
benchmark evidence
```

The receiving installation validates:

```text
site trust
historical policy/config identity
signatures
digests
authority epoch
```

Imported outputs do not automatically receive higher trust than the originating site.

---

## 49. Offline Readiness

```rust
pub enum OfflineReadiness {
    Ready,
    MissingInputs,
    AuthorityInsufficient,
    PolicyExpiring,
    TrustExpiring,
    Unknown,
}
```

Command:

```text
forgeyard federation prepare-offline
```

checks:

```text
source closure
dependency closure
toolchains
policy/config
CAS space
authority grants
certificate/trust expiry
```

---

## 50. Reconnection

Recommended order:

```text
connect
  ↓
validate site identity/epoch
  ↓
upload immutable journal/CAS objects
  ↓
verify
  ↓
replay accepted authoritative events
  ↓
rebuild derived state
  ↓
receive newer config/policy
  ↓
resume connected mode
```

Historical runs retain historical policy/config identity.

Do not silently reinterpret them using new policy.

---

## 51. Site Trust Freshness

Long-disconnected sites may exceed:

```text
certificate validity
policy validity
image trust window
security epoch
```

Their new outputs can therefore be quarantined or downgraded until re-attested.

---

## 52. Site Quarantine

A quarantined site:

```text
receives no new authority
receives no privileged work
cannot publish trusted release outputs
cannot update policy
```

Existing outputs are reviewed according to compromise window.

---

## 53. Site Trust Epoch

```rust
pub struct SiteTrustEpoch(u64);
```

Re-enrollment/reimage creates a new trust epoch where needed.

This integrates with Part 40 compromise epochs.

---

## 54. Site Removal

Safe site removal:

```text
mark Draining
  ↓
transfer authority domains
  ↓
drain runners
  ↓
replicate required evidence
  ↓
verify no exclusive data remains
  ↓
revoke site identity/certs
  ↓
delete replicas under lifecycle policy
  ↓
mark Removed
```

---

## 55. Tenant Relocation

```rust
pub struct TenantRelocationPlan {
    pub tenant: TenantId,
    pub from: SiteId,
    pub to: SiteId,
    pub cutover_epoch: AuthorityEpoch,
}
```

Safe relocation:

```text
replicate tenant data
  ↓
validate destination
  ↓
quiesce mutable writes
  ↓
advance authority epoch
  ↓
activate destination
  ↓
fence old site
  ↓
purge old replicas when policy allows
```

No live dual-writer relocation.

---

## 56. Global Search and Catalog

Global search/catalog is a derived federation view.

It may:

```text
query remote sites
merge authorized result summaries
show freshness/site provenance
```

It must not copy source/private metadata into forbidden regions merely to improve search.

---

## 57. Global Read Model

```rust
pub struct FederatedReadProjection {
    pub site: SiteId,
    pub freshness: ReplicaFreshness,
    pub generated_at: Timestamp,
}
```

UI must surface:

```text
site
freshness
authority
```

when relevant.

---

## 58. Runner Site Identity

Runner registration includes verified `SiteId`.

A runner cannot self-assert a privileged site.

Site assignment derives from enrollment/provisioning authority.

---

## 59. Cross-Site Runner Routing

Useful for scarce resources:

```text
macOS
GPU
device labs
confidential compute
```

Flow:

```text
job requirements
  ↓
federation site filter
  ↓
eligible sites
  ↓
site selection
  ↓
local scheduler
  ↓
runner lease
```

---

## 60. Device Lab Federation

Device labs stay site-local.

A remote job can be routed to a site hosting the required device capability.

The device lease itself remains local/authoritative in Part 20.

---

## 61. Cache Federation

Cache remains optional acceleration.

A cross-site cache hit is allowed only when:

```text
tenant scope matches
trust is sufficient
platform portability matches
cache evidence is valid
```

Cache federation never becomes a reason to violate residency.

---

## 62. Dependency Mirrors

Part 36 mirrors can be site-local.

Disconnected sites should prefetch the immutable dependency closure.

A public registry outage should not matter when the promoted closure already exists locally.

---

## 63. Toolchain Replication

Immutable toolchain descriptors/packages can be replicated to sites.

A site may not silently substitute a locally installed mutable toolchain when exact tooling is required.

---

## 64. Source Snapshot Replication

Source snapshots are immutable.

Private source residency applies independently from artifact residency.

A release may be globally distributable while source remains region-restricted.

---

## 65. Bandwidth Governance

```rust
pub struct ReplicationBandwidthPolicy {
    pub max_bytes_per_sec: Option<u64>,
    pub priority: ReplicationPriority,
}
```

Priority examples:

```text
release critical
active run inputs
security evidence
cache
archive
```

Replication throttling changes performance, not integrity.

---

## 66. Cross-Site Transfer Reliability

Large transfers support:

```text
chunking
resume
per-chunk verification
final digest verification
retry/backoff
```

A partial object is never published as a verified replica.

---

## 67. Site Metadata Replication Modes

```rust
pub enum MetadataReplicationMode {
    ReadReplica,
    DelegatedSubset,
    SnapshotBundle,
    EventJournal,
}
```

- **ReadReplica** — connected secondary site.
- **DelegatedSubset** — limited local authority.
- **SnapshotBundle** — air-gap import.
- **EventJournal** — disconnected return path.

---

## 68. Provider-Native Replication

Cloud/Postgres providers may implement cross-region replication.

Forgeyard does not infer business write authority from provider replication topology.

Provider replication is transport/durability.

Forgeyard authority is explicit.

---

## 69. Federation Protocol

Internal federation messages use versioned envelopes.

Prefer:

```text
QUIC + mTLS + Postcard
```

for native Forgeyard federation transport.

Protocol carries:

```text
SiteId
AuthorityDomainId
AuthorityEpoch
message kind
correlation/idempotency
```

---

## 70. Mixed-Version Sites

```rust
pub struct FederationCompatibility {
    pub protocol: VersionRange,
    pub metadata_schema: VersionRange,
    pub event_semantics: VersionRange,
}
```

N/N-1 compatibility where supported.

Unsupported versions cannot assume authority.

---

## 71. Site Upgrade Order

Typical:

```text
replica/edge sites first
  ↓
validate compatibility
  ↓
secondary authority-capable site
  ↓
controlled authority transfer if needed
  ↓
primary authority site
```

Part 41 handles package/update delivery; Part 51 handles federation safety.

---

## 72. Disaster Recovery Relationship

Federation can improve regional resilience.

It does not replace:

```text
PITR
offline backup
trust-root recovery
CAS durability
clean-room restore
```

A replica is not a backup.

Corruption or malicious mutation can replicate.

---

## 73. RTO/RPO

Part 25 defines recovery strategy and Part 50 reliability objectives.

Part 51 provides measurable federation evidence such as:

```text
replication point
failover duration
authority transfer duration
CAS replica availability
```

---

## 74. Cost Integration

Part 45 tracks:

```text
cross-region egress
replicated storage
regional runner cost
warm standby cost
air-gap media/export cost
```

Cost remains a soft consideration after:

```text
residency
trust
correctness
platform
authority
```

---

## 75. Reliability Integration

Possible federation SLOs:

```text
regional read freshness
authority failover time
CAS replication success
federation routing availability
```

Expected-offline edge sites are scoped appropriately rather than treated as globally failed.

---

## 76. Data Lifecycle

Part 46 applies independently to every replica.

Deletion/tombstones replicate according to authority.

CAS physical deletion occurs only after:

```text
all required roots removed
replica lifecycle permits
legal/security holds cleared
```

---

## 77. Restore and Deletion Journal

If a site is restored from older backup:

```text
restore
  ↓
apply current authority fences
  ↓
replay deletion journal
  ↓
reconcile federation state
  ↓
only then rejoin
```

A restored stale site must never resurrect deleted data or stale authority.

---

## 78. Audit

Audit:

```text
site join/remove
trust changes
authority transfer
failover/failback
offline authority grant
residency policy change
site quarantine
tenant relocation
```

Routine CAS replication is operational telemetry, not privileged audit per chunk.

---

## 79. Notifications

Examples:

```text
authority grant expiring
site partitioned
site trust expiring
replication lag high
CAS corruption
journal nearing capacity
failover prerequisite failed
stale epoch writes detected
```

---

## 80. Observability Metrics

```text
federation_sites_total
federation_authority_domains_total
federation_replication_lag_seconds
federation_cas_replication_backlog_bytes
federation_site_partitions_total
federation_failovers_total
federation_stale_epoch_rejections_total
federation_journal_utilization_ratio
```

Use controlled labels:

```text
site
region
state
replication_class
```

---

## 81. Tracing

```text
federation.route
federation.replicate
federation.authority
federation.reconcile
federation.failover
federation.failback
federation.reconnect
```

---

## 82. Health

Health checks include:

```text
authority uniqueness
replica freshness
CAS replica integrity
site certificate/trust state
event journal capacity
policy/config freshness
```

---

## 83. Doctor

```text
forgeyard federation doctor
```

Checks:

```text
multiple accepted writers
expired offline grants
stale epoch traffic
site identity mismatch
missing CAS roots
replica freshness
certificate expiry
journal overflow risk
mixed-version incompatibility
```

---

## 84. Dioxus UI

Pages:

```text
Federation Overview
Sites
Authority Domains
Replication
Failover
Disconnected Sites
Residency
Tenant Placement
```

Federation overview shows:

```text
site health
connectivity
authority ownership
metadata lag
CAS lag
trust state
```

---

## 85. Authority UI

The UI must always be able to answer:

> **Which site is currently allowed to write this authority domain, and at which epoch?**

This is more important than a decorative topology map.

---

## 86. Failover UI

Failover requires:

```text
plan
precondition checks
authorization
optional approval
execute
observe
```

No blind one-click failover.

---

## 87. CLI

```text
forgeyard federation status
forgeyard federation site list
forgeyard federation site show
forgeyard federation authority show
forgeyard federation replicate
forgeyard federation prepare-offline
forgeyard federation failover plan
forgeyard federation failover execute
forgeyard federation failback plan
forgeyard federation site quarantine
forgeyard federation doctor
```

---

## 88. API

Potential endpoints:

```text
GET  /v1/federation/sites
GET  /v1/federation/authority
GET  /v1/federation/replication
POST /v1/federation/failover/plan
POST /v1/federation/failover/execute
POST /v1/federation/failback/plan
POST /v1/federation/sites/{id}/quarantine
```

---

## 89. Permissions

```text
federation.read
federation.site.manage
federation.authority.manage
federation.failover.plan
federation.failover.execute
federation.failback.execute
federation.quarantine
federation.residency.manage
```

Authority transfer/failover execution is high privilege.

Separation of duties may require different planner/approver/executor identities.

---

## 90. Standalone Mode

Federation is disabled by default.

Standalone may still use:

```text
air-gap export/import
offline bundle verification
```

without becoming a federation.

---

## 91. Distributed Single-Region Mode

No federation layer is required.

This is important: multi-region federation is optional complexity, not mandatory baseline architecture.

---

## 92. Migration Path

Recommended adoption:

```text
single region
  ↓
regional CAS/read replica
  ↓
regional execution pools
  ↓
authority-transfer support
  ↓
secondary authority-capable site
  ↓
disconnected edge sites
```

Do not begin with global active-active complexity.

---

## 93. Testkit

```text
forgeyard-federation-testkit/src/
├── lib.rs
├── authority.rs
├── site.rs
├── replication.rs
├── partition.rs
├── offline.rs
├── failover.rs
├── failback.rs
├── relocation.rs
└── assertions.rs
```

---

## 94. Core Tests

### Authority
- one writer accepted;
- stale epoch rejected;
- old site cannot write after transfer;
- concurrent ownership conflict fails closed.

### Replication
- CAS digest verified;
- corrupted replica quarantined;
- missing replica does not corrupt original;
- partial transfer never published.

### Residency
- prohibited region cannot receive metadata;
- prohibited region cannot receive source;
- prohibited region cannot execute job;
- cheaper region cannot bypass policy.

### Offline
- complete bundle builds successfully;
- missing toolchain/dependency fails explicitly;
- policy expiry blocks privileged operation;
- journal overflow fails closed;
- reconnect merges immutable results idempotently.

### Failover
- authority epoch advances;
- old writer fenced;
- idempotency survives;
- reads/mutations route correctly;
- unknown last-write window surfaced.

### Failback
- former site resyncs;
- old epoch never reused;
- authority moves only after validation.

### Security
- quarantined site receives no privileged work;
- signing unavailable does not fall back unsigned;
- secret values not globally replicated;
- site identity cannot self-escalate.

### Schedules/Webhooks
- no duplicate schedule firing;
- webhook edge persists/forwards exactly-once semantically through idempotency.

### DR
- replica is not treated as backup;
- restored stale site reapplies deletion/fencing state before rejoin.

---

## 95. Chaos Tests

Inject:

```text
WAN partition
high packet loss
high latency
entire region loss
DB primary failover
CAS backend outage
object-store replication lag
site clock skew
certificate expiry
journal disk-full
```

Verify that protected operations either remain correct or fail closed.

---

## 96. Scale Tests

Test:

```text
large artifact replication backlogs
many runner regions
many tenant residency policies
large disconnected journals
thousands of authority domains
```

Federation state must remain bounded and queryable.

---

## 97. Implementation Phases

### Phase 1 — Site and Authority Model
Implement:

```text
SiteId
RegionId
AuthorityDomainId
AuthorityEpoch
fencing
membership
```

### Phase 2 — Read/CAS Replication
Add:

```text
regional CAS replicas
read models
freshness
```

### Phase 3 — Site-Aware Scheduling
Add federation site filtering before local scheduler placement.

### Phase 4 — Failover/Failback
Add authority transfer and stale-site fencing.

### Phase 5 — Edge Controller
Support bounded site-local work.

### Phase 6 — Disconnected Grants
Add expiring delegated authority.

### Phase 7 — Air-Gap Return/Reconciliation
Add signed bundle and journal reconciliation.

### Phase 8 — Residency/Tenant Relocation
Add sovereign placement and controlled tenant migration.

### Phase 9 — Federated Search/Catalog
Add global authorized read views.

### Phase 10 — Bandwidth/Cost Optimization
Add replication priorities and cost visibility.

### Phase 11 — Mixed-Version Site Upgrades
Add N/N-1 federation compatibility.

### Phase 12 — Chaos/Scale/Security Hardening
Production readiness.

---

## 98. Acceptance Tests

1. Every mutable authority domain has exactly one accepted writer epoch.
2. Stale authority epochs are rejected.
3. Federation never uses last-write-wins for control-plane truth.
4. Immutable CAS objects may safely exist at multiple sites.
5. Every CAS replica is digest verified.
6. Regional read replicas are never implicit write authority.
7. Replica freshness is explicit.
8. Tenant residency is a hard constraint.
9. Cost/latency cannot override residency or trust.
10. Disconnected authority is bounded and expiring.
11. Protected release/deployment is forbidden without explicit authority.
12. Offline Run/Job IDs are globally unique.
13. Reconnection merges immutable results idempotently.
14. Unexpected concurrent mutable authority fails closed.
15. Failover advances authority epoch.
16. The old site cannot resume stale writes.
17. Failback requires resynchronization.
18. Federation is not implemented as one WAN-spanning Raft cluster by default.
19. Site-local HA remains independent.
20. Schedules cannot fire authoritatively in two sites simultaneously.
21. Regional webhook ingress remains idempotent.
22. Secret values are not globally replicated for convenience.
23. Signing authority remains site/trust scoped.
24. Air-gapped sites can build/test from complete signed bundles.
25. Missing offline inputs fail explicitly.
26. Journal overflow cannot silently drop authoritative events.
27. Global search/catalog obeys residency and exposes freshness.
28. Tenant relocation never creates live dual-writer state.
29. Replica state is never treated as backup.
30. Site quarantine removes new authority/work.
31. Mixed-version sites obey compatibility matrix.
32. Standalone/single-region installations do not require federation.
33. Restore replays fencing/deletion state before site rejoin.
34. WAN partition/region-loss behavior is chaos-tested.
35. Forgeyard can dogfood federation across multiple runner regions/sites.

---

## 99. Production Readiness Gates

Do not call federation production-ready until:

```text
authority ownership is machine-enforced
epoch fencing rejects stale writers
CAS replication validates all target digests
residency enforcement passes
replica staleness is visible
offline grants expire correctly
journal overflow fails safely
failover/failback preserve authority uniqueness
split-brain tests pass
WAN partition and region-loss chaos tests pass
```

---

## 100. Architectural Invariants

1. federation never creates ambiguous mutable authority;
2. every mutable domain has explicit owner and epoch;
3. authority transfer advances epoch;
4. stale epochs cannot mutate;
5. immutable content may replicate where policy permits;
6. metadata replication respects authority;
7. read replicas are never automatic writers;
8. disconnected operation is bounded autonomy;
9. offline authority expires;
10. protected release/deployment requires explicit authority;
11. globally unique IDs avoid offline collisions;
12. last-write-wins is forbidden for protected state;
13. reconnection is idempotent/reconciled;
14. unexpected concurrent authority fails closed;
15. residency is a hard constraint;
16. cost/latency are secondary optimizations;
17. secrets remain site-local where possible;
18. signing keys stay high-trust/site-scoped;
19. federation is not a global WAN Raft cluster;
20. site-local HA is independent;
21. read staleness is explicit;
22. schedule/webhook semantics remain idempotent;
23. journals cannot silently overflow;
24. replicas do not replace backup;
25. quarantine revokes new work/authority;
26. mixed-version compatibility is explicit;
27. federation remains optional for simpler deployments;
28. DR still requires backups;
29. all authority/failover operations are auditable;
30. Forgeyard dogfoods its own federation architecture.

---

## 101. Final Target Architecture

```text
                    Global Forgeyard Federation
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
      Authority Region    Replica Region     Edge Site
            │                 │                 │
     mutable authority     read/CAS copy    delegated scope
            │                 │                 │
            └─────────────────┼─────────────────┘
                              ▼
                    Durable Reconciliation
                              │
                              ▼
                     Canonical Global View
```

Mutable authority:

```text
AuthorityDomain
+
SiteId
+
AuthorityEpoch
  ↓
single accepted writer
```

Immutable replication:

```text
CAS digest
  ↓
replicate
  ↓
verify
  ↓
regional availability
```

Disconnected operation:

```text
immutable source/dependency/toolchain closure
+
signed policy/config
+
bounded authority grant
  ↓
local build/test
  ↓
durable journal
  ↓
reconnect
  ↓
idempotent reconciliation
```

Failover:

```text
authority unavailable
  ↓
verify recovery state
  ↓
advance epoch
  ↓
activate new site
  ↓
fence old site
  ↓
route + reconcile
```

> **Forgeyard can operate across regions, sites, unreliable WAN links, and air-gapped environments without turning geographic distribution into split-brain. Immutable data can move freely where policy allows, while mutable control-plane truth always has explicit ownership, fenced epochs, and deterministic reconciliation.**

---

## 102. Extended Architecture Sequence

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
42 Workflow Templates / Reusable Pipelines / Organization Standards / Golden Paths
43 Runner Fleet Autoscaling / Capacity Provisioning / Infrastructure Providers
44 Pipeline Triggers / Schedules / Manual Dispatch / Event-Driven Execution
45 Cost Accounting / FinOps / Chargeback / Showback / Resource Economics
46 Data Lifecycle / Retention / Archival / Deletion / Legal Hold / Privacy Governance
47 CI/CD Migration / Import / Compatibility / Legacy-System Interoperability
48 Failure Diagnosis / Debugging / Reproduction / Bisect / Root-Cause Intelligence
49 Service Catalog / Component Ownership / Environment Inventory / Developer Portal
50 Reliability Engineering / SLO / Error Budget / Availability / Resilience Governance
51 Multi-Region Federation / Edge Sites / Disconnected Operation / Cross-Site Replication
```
