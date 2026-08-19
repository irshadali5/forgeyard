# 35 — Forgeyard Developer Experience, Local Dev Environment, CLI Workflows & Reproducible Workstation System Architecture

**Document type:** Core Developer Experience, Local Development, Workstation Bootstrap & CLI Workflow System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** repository initialization, developer onboarding, local workspace bootstrap, reproducible dev environments, local execution parity, CLI workflows, service emulation, local runners, environment variables, secrets, debugging, test fixtures, IDE integration, preview environments, developer diagnostics, and team-wide workflow standardization  
**Architecture style:** Local-first, reproducible, hermetic where practical, single-source-of-truth workflows, CI-parity-first, typed configuration, explicit environment contracts, fast feedback loops, no hidden host dependencies, and no second unofficial build/test path  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Pipeline IR, Hermetic/Reproducible Environments, CLI, Dioxus UI, Runner/Sandbox/Executor, Toolchains, Monorepo Intelligence, Tests, Benchmarks, SCM, Secrets, Search, and Self-Hosting. This subsystem makes Forgeyard pleasant and reliable for everyday development without weakening CI correctness.

---

# 1. Purpose

CI/CD systems often fail developers because the local workflow and CI workflow become two different systems.

Typical failure patterns:

```text
works on my machine
different toolchain locally
different env vars
different service versions
different test commands
hidden local dependencies
different build flags
different generated code
```

Forgeyard should instead make this possible:

```text
developer local run
≈
CI run
≈
release build
```

subject to explicit platform differences.

The central rule is:

> **Local development uses the same pipeline definitions, toolchain identities, environment contracts, test declarations, and artifact semantics as CI wherever practical.**

A second rule is:

> **Developer convenience may add shortcuts, caching, hot reload, and service emulation, but must not silently create a separate correctness path.**

A third rule is:

> **The repository should explain its own development environment declaratively enough that a new developer can bootstrap it with minimal undocumented host knowledge.**

---

# 2. Architectural Position

```text
                     Source Repository
                           │
                           ▼
                      forgeyard init
                           │
                           ▼
                  Development Manifest
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
        Toolchains      Services       Secrets
            │              │              │
            └──────────────┼──────────────┘
                           ▼
                  Local Dev Environment
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
          CLI Run        IDE/Editor     Dioxus UI
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                     Pipeline IR
                           │
                           ▼
                 Local Runner/Sandbox
                           │
                           ▼
                       Same Jobs
```

---

# 3. Goals

The subsystem MUST:

1. simplify project initialization;
2. simplify developer onboarding;
3. pin toolchains;
4. define dev environments;
5. define local services;
6. define local secrets;
7. support local execution;
8. support affected-only runs;
9. support hot reload where appropriate;
10. support local caching;
11. support debugging;
12. support test fixtures;
13. support data seeding;
14. support temporary environments;
15. support IDE integration;
16. support shell activation;
17. support CLI completions;
18. support project diagnostics;
19. support CI parity checks;
20. support offline development;
21. support multi-platform workflows;
22. support developer-specific overrides safely;
23. support local logs/traces;
24. support service emulation;
25. support local databases;
26. support local device development;
27. support preview environments;
28. support shared team presets;
29. support onboarding documentation generation;
30. remain deterministic/explainable.

---

# 4. Non-Goals

This subsystem does not:

```text
replace IDEs
replace shells
replace Cargo/npm/Go tooling
replace containers/VMs
force every local command through remote CI
hide platform-specific realities
```

---

# 5. Workspace Structure

```text
crates/dev/
├── forgeyard-dev/
├── forgeyard-dev-model/
├── forgeyard-dev-env/
├── forgeyard-dev-toolchain/
├── forgeyard-dev-service/
├── forgeyard-dev-run/
├── forgeyard-dev-shell/
├── forgeyard-dev-fixture/
├── forgeyard-dev-seed/
├── forgeyard-dev-debug/
├── forgeyard-dev-ide/
├── forgeyard-dev-preview/
├── forgeyard-dev-doctor/
└── forgeyard-dev-testkit/
```

CLI:

```text
apps/forgeyard-cli/
```

Repository config:

```text
.forgeyard/
├── pipeline.ron
├── dev.ron
├── services.ron
├── fixtures.ron
└── toolchains.ron
```

Use modules first; split crates only if runtime/security/dependency boundaries justify.

---

# 6. Development Environment Identity

```rust
pub struct DevEnvironmentId(Digest);
```

Content-derived from:

```text
toolchains
environment declaration
service definitions
dev profile
platform
```

---

# 7. Dev Manifest

```rust
pub struct DevManifest {
    pub toolchains: Vec<ToolchainRef>,
    pub environment: DevEnvironmentSpec,
    pub services: Vec<DevServiceSpec>,
    pub commands: Vec<DevCommand>,
    pub fixtures: Vec<FixtureSetRef>,
}
```

---

# 8. `forgeyard init`

Creates a minimal `.forgeyard/` structure.

---

# 9. Init Modes

```text
interactive
non-interactive
template
import-existing
```

---

# 10. Import Existing

Detect:

```text
Cargo workspace
package.json workspace
go.work
Gradle/Maven
CMake
```

and propose config.

---

# 11. Detection Is Advisory

Do not overwrite project files automatically without explicit action.

---

# 12. Init Output

Creates:

```text
pipeline.ron
dev.ron
toolchains.ron
```

plus optional examples.

---

# 13. Minimalism

Start with few files.

Avoid generating a giant config tree.

---

# 14. `forgeyard doctor`

First onboarding command.

---

# 15. Doctor Checks

```text
Forgeyard version
toolchains
OS dependencies
sandbox support
disk space
local DB/CAS
service ports
secrets availability
```

---

# 16. Actionable Diagnostics

Each failure should explain:

```text
what is missing
why it matters
how to fix
```

---

# 17. Dev Environment

```rust
pub struct DevEnvironmentSpec {
    pub toolchains: Vec<ToolchainRef>,
    pub env: BTreeMap<EnvKey, DevEnvValue>,
    pub working_directory: RepoRelativePath,
    pub services: Vec<DevServiceRef>,
}
```

---

# 18. DevEnvValue

```rust
pub enum DevEnvValue {
    Literal(BoundedString),
    Derived(DevDerivedValue),
    Secret(SecretRef),
}
```

---

# 19. Secret Values

Never committed.

---

# 20. Dev Secrets

Can come from:

```text
OS keyring
local encrypted Forgeyard secret store
external provider
interactive session
```

---

# 21. `.env`

Supported only as compatibility/import option.

---

# 22. `.env` Policy

Never default source of production secrets.

---

# 23. Toolchain Pinning

Use existing immutable ToolchainDescriptor.

---

# 24. Rust

`rust-toolchain.toml` integrates.

---

# 25. Native SDKs

Version/profile references.

---

# 26. Toolchain Bootstrap

`forgeyard dev setup`.

---

# 27. Setup

Installs/verifies allowed tools or explains manual requirements.

---

# 28. No Arbitrary Curl Scripts

Pinned/verifiable installers only.

---

# 29. Toolchain Cache

Local.

---

# 30. Offline Setup

Works if required toolchains/dependencies mirrored locally.

---

# 31. Dev Shell

```text
forgeyard dev shell
```

---

# 32. Shell Semantics

Launch shell with declared environment.

---

# 33. Shell Does Not Mutate Global User Environment

Critical.

---

# 34. Shell Lifetime

Process-scoped.

---

# 35. Supported Shells

```text
bash
zsh
fish
PowerShell
```

adapter behavior.

---

# 36. Environment Export

Optional:

```text
forgeyard dev env --format shell
```

---

# 37. Safe Output

Secret values not printed unless explicit secure mode.

---

# 38. Dev Command

```rust
pub struct DevCommand {
    pub name: DevCommandName,
    pub pipeline_job: Option<JobSelector>,
    pub command: Option<CommandSpec>,
    pub mode: DevCommandMode,
}
```

---

# 39. Prefer Pipeline Job Reference

Avoid duplicated local commands.

---

# 40. Example

```text
forgeyard dev test
```

maps to same test job definition.

---

# 41. Direct Command

Only for interactive convenience.

---

# 42. CI Parity Warning

If command has no pipeline equivalent, mark local-only.

---

# 43. Dev Command Mode

```rust
pub enum DevCommandMode {
    PipelineEquivalent,
    LocalOnly,
    Interactive,
}
```

---

# 44. PipelineEquivalent

Strong parity.

---

# 45. LocalOnly

Cannot be used as merge/release evidence.

---

# 46. Interactive

Long-running development process.

---

# 47. Local Run

```text
forgeyard run
```

uses local standalone execution by default.

---

# 48. Remote Run

```text
forgeyard run --remote
```

against distributed Forgeyard.

---

# 49. Same PipelinePlan

Where platform/environment equivalent.

---

# 50. Plan Diff

```text
forgeyard plan diff --local --remote
```

---

# 51. CI Parity Check

Highlights differences:

```text
runner platform
toolchain
env
services
network policy
sandbox
```

---

# 52. Parity Status

```rust
pub enum DevParity {
    Equivalent,
    EquivalentWithDeclaredDifferences,
    Divergent,
    Unknown,
}
```

---

# 53. Declared Difference

Example:

```text
local macOS
CI Linux
```

explicit.

---

# 54. Divergent

Undeclared config/toolchain mismatch.

---

# 55. Unknown

Cannot establish equivalence.

---

# 56. Local Runner

Runs on developer machine.

---

# 57. Trust Class

LocalDeveloper.

---

# 58. Local Evidence

Not automatically equivalent to trusted CI evidence.

---

# 59. Why

Developer can modify machine/environment.

---

# 60. Local Run Provenance

Still records:

```text
source snapshot
toolchain
host
job
```

---

# 61. Promotion

Local artifacts are not release-promoted by default.

---

# 62. Developer Artifact

Can inspect/use locally.

---

# 63. CI Rebuild

Required for protected release unless policy explicitly accepts local trusted workstation.

---

# 64. Local Cache

Use same CAS/cache semantics.

---

# 65. Local CAS

Default.

---

# 66. Shared Remote Cache

Optional.

---

# 67. Remote Cache Authentication

Normal auth.

---

# 68. Cross-Tenant Rules

Part 27.

---

# 69. Local Cache Poisoning

Local cache never automatically becomes trusted shared cache.

---

# 70. Push Cache

Explicit permission.

---

# 71. Dev Services

Applications often need:

```text
PostgreSQL
Redis-like cache
object storage
mail
mock APIs
```

---

# 72. DevServiceSpec

```rust
pub struct DevServiceSpec {
    pub id: DevServiceId,
    pub kind: DevServiceKind,
    pub version: DevServiceVersion,
    pub ports: Vec<ServicePort>,
    pub health: DevServiceHealthCheck,
    pub persistence: DevServicePersistence,
}
```

---

# 73. Service Runtime

Possible:

```text
native process
container
VM
embedded implementation
remote shared service
```

---

# 74. Forgeyard Does Not Require Docker

Critical.

---

# 75. Container Runtime

Optional adapter.

---

# 76. Native/Embedded

Preferred where practical.

---

# 77. Service Emulation

Examples:

```text
local object store emulator
mock SMTP
mock webhook receiver
```

---

# 78. Service Version Pinning

Exact.

---

# 79. Port Allocation

Dynamic where possible.

---

# 80. No Hardcoded Port Collisions

Forgeyard can allocate available local ports.

---

# 81. Service Discovery

Expose via derived env/config.

---

# 82. Service Health

Wait for readiness before dependent dev command.

---

# 83. Dev Service Lifecycle

```text
Stopped
Starting
Ready
Degraded
Failed
Stopping
```

---

# 84. `forgeyard dev up`

Starts declared services.

---

# 85. `forgeyard dev down`

Stops them.

---

# 86. `forgeyard dev status`

Shows health.

---

# 87. Persistent Services

Database can retain local state.

---

# 88. Ephemeral Mode

```text
forgeyard dev up --ephemeral
```

fresh state.

---

# 89. Fixtures

```rust
pub struct FixtureSet {
    pub id: FixtureSetId,
    pub version: FixtureVersion,
    pub resources: Vec<FixtureResource>,
}
```

---

# 90. Fixture Examples

```text
database rows
files
mock API responses
test accounts
sample artifacts
```

---

# 91. Fixtures Are Non-Secret

Secrets use SecretRef.

---

# 92. Fixture Determinism

Exact fixture digest.

---

# 93. Seed

```text
forgeyard dev seed <fixture>
```

---

# 94. Reset

```text
forgeyard dev reset
```

---

# 95. Reset Safety

Requires local/dev environment identity.

Never accidentally target production.

---

# 96. Environment Safety Tag

```rust
pub enum EnvironmentSafetyClass {
    Local,
    Disposable,
    SharedDevelopment,
    ProductionLike,
    Production,
}
```

---

# 97. Destructive Command

Checks class.

---

# 98. `--force`

Not enough for production.

Production requires authz/policy.

---

# 99. Local Database

Standalone can use Stoolap.

---

# 100. Application Dev DB

Project-defined service.

---

# 101. Database Migrations

Same migration artifacts/tools as CI/deploy where possible.

---

# 102. Dev Migration

```text
forgeyard dev migrate
```

---

# 103. Seed vs Migration

Separate.

---

# 104. Hot Reload

Interactive dev feature.

---

# 105. Hot Reload Mode

Can run local application with file watcher.

---

# 106. Watcher

Maps changed files via graph.

---

# 107. Rebuild Scope

Affected target only.

---

# 108. Hot Reload Is LocalOnly

Not CI evidence.

---

# 109. Dioxus

Can integrate Dioxus dev workflows.

---

# 110. Web/Desktop/Mobile

Platform-specific dev adapters.

---

# 111. Mobile Dev

Device Lab/local connected device support.

---

# 112. Local Device

Developer USB-connected Android device can register ephemeral local device session.

---

# 113. Production Device Lab

Separate trust.

---

# 114. Emulator

Can be DevService/DeviceTarget.

---

# 115. Debugging

```text
forgeyard dev debug <job>
```

---

# 116. Debug Profile

```rust
pub struct DebugProfile {
    pub debugger: DebuggerKind,
    pub target: JobSelector,
    pub breakpoints: DebugBreakpointSpec,
}
```

---

# 117. Debugger Adapters

```text
gdb
lldb
cdb/WinDbg integration
```

---

# 118. No Remote Debug Shell by Default

Critical.

---

# 119. Local Debug

Safe developer machine.

---

# 120. Remote Debug

Separate high-risk feature, not baseline.

---

# 121. Debug Build

Explicit profile.

---

# 122. Debug Evidence

Not release artifact.

---

# 123. Test Debugging

Re-run exact test locally with same inputs.

---

# 124. Reproduction Command

```text
forgeyard reproduce job <JobId>
```

---

# 125. Reproduce

Fetches:

```text
source snapshot
toolchain
config
declared inputs
```

subject to access.

---

# 126. Secrets

Not replayed automatically.

---

# 127. Secretful Job Reproduction

Requires fresh authorization/SecretRef resolution.

---

# 128. Reproduction Parity

Shows differences if local host cannot match.

---

# 129. Failed CI Reproduction

Core developer experience.

---

# 130. `forgeyard inspect job`

Shows:

```text
argv
env refs
toolchain
sandbox
inputs
outputs
```

safe/redacted.

---

# 131. Shell Into Failed Sandbox

Not baseline.

---

# 132. Why

Sandbox may contain secrets/unsafe state.

---

# 133. Better

Recreate clean debug sandbox from recorded inputs.

---

# 134. Interactive Sandbox

```text
forgeyard reproduce --interactive
```

local only.

---

# 135. IDE Integration

Provide machine-readable project info.

---

# 136. `forgeyard dev info --json`

Outputs:

```text
toolchains
generated env metadata
commands
services
targets
```

---

# 137. IDE Plugin

Future optional.

---

# 138. Baseline

Editor-independent CLI.

---

# 139. VS Code/Zed/JetBrains

Can consume generated launch/task config optionally.

---

# 140. Generated IDE Config

Should be derived, not canonical project truth.

---

# 141. Do Not Require Committing IDE-Specific Files

Optional.

---

# 142. Language Servers

Dev environment can expose correct toolchain paths/env.

---

# 143. Rust Analyzer

Uses workspace/toolchain.

---

# 144. C/C++

Can generate `compile_commands.json` from build system.

---

# 145. Generated File

Derived.

---

# 146. CLI Completion

```text
forgeyard completions bash
forgeyard completions zsh
forgeyard completions fish
forgeyard completions powershell
```

---

# 147. Command Discoverability

`forgeyard help`.

---

# 148. Contextual Help

```text
forgeyard help dev
forgeyard help run
```

---

# 149. Error UX

Errors should include:

```text
stable code
short explanation
suggested next command
```

---

# 150. No Giant Stack Trace for Normal User Error

Debug flag for internals.

---

# 151. CLI Output Modes

```text
human
json
ron
quiet
```

---

# 152. JSON

Automation interoperability.

---

# 153. RON

Forgeyard-native diagnostics/config.

---

# 154. Exit Codes

Stable classes.

---

# 155. Shell Scriptability

Commands should behave predictably.

---

# 156. Interactive Prompt

Only when TTY and action permits.

---

# 157. Non-TTY

Fails with explicit missing argument, never hangs.

---

# 158. CI Detection

CLI not special-casing correctness.

---

# 159. Developer Overrides

Need safe mechanism.

---

# 160. Local Override File

Example:

```text
.forgeyard/local.ron
```

gitignored.

---

# 161. Override Scope

Only fields marked developer-overridable.

---

# 162. Cannot Override

```text
required policy
production secret provider
release signing
tenant authz
```

---

# 163. DevOverridePolicy

```rust
pub struct DevOverridePolicy {
    pub allowed_fields: BTreeSet<ConfigFieldPath>,
}
```

---

# 164. Override Visibility

`forgeyard config explain`.

---

# 165. Plan Records Overrides

Local Run provenance includes override digest.

---

# 166. No Hidden Dotfile Magic

Critical.

---

# 167. Environment Variables

Explicit allowlist.

---

# 168. Host Env Import

Disabled by default except selected safe vars.

---

# 169. Why

Prevents accidental CI/local divergence.

---

# 170. Allowed Host Env

Examples:

```text
TERM
DISPLAY
WAYLAND_DISPLAY
SSH_AUTH_SOCK only if explicit interactive workflow
```

---

# 171. Build Jobs

No SSH agent by default.

---

# 172. PATH

Constructed from toolchain environment.

---

# 173. HOME

Controlled dev sandbox value where needed.

---

# 174. Locale/TZ

Declared/default deterministic for parity.

---

# 175. Time

Interactive apps may use real time; build/test profiles can fix/declare where relevant.

---

# 176. Network

Dev mode may be more permissive.

---

# 177. Difference Must Be Explicit

Parity report.

---

# 178. Dependency Fetch

Resolve/fetch phase can use network.

---

# 179. Build/Test

Can use hermetic network-denied mode.

---

# 180. Local Proxy

Optional dependency mirror.

---

# 181. Offline Dev

```text
forgeyard dev --offline
```

---

# 182. Offline Mode

Fails clearly on missing dependency/toolchain.

---

# 183. Workspace Bootstrap

```text
forgeyard dev bootstrap
```

---

# 184. Bootstrap Steps

```text
check toolchains
fetch dependencies
initialize local store/CAS
start services if configured
run migrations
install hooks if explicitly requested
```

---

# 185. Git Hooks

Optional.

---

# 186. Do Not Silently Install Hooks

Explicit.

---

# 187. Pre-Commit

Can run fast affected checks.

---

# 188. Pre-Push

Can run broader checks.

---

# 189. Hooks Are Convenience

Server-side CI/policy remains authority.

---

# 190. Local Affected Workflow

Part 34.

---

# 191. Example

```text
forgeyard run --affected
```

---

# 192. Fast Feedback

Changed targets only.

---

# 193. Full Check

```text
forgeyard run --full
```

---

# 194. Before Push Recommendation

Configurable.

---

# 195. Dev Profiles

```rust
pub enum DevProfile {
    Fast,
    Standard,
    Full,
    Debug,
    Offline,
}
```

---

# 196. Fast

Affected checks.

---

# 197. Standard

Normal local equivalent.

---

# 198. Full

CI-like comprehensive.

---

# 199. Debug

Instrumentation/debug build.

---

# 200. Profile Is Explicit

No hidden behavior.

---

# 201. Preview Environment

Ephemeral deployment for a Change Proposal/branch.

---

# 202. PreviewEnvironmentId

```rust
pub struct PreviewEnvironmentId(Ulid);
```

---

# 203. Preview Input

Exact:

```text
SourceSnapshotId
Artifact/Release candidate
DeploymentPlan
```

---

# 204. Preview Lifecycle

```text
Requested
Provisioning
Ready
Expired
Destroying
Destroyed
Failed
```

---

# 205. Preview Is Not Production

Separate EnvironmentClass.

---

# 206. Automatic Expiry

Mandatory.

---

# 207. Preview Secrets

Scoped, low-privilege.

---

# 208. Preview Data

Synthetic/non-production by default.

---

# 209. Production Data Clone

High risk, not baseline.

---

# 210. Preview URL

Generated provider endpoint.

---

# 211. SCM Integration

Post preview link to Change Proposal.

---

# 212. Preview Update

New source revision creates new deployment revision; may update same preview environment.

---

# 213. Preview Cleanup

Reconciled.

---

# 214. Cost/Quota

Part 27.

---

# 215. Entitlement

Part 30 if commercially gated.

---

# 216. Local Preview

Could run local services/UI.

---

# 217. Shared Dev Environment

Team-level remote environment.

---

# 218. Shared Environment Risks

State collisions.

---

# 219. Namespace

Per developer/branch.

---

# 220. Dev Identity

Principal-scoped.

---

# 221. Ephemeral Database

Per preview where feasible.

---

# 222. Seed

Fixtures.

---

# 223. Snapshot

Optional database template.

---

# 224. No Production Credentials

Critical.

---

# 225. Dev Logs

Structured.

---

# 226. Local Trace Viewer

Optional embedded view.

---

# 227. `forgeyard logs`

Same CLI semantics local/remote.

---

# 228. Trace Correlation

Local RunId/JobId.

---

# 229. Dev Telemetry

Local-only by default.

---

# 230. No Automatic Telemetry Upload Without configuration

Privacy.

---

# 231. Crash Diagnostics

Local bundle.

---

# 232. Support Bundle

Reuse Part 17 sanitization.

---

# 233. Onboarding

Repository can define onboarding steps.

---

# 234. Onboarding Manifest

```rust
pub struct OnboardingPlan {
    pub checks: Vec<OnboardingCheck>,
    pub commands: Vec<DevCommandName>,
}
```

---

# 235. `forgeyard dev onboarding`

Runs safe checks.

---

# 236. Examples

```text
bootstrap
doctor
build
unit test
sample run
```

---

# 237. Documentation

Can generate:

```text
forgeyard dev docs
```

summary from manifest.

---

# 238. Generated README Section

Optional.

---

# 239. Canonical Config

Manifest remains truth.

---

# 240. Team Presets

Version-controlled.

---

# 241. Personal Presets

Local.

---

# 242. Dev Environment Lock

Optional:

```text
forgeyard.dev.lock
```

---

# 243. Lock Contains

Resolved toolchain/service image/version identities.

---

# 244. Human Manifest

`dev.ron`.

---

# 245. Lock Regeneration

Explicit.

---

# 246. Lock Diff

Reviewable.

---

# 247. Platform Differences

Some tools/services unavailable on all platforms.

---

# 248. Capability Requirement

Dev command declares.

---

# 249. Unsupported Platform

Clear explanation.

---

# 250. Remote Fallback

Developer can run job on remote runner.

---

# 251. Example

macOS developer needs Linux GPU:

```text
forgeyard run job --remote
```

---

# 252. Remote Dev Execution

Uses exact local source snapshot upload.

---

# 253. Dirty Worktree

Explicit SourceSnapshot.

---

# 254. No Commit Required

Good developer UX.

---

# 255. Snapshot Upload

CAS.

---

# 256. Privacy

Only selected repository tree, respecting project rules.

---

# 257. Untracked Files

Explicit include policy.

---

# 258. Secrets

Never snapshot.

---

# 259. Remote Result

Artifacts/logs returned.

---

# 260. Remote Interactive Dev

Not baseline.

---

# 261. Remote Cache Warmup

Possible.

---

# 262. Developer Workspace State

```rust
pub struct DevWorkspaceState {
    pub repository: RepositoryId,
    pub source: SourceSnapshotId,
    pub environment: DevEnvironmentId,
    pub services: Vec<DevServiceState>,
}
```

---

# 263. State Location

Local metadata.

---

# 264. Not Shared Business Authority

---

# 265. Workspace Cleanup

```text
forgeyard dev clean
```

---

# 266. Clean Types

```text
build outputs
local cache
services
fixtures
all
```

---

# 267. Safe Defaults

Do not remove user source.

---

# 268. `--all`

Still never source repo.

---

# 269. Toolchain Cleanup

Separate.

---

# 270. Disk Pressure

Part 25.

---

# 271. Local CAS GC

Safe.

---

# 272. Developer Cache Stats

```text
forgeyard cache stats
```

---

# 273. Explain Cache Miss

Useful:

```text
forgeyard cache explain <job>
```

---

# 274. Reasons

```text
source changed
toolchain changed
env changed
config changed
uncacheable
```

---

# 275. Dev Performance

Startup latency matters.

---

# 276. Local Daemon

Can stay resident.

---

# 277. Single Binary

Runs daemon/UI/runner.

---

# 278. Fast IPC

Local loopback/IPC.

---

# 279. Incremental Graph Cache

Part 34.

---

# 280. Incremental Test Selection

Part 32, policy-safe.

---

# 281. Watch Mode

Use graph to select affected work.

---

# 282. Debounce

Bound file change storms.

---

# 283. File Watcher Overflow

Triggers conservative rescan.

---

# 284. Symlink

Respect source snapshot rules.

---

# 285. Generated Files

Avoid watch loops.

---

# 286. Dev Service Logs

Separate streams.

---

# 287. Port Exposure

Localhost by default.

---

# 288. Shared LAN

Explicit.

---

# 289. TLS

Remote/shared dev should use TLS/auth.

---

# 290. Local Browser UI

If exposed, session/CSRF rules.

---

# 291. Dioxus Native UI

Can call local app services/API.

---

# 292. CLI and UI Parity

Actions share application service APIs.

---

# 293. No UI-Only Dev Feature Logic

Critical.

---

# 294. Dev API

Potential local endpoints under normal API.

---

# 295. No Separate Unauthenticated Admin Socket

Critical.

---

# 296. Local IPC Authentication

OS user/process boundary or ephemeral token.

---

# 297. Shared Machine

Per-user data directories.

---

# 298. Multi-User Workstation

No cross-user local CAS/secret access by default.

---

# 299. Filesystem Permissions

Strict.

---

# 300. Local Secret Store

0600/user-protected equivalent.

---

# 301. Windows/macOS

Native secure storage.

---

# 302. Developer Analytics

Optional local stats:

```text
command duration
cache hit
test time
```

---

# 303. No Productivity Surveillance Baseline

Critical.

---

# 304. Team Analytics

Only aggregate CI data unless explicitly configured.

---

# 305. Developer Privacy

Local command history stays local by default.

---

# 306. Command History

Can store recent local runs.

---

# 307. Sensitive Args

Redacted.

---

# 308. SCM Workflow

CLI can:

```text
forgeyard change status
forgeyard run --affected
```

---

# 309. Commit/Push

Forgeyard does not replace Git CLI baseline.

---

# 310. Provider Actions

Optional.

---

# 311. Developer Check

```text
forgeyard check
```

Convenience alias to configured standard local validation.

---

# 312. Check Definition

Repository-configured.

---

# 313. No Universal Hidden Command Set

---

# 314. Exit Summary

Human-readable:

```text
12 jobs
8 cache hits
4 executed
0 failed
```

---

# 315. Machine Output

JSON.

---

# 316. Dev UX State Machine

```rust
pub enum DevEnvironmentState {
    Uninitialized,
    Ready,
    Degraded,
    Running,
    Broken,
}
```

---

# 317. Recovery

Doctor + setup can repair.

---

# 318. No Destructive Auto-Repair

Explicit.

---

# 319. Version Mismatch

CLI/daemon compatibility.

---

# 320. Local Auto-Restart

Allowed.

---

# 321. Upgrade

Part 26/25.

---

# 322. Dev Tool Compatibility

Manifest can define minimum Forgeyard version.

---

# 323. Configuration Validation

`forgeyard config check`.

---

# 324. Config Explain

Shows effective layered values and source.

---

# 325. Layer Order

Example:

```text
defaults
repository
workspace
user local
CLI flags
```

---

# 326. Security Fields

Some layers cannot override.

---

# 327. Effective Config Digest

Recorded in local run.

---

# 328. Developer Shell Reproducibility

Environment identity includes resolved config.

---

# 329. CI Parity Report

```rust
pub struct ParityReport {
    pub local: DevEnvironmentId,
    pub ci: ExecutionEnvironmentId,
    pub differences: Vec<ParityDifference>,
    pub status: DevParity,
}
```

---

# 330. Difference Classes

```text
Toolchain
Platform
Environment
Network
Service
Sandbox
SecretProvider
```

---

# 331. Declared Difference

Can be acceptable.

---

# 332. Undeclared Difference

Warning/fail strict check.

---

# 333. `forgeyard dev parity`

Core command.

---

# 334. CI Check

Optional:

```text
development manifest still matches pipeline assumptions
```

---

# 335. Testkit

```text
forgeyard-dev-testkit/src/
├── lib.rs
├── manifest.rs
├── environment.rs
├── service.rs
├── fixture.rs
├── parity.rs
├── cli.rs
└── assertions.rs
```

---

# 336. Unit Tests

Config layering.

---

# 337. Secret Test

Secret value not printed/exported.

---

# 338. Local Override Test

Cannot override protected field.

---

# 339. Parity Test

Toolchain mismatch detected.

---

# 340. Service Port Test

Dynamic allocation avoids collision.

---

# 341. Service Failure Test

Dependent command blocked with clear diagnostic.

---

# 342. Fixture Reset Test

Cannot target Production safety class.

---

# 343. Dirty Snapshot Test

Remote run captures correct local source.

---

# 344. Untracked File Test

Include policy explicit.

---

# 345. Offline Test

No hidden network access.

---

# 346. Local Cache Test

Local artifact not trusted as release artifact.

---

# 347. Reproduce Test

Failed CI job reconstructs declared inputs.

---

# 348. Secretful Reproduce Test

Requires fresh secret authorization.

---

# 349. Watch Overflow Test

Conservative rescan.

---

# 350. Multi-User Test

No local data cross-access.

---

# 351. Preview Expiry Test

Environment cleaned.

---

# 352. Preview Secret Test

No production credential.

---

# 353. Remote Fallback Test

Unsupported local platform can use matching remote runner.

---

# 354. CLI Non-TTY Test

No hanging prompts.

---

# 355. JSON Output Test

Stable schema/version.

---

# 356. Fuzzing

Fuzz dev config/fixture/service definitions.

---

# 357. Failure Injection

```text
toolchain missing
service crash
disk full
local DB corruption
network offline
runner unavailable
```

---

# 358. Load Test

Large monorepo local `--affected`.

---

# 359. Implementation Phase 1 — Dev Manifest/Doctor

Onboarding basics.

---

# 360. Phase 2 — Local Pipeline Execution

Same Pipeline IR/jobs.

---

# 361. Phase 3 — Toolchain/Dev Shell

Reproducibility.

---

# 362. Phase 4 — Dev Services/Fixtures

Local dependencies.

---

# 363. Phase 5 — Reproduce Failed Job

High developer value.

---

# 364. Phase 6 — Config Explain/Parity

Works-on-my-machine prevention.

---

# 365. Phase 7 — Watch/Affected

Fast feedback.

---

# 366. Phase 8 — IDE Metadata

Editor integration.

---

# 367. Phase 9 — Preview Environments

Team workflows.

---

# 368. Phase 10 — Mobile/Device Dev

Dioxus/device projects.

---

# 369. Phase 11 — Offline/Remote Fallback

Resilience.

---

# 370. Phase 12 — UX/Scale/Security Hardening

Production-grade developer experience.

---

# 371. Acceptance Tests

1. A new repository can initialize Forgeyard with a minimal config.
2. Existing Rust workspace import is advisory and non-destructive.
3. Dev toolchains are pinned/versioned.
4. Local dev environment has content-derived identity.
5. Local pipeline-equivalent commands reuse Pipeline IR rather than duplicate CI scripts.
6. Local-only commands are clearly marked and cannot become release evidence.
7. Local runner artifacts are not trusted production artifacts by default.
8. Developer host environment is not imported wholesale.
9. Secret values are never committed or printed by default.
10. Developer overrides cannot change protected security/release fields.
11. Local overrides are included in local provenance.
12. Dev shell does not mutate global user environment.
13. Forgeyard does not require Docker for local development.
14. Dev services have explicit versions and health checks.
15. Fixture reset cannot accidentally target production.
16. Local/remote plan differences are explainable.
17. CI parity can detect undeclared toolchain/environment differences.
18. Failed CI jobs can be reproduced from recorded immutable inputs.
19. Secretful reproduction requires fresh authorization.
20. Affected/watch workflows use Part 34 graph semantics.
21. File-watcher overflow causes conservative rescan.
22. Remote dev runs can use an exact dirty working-tree snapshot.
23. Source snapshots never include secret-store values.
24. Unsupported local platform can fall back to an authorized remote runner.
25. Preview environments bind exact source/artifact/deployment identities.
26. Preview environments expire and reconcile cleanup.
27. Preview environments do not receive production secrets by default.
28. CLI is usable non-interactively with stable structured output.
29. IDE integration is derived, not project authority.
30. Local logs/traces use same RunId/JobId semantics.
31. Shared-machine users cannot access each other's local secrets/data by default.
32. Offline mode exposes missing dependencies rather than silently using network.
33. Git hooks remain convenience, not server policy authority.
34. Standalone/distributed share developer workflow semantics.
35. Forgeyard dogfoods this developer environment for its own repository.

---

# 372. Production Readiness Gates

Do not call developer experience production-ready until:

```text
init/bootstrap/doctor are reliable
local Pipeline IR execution works
toolchain/env identity is reproducible
secret handling is safe
dev-service lifecycle is robust
failed-job reproduction works
parity reports identify local/CI divergence
affected/watch flow is conservative
preview cleanup is reconciled
CLI non-interactive behavior is stable
```

---

# 373. Architectural Invariants

1. local and CI workflows share the same underlying pipeline semantics;
2. developer convenience never creates hidden release authority;
3. local artifacts are untrusted for protected promotion by default;
4. toolchains are explicit/pinned;
5. host environment import is allowlisted;
6. secrets are SecretRefs, not committed plaintext;
7. local override scope is explicit;
8. protected fields cannot be overridden locally;
9. dev shell is process-scoped;
10. Docker is optional, not required architecture;
11. service versions are explicit;
12. destructive fixture actions respect environment safety class;
13. CI parity differences are visible;
14. failed jobs reproduce from immutable inputs;
15. secretful reproduction requires fresh authorization;
16. hot reload/watch are local convenience only;
17. affected execution uses conservative graph semantics;
18. remote dev uses exact source snapshots;
19. preview environments are ephemeral and scoped;
20. production secrets do not enter previews by default;
21. IDE metadata is derived;
22. CLI structured output is stable/versioned;
23. no hidden unauthenticated local admin path;
24. shared-machine local data is isolated;
25. offline mode is explicit;
26. hooks are convenience only;
27. local telemetry stays local by default;
28. standalone/distributed share UX semantics;
29. developer workflows remain explainable;
30. Forgeyard dogfoods its own development system.

---

# 374. Final Target Architecture

```text
                     Repository
                         │
                         ▼
                   Dev Manifest
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
        Toolchains     Services     Fixtures
            │            │            │
            └────────────┼────────────┘
                         ▼
                 Dev Environment
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
          CLI          IDE/UI       Watch
            │            │            │
            └────────────┼────────────┘
                         ▼
                    Pipeline IR
                         │
                         ▼
                Local/Remote Runner
                         │
                         ▼
                  Same Job Semantics
```

---

# 375. Final Architectural Position

Onboarding:

```text
clone source
  ↓
forgeyard dev bootstrap
  ↓
doctor
  ↓
toolchains/services ready
  ↓
forgeyard check
```

Local/CI parity:

```text
same source snapshot
+
same pipeline/job definitions
+
same toolchain identity
+
declared environment differences
  ↓
ParityReport
```

Failure reproduction:

```text
failed CI JobId
  ↓
source/toolchain/inputs
  ↓
fresh local sandbox
  ↓
fresh secret authorization if needed
  ↓
interactive/debug reproduction
```

Preview:

```text
exact ChangeProposal revision
  ↓
build/package
  ↓
ephemeral deployment
  ↓
scoped secrets/data
  ↓
automatic expiry/cleanup
```

The key guarantee is:

> **Forgeyard gives developers fast local workflows without allowing local convenience to drift away from CI truth. The same source snapshots, toolchains, pipeline semantics, test declarations, and artifact identities underpin both environments; differences are explicit and inspectable rather than hidden behind “works on my machine.”**

---

# 376. Extended Architecture Sequence

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
```
