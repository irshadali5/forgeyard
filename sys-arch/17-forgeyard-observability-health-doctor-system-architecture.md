# 17 — Forgeyard Observability, Health & Doctor System Architecture

**Document type:** Core Observability & Operational Diagnostics System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Structured logs, metrics, traces, OTLP export, W3C trace context, health models, subsystem probes, SLOs, alerting, diagnostic snapshots, doctor commands, incident evidence, correlation, degraded-mode signals, and operational observability contracts  
**Architecture style:** Structured, bounded, correlation-first observability with non-authoritative telemetry, explicit health states, pluggable exporters, local-first diagnostics, and production-grade self-diagnosis  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Integrates with Run/Job, Scheduler, Runner/Agent, Sandbox/Executor, Transport/QUIC, Events/Reconciliation, Policy/Authz/Identity, Secrets/Trust, Supply Chain, Packaging, Release, Deployment, Storage, and CAS. It exposes signals to operators and health gates but never replaces authoritative persisted state.

---

# 1. Purpose

Forgeyard is a distributed CI/CD platform with many moving parts:

```text
daemon
agent
runner
scheduler
CAS
database
release workers
deployment providers
device agents
signing workers
webhooks
SCM providers
```

Operators must be able to answer:

```text
what is broken?
where is latency coming from?
why is a job stuck?
why is a runner unhealthy?
is CAS slow?
is the DB overloaded?
did release publication stall?
is deployment degraded?
what changed before the incident?
```

The central rule is:

> **Observability explains system behavior; it does not define system truth. Persisted domain state remains authoritative.**

A second rule is:

> **Every significant distributed operation should be correlated across logs, metrics, traces, and domain identities without turning high-cardinality identifiers into metric labels.**

A third rule is:

> **Health reporting must distinguish component health, dependency health, readiness, liveness, and user-visible service degradation.**

---

# 2. Architectural Position

```text
                    Forgeyard Components
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       Logs             Metrics           Traces
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                     Telemetry Core
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
             OTLP        Local       Diagnostics
                           │
                           ▼
                       Health Model
                           │
                           ▼
                     Doctor / Alerts
```

---

# 3. Goals

The subsystem MUST:

1. define structured logging;
2. define metrics conventions;
3. define tracing conventions;
4. support OpenTelemetry/OTLP;
5. support W3C trace context;
6. support local-only operation without external telemetry backend;
7. support distributed context propagation;
8. support run/job/attempt/lease correlation;
9. support runner/scheduler/deployment/release correlation;
10. avoid high-cardinality metrics abuse;
11. expose component health;
12. expose dependency health;
13. expose readiness;
14. expose liveness;
15. expose degraded mode;
16. support SLO metrics;
17. support alerting hooks;
18. support doctor commands;
19. support diagnostic snapshots;
20. support privacy/redaction;
21. support telemetry backpressure;
22. support bounded telemetry queues;
23. support incident investigation;
24. support trace sampling;
25. support metrics aggregation;
26. support logs retention;
27. support audit separation;
28. support deployment health gates;
29. support self-hosting observability;
30. remain exporter-neutral.

---

# 4. Non-Goals

Observability does not:

```text
change job state
declare release success
override policy
replace audit logs
replace event store
replace authoritative health checks
```

---

# 5. Workspace Structure

```text
crates/telemetry/
├── forgeyard-telemetry/
├── forgeyard-telemetry-model/
├── forgeyard-telemetry-log/
├── forgeyard-telemetry-metrics/
├── forgeyard-telemetry-trace/
├── forgeyard-telemetry-context/
├── forgeyard-telemetry-otlp/
├── forgeyard-telemetry-export/
├── forgeyard-telemetry-sampling/
├── forgeyard-telemetry-redaction/
├── forgeyard-telemetry-health/
├── forgeyard-telemetry-slo/
└── forgeyard-telemetry-testkit/
```

Health:

```text
crates/health/
├── forgeyard-health/
├── forgeyard-health-model/
├── forgeyard-health-check/
├── forgeyard-health-aggregate/
├── forgeyard-health-readiness/
├── forgeyard-health-liveness/
├── forgeyard-health-degraded/
├── forgeyard-health-doctor/
├── forgeyard-health-snapshot/
└── forgeyard-health-testkit/
```

---

# 6. Telemetry Principles

1. structured by default;
2. bounded by default;
3. redacted by default;
4. correlation IDs explicit;
5. metrics low-cardinality;
6. traces sampled;
7. logs searchable;
8. no secret values;
9. no auth tokens;
10. local diagnostics always available.

---

# 7. Structured Logging

Use `tracing`-style structured events.

Example fields:

```text
component
operation
tenant_scope
run_id
job_id
attempt_id
runner_id
lease_id
release_id
deployment_id
error_code
```

IDs go in logs/traces, not metrics labels.

---

# 8. Log Levels

```text
TRACE
DEBUG
INFO
WARN
ERROR
```

---

# 9. INFO

Significant lifecycle events.

---

# 10. DEBUG

Operational detail.

---

# 11. TRACE

High-volume deep diagnostics, disabled by default.

---

# 12. ERROR

Unexpected failure requiring attention.

---

# 13. WARN

Degraded/retryable/unusual but handled.

---

# 14. Log Event Schema

```rust
pub struct StructuredLogEvent {
    pub timestamp: Timestamp,
    pub level: LogLevel,
    pub target: LogTarget,
    pub message: BoundedString,
    pub fields: LogFields,
    pub trace: Option<TraceContext>,
}
```

---

# 15. Log Target

Stable subsystem target:

```text
forgeyard.scheduler
forgeyard.runner
forgeyard.cas
forgeyard.release
```

---

# 16. Error Logging

Always include typed `ErrorCode`.

---

# 17. No Raw `Debug` of Sensitive Types

Secret/security types implement redacted Debug.

---

# 18. Job Logs vs System Logs

Separate:

```text
job workload logs
system/operator logs
```

---

# 19. Job Logs

Already handled by runner/log data plane.

Observability can correlate, not merge semantics blindly.

---

# 20. Audit Logs

Separate immutable audit subsystem.

Do not treat ordinary telemetry retention as audit retention.

---

# 21. Metrics

Use counters, gauges, histograms.

---

# 22. Metric Naming

Prefix:

```text
forgeyard_
```

Example:

```text
forgeyard_scheduler_queue_wait_seconds
```

---

# 23. Metric Units

Follow OpenTelemetry semantic conventions where applicable.

---

# 24. Cardinality Rule

Never label metrics with:

```text
RunId
JobId
AttemptId
LeaseId
PrincipalId
ArtifactId
```

---

# 25. Good Labels

```text
component
result
platform
executor_kind
environment_class
release_channel
provider_type
```

---

# 26. Bad Labels

```text
project_name
branch
commit
runner_id
user_email
```

unless tightly bounded and explicitly approved.

---

# 27. Core System Metrics

```text
request latency
error rate
queue depth
worker utilization
DB latency
CAS latency
event backlog
reconcile backlog
```

---

# 28. Run Metrics

```text
run_duration
job_duration
job_queue_wait
retry_count
lost_attempt_count
```

---

# 29. Scheduler Metrics

Already defined but standardized here.

---

# 30. Runner Metrics

```text
active jobs
CPU usage
memory usage
disk pressure
reconnect count
lease rejection
```

---

# 31. CAS Metrics

```text
hit rate
miss rate
fetch latency
upload latency
replication lag
GC backlog
```

---

# 32. Storage Metrics

```text
query latency
pool saturation
transaction retries
migration state
```

---

# 33. Release Metrics

```text
candidate verify duration
approval wait
publication latency
unknown outcomes
partial releases
```

---

# 34. Deployment Metrics

```text
apply latency
rollout duration
health gate duration
rollback count
drift count
```

---

# 35. Metrics Registry

No global mutable application registry.

Construct at bootstrap.

---

# 36. Metric Descriptor

```rust
pub struct MetricDescriptor {
    pub name: MetricName,
    pub unit: MetricUnit,
    pub description: &'static str,
}
```

---

# 37. Tracing

Distributed traces model one logical operation across services/processes.

---

# 38. Trace Identity

Use standard:

```text
TraceId
SpanId
```

---

# 39. W3C Context

Use:

```text
traceparent
tracestate
```

for interoperable propagation.

---

# 40. Internal Protocol

QUIC/Postcard envelope may include trace context.

---

# 41. REST API

HTTP headers carry W3C trace context.

---

# 42. Provider Calls

Propagate where safe/supported.

---

# 43. Trace Root Examples

```text
create run
schedule job
execute attempt
release promotion
deployment rollout
```

---

# 44. Run Trace

Can span:

```text
API
planning
scheduler
agent
executor
CAS
completion
```

---

# 45. Trace Context Is Not Domain Identity

Trace may be sampled/lost.

RunId remains domain authority.

---

# 46. Span Attributes

Use low-risk structured fields.

---

# 47. Span Events

Useful for:

```text
lease granted
CAS miss
retry
rollout step
```

---

# 48. Sampling

Production should not record every trace forever.

---

# 49. Sampling Policies

```rust
pub enum TraceSamplingPolicy {
    Always,
    Never,
    Ratio(f64),
    TailBased,
    ErrorBiased,
}
```

---

# 50. Error-Biased Sampling

Prefer retaining failed/slow traces.

---

# 51. Local Mode

Can default to higher sampling because scale low.

---

# 52. Distributed Mode

Configurable.

---

# 53. OTLP

Primary export protocol.

---

# 54. OTLP Export

Support:

```text
gRPC
HTTP/protobuf
```

depending deployed collector.

---

# 55. Exporter Independence

Core telemetry code does not depend on vendor-specific backend.

---

# 56. Potential Backends

Examples:

```text
OpenTelemetry Collector
Prometheus-compatible metrics
Loki-compatible logs
Tempo/Jaeger-compatible traces
commercial APM
```

Adapters/exporters only.

---

# 57. No External Backend Requirement

Standalone Forgeyard works without one.

---

# 58. Local Telemetry

Store bounded local diagnostics.

---

# 59. Local Ring Buffer

```rust
pub struct LocalTelemetryBuffer {
    pub max_events: usize,
    pub max_bytes: ByteSize,
}
```

---

# 60. Diagnostic Snapshot

Captures recent local telemetry + health.

---

# 61. Telemetry Backpressure

Exporter unavailable must not block core job execution.

---

# 62. Bounded Export Queue

On overflow:

```text
drop according to priority
increment dropped metric
```

---

# 63. Drop Priority

Preserve:

```text
ERROR/WARN
critical health
```

before TRACE/DEBUG.

---

# 64. Never Block Job Completion on Telemetry Export

Critical invariant.

---

# 65. Telemetry Failure

Observability may degrade.

Core system continues if safe.

---

# 66. Health Model

```rust
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

---

# 67. HealthCheck

```rust
pub struct HealthCheckResult {
    pub id: HealthCheckId,
    pub status: HealthStatus,
    pub observed_at: Timestamp,
    pub latency: Option<Duration>,
    pub code: Option<HealthCode>,
    pub message: Option<BoundedString>,
}
```

---

# 68. Liveness

Answers:

```text
is process alive enough to continue?
```

---

# 69. Readiness

Answers:

```text
can process serve new work?
```

---

# 70. Dependency Health

Answers:

```text
is required dependency usable?
```

---

# 71. Service Health

Aggregate of local subsystem checks.

---

# 72. User-Visible Health

Reflects effect on actual Forgeyard operations.

---

# 73. Example

DB unavailable:

```text
liveness = Healthy
readiness = Unhealthy
service = Degraded/Unhealthy
```

---

# 74. Health Check Categories

```rust
pub enum HealthCheckCategory {
    Process,
    Storage,
    Cas,
    Transport,
    Scheduler,
    Runner,
    Secrets,
    Trust,
    Provider,
    Release,
    Deployment,
    Device,
}
```

---

# 75. Critical vs Optional Dependency

```rust
pub enum HealthImportance {
    Critical,
    RequiredForWrites,
    Optional,
}
```

---

# 76. Aggregation

Critical failed -> unhealthy.

Optional failed -> degraded.

---

# 77. Health Aggregator

```rust
pub trait HealthAggregator {
    fn aggregate(
        &self,
        checks: &[HealthCheckResult],
    ) -> HealthSummary;
}
```

---

# 78. Health Summary

```rust
pub struct HealthSummary {
    pub status: HealthStatus,
    pub readiness: ReadinessStatus,
    pub liveness: LivenessStatus,
    pub degraded_reasons: Vec<HealthCode>,
}
```

---

# 79. Degraded Mode

Forgeyard can enter explicit degraded modes.

---

# 80. DegradedMode

```rust
pub enum DegradedMode {
    None,
    ReadOnly,
    NoNewRuns,
    NoScheduling,
    NoReleasePromotion,
    NoDeployment,
    LocalOnly,
}
```

---

# 81. Example: DB Read Replica Only

Could permit:

```text
read-only UI
```

but no writes.

---

# 82. CAS Write Failure

May permit viewing metadata but block jobs needing outputs.

---

# 83. Secrets Provider Failure

Non-secret builds may continue.

Secret-dependent jobs fail preparation.

---

# 84. IdP Outage

Existing sessions may continue per auth policy.

---

# 85. Health Does Not Auto-Override Domain State

Degraded mode is explicit application policy, not hidden telemetry side effect.

---

# 86. Health Probe Frequency

Bounded and jittered.

---

# 87. Probe Timeout

Every external health check bounded.

---

# 88. Health Check Caching

Short-lived to avoid expensive repeated calls.

---

# 89. Health Freshness

```rust
pub struct HealthFreshness {
    pub valid_for: Duration,
}
```

---

# 90. Unknown

Used when health data stale/unavailable.

---

# 91. Doctor

Doctor is active diagnostics.

---

# 92. Health vs Doctor

Health:

```text
continuous lightweight signals
```

Doctor:

```text
deeper on-demand checks
```

---

# 93. Doctor Command

```text
forgeyard doctor
```

---

# 94. Doctor Scope

Can run:

```text
global
component
runner
project
release
deployment
```

---

# 95. Doctor Result

```rust
pub struct DoctorReport {
    pub generated_at: Timestamp,
    pub checks: Vec<DoctorCheckResult>,
    pub summary: DoctorSummary,
}
```

---

# 96. Doctor Check

```rust
pub struct DoctorCheckResult {
    pub id: DoctorCheckId,
    pub status: DoctorStatus,
    pub message: BoundedString,
    pub remediation: Option<RemediationHint>,
}
```

---

# 97. Doctor Status

```text
Pass
Warn
Fail
Skipped
```

---

# 98. Remediation Hint

Actionable.

---

# 99. Doctor Never Auto-Fixes by Default

Diagnostics first.

---

# 100. Repair Mode

If later added:

```text
explicit --repair
```

and permission/audit required.

---

# 101. Global Doctor

Checks:

```text
DB
CAS
migrations
scheduler
transport
secrets
trust
event backlog
reconcile backlog
```

---

# 102. Runner Doctor

Checks:

```text
capabilities
disk
sandbox
executor
CAS
cert
connectivity
```

---

# 103. Release Doctor

Checks:

```text
signer
publisher credentials
evidence
destination reachability
```

---

# 104. Deployment Doctor

Checks:

```text
target access
provider auth
health backend
rollback capability
```

---

# 105. Doctor Output Formats

Human:

```text
terminal
```

Machine:

```text
RON/Postcard internal
JSON public CLI interoperability
```

---

# 106. Diagnostic Snapshot

Useful for support/incidents.

---

# 107. Snapshot Contents

```text
version
config summary
health
recent errors
queue stats
resource stats
migration state
protocol versions
```

---

# 108. Snapshot Excludes

```text
secret values
tokens
private keys
raw sensitive env
```

---

# 109. Support Bundle

Optional:

```text
forgeyard doctor bundle
```

Produces sanitized archive.

---

# 110. Support Bundle Security

Explicit warning + reviewable manifest.

---

# 111. Support Bundle Manifest

Lists included files/data.

---

# 112. No Automatic Upload

User/operator explicitly sends if desired.

---

# 113. Incident Evidence

During incident, preserve:

```text
critical logs
trace samples
health changes
domain events
```

---

# 114. Incident Marker

```rust
pub struct IncidentId(Ulid);
```

Optional operational grouping.

---

# 115. Incident Timeline

Can correlate:

```text
health degradation
release
deployment
runner loss
DB errors
```

---

# 116. SLOs

Forgeyard can define service objectives.

---

# 117. SLO Examples

```text
API availability
scheduler placement latency
job queue wait
CAS fetch latency
release promotion latency
deployment success rate
```

---

# 118. SLO Model

```rust
pub struct SloDefinition {
    pub id: SloId,
    pub objective: Percent,
    pub window: Duration,
    pub indicator: SliDefinition,
}
```

---

# 119. SLI

Metric-derived.

---

# 120. Error Budget

Optional:

```rust
pub struct ErrorBudget {
    pub allowed: DurationOrCount,
    pub consumed: DurationOrCount,
}
```

---

# 121. SLO Is Operational

Does not change domain correctness automatically.

---

# 122. Alerting

Alert rules consume metrics/health.

---

# 123. Alert Destinations

Potential:

```text
email
Slack
PagerDuty-like
webhook
```

Notification subsystem handles delivery.

---

# 124. Alert Severity

```text
Info
Warning
Critical
```

---

# 125. Alert Dedup

Avoid storms.

---

# 126. Alert Grouping

By:

```text
component
failure class
environment
```

---

# 127. Alert Recovery

Send resolved signal if configured.

---

# 128. Alert Examples

```text
DB unavailable
CAS replication lag
scheduler queue age
runner pool zero capacity
release publication Unknown too long
deployment rollback failed
cert expiry
```

---

# 129. Health History

Store summarized health transitions.

---

# 130. Not Every Probe

Avoid high-volume DB writes.

---

# 131. Health Transition Event

Persist:

```text
Healthy -> Degraded
Degraded -> Healthy
```

for important components.

---

# 132. Flapping

Apply hysteresis/debounce.

---

# 133. Health Hysteresis

```rust
pub struct HealthHysteresis {
    pub fail_after: u32,
    pub recover_after: u32,
}
```

---

# 134. Dependency Fan-Out

One failing dependency can degrade many components.

---

# 135. Root Cause Hinting

Health graph can identify likely upstream dependency.

---

# 136. Health Dependency Graph

```text
API -> DB
Scheduler -> DB
Runner -> Transport/CAS
Release -> Signer/Publisher
```

---

# 137. Health Graph Is Diagnostic

Not a formal causal proof.

---

# 138. Component Registration

At application bootstrap.

---

# 139. Health Probe Trait

```rust
#[async_trait]
pub trait HealthProbe {
    async fn check(&self) -> HealthCheckResult;
}
```

---

# 140. Doctor Check Trait

```rust
#[async_trait]
pub trait DoctorCheck {
    async fn run(&self, ctx: DoctorContext)
        -> DoctorCheckResult;
}
```

---

# 141. Metrics Exporter Trait

```rust
pub trait MetricsExporter: Send + Sync {
    fn export(&self, batch: MetricsBatch)
        -> Result<(), TelemetryExportError>;
}
```

---

# 142. Trace Exporter

Similarly.

---

# 143. Log Exporter

Similarly.

---

# 144. OTLP Adapter

Implementation only.

---

# 145. Prometheus Scrape

Can expose metrics endpoint.

---

# 146. Metrics Endpoint

```text
/metrics
```

admin/internal.

---

# 147. Public Exposure

Do not expose sensitive metrics publicly by default.

---

# 148. Health Endpoints

Potential:

```text
/health/live
/health/ready
/health
```

---

# 149. Health Endpoint Authentication

Liveness may be unauthenticated internally.

Detailed health requires auth.

---

# 150. Kubernetes Probes

Map:

```text
liveness -> /health/live
readiness -> /health/ready
```

---

# 151. Readiness Logic

Daemon not ready if:

```text
DB required writes unavailable
migration incompatible
critical config invalid
```

---

# 152. Liveness Logic

Process deadlock/internal loop broken.

External DB outage should not necessarily fail liveness.

---

# 153. Startup Probe

Useful during migrations/warmup.

---

# 154. Runner Local Health

Agent can expose CLI/local status.

---

# 155. Transport Health

Connection state, heartbeat age.

---

# 156. Scheduler Health

Loop progress, queue age.

---

# 157. Event Health

Outbox age/backlog.

---

# 158. Reconciler Health

Last successful cycle.

---

# 159. CAS Health

Read/write/replication.

---

# 160. DB Health

Connection/query/migration state.

---

# 161. Secrets Health

Provider availability without value disclosure.

---

# 162. Trust Health

cert/root expiry.

---

# 163. Release Health

publisher/signer readiness.

---

# 164. Deployment Health

provider/metric backend.

---

# 165. Health Code Registry

Stable typed codes.

---

# 166. Example Codes

```text
DB_UNAVAILABLE
DB_MIGRATION_REQUIRED
CAS_WRITE_FAILED
RUNNER_POOL_EMPTY
SIGNER_UNAVAILABLE
OIDC_JWKS_STALE
DEPLOY_PROVIDER_UNREACHABLE
```

---

# 167. User-Facing Health Message

Safe and actionable.

---

# 168. Internal Diagnostic Detail

Separate, permission-protected.

---

# 169. Privacy

Telemetry must obey tenant/privacy boundaries.

---

# 170. Tenant IDs

May appear in traces/logs internally if required, but not exported without policy.

---

# 171. PII

Avoid user email/name in telemetry.

Use PrincipalId if needed.

---

# 172. Secret Redaction

Reuses secrets redaction layer.

---

# 173. URL Redaction

Strip query tokens/credentials.

---

# 174. HTTP Headers

Never log Authorization/Cookie by default.

---

# 175. Command Logging

Job command can be sensitive.

Respect pipeline redaction policy.

---

# 176. Environment Logging

Never dump full env.

---

# 177. SQL Logging

Avoid parameter values by default.

---

# 178. Provider Error Logging

Redact provider tokens/secrets.

---

# 179. Telemetry Retention

Different classes:

```text
system logs
job logs
traces
metrics
health history
```

---

# 180. Metrics Retention

External backend responsibility.

---

# 181. Local Retention

Bounded disk.

---

# 182. Trace Retention

Sampled.

---

# 183. System Log Rotation

Size/time based.

---

# 184. Job Log Retention

Run/artifact policy.

---

# 185. Health History Retention

Summarized.

---

# 186. Support Bundle Retention

Manual/operator controlled.

---

# 187. Disk Pressure

Telemetry storage should yield before core data.

---

# 188. Telemetry Disk Budget

Explicit quota.

---

# 189. Drop Strategy

Drop oldest/debug first.

---

# 190. Never Delete Audit for Telemetry Pressure

Audit separate.

---

# 191. Query Model

Observability UI can query:

```text
logs
metrics summaries
traces
health
```

---

# 192. External Backend Queries

Use adapter abstraction.

---

# 193. Built-In Minimal Query

Local recent logs/health even without backend.

---

# 194. Dioxus UI

Operational pages:

```text
System Health
Runners
Scheduler
Storage
CAS
Events
Reconciliation
Releases
Deployments
Incidents
```

---

# 195. System Health Dashboard

Show:

```text
overall
critical dependencies
degraded components
recent transitions
```

---

# 196. Run Observability

Run page links:

```text
timeline
logs
trace
metrics summary
```

---

# 197. Job Observability

Shows:

```text
queue wait
runner
attempt
execution duration
resource usage
CAS transfer
```

---

# 198. Runner Observability

Shows:

```text
CPU
memory
disk
active jobs
reconnects
health
```

---

# 199. Release Observability

Shows:

```text
verification time
approval wait
publication latency
errors
```

---

# 200. Deployment Observability

Shows:

```text
rollout step
health metrics
drift
rollback
```

---

# 201. Trace Links

UI can deep-link by RunId/AttemptId.

---

# 202. Log Search

If external backend configured.

---

# 203. No Built-In Massive Log Database Initially

Use external backend or bounded local logs.

---

# 204. Standalone Mode

Local dashboard reads local metrics/health.

---

# 205. Distributed Mode

Daemon aggregates control-plane health; runners export/forward according to config.

---

# 206. Runner Telemetry Path

Options:

```text
agent -> daemon
agent -> OTLP collector
agent -> both
```

---

# 207. Recommendation

Direct OTLP to collector for high-volume telemetry where available.

Daemon still receives critical operational state.

---

# 208. No Telemetry Authority Dependency

If collector down:

jobs still run.

---

# 209. Trace Correlation Through Agent

Propagate trace context with lease/spec.

---

# 210. Workload Trace Integration

Optional user application tracing is separate from Forgeyard's internal traces.

---

# 211. Build Tool Spans

Forgeyard may create spans around steps.

---

# 212. Step Span

```text
job.step
```

---

# 213. Service Process Span

Can model startup/wait.

---

# 214. CAS Span

Fetch/upload child spans.

---

# 215. Scheduler Span

Placement/filter/score.

---

# 216. Reconcile Span

Repair operation.

---

# 217. SLO Dashboard

Show:

```text
objective
current SLI
error budget
trend
```

---

# 218. SLO Config

RON.

---

# 219. SLO Validation

Check metric exists/labels bounded.

---

# 220. Alert Config

RON/policy.

---

# 221. Alert Routes

Notification refs.

---

# 222. Health Gate Integration

Deployment can consume health/metric provider.

---

# 223. Release Health Gate

Release promotion may consume operational soak metrics if policy wants.

---

# 224. Policy Integration

Observability provides facts.

Central policy evaluates.

---

# 225. No Telemetry-Created Permission

Health does not authorize.

---

# 226. Incident Trigger

Critical alert can create incident record.

Optional.

---

# 227. Incident Record

```rust
pub struct Incident {
    pub id: IncidentId,
    pub started_at: Timestamp,
    pub severity: IncidentSeverity,
    pub state: IncidentState,
}
```

---

# 228. Incident State

```text
Open
Mitigating
Resolved
```

---

# 229. Incident Correlation

Link:

```text
releases
deployments
health transitions
alerts
```

---

# 230. Automatic Causality

Do not claim release caused incident solely from temporal correlation.

---

# 231. Change Overlay

UI may show recent release/deploy events near incident.

---

# 232. Doctor Dry Run

Default diagnostic only.

---

# 233. Doctor Deep

```text
forgeyard doctor --deep
```

Runs more expensive probes.

---

# 234. Doctor Offline

Can run local diagnostics with no network.

---

# 235. Doctor Runner

Remote control-plane can request approved runner doctor checks.

---

# 236. Remote Doctor Safety

No arbitrary shell.

Typed checks only.

---

# 237. Doctor Check Registry

Static/bootstrap registry.

---

# 238. Plugin Checks

Future plugin system can register constrained doctor checks.

---

# 239. Health Snapshot

```rust
pub struct HealthSnapshot {
    pub id: HealthSnapshotId,
    pub generated_at: Timestamp,
    pub summary: HealthSummary,
    pub checks: Vec<HealthCheckResult>,
}
```

---

# 240. Snapshot Persistence

Important incident snapshots can be stored.

Routine health remains ephemeral/summarized.

---

# 241. Snapshot CAS

Large diagnostic attachments can go CAS.

---

# 242. Health Event

Status transition emits domain operational event.

---

# 243. Event Payload

No secret values.

---

# 244. Degraded Mode Event

Explicit.

---

# 245. Automatic Degraded Mode

Only when application policy explicitly maps health condition to mode.

---

# 246. Example

DB write failure:

```text
enter ReadOnly
```

if safe.

---

# 247. Recovery

When health recovers, application exits degraded mode after checks.

---

# 248. Hysteresis

Prevent flapping.

---

# 249. Startup Health

Before readiness:

```text
config loaded
DB schema compatible
store reachable
required trust material valid
```

---

# 250. Schema Migration Health

If migration pending:

```text
readiness false
```

for incompatible writes.

---

# 251. Rolling Upgrade Health

N/N-1 compatible replicas can remain ready.

---

# 252. Build Version Metric

Expose Forgeyard version/build info as low-cardinality info metric.

---

# 253. Protocol Version Metric

Bounded labels.

---

# 254. Dependency Version

Do not expose huge dependency list as metrics.

Doctor snapshot can.

---

# 255. Performance Profiling

Optional.

---

# 256. CPU Profiling

Could integrate `pprof`/platform profiler in admin/debug mode.

---

# 257. Heap Profiling

Optional.

---

# 258. Profiling Security

Admin-only, bounded, potentially sensitive.

---

# 259. Continuous Profiling

External optional integration.

---

# 260. eBPF Telemetry

Optional Linux enhancer.

---

# 261. eBPF Uses

```text
system call latency
network
CPU
IO
```

---

# 262. eBPF Not Required

Correctness/health must work without it.

---

# 263. OpenTelemetry Resource Attributes

Examples:

```text
service.name
service.version
deployment.environment
host.arch
```

---

# 264. Runner Resource Attributes

Avoid RunnerId if backend cardinality concern; use it in trace/log resource if acceptable.

---

# 265. Service Names

```text
forgeyard-daemon
forgeyard-agent
forgeyard-signing-worker
forgeyard-device-agent
```

---

# 266. Metrics Security

Admin endpoint protected.

---

# 267. Trace Export Security

TLS/credentials through SecretRef.

---

# 268. Collector Credential

Short-lived/provider identity where possible.

---

# 269. Export Failure Retry

Bounded.

---

# 270. Spooling

Optional small local spool.

---

# 271. No Unbounded Telemetry Spool

Quota.

---

# 272. Export Retry Priority

Critical health logs before debug traces.

---

# 273. Telemetry Config

Example:

```ron
(
    telemetry: (
        logs: (
            level: "info",
            local_rotation: (
                max_size: "100MiB",
                max_files: 5,
            ),
        ),
        metrics: (
            enabled: true,
        ),
        tracing: (
            sampling: Ratio(0.10),
        ),
        otlp: Some((
            endpoint: "https://otel.example.internal",
            credential: Secret("otel/export"),
        )),
    ),
)
```

---

# 274. Health Config

```ron
(
    health: (
        probe_interval: "15s",
        probe_timeout: "3s",
    ),
)
```

---

# 275. Alert Config

```ron
(
    alerts: [
        (
            condition: "scheduler.oldest_queue_wait > 300s",
            severity: Warning,
        ),
    ],
)
```

Actual typed config preferred over raw string expression if possible.

---

# 276. Typed Alert Rules

```rust
pub enum AlertCondition {
    MetricThreshold(...),
    HealthStatus(...),
    BacklogAge(...),
    CertificateExpiry(...),
}
```

---

# 277. Alert Engine

Small deterministic evaluator.

---

# 278. No General Scripting Initially

Same rationale as policy.

---

# 279. Testkit

```text
forgeyard-telemetry-testkit/src/
├── lib.rs
├── logger.rs
├── metrics.rs
├── traces.rs
├── exporter.rs
├── sampling.rs
└── assertions.rs
```

Health:

```text
forgeyard-health-testkit/src/
├── lib.rs
├── probe.rs
├── aggregate.rs
├── doctor.rs
├── degraded.rs
└── assertions.rs
```

---

# 280. Unit Tests

Test:

```text
health aggregation
sampling
redaction
metric labels
doctor reports
```

---

# 281. Cardinality Tests

Static/lint test rejects known high-cardinality label usage.

---

# 282. Secret Leakage Tests

Ensure test secret not present in:

```text
logs
trace attributes
doctor bundle
health messages
```

---

# 283. Exporter Failure Test

Collector down -> core system remains functional.

---

# 284. Queue Overflow Test

Telemetry queue bounded.

---

# 285. Error Priority Test

ERROR preserved before DEBUG under pressure.

---

# 286. Trace Propagation Test

API -> daemon -> agent -> executor keeps same TraceId.

---

# 287. Sampling Test

Unsampled trace does not break domain correlation.

---

# 288. Health Probe Timeout Test

Slow dependency -> Unknown/Unhealthy according to policy, no worker deadlock.

---

# 289. Hysteresis Test

Flapping probe does not flap service state excessively.

---

# 290. Readiness Test

DB unavailable -> daemon not ready.

---

# 291. Liveness Test

DB unavailable does not necessarily kill liveness.

---

# 292. Degraded Mode Test

Configured dependency failure enters explicit safe mode.

---

# 293. Recovery Test

Health recovery exits degraded mode safely.

---

# 294. Doctor Test

Produces actionable remediation.

---

# 295. Support Bundle Test

Contains manifest, excludes secrets.

---

# 296. Metrics Conformance Test

Metric names/units stable.

---

# 297. OTLP Test

Export to test collector.

---

# 298. W3C Test

Trace headers round-trip.

---

# 299. Large Scale Test

High job volume without telemetry becoming bottleneck.

---

# 300. Fuzzing

Fuzz:

```text
doctor report decoder
health config
telemetry config
support bundle manifest
```

---

# 301. Failure Injection

```text
collector unavailable
metrics exporter panic
disk full
log file permission failure
health dependency timeout
```

---

# 302. Telemetry Panic Isolation

Exporter failures must not crash daemon.

---

# 303. Implementation Phase 1 — Structured Logs

Implement tracing/logging conventions + redaction.

---

# 304. Phase 2 — Metrics

Core low-cardinality metric registry.

---

# 305. Phase 3 — Traces

W3C propagation, Run/Job/Attempt spans.

---

# 306. Phase 4 — Health

Liveness/readiness/dependency model.

---

# 307. Phase 5 — Doctor

Global + runner + storage/CAS diagnostics.

---

# 308. Phase 6 — OTLP

Exporter adapter.

---

# 309. Phase 7 — SLO/Alerts

Operational metrics.

---

# 310. Phase 8 — Release/Deployment Health

Integrate rollout/release dashboards.

---

# 311. Phase 9 — Support/Incident Bundles

Sanitized diagnostic evidence.

---

# 312. Phase 10 — Hardening

Scale, drop policies, failure isolation, security testing.

---

# 313. Acceptance Tests

1. Structured logs include stable component/error fields.
2. Secret values never appear in normal telemetry.
3. RunId/JobId appear in logs/traces but not metric labels.
4. W3C trace context propagates API -> daemon -> agent.
5. Trace loss/sampling never affects domain correctness.
6. Metrics exporter outage does not block jobs.
7. OTLP collector is optional.
8. Standalone mode has useful local observability.
9. Telemetry queues are bounded.
10. Telemetry disk usage is quota-bounded.
11. ERROR/WARN events survive preferentially under pressure.
12. Health distinguishes liveness/readiness.
13. External dependency failure does not automatically fail liveness.
14. Readiness fails when critical serving dependency unavailable.
15. Optional dependency failure produces Degraded rather than false Unhealthy.
16. Health probes are timeout-bounded.
17. Health aggregation is deterministic.
18. Degraded modes are explicit.
19. Doctor gives actionable diagnostics.
20. Doctor never executes arbitrary shell remotely.
21. Support bundle excludes secrets/private keys/tokens.
22. Support bundle has explicit manifest.
23. Scheduler queue SLO can be measured.
24. Release publication latency can be measured.
25. Deployment health gates can consume metric provider abstraction.
26. Health transitions are debounced/hysteretic.
27. Incident snapshots can correlate release/deployment changes.
28. Metrics names/units remain stable.
29. High-cardinality metric label linting exists.
30. Audit retention is separate from telemetry retention.
31. Runner health reports actual capability/dependency status.
32. DB/CAS/secret/trust health checks are distinct.
33. Self-hosted Forgeyard can diagnose itself without external SaaS.
34. External telemetry exporters remain adapter-local.
35. Forgeyard dogfoods its observability/doctor system.

---

# 314. Production Readiness Gates

Do not call observability/health production-ready until:

```text
structured logging standardized
secret redaction tested
low-cardinality metrics enforced
W3C trace propagation stable
OTLP optional export works
health/readiness/liveness model stable
doctor covers DB/CAS/scheduler/runner/transport
telemetry backpressure bounded
degraded modes explicit
alerting/SLO basics available
support bundle sanitized
```

Advanced incident management, continuous profiling, eBPF telemetry, and sophisticated SLO tooling can mature later.

---

# 315. Architectural Invariants

1. telemetry is not domain authority;
2. persisted state remains truth;
3. logs are structured;
4. metrics are low-cardinality;
5. traces use standard context;
6. domain IDs belong in logs/traces, not metric labels;
7. secret values never enter telemetry;
8. exporter failure never blocks core correctness;
9. telemetry queues are bounded;
10. external telemetry backend is optional;
11. local diagnostics always exist;
12. liveness and readiness are distinct;
13. dependency health is explicit;
14. degraded state is explicit;
15. health does not silently mutate domain state;
16. doctor is deeper than health probes;
17. doctor does not arbitrary-shell remote hosts;
18. support bundles are sanitized and explicit;
19. audit is separate from telemetry;
20. health gates consume observability facts through typed APIs;
21. SLOs are operational signals, not policy authority by themselves;
22. trace sampling never changes behavior;
23. health probes have deadlines;
24. health aggregation is deterministic;
25. hysteresis prevents flapping;
26. provider-specific observability stays adapter-local;
27. high-volume job logs remain a separate data plane;
28. standalone/distributed share observability semantics;
29. telemetry storage yields before critical system data;
30. Forgeyard dogfoods its observability system.

---

# 316. Final Target Architecture

```text
                 Forgeyard Components
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       Logs          Metrics        Traces
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                   Telemetry Core
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       Local         OTLP          Health
      Buffer        Export         Aggregator
          │                            │
          ▼                            ▼
      Doctor/UI                 Readiness/Liveness
                                       │
                                       ▼
                                  Degraded Mode
                                       │
                                       ▼
                               Operator / Automation
```

---

# 317. Final Architectural Position

Correlation:

```text
RunId / JobId / AttemptId
+
TraceId / SpanId
  ↓
logs + traces
```

Metrics:

```text
bounded dimensions
+
aggregated counters/histograms
  ↓
SLOs / alerts
```

Health:

```text
lightweight probes
+
dependency status
+
readiness/liveness
  ↓
HealthSummary
```

Doctor:

```text
on-demand deeper checks
+
sanitized diagnostics
  ↓
actionable remediation
```

The key guarantee is:

> **Forgeyard can explain what the system is doing, how well it is doing it, and what is failing without making observability infrastructure part of the correctness path. Telemetry may disappear; authoritative execution and recovery must still remain correct.**

---

# 318. New-Repository Sequence

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
