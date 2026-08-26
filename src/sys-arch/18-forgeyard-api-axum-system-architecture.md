# 18 — Forgeyard API / Axum System Architecture

**Document type:** Core Public API & Edge Interface System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Axum HTTP API, REST/JSON DTOs, authentication/authorization middleware, idempotency, pagination, filtering, WebSocket/SSE, uploads/downloads, public/admin API separation, webhooks, OpenAPI, rate limiting, CORS/CSRF, API versioning, error envelopes, backpressure, and API observability  
**Architecture style:** Thin transport boundary over domain services, versioned public contracts, explicit authorization, stateless request handling where practical, event-stream subscriptions for live updates, and strict separation from native QUIC agent protocol  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds on Identity/Authz/Policy, Events/Reconciliation, Observability/Health, Release, Deployment, Run/Job, CAS, Storage, Transport/Protocol, and all domain service APIs. It exposes Forgeyard to humans, CLIs, Dioxus UI, automation clients, and external integrations.

---

# 1. Purpose

Forgeyard needs a production-grade public API for:

```text
CLI
Dioxus UI
automation
SCM integrations
administration
release/deployment control
artifact access
live status
```

The API must not become a second implementation of domain logic.

The central rule is:

> **Axum is the transport and composition boundary. Domain services own business rules.**

A second rule is:

> **Public HTTP/JSON APIs and native agent QUIC/Postcard APIs are separate protocol families with separate versioning and security models.**

A third rule is:

> **Every mutating public API is authenticated, authorized, validated, idempotent where retried, observable, and bounded.**

---

# 2. Architectural Position

```text
Browser / CLI / Automation / Provider
               │
          HTTPS / JSON
               │
               ▼
            Axum API
               │
      ┌────────┼─────────┐
      ▼        ▼         ▼
   Authn     Authz     Validation
      │        │         │
      └────────┼─────────┘
               ▼
          Domain Service
               │
               ▼
       Store / CAS / Events
```

Live updates:

```text
Domain Events
    ↓
Event Stream
    ↓
WS / SSE
    ↓
UI / CLI
```

Internal agent path remains:

```text
Daemon ↔ Agent
QUIC + Postcard
```

not REST.

---

# 3. Goals

The API subsystem MUST:

1. use Axum;
2. expose versioned REST/JSON;
3. support CLI and UI;
4. support browser sessions;
5. support API tokens/service accounts;
6. support OIDC-backed sessions;
7. enforce authz per operation;
8. support idempotency keys;
9. support cursor pagination;
10. support filtering/sorting;
11. support WebSocket/SSE;
12. support artifact upload/download;
13. support presigned/direct CAS flows;
14. support webhook ingress;
15. support admin APIs;
16. support health/readiness endpoints;
17. support OpenAPI generation;
18. support stable error envelopes;
19. support request size limits;
20. support rate limiting;
21. support CORS;
22. support CSRF protection;
23. support request deadlines;
24. support cancellation on client disconnect where appropriate;
25. support backpressure;
26. support trace propagation;
27. support structured audit for sensitive actions;
28. remain domain-thin;
29. avoid agent-protocol duplication;
30. support rolling API evolution.

---

# 4. Non-Goals

The HTTP API does not:

```text
execute arbitrary shell
schedule jobs directly
hold business policy logic
replace QUIC worker protocol
serve as CAS blob store for all large traffic if direct object storage is better
```

---

# 5. Workspace Structure

```text
crates/api/
├── forgeyard-api/
├── forgeyard-api-model/
├── forgeyard-api-error/
├── forgeyard-api-auth/
├── forgeyard-api-authz/
├── forgeyard-api-pagination/
├── forgeyard-api-idempotency/
├── forgeyard-api-rate-limit/
├── forgeyard-api-websocket/
├── forgeyard-api-sse/
├── forgeyard-api-upload/
├── forgeyard-api-download/
├── forgeyard-api-webhook/
├── forgeyard-api-openapi/
├── forgeyard-api-admin/
├── forgeyard-api-health/
└── forgeyard-api-testkit/
```

Axum composition:

```text
apps/forgeyard-daemon/src/
├── api/
│   ├── mod.rs
│   ├── router.rs
│   ├── middleware.rs
│   ├── state.rs
│   └── shutdown.rs
```

---

# 6. Public API Version

Use path-based major version:

```text
/v1/...
```

---

# 7. Why Major Path Version

Clear external compatibility boundary.

---

# 8. Minor Evolution

Within `/v1`:

```text
add optional fields
add endpoints
add enum variants only with compatibility discipline
```

---

# 9. Breaking Change

Create:

```text
/v2
```

---

# 10. API DTOs

Public DTOs live in:

```text
forgeyard-api-model
```

---

# 11. DTO / Domain Separation

Never expose domain structs directly just because Serde can serialize them.

---

# 12. Why Separate DTOs

Allows:

```text
domain refactor
field hiding
redaction
public compatibility
```

---

# 13. JSON

Public REST uses JSON.

This is appropriate interoperability usage.

---

# 14. RON/Postcard

Not used as default public web API body format.

---

# 15. Content Type

```text
application/json
```

---

# 16. Request Envelope

Most REST requests need no generic outer envelope.

Use normal resource-oriented JSON.

---

# 17. Error Envelope

```rust
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
    pub request_id: RequestId,
}
```

---

# 18. ApiErrorBody

```rust
pub struct ApiErrorBody {
    pub code: ApiErrorCode,
    pub message: String,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
}
```

---

# 19. Safe Details

Never expose:

```text
SQL
stack trace
secret
private filesystem path
```

---

# 20. Error Mapping

Domain error -> API error.

---

# 21. HTTP Status Mapping

Examples:

```text
400 validation
401 unauthenticated
403 forbidden
404 not found
409 conflict
412 precondition
422 semantic invalid
429 rate limited
503 unavailable
```

---

# 22. RequestId

Generated per HTTP request.

---

# 23. TraceId

Separate from RequestId.

---

# 24. Authentication Middleware

Extracts:

```text
browser session
Bearer token
service token
```

---

# 25. Principal Context

Request extensions contain validated:

```rust
pub struct RequestPrincipal {
    pub principal: PrincipalId,
    pub authn: AuthenticationContext,
}
```

---

# 26. Middleware Does Not Fully Authorize

Route/domain action still checks required permission/resource.

---

# 27. Authorization Layer

Pattern:

```text
extract principal
  ↓
resolve target resource
  ↓
authorize permission
  ↓
call service
```

---

# 28. Avoid Hidden Route-Only Authz

Critical services can also require authorization/capability argument.

---

# 29. Browser Session

Cookie-based.

---

# 30. Cookie Requirements

```text
Secure
HttpOnly
SameSite
```

---

# 31. CSRF

Required for cookie-authenticated state-changing requests.

---

# 32. Bearer Token

CSRF not applicable in same way.

---

# 33. CORS

Deny broad wildcard by default.

---

# 34. UI Same-Origin

Preferred deployment.

---

# 35. Cross-Origin Clients

Explicit allowed origins.

---

# 36. Preflight

Handled centrally.

---

# 37. API Token Scope

Authz still enforced.

---

# 38. Token Prefix

May use recognizable token prefix for operational identification.

---

# 39. Never Log Token

Redacted.

---

# 40. Idempotency

Mutating operations that clients may retry should support:

```text
Idempotency-Key
```

---

# 41. Idempotency Record

```rust
pub struct ApiIdempotencyRecord {
    pub principal: PrincipalId,
    pub key: IdempotencyKey,
    pub route: ApiOperationId,
    pub request_digest: Digest,
    pub response_ref: IdempotentResponseRef,
}
```

---

# 42. Same Key, Same Request

Return prior result.

---

# 43. Same Key, Different Request

```text
409 Conflict
```

---

# 44. Idempotency Scope

Principal + operation + key.

---

# 45. Idempotency TTL

Operation-specific.

---

# 46. Strong Candidates

```text
create run
create release
promote release
create deployment
rollback deployment
create token
```

---

# 47. GET

Naturally idempotent.

---

# 48. PUT/PATCH

Use optimistic concurrency/precondition where appropriate.

---

# 49. Entity Version

Expose ETag/version.

---

# 50. ETag

Can represent:

```text
EntityVersion
```

---

# 51. If-Match

For update/delete.

---

# 52. Lost Update Prevention

Mutating stale version -> 412/409.

---

# 53. Pagination

Use cursor/keyset pagination.

---

# 54. Avoid Offset at Scale

Offset may still exist for tiny admin lists, but not default.

---

# 55. Cursor Request

```text
?cursor=...&limit=50
```

---

# 56. Cursor Response

```rust
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}
```

---

# 57. Cursor Opacity

Clients must not parse.

---

# 58. Cursor Authorization

Cursor does not bypass tenant/authz filtering.

---

# 59. Limit

Bound maximum.

---

# 60. Filtering

Typed query parameters.

Examples:

```text
state=running
project_id=...
created_after=...
```

---

# 61. Sorting

Allow bounded predefined sort fields.

---

# 62. No Arbitrary SQL-Like Filters

Avoid injection/complexity.

---

# 63. Search

Separate endpoint/typed search service if needed.

---

# 64. Resource Endpoints

Examples:

```text
/v1/projects
/v1/runs
/v1/jobs
/v1/runners
/v1/artifacts
/v1/releases
/v1/deployments
/v1/policies
```

---

# 65. Run API

```text
POST /v1/runs
GET  /v1/runs/{id}
POST /v1/runs/{id}/cancel
GET  /v1/runs/{id}/jobs
GET  /v1/runs/{id}/events
```

---

# 66. Job API

```text
GET /v1/jobs/{id}
GET /v1/jobs/{id}/attempts
GET /v1/jobs/{id}/logs
```

---

# 67. Scheduler API

Mostly admin/read:

```text
GET /v1/admin/scheduler/status
GET /v1/admin/scheduler/explain/{job}
```

---

# 68. Runner API

```text
GET  /v1/runners
GET  /v1/runners/{id}
POST /v1/runners/{id}/drain
POST /v1/runners/{id}/resume
```

---

# 69. Artifact API

```text
GET /v1/artifacts/{id}
GET /v1/artifacts/{id}/download
GET /v1/artifacts/{id}/evidence
```

---

# 70. Package API

```text
GET /v1/packages
GET /v1/packages/{id}
POST /v1/packages/{id}/validate
```

---

# 71. Release API

As defined in release architecture.

---

# 72. Deployment API

As defined in deployment architecture.

---

# 73. Policy API

```text
GET  /v1/policies/effective
POST /v1/policies/validate
POST /v1/policies/activate
```

---

# 74. Secret API

Metadata only by default:

```text
GET  /v1/secrets
POST /v1/secrets
POST /v1/secrets/{id}/rotate
```

---

# 75. No Plaintext Secret GET by Default

Explicit privileged reveal endpoint only if product supports it.

---

# 76. Audit API

```text
GET /v1/audit/events
```

permission-gated.

---

# 77. Health API

```text
GET /health/live
GET /health/ready
GET /v1/admin/health
```

---

# 78. Metrics API

```text
/metrics
```

internal/admin.

---

# 79. WebSocket

Use for interactive live updates where bidirectional behavior useful.

---

# 80. SSE

Use for simple server→client event streams.

---

# 81. Recommendation

Prefer SSE for:

```text
run/release/deployment live events
```

when only server push needed.

---

# 82. WebSocket Use

Potential for:

```text
interactive log control
future terminal-like UI
```

but no arbitrary shell.

---

# 83. Event Stream Endpoint

```text
GET /v1/events/stream
```

with filters.

---

# 84. Run Stream

```text
GET /v1/runs/{id}/events/stream
```

---

# 85. Cursor Resume

Header/query:

```text
Last-Event-ID
cursor
```

---

# 86. Backfill

Event stream service reads durable event store.

---

# 87. Slow Client

Bound buffer.

Disconnect when client falls too far behind.

---

# 88. Reconnect

Client resumes from cursor.

---

# 89. WebSocket/SSE Auth

Same principal/session auth.

---

# 90. Event Authorization

Filter by tenant/resource permission.

---

# 91. No Global Internal Event Leakage

Public event DTO is redacted projection.

---

# 92. Job Logs API

Live:

```text
SSE/WebSocket
```

Historical:

```text
paged/chunked download
```

---

# 93. Log Cursor

Sequence-based.

---

# 94. Binary Logs

Can use downloadable blob/chunk endpoint.

---

# 95. Uploads

Small metadata JSON through normal API.

Large artifacts/files use streaming/direct CAS.

---

# 96. Artifact Upload Flow

```text
request upload intent
  ↓
authz
  ↓
create upload session
  ↓
direct/presigned upload
  ↓
finalize
  ↓
digest verify
```

---

# 97. Upload Session

```rust
pub struct UploadSession {
    pub id: UploadSessionId,
    pub expected_digest: Option<Digest>,
    pub max_size: ByteSize,
    pub expires_at: Timestamp,
}
```

---

# 98. Direct Daemon Upload

Supported for standalone/small objects.

---

# 99. Object Store Presigned Upload

Preferred at scale.

---

# 100. Upload Auth

Presigned capability scoped:

```text
object
size
expiry
```

---

# 101. Finalize Upload

Daemon verifies object exists/digest.

---

# 102. No Trust in Client-Claimed Digest Alone

Server/CAS verifies.

---

# 103. Downloads

Small metadata via API.

Large artifacts:

```text
presigned/direct object store
streaming endpoint
```

---

# 104. Download Authorization

Check artifact metadata authz before issuing URL.

---

# 105. Presigned URL TTL

Short.

---

# 106. Range Requests

Support for large artifact download where backend allows.

---

# 107. Content-Disposition

Safe sanitized filename.

---

# 108. Content-Type

From artifact/package metadata.

---

# 109. Checksums

Expose exact digest headers/metadata.

---

# 110. Webhook Ingress

Separate route family:

```text
/webhooks/{provider}/...
```

---

# 111. Webhook Flow

```text
receive
  ↓
size limit
  ↓
verify signature
  ↓
dedupe delivery
  ↓
persist
  ↓
normalize
  ↓
domain event/command
```

---

# 112. Verify Before Parse Deeply

Use raw body signature where provider requires.

---

# 113. Webhook Delivery ID

Dedup.

---

# 114. Provider-Specific Parser

SCM adapter layer.

---

# 115. Webhook Response

Fast.

Do not block on full CI workflow.

---

# 116. Webhook Queue

Persist accepted delivery then process asynchronously.

---

# 117. Webhook Retry

Provider may redeliver.

Safe.

---

# 118. Webhook Secrets

SecretRef.

---

# 119. Admin API

Separate path:

```text
/v1/admin/...
```

---

# 120. Admin Authz

Explicit permissions.

---

# 121. Admin Endpoints

Examples:

```text
health
migrations
scheduler
reconciliation
events/deadletters
runner enrollment
trust roots
system config
```

---

# 122. No Hidden Superuser Route

Same authz framework.

---

# 123. Break-Glass API

Explicit action endpoint if needed.

---

# 124. Break-Glass Requirements

Strong auth/reason/audit.

---

# 125. API State

```rust
pub struct ApiState {
    pub services: Arc<ServiceRegistry>,
    pub authn: Arc<AuthenticationService>,
    pub authz: Arc<AuthorizationService>,
    pub telemetry: Arc<Telemetry>,
}
```

---

# 126. ServiceRegistry

Explicit typed app composition.

Avoid service locator abuse if possible.

---

# 127. Better Pattern

State contains concrete service handles/grouped modules.

---

# 128. Router Composition

```rust
pub fn build_router(state: ApiState) -> Router
```

---

# 129. Route Modules

```text
api/routes/
├── runs.rs
├── jobs.rs
├── runners.rs
├── artifacts.rs
├── releases.rs
├── deployments.rs
├── policies.rs
├── secrets.rs
├── admin.rs
└── webhooks.rs
```

---

# 130. Handler Rule

Handler should:

```text
extract
validate
authorize
call service
map response
```

---

# 131. Handler Must Not

```text
write SQL
implement scheduler
open CAS internals directly unless API service abstraction
decide release policy
```

---

# 132. Validation

Use typed request DTO validation.

---

# 133. Validation Error

Return field-level diagnostics.

---

# 134. Request Size Limits

Per route.

---

# 135. JSON Size

Bound globally + route-specific.

---

# 136. Webhook Size

Provider-specific limit.

---

# 137. Multipart

Use only where appropriate.

Streaming.

---

# 138. Timeouts

Global request timeout.

Long operations return operation resource rather than holding HTTP open.

---

# 139. Async Operation Pattern

Example release promotion:

```text
POST
  ↓
202 Accepted
  ↓
operation/release state
```

---

# 140. Operation Resource

Optional generic:

```rust
pub struct OperationId(Ulid);
```

---

# 141. Prefer Domain Resource State

If Release/Deployment already has async state, no generic operation needed.

---

# 142. Client Disconnect

Do not automatically cancel durable domain action after accepted commit.

---

# 143. Before Commit Disconnect

Request cancellation can propagate if safe.

---

# 144. Streaming Disconnect

Stop stream only.

---

# 145. Rate Limiting

Multi-layer:

```text
global
principal
route class
IP supplemental
```

---

# 146. IP Limit

Not sole identity because NAT.

---

# 147. Rate Limit Classes

```text
read
write
login
webhook
upload
admin
```

---

# 148. RateLimitKey

Principal when authenticated.

---

# 149. 429 Response

Include retry guidance.

---

# 150. Distributed Rate Limit

Can use DB/Redis-like external later.

Initial per-daemon + security-sensitive central counters where needed.

---

# 151. Correctness

Rate limiter is protection, not domain authority.

---

# 152. Load Shedding

If server overloaded:

```text
reject low-priority requests
preserve health/control
```

---

# 153. Backpressure

Bound:

```text
request body
inflight handlers
DB concurrency
upload streams
event streams
```

---

# 154. Axum Concurrency Layer

Use Tower layers.

---

# 155. Tower

Appropriate for:

```text
timeout
load shed
rate limit
trace
CORS
compression
```

---

# 156. Compression

JSON responses can compress.

Avoid already compressed artifacts.

---

# 157. Request Decompression

If supported, bound decompressed size.

---

# 158. TLS

Terminate in daemon or trusted reverse proxy.

---

# 159. Reverse Proxy

Support:

```text
Caddy
nginx
cloud LB
```

---

# 160. Trusted Proxy Headers

Only honor forwarded headers from configured trusted proxy.

---

# 161. Client IP

Diagnostic/rate supplemental.

---

# 162. HTTP/2

Supported.

---

# 163. HTTP/3

Optional future.

Do not conflate with internal QUIC protocol.

---

# 164. OpenAPI

Generate/document public API.

---

# 165. OpenAPI Source

DTO/routes annotations or code generation approach.

---

# 166. Schema Stability

Published OpenAPI becomes compatibility artifact.

---

# 167. API Documentation

Host:

```text
/docs/api
```

optional.

---

# 168. SDK Generation

Possible future from OpenAPI.

---

# 169. Rust CLI Client

Can use typed internal client crate generated/handwritten from DTOs.

---

# 170. API Client Crate

```text
crates/api/forgeyard-api-client/
```

optional.

---

# 171. Client Responsibilities

```text
auth
pagination
idempotency
retry safe operations
SSE reconnect
```

---

# 172. Retry Policy

Only retry safe/idempotent operations.

---

# 173. Retryable Codes

```text
429
502
503
504
```

with idempotency considerations.

---

# 174. No Automatic Retry on Arbitrary POST

Unless Idempotency-Key present/operation known safe.

---

# 175. API Compatibility Tests

Golden JSON fixtures.

---

# 176. Unknown Fields

Clients should ignore unknown response fields where possible.

---

# 177. Required Fields

Do not remove/change semantics within major version.

---

# 178. Enum Evolution

Use string enums carefully.

Unknown enum values can break generated clients.

---

# 179. Extensible Enum Pattern

Potential wrapper/string newtype for provider/extensible values.

---

# 180. Timestamps

RFC 3339 UTC.

---

# 181. IDs

Canonical string representation.

---

# 182. Byte Sizes

Numeric bytes + optional human rendering.

---

# 183. Durations

Use explicit units/string schema.

---

# 184. Digests

Expose:

```text
algorithm
hex
```

or standardized digest string.

---

# 185. Pagination Stability

Cursor tied to sort/filter parameters.

---

# 186. Invalid Cursor

400.

---

# 187. Expired Cursor

410/400 depending API convention.

---

# 188. Field Selection

Not needed initially.

---

# 189. Expand Relations

Avoid arbitrary GraphQL-like expansion initially.

---

# 190. GraphQL

Not required.

REST + SSE sufficient.

---

# 191. gRPC Public API

Not needed.

Keep only RBE/interoperability.

---

# 192. API Security Headers

Set:

```text
X-Content-Type-Options
Referrer-Policy
Content-Security-Policy for UI
HSTS where HTTPS direct
```

---

# 193. CSP

UI-specific but server can provide.

---

# 194. Frame Protection

Use CSP `frame-ancestors`.

---

# 195. Request Logging

Log:

```text
method
route template
status
duration
request_id
principal_id optional internal
```

---

# 196. Do Not Log Full URL Query Blindly

May contain sensitive data.

---

# 197. Route Template

Better metric dimension than raw path.

---

# 198. HTTP Metrics

```text
forgeyard_http_requests_total
forgeyard_http_request_duration_seconds
forgeyard_http_inflight_requests
forgeyard_http_rate_limited_total
```

---

# 199. Labels

```text
method
route
status_class
```

bounded.

---

# 200. Trace Middleware

Create span per request.

---

# 201. W3C Trace

Extract/inject.

---

# 202. Request Context

```rust
pub struct RequestContext {
    pub request_id: RequestId,
    pub principal: Option<PrincipalId>,
    pub trace: TraceContext,
}
```

---

# 203. Audit Trigger

Sensitive mutation route/domain service emits audit.

---

# 204. No Audit from Access Log Alone

Audit semantics separate.

---

# 205. Health Endpoint Metrics

Exclude/special-case from noisy request metrics if needed.

---

# 206. SSE Metrics

Connections, lag, disconnect reason.

---

# 207. WebSocket Metrics

Same.

---

# 208. Upload Metrics

Bytes, duration, failures.

---

# 209. Webhook Metrics

Verification failure, duplicate, processing lag.

---

# 210. API Health

Checks:

```text
router ready
authn provider
store
event stream
```

---

# 211. API Readiness

Depends on required store/services.

---

# 212. API Degraded Mode

Read-only if storage writes unavailable and app policy allows.

---

# 213. Read-Only Middleware

Can reject mutating methods with 503/423 depending convention.

---

# 214. Maintenance Mode

Explicit admin state.

---

# 215. Maintenance Response

Retry guidance.

---

# 216. Migration Mode

During incompatible DB migration:

```text
readiness false
```

---

# 217. Webhook During Degraded DB

Reject so provider retries.

---

# 218. API Config

Example:

```ron
(
    http: (
        listen: "0.0.0.0:8080",
        public_base_url: "https://forgeyard.example.com",
        request_timeout: "30s",
        max_json_body: "2MiB",
        cors: (
            allowed_origins: [
                "https://forgeyard.example.com",
            ],
        ),
    ),
)
```

---

# 219. Session Config

Separate auth config.

---

# 220. API Base URL

Used for callbacks/webhooks.

---

# 221. Host Validation

Protect against Host header abuse where relevant.

---

# 222. OIDC Callback

Dedicated endpoint.

---

# 223. Login Flow

```text
/login
/auth/oidc/callback
/logout
```

---

# 224. API vs UI Routes

UI static/app routes separate from `/v1`.

---

# 225. Dioxus UI

Calls same API as CLI where possible.

---

# 226. Internal Server-Side UI Shortcuts

Avoid bypassing API/service boundaries unless same service layer and authz.

---

# 227. CLI

Uses public API.

---

# 228. Local Standalone CLI

Can optionally use local socket/native internal client for zero-config, but semantics same.

---

# 229. Unix Socket API

Optional local admin/standalone transport.

---

# 230. Named Pipe

Windows equivalent.

---

# 231. HTTP Over Local Socket

Possible.

---

# 232. Local Auth

OS peer credentials.

---

# 233. Same DTO/Service

Do not fork logic.

---

# 234. API Testkit

```text
forgeyard-api-testkit/src/
├── lib.rs
├── app.rs
├── auth.rs
├── request.rs
├── response.rs
├── idempotency.rs
├── pagination.rs
├── sse.rs
├── webhook.rs
└── assertions.rs
```

---

# 235. Unit Tests

Test DTO validation/error mapping.

---

# 236. Router Integration Tests

Real Axum router with fake services.

---

# 237. Auth Tests

```text
no auth -> 401
permission missing -> 403
scope mismatch -> 403
```

---

# 238. Tenant Isolation Test

Guess other tenant resource ID -> denied/not-found per anti-enumeration policy.

---

# 239. Idempotency Test

Duplicate create run same key -> same result.

---

# 240. Idempotency Conflict Test

Same key different body -> 409.

---

# 241. ETag Test

Stale If-Match rejected.

---

# 242. Pagination Test

No duplicate/missing items under stable snapshot assumptions.

---

# 243. Cursor Tamper Test

Invalid signed/opaque cursor rejected.

---

# 244. SSE Resume Test

Disconnect/reconnect from cursor receives missed events.

---

# 245. Slow SSE Client Test

Bounded buffer/disconnect.

---

# 246. Upload Test

Oversized upload rejected.

---

# 247. Digest Test

Client-declared wrong digest rejected at finalize.

---

# 248. Presigned URL Auth Test

Unauthorized user cannot obtain URL.

---

# 249. Webhook Signature Test

Invalid signature rejected.

---

# 250. Webhook Duplicate Test

Same delivery processed once semantically.

---

# 251. Webhook Fast Ack Test

Does not wait for full CI completion.

---

# 252. Rate Limit Test

429 + retry guidance.

---

# 253. CSRF Test

Cookie mutation without CSRF token rejected.

---

# 254. CORS Test

Unknown origin denied.

---

# 255. Security Header Test

Expected headers set.

---

# 256. Body Bomb Test

Compressed/decompressed size bounded.

---

# 257. JSON Fuzzing

Fuzz request decoders/validation.

---

# 258. SSE/WS Fuzzing

Malformed client messages bounded.

---

# 259. OpenAPI Test

Generated schema matches route contract.

---

# 260. Compatibility Test

Old v1 JSON fixtures still decode/serve.

---

# 261. Failure Injection

```text
DB timeout
CAS unavailable
authz unavailable
event stream lag
rate limiter failure
```

---

# 262. Fail-Closed Authz

If protected authz cannot determine:

```text
deny/503
```

not allow.

---

# 263. Load Test

High concurrent reads/writes/SSE.

---

# 264. Long Polling

Not preferred.

SSE better.

---

# 265. API Performance

Avoid N+1 store calls.

---

# 266. Batch Endpoints

Add when real workflows need.

---

# 267. Batch Authorization

Authorize each resource/scope safely.

---

# 268. Bulk Operations

Return per-item result.

---

# 269. Partial Failure

Explicit.

---

# 270. Transaction Scope

Only where semantically atomic.

---

# 271. No Huge DB Tx Around HTTP Stream

Critical.

---

# 272. API Documentation Examples

Use synthetic data.

---

# 273. OpenAPI Security Schemes

Document:

```text
cookie session
Bearer token
```

---

# 274. Webhook Docs

Provider-specific.

---

# 275. API Deprecation

Return:

```text
Deprecation
Sunset
```

headers where useful.

---

# 276. Deprecation Policy

Document before removal.

---

# 277. Rolling Upgrade

New daemon must serve current public API throughout deployment.

---

# 278. API Feature Discovery

Optional:

```text
GET /v1/capabilities
```

---

# 279. Capability Endpoint

Returns supported public features.

---

# 280. No Internal Capability Leak

Public safe subset only.

---

# 281. API Error Codes

Stable registry.

---

# 282. Example Codes

```text
RUN_NOT_FOUND
PERMISSION_DENIED
VERSION_CONFLICT
IDEMPOTENCY_CONFLICT
RATE_LIMITED
SERVICE_UNAVAILABLE
```

---

# 283. Client Behavior

Clients should key logic on code, not message.

---

# 284. Localization

API messages may remain English stable; UI localizes separately.

---

# 285. Error Details

Machine-structured field errors.

---

# 286. Validation Error Example

```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "request validation failed",
    "retryable": false,
    "details": {
      "fields": {
        "version": "invalid format"
      }
    }
  }
}
```

---

# 287. Sensitive Resource Not Found

May return 404 instead of 403 to reduce enumeration, per policy.

---

# 288. API Authz Explain

Admin endpoint can expose decision details if permitted.

---

# 289. API Request Replay

Idempotency cache is not a general replay log.

---

# 290. API Operation IDs

Stable OpenAPI operation names.

---

# 291. Handler Naming

Match operation IDs.

---

# 292. Code Generation

Optional.

---

# 293. Axum State Cloning

Use Arc handles.

---

# 294. Blocking Work

Never run blocking CPU/FS-heavy work on async executor directly.

---

# 295. CPU Work

Use Rayon/spawn_blocking/service workers where appropriate.

---

# 296. Streaming

Use `Body` streaming.

---

# 297. Backpressure-Aware Body

Yes.

---

# 298. Database Pool

Shared through store adapter.

---

# 299. Per-Request Transaction

Only when domain operation needs.

---

# 300. Middleware Ordering

Recommended:

```text
request ID
trace
security headers
body limit
CORS
rate limit
authn
CSRF
handler/authz
timeout
error mapping
```

Exact Tower order tested.

---

# 301. Compression Placement

After response generation.

---

# 302. Panic Handling

Convert panic to 500 safely.

---

# 303. Panic Logging

Trace/request ID, no sensitive body dump.

---

# 304. Graceful Shutdown

Stop accepting new connections.

---

# 305. Drain

Allow in-flight bounded requests.

---

# 306. SSE Shutdown

Send close/reconnect hint if possible.

---

# 307. WebSocket Shutdown

Close frames.

---

# 308. Upload Shutdown

May abort resumable upload safely.

---

# 309. API Server Health on Shutdown

Readiness false first.

---

# 310. Implementation Phase 1 — Core Router/DTO/Error

Implement:

```text
Axum app
/v1
DTO separation
error envelope
request IDs
```

---

# 311. Phase 2 — Authn/Authz

Sessions/tokens/permission enforcement.

---

# 312. Phase 3 — Run/Job/Artifact API

Core CI workflow.

---

# 313. Phase 4 — Pagination/Idempotency/ETag

Production mutation/read semantics.

---

# 314. Phase 5 — SSE

Run/job/release/deployment live updates.

---

# 315. Phase 6 — Upload/Download

CAS/presigned flow.

---

# 316. Phase 7 — Webhooks

SCM/provider ingress.

---

# 317. Phase 8 — Release/Deployment APIs

Full orchestration.

---

# 318. Phase 9 — Admin/Health/OpenAPI

Operations.

---

# 319. Phase 10 — Rate Limit/Load Shed

Hardening.

---

# 320. Phase 11 — Local Socket

Standalone convenience.

---

# 321. Phase 12 — Compatibility/Fuzz/Scale

Production hardening.

---

# 322. Acceptance Tests

1. Axum handlers contain no core business logic.
2. Public DTOs are separate from domain models.
3. Public API version is explicit.
4. Breaking changes require new major version.
5. Unauthenticated protected request returns 401.
6. Unauthorized resource action returns safe denial.
7. Tenant boundary is enforced server-side.
8. UI-hidden controls are not relied on for security.
9. Idempotency-Key safely deduplicates supported mutations.
10. Same idempotency key with different body conflicts.
11. Entity version/ETag prevents lost updates.
12. Cursor pagination is opaque and bounded.
13. Invalid cursor cannot bypass authz.
14. SSE reconnect resumes from durable event cursor.
15. Slow live clients cannot consume unbounded server memory.
16. Artifact upload is size-bounded.
17. Uploaded artifact digest is verified.
18. Large download can use direct/presigned flow.
19. Download URL is issued only after authz.
20. Webhook signature is verified before semantic processing.
21. Duplicate webhook delivery is idempotent.
22. Webhook route responds quickly after durable acceptance.
23. Cookie mutations require CSRF protection.
24. CORS is deny-by-default.
25. API tokens are never logged.
26. Request bodies are bounded.
27. Request deadlines are bounded.
28. Telemetry traces propagate through API.
29. Metrics use route templates, not raw IDs.
30. Authz uncertainty fails closed for protected writes.
31. Public REST remains separate from agent QUIC protocol.
32. OpenAPI matches implemented routes.
33. Standalone/local socket uses same service semantics.
34. Graceful shutdown drains safely.
35. Forgeyard UI and CLI dogfood the same public API.

---

# 323. Production Readiness Gates

Do not call API production-ready until:

```text
DTO/domain separation enforced
authn/authz complete
tenant isolation tested
idempotency supported for critical writes
pagination stable
SSE backfill/resume works
upload/download bounded
webhook verification/dedup works
CSRF/CORS hardened
rate limiting/load shedding configured
OpenAPI published
compatibility tests pass
```

---

# 324. Architectural Invariants

1. Axum is transport boundary, not business layer;
2. public DTOs are not domain models;
3. public HTTP and internal QUIC protocols are separate;
4. JSON is used for public interoperability;
5. every protected mutation requires authn/authz;
6. UI is never security authority;
7. mutating retries use idempotency where needed;
8. stale writes use version/precondition checks;
9. pagination is cursor/keyset-based by default;
10. live streams are bounded and resumable;
11. public events are redacted projections;
12. large blobs are streamed/direct, not giant JSON;
13. uploaded content is digest-verified;
14. webhook signatures are verified;
15. webhook deliveries are deduplicated;
16. cookie-authenticated writes are CSRF-protected;
17. CORS is explicit;
18. request/body sizes are bounded;
19. request deadlines are bounded;
20. external client disconnect does not undo committed durable work;
21. rate limiting is defense, not domain authority;
22. authz fails closed for protected actions;
23. internal stack traces/secrets never enter API errors;
24. metric labels use route templates, not raw IDs;
25. OpenAPI is compatibility artifact;
26. breaking changes create new API major version;
27. provider-specific webhooks stay adapter-local;
28. standalone/distributed share API semantics;
29. local socket convenience does not fork business logic;
30. Forgeyard UI/CLI dogfood the same API.

---

# 325. Final Target Architecture

```text
                 Browser / CLI / Automation
                           │
                       HTTPS / JSON
                           │
                           ▼
                        Axum
                           │
         ┌─────────────────┼──────────────────┐
         ▼                 ▼                  ▼
      Authn             Validation          Limits
         │                 │                  │
         └─────────────────┼──────────────────┘
                           ▼
                         Authz
                           │
                           ▼
                    Domain Services
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
          Store            CAS           Events
                                             │
                                             ▼
                                          SSE/WS
```

---

# 326. Final Architectural Position

Request path:

```text
HTTP request
  ↓
request ID + trace
  ↓
body/size validation
  ↓
authentication
  ↓
resource-scoped authorization
  ↓
domain service
  ↓
stable API DTO/error
```

Mutation safety:

```text
Idempotency-Key
+
EntityVersion / If-Match
+
authz
  ↓
safe retry / conflict handling
```

Live state:

```text
durable events
  ↓
public projection
  ↓
SSE/WS
  ↓
cursor resume
```

The key guarantee is:

> **Forgeyard's Axum API is a stable, secure, observable edge over the same domain services that power the rest of the platform. It never becomes a parallel source of business truth, and it never substitutes REST for the purpose-built internal runner protocol.**

---

# 327. New-Repository Sequence

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
