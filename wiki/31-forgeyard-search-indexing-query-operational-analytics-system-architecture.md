# 31 — Forgeyard Search, Indexing, Query & Operational Analytics System Architecture

**Document type:** Core Search, Indexing, Query Projection & Operational Analytics System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** global search, entity indexing, metadata search, faceted filtering, saved queries, indexed projections, log search metadata, audit/event search, operational analytics, aggregation, reindexing, freshness, query isolation, query performance, and search-provider abstraction  
**Architecture style:** Derived read models, tenant-scoped indexes, event-driven projection plus reconciliation, explicit freshness semantics, bounded query complexity, source-of-truth separation, and provider-neutral indexing backends  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Storage/Metadata, Events/Reconciliation, Audit/Compliance, Observability, API/Axum, Dioxus UI, Multi-Tenancy, Runs/Jobs, SCM/Change Proposal, Release, Deployment, Device Lab, RBE, Plugins, and Operations/DR. This subsystem provides fast discovery and analytics without becoming authoritative for domain state.

---

# 1. Purpose

Forgeyard will eventually contain large volumes of:

```text
projects
repositories
change proposals
runs
jobs
attempts
runners
artifacts
packages
releases
deployments
devices
audit records
events
logs
evidence
```

Users need to answer questions such as:

```text
find run 01J...
show failed Android jobs in the last 7 days
find artifacts built from this source snapshot
show releases containing package X
find deployments of release Y
show jobs that ran on runner Z
find all approvals by principal P
show audit records for secret.use
show slowest projects this week
show queue wait by runner pool
```

Relational OLTP queries alone should not be forced to serve every interactive search and analytics workload.

The central rule is:

> **Search indexes and analytics projections are derived views. PostgreSQL/Neon and CAS remain authoritative; search can be rebuilt from authoritative state.**

A second rule is:

> **Search freshness is explicit. A fast indexed result may be slightly stale, but protected actions must always re-read authoritative state before execution.**

A third rule is:

> **Every search and analytics query is tenant-scoped, complexity-bounded, and permission-filtered.**

---

# 2. Architectural Position

```text
                   Authoritative Systems
      ┌───────────────┼────────────────┐
      ▼               ▼                ▼
 PostgreSQL/Neon     CAS         Audit/Event State
      │               │                │
      └───────────────┼────────────────┘
                      ▼
             Projection / Indexer
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
    Search Index   Aggregates   Materialized Views
        │             │             │
        └─────────────┼─────────────┘
                      ▼
                Query Service
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
         API         CLI         Dioxus UI
```

---

# 3. Goals

The subsystem MUST:

1. define search documents;
2. define index identity;
3. support global search;
4. support project-scoped search;
5. support entity lookup;
6. support faceted filtering;
7. support sorting;
8. support saved queries;
9. support indexed audit search;
10. support event search;
11. support operational analytics;
12. support materialized aggregates;
13. support tenant isolation;
14. support authz filtering;
15. support freshness metadata;
16. support event-driven indexing;
17. support reconciliation;
18. support full reindex;
19. support schema versioning;
20. support query limits;
21. support pagination;
22. support large datasets;
23. support index-provider abstraction;
24. support standalone mode;
25. support distributed mode;
26. support analytics retention;
27. support export;
28. support index health;
29. support observability;
30. remain rebuildable.

---

# 4. Non-Goals

Search/indexing does not:

```text
replace PostgreSQL
replace CAS
authorize protected actions
become audit authority
become job state authority
replace observability time-series backend
```

---

# 5. Workspace Structure

```text
crates/search/
├── forgeyard-search/
├── forgeyard-search-model/
├── forgeyard-search-document/
├── forgeyard-search-index-api/
├── forgeyard-search-query/
├── forgeyard-search-parser/
├── forgeyard-search-projection/
├── forgeyard-search-reconcile/
├── forgeyard-search-schema/
├── forgeyard-search-facet/
├── forgeyard-search-saved/
├── forgeyard-search-health/
└── forgeyard-search-testkit/
```

Operational analytics:

```text
crates/analytics/
├── forgeyard-analytics/
├── forgeyard-analytics-model/
├── forgeyard-analytics-projection/
├── forgeyard-analytics-aggregate/
├── forgeyard-analytics-query/
├── forgeyard-analytics-export/
└── forgeyard-analytics-testkit/
```

Provider adapters:

```text
crates/search-adapters/
├── forgeyard-search-postgres/
├── forgeyard-search-tantivy/
└── forgeyard-search-external/
```

Use modules first; split only at real dependency/runtime boundaries.

---

# 6. SearchDocumentId

```rust
pub struct SearchDocumentId(Digest);
```

Derived from:

```text
tenant
entity kind
entity ID
document schema version
```

---

# 7. Search Entity Kinds

```rust
pub enum SearchEntityKind {
    Project,
    Repository,
    ChangeProposal,
    Run,
    Job,
    Attempt,
    Runner,
    Device,
    Artifact,
    Package,
    Release,
    Deployment,
    AuditRecord,
    Evidence,
    Plugin,
    Custom(SearchEntityKindId),
}
```

---

# 8. Search Document

```rust
pub struct SearchDocument {
    pub id: SearchDocumentId,
    pub tenant: TenantId,
    pub kind: SearchEntityKind,
    pub entity_id: SearchEntityId,
    pub title: BoundedString,
    pub text: SearchText,
    pub facets: BTreeMap<SearchFacetKey, SearchFacetValue>,
    pub updated_at: Timestamp,
    pub source_version: EntityVersion,
}
```

---

# 9. SearchText

Sanitized projection only.

---

# 10. No Secret Values

Critical.

---

# 11. No Raw Tokens

Never index:

```text
bearer tokens
API tokens
private keys
secret plaintext
authorization headers
```

---

# 12. Sensitive Data

Index only fields explicitly declared searchable.

---

# 13. Search Schema

Versioned.

---

# 14. SearchSchemaVersion

```rust
pub struct SearchSchemaVersion(u16);
```

---

# 15. Document Builder

Each entity has explicit projection function.

---

# 16. Example

```text
Run
  ↓
RunSearchDocument
```

---

# 17. No Automatic `Serialize`-Everything Indexing

Critical.

---

# 18. Why

Prevents accidental sensitive-field leakage and schema instability.

---

# 19. Source Version

Document records authoritative entity revision/version.

---

# 20. Freshness

Search result can expose:

```text
indexed_at
source_version
freshness
```

---

# 21. Freshness State

```rust
pub enum SearchFreshness {
    Current,
    PotentiallyStale,
    Stale,
    Unknown,
}
```

---

# 22. Search Authority

Never use search result alone to:

```text
approve
release
deploy
delete
change policy
```

---

# 23. Protected Action Flow

```text
search result
  ↓
entity ID
  ↓
authoritative API GET
  ↓
authz/policy
  ↓
action
```

---

# 24. Search Backend Trait

```rust
#[async_trait]
pub trait SearchIndex {
    async fn upsert(
        &self,
        document: SearchDocument,
    ) -> Result<(), SearchIndexError>;

    async fn delete(
        &self,
        tenant: TenantId,
        kind: SearchEntityKind,
        entity_id: SearchEntityId,
    ) -> Result<(), SearchIndexError>;

    async fn search(
        &self,
        request: SearchRequest,
    ) -> Result<SearchPage, SearchIndexError>;
}
```

---

# 25. Provider Neutral

Core search service does not depend on Tantivy/Postgres/external engine types.

---

# 26. Standalone Backend

Recommended:

```text
Tantivy or embedded index
```

if operationally appropriate.

---

# 27. Simple Standalone Alternative

PostgreSQL-like embedded/local structured search may be enough initially.

---

# 28. Distributed Baseline

Can begin with PostgreSQL full-text/trigram/materialized projections.

---

# 29. External Search Engine

Optional later for scale.

---

# 30. No Mandatory Elasticsearch-Like Service

Forgeyard should not require extra infrastructure just to run.

---

# 31. Backend Evolution

```text
Postgres projection
  ↓
Tantivy/internal
  ↓
external distributed search if scale requires
```

without changing domain semantics.

---

# 32. Global Search

Search across permitted entity kinds.

---

# 33. Global Search Request

```rust
pub struct GlobalSearchRequest {
    pub tenant: TenantId,
    pub principal: PrincipalId,
    pub query: SearchExpression,
    pub kinds: BTreeSet<SearchEntityKind>,
    pub limit: PageLimit,
    pub cursor: Option<SearchCursor>,
}
```

---

# 34. Permission Filter

Search results must be filtered by resource visibility.

---

# 35. Best Strategy

Avoid indexing unauthorized private content into a principal-global index.

Index tenant-scoped resources and apply scope/ACL filtering.

---

# 36. High-Cardinality ACLs

Do not embed every PrincipalId into documents if avoidable.

---

# 37. Authorization Scope Facet

Index stable scope:

```text
tenant
organization
project
```

then authz service filters accessible scopes.

---

# 38. Result Recheck

For highly sensitive entity types, re-check authz before returning detail.

---

# 39. Search Query Language

Keep intentionally constrained.

---

# 40. Baseline Syntax

Examples:

```text
failed android
kind:run state:failed
project:forgeyard after:2026-08-01
runner:linux-x64 duration:>10m
```

---

# 41. Search AST

```rust
pub enum SearchExpr {
    Text(SearchTextTerm),
    Filter(SearchFilter),
    And(Vec<SearchExpr>),
    Or(Vec<SearchExpr>),
    Not(Box<SearchExpr>),
}
```

---

# 42. No Arbitrary SQL

Critical.

---

# 43. No Arbitrary Regex by Default

Can cause expensive execution.

---

# 44. Bounded Wildcards

If supported, limit.

---

# 45. Query Complexity

```rust
pub struct SearchComplexity {
    pub terms: u16,
    pub clauses: u16,
    pub depth: u8,
}
```

---

# 46. Max Complexity

Server-configured.

---

# 47. Query Timeout

Mandatory.

---

# 48. Result Limit

Bounded.

---

# 49. Pagination

Cursor-based.

---

# 50. Search Cursor

Opaque, versioned, backend-independent externally.

---

# 51. Cursor Binding

Bind:

```text
tenant
query digest
sort
index version
```

---

# 52. Cursor Expiry

Explicit.

---

# 53. Stable Sort

Tie-break with stable ID.

---

# 54. Sort Options

Allowlist.

Examples:

```text
relevance
updated_at
created_at
duration
```

---

# 55. Facets

Examples:

```text
kind
project
state
platform
runner pool
release channel
environment
severity
```

---

# 56. Facet API

Typed.

---

# 57. No Arbitrary Field Access

Only searchable/indexed fields.

---

# 58. Saved Query

```rust
pub struct SavedQuery {
    pub id: SavedQueryId,
    pub owner: SavedQueryOwner,
    pub name: SavedQueryName,
    pub expression: SearchExpression,
    pub visibility: SavedQueryVisibility,
}
```

---

# 59. SavedQueryOwner

```text
Principal
Project
Tenant
```

---

# 60. Saved Query Visibility

```text
Private
Project
Tenant
```

---

# 61. Saved Query Is Not Alert

Separate.

---

# 62. Alert Integration

A saved query may later feed notification/automation with explicit scheduler/condition semantics.

---

# 63. Index Projection

Authoritative state change:

```text
DB transaction
  ↓
domain event/outbox
  ↓
search projector
  ↓
upsert index
```

---

# 64. At-Least-Once Projection

Upsert idempotent.

---

# 65. Projection Idempotency

Document identity + source version.

---

# 66. Older Event

Must not overwrite newer index document.

---

# 67. Source Version Compare

Reject stale projection.

---

# 68. Deleted Entity

Tombstone/delete projection.

---

# 69. Missed Event

Reconciliation.

---

# 70. Search Reconciler

Scans authoritative entities by updated/version cursor.

---

# 71. Reconcile Goals

```text
missing docs
stale docs
orphan docs
schema-old docs
```

---

# 72. Full Reindex

Supported.

---

# 73. Reindex State

```rust
pub enum ReindexState {
    Planned,
    Building,
    CatchingUp,
    Ready,
    Switching,
    Completed,
    Failed,
}
```

---

# 74. Zero/Low Downtime Reindex

Use new index generation.

---

# 75. Index Generation

```rust
pub struct SearchIndexGeneration(u64);
```

---

# 76. Reindex Flow

```text
create generation N+1
  ↓
bulk snapshot authoritative data
  ↓
replay/catch up changes
  ↓
verify
  ↓
atomic alias/switch
  ↓
retire N
```

---

# 77. No In-Place Destructive Schema Migration

Prefer new generation.

---

# 78. Index Schema Upgrade

New generation.

---

# 79. Failure

Old generation remains active.

---

# 80. Search Health

```text
current generation
projection lag
reconcile lag
document count
failed docs
```

---

# 81. Search Lag

```rust
pub struct SearchLag {
    pub max_event_age: Duration,
    pub pending_documents: u64,
}
```

---

# 82. UI Freshness Banner

If lag high:

```text
Search results may be delayed.
```

---

# 83. Exact Entity Lookup

Use authoritative database/API, not search.

---

# 84. Search for Discovery

Search returns references.

---

# 85. Logs Search

Important distinction.

---

# 86. Job Log Stream

Primary log storage remains Part 17/log subsystem.

---

# 87. Searchable Log Metadata

Index:

```text
job
attempt
stream
time ranges
severity/structured fields
```

---

# 88. Full Log Text Index

Optional.

---

# 89. Why Optional

Could be enormous and sensitive.

---

# 90. Baseline

Support targeted server-side log search against retained logs, not necessarily global full-text index.

---

# 91. Log Search Scope

Exact Run/Job/Attempt first.

---

# 92. Cross-Run Log Search

Enterprise optional.

---

# 93. Secret Redaction

Logs must already be redacted before indexing.

---

# 94. Search Index Does Not Re-Introduce Redacted Secret

Critical.

---

# 95. Audit Search

Audit canonical store remains Part 28.

---

# 96. Audit Search Index

Can accelerate:

```text
actor
action
resource
time
severity
```

---

# 97. Audit Integrity

Search index not used for verification.

---

# 98. Audit Query Fallback

Canonical audit store.

---

# 99. Event Search

Operational event timeline.

---

# 100. Event Index

Searchable significant domain events.

---

# 101. Event Payload

Only safe searchable projection.

---

# 102. Analytics

Separate from full-text search.

---

# 103. Operational Analytics Goals

Examples:

```text
run success rate
queue wait
job duration
cache hit rate
runner utilization
deployment frequency
deployment failure rate
release lead time
device reliability
RBE cache hit rate
```

---

# 104. Analytics Source

Authoritative domain/event records.

---

# 105. Metrics Backend vs Analytics Store

Metrics optimized for operational time series.

Analytics optimized for product/domain aggregations.

---

# 106. Example Difference

Prometheus:

```text
scheduler queue latency over last 5 minutes
```

Analytics:

```text
median queue wait per project over 90 days
```

---

# 107. Analytics Fact

```rust
pub struct AnalyticsFact {
    pub kind: AnalyticsFactKind,
    pub tenant: TenantId,
    pub occurred_at: Timestamp,
    pub dimensions: AnalyticsDimensions,
    pub measures: AnalyticsMeasures,
    pub source: AnalyticsSourceRef,
}
```

---

# 108. Fact Types

```text
RunCompleted
JobCompleted
CacheLookup
RunnerAllocation
ReleasePublished
DeploymentCompleted
DeviceSessionCompleted
```

---

# 109. Idempotency

Source reference unique.

---

# 110. No Duplicate Fact

At-least-once safe.

---

# 111. Dimensions

Low/moderate cardinality:

```text
project
platform class
runner pool
job class
environment class
```

---

# 112. Sensitive Dimensions

Avoid user identity unless specifically required and authorized.

---

# 113. Measures

```text
duration
queue time
CPU time
bytes
count
```

---

# 114. Aggregation Windows

```text
hour
day
week
month
```

---

# 115. Materialized Aggregate

```rust
pub struct AnalyticsAggregate {
    pub key: AnalyticsAggregateKey,
    pub window: TimeWindow,
    pub measures: AggregatedMeasures,
}
```

---

# 116. Rollups

Fine-grained facts can expire after durable rollups if policy.

---

# 117. Rebuildability

Aggregates should be reconstructible from retained facts/domain history where possible.

---

# 118. Historical Limitation

If raw facts expire, old rollups remain but cannot be re-derived fully.

Document.

---

# 119. Analytics Query

```rust
pub struct AnalyticsQuery {
    pub scope: ResourceScope,
    pub metric: AnalyticsMetric,
    pub from: Timestamp,
    pub to: Timestamp,
    pub group_by: Vec<AnalyticsDimension>,
    pub filters: Vec<AnalyticsFilter>,
}
```

---

# 120. Bounded Group-By

Prevent high-cardinality explosion.

---

# 121. Max Time Range

Configurable.

---

# 122. Export

Large analytics export async.

---

# 123. CSV

Appropriate for analytics export.

---

# 124. JSON

API.

---

# 125. Operational KPI Examples

```text
RunSuccessRate
MedianRunDuration
P95QueueWait
CacheHitRate
RunnerUtilization
DeploymentFrequency
DeploymentFailureRate
MeanRecoveryTime
ReleaseLeadTime
```

---

# 126. KPI Definitions

Versioned.

---

# 127. No Ambiguous "DORA" Claim

If DORA-like metrics exposed, define exact formulas and scope.

---

# 128. Analytics Definition Version

```rust
pub struct AnalyticsDefinitionVersion(u16);
```

---

# 129. Metric Evolution

Changing formula creates new definition version.

---

# 130. Saved Dashboard

UI presentation object.

---

# 131. Dashboard

```rust
pub struct AnalyticsDashboard {
    pub id: DashboardId,
    pub owner: DashboardOwner,
    pub widgets: Vec<DashboardWidget>,
}
```

---

# 132. Dashboard Owner

Principal/project/tenant.

---

# 133. Dashboard Does Not Store Authority

Only queries/layout.

---

# 134. Search Result

```rust
pub struct SearchHit {
    pub kind: SearchEntityKind,
    pub entity_id: SearchEntityId,
    pub title: BoundedString,
    pub snippet: Option<SanitizedSnippet>,
    pub facets: SearchHitFacets,
    pub freshness: SearchFreshness,
}
```

---

# 135. Snippets

Escaped/sanitized.

---

# 136. Highlighting

Never emit unsafe HTML.

---

# 137. Dioxus Global Search

Command palette integration.

---

# 138. Search UX

```text
typeahead
recent searches
filters
entity icons
keyboard navigation
```

---

# 139. Typeahead

Debounced.

---

# 140. Minimum Query Length

For expensive full-text.

---

# 141. Recent Searches

Local/private preference.

---

# 142. Search Suggestions

Derived from accessible entities only.

---

# 143. No Cross-Tenant Suggestion Leakage

Critical.

---

# 144. Search Pages

```text
Global Search
Run Search
Artifact Search
Audit Search
Analytics
Saved Queries
```

---

# 145. Search Deep Link

Result to canonical entity route.

---

# 146. Search URL State

Query/filter in URL where safe.

---

# 147. Sensitive Search Terms

Do not persist automatically in shared telemetry.

---

# 148. Query Logging

Sanitize/minimize.

---

# 149. Search Telemetry

Track latency/result count, not raw query text by default.

---

# 150. Search Permissions

```text
search.use
search.saved.manage
analytics.read
analytics.export
search.admin
```

---

# 151. Audit Search

Also requires `audit.read`.

---

# 152. Security Evidence Search

Requires corresponding permissions.

---

# 153. Result-Level Authz

Search service receives accessible scope set.

---

# 154. Large Scope Set

Use organization/project filters.

---

# 155. System Admin Search

Explicit.

---

# 156. Search Index Encryption

At-rest according to deployment.

---

# 157. External Search Provider

If used:

```text
TLS
auth SecretRef
tenant index naming
network policy
backup
```

---

# 158. Provider Credentials

SecretRef.

---

# 159. External Provider Data Residency

Must be configurable.

---

# 160. High-Assurance Deployment

May forbid external search service.

---

# 161. Tantivy/Internal Search

Good privacy/control option for self-hosted.

---

# 162. Distributed Internal Search Complexity

Do not prematurely build a distributed search engine.

---

# 163. Baseline Recommendation

```text
PostgreSQL indexed projections first
+
Tantivy for local/standalone/full-text where useful
```

then external provider only if justified.

---

# 164. Search Service HA

Derived backend can have replicas.

---

# 165. Search Outage

Core CI functionality continues.

---

# 166. Degraded Behavior

```text
global search unavailable
exact entity pages still work
```

---

# 167. Analytics Outage

Does not stop builds/releases.

---

# 168. Protected Policy

Should never depend solely on analytics search result.

---

# 169. Search Backpressure

Indexer queue bounded.

---

# 170. Projection Priority

Critical entity metadata before low-value historical enrichment.

---

# 171. Reindex Throttle

Do not saturate DB/CAS.

---

# 172. Bulk Read

Use paged snapshots.

---

# 173. DB Replica

Analytics/reindex may use read replica where consistency allows.

---

# 174. Read Replica Freshness

Explicit.

---

# 175. No Long Locks

Reindex avoids blocking OLTP.

---

# 176. Search Tombstone

Deleted/inaccessible entity removed quickly.

---

# 177. Security Deletion Priority

Permission/tenant membership changes should invalidate visibility fast.

---

# 178. Search ACL Drift

Reconciler checks.

---

# 179. Authz Change

If index stores scope only, runtime authz immediately reflects new access even before reindex.

---

# 180. Advantage

Prefer runtime authz filtering over embedded principal ACL copies.

---

# 181. Tenant Closure

Delete/archive tenant index generation according to retention.

---

# 182. Legal Hold

Audit evidence search index can be rebuilt; canonical held audit remains source.

---

# 183. Search Backup

Derived index backup optional.

---

# 184. DR

Can rebuild.

---

# 185. Fast DR

Snapshot index can reduce recovery time but not required for correctness.

---

# 186. Search Snapshot

Versioned per generation.

---

# 187. Index Restore

Verify generation/schema, then catch up from authoritative state.

---

# 188. Analytics Backup

Aggregates may be backed up if expensive to reconstruct.

---

# 189. Raw Fact Retention

Policy.

---

# 190. Search Reconciliation Cursor

```rust
pub struct SearchReconcileCursor {
    pub entity_kind: SearchEntityKind,
    pub updated_after: Timestamp,
    pub last_id: Option<SearchEntityId>,
}
```

---

# 191. Search Error Model

```rust
pub enum SearchError {
    InvalidQuery,
    QueryTooComplex,
    Unauthorized,
    BackendUnavailable,
    CursorExpired,
    IndexStale,
    Internal,
}
```

---

# 192. Analytics Error Model

```text
InvalidMetric
UnsupportedGrouping
RangeTooLarge
BackendUnavailable
```

---

# 193. API

Potential:

```text
GET  /v1/search
GET  /v1/search/suggestions
GET  /v1/saved-queries
POST /v1/saved-queries
GET  /v1/analytics/query
POST /v1/analytics/exports
GET  /v1/admin/search/health
POST /v1/admin/search/reindex
```

---

# 194. Search Query HTTP

Use encoded query params for simple queries.

POST query endpoint for complex typed body if needed.

---

# 195. Analytics Query

POST is reasonable for structured queries.

---

# 196. ETag

Saved queries/dashboards mutable via optimistic concurrency.

---

# 197. Admin Reindex

High operational action.

---

# 198. Reindex Does Not Modify Domain Data

Still audited.

---

# 199. Index Generation Switch

Atomic metadata pointer.

---

# 200. Search Health UI

Shows:

```text
backend
generation
lag
document count
last reconcile
reindex state
```

---

# 201. Analytics UI

Charts/tables.

---

# 202. No Chart-Only Meaning

Accessibility table equivalent.

---

# 203. Query Builder

Typed fields.

---

# 204. Saved Filters

Project/team.

---

# 205. Operational Analytics Dashboard

Examples:

```text
build health
queue performance
runner utilization
cache efficiency
release/deployment throughput
device reliability
```

---

# 206. Run Analytics

```text
success/failure by branch/source trust
duration distribution
retry rate
```

---

# 207. Scheduler Analytics

```text
queue wait
placement failures
capacity pressure
scarcity
```

---

# 208. Runner Analytics

```text
utilization
lost attempts
reliability
```

---

# 209. CAS Analytics

```text
hit rate
bytes
replication lag
GC
```

---

# 210. Release Analytics

```text
candidate-to-release lead time
publication failures
```

---

# 211. Deployment Analytics

```text
frequency
failure rate
rollback rate
health-gate failures
```

---

# 212. Device Analytics

```text
device infra failure
quarantine
test duration
```

---

# 213. RBE Analytics

```text
cache hit
execution queue
CAS transfer
```

---

# 214. Governance Analytics

Use Part 27 usage store rather than duplicating.

---

# 215. Billing Analytics

Part 30 owns commercial views.

---

# 216. Analytics Composition

Can join derived data at query layer where safe.

---

# 217. No Giant Data Warehouse Baseline

Avoid unnecessary infrastructure.

---

# 218. Warehouse Export

Future optional.

---

# 219. Export Sink

Object storage/CSV/Parquet later.

---

# 220. Parquet

Useful for analytics interoperability if added.

---

# 221. JSON/RON Preference

Internal config remains RON/Postcard.

Parquet/CSV/JSON are appropriate external analytics formats.

---

# 222. Search Event

```text
SearchDocumentUpdated
SearchDocumentDeleted
IndexGenerationActivated
```

operational events.

---

# 223. Analytics Event

```text
AnalyticsFactRecorded
AggregateUpdated
```

not business authority.

---

# 224. Metrics

```text
search_query_total
search_query_latency_seconds
search_projection_lag_seconds
search_reconcile_failures_total
search_documents
analytics_query_latency_seconds
analytics_projection_lag_seconds
analytics_export_failures_total
```

---

# 225. Metric Labels

Low cardinality:

```text
entity_kind
query_class
backend
result
```

---

# 226. No Query Text Metrics

Critical.

---

# 227. Tracing

```text
search.query
search.index.upsert
search.reconcile
search.reindex
analytics.fact
analytics.aggregate
analytics.query
```

---

# 228. Doctor

```text
forgeyard search doctor
forgeyard analytics doctor
```

---

# 229. Search Doctor Checks

```text
backend connectivity
generation
lag
stale docs
orphan sample
schema compatibility
```

---

# 230. Analytics Doctor

```text
fact lag
aggregate lag
retention
query backend
```

---

# 231. CLI

```text
forgeyard search "<query>"
forgeyard search --kind run
forgeyard analytics run-health
forgeyard analytics queue
forgeyard search reindex --dry-run
```

---

# 232. Dry Run

Estimate reindex scope.

---

# 233. Query Explain

Admin/developer:

```text
forgeyard search explain
```

shows parsed AST, filters, estimated complexity.

---

# 234. Not Raw Backend Query

Avoid leaking implementation.

---

# 235. Saved Query Notification

Future integration:

```text
saved query
  +
condition
  ↓
automation/notification
```

not implicit.

---

# 236. Search Autocomplete

Prefix index.

---

# 237. Entity IDs

Exact ID lookup route can bypass full-text.

---

# 238. Search Ranking

Simple predictable ranking first.

---

# 239. Ranking Inputs

```text
exact ID
exact name
prefix
text relevance
recency
```

---

# 240. No Opaque ML Ranking Baseline

Predictability > novelty.

---

# 241. Search Ranking Security

Do not rank unauthorized content before filtering if backend could leak counts/snippets.

---

# 242. Facet Count Security

Counts must respect permissions.

---

# 243. Side-Channel

Avoid showing total hit counts including inaccessible documents.

---

# 244. Search Result Count

Authorized only.

---

# 245. Search Snippet

Authorized fields only.

---

# 246. Deleted Permission

Search index may lag, but runtime authz blocks result detail.

---

# 247. Better

Runtime result filter can remove inaccessible hit immediately.

---

# 248. Result Refill

If filtering removes hits, fetch extra backend hits to fill page within bounded effort.

---

# 249. Query Budget

Cap backend fetch multiplier.

---

# 250. Project Search

Cheaper because scope known.

---

# 251. Global Search

Potentially more expensive; stricter limits.

---

# 252. Search Rate Limit

Part 18/API.

---

# 253. Analytics Rate Limit

Cost-aware.

---

# 254. Query Cost Class

```rust
pub enum QueryCostClass {
    Cheap,
    Normal,
    Expensive,
}
```

---

# 255. Admin Analytics

Can use async export for expensive requests.

---

# 256. Query Cache

Safe for identical tenant/query/authz-scope snapshot.

---

# 257. Cache Key

```text
tenant
query digest
scope digest
index generation
```

---

# 258. Short TTL

Because search freshness changes.

---

# 259. No Cross-Tenant Query Cache

Critical.

---

# 260. Analytics Cache

Can cache immutable past windows longer.

---

# 261. Current Window

Short TTL.

---

# 262. Search Indexer Identity

System service principal.

---

# 263. Least Privilege

Read only required authoritative projections + write index.

---

# 264. External Search Credentials

SecretRef.

---

# 265. Indexer Cannot Mutate Domain

Critical.

---

# 266. Search Provider Plugin

Part 24 could support trusted search backend adapter later.

---

# 267. Sandboxed Third-Party Search Plugin

Sensitive because it receives indexed data.

Require trusted external at minimum.

---

# 268. External Search Data Exposure

Explicit enterprise policy.

---

# 269. Analytics Provider Plugin

Could export aggregates.

---

# 270. No Raw Secret/Data Exfiltration

Projection allowlist.

---

# 271. Data Classification

Searchable fields can carry classification:

```text
Public
Internal
Sensitive
Restricted
```

---

# 272. Index Backend Eligibility

Policy can disallow Restricted data in external provider.

---

# 273. Field Classification

```rust
pub struct SearchFieldDefinition {
    pub key: SearchFieldKey,
    pub classification: DataClassification,
    pub indexed: bool,
    pub stored: bool,
}
```

---

# 274. Stored vs Indexed

Some field can support filtering without returning original value.

---

# 275. Searchable Hash

For exact sensitive lookup, keyed hash could be used in special cases.

Not baseline.

---

# 276. Privacy

Avoid indexing email unless necessary.

---

# 277. Principal Search

Identity directory handled by Contacts/Identity APIs, not generic global search by default.

---

# 278. Audit Actor Search

Permission-protected.

---

# 279. Retention

Search docs removed when source beyond retention/deleted.

---

# 280. Analytics Retention

Configurable.

---

# 281. Legal Hold

Canonical source determines preservation.

Derived index can be rebuilt.

---

# 282. Reindex Security

New schema undergoes field-classification review.

---

# 283. Architecture Check

Prevent accidental indexing of secret-bearing types.

---

# 284. `SearchProjectable` Trait

Explicit implementation.

---

# 285. Avoid Blanket Trait

No `impl<T: Serialize> SearchProjectable`.

---

# 286. Example

```rust
pub trait SearchProjectable {
    fn to_search_document(
        &self,
        context: SearchProjectionContext,
    ) -> SearchDocument;
}
```

---

# 287. Projection Context

Includes tenant/scope and safe link generation.

---

# 288. Analytics Projectable

Explicit facts emitted from state transitions.

---

# 289. Testkit

```text
forgeyard-search-testkit/src/
├── lib.rs
├── documents.rs
├── query.rs
├── index.rs
├── reconcile.rs
├── authz.rs
└── assertions.rs
```

Analytics:

```text
forgeyard-analytics-testkit/
```

---

# 290. Unit Tests

Parser/AST/facets/cursors.

---

# 291. Secret Leakage Test

Secret field never enters search doc.

---

# 292. Cross-Tenant Search Test

Tenant A gets no Tenant B hit/count/facet.

---

# 293. Authz Revocation Test

Principal loses project access; search result no longer returned even if index stale.

---

# 294. Stale Event Test

Older projection event cannot overwrite newer doc.

---

# 295. Missing Event Test

Reconciler repairs document.

---

# 296. Orphan Document Test

Reconciler removes.

---

# 297. Reindex Test

N+1 built/caught-up/switched without query outage.

---

# 298. Reindex Failure Test

N remains active.

---

# 299. Cursor Test

Cursor bound to query/generation.

---

# 300. Cursor Expiry Test

Explicit reset.

---

# 301. Query Complexity Test

Pathological nested query rejected.

---

# 302. Regex/Empty Query Abuse Test

Bounded.

---

# 303. Analytics Dedup Test

Duplicate event yields one fact.

---

# 304. Aggregate Rebuild Test

Correct.

---

# 305. KPI Version Test

Formula version explicit.

---

# 306. Search Outage Test

Exact domain APIs still work.

---

# 307. Analytics Outage Test

CI execution unaffected.

---

# 308. DR Test

Index rebuilt from restored authority.

---

# 309. External Backend Failure

System degrades gracefully.

---

# 310. Fuzzing

Fuzz:

```text
query parser
cursor decoder
facet parser
search document decoder
```

---

# 311. Load Test

Millions of documents.

---

# 312. Search Latency Test

P50/P95.

---

# 313. Facet Scale Test

Large project counts.

---

# 314. Analytics Scale Test

Long windows/rollups.

---

# 315. Reindex Load Test

Does not overload primary DB.

---

# 316. Implementation Phase 1 — Search Model & Projectors

Core typed documents.

---

# 317. Phase 2 — PostgreSQL Search Baseline

FTS/trigram/indexed projections.

---

# 318. Phase 3 — Global Search API/UI

Projects/runs/releases/deployments.

---

# 319. Phase 4 — Reconciliation/Reindex

Operational safety.

---

# 320. Phase 5 — Saved Queries/Facets

Power-user UX.

---

# 321. Phase 6 — Audit/Event Search

Permission-sensitive.

---

# 322. Phase 7 — Operational Analytics Facts

Run/job/scheduler.

---

# 323. Phase 8 — Aggregates/Dashboards

Longer-term trends.

---

# 324. Phase 9 — Tantivy/Embedded Full-Text

Standalone/performance.

---

# 325. Phase 10 — External Search Adapter

Only if needed.

---

# 326. Phase 11 — Advanced Log Search

Optional.

---

# 327. Phase 12 — Scale/Security/DR Hardening

Production.

---

# 328. Acceptance Tests

1. Search index is derived, not authoritative.
2. Protected actions always re-read authoritative state.
3. Every search document is tenant scoped.
4. Secret values/tokens/private keys never enter search documents.
5. Search projection is explicit rather than generic serialization.
6. Tenant A receives no Tenant B search hit.
7. Tenant A receives no Tenant B facet/count leakage.
8. Runtime authz blocks stale-index access after permission revocation.
9. Search query language cannot inject SQL/backend syntax.
10. Query complexity is bounded.
11. Pagination cursors bind query/sort/index generation.
12. Search events are at-least-once/idempotent.
13. Older projection updates cannot overwrite newer document state.
14. Missing index updates are repaired by reconciliation.
15. Orphan documents are removed.
16. Full reindex builds a new generation before switch.
17. Failed reindex leaves prior generation usable.
18. Search outage does not stop CI/domain APIs.
19. Exact ID lookup can use authoritative store directly.
20. Audit index never replaces canonical audit verification.
21. Log indexing never reintroduces redacted secrets.
22. Analytics facts are idempotent.
23. Analytics metrics/formulas are versioned.
24. Metrics/telemetry are not silently treated as analytics authority.
25. External search provider is optional.
26. Standalone does not require external search infrastructure.
27. DR can rebuild search from restored authoritative state.
28. Search provider credentials use SecretRef.
29. High-assurance mode can prohibit external indexing of restricted fields.
30. Search query logging does not capture raw sensitive terms by default.
31. Saved queries are tenant/visibility scoped.
32. Analytics exports are bounded/asynchronous for expensive ranges.
33. Search/analytics health exposes freshness/lag.
34. Plugin/provider paths cannot bypass field-classification rules.
35. Forgeyard dogfoods search and analytics for its own operations.

---

# 329. Production Readiness Gates

Do not call search/indexing/analytics production-ready until:

```text
explicit projection schemas are stable
secret-field leakage tests pass
tenant/count/facet isolation passes
runtime authz filtering is enforced
projection reconciliation works
generation-based reindex works
query complexity/cursor limits are hardened
search outage degrades safely
analytics fact dedup works
DR rebuild is tested
```

---

# 330. Architectural Invariants

1. search is derived state;
2. analytics is derived state;
3. PostgreSQL/Neon/CAS remain authoritative;
4. protected actions never trust index state alone;
5. every document carries tenant scope;
6. secret values never enter indexes;
7. projection is explicit and allowlisted;
8. query language is constrained;
9. query complexity is bounded;
10. cursor state is opaque/versioned;
11. runtime authz prevents stale-index permission leaks;
12. facet/count results respect permissions;
13. projection delivery is at-least-once;
14. source version prevents stale overwrite;
15. reconciliation repairs missing/orphan documents;
16. schema migration uses new index generation;
17. reindex failure does not destroy active generation;
18. external search infrastructure is optional;
19. search outage never stops execution correctness;
20. audit search never replaces audit integrity verification;
21. analytics definitions are versioned;
22. metrics are not analytics authority;
23. raw facts/aggregates use idempotent source identity;
24. sensitive queries are not logged verbatim by default;
25. external search credentials are scoped SecretRefs;
26. field classification controls external indexing eligibility;
27. standalone/distributed share search semantics;
28. DR can rebuild indexes;
29. plugins cannot bypass projection/classification rules;
30. Forgeyard dogfoods its search and operational analytics system.

---

# 331. Final Target Architecture

```text
                Authoritative Forgeyard State
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
             DB          CAS       Audit/Events
              │           │           │
              └───────────┼───────────┘
                          ▼
                  Projection Layer
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
          Search Docs   Facts      Aggregates
              │           │           │
              └───────────┼───────────┘
                          ▼
                    Query Service
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
             API         CLI         Dioxus
```

---

# 332. Final Architectural Position

Search:

```text
authoritative entity
  ↓
explicit safe projection
  ↓
tenant-scoped index
  ↓
search result
  ↓
authoritative entity reload before protected action
```

Analytics:

```text
domain transition
  ↓
idempotent analytics fact
  ↓
rollup/materialized aggregate
  ↓
bounded operational query
```

Reindex:

```text
generation N active
  ↓
build N+1
  ↓
catch up
  ↓
verify
  ↓
atomic switch
  ↓
retire N
```

The key guarantee is:

> **Forgeyard can provide fast, expressive discovery and long-term operational insight without compromising correctness. Search and analytics are rebuildable projections over authoritative Forgeyard state, every result remains tenant- and permission-scoped, and stale indexes can affect convenience but never the correctness of protected actions.**

---

# 333. Extended Architecture Sequence

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
```
