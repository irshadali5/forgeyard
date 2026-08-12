# 30 — Forgeyard Entitlements, Licensing, Subscription & Commercial Access-Control System Architecture

**Document type:** Core Entitlement, Licensing, Subscription-State & Commercial Feature-Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** feature entitlements, self-hosted licensing, hosted subscription state, plan capabilities, signed license documents, offline verification, trials, grace periods, read-only fallbacks, seat limits, commercial quotas, billing-provider adapters, usage export, invoice-state integration, tenant suspension policy, license audit, and enterprise edition governance  
**Architecture style:** Signed and versioned entitlements, provider-neutral billing integration, fail-safe feature gating, security-authority separation, graceful degradation, offline-verifiable self-hosted licenses, deterministic entitlement snapshots, and no commercial provider dependency in correctness-critical runtime paths  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds directly on Multi-Tenancy/Quotas/Resource Governance, Policy/Authz/Identity, Audit/Compliance, Notifications, Operations/DR, Plugins, API, Release, Deployment, and Self-Hosting. Resource truth remains in Part 27; this subsystem decides what commercial capabilities a tenant/install is entitled to use.

---

# 1. Purpose

Forgeyard can operate as:

```text
single-developer local software
self-hosted team software
enterprise self-hosted platform
hosted multi-tenant SaaS
managed enterprise service
```

Commercial deployments eventually need answers to questions such as:

```text
is this tenant entitled to HA?
is this installation allowed 50 runners?
is Device Lab included?
is RBE enabled?
how many users/seats are allowed?
is the subscription active?
is the license expired?
what happens during payment failure?
can an offline enterprise installation continue?
can a customer still read/export data after expiration?
```

The central rule is:

> **Commercial entitlement controls feature availability and service level; it never replaces identity, authorization, policy, or tenant-isolation security.**

A second rule is:

> **Billing providers are external accounting systems. Forgeyard must remain operationally coherent if Stripe-like billing APIs are temporarily unavailable.**

A third rule is:

> **Self-hosted licenses must be verifiable offline through signed entitlement documents so an enterprise installation does not require continuous contact with Forgeyard's commercial service.**

---

# 2. Architectural Position

```text
                   Commercial Source
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      Hosted Billing   License Issuer   Trial/Admin
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                  Entitlement Service
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      Feature Access   Limits        Lifecycle
          │              │              │
          └──────────────┼──────────────┘
                         ▼
               Forgeyard Domain/API/UI
                         │
                         ▼
              Authz + Policy + Quotas
```

Commercial entitlement is evaluated **before or alongside** feature admission, but never substitutes for security checks.

---

# 3. Goals

The subsystem MUST:

1. define commercial entitlement identity;
2. define edition/plan capability model;
3. support hosted subscriptions;
4. support self-hosted licenses;
5. support offline verification;
6. support trial periods;
7. support grace periods;
8. support expiration;
9. support read-only fallback;
10. support seat limits;
11. support runner limits;
12. support feature gates;
13. support commercial quota ceilings;
14. support usage export;
15. support billing-provider adapters;
16. support payment-state ingestion;
17. support license renewal;
18. support license revocation;
19. support signed license documents;
20. support license key rotation;
21. support tenant-level entitlements;
22. support installation-level entitlements;
23. support enterprise overrides;
24. support audit;
25. support notification;
26. support HA;
27. support DR;
28. avoid online-license single points of failure;
29. avoid security bypass through commercial gates;
30. remain provider-neutral.

---

# 4. Non-Goals

This subsystem does not itself implement:

```text
tax calculation
invoice PDF generation
payment-card processing
bank settlement
accounting ledger
financial ERP
```

Those belong to billing/payment providers or a separate finance system.

---

# 5. Separation of Concerns

```text
Authn:
Who are you?

Authz:
May you perform this action?

Policy:
Under what operational/security conditions?

Quota:
How much resource may you consume?

Entitlement:
Has the customer purchased/been granted this product capability?

Billing:
What money/payment state backs that entitlement?
```

---

# 6. Critical Separation

A tenant with no subscription must still not gain access to another tenant.

A paid customer must still not bypass authorization.

---

# 7. Workspace Structure

```text
crates/entitlement/
├── forgeyard-entitlement/
├── forgeyard-entitlement-model/
├── forgeyard-entitlement-service/
├── forgeyard-entitlement-store-api/
├── forgeyard-license/
├── forgeyard-license-signature/
├── forgeyard-license-verifier/
├── forgeyard-subscription/
├── forgeyard-commercial-plan/
├── forgeyard-feature-gate/
├── forgeyard-seat/
├── forgeyard-commercial-limit/
├── forgeyard-billing-adapter/
├── forgeyard-usage-export/
├── forgeyard-entitlement-reconcile/
├── forgeyard-entitlement-health/
└── forgeyard-entitlement-testkit/
```

Provider adapters:

```text
crates/billing/
├── forgeyard-billing/
├── forgeyard-billing-webhook/
├── forgeyard-billing-provider/
└── forgeyard-billing-<provider>/
```

Use modules first; split only where dependency/security/runtime boundaries justify.

---

# 8. EntitlementScope

```rust
pub enum EntitlementScope {
    Installation(InstallationId),
    Tenant(TenantId),
    Organization(OrganizationId),
}
```

---

# 9. EntitlementSetId

```rust
pub struct EntitlementSetId(Ulid);
```

---

# 10. Entitlement Set

```rust
pub struct EntitlementSet {
    pub id: EntitlementSetId,
    pub scope: EntitlementScope,
    pub source: EntitlementSource,
    pub version: EntitlementVersion,
    pub state: EntitlementState,
    pub features: BTreeMap<FeatureId, FeatureEntitlement>,
    pub limits: BTreeMap<CommercialLimitKind, CommercialLimit>,
    pub valid_from: Timestamp,
    pub valid_until: Option<Timestamp>,
}
```

---

# 11. Entitlement Source

```rust
pub enum EntitlementSource {
    HostedSubscription(SubscriptionId),
    SignedLicense(LicenseDocumentId),
    Trial(TrialId),
    AdministrativeGrant(EntitlementGrantId),
}
```

---

# 12. FeatureId

Stable namespaced identifier.

Examples:

```text
forgeyard.ha
forgeyard.rbe
forgeyard.device_lab
forgeyard.enterprise_oidc
forgeyard.saml
forgeyard.scim
forgeyard.audit_worm
forgeyard.plugin_external
forgeyard.multi_tenant
forgeyard.advanced_policy
```

---

# 13. Feature Entitlement

```rust
pub enum FeatureEntitlement {
    Enabled,
    Disabled,
    Limited(CommercialLimitRef),
}
```

---

# 14. No Plan-Name Checks in Domain

Bad:

```rust
if plan == "enterprise"
```

Better:

```rust
entitlements.require(FeatureId::HA)
```

---

# 15. Plan

A plan is just a template that produces entitlements.

---

# 16. CommercialPlanId

```rust
pub struct CommercialPlanId(BoundedString);
```

---

# 17. Plan Definition

```rust
pub struct CommercialPlan {
    pub id: CommercialPlanId,
    pub features: BTreeMap<FeatureId, FeatureEntitlement>,
    pub limits: BTreeMap<CommercialLimitKind, CommercialLimit>,
}
```

---

# 18. Plan Versioning

Immutable versions.

---

# 19. Existing Customers

Can remain bound to prior plan version where business policy requires.

---

# 20. Entitlement Version

Monotonic.

---

# 21. Entitlement Snapshot

Critical actions can capture exact entitlement version used for admission.

---

# 22. Not Historical Security Authority

If entitlement later expires, existing historical run remains valid record.

---

# 23. New Work

Uses current effective entitlement.

---

# 24. Entitlement State

```rust
pub enum EntitlementState {
    Active,
    Trial,
    Grace,
    ReadOnly,
    Suspended,
    Expired,
    Revoked,
}
```

---

# 25. Active

Normal.

---

# 26. Trial

Temporary capability.

---

# 27. Grace

New operations may continue within bounded period according to policy.

---

# 28. ReadOnly

Reads/export/history allowed.

New compute/release/deploy denied.

---

# 29. Suspended

More restrictive.

---

# 30. Expired

No new entitled operations.

---

# 31. Revoked

Explicit invalidation, e.g. compromised/stolen license.

---

# 32. Grace Period

```rust
pub struct GracePolicy {
    pub duration: Duration,
    pub allowed_operations: BTreeSet<GraceOperationClass>,
}
```

---

# 33. Suggested Grace Behavior

Allow:

```text
read existing data
download existing artifacts
export backups
view audit
```

Potentially allow limited normal operations for short subscription payment grace in hosted service.

---

# 34. Self-Hosted Expiry

Prefer:

```text
do not brick data
```

Transition to read-only/limited management.

---

# 35. Never Lock Customer Out of Export

Data export/backup access should remain possible unless security/legal policy forbids.

---

# 36. Trial

```rust
pub struct Trial {
    pub id: TrialId,
    pub scope: EntitlementScope,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub plan: CommercialPlanVersionRef,
}
```

---

# 37. Trial Extension

Explicit audited admin/commercial action.

---

# 38. Trial Abuse

Hosted service may use identity/payment/domain verification outside this core.

---

# 39. Signed Self-Hosted License

Primary enterprise self-hosted mechanism.

---

# 40. LicenseDocumentId

```rust
pub struct LicenseDocumentId(Digest);
```

Content-derived.

---

# 41. License Document

Human/export format may use signed JSON/RON.

Canonical payload:

```rust
pub struct LicenseDocument {
    pub schema: LicenseSchemaVersion,
    pub license_id: LicenseId,
    pub customer_id: CustomerCommercialId,
    pub installation_binding: LicenseInstallationBinding,
    pub entitlements: LicenseEntitlements,
    pub issued_at: Timestamp,
    pub not_before: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub issuer_key_id: LicenseIssuerKeyId,
}
```

---

# 42. LicenseId

```rust
pub struct LicenseId(Ulid);
```

---

# 43. CustomerCommercialId

Commercial identity, not auth principal.

---

# 44. Installation Binding

Options:

```rust
pub enum LicenseInstallationBinding {
    AnyInstallation,
    InstallationId(InstallationId),
    DeploymentFingerprint(LicenseFingerprint),
}
```

---

# 45. Recommended

Prefer stable Forgeyard `InstallationId`.

---

# 46. Hardware Fingerprinting

Avoid as primary mechanism.

---

# 47. Why Avoid Hardware Lock

VM migrations/hardware replacement/DR can break it.

---

# 48. Installation Binding

Logical installation identity is more recoverable.

---

# 49. License Signature

Detached or envelope signature.

---

# 50. License Issuer Root

Pinned public key distributed with Forgeyard.

---

# 51. Offline Verification

No network required.

---

# 52. License Verification Flow

```text
load document
  ↓
canonical decode
  ↓
verify issuer signature
  ↓
verify time window
  ↓
verify installation binding
  ↓
verify revocation state if locally available
  ↓
derive entitlement set
```

---

# 53. Time

Clock rollback can affect expiry.

---

# 54. Clock Defense

Record monotonic last-known valid wall time in protected local metadata.

---

# 55. No Perfect Offline Clock Security

State honestly.

---

# 56. High-Assurance License

Can use periodic signed renewal tokens if business requires, but avoid frequent online heartbeat.

---

# 57. License Refresh

Optional:

```text
30-90 day signed refresh lease
```

for commercial revocation enforcement.

---

# 58. Fully Offline Enterprise

Can receive long-lived license file manually.

---

# 59. Air-Gapped Renewal

Import signed license file via removable media.

---

# 60. License Revocation

Online hosted/self-hosted-connected installations can receive signed revocation list.

---

# 61. RevocationList

```rust
pub struct LicenseRevocationList {
    pub version: u64,
    pub revoked: Vec<LicenseId>,
    pub issued_at: Timestamp,
    pub signature: SignatureRef,
}
```

---

# 62. Offline Revocation

Only visible when updated list imported.

---

# 63. Capability Honesty

Do not claim immediate revocation for fully offline installations.

---

# 64. Issuer Key Rotation

```text
old trusted key
  ↓ signs
new issuer key
  ↓ overlap
```

---

# 65. Historical License Verification

Keep old public issuer keys.

---

# 66. License File Storage

Metadata DB/local secure config.

---

# 67. License File Is Not Secret

But should be integrity protected.

---

# 68. Customer Data in License

Minimize.

---

# 69. Subscription Model

Hosted service uses:

```rust
pub struct Subscription {
    pub id: SubscriptionId,
    pub tenant: TenantId,
    pub provider: BillingProviderId,
    pub provider_customer_ref: ProviderCustomerRef,
    pub provider_subscription_ref: ProviderSubscriptionRef,
    pub state: SubscriptionState,
    pub plan: CommercialPlanVersionRef,
}
```

---

# 70. Subscription State

```rust
pub enum SubscriptionState {
    Trialing,
    Active,
    PastDue,
    Grace,
    Paused,
    Cancelled,
    Expired,
    Unknown,
}
```

---

# 71. Provider State vs Forgeyard State

Normalize external provider state.

---

# 72. Never Store Raw Provider Enum in Core

Provider adapter maps.

---

# 73. Unknown

First-class.

---

# 74. Billing Provider Outage

Existing last-known entitlements remain valid within bounded freshness/grace.

---

# 75. Do Not Disable Product Immediately Because Billing API Is Down

Critical.

---

# 76. Billing Snapshot

```rust
pub struct BillingStateSnapshot {
    pub subscription: SubscriptionId,
    pub state: SubscriptionState,
    pub observed_at: Timestamp,
    pub provider_version: Option<BoundedString>,
}
```

---

# 77. Freshness

Entitlement policy defines maximum stale billing-state window.

---

# 78. Billing Reconciliation

Periodic.

---

# 79. Webhook

Provider webhook is fast path.

---

# 80. Webhook Flow

```text
verify signature
dedup
persist
normalize
update subscription snapshot
recompute entitlements
```

---

# 81. Missed Webhook

Reconcile provider API.

---

# 82. Payment Failure

Provider state becomes PastDue.

---

# 83. Forgeyard Mapping

PastDue -> Grace according to commercial policy.

---

# 84. Grace Expiry

Grace -> ReadOnly/Suspended.

---

# 85. Payment Recovery

Active -> restore entitlement.

---

# 86. No Data Loss

Commercial lifecycle does not delete customer artifacts automatically.

Retention remains separate.

---

# 87. Cancellation

Can mean:

```text
cancel at period end
immediate cancellation
```

normalized.

---

# 88. Effective End

Persist exact `service_until`.

---

# 89. Billing Provider Adapter

```rust
#[async_trait]
pub trait BillingProvider {
    async fn get_subscription(
        &self,
        reference: &ProviderSubscriptionRef,
    ) -> Result<ProviderSubscriptionState, BillingProviderError>;

    async fn report_usage(
        &self,
        report: UsageReport,
    ) -> Result<UsageReportResult, BillingProviderError>;
}
```

---

# 90. Provider Adapter Boundary

Provider-specific APIs stay adapter-local.

---

# 91. Payment Cards

Forgeyard should not handle card details directly if avoidable.

---

# 92. Hosted Checkout

Use provider-hosted checkout/customer portal.

---

# 93. PCI Reduction

Good architectural goal.

---

# 94. Customer Portal

External billing provider.

---

# 95. Forgeyard UI

Shows safe subscription summary/link.

---

# 96. Usage Export

Part 27 generates resource truth.

---

# 97. Billing Usage

Commercial subsystem transforms usage into billable dimensions.

---

# 98. Raw Usage vs Billable Usage

Separate.

---

# 99. Usage Pricing Model

Do not modify raw usage.

---

# 100. BillingDimension

```rust
pub enum BillingDimension {
    SeatMonth,
    RunnerMinute,
    CpuMinute,
    DeviceMinute,
    StorageGbMonth,
    RbeExecution,
    Custom(BillingDimensionId),
}
```

---

# 101. Usage Report

```rust
pub struct UsageReport {
    pub tenant: TenantId,
    pub period: BillingPeriod,
    pub dimensions: Vec<BillingUsageLine>,
    pub source_digest: Digest,
}
```

---

# 102. Idempotency

Usage reporting must be idempotent.

---

# 103. Provider Usage Key

Tenant + billing period + dimension + report version.

---

# 104. Corrections

Emit adjustment report, not rewrite external history silently.

---

# 105. Usage Reconciliation

Compare provider accepted reports vs Forgeyard billing export ledger.

---

# 106. Seat Entitlement

```rust
pub struct SeatLimit(u32);
```

---

# 107. Seat Definition

Must be explicit.

Recommended:

```text
active human principal assigned to tenant
```

---

# 108. Service Accounts

Do not count as human seats unless plan says.

---

# 109. Guest/External Reviewer

Commercial plan decides.

---

# 110. Seat Allocation

```rust
pub struct SeatAssignment {
    pub tenant: TenantId,
    pub principal: PrincipalId,
    pub assigned_at: Timestamp,
}
```

---

# 111. Seat Overflow

New user assignment blocked or tenant enters overage/grace.

---

# 112. Existing Users

Do not abruptly invalidate active sessions solely because temporary seat count exceeds by one; use explicit policy.

---

# 113. Seat Reconcile

Count active assignments.

---

# 114. SCIM

Provisioning attempts subject to seat entitlement.

---

# 115. SCIM Deprovision

Frees seat.

---

# 116. Runner Limit

Commercial ceiling on registered/active runners.

---

# 117. Runner Commercial Limit

Separate from technical scheduler capacity.

---

# 118. Example

```text
entitled active runners: 20
physical registered: 25
```

Policy decides which 20 can be active.

---

# 119. Prefer Explicit Active Assignment

Do not randomly disable.

---

# 120. Device Lab Limit

Feature entitlement + device/concurrency limits.

---

# 121. HA Entitlement

Feature flag.

---

# 122. HA Expiration Safety

Do not deliberately break an already-running cluster by killing voters.

---

# 123. Recommended Behavior

If HA entitlement lapses:

```text
cluster continues safely
new HA-specific management/config expansion blocked
commercial warning/read-only policy
```

---

# 124. Never Violate Consensus Safety for Licensing

Critical.

---

# 125. RBE Entitlement

Controls public RBE service availability.

---

# 126. Existing RBE Jobs

Allowed to finish if already admitted.

---

# 127. Plugin Entitlement

External plugin support may be feature-gated.

---

# 128. Audit WORM Entitlement

Commercial feature may enable advanced exporter, but baseline audit correctness remains.

---

# 129. Security Baseline Must Not Be Paywalled

Critical principle.

---

# 130. Do Not Entitle Away Core Security

Features such as:

```text
tenant isolation
authz enforcement
secret redaction
TLS
audit of critical actions
```

must not be disabled because plan is lower.

---

# 131. Commercial Feature Gate

Only optional/advanced capabilities.

---

# 132. FeatureGate Service

```rust
pub trait FeatureGate {
    fn evaluate(
        &self,
        scope: EntitlementScope,
        feature: FeatureId,
    ) -> FeatureGateDecision;
}
```

---

# 133. Decision

```rust
pub enum FeatureGateDecision {
    Allowed,
    AllowedGracefully(GraceContext),
    Denied(EntitlementViolation),
}
```

---

# 134. Feature Gate Inputs

Current effective entitlement snapshot.

---

# 135. No Network Call in Hot Path

Critical.

---

# 136. Billing Provider Not Called Per Request

Entitlements cached/persisted locally.

---

# 137. Deterministic Entitlement Snapshot

```rust
pub struct EffectiveEntitlements {
    pub scope: EntitlementScope,
    pub version: EntitlementVersion,
    pub state: EntitlementState,
    pub features: BTreeMap<FeatureId, EffectiveFeature>,
    pub limits: BTreeMap<CommercialLimitKind, CommercialLimit>,
}
```

---

# 138. Entitlement Evaluation

Pure/local.

---

# 139. Hosted Update

Webhook/reconciler refreshes snapshot.

---

# 140. Self-Hosted Update

License import refreshes snapshot.

---

# 141. Entitlement Cache

Memory optimization.

Persistent snapshot remains source.

---

# 142. Commercial Limit

Examples:

```rust
pub enum CommercialLimitKind {
    HumanSeats,
    ActiveRunners,
    ManagedProjects,
    DevicePoolSize,
    RetentionDays,
    RbeConcurrentExecutions,
    Custom(CommercialLimitKindId),
}
```

---

# 143. Commercial Limit vs Resource Quota

Commercial ceiling can feed Part 27 quota.

---

# 144. Example

```text
plan runner ceiling = 20
tenant admin quota = 15
effective technical quota = min(20, 15)
```

---

# 145. Effective Limit Composition

```text
commercial entitlement
+
resource governance
+
system capacity
+
policy
```

---

# 146. No Duplicate Quota Engine

Entitlement exports ceilings to governance.

Part 27 enforces actual resource quotas.

---

# 147. Entitlement Reconciliation

Checks:

```text
license expiry
billing state freshness
seat count
runner count
plan version
revocation list
```

---

# 148. Durable Timers

For:

```text
trial expiry
grace expiry
license expiry
renewal reminders
```

---

# 149. Restart Safe

Part 10 timer semantics.

---

# 150. Notification Integration

Notify:

```text
trial ending
license expiring
payment past due
grace ending
seat limit near
runner limit reached
```

---

# 151. Security

Avoid putting payment details in generic notifications.

---

# 152. Audit

Audit:

```text
license import
license renewal
license revoke
plan change
administrative entitlement grant
grace override
tenant commercial suspension
```

---

# 153. Billing Webhook Audit

Record normalized state transition, not sensitive raw payload.

---

# 154. Entitlement Grant

```rust
pub struct EntitlementGrant {
    pub id: EntitlementGrantId,
    pub scope: EntitlementScope,
    pub features: Vec<FeatureGrant>,
    pub expires_at: Option<Timestamp>,
    pub reason: BoundedString,
}
```

---

# 155. Admin Grant

For:

```text
support
evaluation
contract exception
incident workaround
```

---

# 156. Grant Permission

Highly restricted.

---

# 157. Grant Expiry

Prefer required for temporary exception.

---

# 158. Grant Audit

Mandatory.

---

# 159. Grant Precedence

Explicit deterministic rules.

Example:

```text
revocation
> security/system prohibition
> active signed/admin entitlement
> plan default
```

---

# 160. Commercial Grant Cannot Bypass Security Policy

Critical.

---

# 161. Plan Downgrade

Need deterministic handling.

---

# 162. Existing Data

Never delete immediately.

---

# 163. Feature Over-Limit

Example:

```text
50 projects exist
new plan allows 20
```

---

# 164. Recommended

Grandfather existing objects read-only/manageable; block creation of additional objects.

---

# 165. Explicit OverLimit State

```rust
pub enum CommercialLimitState {
    WithinLimit,
    AtLimit,
    OverLimit,
}
```

---

# 166. Remediation

User can:

```text
upgrade plan
delete/archive resources
request grant
```

---

# 167. No Random Deletion

Critical.

---

# 168. Retention Downgrade

Do not immediately purge old artifacts solely due plan change.

Apply future retention policy after clear notice/grace.

---

# 169. Hosted Suspension

Commercial suspension state separate from security suspension.

---

# 170. Tenant Access State Composition

```text
SecurityState
+
CommercialState
+
OperationalState
```

---

# 171. Security Suspension Wins

Commercial payment cannot reactivate security-suspended tenant.

---

# 172. Operational Maintenance Wins

Subscription active does not bypass maintenance.

---

# 173. Commercial ReadOnly

Can produce UI banner.

---

# 174. API Error

Stable codes:

```text
ENTITLEMENT_REQUIRED
LICENSE_EXPIRED
SUBSCRIPTION_PAST_DUE
SEAT_LIMIT_REACHED
FEATURE_NOT_ENTITLED
COMMERCIAL_LIMIT_REACHED
```

---

# 175. HTTP Status

Usually 402/403/409 depending client/API convention.

---

# 176. Recommendation

Use stable error code as primary semantic signal.

---

# 177. 402 Payment Required

Can be used for hosted commercial blocking, but not every entitlement denial.

---

# 178. CLI

```text
forgeyard license status
forgeyard license verify
forgeyard license install
forgeyard license inspect
forgeyard entitlement list
forgeyard entitlement explain
forgeyard subscription status
```

---

# 179. License Status

Shows:

```text
LicenseId
issuer
validity
features
limits
state
```

---

# 180. No Private Commercial Secret

Safe.

---

# 181. License Verify

Offline.

---

# 182. License Install

Admin.

---

# 183. License Remove

Dangerous; may transition to community/free/read-only.

---

# 184. Entitlement Explain

Shows why feature allowed/denied.

---

# 185. UI

Admin/commercial page:

```text
Plan
Entitlements
License
Subscription
Usage
Seats
Limits
Billing
```

---

# 186. Self-Hosted UI

License-focused.

---

# 187. Hosted UI

Subscription/customer-portal focused.

---

# 188. Feature Badge

UI can show:

```text
Available
Trial
Requires upgrade
```

---

# 189. UI Is Not Authority

Server still checks entitlement.

---

# 190. Upsell UX

Optional presentation concern.

---

# 191. Never Hide Customer Data Behind Upgrade Modal

History/export should remain accessible according to lifecycle policy.

---

# 192. Subscription Portal Link

Safe configured provider URL.

---

# 193. Billing Provider Credentials

SecretRef.

---

# 194. Webhook Signature Secret

SecretRef.

---

# 195. Provider Customer ID

Metadata, not secret.

---

# 196. Webhook Raw Payload

Short protected retention if needed.

---

# 197. Webhook Dedup

Provider event/delivery ID.

---

# 198. Provider Retry

At-least-once.

---

# 199. Unknown Billing State

Do not instantly suspend.

---

# 200. Stale Billing State Policy

Example:

```text
last verified Active
billing provider unavailable
  ↓
continue for 72h
  ↓
warn
```

---

# 201. Hosted Fraud/Chargeback

Commercial system can revoke/grace based on provider/ops policy.

---

# 202. Security Fraud

Separate security subsystem.

---

# 203. Self-Hosted License Telemetry

Optional.

---

# 204. Privacy

License verification should not require telemetry.

---

# 205. Phone-Home

Not baseline requirement.

---

# 206. Optional License Check-In

Enterprise contract may enable periodic signed lease renewal.

---

# 207. Check-In Data

Minimize:

```text
LicenseId
InstallationId
Forgeyard version
```

only as needed.

---

# 208. No Source/Project Data

Critical privacy principle.

---

# 209. Offline Grace

Must support network outage.

---

# 210. License Server Outage

Existing signed lease remains valid until expiry/grace.

---

# 211. License Server Compromise

Issuer key rotation/revocation.

---

# 212. Signature Algorithm

Use modern well-supported asymmetric signature scheme.

---

# 213. Canonical Payload

Stable/versioned.

---

# 214. No Homemade Crypto

Critical.

---

# 215. Root Public Key

Embedded and/or supplied in trust store.

---

# 216. Enterprise Custom Issuer

Potential OEM/private distribution later.

---

# 217. Not Baseline

Keep issuer abstraction.

---

# 218. License Schema Version

```rust
pub struct LicenseSchemaVersion(u16);
```

---

# 219. Backward Compatibility

New Forgeyard reads prior supported schema versions.

---

# 220. Forward Unknown Fields

Safe handling.

---

# 221. Unknown Feature

Older Forgeyard ignores only if not marked required.

---

# 222. Required Feature Semantic

License can declare minimum binary version.

---

# 223. Minimum Forgeyard Version

```text
min_version
max_version optional
```

---

# 224. License Upgrade

New binary verifies compatibility.

---

# 225. Downgrade

Older binary may not understand entitlement schema.

Fail safe.

---

# 226. License DR

License document included in configuration backup.

---

# 227. Issuer Public Keys

Included in release/trust metadata.

---

# 228. Air-Gap Recovery

License file restored/imported manually.

---

# 229. InstallationId DR

Must be recoverable.

---

# 230. Installation Clone

If backup accidentally cloned into second live installation, same license might be reused.

---

# 231. Commercial Policy

Could permit active/passive DR clone.

---

# 232. DR License Mode

Explicit:

```rust
pub enum LicenseDeploymentMode {
    SingleActive,
    ActivePassiveDr,
    MultipleInstallations(u32),
}
```

---

# 233. Do Not Break DR

License terms should model DR.

---

# 234. HA Nodes

Do not count each daemon as separate installation.

---

# 235. Cluster

One InstallationId.

---

# 236. Runner Count

Separate limit.

---

# 237. Hosted Multi-Tenant

Entitlements usually tenant-scoped.

---

# 238. Self-Hosted Enterprise

Entitlements usually installation-wide with optional tenant sub-allocation.

---

# 239. Sub-Allocation

Enterprise installation admin may assign purchased capacity across tenants.

---

# 240. Parent Ceiling

Cannot exceed license.

---

# 241. Commercial Reservation

Can feed governance reservations.

---

# 242. Seat Usage Meter

Derived from identity membership state.

---

# 243. Runner Usage Meter

Derived from active runner registration/state.

---

# 244. Storage Usage

Part 27 logical accounting.

---

# 245. Usage Billing Period

Explicit timezone/calendar.

---

# 246. Period Closure

Billable usage snapshot sealed.

---

# 247. BillingLedgerEntry

Not financial double-entry accounting; just export ledger.

---

# 248. UsageExportRecord

```rust
pub struct UsageExportRecord {
    pub id: UsageExportId,
    pub tenant: TenantId,
    pub period: BillingPeriod,
    pub source_digest: Digest,
    pub state: UsageExportState,
}
```

---

# 249. Export State

```text
Pending
Submitted
Accepted
Failed
Unknown
```

---

# 250. Unknown Provider Result

Inspect/reconcile before duplicate submission when provider supports.

---

# 251. Corrections

Versioned adjustment.

---

# 252. Invoice State

Forgeyard may display:

```text
Open
Paid
PastDue
Void
Unknown
```

from provider, but does not become accounting authority.

---

# 253. Billing Provider Is Accounting Authority for Hosted Payment State

Forgeyard retains normalized subscription snapshot.

---

# 254. Customer Portal

Preferred for invoice/payment method changes.

---

# 255. No Card Number Storage

Critical.

---

# 256. Tax IDs

Keep at billing provider if possible.

---

# 257. Entitlement Health

```text
license validity
billing freshness
reconcile lag
issuer key validity
usage export lag
```

---

# 258. Doctor

```text
forgeyard entitlement doctor
```

---

# 259. Doctor Checks

Self-hosted:

```text
license signature
expiry
installation binding
issuer trust
```

Hosted:

```text
billing webhook
provider API
subscription freshness
usage export
```

---

# 260. Metrics

```text
entitlement_denied_total
entitlement_grace_scopes
license_expiry_seconds
billing_reconcile_lag_seconds
usage_export_failures_total
seat_limit_utilization
commercial_limit_denied_total
```

---

# 261. Metric Labels

Low-cardinality:

```text
feature_class
state
limit_kind
provider_type
```

---

# 262. No Customer/Tenant ID Labels

Use admin query store.

---

# 263. Tracing

```text
entitlement.evaluate
license.verify
subscription.reconcile
billing.webhook
usage.export
```

---

# 264. Audit Integration

Commercial state transitions recorded.

---

# 265. Notification Integration

Expirations/past-due/limits.

---

# 266. Policy Integration

Policy can require entitlement plus operational conditions.

---

# 267. Example

```text
Feature HA entitled?
  +
cluster policy permits?
  ↓
allow config
```

---

# 268. Feature Gate Placement

At service/application boundary.

---

# 269. Not Only UI

Critical.

---

# 270. Not Inside Low-Level Core Types

Keep commercial concerns out of primitive domain logic.

---

# 271. High-Risk Feature Gate

Explicit at command admission.

---

# 272. Example Release Feature

If advanced release channels are commercial:

```text
create advanced channel
  ↓
entitlement check
  ↓
authz
  ↓
policy
```

---

# 273. Order

Recommended:

```text
authn
resource scope
authz
entitlement
policy
quota/capacity
domain action
```

---

# 274. Why Authz Before Entitlement

Avoid leaking feature/customer info to unauthorized user.

---

# 275. Entitlement Error Concealment

Unauthorized users should still see 403/404 rather than plan details.

---

# 276. Testkit

```text
forgeyard-entitlement-testkit/src/
├── lib.rs
├── plan.rs
├── license.rs
├── verifier.rs
├── subscription.rs
├── billing.rs
├── seat.rs
├── feature_gate.rs
└── assertions.rs
```

---

# 277. Unit Tests

Plan -> entitlement derivation.

---

# 278. Signature Test

Valid signed license accepted.

---

# 279. Tampered License Test

Rejected.

---

# 280. Wrong Installation Test

Rejected.

---

# 281. Expired License Test

Transitions correctly.

---

# 282. Grace Test

Allowed operations match policy.

---

# 283. Read-Only Test

Historical/export access preserved.

---

# 284. Security Test

Expired entitlement does not bypass security/isolation.

---

# 285. HA Safety Test

License expiry does not tear down consensus quorum.

---

# 286. Existing Job Test

Entitlement expires while job running; admitted job can finish according to policy.

---

# 287. New Job Test

New disallowed work rejected.

---

# 288. Seat Limit Test

SCIM/user assignment cannot exceed hard commercial seat ceiling.

---

# 289. Runner Limit Test

Extra runner cannot become active if limit reached.

---

# 290. Plan Downgrade Test

Existing over-limit resources preserved; creation blocked.

---

# 291. Billing Webhook Duplicate Test

Idempotent.

---

# 292. Billing Missed Webhook Test

Reconcile fixes.

---

# 293. Billing Outage Test

Last-known active subscription remains in bounded freshness/grace.

---

# 294. Usage Export Duplicate Test

No double reporting.

---

# 295. Usage Adjustment Test

Correction is explicit.

---

# 296. Provider Unknown Test

No blind duplicate usage submission.

---

# 297. Air-Gap Test

License verifies without internet.

---

# 298. DR Test

Restored installation keeps valid InstallationId/license semantics.

---

# 299. Issuer Rotation Test

Old/new licenses verify during overlap.

---

# 300. Revocation Test

Imported signed revocation list blocks revoked license.

---

# 301. Clock Rollback Test

Detected/bounded.

---

# 302. Feature-Gate Bypass Test

CLI/API/RBE/plugin path cannot use disabled commercial feature.

---

# 303. UI-Only Gate Test

Ensure hidden UI is not sole enforcement.

---

# 304. Fuzzing

Fuzz license parser/canonicalization/provider webhook normalization.

---

# 305. Failure Injection

```text
billing API outage
license file unreadable
issuer key unavailable
DB restart
webhook loss
usage export timeout
```

---

# 306. Implementation Phase 1 — Feature Entitlement Model

Feature IDs, limits, effective snapshot.

---

# 307. Phase 2 — Self-Hosted Signed License

Offline verification.

---

# 308. Phase 3 — FeatureGate Integration

Service admission.

---

# 309. Phase 4 — Seats/Runner Commercial Limits

Enterprise controls.

---

# 310. Phase 5 — Trial/Grace/Read-Only Lifecycle

Safe degradation.

---

# 311. Phase 6 — Hosted Subscription Adapter

Provider-neutral.

---

# 312. Phase 7 — Billing Webhooks/Reconciliation

Durable normalized state.

---

# 313. Phase 8 — Usage Export

Part 27 integration.

---

# 314. Phase 9 — UI/CLI/Notifications

Commercial operations.

---

# 315. Phase 10 — License Revocation/Issuer Rotation

High assurance.

---

# 316. Phase 11 — Air-Gap/DR

Enterprise hardening.

---

# 317. Phase 12 — Scale/Security/Chaos

Production validation.

---

# 318. Acceptance Tests

1. Commercial entitlement never replaces authn/authz/policy.
2. Domain code does not branch directly on plan names.
3. Feature checks use stable FeatureId.
4. Hosted subscription and self-hosted license produce the same effective entitlement model.
5. Self-hosted signed license verifies offline.
6. License tampering invalidates signature.
7. Installation binding uses stable InstallationId rather than fragile hardware fingerprint by default.
8. Fully offline installation does not require continuous phone-home.
9. Billing provider outage does not instantly disable active tenants.
10. Last-known subscription state has explicit freshness/grace.
11. Billing webhook delivery is verified/deduplicated.
12. Missed billing webhooks are repaired by reconciliation.
13. Payment failure maps through explicit PastDue/Grace/ReadOnly lifecycle.
14. Commercial expiration does not delete customer data.
15. Historical data/export/backup remain accessible according to lifecycle policy.
16. Existing admitted jobs are not abruptly killed merely because entitlement expires.
17. New disallowed work is blocked at server admission.
18. UI hiding is never the sole feature gate.
19. HA license expiration never intentionally violates Raft quorum safety.
20. Commercial quotas feed Part 27 rather than creating a duplicate quota engine.
21. Resource truth remains independent from billing pricing.
22. Usage export is idempotent.
23. Usage corrections are explicit adjustments.
24. Forgeyard does not store payment-card details.
25. Seat semantics are explicit.
26. Runner limits are explicit and deterministic.
27. Plan downgrade never randomly deletes over-limit resources.
28. Admin entitlement grants are scoped, expiring where practical, and audited.
29. License issuer key rotation preserves historical verification.
30. Revocation semantics are honest for offline installations.
31. Air-gapped renewal/import works.
32. DR restore preserves installation/license semantics.
33. Security baseline controls are not disabled because of lower commercial plan.
34. Alternate paths—API, RBE, plugins, device lab—cannot bypass feature entitlements.
35. Forgeyard can operate community/local, self-hosted enterprise, and hosted SaaS through the same entitlement model.

---

# 319. Production Readiness Gates

Do not call entitlement/licensing production-ready until:

```text
FeatureId-based gating replaces plan-name checks
signed offline license verification is stable
trial/grace/read-only lifecycle is tested
server-side feature enforcement covers all relevant entry points
seat/runner limit behavior is deterministic
billing webhook/reconciliation is resilient
provider outages do not brick service
usage export is idempotent
issuer key rotation/revocation is tested
DR/air-gap license recovery works
```

---

# 320. Architectural Invariants

1. entitlement is commercial capability, not security authority;
2. authz always remains mandatory;
3. plan names never drive domain logic directly;
4. stable FeatureId drives feature gates;
5. effective entitlements are deterministic/versioned;
6. billing providers are adapters, not runtime hot-path dependencies;
7. self-hosted licenses verify offline;
8. license signatures use standard asymmetric cryptography;
9. hardware fingerprinting is not the default binding;
10. fully offline revocation limitations are stated honestly;
11. grace/read-only states are explicit;
12. expiration never causes automatic data deletion;
13. existing valid work is not normally killed on entitlement expiry;
14. new disallowed work is rejected server-side;
15. HA safety is never compromised for licensing enforcement;
16. security baseline protections are not commercial gates;
17. commercial ceilings compose with Part 27 quotas;
18. raw usage remains separate from billable/priced usage;
19. usage export is idempotent/reconcilable;
20. payment-card data stays outside Forgeyard where possible;
21. plan downgrade preserves existing over-limit resources safely;
22. administrative grants are explicit/audited;
23. entitlement changes notify/audit appropriately;
24. issuer keys rotate with historical verification;
25. air-gapped licensing is supported;
26. DR retains installation identity;
27. UI feature hiding is presentation only;
28. alternate protocols cannot bypass entitlement checks;
29. standalone/self-hosted/hosted share the same entitlement semantics;
30. Forgeyard dogfoods its own entitlement architecture without making billing part of execution correctness.

---

# 321. Final Target Architecture

```text
                    Commercial Source
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       Billing        License File      Trial/Admin
          │              │              │
          └──────────────┼──────────────┘
                         ▼
                 Entitlement Snapshot
                         │
               ┌─────────┼─────────┐
               ▼         ▼         ▼
            Features    Limits    State
               │         │         │
               └─────────┼─────────┘
                         ▼
                  Feature Admission
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
          Authz         Policy        Quota
           │             │             │
           └─────────────┼─────────────┘
                         ▼
                    Domain Action
```

---

# 322. Final Architectural Position

Self-hosted license:

```text
signed license document
  ↓
offline signature verification
  ↓
installation binding
  ↓
validity/revocation check
  ↓
effective entitlement snapshot
  ↓
feature admission
```

Hosted subscription:

```text
billing webhook/poll
  ↓
normalized subscription state
  ↓
durable local snapshot
  ↓
trial/active/grace/read-only lifecycle
  ↓
effective entitlement snapshot
```

Usage billing:

```text
Part 27 authoritative usage
  ↓
billable transformation
  ↓
idempotent usage export
  ↓
billing provider
```

The key guarantee is:

> **Forgeyard can support free/local, hosted, and enterprise-commercial editions without mixing money state into execution correctness or security. Entitlements decide which optional capabilities a customer has purchased; identity, policy, tenant isolation, scheduling correctness, audit, and data safety remain authoritative regardless of commercial state.**

---

# 323. Extended Architecture Sequence

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
```
