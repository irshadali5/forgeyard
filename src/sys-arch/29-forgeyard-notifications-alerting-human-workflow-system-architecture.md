# 29 — Forgeyard Notifications, Alerting & Human Workflow System Architecture

**Document type:** Core Notification, Alert Delivery & Human Workflow Communication System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** in-app notifications, email, chat integrations, outbound webhooks, actionable approval requests, escalation, digests, preferences, quiet hours, delivery receipts, deduplication, provider retry/reconciliation, templates, localization, incident/security alerts, and human-in-the-loop workflow communication  
**Architecture style:** Event-driven, recipient-aware, tenant-scoped, idempotent, channel-neutral, preference-controlled, policy-aware, delivery-reconciled, and strictly non-authoritative for approvals or protected state transitions  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Events/Reconciliation, Observability/Alerts, Policy/Authz/Identity, Change Proposal, Release, Deployment, Audit/Compliance, API/Axum, Dioxus UI, Plugins, SCM, and Multi-Tenancy. It provides the human communication plane referenced throughout those systems.

---

# 1. Purpose

Forgeyard produces many events that require human attention.

Examples:

```text
run failed
approval requested
release ready
deployment degraded
runner offline
secret expiring
certificate expiring
backup failed
restore verification failed
quota near limit
security alert
break-glass activated
plugin quarantined
SCM integration unhealthy
```

Without a dedicated subsystem, each feature tends to implement its own:

```text
email logic
Slack logic
webhook retry
recipient lookup
message formatting
deduplication
preferences
```

That creates inconsistency and security risk.

The central rule is:

> **Domain systems emit facts and requests for attention; the notification subsystem decides how, when, and where to communicate them.**

A second rule is:

> **A notification can request or link to an approval, but it never becomes the approval authority. The authoritative decision still occurs through Forgeyard's policy/authz-protected domain API.**

A third rule is:

> **Notification delivery is at-least-once and reconciled. Duplicate provider deliveries must be harmless, and provider success must never be confused with domain-action success.**

---

# 2. Architectural Position

```text
                    Domain / Security Event
                              │
                              ▼
                    Notification Intent
                              │
                 ┌────────────┼────────────┐
                 ▼            ▼            ▼
              Routing      Preferences    Policy
                 │            │            │
                 └────────────┼────────────┘
                              ▼
                         Delivery Plan
                              │
             ┌────────────────┼─────────────────┐
             ▼                ▼                 ▼
          In-App            Email            Chat/Webhook
             │                │                 │
             └────────────────┼─────────────────┘
                              ▼
                      Delivery Receipts
                              │
                              ▼
                        Reconciliation
```

---

# 3. Goals

The subsystem MUST:

1. define notification identity;
2. define recipient identity;
3. define notification kinds;
4. support in-app notifications;
5. support email;
6. support outbound webhook;
7. support chat providers;
8. support plugin-based providers;
9. support per-user preferences;
10. support tenant/org/project routing;
11. support severity;
12. support actionable notifications;
13. support approval-request links;
14. support escalation;
15. support deduplication;
16. support aggregation;
17. support digests;
18. support quiet hours;
19. support delivery retries;
20. support delivery receipts;
21. support unknown provider outcomes;
22. support template versioning;
23. support localization;
24. support rate limiting;
25. support provider health;
26. support security-critical bypass rules;
27. support audit integration;
28. support multi-tenancy;
29. support HA;
30. remain separate from domain authority.

---

# 4. Non-Goals

The notification subsystem does not:

```text
approve releases
merge changes
cancel runs
deploy production
change policy
act as identity provider
replace incident-management systems
replace domain events
```

---

# 5. Workspace Structure

```text
crates/notification/
├── forgeyard-notification/
├── forgeyard-notification-model/
├── forgeyard-notification-intent/
├── forgeyard-notification-routing/
├── forgeyard-notification-preference/
├── forgeyard-notification-template/
├── forgeyard-notification-delivery/
├── forgeyard-notification-provider/
├── forgeyard-notification-inapp/
├── forgeyard-notification-email/
├── forgeyard-notification-webhook/
├── forgeyard-notification-chat/
├── forgeyard-notification-digest/
├── forgeyard-notification-escalation/
├── forgeyard-notification-reconcile/
├── forgeyard-notification-health/
└── forgeyard-notification-testkit/
```

Use modules first; split only at genuine runtime/provider/security boundaries.

---

# 6. NotificationId

```rust
pub struct NotificationId(Ulid);
```

Represents a logical human-facing notification.

---

# 7. Notification Intent

```rust
pub struct NotificationIntent {
    pub id: NotificationIntentId,
    pub kind: NotificationKind,
    pub scope: ResourceScope,
    pub severity: NotificationSeverity,
    pub subject: NotificationSubject,
    pub recipients: RecipientSelector,
    pub action: Option<NotificationAction>,
    pub dedup: NotificationDedupKey,
}
```

---

# 8. Intent vs Delivery

One logical intent may produce multiple channel deliveries.

---

# 9. DeliveryId

```rust
pub struct NotificationDeliveryId(Ulid);
```

---

# 10. Notification Kind

Examples:

```rust
pub enum NotificationKind {
    RunFailed,
    RunCompleted,
    ApprovalRequested,
    ApprovalExpiring,
    ReleaseReady,
    ReleaseFailed,
    DeploymentStarted,
    DeploymentDegraded,
    DeploymentRolledBack,
    RunnerOffline,
    DeviceQuarantined,
    SecretExpiring,
    CertificateExpiring,
    BackupFailed,
    RestoreVerificationFailed,
    QuotaNearLimit,
    QuotaExceeded,
    SecurityAlert,
    BreakGlassActivated,
    PluginQuarantined,
    Custom(NotificationKindId),
}
```

---

# 11. Severity

```rust
pub enum NotificationSeverity {
    Informational,
    Success,
    Warning,
    Error,
    Critical,
    ActionRequired,
}
```

---

# 12. ActionRequired

Used when user/operator needs to act.

---

# 13. Critical

May bypass some quiet-hour suppression according to policy.

---

# 14. Notification Subject

```rust
pub enum NotificationSubject {
    Run(RunId),
    Job(JobId),
    ChangeProposal(ChangeProposalId),
    Release(ReleaseId),
    Deployment(DeploymentId),
    Runner(RunnerId),
    Device(DeviceId),
    Plugin(InstalledPluginId),
    Backup(BackupSetId),
    SecurityReview(SecurityReviewId),
    Custom(...),
}
```

---

# 15. Recipient

Never just free-form email string in core.

---

# 16. Recipient Selector

```rust
pub enum RecipientSelector {
    Principal(PrincipalId),
    Principals(Vec<PrincipalId>),
    RoleMembers(RoleId, AuthorizationScope),
    ProjectWatchers(ProjectId),
    TenantAdmins(TenantId),
    OnCall(OnCallRouteId),
    ExplicitEndpoint(NotificationEndpointId),
}
```

---

# 17. Identity Resolution

Principal -> configured endpoints/preferences.

---

# 18. Email Address

Stored in identity/contact metadata, not notification authority.

---

# 19. Channel

```rust
pub enum NotificationChannel {
    InApp,
    Email,
    Webhook,
    Chat(ChatProviderKind),
    Plugin(NotificationProviderId),
}
```

---

# 20. In-App

Always available if user has Forgeyard account.

---

# 21. Email

General external channel.

---

# 22. Chat

Examples:

```text
Slack
Microsoft Teams
Mattermost
Discord-like internal integrations
```

adapter/plugin based.

---

# 23. Webhook

Machine-to-machine notification sink.

---

# 24. Channel-Neutral Core

No Slack-specific type in core notification model.

---

# 25. Notification Endpoint

```rust
pub struct NotificationEndpoint {
    pub id: NotificationEndpointId,
    pub owner: NotificationEndpointOwner,
    pub channel: NotificationChannel,
    pub config: NotificationEndpointConfigRef,
    pub state: NotificationEndpointState,
}
```

---

# 26. Endpoint Owner

```text
Principal
Project
Organization
Tenant
System
```

---

# 27. Endpoint Config

Secrets use SecretRef.

---

# 28. Endpoint State

```text
Active
Disabled
Degraded
Invalid
```

---

# 29. User Preferences

```rust
pub struct NotificationPreferences {
    pub principal: PrincipalId,
    pub rules: Vec<NotificationPreferenceRule>,
    pub quiet_hours: Option<QuietHours>,
    pub locale: Option<LocaleId>,
}
```

---

# 30. Preference Rule

```rust
pub struct NotificationPreferenceRule {
    pub kind: NotificationKindSelector,
    pub minimum_severity: NotificationSeverity,
    pub channels: Vec<NotificationChannel>,
    pub digest: DigestPreference,
}
```

---

# 31. Preferences Cannot Suppress Mandatory Security Notices

Policy defines non-suppressible classes.

---

# 32. Mandatory Notifications

Examples:

```text
credential compromise
break-glass on user's account
security-critical admin action
```

as policy chooses.

---

# 33. Quiet Hours

```rust
pub struct QuietHours {
    pub timezone: TimeZoneId,
    pub start: LocalTime,
    pub end: LocalTime,
    pub days: BTreeSet<Weekday>,
}
```

---

# 34. Quiet Hours Behavior

```text
delay low priority
send critical immediately
```

---

# 35. Quiet Hours Are Delivery Preference

Domain action remains unaffected.

---

# 36. Digest Preference

```rust
pub enum DigestPreference {
    Immediate,
    Hourly,
    Daily,
    NeverDigest,
}
```

---

# 37. NeverDigest

For:

```text
critical alerts
approval expiry
security events
```

---

# 38. Notification Routing

```text
intent
  ↓
recipient resolution
  ↓
preference evaluation
  ↓
policy mandatory rules
  ↓
channel endpoints
```

---

# 39. Routing Is Deterministic

Given same intent/preferences/policy snapshot.

---

# 40. Delivery Plan

```rust
pub struct NotificationDeliveryPlan {
    pub intent: NotificationIntentId,
    pub recipients: Vec<ResolvedRecipient>,
    pub deliveries: Vec<PlannedDelivery>,
}
```

---

# 41. Planned Delivery

```rust
pub struct PlannedDelivery {
    pub recipient: ResolvedRecipient,
    pub channel: NotificationChannel,
    pub endpoint: NotificationEndpointId,
    pub template: NotificationTemplateRef,
    pub send_at: Timestamp,
}
```

---

# 42. Recipient Privacy

Do not expose recipient lists across tenant boundaries.

---

# 43. Multi-Tenant Routing

Every intent is tenant-scoped unless truly system-wide.

---

# 44. System-Wide Notifications

Examples:

```text
platform maintenance
security incident
```

require system-level service.

---

# 45. In-App Notification

Persisted state.

---

# 46. InAppNotification

```rust
pub struct InAppNotification {
    pub id: NotificationId,
    pub principal: PrincipalId,
    pub intent: NotificationIntentId,
    pub state: InAppNotificationState,
    pub created_at: Timestamp,
}
```

---

# 47. In-App State

```text
Unread
Read
Archived
```

---

# 48. Read State

Presentation only.

Does not affect action authority.

---

# 49. Notification Center

Dioxus UI.

---

# 50. Badge Count

Derived.

---

# 51. Deep Link

Safe internal route.

---

# 52. Actionable Notification

```rust
pub struct NotificationAction {
    pub kind: NotificationActionKind,
    pub resource: AuditResourceRef,
    pub required_permission: Permission,
    pub expires_at: Option<Timestamp>,
}
```

---

# 53. Action Examples

```text
ViewRun
ReviewChange
ApproveRelease
ReviewDeployment
OpenSecurityReview
```

---

# 54. No Embedded Authorization Token

Critical.

---

# 55. Email "Approve" Link

Must lead to Forgeyard UI/API requiring normal authentication/authz.

---

# 56. One-Click Approval

Not baseline.

---

# 57. If Added Later

Would require:

```text
single-use signed action token
short expiry
exact resource/candidate binding
step-up policy
audit
```

---

# 58. Recommended Baseline

No approval directly from email/chat.

---

# 59. Why

Avoid forwarded-link authorization errors.

---

# 60. Approval Request

Notification points to exact:

```text
ProposalRevisionId
ReleaseCandidateId
DeploymentPlanId
```

---

# 61. Stale Approval

When candidate changes, notification becomes stale.

---

# 62. UI

Show:

```text
This approval request is no longer current.
```

---

# 63. Notification Action Freshness

Server validates current state.

---

# 64. Escalation

```rust
pub struct EscalationPolicy {
    pub id: EscalationPolicyId,
    pub steps: Vec<EscalationStep>,
}
```

---

# 65. Escalation Step

```rust
pub struct EscalationStep {
    pub after: Duration,
    pub recipients: RecipientSelector,
    pub channels: Vec<NotificationChannel>,
}
```

---

# 66. Use Cases

```text
release approval pending
production deployment degraded
security incident unacknowledged
```

---

# 67. Escalation Trigger

Based on authoritative unresolved condition.

---

# 68. Notification Not Acknowledged vs Condition Unresolved

Different.

---

# 69. Recommended

Escalate based on domain condition when possible.

---

# 70. Example

Release approval:

```text
still AwaitingApproval after 2h
```

not merely email unread.

---

# 71. Durable Escalation Timer

Stored.

---

# 72. Restart Safe

Events/Reconciliation.

---

# 73. On-Call Route

```rust
pub struct OnCallRouteId(Ulid);
```

---

# 74. On-Call Integration

May map to external incident tool/plugin.

---

# 75. Built-In Rotation Scheduler

Not required baseline.

---

# 76. External On-Call

Adapter/plugin returns current recipients.

---

# 77. Notification Template

```rust
pub struct NotificationTemplate {
    pub id: NotificationTemplateId,
    pub version: NotificationTemplateVersion,
    pub kind: NotificationKind,
    pub channel: NotificationChannelKind,
    pub locale: LocaleId,
}
```

---

# 78. Template Versioning

Immutable.

---

# 79. Template Inputs

Typed safe context.

---

# 80. No Arbitrary Domain Object Serialization

Use explicit presentation model.

---

# 81. Template Data

```rust
pub struct NotificationViewModel {
    pub title: BoundedString,
    pub summary: BoundedString,
    pub resource_name: Option<BoundedString>,
    pub deep_link: Option<SafeUrl>,
    pub fields: Vec<NotificationField>,
}
```

---

# 82. Secret Values

Never template input.

---

# 83. Sensitive Logs

Do not embed full logs in email/chat.

---

# 84. Failure Summary

Safe short summary + Forgeyard link.

---

# 85. Markdown

Channel-safe rendering.

---

# 86. HTML Email

Escaped/sanitized.

---

# 87. Chat Formatting

Provider adapter.

---

# 88. Localization

Message templates may have locales.

---

# 89. Fallback

Default English.

---

# 90. Stable Error Codes

Can localize UI notification text.

---

# 91. Provider Trait

```rust
#[async_trait]
pub trait NotificationProvider {
    async fn deliver(
        &self,
        request: NotificationProviderRequest,
    ) -> Result<NotificationProviderResult, NotificationProviderError>;

    async fn inspect(
        &self,
        request: NotificationProviderInspectRequest,
    ) -> Result<NotificationProviderInspection, NotificationProviderError>;
}
```

---

# 92. Inspect

Needed for providers supporting receipt/status.

---

# 93. Provider Capability

```rust
pub struct NotificationProviderCapabilities {
    pub supports_receipts: bool,
    pub supports_idempotency: bool,
    pub supports_updates: bool,
    pub supports_threads: bool,
}
```

---

# 94. Delivery State

```rust
pub enum NotificationDeliveryState {
    Pending,
    Scheduled,
    InProgress,
    Delivered,
    Failed,
    Unknown,
    Suppressed,
    Cancelled,
}
```

---

# 95. Delivered

Provider accepted/delivered according to channel semantics.

Does not mean user read it.

---

# 96. Unknown

Remote outcome uncertain.

---

# 97. Unknown Delivery

Inspect before retry where possible.

---

# 98. At-Least-Once

Duplicate delivery may occur.

---

# 99. Idempotency Key

```rust
pub struct NotificationIdempotencyKey(Digest);
```

---

# 100. Key Inputs

```text
intent
recipient
channel
template version
semantic delivery attempt class
```

---

# 101. Provider Idempotency

Use native provider key if available.

---

# 102. Email

SMTP generally lacks strong idempotency.

Forgeyard dedups before send, but ambiguous network result may still duplicate.

---

# 103. Honesty

Do not claim exactly-once email.

---

# 104. Retry Class

```rust
pub enum NotificationRetryClass {
    SafeRetry,
    RetryAfterInspect,
    DoNotRetry,
}
```

---

# 105. Retryable

Examples:

```text
timeout
429
temporary provider 5xx
```

---

# 106. Non-Retryable

```text
invalid email
endpoint revoked
template invalid
permission/config error
```

---

# 107. Backoff

Exponential + jitter.

---

# 108. Max Attempts

Channel/severity specific.

---

# 109. Critical Delivery

May escalate to alternate channel if primary fails.

---

# 110. Channel Failover

Example:

```text
chat failed
  ↓
email
```

only configured policy.

---

# 111. Do Not Spray All Channels by Default

Respect preferences/routing.

---

# 112. Deduplication

Prevents repeated identical logical notifications.

---

# 113. Dedup Window

Notification-kind specific.

---

# 114. Example

Runner offline flapping:

```text
one active incident-like notification
```

instead of 50 emails.

---

# 115. Dedup Key

```text
kind + subject + recipient + semantic state version
```

---

# 116. State Transition Notification

Use exact transition.

---

# 117. Recovery Notification

Separate:

```text
RunnerRecovered
DeploymentRecovered
```

if desired.

---

# 118. Flapping Suppression

Hysteresis.

---

# 119. Alert Aggregation

Group similar notifications.

---

# 120. Example

10 failed jobs in same run:

```text
one run-failed notification
```

rather than 10 emails.

---

# 121. Digest

Collect low-priority notifications.

---

# 122. Digest Window

Hourly/daily.

---

# 123. Digest Content

Summaries + links.

---

# 124. Action-Required Item

Not delayed into daily digest unless policy explicitly permits.

---

# 125. Digest State

```rust
pub struct NotificationDigest {
    pub id: NotificationDigestId,
    pub principal: PrincipalId,
    pub window: TimeWindow,
    pub intents: Vec<NotificationIntentId>,
}
```

---

# 126. Digest Delivery

Normal provider pipeline.

---

# 127. Email Provider

Built-in SMTP provider.

---

# 128. SMTP Configuration

```text
host
port
TLS
auth SecretRef
sender
```

---

# 129. STARTTLS/TLS

Production required according to config/security policy.

---

# 130. SMTP Credentials

SecretRef.

---

# 131. Email From

Validated domain/address.

---

# 132. Bounce

If provider gives DSN/webhook, adapter can update endpoint health.

---

# 133. Email Endpoint Invalid

Disable after repeated permanent failures with notification/admin visibility.

---

# 134. Chat Provider

Could be built-in/plugin.

---

# 135. Slack-Like Integration

Use app/bot token or incoming webhook depending adapter.

---

# 136. Token

SecretRef.

---

# 137. Chat Message Update

If provider supports, Forgeyard can update one status thread instead of spam.

---

# 138. Example

Deployment:

```text
Started
→ Canary passed
→ 50%
→ Succeeded
```

one threaded/updateable message.

---

# 139. But Core History

Still separate notification delivery records.

---

# 140. Outbound Webhook

Machine integration.

---

# 141. Webhook Endpoint

```rust
pub struct NotificationWebhookEndpoint {
    pub url: SafeExternalUrl,
    pub signing_secret: SecretRef,
    pub event_filter: NotificationEventFilter,
}
```

---

# 142. SSRF

Endpoint URL validated and network policy applied.

---

# 143. Private Networks

Blocked by default for hosted mode unless allowed.

---

# 144. Webhook Payload

Versioned JSON.

---

# 145. Signature

HMAC/signature header.

---

# 146. Timestamp

Include to prevent replay.

---

# 147. Delivery ID

Include.

---

# 148. Webhook Receiver Dedup

Consumer uses DeliveryId.

---

# 149. Payload Example Shape

```json
{
  "version": "1",
  "delivery_id": "...",
  "kind": "release.ready",
  "tenant_id": "...",
  "subject": {
    "type": "release",
    "id": "..."
  }
}
```

---

# 150. No Secret Payload

Critical.

---

# 151. Outbound Webhook vs SCM Webhook

Different directions/subsystems.

---

# 152. Inbound SCM

Part 21.

---

# 153. Outbound Notification Webhook

Part 29.

---

# 154. Security Alert

Source can be:

```text
audit
trust
authn
policy
scanner
runtime security
```

---

# 155. Security Alert Routing

May bypass user preference.

---

# 156. Security Distribution List

Tenant/system security contacts.

---

# 157. Break-Glass Alert

Immediate.

---

# 158. Secret Reveal Alert

Immediate if configured.

---

# 159. Signing-Key Compromise

Highest severity.

---

# 160. Compliance Review Notification

ActionRequired.

---

# 161. Approval Workflow

Change/release/deployment subsystem creates approval requirement.

---

# 162. Notification Intent

Generated from:

```text
ApprovalRequested
```

event/state.

---

# 163. Approver Resolution

Policy/approval service determines eligible approvers.

Notification layer does not decide approval eligibility.

---

# 164. Eligible Approver Set

Snapshot/reference from domain service.

---

# 165. Avoid Notifying Every Admin

Use exact approval requirement.

---

# 166. Approval Completion

Pending approval notifications can be marked resolved/superseded.

---

# 167. Resolved Notification

In-app action state updated.

---

# 168. Email Cannot Be Recalled

But future clicks validate current state.

---

# 169. Approval Expiry

Durable timer.

---

# 170. Escalation

Notify next eligible approvers if policy.

---

# 171. Separation of Duties

Notification routing respects approval domain's eligibility facts.

---

# 172. Incident Alerts

Part 17 alerting can create NotificationIntent.

---

# 173. Alert Engine

Does not send directly.

---

# 174. Pipeline

```text
metric/health condition
  ↓
Alert
  ↓
NotificationIntent
  ↓
routing/delivery
```

---

# 175. Alert Recovery

Can create resolution notification.

---

# 176. Rate Limiting

Per provider/channel/tenant.

---

# 177. Provider Rate Budget

Avoid provider bans.

---

# 178. Recipient Rate Limit

Avoid spam.

---

# 179. Security Critical

Can exceed normal low-priority per-user limits within safety bounds.

---

# 180. Notification Storm Protection

Global dedup/aggregation/backpressure.

---

# 181. Queue

Durable delivery queue.

---

# 182. Queue Priority

```text
Critical
ActionRequired
Error
Warning
Informational
Digest
```

---

# 183. Backpressure

Low-priority digest can wait.

Critical gets priority.

---

# 184. Bounded Worker Concurrency

Per provider.

---

# 185. Outbox

Domain event -> notification intent can use outbox.

---

# 186. No Long DB Tx Around Provider Call

Critical.

---

# 187. Delivery Attempt

```rust
pub struct NotificationDeliveryAttempt {
    pub id: NotificationAttemptId,
    pub delivery: NotificationDeliveryId,
    pub number: u32,
    pub started_at: Timestamp,
    pub outcome: NotificationAttemptOutcome,
}
```

---

# 188. Provider Message ID

Store if available.

---

# 189. Receipt

```rust
pub struct NotificationReceipt {
    pub delivery: NotificationDeliveryId,
    pub provider_message_id: Option<BoundedString>,
    pub status: NotificationReceiptStatus,
    pub observed_at: Timestamp,
}
```

---

# 190. Receipt Status

```text
Accepted
Delivered
Bounced
Rejected
Read
Unknown
```

channel-dependent.

---

# 191. Do Not Normalize Unsupported Receipt as Delivered

Capability honesty.

---

# 192. In-App Read Receipt

Forgeyard-owned.

---

# 193. Email Read Tracking

Not baseline.

Privacy/security concerns.

---

# 194. Chat Read Tracking

Not relied on.

---

# 195. Delivery Reconciliation

Checks:

```text
InProgress too long
Unknown outcome
provider endpoint invalid
scheduled notification overdue
digest stuck
```

---

# 196. Notification Reconciler

Idempotent.

---

# 197. Stale Approval Intent

Mark superseded.

---

# 198. Escalation Reconciler

Checks unresolved domain condition.

---

# 199. Preference Change

Affects future delivery.

Does not retract already sent messages.

---

# 200. Template Change

New version for future messages.

Historical delivery records retain template version.

---

# 201. Tenant Branding

Optional:

```text
display name
logo URL
sender display name
```

safe tenant-owned config.

---

# 202. Email Domain

Hosted Forgeyard should use controlled sending domain.

---

# 203. Custom Sender Domain

Later enterprise option with DNS verification.

---

# 204. Branding Cannot Spoof System Security Alert

Core security notices preserve Forgeyard/system identity.

---

# 205. Localization

Locale precedence:

```text
user preference
tenant default
system default
```

---

# 206. Time Formatting

Recipient timezone.

---

# 207. Deep Links

Use configured canonical public base URL.

---

# 208. Deep Link Security

Never encode secrets/action authority in URL.

---

# 209. Unsubscribe

For optional informational email.

---

# 210. Mandatory Security Notices

No unsubscribe if policy/legal basis requires.

---

# 211. Preference Center

Dioxus UI.

---

# 212. Notification Center UI

Tabs:

```text
All
Action Required
Runs
Releases
Deployments
Security
System
```

---

# 213. Filters

Read/unread/severity/project.

---

# 214. Bulk Mark Read

Presentation action.

---

# 215. Snooze

Optional.

---

# 216. Snooze

Only delivery visibility; never postpones domain deadline.

---

# 217. Approval Deadline

Still runs.

---

# 218. Notification Detail

Shows subject/current domain state.

---

# 219. Stale Message

Clearly marked.

---

# 220. Preferences UI

Per kind/channel.

---

# 221. Admin Routing UI

Configure:

```text
tenant security route
deployment alerts
backup alerts
quota alerts
```

---

# 222. Provider Health UI

Shows:

```text
SMTP
chat
webhook
plugin providers
```

---

# 223. Delivery Log UI

Permission-gated.

---

# 224. Recipient Privacy

Admin can see metadata according to role.

---

# 225. API

Potential:

```text
GET  /v1/notifications
POST /v1/notifications/{id}/read
POST /v1/notifications/read-all
GET  /v1/notification-preferences
PUT  /v1/notification-preferences
GET  /v1/admin/notification-endpoints
POST /v1/admin/notification-endpoints
GET  /v1/admin/notification-deliveries
```

---

# 226. No Public Arbitrary "Send Email" API

Baseline.

---

# 227. Test Notification

Admin endpoint with controlled template.

---

# 228. Permissions

```text
notification.read
notification.preference.manage
notification.endpoint.read
notification.endpoint.manage
notification.delivery.read
notification.test
```

---

# 229. Send Authority

Domain services/internal notification service.

---

# 230. Plugin Provider

Part 24.

---

# 231. Sandboxed Notification Plugin

Good third-party extension.

---

# 232. Plugin Inputs

Sanitized NotificationProviderRequest.

---

# 233. Plugin Secret

Host-mediated HTTP preferred.

---

# 234. Plugin Cannot Read Arbitrary Recipient Data

Only exact resolved delivery.

---

# 235. Notification Provider Config

RON.

---

# 236. Example

```ron
(
    notifications: (
        email: Some((
            provider: "smtp",
            host: "smtp.example.com",
            port: 587,
            username: "forgeyard",
            password: Secret("notifications/smtp"),
            from: "forgeyard@example.com",
        )),
    ),
)
```

---

# 237. Preference Model Storage

Metadata DB.

---

# 238. Delivery Records

Metadata DB.

---

# 239. Large Payload

Do not store huge email bodies repeatedly if template+view model enough.

---

# 240. Rendered Body Retention

Configurable.

---

# 241. Compliance Need

For critical notification, may preserve body digest/rendered sanitized artifact.

---

# 242. Body Digest

Useful for audit.

---

# 243. Notification Audit

Audit:

```text
endpoint created
preference changed
security notice emitted
critical delivery failed
```

---

# 244. Not Every Informational Delivery in Audit

Would be noisy.

---

# 245. Security-Critical Delivery

Can be audited.

---

# 246. Approval Notification

Approval itself audited in domain; notification delivery separate.

---

# 247. Metrics

```text
notification_intents_total
notification_deliveries_total
notification_delivery_failures_total
notification_delivery_latency_seconds
notification_queue_depth
notification_queue_age_seconds
notification_suppressed_total
notification_deduplicated_total
notification_escalations_total
```

---

# 248. Labels

Low cardinality:

```text
channel
kind_class
severity
result
provider_type
```

---

# 249. No Recipient/Principal Metric Labels

Critical.

---

# 250. Tracing

```text
notification.route
notification.render
notification.deliver
notification.reconcile
notification.digest
notification.escalate
```

---

# 251. Logs

Use NotificationDeliveryId/IntentId.

---

# 252. Health

Checks:

```text
queue
providers
template registry
digest worker
reconciler
```

---

# 253. Doctor

```text
forgeyard notification doctor
```

---

# 254. Doctor Checks

```text
SMTP connection
provider auth
endpoint health
template validity
queue lag
stuck deliveries
```

---

# 255. Safe Doctor

Does not send message by default.

---

# 256. `--send-test`

Explicit.

---

# 257. Email Security

TLS verification.

---

# 258. SPF/DKIM/DMARC

Operational domain concern.

Forgeyard can expose readiness checks/documentation, not automatically own DNS.

---

# 259. DKIM

If Forgeyard itself signs outbound email later, key via SecretRef.

---

# 260. Baseline

Use provider/SMTP infrastructure.

---

# 261. Header Injection

Validate subject/from/reply-to.

---

# 262. HTML Injection

Escape template values.

---

# 263. URL Injection

Only SafeUrl.

---

# 264. Webhook SSRF

Strict.

---

# 265. Chat Markdown Injection

Provider renderer escapes.

---

# 266. Recipient Enumeration

API does not expose arbitrary user emails.

---

# 267. Untrusted Project Names

Escaped.

---

# 268. Untrusted Commit/PR Titles

Escaped.

---

# 269. Log Snippet

Avoid by default; if included, sanitized/bounded.

---

# 270. Privacy

Notification payload should be minimal.

---

# 271. Lock-Screen Push

Future mobile push should avoid sensitive content by default.

---

# 272. Mobile Push

Optional future channel.

---

# 273. Push Provider

APNs/FCM adapter/plugin.

---

# 274. Device Token

Sensitive endpoint data.

---

# 275. Push Action

Deep link only.

---

# 276. No approval token in push.

---

# 277. Delivery Priority

```rust
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}
```

---

# 278. Severity vs Priority

Severity describes meaning.

Priority describes delivery urgency.

---

# 279. Mapping

Policy-driven.

---

# 280. Maintenance Notifications

Scheduled.

---

# 281. Schedule

Durable timer.

---

# 282. Cancellation

If maintenance cancelled, unsent notice can cancel.

---

# 283. Scheduled Release Notification

Can notify on release publication.

---

# 284. Scheduled Notification Is Not Scheduler Cron Authority

Uses durable timer/event subsystem.

---

# 285. Notification Replay

Domain event replay must not blindly resend old notifications.

---

# 286. Replay Mode

Default projection/rebuild only.

---

# 287. Live Redelivery

Explicit admin action with dedup override.

---

# 288. Notification History Rebuild

Can reconstruct in-app projections from durable intents/events if desired.

---

# 289. External Delivery History

Cannot infer all past sends from domain events alone.

Keep delivery records.

---

# 290. HA

Any daemon/worker can process delivery queue.

---

# 291. Claim Lease

Worker claims delivery attempt.

---

# 292. Duplicate Worker

Idempotency protects.

---

# 293. Scheduler Independence

Notification workers do not depend on scheduler unless implemented as system jobs.

---

# 294. Recommended

Dedicated async service/worker pool.

---

# 295. Outage Behavior

Notification provider outage does not stop builds/releases unless notification is policy-required for a specific high-risk workflow.

---

# 296. Example

Security policy could require successful notification of break-glass activation before allowing session use.

---

# 297. Baseline

Record/attempt notification, but domain emergency action should follow explicit security policy—not hidden coupling.

---

# 298. Notification Required Gate

```rust
pub enum NotificationRequirement {
    BestEffort,
    RequiredBeforeAction,
    RequiredAfterAction,
}
```

---

# 299. Use Sparingly

Only high-assurance workflows.

---

# 300. Required Delivery Semantics

Provider "accepted" may be best achievable, not human read.

---

# 301. Capability Honesty

Policy must define acceptable acknowledgment level.

---

# 302. Testkit

```text
forgeyard-notification-testkit/src/
├── lib.rs
├── intent.rs
├── routing.rs
├── preference.rs
├── provider.rs
├── delivery.rs
├── digest.rs
├── escalation.rs
└── assertions.rs
```

---

# 303. Unit Tests

Routing/preferences/template selection.

---

# 304. Mandatory Security Test

User cannot suppress configured critical notice.

---

# 305. Quiet Hours Test

Low priority delayed; critical immediate.

---

# 306. Digest Test

Informational notifications grouped.

---

# 307. Action Required Test

Not delayed into inappropriate digest.

---

# 308. Approval Staleness Test

Candidate changes; old notification cannot authorize old action.

---

# 309. Email Duplicate Test

Ambiguous SMTP outcome recorded Unknown, safe retry policy applied.

---

# 310. Webhook Idempotency Test

Receiver can dedup DeliveryId.

---

# 311. Webhook Signature Test

Valid HMAC.

---

# 312. SSRF Test

Endpoint cannot hit blocked metadata/private network.

---

# 313. Template Injection Test

Malicious project/PR title escaped.

---

# 314. Secret Leakage Test

Secret never appears in notification body.

---

# 315. Cross-Tenant Test

Tenant A recipient does not receive Tenant B event.

---

# 316. Recipient Resolution Test

Role membership resolved correctly.

---

# 317. Preference Change Test

Future notifications reflect change.

---

# 318. Escalation Test

Unresolved approval escalates after timer.

---

# 319. Resolution Test

Completed approval cancels future escalation.

---

# 320. Provider Rate Limit Test

429 respected.

---

# 321. Provider Outage Test

Queue/retry without losing intent.

---

# 322. HA Test

Worker dies mid-delivery; another reconciles.

---

# 323. In-App Read Test

Read state does not change domain state.

---

# 324. SIEM/Audit Separation Test

Notification records do not replace security audit.

---

# 325. Load Test

Large event burst/storm.

---

# 326. Storm Dedup Test

Runner fleet outage does not send millions of emails.

---

# 327. Fuzzing

Fuzz webhook payload/template render inputs/provider responses.

---

# 328. Failure Injection

```text
SMTP timeout
DNS failure
chat provider 429
webhook timeout
template missing
DB restart
worker crash
```

---

# 329. Implementation Phase 1 — In-App Notifications

Core model, preferences, notification center.

---

# 330. Phase 2 — Intent/Router

Domain-event → notification intent.

---

# 331. Phase 3 — Email

SMTP provider.

---

# 332. Phase 4 — Delivery Queue/Reconciliation

Retries/Unknown.

---

# 333. Phase 5 — Approval Requests

Change/release/deploy.

---

# 334. Phase 6 — Digests/Quiet Hours

User experience.

---

# 335. Phase 7 — Security Alerts/Escalation

Critical operational workflows.

---

# 336. Phase 8 — Outbound Webhooks

Signed delivery.

---

# 337. Phase 9 — Chat Providers

Built-in/plugin.

---

# 338. Phase 10 — Admin Routing/Health

Operations.

---

# 339. Phase 11 — Localization/Templates

Polish.

---

# 340. Phase 12 — Mobile Push/Advanced On-Call

Optional future.

---

# 341. Acceptance Tests

1. Domain systems do not send email/chat directly.
2. Domain events create notification intents through typed adapters.
3. One logical intent can create multiple channel deliveries.
4. Notification delivery cannot approve/release/deploy anything.
5. Approval notification binds exact proposal/release/deployment subject.
6. Stale notification action is rejected by authoritative server state.
7. User preferences affect optional notifications only.
8. Mandatory security notices cannot be suppressed when policy forbids.
9. Quiet hours delay only eligible low-priority notifications.
10. Critical notifications can bypass quiet hours according to policy.
11. Digests do not delay critical/action-required items incorrectly.
12. Every external delivery has durable state.
13. Provider calls are outside DB transaction.
14. External delivery uses at-least-once/idempotent semantics.
15. Unknown provider outcome is not blindly retried when inspect is possible.
16. Email exactly-once is never claimed.
17. Webhook delivery includes stable DeliveryId.
18. Outbound webhook is signed.
19. Webhook endpoints are protected from SSRF.
20. Secrets/tokens are never rendered in notification bodies.
21. Untrusted project/change text is escaped.
22. Notification routing is tenant isolated.
23. Notification storm protection aggregates/deduplicates.
24. Provider rate limits/backoff are respected.
25. Delivery worker crash is recoverable by reconciliation.
26. Approval escalation depends on unresolved domain state.
27. Completed approval cancels pending escalation.
28. In-app read/unread is presentation state only.
29. Critical delivery failures are observable/auditable.
30. Plugins can add providers without bypassing core routing/security.
31. Standalone mode works with in-app/local SMTP optional.
32. Distributed mode supports HA delivery workers.
33. Alerting feeds this subsystem rather than implementing providers directly.
34. Audit remains separate from notification history.
35. Forgeyard dogfoods notifications for its own releases, deployments, backups, and security events.

---

# 342. Production Readiness Gates

Do not call notification/human workflow production-ready until:

```text
in-app notification center works
recipient resolution is tenant-safe
preferences/mandatory security rules are stable
SMTP delivery works with retry/reconciliation
approval notifications bind exact immutable subjects
stale actions are rejected
dedup/storm protection works
webhook signing/SSRF protections pass
provider outages do not lose intents
critical delivery health/doctor is available
```

---

# 343. Architectural Invariants

1. notification is communication, not authority;
2. approval links never replace authn/authz/policy;
3. stale notification actions are rejected;
4. domain systems do not implement provider delivery directly;
5. one intent may have many deliveries;
6. delivery state is durable;
7. provider side effects occur outside DB transactions;
8. external delivery is at-least-once;
9. exactly-once email is never claimed;
10. recipient resolution is tenant scoped;
11. preferences cannot suppress mandatory notices when policy forbids;
12. quiet hours do not pause domain deadlines;
13. critical/action-required notifications avoid inappropriate digesting;
14. deduplication prevents notification storms;
15. escalation is based on authoritative unresolved conditions;
16. template values are escaped and bounded;
17. secrets/tokens never enter notification payloads;
18. outbound webhook destinations are SSRF-controlled;
19. webhook payloads are signed;
20. provider credentials are SecretRefs;
21. plugin notification providers remain sandboxed/scoped;
22. notification read state does not mutate domain state;
23. audit remains separate from notification history;
24. alerting emits intents rather than owning delivery;
25. provider capabilities are represented honestly;
26. channel failures can degrade/fail over without corrupting core state;
27. notification delivery is observable and reconcilable;
28. standalone/distributed share semantics;
29. high-assurance required-delivery gates are explicit, not hidden;
30. Forgeyard dogfoods its human communication plane.

---

# 344. Final Target Architecture

```text
                    Domain / Security Fact
                            │
                            ▼
                    Notification Intent
                            │
           ┌────────────────┼─────────────────┐
           ▼                ▼                 ▼
       Recipients       Preferences         Policy
           │                │                 │
           └────────────────┼─────────────────┘
                            ▼
                      Delivery Plan
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
    In-App                Email             Chat/Webhook
       │                    │                    │
       └────────────────────┼────────────────────┘
                            ▼
                    Delivery Records
                            │
                            ▼
                     Reconciliation
```

---

# 345. Final Architectural Position

Approval communication:

```text
authoritative approval requirement
  ↓
NotificationIntent
  ↓
eligible approver routing
  ↓
email/chat/in-app
  ↓
user opens Forgeyard
  ↓
normal authn + authz + policy
  ↓
authoritative approval
```

Security escalation:

```text
security event
  ↓
critical intent
  ↓
mandatory route
  ↓
primary provider
  ↓
failure/timeout
  ↓
alternate channel/escalation
```

Delivery correctness:

```text
persist delivery intent
  ↓
provider call
  ↓
Succeeded / Failed / Unknown
  ↓
inspect/retry/reconcile
```

The key guarantee is:

> **Forgeyard can reliably get the right information to the right people without making communication channels part of the security authority. Email, chat, webhooks, and in-app messages are delivery mechanisms; every protected decision still occurs against Forgeyard's current authoritative state, identity, authorization, policy, and audit boundaries.**

---

# 346. Extended Architecture Sequence

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
```
