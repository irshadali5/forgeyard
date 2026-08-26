# 37 — Forgeyard Static Analysis, Code Quality, Security Scanning & Findings Management System Architecture

**Document type:** Core Static Analysis, Code Quality, Security Scanning, Finding Lifecycle & Remediation System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** SAST, linters, type checking, code-quality analyzers, secret scanning, dependency/source security analysis, DAST integration hooks, normalized findings, deduplication, suppression, baselines, severity/confidence, remediation state, ownership, SCM annotations, quality/security policy gates, evidence retention, and scanner-provider integration  
**Architecture style:** Analyzer-neutral, evidence-first, immutable observations, normalized finding identity, explicit source/artifact binding, policy-controlled gating, auditable suppressions, versioned scanner semantics, and no scanner becoming authorization or release authority  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Pipeline IR, Run/Job/Attempt, Test Intelligence, Dependency Governance, Supply Chain, Change Proposal, Search/Analytics, Audit, Notifications, Policy/Authz, Monorepo Intelligence, and Release/Deployment. It centralizes the static/security analysis concepts referenced elsewhere into one coherent subsystem.

---

# 1. Purpose

Forgeyard already builds, tests, packages, signs, and releases software.

Production CI/CD also needs to answer:

```text
did the compiler/type checker find issues?
did a linter find unsafe or suspicious code?
did static analysis identify a vulnerability?
was a secret committed?
did code quality regress?
is this finding new or pre-existing?
is it a duplicate?
was it suppressed?
who approved that suppression?
is the issue fixed in this exact revision?
does this finding block merge/release?
```

Without a normalized findings subsystem, every scanner becomes its own silo:

```text
cargo clippy output
Semgrep JSON
CodeQL SARIF
Trivy findings
secret scanner output
custom analyzer logs
compiler warnings
```

The central rule is:

> **Analyzers produce evidence. Forgeyard normalizes, stores, correlates, and evaluates that evidence; analyzers themselves never decide merge, release, deployment, or authorization.**

A second rule is:

> **A finding is always bound to exact analyzed subject identity—source snapshot, artifact, package, image, or deployment target—not merely a branch name.**

A third rule is:

> **Suppressions never delete or rewrite findings. They create auditable policy/context overlays with scope, reason, owner, and expiry.**

---

# 2. Architectural Position

```text
                     Analysis Job
                         │
                         ▼
                  Analyzer Output
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
          SAST         Lint/Type     Secrets
            │            │            │
            └────────────┼────────────┘
                         ▼
                 Finding Normalizer
                         │
             ┌───────────┼───────────┐
             ▼           ▼           ▼
        Deduplicate   Correlate   Classify
             │           │           │
             └───────────┼───────────┘
                         ▼
                    Finding Store
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
         Baseline     Suppression   Ownership
                         │
                         ▼
                    Policy Facts
                         │
                         ▼
                Merge / Release Gate
```

---

# 3. Goals

The subsystem MUST:

1. define analyzer identity;
2. define scanner versions;
3. normalize findings;
4. support SAST;
5. support linters;
6. support compiler/type-check findings;
7. support secret scanning;
8. support dependency/code security findings;
9. support artifact/image findings;
10. support DAST evidence integration;
11. preserve raw reports;
12. define finding identity;
13. deduplicate findings;
14. correlate recurring findings;
15. distinguish new/existing findings;
16. support severity;
17. support confidence;
18. support ownership;
19. support suppression;
20. support suppression expiry;
21. support baseline comparisons;
22. support remediation lifecycle;
23. support SCM annotations;
24. support policy gates;
25. support release evidence;
26. support notifications;
27. support audit;
28. support search/analytics;
29. support multi-tenancy;
30. remain rebuildable/reprocessable.

---

# 4. Non-Goals

This subsystem does not:

```text
replace rustc/clippy/Semgrep/CodeQL/etc.
replace vulnerability databases
replace test execution
replace policy engine
replace incident response
guarantee every scanner is correct
```

---

# 5. Workspace Structure

```text
crates/analysis/
├── forgeyard-analysis/
├── forgeyard-analysis-model/
├── forgeyard-analysis-ingest/
├── forgeyard-analysis-normalize/
├── forgeyard-analysis-finding/
├── forgeyard-analysis-dedup/
├── forgeyard-analysis-baseline/
├── forgeyard-analysis-suppression/
├── forgeyard-analysis-ownership/
├── forgeyard-analysis-quality/
├── forgeyard-analysis-security/
├── forgeyard-analysis-health/
└── forgeyard-analysis-testkit/
```

Scanner adapters:

```text
crates/analysis-adapters/
├── forgeyard-analysis-sarif/
├── forgeyard-analysis-clippy/
├── forgeyard-analysis-rustc/
├── forgeyard-analysis-semgrep/
├── forgeyard-analysis-codeql/
├── forgeyard-analysis-secret-scan/
├── forgeyard-analysis-trivy/
├── forgeyard-analysis-custom/
└── ...
```

Use modules first; split only when parser/provider/tool dependencies justify.

---

# 6. AnalyzerId

```rust
pub struct AnalyzerId(Digest);
```

Logical analyzer identity.

---

# 7. AnalyzerVersion

```rust
pub struct AnalyzerVersion(BoundedString);
```

---

# 8. Analyzer Descriptor

```rust
pub struct AnalyzerDescriptor {
    pub id: AnalyzerId,
    pub name: AnalyzerName,
    pub version: AnalyzerVersion,
    pub kind: AnalyzerKind,
    pub ruleset: AnalyzerRulesetId,
}
```

---

# 9. Analyzer Kind

```rust
pub enum AnalyzerKind {
    Compiler,
    TypeChecker,
    Linter,
    StaticSecurity,
    SecretScanner,
    DependencySecurity,
    ArtifactSecurity,
    DynamicSecurity,
    License,
    CodeQuality,
    Custom(AnalyzerKindId),
}
```

---

# 10. Ruleset Identity

```rust
pub struct AnalyzerRulesetId(Digest);
```

Includes:

```text
rule configuration
rule versions
enabled/disabled rules
severity overrides
```

---

# 11. Why Ruleset Identity Matters

Same scanner version with different rules is different evidence.

---

# 12. AnalysisSubject

```rust
pub enum AnalysisSubject {
    SourceSnapshot(SourceSnapshotId),
    Artifact(ArtifactId),
    Package(PackageId),
    OciImage(CasObjectRef),
    ReleaseCandidate(ReleaseCandidateId),
    DeploymentRevision(DeploymentRevisionId),
}
```

---

# 13. Exact Subject Binding

Critical.

---

# 14. AnalysisRunId

```rust
pub struct AnalysisRunId(Ulid);
```

One logical analyzer execution.

---

# 15. Analysis Run

```rust
pub struct AnalysisRun {
    pub id: AnalysisRunId,
    pub analyzer: AnalyzerDescriptor,
    pub subject: AnalysisSubject,
    pub job_attempt: JobAttemptId,
    pub raw_report: CasObjectRef,
    pub state: AnalysisRunState,
}
```

---

# 16. Analysis Run State

```rust
pub enum AnalysisRunState {
    Pending,
    Processing,
    Complete,
    InvalidEvidence,
    Failed,
}
```

---

# 17. Raw Report Preservation

Raw scanner output stored in CAS according to retention.

---

# 18. Normalizer Version

```rust
pub struct AnalysisNormalizerVersion(u16);
```

---

# 19. Reprocessing

Scanner report may be re-normalized later without rewriting original evidence.

---

# 20. FindingObservationId

```rust
pub struct FindingObservationId(Ulid);
```

---

# 21. FindingLogicalId

```rust
pub struct FindingLogicalId(Digest);
```

Represents a recurring logical issue across revisions where correlation is sufficiently stable.

---

# 22. Finding Observation

```rust
pub struct FindingObservation {
    pub id: FindingObservationId,
    pub logical_id: FindingLogicalId,
    pub analysis_run: AnalysisRunId,
    pub subject: AnalysisSubject,
    pub rule: FindingRuleRef,
    pub location: Option<FindingLocation>,
    pub severity: FindingSeverity,
    pub confidence: FindingConfidence,
    pub message: SanitizedFindingMessage,
    pub fingerprint: FindingFingerprint,
    pub status: FindingObservationStatus,
}
```

---

# 23. Observation Immutability

Once normalized, do not mutate.

---

# 24. Finding Observation Status

```rust
pub enum FindingObservationStatus {
    Present,
    ResolvedInSubject,
    Unknown,
}
```

---

# 25. Logical Finding Lifecycle

Separate from observation.

---

# 26. FindingLifecycleState

```rust
pub enum FindingLifecycleState {
    Open,
    Acknowledged,
    Suppressed,
    Remediated,
    Reopened,
}
```

---

# 27. Why Separate

Observation is evidence.

Lifecycle is human/system workflow.

---

# 28. Finding Rule

```rust
pub struct FindingRuleRef {
    pub analyzer: AnalyzerId,
    pub rule_id: BoundedString,
    pub rule_version: Option<BoundedString>,
}
```

---

# 29. Stable Rule ID

Use analyzer-native ID when stable.

---

# 30. SARIF

Natural interchange format for many analyzers.

---

# 31. SARIF Adapter

Can ingest:

```text
tool
rules
results
locations
fingerprints
code flows
```

---

# 32. SARIF Is External Format

JSON is appropriate.

---

# 33. Core Model

Not tied to SARIF.

---

# 34. Finding Severity

```rust
pub enum FindingSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}
```

---

# 35. Native Severity

Preserve analyzer-native severity separately.

---

# 36. Severity Mapping

Versioned per adapter/ruleset.

---

# 37. Confidence

```rust
pub enum FindingConfidence {
    Low,
    Medium,
    High,
    Confirmed,
    Unknown,
}
```

---

# 38. Severity != Confidence

Critical distinction.

---

# 39. Example

High severity + Low confidence.

---

# 40. Finding Category

```rust
pub enum FindingCategory {
    Correctness,
    Reliability,
    Maintainability,
    Security,
    SecretExposure,
    License,
    Performance,
    Style,
    Custom(FindingCategoryId),
}
```

---

# 41. Security Taxonomy

Optional mappings:

```text
CWE
OWASP category
CVE where applicable
```

---

# 42. Taxonomy Is Metadata

Not finding identity alone.

---

# 43. Finding Location

```rust
pub struct FindingLocation {
    pub path: RepoRelativePath,
    pub region: Option<SourceRegion>,
    pub symbol: Option<SymbolRef>,
}
```

---

# 44. Artifact Finding

May have no source path.

---

# 45. Binary Location

Can use artifact component/path offset metadata.

---

# 46. Path Validation

Repo-relative only for source.

---

# 47. Finding Message

Sanitized.

---

# 48. Secret Scanner

Never store actual discovered secret value.

---

# 49. Secret Finding

Store:

```text
secret type
location
fingerprint/hash
confidence
```

---

# 50. Secret Fingerprint

Keyed/hash representation where needed.

---

# 51. No Plaintext Secret

Critical.

---

# 52. Scanner Log

Must redact detected secret.

---

# 53. Secret Remediation

Rotate/revoke actual credential via Part 12 workflows.

---

# 54. Finding Fingerprint

```rust
pub struct FindingFingerprint(Digest);
```

---

# 55. Fingerprint Inputs

Potential:

```text
rule
normalized path
symbol
context hash
analyzer-native fingerprint
```

---

# 56. Deduplication

Same logical issue reported by repeated runs should correlate.

---

# 57. Cross-Analyzer Dedup

Harder.

---

# 58. Baseline

Do not aggressively merge different analyzers.

---

# 59. Correlation Confidence

```rust
pub enum FindingCorrelationConfidence {
    Exact,
    Strong,
    Heuristic,
    Unknown,
}
```

---

# 60. Exact

Analyzer stable fingerprint.

---

# 61. Strong

Rule+symbol+context.

---

# 62. Heuristic

Presentation only unless policy allows.

---

# 63. No Silent Merge of Heuristic Findings

Critical.

---

# 64. New Finding

A logical finding absent in baseline subject.

---

# 65. Existing Finding

Present in baseline and candidate.

---

# 66. Resolved Finding

Present in baseline but absent in candidate, assuming comparable analysis.

---

# 67. Baseline Comparability

Must match:

```text
analyzer version/ruleset compatibility
subject type
scope
configuration
```

---

# 68. Baseline Subject

Exact immutable identity.

---

# 69. Baseline Selection

```rust
pub enum FindingBaselineSelection {
    Explicit(AnalysisSubject),
    TargetBranchLatestAccepted,
    LastStableRelease,
}
```

---

# 70. Mutable Ref Resolution

Resolve to exact subject before comparison.

---

# 71. FindingDiff

```rust
pub struct FindingDiff {
    pub new: Vec<FindingLogicalId>,
    pub existing: Vec<FindingLogicalId>,
    pub resolved: Vec<FindingLogicalId>,
    pub unknown: Vec<FindingObservationId>,
}
```

---

# 72. Unknown

Used when analysis not comparable/incomplete.

---

# 73. Do Not Call Missing Finding Resolved If Scanner Failed

Critical.

---

# 74. Analysis Completeness

```rust
pub enum AnalysisCompleteness {
    Complete,
    Partial,
    Failed,
    Unknown,
}
```

---

# 75. Partial

Some modules/files skipped.

---

# 76. Protected Gate

May require Complete.

---

# 77. Incremental Analysis

Part 34 may scan affected subset.

---

# 78. Scope Must Be Explicit

```rust
pub enum AnalysisScope {
    Full,
    AffectedOnly(AffectedSetId),
    Paths(Vec<RepoRelativePath>),
    ArtifactSubset(Vec<ArtifactComponentRef>),
}
```

---

# 79. Full Release Scan

Recommended for Stable.

---

# 80. PR Scan

Affected-only possible when policy accepts.

---

# 81. Scope Included in AnalysisRun Identity

Critical.

---

# 82. Static Analysis Execution

Runs as normal Forgeyard Job.

---

# 83. Analyzer Container/Toolchain

Pinned.

---

# 84. Network

Deny by default for source analyzers unless scanner requires remote service.

---

# 85. Remote SaaS Analyzer

Possible adapter.

---

# 86. Data Egress Policy

Explicit.

---

# 87. High-Assurance Mode

Can forbid source upload to external scanner.

---

# 88. Analyzer Trust Class

```rust
pub enum AnalyzerExecutionMode {
    LocalTrustedTool,
    SandboxedTool,
    RemoteService,
    Plugin,
}
```

---

# 89. Analyzer Credentials

SecretRef.

---

# 90. Source Upload

Exact, policy-authorized.

---

# 91. Minimal Data

Upload only required content.

---

# 92. Remote Result

Still normalized and independently policy-evaluated.

---

# 93. Compiler Diagnostics

Can be normalized as findings.

---

# 94. rustc

Warnings/errors.

---

# 95. Errors

Usually job failure itself.

---

# 96. Warnings

Can become quality findings.

---

# 97. Clippy

First-class Rust linter adapter.

---

# 98. Clippy Rule ID

Use lint name.

---

# 99. Deny Warnings

Policy/build config controls.

---

# 100. Formatting

`cargo fmt --check` is usually check outcome, not finding-heavy.

Can still report file-level findings.

---

# 101. Type Checkers

Examples:

```text
mypy
pyright
tsc
```

---

# 102. Code Quality

Examples:

```text
complexity
dead code
unsafe usage
duplication
API lint
architecture rules
```

---

# 103. Architecture Findings

Part 34 architecture checks can emit normalized findings.

---

# 104. Unsafe Rust

Potential custom quality/security rule.

---

# 105. Secret Scanning

Scan:

```text
source snapshot
diff
commits/history optionally
artifacts optionally
```

---

# 106. Diff Secret Scan

Fast PR gate.

---

# 107. Full Secret Scan

Release/nightly/security baseline.

---

# 108. History Scan

High-cost optional.

---

# 109. Secret Revocation Workflow

Finding -> notification -> secret owner -> rotation.

---

# 110. Dependency Security

Part 36/13 findings can map into common Finding model where useful.

---

# 111. Avoid Duplicate Vulnerability Authority

Canonical vulnerability evidence remains Part 13.

---

# 112. Findings UI Can Project Both

Unified experience.

---

# 113. Artifact Security

Binary/image scanner.

---

# 114. DAST

Dynamic Application Security Testing.

---

# 115. DAST Target

Exact DeploymentRevisionId or preview environment.

---

# 116. DAST Safety

Only authorized Forgeyard-managed targets.

---

# 117. No Arbitrary Internet Scanning

Critical.

---

# 118. DAST Permission

```text
analysis.dast.run
```

---

# 119. Production DAST

Separate high-risk permission/policy.

---

# 120. DAST Finding

Bind exact deployment/release target.

---

# 121. DAST Tool

Normal job/load-like sandbox.

---

# 122. Scan Rate Limits

Protect target.

---

# 123. Suppression

Never edit finding.

---

# 124. FindingSuppressionId

```rust
pub struct FindingSuppressionId(Ulid);
```

---

# 125. Suppression Record

```rust
pub struct FindingSuppression {
    pub id: FindingSuppressionId,
    pub finding: FindingLogicalId,
    pub scope: SuppressionScope,
    pub reason: BoundedString,
    pub created_by: PrincipalId,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub tracking_ref: Option<ExternalTrackingRef>,
}
```

---

# 126. Suppression Scope

```rust
pub enum SuppressionScope {
    ExactFinding,
    RuleForPath,
    RuleForProject,
    AnalyzerRule,
}
```

---

# 127. Broad Suppression

Higher permission.

---

# 128. Expiry

Required/recommended.

---

# 129. Permanent Suppression

Strongly discouraged.

---

# 130. Suppression Reason

Mandatory.

---

# 131. Suppression Audit

Mandatory.

---

# 132. Suppressed Finding

Still visible.

---

# 133. UI Label

```text
Suppressed until ...
```

---

# 134. Suppression Does Not Change Severity

Critical.

---

# 135. Suppression Policy

Policy decides whether suppressed findings count toward gate.

---

# 136. Security Critical

May disallow suppression entirely.

---

# 137. Suppression Expiry

Durable timer/reevaluation.

---

# 138. False Positive

Can be suppression reason category.

---

# 139. Accepted Risk

Separate reason category.

---

# 140. Compensating Control

Separate.

---

# 141. SuppressionReasonKind

```rust
pub enum SuppressionReasonKind {
    FalsePositive,
    AcceptedRisk,
    CompensatingControl,
    TemporaryException,
    ToolLimitation,
}
```

---

# 142. Risk Acceptance

Can require security approver.

---

# 143. Finding Ownership

Derived from source ownership/component.

---

# 144. FindingOwner

```rust
pub struct FindingOwner {
    pub principal: Option<PrincipalId>,
    pub team: Option<OrganizationUnitId>,
    pub source: OwnershipSource,
}
```

---

# 145. Ownership Is Routing

Not authorization.

---

# 146. Remediation State

```rust
pub enum RemediationState {
    Untriaged,
    Triaged,
    Planned,
    FixInProgress,
    FixedPendingVerification,
    VerifiedFixed,
    Won'tFix,
}
```

---

# 147. Won'tFix

Requires reason/policy.

---

# 148. VerifiedFixed

Needs a later comparable analysis showing absence.

---

# 149. No Human "Fixed" Without Evidence

Can mark planned/fix-in-progress, but VerifiedFixed requires analysis.

---

# 150. Finding Ticket

Can link external issue tracker.

---

# 151. No Mandatory Issue Tracker

Forgeyard manages core lifecycle itself.

---

# 152. Finding Triage

Set:

```text
owner
priority
remediation state
suppression request
```

---

# 153. Security Priority

Separate from analyzer severity if organization needs.

---

# 154. FindingPriority

```rust
pub enum FindingPriority {
    P0,
    P1,
    P2,
    P3,
    P4,
}
```

---

# 155. New Finding Gate

Common PR policy:

```text
no new High/Critical security findings
```

---

# 156. Existing Debt

Can be allowed while preventing new debt.

---

# 157. Baseline Strategy

Useful migration path for existing codebase.

---

# 158. "No New Findings"

Must ensure scanner completeness comparable.

---

# 159. Scanner Failure

Cannot yield zero-new-findings Pass.

---

# 160. Quality Fact

```rust
pub enum AnalysisQualityFact {
    FindingCount(FindingCountFact),
    NewFindings(NewFindingFact),
    SecurityFindings(SecurityFindingFact),
    SecretFindings(SecretFindingFact),
    AnalysisCompleteness(AnalysisCompletenessFact),
}
```

---

# 161. Policy

Part 11 central policy.

---

# 162. Example

```text
AnalysisCompleteness == Complete
AND
new Critical == 0
AND
new High security == 0
AND
secret findings == 0
```

---

# 163. Change Proposal

Bind exact ProposalRevisionId/SourceSnapshot.

---

# 164. Release Candidate

Can require full scan and freshness.

---

# 165. Deployment

DAST/runtime findings can gate rollout if policy.

---

# 166. Evidence Freshness

```rust
pub enum AnalysisEvidenceFreshness {
    Fresh,
    Stale,
    Unknown,
}
```

---

# 167. Freshness Inputs

```text
subject
analyzer version
ruleset
vulnerability DB version
time
```

---

# 168. Static Lint

Often tied mostly to subject/tool version.

---

# 169. Vulnerability Scanner

Database freshness matters.

---

# 170. Secret Scan

Subject exact; rule updates may matter.

---

# 171. Re-scan Without Rebuild

Possible for source/artifact.

---

# 172. Artifact Scan

Same immutable artifact can be rescanned with newer rules/database.

---

# 173. Finding History

Logical finding across observations.

---

# 174. First Seen

Exact source/release.

---

# 175. Last Seen

Exact.

---

# 176. Reopened

Previously absent then returns.

---

# 177. Severity Change

Preserve per-observation severity.

---

# 178. Logical Current Severity

Derived latest comparable observation.

---

# 179. Rule Rename

Adapter mapping may preserve logical identity if explicitly known.

---

# 180. Do Not Guess Rule Equivalence

Critical.

---

# 181. SARIF Fingerprints

Use when available.

---

# 182. PartialFingerprints

Can aid correlation.

---

# 183. Code Flows

Store sanitized structured evidence/CAS.

---

# 184. Fix Suggestion

Analyzer may provide patch.

---

# 185. Suggested Fix

Not auto-applied baseline.

---

# 186. SuggestedPatchId

```rust
pub struct SuggestedPatchId(Digest);
```

---

# 187. Apply Fix

Creates source change/Change Proposal revision.

---

# 188. Never Mutate Source Behind Review

Critical.

---

# 189. Auto-Fix

Can generate explicit patch/Change Proposal.

---

# 190. AI-Assisted Fix

Future optional, same rule: proposal only, never hidden mutation.

---

# 191. SCM Check Publishing

Checks:

```text
Static Analysis
Security
Secrets
Code Quality
```

---

# 192. Annotations

Top findings.

---

# 193. Provider Limits

Respect limits; full detail in Forgeyard.

---

# 194. Annotation Location

Exact file/line from proposal source.

---

# 195. Stale Location

If revision changed, old annotation not reused.

---

# 196. Diff-Only Annotation

Prefer new findings.

---

# 197. Dioxus UI

Pages:

```text
Findings
Security
Code Quality
Secrets
Suppressions
Analyzers
```

---

# 198. Finding Detail

Shows:

```text
rule
severity
confidence
location
first seen
last seen
owner
suppression
raw evidence
analysis version
```

---

# 199. Security Finding View

CWE/CVE links where applicable.

---

# 200. Secret Finding View

Never show secret value.

---

# 201. Suppression UI

Requires:

```text
reason
scope
expiry
tracking issue
```

---

# 202. Bulk Suppression

High risk.

---

# 203. Baseline UI

Compare candidate vs base.

---

# 204. New Findings Tab

High value.

---

# 205. Resolved Findings Tab

Motivating.

---

# 206. Scanner Health Page

Shows:

```text
version
ruleset
last run
failure rate
coverage/completeness
```

---

# 207. Analyzer Configuration

RON.

---

# 208. Example

```ron
(
    analyzers: [
        (
            id: "clippy",
            scope: Full,
            required: true,
        ),
        (
            id: "secret-scan",
            scope: AffectedOnly,
            required: true,
        ),
    ],
)
```

---

# 209. Pipeline Integration

Analysis declaration can be first-class metadata around normal jobs.

---

# 210. AnalysisReportDeclaration

```rust
pub struct AnalysisReportDeclaration {
    pub format: AnalysisReportFormat,
    pub path: RelativePath,
    pub analyzer: AnalyzerRef,
    pub required: bool,
}
```

---

# 211. Missing Required Report

`AnalysisCompleteness::Failed/Partial`, gate accordingly.

---

# 212. Scanner Exit Code

Does not replace parsed evidence.

---

# 213. Some Scanners Exit Nonzero on Findings

Adapter understands semantics.

---

# 214. Some Exit Nonzero on Tool Failure

Differentiate.

---

# 215. AnalyzerRunOutcome

```rust
pub enum AnalyzerRunOutcome {
    Completed,
    FindingsDetected,
    ToolFailed,
    TimedOut,
    InfrastructureError,
    InvalidReport,
}
```

---

# 216. FindingsDetected

Not automatically job failure.

Policy decides.

---

# 217. ToolFailed

Analysis incomplete.

---

# 218. Scanner Retry

New JobAttempt.

---

# 219. Original failure retained.

---

# 220. Analyzer Determinism

Some analyzers may be nondeterministic.

---

# 221. Repeatability

Can record.

---

# 222. Do Not Cache Scanner Result Blindly

Cache key must include:

```text
subject
analyzer version
ruleset
database snapshot
scope
```

---

# 223. Vulnerability DB Snapshot

Part 13.

---

# 224. Static Analyzer Cache

Possible.

---

# 225. Secret Scanner Cache

Possible exact subject/ruleset.

---

# 226. Dynamic Scan

Usually run fresh.

---

# 227. Analysis Cache Hit

Still records evidence reuse, not new execution.

---

# 228. EvidenceReuseRecord

Explicit.

---

# 229. Analyzer Provider Trait

```rust
#[async_trait]
pub trait AnalysisAdapter {
    async fn normalize(
        &self,
        report: CasObjectRef,
        context: AnalysisNormalizationContext,
    ) -> Result<NormalizedAnalysis, AnalysisAdapterError>;
}
```

---

# 230. Parser Isolation

Complex/untrusted parsers can run sandboxed.

---

# 231. SARIF Parser

Bound:

```text
file size
result count
codeflow depth
message size
locations
```

---

# 232. JSON Bomb

Bound.

---

# 233. Path Traversal

Reject invalid paths.

---

# 234. HTML

Escape.

---

# 235. Scanner Output URLs

Never auto-fetch.

---

# 236. Remote Scanner Callback

Authenticated webhook if used.

---

# 237. Callback Result

Bind analysis run/request ID.

---

# 238. Replay

Dedupe.

---

# 239. Remote Scanner Unknown Outcome

Inspect/reconcile.

---

# 240. Multi-Tenant

Every AnalysisRun/Finding belongs to tenant/project.

---

# 241. Cross-Tenant Finding

Forbidden.

---

# 242. Shared Analyzer Infrastructure

Does not share finding visibility.

---

# 243. External Scanner Tenant Isolation

Separate project/account/token or explicit provider mapping.

---

# 244. Source Confidentiality

Policy.

---

# 245. High-Assurance

Local analyzers only.

---

# 246. Audit

Audit:

```text
suppression create/update/remove
risk acceptance
baseline admin change
analyzer policy change
manual override
```

---

# 247. Not Audit Every Finding

Finding store already evidence.

---

# 248. Critical Secret Finding

Can produce audit/security notification.

---

# 249. Notifications

Examples:

```text
new Critical finding
secret exposure
suppression expiring
scanner repeatedly failing
```

---

# 250. Security Incident

Some findings may open IncidentId.

---

# 251. Do Not Auto-Escalate Every High Finding

Policy.

---

# 252. Search

Part 31 indexes:

```text
rule
severity
category
owner
path
status
```

---

# 253. Analytics

Examples:

```text
open finding count
new findings/week
mean remediation time
suppression age
scanner failure rate
```

---

# 254. No Vanity Single "Code Quality Score" Baseline

Critical.

---

# 255. If Score Added

Presentation only, with transparent formula.

---

# 256. Quality Trend

Prefer explicit counts/categories.

---

# 257. Metrics

```text
analysis_runs_total
analysis_run_failures_total
analysis_findings_total
analysis_new_findings_total
analysis_secret_findings_total
analysis_suppressions_active
analysis_suppressions_expired
analysis_normalization_failures_total
```

---

# 258. Labels

Low-cardinality:

```text
analyzer_kind
severity
category
result
```

---

# 259. No Rule/File/Tenant metric labels.

---

# 260. Tracing

```text
analysis.run
analysis.normalize
analysis.correlate
analysis.diff
analysis.suppression
analysis.quality
```

---

# 261. Health

Checks:

```text
analyzer registry
normalizer backlog
required analyzer failures
remote provider health
suppression expiry reconciliation
```

---

# 262. Doctor

```text
forgeyard analysis doctor
```

---

# 263. Doctor Checks

```text
analyzer installed/version
ruleset validity
report parser
required scanner availability
remote provider credentials
```

---

# 264. CLI

```text
forgeyard analysis list
forgeyard findings
forgeyard findings new
forgeyard findings show <id>
forgeyard findings suppress <id>
forgeyard analysis compare
forgeyard analysis explain
```

---

# 265. `analysis explain`

Shows:

```text
subject
analyzer/ruleset
scope
completeness
baseline
new/existing/resolved counts
policy result
```

---

# 266. Permissions

```text
analysis.read
analysis.run
analysis.manage
finding.triage
finding.suppress
finding.risk_accept
analysis.override
dast.run
dast.production
```

---

# 267. Risk Acceptance

Higher permission than simple triage.

---

# 268. Manual Override

Does not mutate findings.

---

# 269. Override

Exact quality fact + subject + reason + principal.

---

# 270. Release Override

Stricter.

---

# 271. Finding Retention

Normalized history may be long-lived.

---

# 272. Raw Reports

Retention policy.

---

# 273. Secret Scan Raw Report

Especially sensitive—may require shorter/stricter retention.

---

# 274. CAS Classification

Restricted.

---

# 275. DR

Normalized metadata + suppression records backed up.

---

# 276. Raw reports in CAS according to retention.

---

# 277. Rebuild

Findings can be re-normalized from raw reports if retained.

---

# 278. Scanner Rerun

Can recreate current evidence from immutable source/artifact.

---

# 279. Historical Suppression

Must remain.

---

# 280. HA

Analysis ingest workers idempotent.

---

# 281. Duplicate Report

No duplicate logical observations.

---

# 282. Ingest Key

```text
AnalysisRunId + report digest + normalizer version
```

---

# 283. Reconciler

Checks:

```text
completed jobs with unprocessed reports
stuck processing
expired suppression
missing baseline diff
```

---

# 284. Finding Correlation Reconciler

Can improve derived logical links without rewriting observations.

---

# 285. Correlation Revision

Versioned.

---

# 286. Testkit

```text
forgeyard-analysis-testkit/src/
├── lib.rs
├── report.rs
├── finding.rs
├── baseline.rs
├── suppression.rs
├── correlation.rs
├── quality.rs
└── assertions.rs
```

---

# 287. Unit Tests

Severity/confidence mapping.

---

# 288. SARIF Tests

Multiple producers.

---

# 289. Secret Leakage Test

Detected secret never persists plaintext.

---

# 290. Scanner Failure Test

Does not produce false zero findings.

---

# 291. Baseline Comparability Test

Different ruleset cannot silently claim resolved findings.

---

# 292. New Finding Test

Exact baseline/candidate.

---

# 293. Resolved Test

Only when analysis Complete/comparable.

---

# 294. Heuristic Correlation Test

Does not auto-merge as Exact.

---

# 295. Suppression Test

Finding remains present.

---

# 296. Suppression Expiry Test

Gate re-evaluates.

---

# 297. Broad Suppression Permission Test

Restricted.

---

# 298. Risk Acceptance Audit Test

Mandatory.

---

# 299. Source Revision Test

Old analysis cannot satisfy new ProposalRevision.

---

# 300. Release Full Scope Test

Affected-only PR scan cannot satisfy full-scan release requirement.

---

# 301. DAST Target Authorization Test

Cannot scan arbitrary host.

---

# 302. Cross-Tenant Test

No finding leakage.

---

# 303. Parser Bomb Test

Bounded.

---

# 304. Path Traversal Test

Rejected.

---

# 305. Remote Provider Replay Test

Dedupe.

---

# 306. Remote Provider Unknown Test

Reconcile.

---

# 307. Analyzer Cache Test

Ruleset/database change invalidates reuse.

---

# 308. DR Test

Suppressions/history restored.

---

# 309. Fuzzing

Fuzz:

```text
SARIF
scanner JSON
path/location parsing
fingerprint inputs
```

---

# 310. Failure Injection

```text
scanner crash
CAS report missing
remote scanner timeout
DB restart
normalizer worker loss
```

---

# 311. Scale Test

Millions of findings.

---

# 312. Implementation Phase 1 — Core Finding Model

Analyzer/subject/finding.

---

# 313. Phase 2 — SARIF + Rust/Clippy

First dogfood.

---

# 314. Phase 3 — Baseline/New-Finding Diff

Change Proposal value.

---

# 315. Phase 4 — Secret Scanning

Security-critical.

---

# 316. Phase 5 — Suppression/Risk Acceptance

Governance.

---

# 317. Phase 6 — Security Analyzer Integration

Semgrep/CodeQL-like adapters.

---

# 318. Phase 7 — Artifact/Dependency Projection

Unified findings view.

---

# 319. Phase 8 — Policy/Release Gates

Quality evidence.

---

# 320. Phase 9 — DAST

Controlled deployment targets.

---

# 321. Phase 10 — Search/Analytics/Notifications

Operational UX.

---

# 322. Phase 11 — Remote Scanner/Plugin Providers

Enterprise integrations.

---

# 323. Phase 12 — Scale/Fuzz/DR Hardening

Production readiness.

---

# 324. Acceptance Tests

1. Analyzer output is evidence, not merge/release authority.
2. Every analysis binds an exact immutable subject.
3. Analyzer version and ruleset identity are recorded.
4. Raw reports are preserved according to retention.
5. Finding observations are immutable.
6. Logical finding lifecycle is separate from observations.
7. Severity and confidence are distinct.
8. Analyzer-native severity is retained.
9. Secret scanner never stores detected secret plaintext.
10. Scanner failure cannot be interpreted as zero findings.
11. Partial analysis cannot masquerade as Complete.
12. Baseline comparison resolves mutable refs to exact subjects.
13. New/existing/resolved finding classification requires comparable analysis.
14. A finding is not declared resolved when scanner execution failed.
15. Heuristic correlation cannot silently merge findings as exact identity.
16. Suppression never edits/deletes finding evidence.
17. Suppression has scope, reason, actor, and expiry.
18. Broad suppressions require stronger permission.
19. Risk acceptance is audited.
20. Suppressed findings remain visible.
21. Policy decides whether suppressed findings block.
22. PR affected-only analysis cannot satisfy full release scan if policy requires full.
23. Old analysis cannot satisfy a changed ProposalRevision.
24. DAST can target only authorized Forgeyard-managed targets.
25. Production DAST requires explicit permission/policy.
26. Remote analyzer source egress is policy-controlled.
27. External scanner credentials use SecretRef.
28. Analyzer cache identity includes version/ruleset/database/scope.
29. Search/analytics are derived from canonical findings.
30. Tenant findings are isolated.
31. Remote callback replay is deduplicated.
32. Ingestion/normalization is idempotent/reconcilable.
33. Parser upgrades create new normalization revisions.
34. Standalone/distributed share finding semantics.
35. Forgeyard dogfoods static/security analysis for its own codebase.

---

# 325. Production Readiness Gates

Do not call analysis/findings production-ready until:

```text
immutable finding model is stable
SARIF/Rust adapters are hardened
scanner failure/completeness semantics are explicit
secret-value leakage tests pass
baseline/new-finding logic is exact
suppression/risk acceptance is audited and expiring
policy gates bind exact subjects
DAST target authorization is enforced
tenant isolation passes
normalization/reconciliation survives worker failures
```

---

# 326. Architectural Invariants

1. analyzers produce evidence, not authority;
2. every analysis binds exact subject identity;
3. analyzer/ruleset/version is explicit;
4. finding observations are immutable;
5. logical lifecycle is separate from evidence;
6. severity and confidence are separate;
7. raw secret values never enter finding storage;
8. scanner failure is never equivalent to no findings;
9. partial analysis is explicit;
10. baseline refs resolve to immutable subjects;
11. resolved classification requires comparable complete analysis;
12. correlation confidence is explicit;
13. heuristic correlation never masquerades as exact;
14. suppressions never rewrite findings;
15. suppressions are scoped/reasoned/expiring/audited;
16. policy decides suppression effect;
17. source/artifact scan scope is explicit;
18. affected-only analysis cannot satisfy broader policy silently;
19. old findings cannot satisfy new revisions;
20. remote source egress is policy-controlled;
21. DAST targets are authorized and bounded;
22. parser inputs are hostile/bounded;
23. analyzer caches include all semantic inputs;
24. search/analytics are derived;
25. tenant finding data is isolated;
26. remediation state never fabricates verified fix;
27. parser upgrades create new interpretations;
28. HA ingestion is idempotent/reconcilable;
29. standalone/distributed share semantics;
30. Forgeyard dogfoods its own findings system.

---

# 327. Final Target Architecture

```text
                    Analyzer Job
                        │
                        ▼
                   Raw Report
                        │
                        ▼
                   Normalizer
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
        Findings      Scope      Completeness
            │           │           │
            └───────────┼───────────┘
                        ▼
                Baseline / Correlation
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
          New        Existing     Resolved
            │           │           │
            └───────────┼───────────┘
                        ▼
                Suppression Overlay
                        │
                        ▼
                    Policy Facts
                        │
                        ▼
               Merge / Release Gate
```

---

# 328. Final Architectural Position

Static/security analysis:

```text
exact source/artifact
+
analyzer version
+
ruleset
+
scan scope
  ↓
raw report
  ↓
normalized immutable findings
```

Baseline comparison:

```text
candidate findings
+
exact comparable baseline
+
correlation confidence
+
analysis completeness
  ↓
new / existing / resolved / unknown
```

Suppression:

```text
finding
+
reason
+
scope
+
actor
+
expiry
  ↓
suppression overlay
  ↓
policy evaluation
```

The key guarantee is:

> **Forgeyard can unify code quality and security analysis without trusting any single scanner as the arbiter of software safety. Findings remain immutable evidence tied to exact subjects, suppressions remain visible and auditable, scanner failures cannot produce false green results, and policy remains the final authority over merge, release, and deployment gates.**

---

# 329. Extended Architecture Sequence

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
```
