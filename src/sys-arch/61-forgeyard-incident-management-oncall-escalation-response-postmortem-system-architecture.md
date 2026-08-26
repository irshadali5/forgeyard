# 61 — Forgeyard Incident Management, On-Call, Escalation, Response Coordination & Postmortem System Architecture

**Document type:** Core Incident Management, On-Call, Escalation, Response Coordination, Operational Communications, Postmortem & Corrective-Action System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** incident identity, severity, alert-to-incident promotion, on-call schedules, escalation policies, acknowledgement, incident command, responders, timelines, impact/blast-radius analysis, mitigation tracking, communication, change freezes, evidence capture, postmortems, follow-up actions, operational review, and incident learning  
**Architecture style:** Explicit incident state, role-based coordination, immutable timeline events, evidence-linked decisions, escalation with deadlines, auditability, post-incident learning, and no “incident mode” bypass of core security/correctness guarantees  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Notifications/Alerting, Security Incident Response, Reliability/SLO/Error Budget, Service Catalog/Ownership, Audit/Compliance, Observability, Deployment, Release, Change Management/Merge Queue, Failure Diagnosis, Federation, Infrastructure, and Data Lifecycle. This subsystem turns isolated alerts and security events into coordinated operational response.

---

## 1. Purpose

Forgeyard already detects and reports many operational problems:

```text
runner fleet degradation
queue backlog
failed deployments
SLO burn
CAS corruption
database failover
region outage
security incident
certificate expiry
replication lag
provider outage
network isolation failure
critical runner-image vulnerability
```

Detection alone is not enough. Operators also need to know who owns the response, what is affected, what mitigations are in progress, what has been communicated, and what permanent corrective actions remain.

The central rule is:

> **An incident is a first-class operational object with explicit scope, severity, ownership, roles, timeline, evidence, mitigations, and resolution state.**

A second rule is:

> **Urgency does not eliminate correctness. Incident response may enable explicit emergency procedures, but it never silently bypasses authorization, audit, signing, environment ownership, data-protection, or recovery invariants.**

A third rule is:

> **Postmortems exist to improve systems and processes, not to create blame-oriented individual scorecards.**

---

## 2. Architectural Position

```text
                  Alerts / Signals / Reports
                           │
                           ▼
                    Incident Triage
                           │
                           ▼
                     Incident Record
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
         On-Call        Responders      Evidence
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                    Incident Command
                           │
                           ▼
                  Mitigate / Recover
                           │
                           ▼
                       Resolve
                           │
                           ▼
                     Postmortem
                           │
                           ▼
                 Corrective Actions
```

---

## 3. Goals

The subsystem MUST support:

1. incident identity and severity;
2. alert promotion and manual incident declaration;
3. on-call schedules and escalation policies;
4. acknowledgement deadlines;
5. incident command and responder roles;
6. immutable timeline capture;
7. impact and blast-radius tracking;
8. mitigation actions;
9. change freezes and emergency change procedures;
10. internal/public communication updates;
11. evidence linking;
12. security, reliability, provider, regional and infrastructure incidents;
13. postmortems and corrective actions;
14. audit, retention, federation, and disaster recovery;
15. UI, API, CLI, analytics, and operational health.

---

## 4. Non-Goals

This subsystem does not replace:

```text
monitoring
security response controls
disaster recovery
deployment rollback
notification providers
project-management software
HR/performance systems
```

---

## 5. Workspace Structure

```text
crates/incident/
├── forgeyard-incident/
├── forgeyard-incident-model/
├── forgeyard-incident-triage/
├── forgeyard-incident-oncall/
├── forgeyard-incident-escalation/
├── forgeyard-incident-command/
├── forgeyard-incident-timeline/
├── forgeyard-incident-impact/
├── forgeyard-incident-communication/
├── forgeyard-incident-postmortem/
├── forgeyard-incident-action/
├── forgeyard-incident-reconcile/
├── forgeyard-incident-health/
└── forgeyard-incident-testkit/
```

Provider integrations remain behind adapters.

---

## 6. Incident Identity

```rust
pub struct IncidentId(Ulid);
```

---

## 7. Incident Type

```rust
pub enum IncidentType {
    Availability,
    Reliability,
    Security,
    DataIntegrity,
    DataLossRisk,
    Performance,
    Capacity,
    Provider,
    Networking,
    Deployment,
    Release,
    Infrastructure,
    RunnerFleet,
    Compliance,
    Custom(IncidentTypeId),
}
```

---

## 8. Incident Severity

```rust
pub enum IncidentSeverity {
    Sev0,
    Sev1,
    Sev2,
    Sev3,
    Sev4,
}
```

Example semantics:

```text
Sev0 — catastrophic, broad critical impact / existential control-plane risk
Sev1 — major production impact or critical security/data-integrity risk
Sev2 — significant degraded service with workaround or limited scope
Sev3 — localized operational incident
Sev4 — minor operational issue requiring coordinated tracking
```

Severity is explicitly recorded, may be overridden with reason, and does not collapse into alert priority.

---

## 9. Incident Lifecycle

```rust
pub enum IncidentState {
    Declared,
    Acknowledged,
    Investigating,
    Mitigating,
    Monitoring,
    Resolved,
    Closed,
    Cancelled,
}
```

Key distinction:

```text
Resolved != Root Cause Confirmed
```

An incident can be resolved while root cause remains unknown.

---

## 10. Incident Declaration

```rust
pub enum IncidentDeclarationSource {
    Alert(AlertId),
    SecurityEvent(SecurityIncidentRef),
    SloViolation(SloEvaluationId),
    Deployment(DeploymentId),
    Manual(PrincipalId),
    Provider(ProviderIncidentRef),
}
```

An alert is not automatically an incident.

---

## 11. Incident Subject

```rust
pub enum IncidentSubject {
    Installation,
    Tenant(TenantId),
    Project(ProjectId),
    Component(SoftwareComponentId),
    Environment(EnvironmentId),
    Site(SiteId),
    RunnerFleet(RunnerFleetId),
    Provider(ProviderId),
}
```

---

## 12. Impact Model

```rust
pub enum ImpactClass {
    Unavailable,
    Degraded,
    Delayed,
    DataAtRisk,
    SecurityAtRisk,
    Unknown,
}
```

```rust
pub enum ImpactConfidence {
    Confirmed,
    Strong,
    Suspected,
    Unknown,
}
```

Unknown impact remains first-class.

---

## 13. Blast Radius

Blast radius can combine:

```text
service catalog dependency graph
deployment inventory
site/federation state
observability signals
known affected tenants/components
```

It remains evidence, not automatic causal truth.

---

## 14. Immutable Timeline

```rust
pub struct IncidentTimelineEventId(Ulid);
```

```rust
pub enum IncidentTimelineEventKind {
    Declared,
    Acknowledged,
    SeverityChanged,
    RoleAssigned,
    ImpactUpdated,
    MitigationStarted,
    MitigationCompleted,
    ChangeFreezeApplied,
    ChangeFreezeReleased,
    CommunicationSent,
    EvidenceLinked,
    StateChanged,
    Resolved,
    Reopened,
}
```

Timeline events are append-only. Corrections add a new event rather than silently rewriting history.

---

## 15. Evidence

Incident evidence can point to:

```text
RunId
DeploymentId
ReleaseId
FailureObservationId
SloEvaluationId
log ranges
traces
metric snapshots
audit events
ConfigSnapshotId
PolicyDigest
site/fleet health
```

Evidence remains owned by its canonical subsystem.

---

## 16. Incident Roles

```rust
pub enum IncidentRole {
    IncidentCommander,
    OperationsLead,
    TechnicalLead,
    CommunicationsLead,
    Scribe,
    SecurityLead,
    SubjectMatterExpert,
    Observer,
}
```

Incident roles coordinate response.

They do **not** grant production deployment, secret read, policy edit, or signing permissions.

---

## 17. Incident Commander

Responsibilities:

```text
set response structure
maintain priorities
assign roles
coordinate mitigations
decide update cadence
manage handoffs
drive resolution criteria
```

The IC is not a superuser.

---

## 18. On-Call Schedule

```rust
pub struct OnCallScheduleId(Ulid);
```

```rust
pub struct OnCallSchedule {
    pub id: OnCallScheduleId,
    pub scope: OnCallScope,
    pub rotations: Vec<OnCallRotation>,
    pub timezone: TimeZoneId,
}
```

Schedules must be timezone-aware and handle DST explicitly.

---

## 19. On-Call Scope

```rust
pub enum OnCallScope {
    Installation,
    Team(TeamId),
    Component(SoftwareComponentId),
    Security,
    Infrastructure,
}
```

---

## 20. Overrides

Temporary substitutes and vacation coverage are first-class.

Historical incidents retain the actual routed assignment at declaration time.

---

## 21. Escalation Policy

```rust
pub struct EscalationPolicyId(Digest);
```

```rust
pub struct EscalationPolicy {
    pub steps: Vec<EscalationStep>,
}
```

```rust
pub struct EscalationStep {
    pub target: EscalationTarget,
    pub after: Duration,
    pub channels: Vec<NotificationChannelKind>,
}
```

Timers are durable, not memory-only.

---

## 22. Acknowledgement

Notification delivery is not acknowledgement.

```rust
pub struct IncidentAcknowledgement {
    pub incident: IncidentId,
    pub principal: PrincipalId,
    pub at: Timestamp,
}
```

---

## 23. On-Call Gap Handling

If a schedule resolves to nobody:

```text
fallback schedule
  ↓
team fallback
  ↓
installation emergency fallback
```

No silent "nobody on call."

---

## 24. Notification Channels

Potential channels:

```text
in-app
email
chat
SMS
voice
webhook
```

Part 29 handles actual delivery semantics.

Incident subsystem determines routing/escalation intent.

---

## 25. Incident Room

The Dioxus incident room is the canonical operational workspace.

It shows:

```text
state
severity
roles
impact
timeline
mitigations
evidence
communications
```

---

## 26. Mitigation

```rust
pub struct MitigationId(Ulid);
```

```rust
pub enum MitigationState {
    Proposed,
    Approved,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}
```

```rust
pub enum MitigationRisk {
    Low,
    Moderate,
    High,
    Critical,
}
```

A mitigation record coordinates intent; actual execution happens in the canonical subsystem.

---

## 27. Canonical Mitigation Execution

Examples:

```text
Deployment → rollback or redeploy
Configuration → activate emergency config
Feature Flag → kill switch
Infrastructure → controlled apply
Runner Fleet → drain/quarantine
Network → revoke connector/tunnel
Federation → quarantine/failover
Release → yank/revoke/pause channel
```

No incident-specific shell backdoor exists.

---

## 28. Break-Glass

Emergency actions can reference `IncidentId`.

They still require:

```text
authorization
scope
reason
expiry where applicable
audit
```

An active incident does not auto-authorize break-glass.

---

## 29. Change Freeze

```rust
pub struct ChangeFreezeId(Ulid);
```

```rust
pub enum ChangeFreezeScope {
    Installation,
    Tenant(TenantId),
    Project(ProjectId),
    Environment(EnvironmentId),
    ReleaseChannel(ReleaseChannelId),
    MergeTarget(MergeQueueId),
}
```

A freeze may block:

```text
normal deploys
normal releases
merge submissions
infrastructure applies
config changes
```

Emergency mitigation remains an explicit exception path.

---

## 30. Freeze Lifecycle

```rust
pub enum ChangeFreezeState {
    Planned,
    Active,
    Released,
    Expired,
}
```

Doctor detects stale forgotten freezes.

---

## 31. Security Incident Integration

Security-specific containment remains Part 40 authority.

Operational incident and security investigation may have different lifecycles.

Example:

```text
service restored
  ↓
operational incident Resolved
  ↓
security investigation still Active
```

Do not collapse them.

---

## 32. Provider Incidents

External provider status can be linked as evidence.

Forgeyard validates its own recovery independently.

Provider "resolved" is not enough to close Forgeyard incident automatically.

---

## 33. Incident Communication

```rust
pub struct IncidentUpdateId(Ulid);
```

```rust
pub enum IncidentAudience {
    InternalResponders,
    InternalStakeholders,
    Tenant(TenantId),
    Public,
    Custom(AudienceId),
}
```

Internal and public details are separate.

---

## 34. Public Updates

Should emphasize:

```text
current impact
current state
mitigation status
next update expectation
```

Avoid speculative root-cause claims.

---

## 35. Security-Sensitive Communication

May intentionally omit:

```text
exploit details
private hostnames
credentials
tenant identities
attack indicators
```

---

## 36. Communication Cadence

Severity policy can require updates:

```text
every 15 minutes
every 30 minutes
hourly
on material change
```

A missed required update can trigger reminder/escalation.

---

## 37. Status Page Integration

Optional adapter.

Forgeyard keeps the canonical incident update record.

---

## 38. Alert Correlation

Multiple alerts may map to one incident.

```rust
pub struct IncidentCorrelation {
    pub signal: SignalRef,
    pub incident: IncidentId,
    pub confidence: CorrelationConfidence,
}
```

Heuristic correlation never destroys source alert evidence.

---

## 39. Duplicate Incidents

Can be merged operationally into a canonical parent incident.

Historical IDs and timelines remain retained.

---

## 40. Related Incidents

Recurring or dependent incidents can be linked.

Example:

```text
cloud provider outage
  ├── runner capacity incident
  ├── registry replication incident
  └── deployment delay incident
```

---

## 41. Incident Timers

Track exact timestamps for:

```text
detected
declared
acknowledged
mitigation started
impact ended
resolved
closed
```

Derived metrics such as MTTD/MTTA/MTTR must document exact definitions.

---

## 42. No Individual Ranking

Incident metrics are for system/process improvement.

They are not simplistic performance scores for responders.

---

## 43. Reopen

```text
Resolved -> Investigating
```

with explicit reason and timeline event.

---

## 44. Postmortem Identity

```rust
pub struct PostmortemId(Ulid);
```

---

## 45. Postmortem Requirement

Severity/policy based.

Example baseline:

```text
Sev0/Sev1 → mandatory
Sev2 → configurable
Sev3/Sev4 → optional
```

---

## 46. Postmortem State

```rust
pub enum PostmortemState {
    Draft,
    InReview,
    Approved,
    PublishedInternal,
    PublishedExternal,
    Archived,
}
```

---

## 47. Postmortem Structure

Recommended:

```text
summary
impact
timeline
detection
response
root cause
contributing factors
what went well
what did not go well
corrective actions
lessons
```

---

## 48. Root Cause Confidence

```rust
pub enum RootCauseStatus {
    Confirmed,
    Probable,
    Unknown,
}
```

Unknown is allowed.

Do not force a simplistic single root cause.

---

## 49. Contributing Factors

Useful examples:

```text
missing test
unsafe default
slow alert
incorrect ownership
incomplete runbook
weak rollback path
capacity shortage
dependency failure
```

---

## 50. Blameless Design

Postmortems focus on:

```text
system conditions
controls
assumptions
workflow weaknesses
```

rather than punishment.

This does not remove accountability for completing corrective actions.

---

## 51. Corrective Actions

```rust
pub struct CorrectiveActionId(Ulid);
```

```rust
pub enum CorrectiveActionState {
    Open,
    InProgress,
    Blocked,
    Done,
    Cancelled,
    Superseded,
}
```

---

## 52. Corrective Action Evidence

Completion can link to:

```text
ChangeProposal
ReleaseId
ConfigSnapshotId
test evidence
monitoring rule
runbook update
policy change
infrastructure change
```

---

## 53. Weak vs Strong Corrective Action

Weak:

```text
"be more careful"
```

Strong:

```text
"add policy gate preventing unsigned package promotion"
```

The system may flag weak action wording as advisory, not block closure by opaque AI score.

---

## 54. Problem Records

Optional longer-lived umbrella object:

```rust
pub struct ProblemId(Ulid);
```

Incident = active impact/response.

Problem = recurring/systemic issue spanning multiple incidents.

---

## 55. Recurrence

```rust
pub struct IncidentRecurrenceId(Digest);
```

Recurrence correlation carries confidence.

Heuristic similarity does not auto-merge incidents.

---

## 56. Change Correlation

Incident UI can show recent:

```text
deployments
config changes
feature flags
runner image rollouts
infrastructure applies
network policy changes
releases
```

These are evidence, not proof of cause.

---

## 57. Failure Diagnosis Integration

Part 48 may provide:

```text
failure clusters
reproduction results
bisect results
root-cause hypotheses
```

Incident subsystem links them.

---

## 58. Reliability Integration

Part 50 provides:

```text
SLO status
error-budget burn
resilience state
```

An incident may exist without SLO violation.

An SLO violation may not always require incident declaration.

---

## 59. Federation

Incidents can be:

```text
site-local
regional
global
```

Regional incidents can become children of global incidents.

---

## 60. Disconnected Sites

A disconnected site may declare local incidents using globally unique IDs.

Timeline events reconcile append-only after reconnection where authority permits.

No last-write-wins incident history.

---

## 61. Incident Visibility

```rust
pub enum IncidentVisibility {
    Restricted,
    Internal,
    TenantScoped,
    PublicSummary,
}
```

Security incidents default to restricted.

---

## 62. Multi-Tenant Incident

A single infrastructure incident may affect multiple tenants.

Tenant-facing views must prevent cross-tenant leakage.

---

## 63. Data Lifecycle

Part 46 governs:

```text
incident records
timeline events
communications
postmortems
evidence snapshots
corrective actions
```

Security/compliance incidents may have extended retention or legal holds.

---

## 64. Audit

Audit events include:

```text
severity changes
role assignments
incident close/reopen
change freezes
break-glass linkage
public communication
postmortem approval
corrective-action cancellation
```

Routine timeline events remain operational immutable records.

---

## 65. API

Potential:

```text
GET  /v1/incidents
POST /v1/incidents
GET  /v1/incidents/{id}
POST /v1/incidents/{id}/acknowledge
POST /v1/incidents/{id}/roles
POST /v1/incidents/{id}/mitigations
POST /v1/incidents/{id}/updates
POST /v1/incidents/{id}/resolve
POST /v1/incidents/{id}/close
```

---

## 66. Permissions

```text
incident.read
incident.declare
incident.acknowledge
incident.command
incident.update
incident.resolve
incident.close
incident.public_communicate
incident.freeze.manage
postmortem.edit
postmortem.approve
```

Sensitive incident visibility can require additional permission.

---

## 67. Dioxus UI

Pages:

```text
Incidents
Incident Room
On-Call
Escalation Policies
Postmortems
Corrective Actions
Incident Analytics
```

Incident list shows:

```text
severity
state
title
affected scope
commander
duration
last update
```

---

## 68. CLI

```text
forgeyard incident list
forgeyard incident declare
forgeyard incident show
forgeyard incident ack
forgeyard incident assign
forgeyard incident update
forgeyard incident resolve
forgeyard incident close
forgeyard incident timeline
forgeyard incident doctor
```

Postmortem:

```text
forgeyard incident postmortem create
forgeyard incident postmortem show
forgeyard incident action list
```

---

## 69. Incident Health

```rust
pub enum IncidentSubsystemHealth {
    Healthy,
    EscalationDegraded,
    NotificationDegraded,
    TimerDegraded,
    Unhealthy,
}
```

---

## 70. Doctor

```text
forgeyard incident doctor
```

Checks:

```text
on-call schedule gaps
expired overrides
broken escalation targets
notification channel unavailable
stuck incidents
overdue mandatory postmortems
stale active freezes
```

---

## 71. Observability Metrics

```text
incidents_declared_total
incidents_active
incident_ack_seconds
incident_resolution_seconds
incident_escalations_total
incident_updates_missed_total
postmortems_overdue_total
corrective_actions_overdue_total
```

Use low-cardinality labels only:

```text
severity
incident_type
state
```

Do not put principal/person identity into aggregate metrics.

---

## 72. Search

Part 31 can index authorized incident metadata.

Search remains permission-filtered.

---

## 73. AI Assistance

Part 55 may:

```text
draft timeline summary
summarize evidence
suggest contributing factors
draft postmortem sections
suggest corrective action wording
```

It cannot:

```text
declare confirmed root cause
close critical incident
approve public communication
execute privileged mitigation
```

without normal human/policy control.

---

## 74. Runbook Automation

Read-only diagnostics may be automated.

Privileged remediation still uses canonical subsystem commands.

---

## 75. Incident Templates

Part 42 can define standard response checklists:

```text
assign IC
assess impact
establish communications
freeze risky changes
capture evidence
verify recovery
```

Checklist completion never grants privilege.

---

## 76. Incident Versioning

```rust
pub struct IncidentVersion(u64);
```

Use optimistic concurrency for incident mutations.

---

## 77. Incident Commander Handoff

One current IC baseline.

Handoff appends timeline event.

Prior IC history remains visible.

---

## 78. Reconciliation

Escalation reconciler checks:

```text
timers
ack state
fallback targets
notification state
```

Incident reconciler checks:

```text
state consistency
active freeze
mandatory update cadence
postmortem requirements
external channel linkage
```

---

## 79. High Availability

Multiple incident service instances operate safely using normal durable metadata.

No separate Raft cluster required for ordinary incident state.

---

## 80. Disaster Recovery

Incident metadata and timeline are backed up.

However:

> **Forgeyard must not be the only place where Forgeyard’s own emergency contacts and recovery runbooks exist.**

Maintain:

```text
secure offline on-call export
offline recovery runbook
external/manual status communication path
break-glass emergency contacts
```

This prevents circular dependency during total control-plane outage.

---

## 81. Air-Gap

Air-gapped deployments retain local:

```text
incident declaration
timeline
roles
mitigation tracking
postmortem
```

External notification providers may be unavailable.

---

## 82. Incident Bundle

```rust
pub struct IncidentBundle {
    pub incident: IncidentId,
    pub timeline: CasObjectRef,
    pub evidence_manifest: CasObjectRef,
    pub postmortem: Option<PostmortemId>,
}
```

Audience-specific redaction applies to exports.

---

## 83. Testkit

```text
forgeyard-incident-testkit/src/
├── lib.rs
├── incident.rs
├── oncall.rs
├── escalation.rs
├── timeline.rs
├── mitigation.rs
├── communication.rs
├── postmortem.rs
└── assertions.rs
```

---

## 84. Core Tests

### Incident State
- valid lifecycle transitions;
- resolve without root-cause confirmation;
- reopen preserves history.

### On-Call
- timezone/DST correctness;
- schedule gap detection;
- override behavior;
- fallback routing.

### Escalation
- no acknowledgement triggers next step;
- delivery does not imply ack;
- durable timer survives restart.

### Roles
- IC uniqueness;
- handoff records timeline;
- incident role does not grant production permission.

### Mitigation
- mitigation references canonical action;
- emergency path still requires break-glass authorization;
- no hidden shell execution.

### Change Freeze
- normal deploy blocked;
- authorized emergency exception succeeds;
- stale freeze detected.

### Communication
- internal/public redaction;
- duplicate delivery idempotency;
- correction appends new update.

### Security
- operational incident can resolve while security case remains active;
- restricted evidence inaccessible to unauthorized tenant/user.

### Postmortem
- severity policy requires postmortem;
- unknown root cause allowed;
- corrective action evidence retained.

### Federation/DR
- regional incident reconciliation;
- offline emergency contacts available;
- notification provider outage does not corrupt incident state.

---

## 85. Chaos Tests

Inject:

```text
notification provider outage
DB failover
timer worker crash
federation partition
control-plane partial outage
external status provider failure
```

Expected outcome:

```text
incident truth remains durable
timers recover
escalation resumes
communication failures are explicit
no duplicate privileged mitigation occurs
```

---

## 86. Scale Tests

Test:

```text
large alert storms
many simultaneous incidents
many tenant-scoped communications
high timeline write volume
large evidence-link sets
```

---

## 87. Implementation Phases

### Phase 1 — Incident Model & Timeline
Build canonical state and append-only timeline.

### Phase 2 — On-Call & Escalation
Add schedules, overrides, durable timers, acknowledgements.

### Phase 3 — Incident Room & Communications
Operational UX.

### Phase 4 — Mitigation & Change Freeze
Integrate canonical action systems.

### Phase 5 — Postmortem & Corrective Actions
Learning lifecycle.

### Phase 6 — Service Catalog & Blast Radius
Impact intelligence.

### Phase 7 — Security & Reliability Integration
Cross-system response.

### Phase 8 — Federation & Air-Gap
Regional/local incident operation.

### Phase 9 — Analytics & Recurrence
Process improvement.

### Phase 10 — External Status/Chat Providers
Interoperability.

### Phase 11 — Offline Emergency Export
Self-recovery resilience.

### Phase 12 — Chaos, Scale & Security Hardening
Production readiness.

---

## 88. Acceptance Tests

1. Incident is first-class and distinct from alert.
2. Severity is explicit and auditable.
3. Unknown impact remains representable.
4. Resolution does not imply confirmed root cause.
5. Timeline is append-only.
6. Corrections append rather than rewrite.
7. Incident roles do not grant privileged permissions.
8. On-call schedules are timezone-aware.
9. Schedule gaps have explicit fallback.
10. Acknowledgement is explicit.
11. Escalation timers are durable.
12. Mitigations link to canonical subsystem actions.
13. Incident UI exposes no privileged backdoor.
14. Break-glass remains authz/policy/audit controlled.
15. Change freezes block risky normal changes.
16. Emergency changes remain explicit exceptions.
17. Public communication is separated from internal evidence.
18. Security incident lifecycle remains distinct.
19. Provider status does not substitute for local recovery verification.
20. Duplicate alerts can correlate without evidence loss.
21. Postmortem requirements are severity/policy driven.
22. Unknown root cause is allowed.
23. Corrective actions are evidence-linked.
24. Incident analytics do not rank individual employees.
25. Tenant incident data is isolated.
26. Federation supports regional/global incidents.
27. Air-gapped sites retain local incident capability.
28. Forgeyard has offline emergency paths when Forgeyard itself is unavailable.
29. Incident metadata/timeline is backed up.
30. Incident data obeys lifecycle/privacy controls.
31. Recurrence detection does not auto-merge heuristic matches.
32. Recent change correlation remains evidence, not causal proof.
33. AI remains advisory.
34. Notification provider outage degrades gracefully.
35. Forgeyard dogfoods the incident system for its own operational incidents.

---

## 89. Production Readiness Gates

Do not call incident management production-ready until:

```text
incident lifecycle/timeline invariants pass
on-call gaps are detectable
durable escalation works
role/permission separation is enforced
change-freeze integration works
break-glass remains controlled
tenant/public redaction tests pass
postmortem/action lifecycle works
offline emergency contacts/runbooks exist
notification outage and control-plane DR exercises pass
```

---

## 90. Architectural Invariants

1. alert is not incident;
2. incident state is explicit;
3. severity is explicit/audited;
4. unknown impact/root cause is representable;
5. timeline is append-only;
6. roles do not imply permissions;
7. acknowledgement is explicit;
8. escalation timers are durable;
9. on-call gaps fail visibly;
10. mitigations use canonical subsystem commands;
11. no incident-specific privileged backdoor exists;
12. break-glass remains policy/authz/audit bound;
13. change freeze is explicit/scoped;
14. public/internal communication are separate;
15. security incident lifecycle remains distinct;
16. provider status is evidence only;
17. incident correlation does not erase signals;
18. postmortems are evidence-linked;
19. corrective actions are tracked;
20. analytics improve systems, not rank people;
21. tenant isolation applies;
22. federation preserves scope/authority;
23. air-gap/local incident response works;
24. Forgeyard has an external/offline emergency path;
25. communication corrections append history;
26. incident command handoff is explicit;
27. reconciliation restores timers/state after failures;
28. lifecycle/privacy govern incident data;
29. AI remains advisory;
30. Forgeyard dogfoods its own incident system.

---

## 91. Final Target Architecture

```text
                     Alert / Signal
                          │
                          ▼
                       Triage
                          │
                          ▼
                      IncidentId
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
         On-Call        Impact        Evidence
            │             │             │
            └─────────────┼─────────────┘
                          ▼
                  Incident Commander
                          │
                          ▼
                 Mitigation Actions
                          │
                          ▼
                 Monitor / Resolve
                          │
                          ▼
                    Postmortem
                          │
                          ▼
                Corrective Actions
```

Emergency mitigation remains:

```text
IncidentId
   ↓
proposed mitigation
   ↓
normal authz / policy / break-glass
   ↓
canonical subsystem action
   ↓
timeline + evidence
```

The key guarantee is:

> **Forgeyard can coordinate serious operational incidents without creating a parallel emergency control plane. Incidents organize people, evidence, timing, communication, and mitigations; the actual technical changes still pass through the same governed subsystems that protect normal operation.**

---

## 92. Extended Architecture Sequence

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
12 Secrets / Trust
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
26 Self-Hosting / Bootstrap / Release Forgeyard
27 Multi-Tenancy / Quotas / Resource Governance
28 Audit / Compliance / Security Governance
29 Notifications / Alerting / Human Workflow
30 Entitlements / Licensing / Subscription
31 Search / Indexing / Operational Analytics
32 Test Results / Quality / Coverage / Flaky Intelligence
33 Benchmark / Performance / Load / Capacity
34 Monorepo / Dependency Graph / Affected Work
35 Developer Experience / Local Dev
36 Dependency / Registry / Mirror Governance
37 Static Analysis / Security Findings
38 Cache / Remote Cache / Correctness
39 Configuration / Feature Flags / Runtime
40 Security / Threat Model / Incident Response
41 Release Distribution / Update Delivery
42 Workflow Templates / Golden Paths
43 Runner Fleet Autoscaling / Provisioning
44 Pipeline Triggers / Schedules / Dispatch
45 Cost Accounting / FinOps
46 Data Lifecycle / Retention / Privacy
47 CI/CD Migration / Compatibility
48 Failure Diagnosis / Reproduction / Bisect
49 Service Catalog / Ownership / Developer Portal
50 Reliability / SLO / Error Budget / Resilience
51 Multi-Region Federation / Edge / Disconnected
52 Artifact Registry / OCI / Package Distribution
53 Infrastructure-as-Code / Preview Environments / Drift
54 Merge Queue / Speculative Integration / Batch Validation
55 AI-Assisted CI Optimization / Engineering Copilot
56 Test Data / Fixtures / Ephemeral Databases / Service Virtualization
57 API / ABI / Schema / Protocol Compatibility / Contract Evolution
58 Runner Image Factory / Golden Images / Patch Management / Baseline Attestation
59 Network Connectivity / Private Resource Access / Egress / Tunneling / Zero-Trust Connectivity
60 Workflow Concurrency / Distributed Locks / Idempotency / Reservations / Exclusive Coordination
61 Incident Management / On-Call / Escalation / Response Coordination / Postmortem
```
