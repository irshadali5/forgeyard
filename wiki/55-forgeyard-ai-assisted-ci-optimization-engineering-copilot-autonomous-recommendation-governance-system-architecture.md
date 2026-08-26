# 55 — Forgeyard AI-Assisted CI Optimization, Engineering Copilot & Autonomous Recommendation Governance System Architecture

**Document type:** Core AI Assistance, CI Optimization, Engineering Copilot, Recommendation, Agentic Automation & Model Governance System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** AI-assisted pipeline optimization, failure summarization, test-selection recommendations, resource tuning, cost/reliability optimization, migration assistance, code-review assistance, operator copilot, natural-language query, recommendation lifecycle, bounded agentic workflows, model/provider governance, prompt/data security, evaluation, feedback, explainability, and AI safety controls  
**Architecture style:** Advisory-first, evidence-grounded, capability-scoped, policy-mediated, human-reviewable, model-agnostic, privacy-aware, deterministic core preserved, explicit confidence/provenance, and no AI authority over canonical control-plane truth  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Failure Diagnosis, Static Analysis, Test Intelligence, Monorepo Intelligence, Cost/FinOps, Reliability, Migration, Workflow Templates, Catalog, Search/Analytics, Policy/Authz, Secrets, Security, Audit, Configuration, Release, Deployment, and Infrastructure. This subsystem introduces optional AI/ML assistance without making models correctness dependencies.

---

# 1. Purpose

Forgeyard already produces large volumes of structured technical evidence:

```text
pipeline plans
run/job history
logs
test observations
coverage
static-analysis findings
benchmark results
dependency graphs
cost data
SLO data
deployment history
migration reports
failure clusters
catalog metadata
```

AI can help developers and operators interpret that evidence.

Useful questions include:

```text
why did this run fail?
which files most likely caused the regression?
which tests are worth running first?
how could this pipeline be faster?
why is this project expensive?
which runner class is over-provisioned?
what changed between the last good and first bad run?
how should this GitHub Actions workflow map to Forgeyard?
which golden path fits this service?
which repeated failure clusters deserve attention?
```

The central rule is:

> **AI may explain, summarize, propose, rank, and—only inside explicitly delegated limits—execute low-risk reversible actions. AI never becomes the authority for Forgeyard truth.**

A second rule is:

> **Every AI recommendation must be grounded in identifiable Forgeyard evidence. Unsupported speculation is labeled as such, and model confidence never substitutes for policy or deterministic verification.**

A third rule is:

> **The deterministic Forgeyard architecture must remain fully functional with AI disabled, unavailable, offline, or wrong.**

---

# 2. Architectural Position

```text
                 Canonical Forgeyard Evidence
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
        Runs          Tests         Costs
          │             │             │
          └─────────────┼─────────────┘
                        ▼
                Evidence Retrieval Layer
                        │
                        ▼
                   AI Orchestrator
                        │
             ┌──────────┼──────────┐
             ▼          ▼          ▼
          Explain     Recommend    Draft
             │          │          │
             └──────────┼──────────┘
                        ▼
                  Recommendation
                        │
                  Policy / Human
                        │
                        ▼
              Optional Bounded Action
```

AI sits **beside** the control plane, not inside its correctness core.

---

# 3. Goals

The subsystem MUST:

1. define AI recommendation identity;
2. define AI task identity;
3. define model/provider abstractions;
4. support local models;
5. support remote model APIs;
6. support offline mode;
7. support evidence retrieval;
8. support failure summaries;
9. support root-cause hypotheses;
10. support pipeline optimization recommendations;
11. support test-selection hints;
12. support resource tuning hints;
13. support cache/monorepo optimization hints;
14. support cost optimization recommendations;
15. support reliability recommendations;
16. support migration assistance;
17. support code-review assistance;
18. support natural-language search/query;
19. support operator copilot workflows;
20. support human approval;
21. support bounded autonomous actions;
22. support prompt/data security;
23. support model evaluation;
24. support provenance/explainability;
25. support tenant isolation;
26. support audit;
27. support feedback;
28. support API/UI/CLI;
29. remain optional;
30. never weaken deterministic correctness.

---

# 4. Non-Goals

This subsystem does not:

```text
replace Policy
replace Scheduler
replace static analyzers
replace test results
replace security scanners
replace incident response
replace release approval
replace deployment authority
replace IaC authority
replace human engineering judgment
```

---

# 5. Workspace Structure

```text
crates/ai/
├── forgeyard-ai/
├── forgeyard-ai-model/
├── forgeyard-ai-provider/
├── forgeyard-ai-context/
├── forgeyard-ai-retrieval/
├── forgeyard-ai-recommendation/
├── forgeyard-ai-agent/
├── forgeyard-ai-evaluation/
├── forgeyard-ai-redaction/
├── forgeyard-ai-policy/
├── forgeyard-ai-health/
└── forgeyard-ai-testkit/
```

Provider adapters:

```text
crates/ai-providers/
├── forgeyard-ai-local/
├── forgeyard-ai-openai-compatible/
├── forgeyard-ai-anthropic-compatible/
├── forgeyard-ai-ollama-compatible/
├── forgeyard-ai-llamacpp/
└── forgeyard-ai-custom/
```

Core AI crates remain model/provider-neutral.

---

# 6. AiTaskId

```rust
pub struct AiTaskId(Ulid);
```

One request to AI subsystem.

---

# 7. AiRecommendationId

```rust
pub struct AiRecommendationId(Ulid);
```

One immutable recommendation/output record.

---

# 8. AiTaskKind

```rust
pub enum AiTaskKind {
    FailureSummary,
    RootCauseHypothesis,
    PipelineOptimization,
    TestSelectionHint,
    ResourceTuning,
    CostOptimization,
    ReliabilityRecommendation,
    MigrationAssistance,
    CodeReviewAssistance,
    NaturalLanguageQuery,
    RunbookAssistance,
    Custom(AiTaskKindId),
}
```

---

# 9. AiTask

```rust
pub struct AiTask {
    pub id: AiTaskId,
    pub tenant: TenantId,
    pub kind: AiTaskKind,
    pub subject: AiSubject,
    pub policy: AiExecutionPolicy,
}
```

---

# 10. AiSubject

```rust
pub enum AiSubject {
    Run(RunId),
    Job(JobId),
    Failure(FailureObservationId),
    ChangeProposal(ProposalRevisionId),
    Pipeline(PipelineId),
    Project(ProjectId),
    Component(SoftwareComponentId),
    Migration(MigrationProjectId),
    Deployment(DeploymentId),
    Infrastructure(InfrastructureEnvironmentId),
}
```

---

# 11. Evidence-First Context

AI context must be built from canonical Forgeyard evidence.

Examples:

```text
exact RunId
SourceSnapshotId
FailureObservationId
TestObservation IDs
ConfigSnapshotId
PolicyDigest
DependencyDiff
BenchmarkObservation
CostFacts
SloEvaluation
```

---

# 12. No Freeform Hidden Context

Critical.

Context inputs are explicit and inspectable.

---

# 13. AiContextBundleId

```rust
pub struct AiContextBundleId(Digest);
```

---

# 14. AiContextBundle

```rust
pub struct AiContextBundle {
    pub id: AiContextBundleId,
    pub subject: AiSubject,
    pub evidence: Vec<AiEvidenceRef>,
    pub redaction_policy: RedactionPolicyId,
}
```

---

# 15. Context Provenance

Every context chunk records source.

---

# 16. AiEvidenceRef

Examples:

```text
file lines
log range
test observation
finding
cost fact
SLO evaluation
migration finding
catalog relation
```

---

# 17. Recommendation Grounding

Recommendation references supporting evidence.

---

# 18. Recommendation Model

```rust
pub struct AiRecommendation {
    pub id: AiRecommendationId,
    pub task: AiTaskId,
    pub category: AiRecommendationCategory,
    pub summary: BoundedString,
    pub confidence: AiConfidence,
    pub evidence: Vec<AiEvidenceRef>,
    pub actions: Vec<AiSuggestedAction>,
}
```

---

# 19. Confidence

```rust
pub enum AiConfidence {
    Low,
    Medium,
    High,
    Unknown,
}
```

---

# 20. No `Confirmed` From Model Alone

Critical.

Confirmed state comes only from deterministic/external evidence.

---

# 21. Recommendation Category

```rust
pub enum AiRecommendationCategory {
    Informational,
    Optimization,
    Reliability,
    SecurityAdvisory,
    Cost,
    Migration,
    Debugging,
    DeveloperExperience,
}
```

---

# 22. Model Provider Abstraction

```rust
#[async_trait]
pub trait AiProvider {
    async fn infer(
        &self,
        request: AiInferenceRequest,
    ) -> Result<AiInferenceResponse, AiProviderError>;
}
```

---

# 23. Provider Capability

```rust
pub struct AiProviderCapabilities {
    pub context_window: u32,
    pub structured_output: bool,
    pub tool_calling: bool,
    pub local: bool,
}
```

---

# 24. ModelRef

```rust
pub struct ModelRef {
    pub provider: AiProviderId,
    pub model: ModelId,
    pub version: Option<ModelVersion>,
}
```

---

# 25. Model Identity

Record exact model/provider/version when available.

---

# 26. Provider Selection

Policy-driven.

---

# 27. Local-Only Policy

High-assurance tenants can require local inference.

---

# 28. Remote Egress Policy

Explicit.

---

# 29. Source Code Egress

Never assumed.

---

# 30. Sensitive Context Classes

```rust
pub enum AiDataClass {
    Public,
    Internal,
    Confidential,
    Restricted,
    SourceCode,
    SecurityEvidence,
    PersonalData,
}
```

---

# 31. Provider Allowlist

Each provider declares allowed data classes.

---

# 32. No Context Egress Beyond Policy

Critical.

---

# 33. Secret Redaction

Before inference.

---

# 34. SecretRef

May be described as metadata only.

---

# 35. Secret Value

Never intentionally placed in prompt/context.

---

# 36. Log Redaction

Reuse Part 12/17 redaction system.

---

# 37. Prompt Injection

Repository/source/log content is untrusted input.

---

# 38. Critical Rule

Model instructions found inside:

```text
source files
logs
README
test output
issues
webhooks
```

are data, not authority.

---

# 39. Prompt Segmentation

Use structured role/context boundaries.

---

# 40. Tool Calls

AI cannot directly invoke arbitrary control-plane operations.

---

# 41. AiToolCapability

```rust
pub enum AiToolCapability {
    ReadRun,
    ReadLogs,
    SearchEvidence,
    CompareRuns,
    DraftPatch,
    RequestReproduction,
    RequestBisect,
    CreateRecommendation,
}
```

---

# 42. High-Risk Tools

Not available by default:

```text
PublishRelease
SignArtifact
DeployProduction
ChangePolicy
ReadSecrets
DeleteData
ChangeAuthz
```

---

# 43. Capability Broker

Every tool call passes through Forgeyard authorization/policy.

---

# 44. No Model-Held Credentials

Critical.

---

# 45. Tool Execution Identity

Use bounded service identity + user delegation where required.

---

# 46. Human-in-the-Loop

Default for changes.

---

# 47. AiActionClass

```rust
pub enum AiActionClass {
    ReadOnly,
    DraftOnly,
    ReversibleLowRisk,
    Privileged,
    ForbiddenAutonomous,
}
```

---

# 48. ReadOnly

Can run automatically.

---

# 49. DraftOnly

Examples:

```text
draft pipeline patch
draft migration mapping
draft runbook
draft suppression request
```

---

# 50. ReversibleLowRisk

Examples:

```text
start reproduction
start bounded bisect
create temporary diagnostic query
```

Only if delegated.

---

# 51. Privileged

Requires explicit human approval.

---

# 52. ForbiddenAutonomous

Examples:

```text
sign release
approve release
deploy production
change authz
change policy
delete evidence
release legal hold
rotate root CA
```

---

# 53. Agentic Workflow

Optional.

---

# 54. AiAgentRunId

```rust
pub struct AiAgentRunId(Ulid);
```

---

# 55. Agent Workflow State

```rust
pub enum AiAgentRunState {
    Planned,
    AwaitingApproval,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}
```

---

# 56. Agent Plan

Before tools execute, produce bounded plan.

---

# 57. AgentStep

```rust
pub struct AiAgentStep {
    pub capability: AiToolCapability,
    pub inputs: AiToolInputs,
    pub risk: AiActionClass,
}
```

---

# 58. Step Budget

Limit:

```text
number of tool calls
wall-clock duration
compute cost
tokens
scope
```

---

# 59. No Unbounded Autonomous Loop

Critical.

---

# 60. Action Budget

```rust
pub struct AiActionBudget {
    pub max_steps: u16,
    pub max_duration: Duration,
    pub max_cost: Option<Money>,
}
```

---

# 61. Failure Summary

Input:

```text
job outcome
failure observation
logs
test findings
context diff
```

Output:

```text
concise failure explanation
important evidence
likely next diagnostic steps
```

---

# 62. Canonical Failure State

Unchanged.

---

# 63. Root Cause Assistance

Part 48 owns diagnosis evidence.

AI can:

```text
summarize hypotheses
rank existing evidence
suggest next experiment
```

---

# 64. AI Does Not Declare Root Cause

Critical.

---

# 65. Reproduction Recommendation

AI may suggest:

```text
run exact reproduction
compare toolchain
bisect target range
```

---

# 66. Diagnostic Experiment

Created only through Part 48.

---

# 67. Pipeline Optimization

AI may analyze:

```text
critical path
cache misses
job durations
matrix expansion
dependency graph
runner utilization
```

---

# 68. Recommendation Examples

```text
split job
combine tiny jobs
increase cacheability
pin toolchain
reduce redundant matrix combinations
use affected-work selection
```

---

# 69. Pipeline Patch

Draft only.

---

# 70. Pipeline Compiler

Still validates through Part 04.

---

# 71. No Direct IR Mutation

Critical.

---

# 72. Test Selection Hint

Part 34/32 remain authority.

AI may suggest:

```text
likely relevant tests
additional suspicious tests
historical failure-associated tests
```

---

# 73. Mandatory Test Floor

Cannot be reduced by AI.

---

# 74. Test Impact Analysis

Deterministic graph engine remains baseline.

---

# 75. AI Selection

Advisory additive by default.

---

# 76. Never AI-Only Skip Required Tests

Critical.

---

# 77. Resource Tuning

AI may recommend:

```text
CPU
memory
GPU class
timeout
parallelism
runner pool
```

---

# 78. Evidence Inputs

Historical usage.

---

# 79. Scheduler Hard Requirements

Still authority.

---

# 80. Resource Recommendation

No direct scheduler bypass.

---

# 81. Cost Optimization

Part 45 supplies facts.

AI may identify:

```text
idle capacity
expensive macOS overuse
unnecessary egress
long artifact retention
oversized runners
```

---

# 82. Cost Recommendation

Cannot weaken:

```text
security
residency
test requirements
reliability floors
```

---

# 83. Reliability Recommendation

Part 50 supplies SLO/burn evidence.

AI may suggest:

```text
reduce rollout size
increase warm pool
investigate dependency
run resilience test
```

---

# 84. Reliability State

Never rewritten.

---

# 85. Migration Assistance

Part 47 supplies migration findings.

AI may:

```text
explain unsupported semantics
draft Forgeyard-native replacement
suggest template
generate migration notes
```

---

# 86. Migration Compatibility

AI cannot mark Unknown as Exact.

Critical.

---

# 87. Code Review Assistance

Input:

```text
exact ProposalRevisionId
SourceSnapshot diff
analysis findings
test evidence
```

---

# 88. Output

```text
summary
risk areas
questions
possible missing tests
```

---

# 89. Review Verdict

Human/policy authority.

---

# 90. AI Approval

Never counts as protected human approval baseline.

---

# 91. Suggested Comment

Draft.

---

# 92. Suggested Patch

Explicit new source change.

---

# 93. No Hidden Mutation

Critical.

---

# 94. Natural-Language Query

Examples:

```text
"show failures like this in the last week"
"why did queue latency increase?"
"which projects don't use the Rust golden path?"
```

---

# 95. Query Translation

AI may translate language into constrained query AST.

---

# 96. Query Executor

Validates AST/authz.

---

# 97. No Raw SQL Generation Execution

Critical.

---

# 98. Constrained Query AST

Part 31.

---

# 99. Operator Copilot

Can summarize:

```text
site health
queue backlog
fleet capacity
SLO burn
provider incidents
replication lag
```

---

# 100. Operator Action

Draft or permission-gated.

---

# 101. Incident Copilot

Can assemble:

```text
timeline
affected components
failure clusters
recent changes
runbook links
```

---

# 102. Incident Commands

Human remains authority.

---

# 103. Runbook Assistance

AI can retrieve and summarize runbook.

---

# 104. No Invented Procedure

If no supporting runbook/evidence, state uncertainty.

---

# 105. Recommendation Lifecycle

```rust
pub enum AiRecommendationState {
    Generated,
    Viewed,
    Accepted,
    Rejected,
    Applied,
    Superseded,
    Expired,
}
```

---

# 106. Accepted

Human/user accepted concept.

---

# 107. Applied

Actual change linked to source/action.

---

# 108. Recommendation != Action

Critical.

---

# 109. Recommendation Freshness

```rust
pub enum AiRecommendationFreshness {
    Current,
    SubjectChanged,
    EvidenceChanged,
    ModelChanged,
    Expired,
    Unknown,
}
```

---

# 110. Source Change

Invalidates code/pipeline-specific recommendation.

---

# 111. Evidence Change

May invalidate diagnosis recommendation.

---

# 112. No Applying Stale Recommendation Blindly

Critical.

---

# 113. Recommendation Expiry

Configurable.

---

# 114. Recommendation Provenance

Record:

```text
model
provider
prompt/template version
context bundle
evidence refs
timestamp
```

---

# 115. PromptTemplateId

```rust
pub struct PromptTemplateId(Digest);
```

---

# 116. Prompt Versioning

Critical for evaluation/reproducibility.

---

# 117. System Prompt

Internal/config.

---

# 118. User Prompt

Stored according lifecycle/privacy policy.

---

# 119. Source Text

Not duplicated unnecessarily.

---

# 120. Structured Output

Prefer typed schema.

---

# 121. Parsing

Model output is untrusted.

---

# 122. Validate Against Schema

Critical.

---

# 123. Invalid Output

Reject/retry bounded.

---

# 124. No Eval of Model-Generated Code

Critical.

---

# 125. Draft Code

Displayed/stored as patch text only.

---

# 126. Patch Application

Uses normal source tooling with user approval.

---

# 127. Generated Pipeline Config

Must compile through normal parser/IR.

---

# 128. Generated Policy

Never auto-activate.

---

# 129. Model Routing

```rust
pub struct ModelRoutingPolicy {
    pub task: AiTaskKind,
    pub allowed_providers: Vec<AiProviderId>,
    pub local_preferred: bool,
}
```

---

# 130. Data Residency

Provider region must satisfy policy.

---

# 131. Tenant Policy

Can disable AI entirely.

---

# 132. Project Policy

Can be stricter.

---

# 133. AI Disabled Mode

All core Forgeyard behavior remains available.

---

# 134. Provider Outage

AI tasks fail/degrade only.

---

# 135. CI Correctness

Unaffected.

---

# 136. Local Models

Useful for:

```text
air-gap
sensitive code
offline
cost control
```

---

# 137. Local Model Runtime

Run outside main daemon.

---

# 138. Model Worker

Dedicated process/container/GPU runner.

---

# 139. Untrusted Model Runtime

No DB credentials.

---

# 140. Model Worker Access

Only receives prepared context bundle.

---

# 141. Context Gateway

Enforces redaction/policy.

---

# 142. Remote API Provider

Gateway sends only approved data.

---

# 143. Provider Response

Treated as untrusted input.

---

# 144. Provider Logging/Retention

Configuration metadata should describe provider data handling assumptions where known.

---

# 145. No Secret Contract Assumption

Policy controls.

---

# 146. Prompt Injection Defense

Layers:

```text
structured context
trusted instructions separate
tool allowlist
authz broker
output schema validation
no model credentials
bounded agent steps
```

---

# 147. Repository Instruction File

May optionally provide project AI guidance.

---

# 148. Project AiGuidance

Untrusted project-level hints.

---

# 149. Cannot override system safety/policy.

---

# 150. AI Memory

No uncontrolled cross-project memory.

---

# 151. Task Context Scope

Explicit.

---

# 152. Cross-Tenant Memory

Forbidden.

---

# 153. Project Conversation Memory

Optional and tenant-scoped.

---

# 154. Long-Term Memory

Must have lifecycle/privacy governance.

---

# 155. Default

Task-local context.

---

# 156. Embeddings

Optional.

---

# 157. Embedding Store

Derived index.

---

# 158. Vector Search

Optional acceleration.

---

# 159. Embedding Provider

Data egress policy applies.

---

# 160. Local Embeddings

Preferred for sensitive tenants.

---

# 161. Embedding Does Not Remove Sensitivity

Critical.

---

# 162. Retrieval

Hybrid:

```text
structured query
keyword search
vector search
graph relationships
```

---

# 163. Canonical IDs

Evidence returned with exact IDs.

---

# 164. Retrieval Authorization

Before context assembly.

---

# 165. Model Cannot Search Unauthorized Data

Critical.

---

# 166. Cross-Tenant Similarity

Disabled.

---

# 167. Evaluation System

AI needs measurable quality.

---

# 168. AiEvaluationSuiteId

```rust
pub struct AiEvaluationSuiteId(Digest);
```

---

# 169. Evaluation Cases

Use sanitized/internal fixtures.

---

# 170. Evaluate

```text
grounding
correct evidence citations
unsupported-claim rate
recommendation usefulness
schema validity
security-policy compliance
tool-call safety
```

---

# 171. No Single "AI Accuracy" Metric

Critical.

---

# 172. Evaluation Result

```rust
pub struct AiEvaluationResult {
    pub suite: AiEvaluationSuiteId,
    pub model: ModelRef,
    pub metrics: Vec<AiEvaluationMetric>,
}
```

---

# 173. Model Upgrade

Run evaluation before default promotion.

---

# 174. ModelPromotionState

```rust
pub enum ModelPromotionState {
    Candidate,
    Evaluated,
    Approved,
    Active,
    RolledBack,
}
```

---

# 175. AI Model Rollout

Part 39 feature/config can canary.

---

# 176. Model Rollback

Does not affect canonical state.

---

# 177. Prompt Upgrade

Likewise evaluated/versioned.

---

# 178. Shadow Evaluation

New model can run without surfacing recommendation.

---

# 179. A/B Testing

Optional.

---

# 180. No Hidden User Manipulation

Keep experiments internal/operational.

---

# 181. User Feedback

```rust
pub enum AiFeedback {
    Helpful,
    NotHelpful,
    Incorrect,
    Unsafe,
    Other,
}
```

---

# 182. Feedback

Used for evaluation, not direct online learning baseline.

---

# 183. No Automatic Training on Private Source

Critical.

---

# 184. Training Data

Requires explicit separate governance.

---

# 185. Fine-Tuning

Optional future.

---

# 186. Tenant Data

Never pooled into global training by default.

---

# 187. Cost Accounting

Part 45.

Meter:

```text
tokens
GPU time
provider API cost
embedding cost
```

---

# 188. AiUsageRecord

```rust
pub struct AiUsageRecord {
    pub task: AiTaskId,
    pub provider: AiProviderId,
    pub input_units: u64,
    pub output_units: u64,
}
```

---

# 189. Budget

Tenant AI budget.

---

# 190. Budget Limit

Stops optional AI, not CI.

---

# 191. Rate Limits

Per tenant/project/principal.

---

# 192. No AI Abuse Blocking Core Builds

Critical.

---

# 193. Latency

Async recommendation can arrive after run.

---

# 194. UI

Must show AI status separately.

---

# 195. Dioxus UI

Pages/panels:

```text
AI Assistant
Recommendations
Failure Assistant
Pipeline Optimizer
Migration Assistant
AI Settings
Model Health
```

---

# 196. Recommendation Card

Shows:

```text
recommendation
confidence
evidence
model
freshness
suggested action
```

---

# 197. Evidence Links

Clickable to canonical Forgeyard objects.

---

# 198. AI Badge

Clearly marks generated content.

---

# 199. No AI Output Styled as Canonical Fact

Critical.

---

# 200. Failure Assistant

Can show:

```text
summary
similar failures
likely differences
suggested next experiment
```

---

# 201. Pipeline Optimizer

Can show:

```text
critical path
cache opportunities
resource tuning
parallelism suggestions
estimated impact
```

---

# 202. Estimated Impact

Clearly labeled estimate.

---

# 203. Migration Assistant

Uses Part 47 report.

---

# 204. Code Review Assistant

Suggestion-only.

---

# 205. CLI

```text
forgeyard ai explain-run <run>
forgeyard ai explain-failure <job>
forgeyard ai optimize-pipeline <pipeline>
forgeyard ai suggest-tests <change>
forgeyard ai migrate-assist <migration>
forgeyard ai recommendations list
forgeyard ai doctor
```

---

# 206. Machine Output

JSON/RON.

---

# 207. API

Potential:

```text
POST /v1/ai/tasks
GET  /v1/ai/tasks/{id}
GET  /v1/ai/recommendations
POST /v1/ai/recommendations/{id}/feedback
POST /v1/ai/recommendations/{id}/apply
```

---

# 208. Apply Endpoint

Only invokes allowed bounded action after normal authz/policy.

---

# 209. Permissions

```text
ai.use
ai.source_context
ai.security_context
ai.agent.execute
ai.admin
ai.model.manage
```

---

# 210. Sensitive Context

Separate permissions.

---

# 211. AI Admin

Cannot grant underlying control-plane permissions.

---

# 212. Audit

Audit:

```text
model/provider change
AI policy change
sensitive-context inference
agentic privileged action request
recommendation apply
```

---

# 213. Routine AI query

Operational history according privacy/lifecycle policy.

---

# 214. Recommendation Application

Link:

```text
AiRecommendationId
  ↓
user approval
  ↓
actual ChangeProposal / ConfigPatch / DiagnosticExperiment
```

---

# 215. Actual Action

Canonical subsystem records truth.

---

# 216. No AI-Only Action Record

Critical.

---

# 217. Security

Threats:

```text
prompt injection
source exfiltration
secret leakage
tool abuse
cross-tenant retrieval
malicious model/provider
unsafe generated patch
hallucinated diagnosis
cost exhaustion
```

---

# 218. Security Boundaries

AI worker has no:

```text
database superuser
secret provider admin
signing key
production deploy credential
policy admin token
```

---

# 219. Capability Tokens

Short-lived and narrowly scoped.

---

# 220. Tool Broker Re-Checks Authz

Every invocation.

---

# 221. Model Tool Request Is Not Authorization

Critical.

---

# 222. Source Exfiltration Defense

Context gateway checks:

```text
tenant policy
data class
provider policy
size
redaction
```

---

# 223. DLP

Optional advanced control.

---

# 224. Security Evidence

Remote model may be forbidden.

---

# 225. Vulnerability Details

Policy-specific.

---

# 226. Incident Data

Restricted.

---

# 227. Prompt Retention

Part 46.

---

# 228. AI Conversation Retention

Short by default.

---

# 229. Recommendation Retention

Can outlive raw prompt if useful.

---

# 230. Model Provider Logs

External; Forgeyard cannot guarantee deletion unless provider contract/API supports.

---

# 231. Honest External Limitation

Critical.

---

# 232. Air-Gap Mode

Use local model only.

---

# 233. No AI

Still valid.

---

# 234. Federation

Part 51 routes model execution according:

```text
residency
GPU capacity
site trust
connectivity
```

---

# 235. Model Site

Can be separate site.

---

# 236. Data Residency

Hard constraint.

---

# 237. GPU Fleet

Part 43.

---

# 238. AI Scheduler

Normal scheduler with model-runner capability.

---

# 239. Model Inference Job

Can be regular bounded executor workload.

---

# 240. But AI result remains advisory.

---

# 241. Reliability

Part 50 possible SLOs:

```text
AI task latency
provider availability
recommendation generation success
```

---

# 242. AI SLO Failure

Never impacts build correctness.

---

# 243. Observability Metrics

```text
ai_tasks_total
ai_task_failures_total
ai_recommendations_total
ai_tool_calls_total
ai_tool_denied_total
ai_context_redactions_total
ai_provider_latency_seconds
ai_evaluation_failures_total
```

---

# 244. Labels

Low-cardinality:

```text
task_kind
provider
result
action_class
```

---

# 245. No source/prompt text in metrics.

---

# 246. Tracing

```text
ai.context
ai.retrieve
ai.infer
ai.validate
ai.recommend
ai.tool
ai.evaluate
```

---

# 247. Trace Content

No sensitive prompt bodies by default.

---

# 248. Health

Checks:

```text
provider connectivity
local model runtime
context gateway
tool broker
evaluation status
```

---

# 249. Doctor

```text
forgeyard ai doctor
```

Checks:

```text
provider credentials
model availability
residency policy
redaction
tool permissions
evaluation freshness
local GPU/runtime
```

---

# 250. Model Credential

SecretRef.

---

# 251. Provider API Keys

Late resolved.

---

# 252. No model key in source/config plaintext.

---

# 253. Configuration

Part 39.

Examples:

```text
AI enabled
allowed providers
local-only
task allowlist
max context
tool permissions
budget
```

---

# 254. Feature Flag

AI can be rolled out gradually.

---

# 255. Entitlement

Part 30 may gate expensive AI features.

---

# 256. Security Baseline

Core non-AI safety remains available to all.

---

# 257. Search

Part 31.

AI natural-language query translates into safe query AST.

---

# 258. No Direct Database Query Generation

Existing invariant.

---

# 259. Catalog

Part 49 helps retrieve ownership/docs/runbooks.

---

# 260. Workflow Templates

Part 42 helps AI suggest organization-approved patterns.

---

# 261. Golden Path First

AI should prefer organization-approved template recommendation over inventing custom pipeline.

---

# 262. Policy-aware Suggestions

Show when suggestion violates current org standard.

---

# 263. Static Analysis

Part 37 findings are authoritative.

AI may explain but not suppress automatically.

---

# 264. Suppression Draft

Allowed.

---

# 265. Suppression Apply

Normal policy/approval.

---

# 266. Test Intelligence

Part 32 stability is authoritative.

AI summary cannot relabel test as flaky.

---

# 267. Benchmark

Part 33 metrics authoritative.

---

# 268. AI Performance Diagnosis

Can explain likely drivers.

---

# 269. Cost

Part 45 facts authoritative.

---

# 270. AI Cost Estimate

Marked estimate.

---

# 271. Infrastructure

Part 53 plan authoritative.

AI can explain plan.

---

# 272. No AI `apply` without explicit authorized action.

---

# 273. Merge Queue

Part 54.

AI may explain why item blocked.

---

# 274. AI Cannot reorder queue unless a human-authorized priority action is separately executed.

---

# 275. Release

AI may summarize evidence.

---

# 276. AI Cannot approve/sign release.

Critical.

---

# 277. Deployment

AI may recommend rollback.

---

# 278. Deployment subsystem/human policy decides.

---

# 279. Security Incident

AI may assemble timeline/hypotheses.

---

# 280. AI cannot close incident or rotate root trust autonomously.

---

# 281. Model Evaluation Categories

Recommended:

```text
Grounding
EvidenceCitation
Safety
ActionPolicy
SchemaCompliance
TaskQuality
Robustness
PromptInjectionResistance
```

---

# 282. Golden Evaluation Fixtures

Use stable sanitized cases.

---

# 283. Regression Gate

New default model/prompt must meet configured minimum eval criteria.

---

# 284. No Production Auto-Promotion on Vendor Model Alias Change

Critical.

---

# 285. Mutable Model Alias

Resolve/record concrete version when possible.

---

# 286. If Provider Version Opaque

Record provider/model/time and treat reproducibility as limited.

---

# 287. Model Determinism

Do not assume deterministic.

---

# 288. Recommendation Reproduction

Best-effort only.

---

# 289. Canonical Evidence

Deterministic even if AI output differs.

---

# 290. Temperature/Sampling

Record settings where configurable.

---

# 291. Tool-Calling Determinism

Still bounded by broker/state.

---

# 292. Structured Response Schema

Versioned.

---

# 293. AiResponseSchemaVersion

```rust
pub struct AiResponseSchemaVersion(u16);
```

---

# 294. Model Output Sanitization

Escape unsafe markdown/links.

---

# 295. UI XSS Safety

Critical.

---

# 296. Suggested Shell Commands

Displayed as untrusted text.

---

# 297. No auto-run command from model output.

---

# 298. Recommended Action Types

```rust
pub enum AiSuggestedAction {
    ViewEvidence(EvidenceRef),
    StartReproduction(FailureObservationId),
    StartBisect(BisectProposal),
    DraftPipelinePatch(CasObjectRef),
    DraftSourcePatch(CasObjectRef),
    OpenChangeProposal(DraftChangeRef),
    RequestHumanReview(ReviewRequest),
}
```

---

# 299. OpenChangeProposal

Requires explicit user action/authz.

---

# 300. No Direct Push to Protected Branch

Critical.

---

# 301. Recommendation De-duplication

Same subject/evidence/model may generate repeated suggestions.

---

# 302. AiRecommendationFingerprint

```rust
pub struct AiRecommendationFingerprint(Digest);
```

---

# 303. Duplicate Suppression

UX optimization only.

---

# 304. Recommendation Conflict

Two models may disagree.

---

# 305. Show disagreement.

---

# 306. No majority-vote truth.

Critical.

---

# 307. Multi-Model Review

Optional.

---

# 308. Costly.

---

# 309. Human decides.

---

# 310. Evaluation Feedback Loop

Offline analysis.

---

# 311. Do Not Train Live on thumbs-up/down automatically.

---

# 312. Privacy Review

Required before using feedback for model training.

---

# 313. Testkit

```text
forgeyard-ai-testkit/src/
├── lib.rs
├── context.rs
├── recommendation.rs
├── tool.rs
├── policy.rs
├── injection.rs
├── evaluation.rs
└── assertions.rs
```

---

# 314. Unit Tests

Context bundle identity.

---

# 315. Tenant Isolation Test

No cross-tenant evidence retrieval.

---

# 316. Secret Test

Secret value removed before inference.

---

# 317. Prompt Injection Test

Source text cannot authorize tool.

---

# 318. Tool Authorization Test

Model request denied without permission.

---

# 319. Forbidden Action Test

AI cannot approve/sign/deploy production autonomously.

---

# 320. Stale Recommendation Test

Source change prevents blind apply.

---

# 321. Schema Test

Invalid model output rejected.

---

# 322. Query Test

Natural language becomes constrained AST, not raw SQL.

---

# 323. Pipeline Draft Test

Generated config must pass normal parser/IR.

---

# 324. Test Selection Test

AI cannot remove mandatory policy floor.

---

# 325. Cost Recommendation Test

Cannot override residency/trust.

---

# 326. Failure Summary Test

Canonical failure state unchanged.

---

# 327. Migration Test

AI cannot upgrade Unknown compatibility to Exact without deterministic evidence.

---

# 328. Review Test

AI approval does not satisfy protected human approval.

---

# 329. Provider Outage Test

Builds unaffected.

---

# 330. Local-Only Test

Remote provider blocked.

---

# 331. Air-Gap Test

Local model works/AI disabled fallback works.

---

# 332. Agent Budget Test

Stops at max steps.

---

# 333. Model Credential Test

Never exposed to model context.

---

# 334. Output Sanitization Test

Unsafe HTML/links escaped.

---

# 335. Evaluation Regression Test

New model cannot become default if configured gate fails.

---

# 336. Fuzzing

Fuzz:

```text
model structured output
tool-call payloads
prompt/context metadata
recommendation parser
```

---

# 337. Adversarial Tests

```text
prompt injection in source
prompt injection in logs
malicious dependency README
tool escalation requests
secret extraction attempts
cross-tenant query attempts
```

---

# 338. Scale Test

Large context retrieval across monorepos/history.

---

# 339. Cost Test

AI usage bounded.

---

# 340. Implementation Phase 1 — Read-Only AI Context + Failure Summary

Safe baseline.

---

# 341. Phase 2 — Natural-Language Search

Constrained query AST.

---

# 342. Phase 3 — Pipeline/Cost Recommendations

Advisory.

---

# 343. Phase 4 — Migration Assistant

Part 47.

---

# 344. Phase 5 — Code Review Assistant

Draft-only.

---

# 345. Phase 6 — Bounded Diagnostic Tools

Reproduction/bisect.

---

# 346. Phase 7 — Local Model Runtime

Privacy/air-gap.

---

# 347. Phase 8 — Model Evaluation/Governance

Production quality.

---

# 348. Phase 9 — Agentic Low-Risk Workflows

Explicit delegation.

---

# 349. Phase 10 — Reliability/Infrastructure Copilot

Operational assistance.

---

# 350. Phase 11 — Multi-Model/Advanced Retrieval

Optional.

---

# 351. Phase 12 — Adversarial/Scale/Privacy Hardening

Production readiness.

---

# 352. Acceptance Tests

1. Forgeyard remains fully functional with AI disabled.
2. AI output never becomes canonical Run/Job/Policy/Release/Deployment truth.
3. Every recommendation has explicit subject/context provenance.
4. Sensitive context is policy-classified before provider egress.
5. Secret values are not intentionally sent to models.
6. Source/log prompt injection cannot grant tool authority.
7. Model requests are not authorization.
8. Every tool invocation passes normal authz/policy.
9. AI holds no permanent privileged credentials.
10. AI cannot approve or sign protected releases.
11. AI cannot autonomously deploy production by default.
12. AI cannot change policy/authz/legal hold/root trust.
13. AI code/pipeline changes are drafts until normal source workflow applies them.
14. Generated pipeline config must pass canonical parser/IR.
15. AI test suggestions cannot reduce mandatory test floor.
16. AI cost optimization cannot weaken trust/residency/security.
17. AI review verdict does not count as protected human approval.
18. AI diagnosis cannot rewrite failure classifications/observations.
19. Migration assistant cannot silently claim unsupported semantics are equivalent.
20. Natural-language query compiles to constrained authorized query AST.
21. Raw SQL generated by model is never executed.
22. Recommendation freshness is checked before application.
23. Agentic runs have explicit capability/action/step/time/cost budgets.
24. Autonomous loops are bounded.
25. Tenant context and memory are isolated.
26. Private tenant data is not used for global model training by default.
27. Provider/model/prompt/schema versions are recorded.
28. Model upgrades are evaluated before default promotion.
29. Provider outage cannot break normal CI correctness.
30. Local-only/air-gap tenants can avoid remote egress entirely.
31. AI usage cost is metered separately.
32. Model output is treated as untrusted and schema/sanitization validated.
33. AI conversations/recommendations obey lifecycle/privacy policy.
34. Audit records sensitive/provider/action governance changes.
35. Forgeyard dogfoods AI assistance only as an optional advisory layer over its own deterministic systems.

---

# 353. Production Readiness Gates

Do not call AI assistance production-ready until:

```text
context authorization/redaction is enforced
cross-tenant retrieval tests pass
prompt-injection tool escalation is blocked
tool broker re-checks authz on every call
forbidden autonomous actions are machine-enforced
recommendation freshness works
structured-output validation is robust
model evaluation gates exist
provider outage leaves CI unaffected
adversarial/security/privacy tests pass
```

---

# 354. Architectural Invariants

1. AI is optional;
2. deterministic Forgeyard core does not depend on AI;
3. AI recommendations are not canonical truth;
4. AI never owns policy authority;
5. AI never owns signing authority;
6. AI never owns protected deployment authority;
7. evidence context is explicit/provenanced;
8. model confidence is not deterministic proof;
9. source/log content is untrusted model input;
10. prompt injection cannot grant capability;
11. model tool requests pass authz/policy broker;
12. AI has no permanent privileged credentials;
13. tool capability is narrowly scoped;
14. autonomous steps/time/cost are bounded;
15. generated code/config is draft until canonical validation/workflow;
16. AI cannot skip mandatory tests;
17. AI cannot weaken security/residency/trust for optimization;
18. AI review is advisory;
19. AI diagnosis cannot rewrite failure evidence;
20. migration AI cannot invent compatibility;
21. natural-language query cannot become unrestricted SQL;
22. tenant context is isolated;
23. private data is not global training data by default;
24. remote egress is explicit policy;
25. secret values are excluded from model context;
26. model/prompt versions are recorded;
27. provider outage cannot stop normal CI;
28. model output is untrusted/sanitized;
29. AI history obeys lifecycle/privacy rules;
30. Forgeyard dogfoods AI only as a bounded assistant.

---

# 355. Final Target Architecture

```text
                Canonical Forgeyard Evidence
                          │
                          ▼
                 Authorized Retrieval
                          │
                          ▼
                    Context Bundle
                          │
                          ▼
                     AI Provider
                          │
                          ▼
                 Typed Recommendation
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
           Explain      Draft       Suggest
              │           │           │
              └───────────┼───────────┘
                          ▼
                 Human / Policy Gate
                          │
                          ▼
              Canonical Forgeyard Action
```

Agentic execution:

```text
AiRecommendation
      ↓
bounded Agent Plan
      ↓
capability broker
      ↓
authz + policy
      ↓
low-risk allowed action
      ↓
canonical subsystem records result
```

Failure assistance:

```text
FailureObservation
+
logs/tests/context diffs
  ↓
AI summary/hypotheses
  ↓
evidence-linked next step
  ↓
reproduce / bisect / human decision
```

The key guarantee is:

> **Forgeyard can use AI aggressively to reduce cognitive load, surface patterns, propose optimizations, and automate bounded low-risk workflows without allowing probabilistic models to become hidden authorities. Every important action still terminates in deterministic Forgeyard types, policies, state machines, evidence, and explicit permissions.**

---

# 356. Extended Architecture Sequence

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
44 Pipeline Triggers / Schedules / Manual Dispatch / Event-Driven Execution
45 Cost Accounting / FinOps / Chargeback / Showback / Resource Economics
46 Data Lifecycle / Retention / Archival / Deletion / Legal Hold / Privacy Governance
47 CI/CD Migration / Import / Compatibility / Legacy-System Interoperability
48 Failure Diagnosis / Debugging / Reproduction / Bisect / Root-Cause Intelligence
49 Service Catalog / Component Ownership / Environment Inventory / Developer Portal
50 Reliability Engineering / SLO / Error Budget / Availability / Resilience Governance
51 Multi-Region Federation / Edge Sites / Disconnected Operation / Cross-Site Replication
52 Artifact Registry / Package Repository / OCI Distribution / Internal Software Distribution
53 Infrastructure-as-Code / Environment Provisioning / Preview Environments / Drift Reconciliation
54 Merge Queue / Speculative Integration / Batch Validation / Protected Target Submission
55 AI-Assisted CI Optimization / Engineering Copilot / Autonomous Recommendation Governance
```
