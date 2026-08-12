# 22 — Forgeyard High Availability, Coordination & Raft System Architecture

**Document type:** Core High Availability, Cluster Coordination & Consensus System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** multi-daemon HA, cluster membership, coordination epochs, leader election, Raft-backed narrow consensus state, scheduler fencing, lease coordination, release/deployment exclusivity, failover, split-brain prevention, rolling upgrades, quorum behavior, and cluster recovery  
**Architecture style:** PostgreSQL/Neon remains authoritative business metadata; CAS remains artifact authority; Raft is used only for narrow coordination state that truly requires consensus/fencing  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on Storage/Metadata, Run/Job, Scheduler, Runner/Agent, Events/Reconciliation, Release, Deployment, Transport/QUIC, Health/Doctor, and API architectures. It deliberately avoids turning Forgeyard into an event-sourced or Raft-database architecture.

---

# 1. Purpose

Forgeyard distributed mode needs more than one daemon instance for production availability.

Multiple daemon replicas should survive:

```text
process crash
node crash
VM restart
AZ failure
rolling deployment
network interruption
operator maintenance
```

without producing:

```text
double scheduling
duplicate exclusive release actions
split-brain leadership
stale lease acceptance
conflicting channel promotion
unsafe deployment coordination
```

The central rule is:

> **Raft coordinates ownership and epochs; it does not become Forgeyard's general database.**

A second rule is:

> **PostgreSQL/Neon remains authoritative for Runs, Jobs, Attempts, Releases, Policies, Users, Events, and other business state.**

A third rule is:

> **Every coordination decision that may survive leader change must be either reconstructible from authoritative state or fenced by a monotonic epoch/term.**

---

# 2. Architectural Position

```text
                     Clients / Agents
                           │
                           ▼
                    HA Daemon Fleet
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
          daemon-A       daemon-B      daemon-C
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                    Coordination Layer
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
          Raft Log      Cluster View   Epochs/Fences
                           │
                           ▼
                  PostgreSQL / Neon
                 authoritative metadata
                           │
                           ▼
                           CAS
```

---

# 3. Goals

The subsystem MUST:

1. support multiple daemon replicas;
2. support leader election;
3. support membership;
4. prevent split-brain;
5. expose monotonic coordination epochs;
6. fence stale scheduler leaders;
7. fence stale exclusive operations;
8. survive daemon restart;
9. survive leader failure;
10. support rolling upgrades;
11. support N/N-1 compatibility;
12. support quorum loss behavior;
13. support cluster reconfiguration;
14. support node draining;
15. support health/readiness integration;
16. support observer/non-voting members;
17. support bootstrap;
18. support secure node identity;
19. support TLS/mTLS;
20. support reconciliation after failover;
21. support cluster snapshots/log compaction;
22. support safe disaster recovery;
23. remain narrow in scope;
24. avoid putting large domain data in Raft;
25. avoid putting CAS data in Raft;
26. avoid duplicating PostgreSQL transactions in Raft;
27. keep standalone mode simple;
28. permit HA without forcing Raft for every subsystem;
29. permit Postgres locking where sufficient;
30. be testable under partitions/failures.

---

# 4. Non-Goals

Raft does not store:

```text
Run rows
Job rows
logs
artifacts
SBOMs
release manifests
deployment manifests
policy documents
user sessions
SCM payloads
```

Those remain in normal stores/CAS.

---

# 5. Workspace Structure

```text
crates/coordination/
├── forgeyard-coordination/
├── forgeyard-coordination-model/
├── forgeyard-coordination-api/
├── forgeyard-coordination-raft/
├── forgeyard-coordination-membership/
├── forgeyard-coordination-leader/
├── forgeyard-coordination-epoch/
├── forgeyard-coordination-lock/
├── forgeyard-coordination-scheduler/
├── forgeyard-coordination-release/
├── forgeyard-coordination-deploy/
├── forgeyard-coordination-health/
├── forgeyard-coordination-recovery/
└── forgeyard-coordination-testkit/
```

Potential cluster app integration:

```text
apps/forgeyard-daemon/src/cluster/
├── mod.rs
├── bootstrap.rs
├── membership.rs
├── readiness.rs
└── shutdown.rs
```

---

# 6. Core Principle: Narrow Consensus

Consensus is expensive.

Only coordinate state that genuinely needs one globally agreed ordering.

---

# 7. Good Raft Candidates

```text
cluster membership
current coordination leader
scheduler leader epoch
exclusive release-operation epoch
exclusive deployment-operation epoch
global maintenance epoch
certificate/trust rotation coordination epoch
cluster configuration version
```

---

# 8. Poor Raft Candidates

```text
Job logs
Run state
artifact metadata
CAS indexes
webhook payloads
metrics
audit records
```

---

# 9. PostgreSQL vs Raft

Use PostgreSQL for:

```text
business transactions
entity state
outbox/inbox
leases
reservations
approval state
release metadata
deployment state
```

Use Raft for:

```text
coordination ownership/fencing where distributed consensus adds value
```

---

# 10. Why Not Replace PostgreSQL With Raft

Forgeyard already needs rich relational semantics:

```text
queries
indexes
transactions
constraints
pagination
joins
migrations
```

Rebuilding that over Raft would add major complexity without proportional benefit.

---

# 11. ClusterNodeId

```rust
pub struct ClusterNodeId(Ulid);
```

Stable daemon-node identity.

---

# 12. Node Incarnation

```rust
pub struct NodeIncarnationId(Ulid);
```

New process incarnation.

---

# 13. Why Incarnation

Prevents restarted node process from inheriting old ephemeral authority.

---

# 14. Cluster Member

```rust
pub struct ClusterMember {
    pub node: ClusterNodeId,
    pub incarnation: NodeIncarnationId,
    pub role: ClusterMemberRole,
    pub endpoint: ClusterEndpoint,
    pub state: ClusterMemberState,
}
```

---

# 15. Member Roles

```rust
pub enum ClusterMemberRole {
    Voter,
    Learner,
    Observer,
}
```

---

# 16. Voter

Participates in quorum.

---

# 17. Learner

Receives replicated log but cannot vote.

Useful for adding new node.

---

# 18. Observer

Reads cluster state/health without consensus participation if implementation supports.

---

# 19. Member State

```rust
pub enum ClusterMemberState {
    Joining,
    Active,
    Draining,
    Leaving,
    Removed,
}
```

---

# 20. Cluster Configuration

```rust
pub struct ClusterConfiguration {
    pub version: ClusterConfigVersion,
    pub members: Vec<ClusterMember>,
}
```

---

# 21. Configuration Version

Monotonic.

---

# 22. Cluster Term

Raft term.

---

# 23. Coordination Epoch

Higher-level Forgeyard monotonic value.

```rust
pub struct CoordinationEpoch(u64);
```

---

# 24. Leader Epoch

```rust
pub struct LeaderEpoch {
    pub term: u64,
    pub epoch: CoordinationEpoch,
}
```

---

# 25. Fencing

Every leader-owned mutation can include epoch.

---

# 26. Scheduler Epoch

```rust
pub struct SchedulerEpoch(u64);
```

---

# 27. Scheduler Fencing Rule

A scheduler leader can only issue lease/reservation operations if its epoch is current.

---

# 28. Database Check

Store mutation includes:

```text
expected SchedulerEpoch
```

or server-side coordination validation.

---

# 29. Stale Leader

After partition/leadership loss:

```text
its operations are rejected
```

even if process still alive.

---

# 30. No Time-Based Leadership

Wall clock expiry alone does not define leadership.

---

# 31. Exclusive Operation

```rust
pub struct ExclusiveOperationLease {
    pub scope: ExclusiveScope,
    pub owner: ClusterNodeId,
    pub epoch: CoordinationEpoch,
}
```

---

# 32. Exclusive Scopes

Examples:

```text
ReleaseChannelPromotion
ClusterMigration
TrustRootRotation
GlobalGcCoordination
DeploymentGlobalSwitch
```

---

# 33. Do Not Overuse Exclusive Scope

Project-local work usually can use PostgreSQL row locks/versioning.

---

# 34. Release Lock

Release architecture already has durable DB lock.

Raft can optionally add cluster-level epoch for:

```text
global release channel mutation
```

where needed.

---

# 35. Deployment Lock

Same.

---

# 36. Scheduler Leadership

Recommended initial HA scheduler model:

```text
one active scheduler coordinator
multiple standby daemon replicas
```

---

# 37. Why One Logical Scheduler Leader

Simplifies:

```text
fairness state
queue ordering
scarcity scoring
autoscaler signals
```

while DB still protects lease correctness.

---

# 38. Scheduler Correctness Without Raft

Postgres atomic lease remains correctness safety net.

---

# 39. Raft Adds

```text
clear leadership
faster failover
fencing
coordination state
```

---

# 40. Scheduler Leader Election

Derived from Raft leader or separate coordinated role assignment.

---

# 41. Recommended

Use Raft leader as coordination leader, then assign scheduler epoch.

---

# 42. Role Assignment

Potential:

```rust
pub enum CoordinatedRole {
    SchedulerLeader,
    ReleaseCoordinator,
    DeploymentCoordinator,
    MigrationCoordinator,
}
```

---

# 43. Role Epoch

```rust
pub struct RoleEpoch {
    pub role: CoordinatedRole,
    pub epoch: CoordinationEpoch,
    pub owner: ClusterNodeId,
}
```

---

# 44. Raft State Machine

Small deterministic state machine.

---

# 45. Raft Command

```rust
pub enum CoordinationCommand {
    AddLearner(...),
    PromoteVoter(...),
    RemoveMember(...),
    AssignRole(...),
    BumpEpoch(...),
    SetMaintenance(...),
    StartExclusiveOperation(...),
    CompleteExclusiveOperation(...),
}
```

---

# 46. Raft State

```rust
pub struct CoordinationState {
    pub cluster_config: ClusterConfiguration,
    pub roles: BTreeMap<CoordinatedRole, RoleEpoch>,
    pub maintenance: MaintenanceState,
    pub exclusive: BTreeMap<ExclusiveScope, ExclusiveOperationLease>,
}
```

---

# 47. No Arbitrary Payload

Raft command sizes strictly bounded.

---

# 48. Determinism

State-machine application pure/deterministic.

---

# 49. Raft Storage

Local durable state per daemon node.

---

# 50. Raft Log

Contains only coordination commands.

---

# 51. Snapshot

Periodic compacted coordination snapshot.

---

# 52. Snapshot Contents

Small:

```text
membership
roles/epochs
exclusive operations
maintenance state
```

---

# 53. Snapshot Security

Integrity protected.

---

# 54. Raft Network Transport

Could reuse internal QUIC transport abstraction.

---

# 55. Raft Protocol Family

Separate message family.

---

# 56. Node-to-Node mTLS

Mandatory distributed production.

---

# 57. Cluster Node Identity

Certificate binds:

```text
ClusterNodeId
```

---

# 58. Node Certificate

Separate purpose from runner cert.

---

# 59. Membership Admission

New node must be explicitly authorized.

---

# 60. Bootstrap

First cluster node initializes cluster.

---

# 61. Bootstrap Token

Short-lived join credential.

---

# 62. Join Flow

```text
new daemon
  ↓
authenticate cluster
  ↓
join as learner
  ↓
catch up
  ↓
health verify
  ↓
promote to voter
```

---

# 63. No Direct Voter Join

Learner-first safer.

---

# 64. Remove Node

Graceful:

```text
drain
  ↓
remove voter
  ↓
stop
```

---

# 65. Forced Removal

For permanently lost node.

Requires quorum/operator authorization.

---

# 66. Odd Voter Count

Recommended:

```text
3
5
```

---

# 67. Three Nodes

Minimum practical HA.

---

# 68. Two Nodes

No good automatic failover quorum.

Avoid as HA topology.

---

# 69. Five Nodes

Higher fault tolerance but more latency/ops.

---

# 70. Recommended Initial

3 voters.

---

# 71. Quorum

Majority of voting members.

---

# 72. Quorum Loss

No new consensus writes.

---

# 73. Quorum Loss Does Not Mean Database Gone

PostgreSQL may still be healthy.

---

# 74. Degraded Behavior

On Raft quorum loss:

```text
reads may continue
existing jobs may continue
new coordination-sensitive operations stop
scheduler leadership may stop/expire
```

---

# 75. New Scheduling During Quorum Loss

Recommended:

```text
stop new scheduler-authority work
```

unless design explicitly permits DB-only emergency scheduler mode.

---

# 76. Existing Attempts

Continue under existing DB leases.

---

# 77. Completion

Can still be accepted if DB/control path available and lease valid.

---

# 78. Release Promotion

Block if exclusive coordination unavailable.

---

# 79. Deployment Global Switch

Block if coordination required.

---

# 80. API Readiness

Daemon can remain read-ready but write-degraded.

---

# 81. Health State

```text
Coordination: Degraded/Unhealthy
```

---

# 82. Leader Failure

Followers elect new leader.

---

# 83. Failover

New leader:

```text
acquires current term
bumps role epoch
reconstructs work from DB
runs reconciliation
```

---

# 84. No In-Memory Recovery Dependency

Critical.

---

# 85. Scheduler Recovery

New scheduler leader scans:

```text
Eligible jobs
active leases
reservations
dispatch outbox
runner state
```

---

# 86. Release Recovery

Scans:

```text
Approved/Promoting releases
publication states
locks
```

---

# 87. Deployment Recovery

Scans:

```text
Applying/Verifying/Unknown/RollingBack
```

---

# 88. Reconciliation After Leadership Change

Mandatory targeted sweep.

---

# 89. Split Brain

Raft quorum prevents two valid leaders.

---

# 90. But Process-Level Split Brain

Old leader may still believe it owns role temporarily.

Epoch fencing prevents damage.

---

# 91. Double Defense

```text
consensus leadership
+
DB/store fencing epoch
```

---

# 92. Database Fencing Field

Potential:

```text
coordination_epoch
scheduler_epoch
```

on relevant operations.

---

# 93. Lease Creation Transaction

Validate scheduler epoch.

---

# 94. Release Promotion Transaction

Validate release coordinator epoch if used.

---

# 95. Stale Epoch Result

Typed error:

```text
FENCED
```

---

# 96. Fenced Node Behavior

Stop role immediately, refresh cluster state.

---

# 97. Network Partition

Case:

```text
1 node isolated
2 nodes connected
```

majority elects leader.

Isolated node cannot commit.

---

# 98. Symmetric Partition

No quorum -> no coordination writes.

---

# 99. Partial Connectivity to DB

A node may see DB but not quorum.

Still cannot act as coordination leader.

---

# 100. Partial Connectivity to CAS

Independent subsystem health.

---

# 101. Node Readiness

A daemon ready for general API may depend on role.

---

# 102. Follower API

Can serve most stateless API requests.

---

# 103. Write API

Can write business DB state if no leader-specific operation required.

---

# 104. Leader-Specific API

Forward/internal route or enqueue desired action for coordinator.

---

# 105. Recommendation

Do not redirect public clients to leader.

Any daemon accepts request and persists desired state.

Coordinator processes it.

---

# 106. Benefits

```text
simple load balancing
stable API
leader transparency
```

---

# 107. Example Release Promotion

Any API node:

```text
authorize
persist promotion intent
```

Coordinator:

```text
acquire role/epoch
process intent
```

---

# 108. API Statelessness

Preferred.

---

# 109. Session State

External/shared store or signed session, not leader-local.

---

# 110. Load Balancer

Can send traffic to any ready daemon.

---

# 111. Sticky Sessions

Not required ideally.

---

# 112. WebSocket/SSE

Connections tied to one daemon.

On disconnect, client reconnects/backfills.

---

# 113. No Session Authority in Memory

Critical.

---

# 114. Agent Connections

Agents may connect to any daemon endpoint.

---

# 115. Agent HA

If connected daemon dies:

```text
reconnect to another daemon
```

---

# 116. Active Lease State

DB authority allows new daemon to resync.

---

# 117. Transport Session

Changes after reconnect.

---

# 118. AgentSession

Same process may remain same if agent reconnects.

---

# 119. Daemon Endpoint Discovery

Potential:

```text
DNS
load balancer
configured endpoint list
```

---

# 120. QUIC Load Balancing

May require stable routing/anycast/L4 support.

---

# 121. Simpler Initial

Agent endpoint list + reconnect/failover.

---

# 122. Internal Cluster Discovery

Static seeds + persisted membership.

---

# 123. DNS Discovery

Bootstrap aid, not authority.

---

# 124. Membership Authority

Raft cluster config.

---

# 125. Cluster UUID

```rust
pub struct ClusterId(Ulid);
```

---

# 126. Prevent Wrong Cluster Join

Node verifies ClusterId.

---

# 127. Cluster Name

Human label only.

---

# 128. Cluster Secret

Avoid shared static secret if mTLS/bootstrap tokens suffice.

---

# 129. Maintenance Mode

```rust
pub enum MaintenanceState {
    Normal,
    ReadOnly,
    NoScheduling,
    NoReleasePromotion,
    FullMaintenance,
}
```

---

# 130. Global Maintenance

Good Raft candidate because all nodes need same view.

---

# 131. Maintenance Epoch

Monotonic.

---

# 132. Maintenance Change

Audited in normal metadata store too.

---

# 133. Raft Audit

Consensus state changes should emit normal audit/domain records.

---

# 134. Do Not Use Raft Log as Audit Log

Retention semantics differ.

---

# 135. Coordination Event

Examples:

```text
ClusterLeaderChanged
SchedulerEpochAdvanced
MemberAdded
MemberRemoved
MaintenanceChanged
```

---

# 136. Event Source

Raft state application can publish event/outbox after observing committed state.

---

# 137. Duplicate Event

Safe.

---

# 138. Cluster Health

```rust
pub struct ClusterHealth {
    pub quorum: QuorumHealth,
    pub leader: Option<ClusterNodeId>,
    pub term: u64,
    pub members: Vec<MemberHealth>,
}
```

---

# 139. QuorumHealth

```text
Healthy
Degraded
Lost
Unknown
```

---

# 140. Member Health

```text
Raft replication lag
node heartbeat
API readiness
DB reachability
```

---

# 141. Health Distinction

A node can be alive but lagging.

---

# 142. Leadership Eligibility

Node must satisfy:

```text
current binary compatible
DB schema compatible
coordination storage healthy
trust valid
```

---

# 143. Prevent Bad Leader

Node not ready should step down/not campaign where implementation permits.

---

# 144. Rolling Upgrade

Cluster supports N/N-1 protocol compatibility.

---

# 145. Upgrade Order

Recommended:

```text
add/upgrade follower
catch up
upgrade next follower
transfer leadership
upgrade old leader
```

---

# 146. Leadership Transfer

Useful before maintenance.

---

# 147. Node Drain

```text
stop accepting coordinated roles
transfer leadership
finish API requests
stop
```

---

# 148. Rolling Schema Migration

PostgreSQL expand-contract architecture.

---

# 149. Raft Schema Version

Coordination state schema also versioned.

---

# 150. Compatibility

New binary must read current/previous coordination snapshot/log schema.

---

# 151. Command Version

Explicit.

---

# 152. Unsupported Command

Older node should not remain voter if it cannot interpret committed commands.

---

# 153. Upgrade Gate

Cluster capability intersection determines when new command features can be used.

---

# 154. Cluster Feature Version

```rust
pub struct ClusterFeatureVersion(u32);
```

---

# 155. Feature Activation

After all required voters support feature.

---

# 156. Avoid Flag Day

Use staged capability negotiation.

---

# 157. Membership Change

Use Raft joint-consensus/safe membership mechanism from chosen library.

---

# 158. Never Hand-Roll Membership Algorithm

Use proven Raft implementation semantics.

---

# 159. Rust Raft Implementation

Architecture should wrap chosen implementation behind Forgeyard coordination traits.

---

# 160. Library Boundary

```rust
pub trait CoordinationBackend {
    async fn current_view(&self) -> Result<CoordinationView, CoordinationError>;
    async fn propose(&self, cmd: CoordinationCommand) -> Result<CoordinationCommit, CoordinationError>;
}
```

---

# 161. Domain Does Not Depend on Raft Crate Types

Adapter boundary.

---

# 162. Standalone Backend

Mode 1:

```text
LocalCoordinationBackend
```

single-node in-memory/durable lightweight semantics.

---

# 163. Distributed Backend

```text
RaftCoordinationBackend
```

---

# 164. Same Coordination API

Higher-level services stay mode-neutral.

---

# 165. Local Epoch

Standalone still uses epochs for testing/semantic consistency.

---

# 166. Cluster Persistent Directory

Contains only Raft state/snapshots.

---

# 167. Disk Requirements

Fsync/durable writes.

---

# 168. Disk Corruption

Node should fail safe and rejoin from snapshot if recoverable.

---

# 169. Do Not Silently Reset Voter State

Could cause cluster identity loss/split brain.

---

# 170. Raft Backup

Cluster consensus state can be reconstructed from surviving quorum.

---

# 171. Disaster Recovery

If all Raft members lost but PostgreSQL/CAS survive:

operator performs explicit coordination-cluster recovery.

---

# 172. DR Recovery Rule

Never auto-create a new cluster silently from one stale node.

---

# 173. Recovery Input

Use:

```text
known ClusterId
latest trusted coordination snapshot
authoritative DB state
operator quorum-loss procedure
```

---

# 174. Coordination State Reconstructibility

Design narrow enough that much can be re-established.

---

# 175. Example

Scheduler epoch can be bumped.

Membership can be rebuilt.

Exclusive locks can be reconciled against DB.

---

# 176. DR Safety

Prefer losing transient coordination state over corrupting business state.

---

# 177. Snapshot Export

Admin tool can export coordination snapshot metadata.

---

# 178. Snapshot Import

Dangerous operator-only recovery.

---

# 179. Backup Frequency

Small state; snapshots frequent.

---

# 180. Log Compaction

Standard Raft mechanism.

---

# 181. Snapshot Trigger

Entries/size thresholds.

---

# 182. Snapshot Size Limit

Strict.

---

# 183. No Large Blob in Command

Enforced by type/size limits.

---

# 184. Raft Request Timeout

Bounded.

---

# 185. Election Timeout

Configured with network latency margin.

---

# 186. Heartbeat Interval

Reasonable, jittered per implementation.

---

# 187. WAN Raft

Avoid spanning high-latency global regions initially.

---

# 188. Recommended Topology

3 voters within one low-latency region/AZ set.

---

# 189. Multi-Region

Use PostgreSQL/Neon architecture and regional workers; keep one coordination quorum region initially.

---

# 190. Cross-Region DR

Standby cluster rather than synchronous global Raft initially.

---

# 191. Why

Lower latency/complexity.

---

# 192. Multi-AZ

Good.

---

# 193. Multi-Region Raft

Possible later only with measured need.

---

# 194. Clock

Raft safety does not depend on synchronized wall clocks.

---

# 195. Wall Clock

Used for diagnostics/timeouts but not consensus ordering.

---

# 196. Lease Expiry

Business leases still use server/control time semantics.

---

# 197. Coordination Lease vs Time Lease

Role ownership tied to epoch/leadership, not just clock expiry.

---

# 198. DB Advisory Locks

Can still be used for local narrow DB work.

---

# 199. When DB Lock Is Enough

```text
single row backfill
migration batch ownership
local projection rebuild
```

---

# 200. When Raft Helps

```text
cluster-wide role ownership
global maintenance
scheduler epoch
```

---

# 201. No Duplicate Coordination System

Do not use Raft + ZooKeeper + etcd simultaneously unless external interoperability forces it.

---

# 202. Kubernetes

Running Forgeyard on Kubernetes does not mean Kubernetes leader election should replace internal coordination semantics.

---

# 203. Optional K8s Leader Election

Could be deployment optimization, not core authority.

---

# 204. Recommendation

Keep Forgeyard coordination independent.

---

# 205. Pod Rescheduling

ClusterNodeId persisted in node state or explicitly re-enrolled.

---

# 206. Ephemeral Pods

Learner/voter storage needs persistent volume.

---

# 207. Node Identity vs Pod Name

Never use pod name as stable ClusterNodeId.

---

# 208. Autoscaling Daemons

Voting membership should not churn rapidly.

---

# 209. API Replicas

Can scale stateless non-voting/front-end replicas separately if architecture later supports split API/coordination roles.

---

# 210. Initial Simplicity

Same daemon binary, 3 voting nodes.

---

# 211. Coordination Role Separation

Future:

```text
API-only daemon
coordination voter
worker/control service
```

---

# 212. Cluster Endpoint

Internal-only.

---

# 213. Firewall

Allow Raft node-to-node only.

---

# 214. Certificate Rotation

Trust subsystem.

---

# 215. Node Revocation

Remove membership + revoke cert.

---

# 216. Stolen Node Credential

Membership check prevents unregistered node becoming voter.

---

# 217. Mutual Authentication

Mandatory.

---

# 218. Replay

Raft protocol implementation handles log message protocol safety; application commands include epoch/state validation.

---

# 219. DoS

Bound:

```text
message size
connections
join attempts
proposals
```

---

# 220. Proposal Authorization

Only internal trusted node/services can propose coordination commands.

---

# 221. Public API Cannot Directly Propose Raw Raft Commands

Critical.

---

# 222. Admin Action

Public API calls typed cluster service.

---

# 223. Cluster Service

Validates authz/policy then proposes safe command.

---

# 224. Cluster Permissions

```text
cluster.read
cluster.manage
cluster.maintenance
cluster.recover
```

---

# 225. Membership Change Authorization

Strong admin permission.

---

# 226. Step-Up

Recommended for:

```text
remove voter
force recovery
```

---

# 227. Audit

Every membership/maintenance/recovery action audited.

---

# 228. CLI

```text
forgeyard cluster status
forgeyard cluster members
forgeyard cluster add
forgeyard cluster promote
forgeyard cluster drain
forgeyard cluster remove
forgeyard cluster transfer-leader
forgeyard cluster maintenance
forgeyard cluster doctor
```

---

# 229. `cluster status`

Shows:

```text
ClusterId
leader
term
quorum
members
role epochs
```

---

# 230. `cluster add`

Creates join token/learner.

---

# 231. `cluster promote`

Learner -> voter after catch-up.

---

# 232. `cluster drain`

Transfers coordinated roles away.

---

# 233. `cluster remove`

Safe membership change.

---

# 234. `cluster recover`

Separate dangerous command.

---

# 235. UI

Admin cluster page:

```text
Overview
Members
Leadership
Roles/Epochs
Quorum
Maintenance
Replication Lag
Recovery
Audit
```

---

# 236. Member Card

Shows:

```text
node ID
version
role
state
replication lag
last contact
readiness
```

---

# 237. Leader Badge

Current only.

---

# 238. Quorum Warning

Prominent.

---

# 239. Recovery UI

Do not make destructive DR one-click.

---

# 240. Health Integration

Checks:

```text
leader known
quorum available
replication lag
local raft storage
membership consistency
```

---

# 241. Doctor

```text
forgeyard cluster doctor
```

---

# 242. Doctor Checks

```text
connectivity among voters
cert validity
log replication
snapshot state
DB epoch consistency
stale role owner
```

---

# 243. DB/Epoch Consistency

Example:

```text
current scheduler epoch in coordination state
>= any persisted lease creation epoch
```

---

# 244. Reconciliation

Cluster reconciler checks:

```text
role owner node removed
expired/stale exclusive op
DB lock without current epoch
stale scheduler dispatch
maintenance mismatch
```

---

# 245. Exclusive Operation Recovery

If leader fails mid-operation:

new leader inspects authoritative DB/external state before continuing.

---

# 246. Never Assume Incomplete Means Not Executed

Same Unknown-side-effect principle.

---

# 247. Release Coordinator Recovery

Inspect release/publication state.

---

# 248. Deployment Coordinator Recovery

Inspect provider state.

---

# 249. Scheduler Recovery

Rebuild queue from DB.

---

# 250. Coordination Metrics

```text
raft_term
raft_leader_changes_total
raft_commit_latency
raft_replication_lag
raft_proposals_failed
cluster_quorum_health
coordination_fenced_operations
scheduler_epoch
```

---

# 251. Metric Cardinality

Node role/index okay if bounded; avoid arbitrary IDs in labels where possible.

---

# 252. Tracing

```text
coordination.propose
coordination.commit
coordination.leader_change
coordination.membership
coordination.fence
```

---

# 253. Logs

Leadership changes at INFO.

Quorum loss ERROR/WARN.

---

# 254. Alerting

Alert on:

```text
quorum lost
leader churn
follower lag
snapshot failure
fenced stale node
```

---

# 255. SLO

Example:

```text
coordination leader available 99.9%
```

---

# 256. Testkit

```text
forgeyard-coordination-testkit/src/
├── lib.rs
├── cluster.rs
├── node.rs
├── partition.rs
├── epoch.rs
├── membership.rs
├── scheduler.rs
├── release.rs
└── assertions.rs
```

---

# 257. Unit Tests

State-machine commands deterministic.

---

# 258. Model Tests

Membership/epoch invariants.

---

# 259. Three-Node Failover Test

Kill leader -> new leader -> no duplicate leases.

---

# 260. Partition Test

1/2 partition -> majority leader only.

---

# 261. Quorum Loss Test

No coordination writes.

---

# 262. Stale Leader Test

Old leader DB lease mutation rejected by epoch.

---

# 263. Scheduler Epoch Test

Lease created with old scheduler epoch rejected.

---

# 264. Release Epoch Test

Old coordinator cannot publish after new epoch.

---

# 265. Deployment Epoch Test

Same.

---

# 266. Restart Test

Node incarnation changes.

---

# 267. Learner Join Test

Catch up before voter promotion.

---

# 268. Membership Removal Test

Removed node cannot rejoin as voter without authorization.

---

# 269. Rolling Upgrade Test

N/N-1 cluster remains available.

---

# 270. Leadership Transfer Test

Drain leader cleanly.

---

# 271. Snapshot Test

Compact/restart from snapshot.

---

# 272. Corrupt Local State Test

Node fails safe/rejoins.

---

# 273. DR Test

Recover coordination cluster while preserving PostgreSQL/CAS business state.

---

# 274. Agent Reconnect Test

Agent reconnects to another daemon after node loss.

---

# 275. SSE Reconnect Test

Client reconnects/backfills after API node loss.

---

# 276. DB Healthy / Raft Lost Test

System enters coordination-degraded mode, no unsafe scheduling/promotion.

---

# 277. Raft Healthy / DB Lost Test

No business writes despite coordination leader.

---

# 278. CAS Lost Test

Coordination unaffected; jobs depending CAS fail/degrade correctly.

---

# 279. Fuzzing

Fuzz coordination command decoding/schema.

---

# 280. Property Tests

Epoch monotonicity.

---

# 281. Chaos Tests

Kill/partition nodes during:

```text
lease creation
release promotion
deployment apply
membership change
```

---

# 282. Jepsen-Style Thinking

Test externally observable invariants under partitions.

---

# 283. Core Invariant Under Chaos

At most one current coordination epoch owner.

---

# 284. No Duplicate Authoritative Lease

DB transaction + epoch fencing.

---

# 285. No Split Release Promotion

Exclusive epoch + release lock + reconciliation.

---

# 286. Performance Test

Measure:

```text
proposal latency
leader failover
follower lag
```

---

# 287. Keep Raft Off Hot Path Where Possible

Job logs/CAS transfer never traverse consensus.

---

# 288. Scheduler Hot Path

Role epoch check lightweight.

---

# 289. Lease DB Transaction

Still primary data-plane/control-plane write.

---

# 290. Coordination Cache

Each daemon can cache committed coordination view.

---

# 291. Watch Stream

Coordination backend notifies local services of committed changes.

---

# 292. Watch Is Optimization

Services can reload current state.

---

# 293. Leadership Callback

Starts/stops leader-owned loops.

---

# 294. Stop Order

On leadership loss:

```text
mark role inactive
cancel leader loops
invalidate local epoch
```

---

# 295. Fencing Before Cancellation

Even if loop cancellation delayed, DB rejects old epoch.

---

# 296. Scheduler Loop

Requires active RoleLease guard.

---

# 297. RoleLease Guard

```rust
pub struct RoleGuard {
    pub role: CoordinatedRole,
    pub epoch: CoordinationEpoch,
}
```

---

# 298. Guard Use

Pass into mutation APIs.

---

# 299. Type-System Enforcement

High-risk coordinator methods require `RoleGuard`.

---

# 300. Example

```rust
scheduler.try_lease_job(&scheduler_guard, job)
```

---

# 301. Release Example

```rust
release.promote(&release_guard, release_id)
```

---

# 302. Guard Validation

Store/service checks guard epoch against current coordination epoch.

---

# 303. No Boolean `is_leader`

Avoid:

```rust
if is_leader { ... }
```

Use typed guard carrying epoch.

---

# 304. Lost Guard

Cannot mint locally.

Only coordination service issues.

---

# 305. Capability Pattern

Strong Rust type-system alignment.

---

# 306. Maintenance Guard

Could similarly gate destructive cluster ops.

---

# 307. Cluster Read Model

Any node can show current committed view.

---

# 308. Linearizable Reads

Needed only for coordination-critical checks.

---

# 309. Stale Read

Okay for UI diagnostics with freshness marker.

---

# 310. Coordination API

```rust
pub trait CoordinationService {
    async fn acquire_role(
        &self,
        role: CoordinatedRole,
    ) -> Result<RoleGuard, CoordinationError>;

    async fn current_view(
        &self,
    ) -> Result<CoordinationView, CoordinationError>;
}
```

---

# 311. Role Acquisition

Usually automatic based on committed assignment.

---

# 312. User Services

Do not manually acquire scheduler role.

---

# 313. Internal Bootstrap

Cluster service manages.

---

# 314. Error Model

```rust
pub enum CoordinationError {
    NotLeader,
    NoQuorum,
    Fenced,
    MembershipConflict,
    JoinRejected,
    IncompatibleVersion,
    StorageFailure,
    TransportFailure,
    Internal,
}
```

---

# 315. Retry Mapping

`NotLeader`:

```text
refresh/retry
```

`NoQuorum`:

```text
wait/alert
```

`Fenced`:

```text
stop role
```

---

# 316. NoQuorum API

Expose safe degraded response.

---

# 317. Cluster Bootstrap Config

RON.

---

# 318. Example

```ron
(
    cluster: (
        mode: Distributed,
        node_id: "node-a",
        listen: "10.0.0.10:7443",
        seeds: [
            "10.0.0.11:7443",
            "10.0.0.12:7443",
        ],
    ),
)
```

---

# 319. NodeId Config

Actual stable ID persisted, not human string alone.

---

# 320. Seed List

Discovery only.

---

# 321. TLS Config

SecretRef/trust refs.

---

# 322. Bootstrap Single Node

For initial cluster creation only.

---

# 323. Transition to 3 Nodes

Add learners and promote.

---

# 324. Single-Node Distributed Dev

Allowed, no HA guarantee.

---

# 325. Standalone Mode

No Raft network.

---

# 326. Standalone RoleGuard

Local implementation creates epoch 1 etc.

---

# 327. Same Tests

Coordination API conformance across local/Raft backend.

---

# 328. Implementation Phase 1 — Coordination Model/API

Implement:

```text
ClusterNodeId
ClusterId
CoordinationEpoch
RoleGuard
CoordinationBackend
```

---

# 329. Phase 2 — Local Backend

Standalone semantics.

---

# 330. Phase 3 — Raft 3-Node Cluster

Membership/election/snapshot.

---

# 331. Phase 4 — Scheduler Epoch Fencing

Integrate lease creation.

---

# 332. Phase 5 — HA API/Agent Reconnect

Multi-daemon fleet.

---

# 333. Phase 6 — Exclusive Release/Deployment Coordination

Only where needed.

---

# 334. Phase 7 — Rolling Upgrade

Version/capability gating.

---

# 335. Phase 8 — Cluster Doctor/Health

Operational readiness.

---

# 336. Phase 9 — DR Recovery

Documented procedure/tooling.

---

# 337. Phase 10 — Chaos Hardening

Partitions/failures/load.

---

# 338. Acceptance Tests

1. PostgreSQL remains business metadata authority.
2. CAS bytes never enter Raft.
3. Run/Job rows never enter Raft.
4. Raft state remains narrow and bounded.
5. Three-node cluster elects one leader.
6. Isolated minority cannot commit coordination writes.
7. Old leader cannot create authoritative new leases after fencing.
8. Scheduler mutations carry current epoch/guard.
9. Leadership loss invalidates role guard.
10. Restarted node gets new incarnation.
11. New member joins as learner before voter.
12. Removed node cannot keep coordination authority.
13. Quorum loss blocks coordination-sensitive operations.
14. Existing valid job attempts can continue according to DB lease semantics.
15. New leader reconstructs scheduler/release/deploy work from DB.
16. No correctness depends on in-memory leader state.
17. API clients can hit any healthy daemon.
18. Agent can reconnect to another daemon after failure.
19. SSE/WS reconnect/backfill handles API node loss.
20. Release promotion never relies only on process-local lock.
21. Deployment exclusive actions are fenced where required.
22. Rolling upgrade supports declared N/N-1 matrix.
23. Cluster feature activation waits for compatible voters.
24. Raft log/snapshots are bounded/compacted.
25. Local Raft storage corruption fails safe.
26. DR recovery preserves PostgreSQL/CAS business state.
27. Cluster membership changes are audited.
28. Node-to-node traffic uses mTLS.
29. Public API cannot submit raw coordination commands.
30. Standalone uses same coordination semantics without network Raft.
31. WAN/global Raft is not required for initial production.
32. Kubernetes does not become hidden coordination authority.
33. Typed RoleGuard replaces boolean leader checks in high-risk paths.
34. Chaos tests show no split authoritative lease/promotion.
35. Forgeyard's own HA deployment uses this coordination architecture.

---

# 339. Production Readiness Gates

Do not call HA/coordination production-ready until:

```text
3-node failover proven
scheduler epoch fencing proven
membership lifecycle tested
quorum-loss degraded behavior tested
agent/API failover works
rolling upgrades tested
snapshot/restart tested
cluster doctor available
node mTLS/revocation complete
DR procedure exercised
chaos tests pass
```

---

# 340. Architectural Invariants

1. Raft is not the business database;
2. PostgreSQL/Neon remains metadata authority;
3. CAS remains artifact/data authority;
4. coordination state is small and bounded;
5. one valid consensus leader per term;
6. epochs fence stale leaders;
7. high-risk role mutations require typed RoleGuard;
8. old guards cannot mutate after epoch advance;
9. scheduler lease correctness remains DB-transactional;
10. coordination strengthens, not replaces, DB invariants;
11. membership is Raft-authoritative;
12. new voters catch up as learners first;
13. quorum loss stops consensus-sensitive writes;
14. existing persisted business state remains readable/reconcilable;
15. new leaders reconstruct from DB, not memory;
16. API can be served by non-leader nodes;
17. agents can reconnect to any valid daemon endpoint;
18. node identity is not pod/hostname;
19. cluster node mTLS is mandatory;
20. public clients cannot issue raw Raft commands;
21. membership/recovery actions are audited;
22. rolling upgrades respect protocol/state compatibility;
23. coordination snapshots are compact and versioned;
24. DR never silently invents a new cluster from stale state;
25. global/WAN Raft is optional, not baseline;
26. K8s leader election is not hidden authority;
27. maintenance state can be globally coordinated;
28. standalone shares role/epoch semantics through local backend;
29. reconciliation follows leadership/failover;
30. Forgeyard dogfoods its HA system.

---

# 341. Final Target Architecture

```text
                   External Clients / Agents
                            │
                            ▼
                    Load-Balanced Daemons
                ┌───────────┼───────────┐
                ▼           ▼           ▼
              node A      node B      node C
                │           │           │
                └───────────┼───────────┘
                            ▼
                       Raft Consensus
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
             Membership   Epochs    Role Ownership
                            │
                            ▼
                    PostgreSQL / Neon
                authoritative domain state
                            │
                            ▼
                            CAS
```

---

# 342. Final Architectural Position

Scheduler authority:

```text
Raft role assignment
  ↓
SchedulerEpoch
  ↓
typed Scheduler RoleGuard
  ↓
DB transaction validates epoch
  ↓
JobLease / reservation
```

Failover:

```text
leader dies
  ↓
new Raft leader
  ↓
new coordination epoch
  ↓
old role guards fenced
  ↓
new coordinator scans PostgreSQL
  ↓
reconciliation resumes work
```

Release/deployment exclusivity:

```text
persisted desired operation
+
DB lock/version
+
optional coordination epoch
  ↓
one current coordinator
  ↓
external effect
  ↓
reconciliation
```

The key guarantee is:

> **Forgeyard achieves high availability without turning consensus into the center of the entire product. Raft answers “who currently owns this coordination responsibility, and under which epoch?” while PostgreSQL answers “what is the durable business truth?” and CAS answers “what are the immutable bytes?”**

---

# 343. New-Repository Sequence

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
