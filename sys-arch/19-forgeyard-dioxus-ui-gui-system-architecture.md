# 19 — Forgeyard Dioxus UI / GUI System Architecture

**Document type:** Core Cross-Platform User Interface & Experience System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Dioxus desktop/mobile/web UI, application shell, navigation, route architecture, client state, query/cache layer, live events, logs, dashboards, release/deployment workflows, admin/security surfaces, offline/degraded behavior, accessibility, performance, theming, testing, and UI security boundaries  
**Architecture style:** Thin client over the Axum public API, capability-driven UX, reactive typed state, durable-server-state-first, streaming updates, platform-adaptive composition, and strict non-authoritative UI semantics  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on `18 — API / Axum`, `17 — Observability / Health / Doctor`, Run/Job, Scheduler, Runner, Artifacts/CAS, Change Proposal, Policy/Authz/Identity, Secrets & Trust, Supply Chain, Packaging, Release, Deployment, Device Lab, and later SCM provider integrations. It is the human-facing control/visibility plane and never replaces server-side authority.

---

# 1. Purpose

Forgeyard needs a polished UI that makes a technically deep CI/CD system understandable without hiding critical system truth.

The interface must help users answer:

```text
what is running?
why is this job waiting?
where is this artifact?
what failed?
which runner executed this?
is this release ready?
what evidence is missing?
what is deployed where?
is production healthy?
what permissions do I have?
why is this action blocked?
```

The central rule is:

> **The Dioxus UI is a client and visualization layer. It never becomes authoritative for job state, policy, release, deployment, identity, or security decisions.**

A second rule is:

> **The same domain/API semantics should power desktop, mobile, and web UI, while layout and interaction patterns adapt per platform.**

A third rule is:

> **The UI should expose system state honestly—including Unknown, Degraded, Partial, Stale, and Reconciliation states—instead of flattening them into false success/failure simplicity.**

---

# 2. Architectural Position

```text
                     Forgeyard User
                           │
                           ▼
                       Dioxus UI
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
      REST/JSON          SSE/WS          Local UX
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                       Axum API
                           │
                           ▼
                    Domain Services
                           │
                           ▼
                   Authoritative State
```

---

# 3. Goals

The UI subsystem MUST:

1. use Dioxus;
2. support desktop;
3. support mobile;
4. optionally support web if desired by product mode;
5. share as much UI/domain client code as practical;
6. adapt layouts per form factor;
7. use the public API;
8. use SSE/WS for live updates;
9. support reconnect/backfill;
10. support role/policy-aware controls;
11. show authorization explanations;
12. show run/job state clearly;
13. show queue/scheduler diagnostics;
14. show runner health/capabilities;
15. show logs efficiently;
16. show artifacts/evidence;
17. show Change Proposal state;
18. show release candidate/readiness;
19. show deployment rollout/health;
20. show system health;
21. support admin/security pages;
22. support offline/degraded viewing where possible;
23. avoid storing secrets;
24. support accessibility;
25. support keyboard navigation;
26. support responsive density;
27. support large data sets;
28. support resilient error/loading states;
29. support testing;
30. remain visually consistent.

---

# 4. Non-Goals

The UI does not:

```text
authorize actions
validate release policy authoritatively
choose runner placement
execute jobs
store secret values
replace provider/backend state
```

---

# 5. Workspace Structure

```text
apps/forgeyard-ui/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── routes.rs
│   ├── bootstrap.rs
│   ├── platform.rs
│   └── error_boundary.rs
```

UI crates:

```text
crates/ui/
├── forgeyard-ui-core/
├── forgeyard-ui-model/
├── forgeyard-ui-router/
├── forgeyard-ui-api/
├── forgeyard-ui-query/
├── forgeyard-ui-state/
├── forgeyard-ui-live/
├── forgeyard-ui-components/
├── forgeyard-ui-layout/
├── forgeyard-ui-theme/
├── forgeyard-ui-icons/
├── forgeyard-ui-forms/
├── forgeyard-ui-tables/
├── forgeyard-ui-logs/
├── forgeyard-ui-charts/
├── forgeyard-ui-run/
├── forgeyard-ui-job/
├── forgeyard-ui-runner/
├── forgeyard-ui-artifact/
├── forgeyard-ui-change/
├── forgeyard-ui-release/
├── forgeyard-ui-deployment/
├── forgeyard-ui-health/
├── forgeyard-ui-security/
├── forgeyard-ui-admin/
├── forgeyard-ui-mobile/
├── forgeyard-ui-desktop/
├── forgeyard-ui-web/
└── forgeyard-ui-testkit/
```

---

# 6. UI Dependency Direction

```text
ui-model
  ↓
ui-api/query/state
  ↓
ui-components/layout
  ↓
feature UI crates
  ↓
platform composition
  ↓
apps/forgeyard-ui
```

Feature UI crates must not call SQL/store/domain adapters directly.

---

# 7. API Client Boundary

```rust
#[async_trait]
pub trait ForgeyardApiClient {
    async fn get_run(&self, id: RunId) -> UiResult<RunDto>;
    async fn cancel_run(&self, id: RunId) -> UiResult<RunDto>;
    async fn get_release(&self, id: ReleaseId) -> UiResult<ReleaseDto>;
    // ...
}
```

Actual HTTP implementation lives in `forgeyard-ui-api`.

---

# 8. DTO Reuse

UI may consume `forgeyard-api-model`.

It should not depend on domain persistence types.

---

# 9. App Shell

The root UI contains:

```text
top-level navigation
workspace selector
project selector
global search
notifications
user/account menu
system health indicator
```

---

# 10. Main Navigation

Recommended:

```text
Overview
Projects
Changes
Runs
Runners
Artifacts
Packages
Releases
Deployments
Devices
Health
Admin
```

Visibility depends on permission/capability.

---

# 11. Mobile Navigation

Use bottom navigation for primary destinations.

Overflow/admin in drawer/sheet.

---

# 12. Desktop Navigation

Persistent side rail/sidebar.

---

# 13. Web Navigation

Responsive desktop/tablet/mobile.

---

# 14. Route Model

```rust
pub enum AppRoute {
    Overview,
    Projects,
    Project(ProjectId),
    Run(RunId),
    Job(JobId),
    Runner(RunnerId),
    Artifact(ArtifactId),
    Release(ReleaseId),
    Deployment(DeploymentId),
    Health,
    Admin,
}
```

---

# 15. Deep Links

Every major entity should have stable route.

---

# 16. Route Authorization

UI may hide inaccessible routes.

Server still authorizes.

---

# 17. Unknown/Removed Entity

Show safe Not Found / Access Denied state.

---

# 18. Global Context

```rust
pub struct UiContext {
    pub api: ApiClientHandle,
    pub session: SessionState,
    pub tenant: TenantContext,
    pub capabilities: PublicCapabilities,
}
```

---

# 19. State Categories

Separate:

```text
server state
view state
session state
ephemeral interaction state
```

---

# 20. Server State

Examples:

```text
runs
jobs
runners
releases
deployments
```

Fetched from API and cacheable.

---

# 21. View State

Examples:

```text
selected tab
sort order
expanded row
filter
```

---

# 22. Session State

```text
principal
tenant
permissions summary
session expiry
```

---

# 23. Ephemeral Interaction State

```text
dialog open
form draft
hover/focus
```

---

# 24. Query Cache

Typed cache keyed by API resource identity.

---

# 25. Query Key

```rust
pub enum QueryKey {
    Run(RunId),
    Job(JobId),
    Runner(RunnerId),
    Release(ReleaseId),
    Deployment(DeploymentId),
}
```

---

# 26. Cache Staleness

Each query defines TTL/freshness.

---

# 27. Live Update Integration

SSE event updates can invalidate/refetch relevant cache.

---

# 28. Prefer Refetch Over Complex Local Replay

For critical entity state, event says:

```text
entity changed
```

then query current authority.

---

# 29. Local Optimistic Updates

Use sparingly.

---

# 30. Good Optimistic Candidate

UI-only actions like:

```text
toggle local filter
```

---

# 31. Risky Optimistic Candidate

```text
release approved
deployment healthy
run cancelled
```

Do not pretend success before server confirms.

---

# 32. Loading State

Every async surface needs:

```text
initial loading
background refresh
partial content
```

---

# 33. Error State

Typed and actionable.

---

# 34. Retry State

Show retry for retryable errors.

---

# 35. Offline State

If API unreachable:

```text
show cached/read-only data
mark stale clearly
disable writes
```

---

# 36. No False Freshness

Cached data must display:

```text
last updated
offline/stale badge
```

---

# 37. Reconnect

UI retries API/SSE with backoff.

---

# 38. Session Expiry

Prompt reauthentication.

---

# 39. Live Event Layer

```rust
pub struct LiveEventClient {
    pub connection: LiveConnectionState,
    pub cursor: Option<EventCursor>,
}
```

---

# 40. Live Connection State

```rust
pub enum LiveConnectionState {
    Connecting,
    Connected,
    Reconnecting,
    Offline,
}
```

---

# 41. SSE Preference

Use SSE for most server-push timelines.

---

# 42. WebSocket

Reserve for bidirectional/interactive features if needed.

---

# 43. Run Page

Tabs:

```text
Overview
Jobs
Timeline
Logs
Artifacts
Metrics
Evidence
```

---

# 44. Run Overview

Shows:

```text
state
source snapshot
pipeline
actor
start/finish
duration
progress
```

---

# 45. Job Graph

Visual DAG.

---

# 46. DAG Rendering

Large graphs use virtualization/collapse.

---

# 47. Job State Colors

Do not rely on color alone.

Include icon/text.

---

# 48. State Vocabulary

Use backend canonical:

```text
Pending
Eligible
Leased
Preparing
Running
Uploading
Succeeded
Failed
Cancelled
TimedOut
Lost
Skipped
RetryWaiting
```

---

# 49. Retry Visualization

Show attempt timeline.

---

# 50. Job Page

Tabs:

```text
Overview
Attempts
Logs
Resources
Inputs
Outputs
Scheduler
Sandbox
```

---

# 51. Attempt Page

Show exact:

```text
AttemptId
LeaseId
Runner
AgentSession
Executor
Sandbox profile
exit reason
resource usage
```

---

# 52. Scheduler Explain UI

Show:

```text
eligible runners
hard filter failures
placement score
selected runner
```

---

# 53. Runner Page

Tabs:

```text
Overview
Capabilities
Active Jobs
History
Resources
Health
Doctor
```

---

# 54. Runner Capability UI

Group:

```text
platform
CPU
memory
GPU
toolchains
sandbox
devices
trust
```

---

# 55. Runner Trust UI

Read-only unless authorized admin.

---

# 56. Runner Drain

Confirmation.

---

# 57. Drain Modes

Explain:

```text
graceful
cancel-active
```

---

# 58. Log Viewer

Critical feature.

---

# 59. Log Requirements

```text
stream live
backfill
search
filter stdout/stderr/agent
follow tail
pause follow
download
copy selection
```

---

# 60. Log Virtualization

Must render huge logs efficiently.

---

# 61. Bounded DOM

Do not keep millions of lines mounted.

---

# 62. Binary/Invalid UTF-8

Render safely.

---

# 63. ANSI

Optional safe ANSI rendering.

---

# 64. Secret Redaction

Server/agent primary.

UI should not attempt to recover hidden content.

---

# 65. Log Truncation

Show explicit marker.

---

# 66. Artifacts Page

Shows:

```text
name
type
digest
size
producer
retention
evidence
```

---

# 67. Artifact Download

Request authorized download URL.

---

# 68. Artifact Evidence

Tabs:

```text
SBOM
Provenance
VEX
Vulnerabilities
Licenses
Signatures
Reproducibility
```

---

# 69. Evidence Graph

Visual lineage:

```text
Source → Build → Package → Signed → Release
```

---

# 70. Package Page

Show:

```text
format
target
inputs
manifest
lineage
validation
signing
```

---

# 71. Change Proposal Page

Tabs:

```text
Overview
Diff
Checks
Reviews
Approvals
Evidence
Integration
Timeline
```

---

# 72. Diff Viewer

Large diff virtualization.

---

# 73. Review Anchors

Outdated anchors visually distinct.

---

# 74. Approval UI

Shows exact proposal revision/snapshot.

---

# 75. Merge/Integrate Button

Only enabled if server reports actionable state.

---

# 76. Button Disable Reason

Explain:

```text
missing approval
failed check
target drift
permission
queue
```

---

# 77. Release Page

Tabs:

```text
Overview
Candidate
Packages
Evidence
Approvals
Publications
Channels
Notes
Timeline
Audit
```

---

# 78. Release Readiness

Summarize:

```text
targets complete
signatures valid
SBOM present
vulnerability policy
approvals
```

---

# 79. Release Candidate Digest

Always visible for high-risk approval.

---

# 80. Release Approval Dialog

Show:

```text
candidate digest
version
source snapshot
package count
policy requirements
```

---

# 81. Publication State

Use exact:

```text
Pending
InProgress
Succeeded
Failed
Unknown
```

---

# 82. Unknown Publication

Do not show red Failure only.

Explain:

```text
remote outcome uncertain; reconciliation in progress
```

---

# 83. Deployment Page

Tabs:

```text
Overview
Plan
Diff
Rollout
Health
Migrations
Drift
Approvals
History
Rollback
Audit
```

---

# 84. Rollout Visualization

For Canary:

```text
10% → 25% → 50% → 100%
```

---

# 85. Health Gate UI

Show:

```text
check
threshold
observed
window
status
```

---

# 86. Blue-Green UI

Visual old/new environment and traffic switch.

---

# 87. Rolling UI

Show current/desired replicas.

---

# 88. Migration UI

Show:

```text
compatibility
irreversibility
backup requirement
state
```

---

# 89. Rollback UI

Must explain rollback safety.

---

# 90. Drift UI

Show:

```text
desired
actual
difference
ownership
```

---

# 91. Drift Adoption

Requires explicit confirmation and permission.

---

# 92. Health Page

Sections:

```text
Overall
Daemon
Storage
CAS
Scheduler
Runners
Events
Reconciliation
Secrets
Trust
Release
Deployment
```

---

# 93. Health State

Use canonical:

```text
Healthy
Degraded
Unhealthy
Unknown
```

---

# 94. Readiness/Liveness

Admin view only where relevant.

---

# 95. Doctor UI

Run typed doctor checks.

---

# 96. Doctor Result

```text
Pass
Warn
Fail
Skipped
```

with remediation.

---

# 97. No Arbitrary Remote Shell

Doctor UI triggers typed checks only.

---

# 98. Admin UI

Areas:

```text
Users
Groups
Roles
Permissions
Policies
Secrets
Trust
Runners
System Config
Migrations
Events
Dead Letters
Reconciliation
```

---

# 99. Admin Route Permission

Server-authorized.

---

# 100. Policy UI

Show:

```text
source
effective policy
digest
diff
violations
```

---

# 101. Authz Explain UI

"Why can't I do this?"

---

# 102. Secret UI

Metadata only by default.

---

# 103. Secret Value

Never store in global UI state.

---

# 104. Secret Creation

Input cleared immediately after submission.

---

# 105. Secret Reveal

If supported:

```text
high-risk dialog
step-up auth
short display
no persistent cache
```

---

# 106. Trust UI

Show:

```text
trust roots
cert expiry
runner enrollment
revocations
```

---

# 107. Break-Glass UI

Explicit dangerous action surface.

---

# 108. Break-Glass UX

Require:

```text
reason
scope
expiry
MFA/step-up
```

---

# 109. Confirmation Levels

Use stronger confirmation for:

```text
production deployment
release promotion
runner trust change
secret reveal
break-glass
delete/yank
```

---

# 110. Avoid "Type DELETE" Everywhere

Reserve for destructive high-risk actions.

---

# 111. Forms

Typed form model.

---

# 112. Form State

```rust
pub enum FormState {
    Idle,
    Dirty,
    Submitting,
    Succeeded,
    Failed,
}
```

---

# 113. Client Validation

UX only.

Server validation authoritative.

---

# 114. Validation Errors

Map field-level API errors.

---

# 115. Unsaved Changes

Prompt on route leave.

---

# 116. Large Forms

Use sections and progressive disclosure.

---

# 117. Table Architecture

Virtualized large tables.

---

# 118. Column Sets

Responsive.

---

# 119. Desktop

Dense table.

---

# 120. Mobile

Cards/condensed rows.

---

# 121. Sorting/Filtering

Server-side for large lists.

---

# 122. Pagination

Cursor-based.

---

# 123. Infinite Scroll

Optional.

---

# 124. Explicit Pagination

Better for admin/audit.

---

# 125. Search

Global search can search:

```text
project
run
job
release
deployment
runner
```

via API.

---

# 126. Command Palette

Desktop/web:

```text
Ctrl/Cmd + K
```

---

# 127. Keyboard Navigation

Essential for power users.

---

# 128. Shortcuts

Examples:

```text
g r → runs
g d → deployments
/ → search
```

configurable later.

---

# 129. Accessibility

WCAG-aware.

---

# 130. Keyboard Focus

Visible.

---

# 131. Screen Reader Labels

All controls.

---

# 132. Color Contrast

Sufficient.

---

# 133. Status Icons

Text + icon + color.

---

# 134. Reduced Motion

Respect OS preference.

---

# 135. Animation

Subtle, non-blocking.

---

# 136. Responsive Layout

Breakpoints defined centrally.

---

# 137. Desktop Layout

```text
sidebar
header
main pane
optional detail pane
```

---

# 138. Tablet

Collapsible sidebar.

---

# 139. Mobile

Single-column primary flow.

---

# 140. Split View

Desktop can show:

```text
run list | run detail
```

---

# 141. Dense Mode

Optional for experienced operators.

---

# 142. Touch Targets

Mobile-friendly.

---

# 143. Safe Areas

Respect mobile notches/navigation bars.

---

# 144. Desktop Windowing

Persist:

```text
window size
position
last route
```

locally.

---

# 145. Multi-Window

Not required initially.

---

# 146. Native Notifications

Desktop/mobile can show:

```text
run completed
approval requested
deployment failed
```

subject to permission/preferences.

---

# 147. Notification Source

Server notification/event system.

---

# 148. Push Notifications

Mobile optional later.

---

# 149. Local Notifications

No sensitive payload in lock-screen text by default.

---

# 150. Theme

Support:

```text
system
light
dark
```

---

# 151. Design Tokens

Central:

```text
spacing
radius
typography
status semantics
elevation
```

---

# 152. No Per-Feature Random Styling

Use component system.

---

# 153. CSS Strategy

Dioxus-specific styling choice can be:

```text
vanilla CSS / CSS modules / Tailwind-like generation
```

but architecture must preserve design tokens.

---

# 154. Recommended Direction

Use strongly controlled design-system CSS with utility support only if it does not scatter semantic styling.

---

# 155. Component Library

Core components:

```text
Button
IconButton
Input
Select
Dialog
Sheet
Tabs
Badge
StatusBadge
Tooltip
DataTable
Card
EmptyState
ErrorState
Skeleton
Toast
Progress
Timeline
CodeViewer
LogViewer
DiffViewer
```

---

# 156. StatusBadge

Typed:

```rust
pub enum StatusKind {
    Success,
    Running,
    Waiting,
    Warning,
    Failure,
    Unknown,
    Neutral,
}
```

---

# 157. Canonical Domain State Mapping

Feature crate maps domain DTO state to StatusKind.

---

# 158. Toasts

Use for transient confirmation.

---

# 159. Durable Failure

Do not hide behind toast only.

Show inline state.

---

# 160. Loading Skeletons

Use for predictable layouts.

---

# 161. Empty States

Actionable.

---

# 162. Error Boundaries

Per feature/route.

---

# 163. Global Error Boundary

Prevents total UI crash where possible.

---

# 164. Panic Reporting

Sanitized.

---

# 165. Performance

Critical for large CI datasets.

---

# 166. Virtualization

Use for:

```text
logs
runs
jobs
artifacts
audit
diffs
```

---

# 167. Incremental Rendering

Avoid rebuilding whole page on one event.

---

# 168. Memoization

Use carefully.

---

# 169. Signal Granularity

Keep reactive state scoped.

---

# 170. Background Refresh

Use stale-while-revalidate behavior.

---

# 171. Request Coalescing

Avoid duplicate concurrent identical queries.

---

# 172. Prefetch

Useful:

```text
hover/click run
```

but bounded.

---

# 173. Cache Size

Bound memory.

---

# 174. Mobile Memory

Lower limits.

---

# 175. Images/Icons

Small optimized assets.

---

# 176. No Heavy Dashboard Animation

CI operator UI values clarity.

---

# 177. Charts

Use for:

```text
queue wait
runner utilization
deployment health
```

only where meaningful.

---

# 178. Chart Accessibility

Provide text/table equivalents.

---

# 179. Time Series

Downsample large data sets.

---

# 180. Relative Time

Show:

```text
"5m ago"
```

with exact timestamp tooltip.

---

# 181. Time Zone

User/local display.

Server timestamps UTC.

---

# 182. Duration

Consistent human formatting.

---

# 183. IDs

Short display + copy full ID.

---

# 184. Digests

Shortened in tables.

Full in detail/copy.

---

# 185. Dangerous Action Dialog

Must show exact target identity.

---

# 186. Example Release Promotion

Show:

```text
Release version
ReleaseId
Candidate digest
Target channel
```

---

# 187. Example Deployment Rollback

Show:

```text
current ReleaseId
target previous ReleaseId
migration safety
```

---

# 188. Optimistic UI Rule

Never optimistically show:

```text
Released
Deployed Healthy
Approved
```

before authoritative API response/event.

---

# 189. Action Pending

Use:

```text
Submitting
Queued
Reconciling
```

---

# 190. Partial Success

Represent per sub-operation.

---

# 191. Unknown State

First-class UX.

---

# 192. Unknown Copy

Example:

```text
"Forgeyard cannot yet confirm whether the remote provider applied this action."
```

---

# 193. Reconciliation UI

Show:

```text
last check
next check
manual recheck
```

where useful.

---

# 194. Conflict UI

Entity version conflict:

```text
resource changed
reload/compare
```

---

# 195. Stale Approval UI

Explain why invalidated.

---

# 196. Permission Change

Action may disappear/disable after refresh.

---

# 197. Capability-Driven UI

Fetch safe public capabilities.

---

# 198. Capability Examples

```text
mobile_store_enabled
kubernetes_deploy_enabled
device_lab_enabled
```

---

# 199. Do Not Infer from Missing Route

Use capability endpoint/config.

---

# 200. Feature Flags

Server-provided safe feature state.

---

# 201. Local Feature Flags

Only visual experiments.

---

# 202. Progressive Rollout of UI Features

Can gate by capability/version.

---

# 203. API Compatibility

UI supports server API versions within declared matrix.

---

# 204. Version Mismatch

Show actionable upgrade message.

---

# 205. Offline Desktop Mode

Could connect to local standalone daemon automatically.

---

# 206. Local Daemon Discovery

Explicit local socket/port.

---

# 207. No Hidden Embedded Authority in UI

Even single-binary mode composes same services.

---

# 208. Single-Binary Mode

UI and daemon in same process/package may communicate through same service/API semantics.

---

# 209. Desktop IPC

Optional optimization.

Must not bypass authz/domain semantics.

---

# 210. Mobile

Primarily remote control/visibility.

---

# 211. Mobile Write Actions

Allow safe actions:

```text
approve
cancel
drain
rollback
```

subject to UX/security.

---

# 212. High-Risk Mobile Action

Require reauth/biometric/step-up where available.

---

# 213. Mobile Log Viewer

Virtualized, follow tail, text search.

---

# 214. Mobile Charts

Simplified.

---

# 215. Mobile Admin

Can be limited.

---

# 216. Web

If deployed:

```text
same API
same auth/session
```

---

# 217. Browser Storage

Do not store sensitive tokens in localStorage if cookie session available.

---

# 218. Local Persistence

Safe preferences only:

```text
theme
density
last project
filters
```

---

# 219. Cached Data

If persisted offline:

```text
non-sensitive
encrypted/platform protected if needed
clear stale marking
```

---

# 220. Secret Data

Never persist.

---

# 221. Clipboard

Copy IDs/digests.

Secret copying only explicit privileged flow.

---

# 222. Deep Link Security

Opening a route does not imply access.

---

# 223. External Links

Confirm/untrusted URLs where appropriate.

---

# 224. Provider Links

Show SCM/provider link from normalized metadata.

---

# 225. URL Safety

No javascript/data URL injection.

---

# 226. Markdown

Release notes/descriptions may render Markdown.

---

# 227. Markdown Sanitization

Mandatory.

---

# 228. HTML

Sanitize/disable raw HTML by default.

---

# 229. Code Blocks

Safe escaping.

---

# 230. Diff Content

Untrusted text.

Escape.

---

# 231. Log Content

Untrusted text.

Escape.

---

# 232. XSS

Web target must treat all remote text as untrusted.

---

# 233. Desktop Webview Concern

If Dioxus uses webview backend on some platform, same untrusted-content protections apply.

---

# 234. CSP

Web delivery includes strict CSP via Axum.

---

# 235. URI Handlers

Desktop custom URI schemes must validate.

---

# 236. File Open

Artifact download/open explicit.

---

# 237. Local Filesystem Access

UI should not browse arbitrary server files.

---

# 238. Drag and Drop

Could support artifact upload/config import.

---

# 239. Upload Validation

Server authoritative.

---

# 240. Accessibility Testing

Automated + manual.

---

# 241. Localization

Architecture allows i18n later.

---

# 242. String Catalog

Feature text externalizable.

---

# 243. Initial Language

English.

---

# 244. Number/Date Locale

User locale.

---

# 245. Error Localization

UI maps stable API error codes to user-friendly text.

---

# 246. Do Not Parse Error Message Strings

Use `ApiErrorCode`.

---

# 247. Notification Center

Shows:

```text
approval needed
run failed
release blocked
deployment degraded
runner offline
```

---

# 248. Notification Severity

```text
Info
Warning
Critical
ActionRequired
```

---

# 249. Read/Unread

Client/server preference.

---

# 250. Notification Deep Link

To relevant entity.

---

# 251. Activity Timeline

Unified per entity.

---

# 252. Global Activity

Optional admin.

---

# 253. Audit Timeline

Separate from normal activity.

---

# 254. Audit UI

Permission-protected.

---

# 255. Project Overview

Widgets:

```text
recent runs
open proposals
release status
deployment status
runner health
```

---

# 256. Organization Overview

For enterprise:

```text
projects
fleet health
release/deployment incidents
```

---

# 257. Home Dashboard

Personalized:

```text
my approvals
failed runs
recent projects
```

---

# 258. Personalization

Only presentation.

No business semantics.

---

# 259. Empty First-Run Experience

Guide:

```text
create/import project
configure pipeline
register runner
run first build
```

---

# 260. Onboarding

Mode-specific.

---

# 261. Standalone Onboarding

```text
local project
local runner
embedded store
```

---

# 262. Distributed Onboarding

```text
server URL
login
project
runner enrollment
```

---

# 263. Setup Wizard

Can collect config but server validates.

---

# 264. Runner Enrollment UI

Admin creates one-time enrollment token.

---

# 265. Token Display

Show once.

---

# 266. QR Code

Could help mobile/device enrollment.

---

# 267. QR Security

Short-lived scoped token.

---

# 268. Device Lab UI

Later doc 20 provides details.

UI should support:

```text
devices
availability
health
leases
test runs
```

---

# 269. SCM Provider UI

Later doc 21.

Support:

```text
provider account
repository binding
webhook status
proposal links
```

---

# 270. Plugin UI

Later doc 24.

Extensions may contribute:

```text
pages
panels
actions
```

under constrained extension API.

---

# 271. Plugin Isolation

Do not allow arbitrary UI plugin to bypass auth/API.

---

# 272. UI Extension Registry

At bootstrap/runtime from trusted plugin descriptors.

---

# 273. Layout Stability

Feature plugins cannot arbitrarily replace core security surfaces.

---

# 274. Telemetry in UI

Client telemetry optional.

---

# 275. UI Metrics

```text
page load
API latency
SSE reconnect
render errors
```

---

# 276. Privacy

No secret/user content capture.

---

# 277. Crash Reports

Sanitized.

---

# 278. User Analytics

Optional/consent depending deployment.

Not core requirement.

---

# 279. UI Health

Show API connection status.

---

# 280. Connection Indicator

Subtle global indicator:

```text
Online
Reconnecting
Offline
```

---

# 281. Server Degraded Indicator

Global banner for:

```text
read-only
no scheduling
maintenance
```

---

# 282. Banner Specificity

Explain impact.

---

# 283. Maintenance Mode

Writes disabled.

---

# 284. Version Banner

UI/server version mismatch.

---

# 285. Upgrade Banner

Admin-only where appropriate.

---

# 286. Keyboard Shortcut Layer

Central, conflict-aware.

---

# 287. Focus Management

Dialogs/sheets trap focus correctly.

---

# 288. Modal Overuse

Avoid.

---

# 289. Side Panels

Good for entity detail without losing list context.

---

# 290. URL State

Filters/selected tabs can be encoded in URL where useful.

---

# 291. Shareable Views

Useful for run/release/deployment.

---

# 292. Sensitive Filters

Do not encode secret values.

---

# 293. Pagination URL

Cursor may be in URL.

---

# 294. UI Error Model

```rust
pub enum UiError {
    Network,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Validation,
    RateLimited,
    ServerUnavailable,
    Offline,
    Unknown,
}
```

---

# 295. Error Mapping

Stable API codes -> UI error.

---

# 296. Retry Guidance

Respect `retryable`.

---

# 297. Rate Limited UX

Show retry time.

---

# 298. Conflict UX

Offer refresh.

---

# 299. Permission Denied UX

Offer authz explain if allowed.

---

# 300. Session Timeout UX

Preserve safe form draft if possible.

---

# 301. Security-Sensitive Form Draft

Do not preserve secret fields.

---

# 302. Testkit

```text
forgeyard-ui-testkit/src/
├── lib.rs
├── fake_api.rs
├── fixtures.rs
├── router.rs
├── live.rs
├── permissions.rs
├── assertions.rs
└── snapshots.rs
```

---

# 303. Unit Tests

Test:

```text
state mapping
error mapping
permission visibility
formatters
```

---

# 304. Component Tests

Buttons/forms/tables/status.

---

# 305. Route Tests

Deep links.

---

# 306. Live Event Tests

SSE event invalidates/refetches correct query.

---

# 307. Offline Tests

API loss shows stale/offline.

---

# 308. Permission Tests

Hidden/disabled UI never substitutes for server authz.

---

# 309. Approval Tests

Candidate digest shown.

---

# 310. Unknown State Tests

Publication/deployment Unknown rendered distinctly.

---

# 311. Log Scale Tests

Millions of lines virtualized.

---

# 312. Table Scale Tests

Large run list.

---

# 313. Diff Scale Tests

Large changes.

---

# 314. Accessibility Tests

Keyboard, focus, labels, contrast.

---

# 315. Web Security Tests

XSS with malicious:

```text
log
release notes
provider text
diff
```

---

# 316. Secret Persistence Test

Secret input not in persistent client storage.

---

# 317. Mobile Tests

Small viewport/navigation.

---

# 318. Desktop Tests

Window/split layout.

---

# 319. Snapshot Tests

Use carefully for stable visual components.

---

# 320. E2E Tests

Flows:

```text
login
create run
watch job
approve release
deploy
rollback
```

---

# 321. Fake Server

API test harness.

---

# 322. Contract Tests

UI against current API schema.

---

# 323. Version Compatibility Tests

UI N with server N/N-1 where declared.

---

# 324. Performance Benchmarks

Measure:

```text
first render
route change
log append
table scroll
SSE burst
```

---

# 325. Memory Benchmarks

Large logs/tables.

---

# 326. Failure Injection

```text
API timeout
SSE disconnect
partial response
rate limit
session expiry
```

---

# 327. Implementation Phase 1 — App Shell

Implement:

```text
router
theme
layout
auth/session
API client
```

---

# 328. Phase 2 — Query/Live State

Cache + SSE reconnect.

---

# 329. Phase 3 — Runs/Jobs/Logs

Core CI experience.

---

# 330. Phase 4 — Runners/Artifacts

Fleet/data views.

---

# 331. Phase 5 — Change Proposals

Review/check/integration UI.

---

# 332. Phase 6 — Supply Chain/Packages

Evidence and package lineage.

---

# 333. Phase 7 — Releases

Candidate/approval/publication.

---

# 334. Phase 8 — Deployments

Plan/rollout/health/rollback.

---

# 335. Phase 9 — Health/Admin/Security

Doctor, policies, secrets, trust.

---

# 336. Phase 10 — Mobile Optimization

Navigation/density/offline.

---

# 337. Phase 11 — Accessibility/Performance

Hardening.

---

# 338. Phase 12 — Plugin/Device/SCM Extensions

Later subsystem integration.

---

# 339. Acceptance Tests

1. UI uses public API/domain services, not DB.
2. UI cannot authorize protected action itself.
3. Server permission denial is handled correctly.
4. Desktop/mobile/web share API semantics.
5. Route deep links are stable.
6. Run/job states map exactly to server canonical states.
7. Retry attempts are visible.
8. Scheduler placement explanation is visible.
9. Runner trust is not editable without permission.
10. Log viewer handles large logs without unbounded DOM.
11. Artifact evidence binds exact digest.
12. Change approval shows exact proposal revision.
13. Release approval shows exact candidate digest.
14. Deployment apply shows exact ReleaseId/plan.
15. Unknown external outcome is rendered distinctly.
16. Reconciliation state is visible.
17. Offline mode clearly marks stale data.
18. Writes are disabled while offline/read-only.
19. SSE reconnect resumes safely.
20. UI cache refreshes authoritative state after events.
21. Secret values are never persisted in normal client storage.
22. Secret reveal is explicit and short-lived if enabled.
23. Dangerous actions show exact target identity.
24. Large tables use server pagination/virtualization.
25. Accessibility works with keyboard/screen readers.
26. Status meaning does not rely on color alone.
27. Malicious log/Markdown/diff content cannot XSS.
28. Mobile UI remains usable for approvals/status/logs.
29. Desktop UI supports dense operational workflows.
30. UI/server API mismatch is surfaced clearly.
31. Health/degraded mode is visible globally.
32. Doctor UI triggers typed diagnostics only.
33. Plugin UI cannot bypass core API/authz.
34. Standalone and distributed modes share UI semantics.
35. Forgeyard operators can manage Forgeyard itself through this UI.

---

# 340. Production Readiness Gates

Do not call UI production-ready until:

```text
app shell/navigation stable
API client/auth/session stable
run/job/log views production-ready
SSE reconnect/backfill works
release/deployment high-risk actions show exact identities
offline/degraded states clear
secret persistence tests pass
large-data virtualization proven
accessibility baseline passes
XSS/untrusted-content tests pass
desktop/mobile responsive layouts stable
```

---

# 341. Architectural Invariants

1. UI is never authoritative;
2. server/domain state is truth;
3. UI uses public API semantics;
4. permission-aware UX does not replace authz;
5. optimistic state never fakes protected success;
6. Unknown/Partial/Degraded are first-class states;
7. live events trigger/refine server-state refresh;
8. SSE disconnect is recoverable;
9. cached data is clearly marked stale/offline;
10. writes are disabled when authority unavailable;
11. secrets are not stored in client state persistently;
12. dangerous actions show exact target identity;
13. release approval shows candidate digest;
14. deployment actions show ReleaseId/plan identity;
15. logs/diffs/Markdown are untrusted content;
16. large datasets are virtualized/paginated;
17. mobile/desktop share semantics, not layout;
18. UI feature availability follows server capabilities;
19. admin/security surfaces are permission-gated;
20. doctor never exposes arbitrary shell;
21. accessibility is a design requirement;
22. color is never sole status indicator;
23. API errors are mapped by stable codes;
24. no feature UI crate directly accesses persistence;
25. local mode does not fork business logic;
26. client telemetry is optional/non-authoritative;
27. plugin UI cannot bypass authz/API;
28. provider-specific UI stays adapter/feature-local;
29. UI supports self-hosting/operator workflows;
30. Forgeyard dogfoods the same UI.

---

# 342. Final Target Architecture

```text
                        User
                          │
                          ▼
                     Dioxus App
                          │
             ┌────────────┼────────────┐
             ▼            ▼            ▼
          Router       Query Cache    Live Events
             │            │            │
             └────────────┼────────────┘
                          ▼
                       API Client
                          │
               REST/JSON + SSE/WS
                          │
                          ▼
                       Axum API
                          │
                          ▼
                    Domain Services
                          │
                          ▼
                 Authoritative State
```

---

# 343. Final Architectural Position

Read path:

```text
route
  ↓
query cache
  ↓
Axum API
  ↓
authoritative DTO
  ↓
typed feature view
```

Live path:

```text
SSE event
  ↓
entity invalidation
  ↓
refetch current state
  ↓
update UI
```

Protected action:

```text
user intent
  ↓
confirmation with exact identity
  ↓
API request
  ↓
server authz/policy
  ↓
authoritative result
  ↓
UI refresh
```

Offline path:

```text
API unreachable
  ↓
cached snapshot
  ↓
stale/offline indicator
  ↓
write actions disabled
```

The key guarantee is:

> **Forgeyard's Dioxus UI makes a complex distributed CI/CD system understandable and operable without pretending the client owns truth. The interface visualizes authoritative server state, invokes typed capabilities, survives disconnects, adapts across platforms, and exposes uncertainty honestly rather than hiding it.**

---

# 344. New-Repository Sequence

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
