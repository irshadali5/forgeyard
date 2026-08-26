# 43 — Forgeyard Runner Fleet Autoscaling, Capacity Provisioning & Infrastructure Provider System Architecture

**Document type:** Core Runner Fleet, Elastic Capacity, Provisioning, Autoscaling & Infrastructure Provider System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** runner pools, elastic worker fleets, autoscaling, warm capacity, scale-to-zero, provisioning adapters, cloud/on-prem/Kubernetes/VM/bare-metal capacity, spot/preemptible workers, demand forecasting, cost-aware scaling, drain/termination safety, image lifecycle, capacity reservations, trust bootstrap, and fleet reconciliation  
**Architecture style:** Desired-state capacity management, scheduler-driven demand signals, provider-neutral provisioning, explicit trust bootstrap, bounded elasticity, conservative termination, reconciliation-first correctness, and no autoscaler authority over job state, policy, or leases  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Scheduler, Runner/Agent, Sandbox/Executor, HA/Coordination, Multi-Tenancy/Quotas, Configuration, Security, Observability, Operations/DR, Device Lab, RBE, and Entitlements. This subsystem adds elastic capacity without making infrastructure-provider state authoritative for Forgeyard execution truth.

---

# 1. Purpose

Forgeyard can schedule jobs onto runners, but production deployments need to answer:

```text
how many runners should exist?
when should new runners start?
when should idle runners disappear?
how do we avoid capacity shortages?
how do we handle spot/preemptible workers?
how do we maintain warm pools?
how do we scale different OS/architecture/GPU pools?
how do we safely drain before termination?
how do we bootstrap trust on newly created runners?
```

The central rule is:

> **The autoscaler manages capacity, not work correctness. The scheduler remains the authority for placement and leases; the autoscaler only attempts to make enough eligible capacity available.**

A second rule is:

> **Provisioning is desired-state and reconciled. Provider API calls are external effects with ambiguous outcomes, so Forgeyard must inspect before retrying and never assume exactly-once VM/container creation.**

A third rule is:

> **A newly provisioned machine is not trusted merely because the cloud provider created it. It must complete Forgeyard enrollment, identity validation, capability verification, and runner-health admission before receiving work.**

---

# 2. Architectural Position

```text
                       Scheduler
                          │
                    demand signals
                          │
                          ▼
                    Capacity Planner
                          │
                          ▼
                    Fleet Autoscaler
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
           Cloud        Kubernetes    On-Prem
             │            │            │
             └────────────┼────────────┘
                          ▼
                   Provisioned Hosts
                          │
                          ▼
                Bootstrap / Enrollment
                          │
                          ▼
                  Runner Admission
                          │
                          ▼
                      Scheduler
```

---

# 3. Goals

The subsystem MUST:

1. define runner fleet identity;
2. define pool desired state;
3. define capacity classes;
4. support scale up;
5. support scale down;
6. support warm pools;
7. support scale-to-zero;
8. support min/max capacity;
9. support cloud VMs;
10. support Kubernetes workers;
11. support on-prem provisioners;
12. support bare-metal/manual pools;
13. support GPU pools;
14. support macOS pools;
15. support Windows pools;
16. support spot/preemptible workers;
17. support termination/drain;
18. support provider reconciliation;
19. support runner bootstrap;
20. support image/version rollout;
21. support capacity reservations;
22. support cost-aware decisions;
23. support quota/fairness integration;
24. support demand forecasting;
25. support failure-domain spreading;
26. support multi-region;
27. support health/doctor;
28. support audit;
29. support standalone mode;
30. remain scheduler-subordinate.

---

# 4. Non-Goals

The subsystem does not:

```text
schedule jobs directly
create JobLease
decide job success/failure
replace runner trust
replace tenant quotas
replace deployment autoscaling
replace Kubernetes itself
```

---

# 5. Workspace Structure

```text
crates/fleet/
├── forgeyard-fleet/
├── forgeyard-fleet-model/
├── forgeyard-fleet-capacity/
├── forgeyard-fleet-demand/
├── forgeyard-fleet-autoscale/
├── forgeyard-fleet-provision/
├── forgeyard-fleet-bootstrap/
├── forgeyard-fleet-drain/
├── forgeyard-fleet-cost/
├── forgeyard-fleet-image/
├── forgeyard-fleet-reconcile/
├── forgeyard-fleet-health/
└── forgeyard-fleet-testkit/
```

Provider adapters:

```text
crates/fleet-providers/
├── forgeyard-fleet-aws/
├── forgeyard-fleet-azure/
├── forgeyard-fleet-gcp/
├── forgeyard-fleet-kubernetes/
├── forgeyard-fleet-openstack/
├── forgeyard-fleet-libvirt/
├── forgeyard-fleet-baremetal/
└── forgeyard-fleet-manual/
```

Use modules first; split only where provider SDK/runtime dependencies justify.

---

# 6. RunnerFleetId

```rust
pub struct RunnerFleetId(Ulid);
```

Stable administrative identity.

---

# 7. CapacityClassId

```rust
pub struct CapacityClassId(Digest);
```

Describes a class of equivalent schedulable capacity.

---

# 8. Capacity Class

```rust
pub struct CapacityClass {
    pub id: CapacityClassId,
    pub platform: PlatformDescriptor,
    pub capabilities: CapabilitySet,
    pub resources: ResourceCapacity,
    pub trust: RunnerTrustClass,
    pub image: RunnerImageRef,
}
```

---

# 9. Examples

```text
linux-x86_64-standard
linux-x86_64-gpu
windows-x86_64
macos-arm64
android-device-host
```

---

# 10. Runner Fleet

```rust
pub struct RunnerFleet {
    pub id: RunnerFleetId,
    pub class: CapacityClassId,
    pub provider: FleetProviderRef,
    pub scaling: FleetScalingPolicy,
    pub placement: FleetPlacementPolicy,
}
```

---

# 11. Fleet Scope

Can be:

```text
installation
tenant-dedicated
project-dedicated
shared
```

---

# 12. Isolation Class

Integrate Part 27.

---

# 13. FleetScalingPolicy

```rust
pub struct FleetScalingPolicy {
    pub min: u32,
    pub max: u32,
    pub warm: u32,
    pub scale_to_zero: bool,
    pub scale_up: ScaleUpPolicy,
    pub scale_down: ScaleDownPolicy,
}
```

---

# 14. Minimum Capacity

Guaranteed baseline.

---

# 15. Maximum Capacity

Hard provider/governance ceiling.

---

# 16. Warm Capacity

Idle ready runners kept for latency.

---

# 17. Scale to Zero

Valid only if bootstrap latency acceptable.

---

# 18. Protected Pools

Signing workers generally should not autoscale like normal runners.

---

# 19. Signing Fleet

If automated, separate highly restricted policy.

---

# 20. Demand Signal

Derived from scheduler queue.

---

# 21. DemandSnapshotId

```rust
pub struct DemandSnapshotId(Digest);
```

---

# 22. Demand Snapshot

```rust
pub struct CapacityDemandSnapshot {
    pub id: DemandSnapshotId,
    pub queued_jobs: Vec<QueuedDemandClass>,
    pub running: Vec<RunningCapacityUsage>,
    pub timestamp: Timestamp,
}
```

---

# 23. Demand Class

Groups jobs by hard requirements.

---

# 24. Hard Requirements

Examples:

```text
platform
arch
GPU
device
trusted signing
confidential compute
toolchain
tenant isolation
```

---

# 25. Autoscaler Must Not Reinterpret Scheduler Eligibility

Critical.

---

# 26. Scheduler Export

Scheduler exposes normalized unmet demand.

---

# 27. UnmetDemand

```rust
pub struct UnmetDemand {
    pub requirement: SchedulingRequirementSet,
    pub count: u32,
    pub oldest_wait: Duration,
    pub priority_class: PriorityClass,
}
```

---

# 28. Capacity Mapping

Fleet planner maps requirement sets to eligible capacity classes.

---

# 29. No Eligible Fleet

Explicit diagnostic.

---

# 30. Scale-Up Decision

```rust
pub struct ScaleUpDecision {
    pub fleet: RunnerFleetId,
    pub add: u32,
    pub reason: ScaleReason,
}
```

---

# 31. Scale Reason

```text
queued demand
warm pool deficit
reservation
manual
forecast
```

---

# 32. Scale-Down Decision

```rust
pub struct ScaleDownDecision {
    pub fleet: RunnerFleetId,
    pub remove: u32,
    pub reason: ScaleDownReason,
}
```

---

# 33. Scale Down Never Kills Active Lease Blindly

Critical.

---

# 34. Drain First

```text
select runner
  ↓
mark Draining
  ↓
scheduler stops new leases
  ↓
wait active attempts
  ↓
terminate
```

---

# 35. Emergency Termination

Only provider preemption/failure/security incident.

---

# 36. Preemptible/Spot Capacity

```rust
pub enum CapacityLifetimeClass {
    Stable,
    Preemptible,
    Ephemeral,
}
```

---

# 37. Scheduler Awareness

Jobs can declare:

```text
preemptible_allowed
```

or policy derives.

---

# 38. Long/critical Jobs

Prefer stable capacity.

---

# 39. Retry

Preempted attempt follows normal retry semantics.

---

# 40. Infrastructure Failure Classification

Preemption = infrastructure failure, not test failure.

---

# 41. Spot Interruption Notice

If provider offers notice:

```text
drain immediately
stop new work
checkpoint if job supports
```

---

# 42. Checkpointing

Optional job capability.

---

# 43. No General Transparent Checkpoint Claim

Critical.

---

# 44. Provider Adapter

```rust
#[async_trait]
pub trait FleetProvider {
    async fn inspect(
        &self,
        fleet: &RunnerFleet,
    ) -> Result<ProviderFleetObservation, FleetProviderError>;

    async fn provision(
        &self,
        request: ProvisionRequest,
    ) -> Result<ProvisionOperation, FleetProviderError>;

    async fn terminate(
        &self,
        request: TerminateRequest,
    ) -> Result<TerminateOperation, FleetProviderError>;
}
```

---

# 45. Provider External Effects

At-least-once/ambiguous.

---

# 46. Operation State

```rust
pub enum ProviderOperationState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
    Unknown,
}
```

---

# 47. Unknown

Inspect provider before retry.

---

# 48. Idempotency

Use provider request tokens where available.

---

# 49. ProviderInstanceId

Stored separately from Forgeyard RunnerId.

---

# 50. Provider Instance Is Not Runner Identity

Critical.

---

# 51. Provisioned Host Lifecycle

```text
Requested
Provisioning
Bootstrapping
Enrolling
Healthy
Draining
Terminating
Terminated
Failed
Quarantined
```

---

# 52. Capacity Instance

```rust
pub struct CapacityInstance {
    pub id: CapacityInstanceId,
    pub fleet: RunnerFleetId,
    pub provider_instance: ProviderInstanceRef,
    pub runner: Option<RunnerId>,
    pub state: CapacityInstanceState,
}
```

---

# 53. CapacityInstanceId

Stable Forgeyard object.

---

# 54. Bootstrap

Cloud-init/ignition/custom image startup.

---

# 55. Bootstrap Secret

Short-lived enrollment token.

---

# 56. Enrollment Token

Single-use, scoped to fleet/class, expiring, hashed server-side.

---

# 57. No Long-Lived Static Runner Token in Image

Critical.

---

# 58. Golden Runner Image

Immutable image reference.

---

# 59. RunnerImageId

```rust
pub struct RunnerImageId(Digest);
```

---

# 60. Image Contents

Potential:

```text
OS
forgeyard-agent
sandbox dependencies
base tools
```

---

# 61. Toolchains

Prefer mounted/fetched immutable toolchains rather than baking everything.

---

# 62. Image Provenance

Signed/provenanced.

---

# 63. Image Trust

High-assurance fleets require trusted image pipeline.

---

# 64. Image Rotation

New image version.

---

# 65. No In-Place Mutable Fleet Image

Critical.

---

# 66. Fleet Image Rollout

```text
new image
  ↓
canary capacity
  ↓
health
  ↓
replace old instances gradually
```

---

# 67. Image Drift

Detect provider instance not matching desired image.

---

# 68. Bootstrap Verification

Agent reports version/capabilities, but privileged capabilities are verified/provisioned.

---

# 69. TPM/Attestation

Optional high-assurance.

---

# 70. Runner Admission

New runner is not Available until:

```text
mTLS enrollment
trust assignment
capability validation
health check
time sanity
version compatibility
```

---

# 71. Admission State

```rust
pub enum RunnerAdmissionState {
    Pending,
    Verified,
    Rejected,
    Quarantined,
}
```

---

# 72. Scheduler Visibility

Only Verified runners.

---

# 73. Provider Credentials

SecretRef/workload identity.

---

# 74. Prefer Cloud Workload Federation

Avoid static cloud keys.

---

# 75. Autoscaler Identity

Separate service/workload principal.

---

# 76. Least Privilege

Autoscaler can:

```text
create/delete instances in allowed fleet resources
```

not edit IAM broadly.

---

# 77. Network Bootstrap

New runner connects outbound to daemon where possible.

---

# 78. No Public SSH Required

Critical.

---

# 79. SSH

Emergency/manual administration only, not control plane.

---

# 80. Provider Firewall

Runner inbound minimized.

---

# 81. Cloud Metadata

Agent bootstrap may access provider identity endpoint; build sandbox does not.

---

# 82. Bootstrap vs Build Boundary

Separate credentials/process context.

---

# 83. Kubernetes Provider

Can provision pods/jobs/nodes.

---

# 84. Pod Runner

Useful for moderate trust workloads.

---

# 85. Kubernetes Node Autoscaling

Could delegate node scale to Karpenter/Cluster Autoscaler.

---

# 86. Forgeyard Role

Manage runner workload desired state, not duplicate Kubernetes node-scaling logic blindly.

---

# 87. KEDA

Optional event-based scaling.

---

# 88. K8s Operator

Optional.

---

# 89. Bare Metal

Provisioning may be:

```text
manual
PXE
Redfish
custom provider
```

---

# 90. Bare-Metal Scale Down

May mean power off/drain, not delete.

---

# 91. macOS Capacity

Can use:

```text
Mac minis
MacStadium-like provider
cloud Mac hosts where supported
```

---

# 92. Real macOS Required

Existing invariant.

---

# 93. macOS Provisioning Latency

Often high; warm pool more important.

---

# 94. Windows Capacity

VM image + agent enrollment.

---

# 95. GPU Capacity

Scarce.

---

# 96. GPU Fleet Policy

Warm pool may be zero/low due cost.

---

# 97. GPU Resource Verification

Agent/provider cross-check.

---

# 98. Capacity Reservation

```rust
pub struct CapacityReservation {
    pub id: CapacityReservationId,
    pub scope: ResourceScope,
    pub class: CapacityClassId,
    pub quantity: u32,
    pub start: Timestamp,
    pub end: Timestamp,
}
```

---

# 99. Purpose

Guarantee capacity for:

```text
release window
large migration
benchmark campaign
customer SLA
```

---

# 100. Reservation Is Not Job Lease

Critical.

---

# 101. Reservation Feeds Autoscaler Target

---

# 102. Scheduler Fairness Still Applies

Unless reservation policy includes dedicated pool.

---

# 103. Dedicated Fleet

Explicit.

---

# 104. Cost Model

Optional/advisory.

---

# 105. CapacityCost

```rust
pub struct CapacityCost {
    pub unit_cost: Decimal,
    pub currency: CurrencyCode,
    pub billing_unit: BillingUnit,
}
```

---

# 106. Cost Source

Provider pricing/config.

---

# 107. Current Cloud Prices

External dynamic data; provider adapter can refresh.

---

# 108. Correctness

Cost never overrides hard scheduling requirement.

---

# 109. Cost-Aware Fleet Choice

Among equivalent eligible capacity classes only.

---

# 110. Example

```text
same platform/trust/resources
```

choose cheaper region/provider if policy permits.

---

# 111. Data Residency

Can constrain region.

---

# 112. Latency

Can constrain locality.

---

# 113. Failure Domain

Spread capacity across zones/providers.

---

# 114. FleetPlacementPolicy

```rust
pub struct FleetPlacementPolicy {
    pub regions: Vec<RegionSelector>,
    pub zones: Vec<ZoneSelector>,
    pub spread: SpreadPolicy,
}
```

---

# 115. Spread Policy

```text
none
zone
region
provider
```

---

# 116. HA Control Plane vs Runner Fleet

Runner capacity can span more regions than Raft.

---

# 117. Demand Forecasting

Optional.

---

# 118. Baseline

Reactive autoscaling.

---

# 119. Forecasting Inputs

```text
historical queue
scheduled release windows
time-of-day patterns
reservations
```

---

# 120. Forecast Is Advisory

Never correctness authority.

---

# 121. Forecast Confidence

Explicit.

---

# 122. Warm Pool Prediction

Can prewarm before known peak.

---

# 123. Scale-Up Formula

Baseline conceptual:

```text
required_instances =
ceil(
  unmet_resource_demand
  /
  capacity_per_instance
)
```

plus warm target.

---

# 124. Heterogeneous Jobs

Use demand classes, not single CPU count.

---

# 125. Bin Packing Estimate

Autoscaler can estimate.

---

# 126. Scheduler Makes Actual Placement

Critical.

---

# 127. Scale-Up Rate Limit

Avoid provider storms.

---

# 128. Scale-Down Hysteresis

Avoid thrashing.

---

# 129. IdleGracePeriod

```rust
pub struct IdleGracePeriod(Duration);
```

---

# 130. Scale-Down Criteria

Runner:

```text
no active lease
not reserved
healthy enough to drain
idle beyond grace
not pinned
```

---

# 131. Runner Pin

Manual/diagnostic.

---

# 132. Pin Expiry

Recommended.

---

# 133. Fleet Freeze

```rust
pub enum FleetOperationalState {
    Active,
    ScaleUpOnly,
    Frozen,
    Draining,
}
```

---

# 134. Incident Use

Freeze autoscaling/provider writes.

---

# 135. Security Incident

Can quarantine entire fleet/image version.

---

# 136. Image Compromise

```text
stop new provisioning
drain fleet
quarantine outputs/cache
rotate image/enrollment
```

---

# 137. Provider Compromise

Freeze provider, shift to alternate fleet if possible.

---

# 138. Runner Replacement

Preferred over long-lived mutable repair.

---

# 139. Ephemeral Runners

Recommended for hostile workloads.

---

# 140. Runner Lifetime

```rust
pub enum RunnerLifetimePolicy {
    Persistent,
    MaxAge(Duration),
    OneJob,
    MaxJobs(u32),
}
```

---

# 141. OneJob

High isolation but expensive.

---

# 142. MaxAge

Reduces drift.

---

# 143. Persistent

Useful on-prem.

---

# 144. Cleanup

Runner workspace reset before reuse.

---

# 145. If Cleanup Fails

Quarantine.

---

# 146. Capacity Reconciler

Desired vs observed.

---

# 147. Desired Fleet State

```rust
pub struct DesiredFleetState {
    pub fleet: RunnerFleetId,
    pub desired_ready: u32,
    pub desired_image: RunnerImageId,
}
```

---

# 148. Observed Fleet State

```rust
pub struct ObservedFleetState {
    pub ready: u32,
    pub provisioning: u32,
    pub draining: u32,
    pub failed: u32,
}
```

---

# 149. Reconcile Loop

```text
observe scheduler demand
observe provider
observe runners
compute desired
apply bounded delta
reconcile
```

---

# 150. No Long DB Transaction Around Provider API

Existing invariant.

---

# 151. Provision Intent

Persist before external call.

---

# 152. Provider Operation

External effect.

---

# 153. Reconcile Unknown

Inspect.

---

# 154. Duplicate VM Prevention

Provider idempotency token + tags + inspection.

---

# 155. Resource Tags

Include:

```text
Forgeyard fleet ID
capacity instance ID
tenant/project scope if dedicated
image ID
```

---

# 156. Tags Are Discovery Aid

Not security authority.

---

# 157. Orphan Provider Resource

Detected.

---

# 158. Orphan Reconciler

Can terminate after policy/grace.

---

# 159. Unknown Resource

Do not delete arbitrary infrastructure.

Critical.

---

# 160. Provider Ownership Proof

Require Forgeyard-managed tags/metadata and known resource ID.

---

# 161. Bootstrap Timeout

Failed capacity.

---

# 162. Enrollment Timeout

Terminate/quarantine.

---

# 163. Health Timeout

Do not count as ready.

---

# 164. Provisioning Failure Backoff

Per fleet/provider.

---

# 165. Circuit Breaker

If provider failing repeatedly.

---

# 166. Alternate Fleet

Can satisfy demand if equivalent.

---

# 167. Capacity Degraded State

```rust
pub enum FleetHealth {
    Healthy,
    CapacityConstrained,
    ProviderDegraded,
    BootstrapDegraded,
    Unhealthy,
}
```

---

# 168. Scheduler Visibility

Scheduler sees current runner reality.

---

# 169. Autoscaler Health Does Not Fabricate Runners

---

# 170. Quotas

Part 27 limits capacity consumption.

---

# 171. Tenant Max Concurrency

Autoscaler should not scale beyond demand allowed by quota.

---

# 172. Fairness

Scheduler-level.

---

# 173. Entitlement

Part 30 may limit premium capacity classes.

---

# 174. Entitlement Cannot Grant Scheduler capability directly.

---

# 175. Policy

Can restrict:

```text
allowed providers
regions
spot usage
dedicated fleets
GPU
hostile workload isolation
```

---

# 176. Configuration

Part 39 defines fleet policies.

---

# 177. Runtime Override

Can temporarily raise/lower min/max with authorization.

---

# 178. Scale Override

```rust
pub struct FleetScaleOverride {
    pub fleet: RunnerFleetId,
    pub desired_min: Option<u32>,
    pub desired_max: Option<u32>,
    pub expires_at: Option<Timestamp>,
}
```

---

# 179. Expiry

Recommended.

---

# 180. Manual Scale

Still reconciled.

---

# 181. UI

Pages:

```text
Runner Fleets
Capacity
Autoscaling
Provisioning
Images
Reservations
Cost
```

---

# 182. Fleet Detail

Shows:

```text
ready
busy
idle
provisioning
draining
failed
desired
min/max/warm
```

---

# 183. Demand View

Shows unmet classes.

---

# 184. Scaling Decision View

Explain:

```text
why scaled
how many
which queue demand
```

---

# 185. No Opaque Autoscaler

Critical.

---

# 186. Cost View

Estimated.

---

# 187. Cost Estimate

Clearly marked approximate.

---

# 188. CLI

```text
forgeyard fleet list
forgeyard fleet status
forgeyard fleet scale
forgeyard fleet drain
forgeyard fleet freeze
forgeyard fleet reservation create
forgeyard fleet doctor
```

---

# 189. Dangerous Actions

Authz/audit.

---

# 190. API

Potential:

```text
GET  /v1/fleets
GET  /v1/fleets/{id}
GET  /v1/fleets/{id}/capacity
POST /v1/fleets/{id}/scale
POST /v1/fleets/{id}/drain
POST /v1/fleets/{id}/freeze
GET  /v1/capacity-demand
```

---

# 191. Permissions

```text
fleet.read
fleet.manage
fleet.scale
fleet.drain
fleet.freeze
fleet.image.manage
fleet.reservation.manage
```

---

# 192. Provider Credentials

Never exposed through API/UI.

---

# 193. Audit

Audit:

```text
fleet create/delete
min/max change
manual scale
fleet freeze
image change
reservation create
provider credential change
```

---

# 194. Autoscaler Routine Decisions

Operational events/telemetry, not audit every scale tick.

---

# 195. Notification

Examples:

```text
capacity exhausted
provider degraded
fleet max reached
bootstrap failures
spot interruption spike
```

---

# 196. Search/Analytics

Part 31.

---

# 197. Fleet Analytics

```text
utilization
queue wait
scale latency
provision latency
idle cost
spot interruption
```

---

# 198. Capacity Efficiency

Derived.

---

# 199. No Single "efficiency score" baseline.

---

# 200. Observability Metrics

```text
fleet_desired_instances
fleet_ready_instances
fleet_provisioning_instances
fleet_scale_up_total
fleet_scale_down_total
fleet_provision_latency_seconds
fleet_bootstrap_failures_total
fleet_provider_failures_total
fleet_spot_interruptions_total
```

---

# 201. Labels

Low-cardinality:

```text
fleet
provider_kind
capacity_class
result
```

Fleet count should remain controlled.

---

# 202. Tracing

```text
fleet.plan
fleet.provision
fleet.bootstrap
fleet.admit
fleet.drain
fleet.terminate
fleet.reconcile
```

---

# 203. Health

Checks:

```text
provider connectivity
bootstrap path
enrollment
image availability
capacity deficit
```

---

# 204. Doctor

```text
forgeyard fleet doctor
```

---

# 205. Doctor Checks

```text
provider auth
min/max sanity
image existence
enrollment token path
network egress
runner version compatibility
```

---

# 206. Standalone Mode

Autoscaling optional.

---

# 207. Local Standalone

Usually one local runner.

---

# 208. Standalone External Fleet

Could connect remote manual runners later.

---

# 209. Distributed Mode

Full fleet manager.

---

# 210. Multi-Region

Multiple fleets per capacity class.

---

# 211. Region Preference

Scheduler locality + fleet cost/policy.

---

# 212. Data Residency

Hard filter.

---

# 213. Failure Domain

Soft/hard spread.

---

# 214. Capacity Pooling

Shared fleet across tenants only if isolation/trust profile allows.

---

# 215. Dedicated Tenant Fleet

Strong isolation.

---

# 216. Hostile Tenant

VM-capable runner class.

---

# 217. Runner Image Build

Forgeyard itself can build runner images.

---

# 218. Image Release Pipeline

```text
build
scan
SBOM
sign
publish
```

---

# 219. Image Promotion

Exact image digest.

---

# 220. Provider Image Import

Cloud-specific copy/import.

---

# 221. Image Alias

Mutable human convenience only.

---

# 222. Fleet Desired Image

Exact digest/ID.

---

# 223. Image Rollback

Previous trusted image.

---

# 224. Compromised Image

Mark SecurityState.

---

# 225. New Instances

Blocked.

---

# 226. Existing Instances

Drain/quarantine based on severity.

---

# 227. Agent Version

Can be baked or updated separately Part 41.

---

# 228. Consistency

Fleet image + agent updater semantics explicit.

---

# 229. Capacity Reservation Scheduling

Reservation can prewarm capacity before start.

---

# 230. Reservation Grace

Pre-provision lead time.

---

# 231. Reservation End

Capacity returns to normal autoscaling.

---

# 232. Overlapping Reservations

Aggregate.

---

# 233. Cost Guardrails

```rust
pub struct FleetCostGuardrail {
    pub max_hourly_estimate: Option<Money>,
    pub max_daily_estimate: Option<Money>,
}
```

---

# 234. Cost Guardrail

Operational protection, not billing truth.

---

# 235. Hard Cost Ceiling

May block new capacity.

---

# 236. Result

Queue increases rather than violating configured ceiling.

---

# 237. Security Emergency

Can override with break-glass if policy.

---

# 238. Forecast Scale

Should respect cost guardrail.

---

# 239. Provider Quota Exhaustion

Explicit FleetHealth degradation.

---

# 240. Request Provider Quota

Human operational workflow.

---

# 241. Capacity Shortage

Scheduler queues jobs.

---

# 242. Never Misplace Jobs Just Because Preferred capacity unavailable.

Critical.

---

# 243. Alternative Capacity

Only if eligibility equivalence.

---

# 244. Toolchain Locality

Can influence fleet selection/prewarm.

---

# 245. CAS Locality

Can influence soft score.

---

# 246. Cache Warmup

Part 38 can prewarm new instances.

---

# 247. Warmup Failure

Runner still potentially usable.

---

# 248. Admission vs Warmup

Separate.

---

# 249. New Runner First Job

May have cold cache.

---

# 250. Capacity Forecasting Model

Versioned.

---

# 251. ForecastModelVersion

```rust
pub struct ForecastModelVersion(u16);
```

---

# 252. Baseline Reactive

No forecast required for correctness.

---

# 253. Testkit

```text
forgeyard-fleet-testkit/src/
├── lib.rs
├── demand.rs
├── scaling.rs
├── provider.rs
├── bootstrap.rs
├── drain.rs
├── image.rs
└── assertions.rs
```

---

# 254. Unit Tests

Scale math/min/max/warm.

---

# 255. Hard Requirement Test

Autoscaler maps only eligible class.

---

# 256. No Eligible Fleet Test

Diagnostic.

---

# 257. Drain Test

No active lease terminated.

---

# 258. Spot Interruption Test

Infrastructure retry classification.

---

# 259. Provider Unknown Outcome Test

Inspect before retry.

---

# 260. Duplicate Provision Test

Idempotency prevents/reconciles duplicates.

---

# 261. Orphan Resource Test

Only owned resources terminated.

---

# 262. Bootstrap Token Test

Single-use/expiry.

---

# 263. Runner Admission Test

Unverified runner cannot receive jobs.

---

# 264. Self-Reported Trust Test

Cannot elevate trust.

---

# 265. Image Drift Test

Detected/replaced.

---

# 266. Compromised Image Test

Provisioning blocked.

---

# 267. Cost Ceiling Test

Queues rather than unsafe fallback.

---

# 268. Reservation Test

Prewarms capacity.

---

# 269. Tenant Quota Test

Autoscaler does not scale for quota-blocked work.

---

# 270. Multi-Region Failure Test

Equivalent alternate fleet used.

---

# 271. Provider Credential Leak Test

Build process cannot access.

---

# 272. K8s Provider Test

Pod lifecycle/reconciliation.

---

# 273. macOS Pool Test

Warm capacity behavior.

---

# 274. GPU Pool Test

Scarcity/min-max behavior.

---

# 275. Reconciler Restart Test

Desired state converges.

---

# 276. DB Restart Test

No duplicate destructive operations.

---

# 277. HA Autoscaler Test

One logical fleet decision authority or DB-fenced claims.

---

# 278. Fuzzing

Fuzz provider metadata/scale config decoders.

---

# 279. Property Tests

Never desired < min or > max.

---

# 280. Chaos Tests

```text
provider outage
instance stuck provisioning
runner never enrolls
spot mass interruption
region outage
```

---

# 281. Scale Tests

Thousands of runners.

---

# 282. Burst Test

Large queue sudden arrival.

---

# 283. Thrash Test

Hysteresis prevents oscillation.

---

# 284. Implementation Phase 1 — Fleet Model/Manual Provider

Desired state first.

---

# 285. Phase 2 — Scheduler Demand Integration

Reactive scaling.

---

# 286. Phase 3 — Cloud VM Provider

One primary provider.

---

# 287. Phase 4 — Secure Bootstrap/Enrollment

Trust.

---

# 288. Phase 5 — Drain/Scale-Down

Safe termination.

---

# 289. Phase 6 — Images/Rollout

Immutable worker images.

---

# 290. Phase 7 — Kubernetes Provider

Cloud-native.

---

# 291. Phase 8 — Spot/Preemptible

Cost optimization.

---

# 292. Phase 9 — Reservations/Warm Pools

Latency/SLA.

---

# 293. Phase 10 — Multi-Region/Cost Awareness

Enterprise.

---

# 294. Phase 11 — Forecasting

Optimization.

---

# 295. Phase 12 — Scale/Chaos/Security Hardening

Production readiness.

---

# 296. Acceptance Tests

1. Autoscaler never creates JobLease or chooses job placement.
2. Scheduler remains execution placement authority.
3. Autoscaler consumes normalized unmet demand.
4. Capacity classes encode hard platform/capability/trust requirements.
5. Scale targets obey min/max/warm limits.
6. Scale down always drains before normal termination.
7. Active leases are not blindly killed.
8. Preemption is classified as infrastructure failure.
9. Provider operations are treated as ambiguous external effects.
10. Unknown provision/terminate outcomes are inspected before retry.
11. Provider instance ID is not RunnerId.
12. New host receives work only after Forgeyard admission.
13. Runner image is immutable/versioned.
14. Enrollment tokens are short-lived/single-use/scoped.
15. No static privileged runner token is baked into image.
16. Build workloads cannot access provider credentials.
17. Autoscaler provider credentials are least privilege.
18. Public SSH is not required for normal control.
19. Unverified/self-promoted capabilities do not satisfy scheduler trust.
20. Unknown provider resources are never deleted automatically.
21. Orphan cleanup requires ownership proof.
22. Tenant quota-blocked demand does not cause pointless scale-up.
23. Cost-aware choices only select among semantically eligible capacity.
24. Cost ceilings cause queueing rather than correctness downgrade.
25. Spot/preemptible capacity is used only where policy/job allows.
26. Hostile tenant workloads can require VM-capable fleets.
27. Fleet image compromise can freeze provisioning and drain affected runners.
28. Reservations are capacity promises, not job leases.
29. Warm pools and scale-to-zero are policy-controlled.
30. Reconciliation recovers after autoscaler/provider/DB restart.
31. Multi-region alternate capacity never bypasses residency/trust requirements.
32. Scheduler/cache locality remain soft optimizations.
33. Standalone mode works without autoscaling.
34. Distributed mode can scale thousands of runners.
35. Forgeyard dogfoods fleet autoscaling for its own CI where infrastructure permits.

---

# 297. Production Readiness Gates

Do not call fleet autoscaling production-ready until:

```text
scheduler/autoscaler authority separation is enforced
provider operations reconcile safely
secure bootstrap/enrollment works
drain-before-terminate is reliable
image identity/provenance is verified
quota/cost/trust constraints are respected
spot/preemption behavior is tested
orphan-resource safety passes
HA/restart convergence passes
burst/thrash/region-outage tests pass
```

---

# 298. Architectural Invariants

1. autoscaler manages capacity, not job correctness;
2. scheduler remains placement/lease authority;
3. fleet capacity classes encode hard eligibility semantics;
4. demand is derived from scheduler state;
5. min/max/warm bounds are enforced;
6. normal scale-down drains first;
7. provider calls are reconciled external effects;
8. provider instance identity is distinct from RunnerId;
9. newly created hosts are untrusted until admitted;
10. enrollment tokens are short-lived/scoped/single-use;
11. runner trust cannot be self-asserted;
12. provider credentials never enter build sandboxes;
13. runner images are immutable and provenance-aware;
14. image changes create new desired identity;
15. cost cannot override correctness requirements;
16. quotas constrain autoscaling demand;
17. capacity reservations do not create job leases;
18. spot preemption is infrastructure failure;
19. unknown provider resources are not blindly deleted;
20. orphan cleanup requires ownership proof;
21. provider outage increases queue time rather than causing unsafe placement;
22. cache/toolchain locality remains optimization only;
23. forecast is advisory;
24. hostile tenants can require stronger fleet isolation;
25. fleet health does not fabricate runner availability;
26. reconciliation is the correctness mechanism;
27. standalone does not require fleet manager;
28. distributed and standalone share runner admission semantics;
29. security incidents can freeze/drain fleets;
30. Forgeyard dogfoods its own fleet system.

---

# 299. Final Target Architecture

```text
                       Job Queue
                          │
                          ▼
                       Scheduler
                          │
                          ▼
                     Unmet Demand
                          │
                          ▼
                    Capacity Planner
                          │
                          ▼
                    Desired Fleet
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Provider      Bootstrap    Reconcile
             │            │            │
             └────────────┼────────────┘
                          ▼
                    Admitted Runner
                          │
                          ▼
                       Scheduler
```

---

# 300. Final Architectural Position

Scale up:

```text
scheduler unmet demand
+
fleet eligibility
+
quota/cost/policy
  ↓
desired additional capacity
  ↓
provider provision intent
  ↓
host boot
  ↓
Forgeyard enrollment/admission
  ↓
runner becomes Available
```

Scale down:

```text
idle eligible runner
  ↓
mark Draining
  ↓
stop new leases
  ↓
wait active attempts
  ↓
terminate provider resource
  ↓
reconcile
```

Provider uncertainty:

```text
API timeout after create/delete
  ↓
operation = Unknown
  ↓
inspect provider
  ↓
determine actual state
  ↓
continue/reconcile
```

The key guarantee is:

> **Forgeyard can elastically grow and shrink runner capacity across cloud, Kubernetes, on-prem, GPU, Windows, Linux, and macOS fleets without making infrastructure-provider behavior part of execution correctness. Capacity is provisioned and reconciled around scheduler demand, but only fully admitted Forgeyard runners ever become eligible for leases, and autoscaling never weakens trust, policy, tenant isolation, or scheduling semantics.**

---

# 301. Extended Architecture Sequence

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
```
