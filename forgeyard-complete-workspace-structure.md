# Forgeyard Complete Workspace Structure & Repository Architecture

**Architecture style:** Modular monolith  
**Repository:** One canonical Git repository  
**Workspace:** One Rust Cargo workspace  
**Internal source model:** VCS-neutral `SourceSnapshotId`  
**Primary implementation:** Rust  
**UI:** Dioxus  
**Server:** Axum  
**Internal protocol:** Postcard  
**Human configuration:** RON  
**Standalone metadata:** Stoolap  
**Distributed metadata:** PostgreSQL / Neon

This document consolidates the workspace/repository layout for all Forgeyard systems designed so far: core CI/CD, hermetic/reproducible builds, VCS-neutral source control, Change Proposal/review/integration queue, Rust, C/C++, Go, JavaScript/TypeScript, Python, JVM/Java/Kotlin, Dart/Flutter, Swift, Assembly/native tooling, platform adapters, CAS, scheduling, runners, sandboxes, release/deployment, policy/security, observability, testing, and self-hosting.

---

# 1. Repository Rule

Forgeyard should stay physically monolithic and logically modular:

```text
one Git repository
+ one Cargo workspace
+ many capability-oriented crates
+ strict dependency directions
+ platform/ecosystem adapters
= modular monolith
```

Do **not** split every language or subsystem into separate repositories initially. Rust's compiler should be able to validate the entire architecture in one workspace.

---

# 2. Top-Level Tree

```text
forgeyard/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── deny.toml
├── typos.toml
├── architecture.ron
├── .editorconfig
├── .gitignore
├── .gitattributes
├── .dockerignore
├── LICENSE
├── LICENSES/
├── README.md
├── SECURITY.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── GOVERNANCE.md
├── MAINTAINERS.md
├── RELEASES.md
├── CHANGELOG.md
├── ROADMAP.md
│
├── .forgeyard/
├── apps/
├── crates/
├── ecosystems/
├── native/
├── platforms/
├── protocols/
├── schemas/
├── config/
├── policies/
├── migrations/
├── packaging/
├── deploy/
├── infra/
├── assets/
├── web/
├── fixtures/
├── examples/
├── tests/
├── benches/
├── fuzz/
├── tools/
├── xtask/
├── scripts/
├── docs/
├── adr/
├── rfcs/
├── contrib/
└── vendor/
```

---

# 3. Root Files

## `Cargo.toml`
Workspace members, shared dependencies, lints, profiles, resolver, feature coordination.

## `Cargo.lock`
Canonical lock for Forgeyard itself; required for release/self-hosting builds.

## `rust-toolchain.toml`
Pins Forgeyard's own Rust toolchain and components.

## `rustfmt.toml`
Formatting rules.

## `clippy.toml`
Clippy configuration.

## `deny.toml`
Dependency/license/source policy.

## `architecture.ron`
Machine-checkable layer/dependency rules for workspace architecture.

## `.editorconfig`
Cross-editor formatting defaults.

## `.gitignore`, `.gitattributes`
Git-specific repository behavior. Git is Forgeyard's development VCS, but Forgeyard's internal source model remains VCS-neutral.

## `README.md`
Entry point.

## `SECURITY.md`
Security reporting and supported releases.

## `CONTRIBUTING.md`
Development workflow, testing, architecture rules, Change Proposal workflow.

## `GOVERNANCE.md`, `MAINTAINERS.md`
Project governance and ownership.

## `RELEASES.md`, `CHANGELOG.md`, `ROADMAP.md`
Release operations, public change history, roadmap.

---

# 4. Self-Hosting Configuration — `.forgeyard/`

```text
.forgeyard/
├── forgeyard.ron
├── pipeline.ron
├── policy.ron
├── ownership.ron
├── release.ron
├── toolchains.ron
├── lock/
│   ├── rust.ron
│   ├── tools.ron
│   └── source.ron
└── templates/
```

### `forgeyard.ron`
Repository/project configuration.

### `pipeline.ron`
Forgeyard builds/tests Forgeyard.

### `policy.ron`
Required checks, security rules, queue policy.

### `ownership.ron`
Forgeyard-native ownership model.

### `release.ron`
Signing/promotion/release requirements.

### `toolchains.ron`
Locked required toolchain declarations.

---

# 5. Runnable Applications — `apps/`

```text
apps/
├── forgeyard/
├── forgeyard-daemon/
├── forgeyard-agent/
├── forgeyard-cli/
├── forgeyard-ui/
├── forgeyard-worker/
├── forgeyard-signing-worker/
├── forgeyard-device-agent/
└── forgeyard-migration/
```

## `apps/forgeyard/`
Standalone all-in-one Mode 1 binary.

```text
src/
├── main.rs
├── bootstrap.rs
├── mode.rs
├── runtime.rs
├── shutdown.rs
└── diagnostics.rs
```

## `apps/forgeyard-daemon/`
Distributed control plane.

```text
src/
├── main.rs
├── bootstrap.rs
├── api.rs
├── quic.rs
├── websocket.rs
├── reconciliation.rs
├── leadership.rs
├── health.rs
└── shutdown.rs
```

## `apps/forgeyard-agent/`
Distributed runner agent.

```text
src/
├── main.rs
├── bootstrap.rs
├── registration.rs
├── capabilities.rs
├── lease_loop.rs
├── execution.rs
├── heartbeat.rs
├── reconnect.rs
└── shutdown.rs
```

## `apps/forgeyard-cli/`

```text
src/
├── main.rs
├── cli.rs
├── config.rs
├── output.rs
└── commands/
    ├── init.rs
    ├── run.rs
    ├── plan.rs
    ├── status.rs
    ├── logs.rs
    ├── artifacts.rs
    ├── cache.rs
    ├── runner.rs
    ├── release.rs
    ├── deploy.rs
    ├── health.rs
    ├── doctor.rs
    ├── vcs.rs
    ├── change.rs
    ├── rust.rs
    ├── cpp.rs
    ├── go.rs
    ├── python.rs
    ├── node.rs
    ├── jvm.rs
    ├── flutter.rs
    └── swift.rs
```

## `apps/forgeyard-ui/`
Dioxus client; never authoritative for policy/business rules.

```text
src/
├── main.rs
├── app.rs
├── router.rs
├── state.rs
├── api.rs
├── session.rs
├── platform.rs
├── pages/
├── views/
├── components/
├── hooks/
└── theme/
```

## `apps/forgeyard-signing-worker/`
Restricted signing boundary; cannot execute arbitrary builds.

## `apps/forgeyard-device-agent/`
Controls Android/Apple/embedded test devices.

## `apps/forgeyard-worker/`
Optional non-build background/reconciliation process.

## `apps/forgeyard-migration/`
Controlled administrative migration utility.

---

# 6. Core Crate Areas — `crates/`

```text
crates/
├── core/
├── ids/
├── digest/
├── time/
├── error/
├── config/
├── protocol/
├── project/
├── source/
├── run/
├── lease/
├── store/
├── cas/
├── pipeline/
├── scheduler/
├── runner/
├── sandbox/
├── executor/
├── toolchain/
├── hermetic/
├── reproducibility/
├── cache/
├── artifact/
├── package/
├── release/
├── deploy/
├── policy/
├── identity/
├── authz/
├── secrets/
├── audit/
├── events/
├── logs/
├── telemetry/
├── health/
├── reconciliation/
├── analysis/
├── device/
├── notification/
├── vcs/
├── change/
├── scm/
├── supply-chain/
├── security/
├── api/
├── transport/
├── coordination/
├── rbe/
└── plugin/
```

---

# 7. Primitive Layer

## `crates/core/forgeyard-core/`

```text
src/
├── lib.rs
├── mode.rs
├── tenant.rs
├── project.rs
├── capability.rs
├── state.rs
└── invariant.rs
```

No SQL, Axum, Dioxus, Git, or ecosystem dependencies.

## `crates/ids/forgeyard-ids/`

```text
src/
├── lib.rs
├── project.rs
├── pipeline.rs
├── run.rs
├── job.rs
├── runner.rs
├── lease.rs
├── artifact.rs
├── release.rs
├── deployment.rs
├── source.rs
├── repository.rs
├── change.rs
├── policy.rs
└── principal.rs
```

Use newtypes for all identities.

## `crates/digest/forgeyard-digest/`

```text
src/
├── lib.rs
├── blake3.rs
├── sha256.rs
├── alias.rs
├── encoding.rs
└── serde.rs
```

## `crates/time/forgeyard-time/`

```text
src/
├── lib.rs
├── timestamp.rs
├── duration.rs
├── deadline.rs
├── clock.rs
└── test_clock.rs
```

## `crates/error/forgeyard-error/`

```text
src/
├── lib.rs
├── code.rs
├── diagnostic.rs
├── retry.rs
└── user_message.rs
```

---

# 8. Config / Protocol

```text
crates/config/
├── forgeyard-config/
├── forgeyard-config-loader/
├── forgeyard-config-schema/
└── forgeyard-config-policy/
```

```text
crates/protocol/
├── forgeyard-envelope/
├── forgeyard-wire/
├── forgeyard-api-model/
└── forgeyard-version/
```

Internal messages: Postcard.  
Human configuration: RON.  
Public interoperability: JSON where required.

---

# 9. Store

```text
crates/store/
├── forgeyard-store/
├── forgeyard-store-stoolap/
├── forgeyard-store-postgres/
├── forgeyard-store-migration/
└── forgeyard-store-testkit/
```

`forgeyard-store` contains traits only.

`forgeyard-store-stoolap` implements Mode 1.

`forgeyard-store-postgres` implements shared distributed metadata.

No domain service directly writes SQL.

### Postgres adapter internal tree

```text
src/
├── lib.rs
├── pool.rs
├── transaction.rs
├── row/
│   ├── run.rs
│   ├── job.rs
│   ├── vcs.rs
│   ├── change.rs
│   ├── artifact.rs
│   └── release.rs
└── query/
    ├── run.rs
    ├── job.rs
    ├── vcs.rs
    ├── change.rs
    ├── artifact.rs
    └── release.rs
```

---

# 10. CAS

```text
crates/cas/
├── forgeyard-cas/
├── forgeyard-cas-local/
├── forgeyard-cas-s3/
├── forgeyard-cas-gcs/
├── forgeyard-cas-azure/
├── forgeyard-cas-iroh/
├── forgeyard-cas-tiered/
├── forgeyard-cas-gc/
├── forgeyard-cas-transfer/
└── forgeyard-cas-testkit/
```

Iroh is acceleration, never durability/authority.

---

# 11. Pipeline

```text
crates/pipeline/
├── forgeyard-pipeline-ir/
├── forgeyard-pipeline-parser/
├── forgeyard-pipeline-normalize/
├── forgeyard-pipeline-dag/
├── forgeyard-pipeline-matrix/
├── forgeyard-pipeline-template/
├── forgeyard-pipeline-validate/
├── forgeyard-pipeline-plan/
└── forgeyard-pipeline-import/
```

`forgeyard-pipeline-ir/src/`:

```text
lib.rs
pipeline.rs
stage.rs
job.rs
step.rs
dependency.rs
condition.rs
environment.rs
artifact.rs
cache.rs
source.rs
```

---

# 12. Run / Job / Lease

```text
crates/run/
├── forgeyard-run/
├── forgeyard-run-model/
├── forgeyard-run-state/
├── forgeyard-run-service/
├── forgeyard-job/
├── forgeyard-job-state/
└── forgeyard-job-attempt/
```

```text
crates/lease/
├── forgeyard-lease/
├── forgeyard-job-lease/
├── forgeyard-device-lease/
└── forgeyard-integration-lease/
```

Job states:

```text
Pending
Eligible
Leased
Preparing
Running
UploadingOutputs
Succeeded
Failed
Cancelled
TimedOut
Lost
```

---

# 13. Scheduler / Runner

```text
crates/scheduler/
├── forgeyard-scheduler/
├── forgeyard-scheduler-model/
├── forgeyard-scheduler-placement/
├── forgeyard-scheduler-score/
├── forgeyard-scheduler-lease/
├── forgeyard-scheduler-resource/
├── forgeyard-scheduler-queue/
└── forgeyard-scheduler-testkit/
```

```text
crates/runner/
├── forgeyard-runner/
├── forgeyard-runner-model/
├── forgeyard-runner-capability/
├── forgeyard-runner-registration/
├── forgeyard-runner-heartbeat/
├── forgeyard-runner-workspace/
└── forgeyard-runner-log/
```

Scheduler:
- filters hard capabilities;
- scores remaining runners;
- issues expiring leases;
- applies resource/backpressure/fairness policy.

---

# 14. Sandbox / Executor

```text
crates/sandbox/
├── forgeyard-sandbox/
├── forgeyard-sandbox-linux/
├── forgeyard-sandbox-windows/
├── forgeyard-sandbox-apple/
├── forgeyard-sandbox-android/
├── forgeyard-sandbox-policy/
└── forgeyard-sandbox-testkit/
```

```text
crates/executor/
├── forgeyard-executor/
├── forgeyard-executor-process/
├── forgeyard-executor-container/
├── forgeyard-executor-windows/
├── forgeyard-executor-apple/
├── forgeyard-executor-android/
├── forgeyard-executor-confidential/
└── forgeyard-executor-testkit/
```

---

# 15. Toolchain / Hermetic Foundation

```text
crates/toolchain/
├── forgeyard-toolchain/
├── forgeyard-toolchain-model/
├── forgeyard-toolchain-store/
├── forgeyard-toolchain-resolver/
├── forgeyard-toolchain-trust/
├── forgeyard-toolchain-mirror/
└── forgeyard-toolchain-testkit/
```

```text
crates/hermetic/
├── forgeyard-hermetic/
├── forgeyard-derivation/
├── forgeyard-functional-store/
├── forgeyard-lock/
├── forgeyard-realizer/
├── forgeyard-substituter/
├── forgeyard-environment/
├── forgeyard-impurity/
└── forgeyard-hermetic-testkit/
```

`forgeyard-derivation/src/`:

```text
lib.rs
id.rs
input.rs
output.rs
platform.rs
environment.rs
builder.rs
canonical.rs
```

---

# 16. Reproducibility

```text
crates/reproducibility/
├── forgeyard-reproducibility/
├── forgeyard-reproducer/
├── forgeyard-repro-diff/
├── forgeyard-normalize-output/
└── forgeyard-repro-report/
```

`forgeyard-reproducibility/src/`:

```text
lib.rs
level.rs
request.rs
evidence.rs
compare.rs
policy.rs
mismatch.rs
report.rs
```

---

# 17. Artifact / Cache / Package

```text
crates/cache/
├── forgeyard-cache/
├── forgeyard-action-cache/
├── forgeyard-cache-key/
├── forgeyard-cache-policy/
└── forgeyard-cache-analysis/
```

```text
crates/artifact/
├── forgeyard-artifact/
├── forgeyard-artifact-store/
├── forgeyard-artifact-index/
├── forgeyard-artifact-retention/
└── forgeyard-artifact-download/
```

```text
crates/package/
├── forgeyard-package/
├── forgeyard-package-archive/
├── forgeyard-package-deb/
├── forgeyard-package-rpm/
├── forgeyard-package-msi/
├── forgeyard-package-msix/
├── forgeyard-package-dmg/
├── forgeyard-package-pkg/
├── forgeyard-package-apk/
├── forgeyard-package-aab/
├── forgeyard-package-oci/
├── forgeyard-package-wasm/
└── forgeyard-package-fypkg/
```

---

# 18. Release / Deploy

```text
crates/release/
├── forgeyard-release/
├── forgeyard-release-model/
├── forgeyard-release-candidate/
├── forgeyard-release-approval/
├── forgeyard-release-promotion/
├── forgeyard-release-signing/
└── forgeyard-release-publish/
```

```text
crates/deploy/
├── forgeyard-deploy/
├── forgeyard-deploy-model/
├── forgeyard-deploy-plan/
├── forgeyard-deploy-state/
├── forgeyard-deploy-rollback/
└── forgeyard-deploy-provider/
```

Build once → verify → promote exact bytes.

---

# 19. Policy / Identity / Authorization / Secrets

```text
crates/policy/
├── forgeyard-policy/
├── forgeyard-policy-model/
├── forgeyard-policy-engine/
├── forgeyard-policy-loader/
├── forgeyard-policy-decision/
├── forgeyard-policy-exception/
└── forgeyard-policy-testkit/
```

```text
crates/identity/
├── forgeyard-identity/
├── forgeyard-identity-local/
├── forgeyard-identity-oidc/
├── forgeyard-identity-saml/
├── forgeyard-identity-scim/
├── forgeyard-principal/
└── forgeyard-session/
```

```text
crates/authz/
├── forgeyard-authz/
├── forgeyard-permission/
├── forgeyard-role/
└── forgeyard-authz-policy/
```

```text
crates/secrets/
├── forgeyard-secret/
├── forgeyard-secret-provider/
├── forgeyard-secret-local/
├── forgeyard-secret-vault/
├── forgeyard-secret-cloud/
└── forgeyard-secret-zeroize/
```

---

# 20. Audit / Events / Reconciliation / Telemetry

```text
crates/audit/
├── forgeyard-audit/
├── forgeyard-audit-event/
├── forgeyard-audit-store/
├── forgeyard-audit-export/
└── forgeyard-audit-integrity/
```

```text
crates/events/
├── forgeyard-event/
├── forgeyard-event-log/
├── forgeyard-event-publisher/
├── forgeyard-event-subscriber/
└── forgeyard-event-testkit/
```

```text
crates/reconciliation/
├── forgeyard-reconcile/
├── forgeyard-reconcile-run/
├── forgeyard-reconcile-runner/
├── forgeyard-reconcile-artifact/
├── forgeyard-reconcile-provider/
└── forgeyard-reconcile-change/
```

```text
crates/telemetry/
├── forgeyard-telemetry/
├── forgeyard-tracing/
├── forgeyard-metrics/
├── forgeyard-logging/
└── forgeyard-otlp/
```

```text
crates/health/
├── forgeyard-health/
├── forgeyard-health-check/
├── forgeyard-doctor/
└── forgeyard-readiness/
```

---

# 21. Analysis / Device / Notifications

```text
crates/analysis/
├── forgeyard-analysis/
├── forgeyard-change-impact/
├── forgeyard-predictive-cache/
├── forgeyard-build-graph/
└── forgeyard-regression/
```

```text
crates/device/
├── forgeyard-device/
├── forgeyard-device-model/
├── forgeyard-device-lease/
├── forgeyard-device-android/
├── forgeyard-device-apple/
├── forgeyard-device-embedded/
└── forgeyard-device-testkit/
```

```text
crates/notification/
├── forgeyard-notification/
├── forgeyard-notification-model/
├── forgeyard-notification-email/
├── forgeyard-notification-web/
└── forgeyard-notification-testkit/
```

---

# 22. VCS-Neutral Source System

```text
crates/vcs/
├── forgeyard-vcs/
├── forgeyard-vcs-model/
├── forgeyard-vcs-source/
├── forgeyard-vcs-snapshot/
├── forgeyard-vcs-canonical/
├── forgeyard-vcs-graph/
├── forgeyard-vcs-diff/
├── forgeyard-vcs-provenance/
├── forgeyard-vcs-signature/
├── forgeyard-vcs-auth/
├── forgeyard-vcs-cache/
├── forgeyard-vcs-events/
├── forgeyard-vcs-git/
├── forgeyard-vcs-mercurial/
├── forgeyard-vcs-fossil/
├── forgeyard-vcs-breezy/
├── forgeyard-vcs-jujutsu/
├── forgeyard-vcs-darcs/
├── forgeyard-vcs-pijul/
├── forgeyard-vcs-local/
└── forgeyard-vcs-archive/
```

## `forgeyard-vcs-model/src/`

```text
lib.rs
kind.rs
repository.rs
revision.rs
change_id.rs
reference.rs
capability.rs
metadata.rs
snapshot.rs
provenance.rs
error.rs
```

## `forgeyard-vcs-snapshot/src/`

```text
lib.rs
snapshot.rs
tree.rs
blob.rs
entry.rs
composite.rs
policy.rs
```

## `forgeyard-vcs-canonical/src/`

```text
lib.rs
path.rs
unicode.rs
case_collision.rs
symlink.rs
serialize.rs
hash.rs
```

## Git adapter

```text
forgeyard-vcs-git/src/
├── lib.rs
├── backend.rs
├── detect.rs
├── repository.rs
├── refs.rs
├── revision.rs
├── tree.rs
├── worktree.rs
├── diff.rs
├── submodule.rs
├── signature.rs
├── fetch.rs
├── partial.rs
├── shallow.rs
└── auth.rs
```

## Mercurial adapter

```text
forgeyard-vcs-mercurial/src/
├── lib.rs
├── backend.rs
├── repository.rs
├── changeset.rs
├── bookmark.rs
├── named_branch.rs
├── tag.rs
├── phase.rs
├── revset.rs
├── working_copy.rs
├── subrepo.rs
└── diff.rs
```

## Fossil adapter

```text
forgeyard-vcs-fossil/src/
├── lib.rs
├── backend.rs
├── artifact.rs
├── checkin.rs
├── manifest.rs
├── baseline.rs
├── refs.rs
└── sync.rs
```

## Breezy adapter

```text
forgeyard-vcs-breezy/src/
├── lib.rs
├── backend.rs
├── repository.rs
├── branch.rs
├── revision.rs
├── working_tree.rs
└── diff.rs
```

## Jujutsu adapter

```text
forgeyard-vcs-jujutsu/src/
├── lib.rs
├── backend.rs
├── commit.rs
├── change.rs
├── bookmark.rs
├── divergence.rs
├── rewrite.rs
├── git_backend.rs
└── diff.rs
```

## Darcs adapter

```text
forgeyard-vcs-darcs/src/
├── lib.rs
├── backend.rs
├── patch.rs
├── context.rs
├── materialize.rs
└── diff.rs
```

## Pijul adapter

```text
forgeyard-vcs-pijul/src/
├── lib.rs
├── backend.rs
├── change.rs
├── channel.rs
├── conflict.rs
├── materialize.rs
└── diff.rs
```

---

# 23. Change Proposal System

```text
crates/change/
├── forgeyard-change/
├── forgeyard-change-model/
├── forgeyard-change-store/
├── forgeyard-change-service/
├── forgeyard-change-review/
├── forgeyard-change-comment/
├── forgeyard-change-approval/
├── forgeyard-change-ownership/
├── forgeyard-change-check/
├── forgeyard-change-policy/
├── forgeyard-change-mergeability/
├── forgeyard-change-integration/
├── forgeyard-change-queue/
├── forgeyard-change-provider/
├── forgeyard-change-events/
├── forgeyard-change-notification/
└── forgeyard-change-audit/
```

## `forgeyard-change-model/src/`

```text
lib.rs
proposal.rs
revision.rs
source.rs
target.rs
lifecycle.rs
status.rs
review.rs
approval.rs
discussion.rs
comment.rs
suggestion.rs
ownership.rs
check.rs
policy.rs
mergeability.rs
integration.rs
queue.rs
provider.rs
event.rs
error.rs
```

## Review

```text
forgeyard-change-review/src/
├── lib.rs
├── service.rs
├── verdict.rs
├── stale.rs
├── scope.rs
└── eligibility.rs
```

## Comment

```text
forgeyard-change-comment/src/
├── lib.rs
├── thread.rs
├── anchor.rs
├── relocation.rs
├── suggestion.rs
└── moderation.rs
```

## Approval

```text
forgeyard-change-approval/src/
├── lib.rs
├── approval.rs
├── quorum.rs
├── invalidation.rs
├── path_scope.rs
└── separation_of_duties.rs
```

## Ownership

```text
forgeyard-change-ownership/src/
├── lib.rs
├── rule.rs
├── matcher.rs
├── resolver.rs
├── codeowners.rs
├── domain.rs
└── precedence.rs
```

## Checks

```text
forgeyard-change-check/src/
├── lib.rs
├── check.rs
├── planner.rs
├── evidence.rs
├── aggregate.rs
├── stale.rs
└── reuse.rs
```

## Mergeability

```text
forgeyard-change-mergeability/src/
├── lib.rs
├── evaluator.rs
├── reason.rs
├── evidence_digest.rs
└── cache.rs
```

## Integration

```text
forgeyard-change-integration/src/
├── lib.rs
├── backend.rs
├── candidate.rs
├── strategy.rs
├── submit.rs
├── verification.rs
├── conflict.rs
└── provenance.rs
```

## Queue

```text
forgeyard-change-queue/src/
├── lib.rs
├── queue.rs
├── entry.rs
├── lease.rs
├── serial.rs
├── speculative.rs
├── batch.rs
├── fairness.rs
├── retry.rs
└── reconciliation.rs
```

---

# 24. SCM Provider Layer

SCM provider != VCS.

```text
crates/scm/
├── forgeyard-scm/
├── forgeyard-scm-model/
├── forgeyard-scm-webhook/
├── forgeyard-scm-github/
├── forgeyard-scm-gitlab/
├── forgeyard-scm-forgejo/
├── forgeyard-scm-gitea/
├── forgeyard-scm-sourcehut/
└── forgeyard-scm-generic/
```

GitHub adapter internal example:

```text
src/
├── lib.rs
├── client.rs
├── auth.rs
├── webhook.rs
├── repository.rs
├── pull_request.rs
├── review.rs
├── check.rs
├── comment.rs
├── status.rs
└── error.rs
```

GitLab keeps MR-specific terminology inside its own adapter only.

---

# 25. Ecosystem API

```text
ecosystems/api/
├── forgeyard-ecosystem-api/
├── forgeyard-ecosystem-model/
├── forgeyard-ecosystem-detect/
└── forgeyard-ecosystem-testkit/
```

Main trait lives in:

```text
forgeyard-ecosystem-api/src/adapter.rs
```

High-level ecosystem crates depend on shared native/platform capability APIs, not each other's internals.

---

# 26. Rust Ecosystem

```text
ecosystems/rust/
├── forgeyard-rust/
├── forgeyard-rust-model/
├── forgeyard-rust-detect/
├── forgeyard-rust-toolchain/
├── forgeyard-rust-cargo/
├── forgeyard-rust-lock/
├── forgeyard-rust-registry/
├── forgeyard-rust-features/
├── forgeyard-rust-build-script/
├── forgeyard-rust-proc-macro/
├── forgeyard-rust-native/
├── forgeyard-rust-bindgen/
├── forgeyard-rust-cross/
├── forgeyard-rust-analysis/
├── forgeyard-rust-test/
├── forgeyard-rust-doc/
├── forgeyard-rust-miri/
├── forgeyard-rust-fuzz/
├── forgeyard-rust-coverage/
├── forgeyard-rust-package/
├── forgeyard-rust-publish/
└── forgeyard-rust-selfhost/
```

Representative Cargo adapter files:

```text
src/
├── lib.rs
├── metadata.rs
├── workspace.rs
├── package.rs
├── target.rs
├── resolver.rs
├── lock.rs
├── offline.rs
├── messages.rs
├── config.rs
└── error.rs
```

---

# 27. C/C++ Ecosystem

```text
ecosystems/cpp/
├── forgeyard-cpp/
├── forgeyard-cpp-model/
├── forgeyard-cpp-detect/
├── forgeyard-cpp-toolchain/
├── forgeyard-cpp-gcc/
├── forgeyard-cpp-clang/
├── forgeyard-cpp-msvc/
├── forgeyard-cpp-mingw/
├── forgeyard-cpp-cmake/
├── forgeyard-cpp-meson/
├── forgeyard-cpp-ninja/
├── forgeyard-cpp-make/
├── forgeyard-cpp-conan/
├── forgeyard-cpp-vcpkg/
├── forgeyard-cpp-pkgconfig/
├── forgeyard-cpp-deps/
├── forgeyard-cpp-linkage/
├── forgeyard-cpp-analysis/
├── forgeyard-cpp-test/
├── forgeyard-cpp-sanitizer/
├── forgeyard-cpp-fuzz/
├── forgeyard-cpp-coverage/
├── forgeyard-cpp-lto/
├── forgeyard-cpp-pgo/
└── forgeyard-cpp-package/
```

---

# 28. Go Ecosystem

```text
ecosystems/go/
├── forgeyard-go/
├── forgeyard-go-model/
├── forgeyard-go-detect/
├── forgeyard-go-toolchain/
├── forgeyard-go-mod/
├── forgeyard-go-work/
├── forgeyard-go-resolve/
├── forgeyard-go-vendor/
├── forgeyard-go-cgo/
├── forgeyard-go-analysis/
├── forgeyard-go-test/
├── forgeyard-go-fuzz/
├── forgeyard-go-coverage/
├── forgeyard-go-bench/
├── forgeyard-go-cross/
└── forgeyard-go-package/
```

---

# 29. JavaScript / TypeScript Ecosystem

```text
ecosystems/javascript-typescript/
├── forgeyard-js/
├── forgeyard-js-model/
├── forgeyard-js-detect/
├── forgeyard-js-runtime/
├── forgeyard-js-node/
├── forgeyard-js-bun/
├── forgeyard-js-npm/
├── forgeyard-js-pnpm/
├── forgeyard-js-yarn/
├── forgeyard-js-lock/
├── forgeyard-js-workspace/
├── forgeyard-js-typescript/
├── forgeyard-js-bundler/
├── forgeyard-js-vite/
├── forgeyard-js-rollup/
├── forgeyard-js-esbuild/
├── forgeyard-js-webpack/
├── forgeyard-js-swc/
├── forgeyard-js-babel/
├── forgeyard-js-test/
├── forgeyard-js-browser-test/
├── forgeyard-js-analysis/
├── forgeyard-js-native-addon/
├── forgeyard-js-web/
└── forgeyard-js-package/
```

---

# 30. Python Ecosystem

```text
ecosystems/python/
├── forgeyard-python/
├── forgeyard-python-model/
├── forgeyard-python-detect/
├── forgeyard-python-interpreter/
├── forgeyard-python-pyproject/
├── forgeyard-python-lock/
├── forgeyard-python-uv/
├── forgeyard-python-pip/
├── forgeyard-python-poetry/
├── forgeyard-python-pep517/
├── forgeyard-python-build-backend/
├── forgeyard-python-wheel/
├── forgeyard-python-sdist/
├── forgeyard-python-venv/
├── forgeyard-python-native/
├── forgeyard-python-pyo3/
├── forgeyard-python-cython/
├── forgeyard-python-analysis/
├── forgeyard-python-test/
├── forgeyard-python-coverage/
└── forgeyard-python-package/
```

---

# 31. Java / Kotlin / JVM Ecosystem

```text
ecosystems/jvm/
├── forgeyard-jvm/
├── forgeyard-jvm-model/
├── forgeyard-jvm-detect/
├── forgeyard-jvm-jdk/
├── forgeyard-jvm-java/
├── forgeyard-jvm-kotlin/
├── forgeyard-jvm-gradle/
├── forgeyard-jvm-maven/
├── forgeyard-jvm-lock/
├── forgeyard-jvm-deps/
├── forgeyard-jvm-plugins/
├── forgeyard-jvm-annotation-processing/
├── forgeyard-jvm-kapt/
├── forgeyard-jvm-ksp/
├── forgeyard-jvm-jpms/
├── forgeyard-jvm-test/
├── forgeyard-jvm-analysis/
├── forgeyard-jvm-coverage/
├── forgeyard-jvm-jni/
├── forgeyard-jvm-package/
└── forgeyard-jvm-publish/
```

---

# 32. Dart / Flutter Ecosystem

```text
ecosystems/dart-flutter/
├── forgeyard-dart/
├── forgeyard-dart-model/
├── forgeyard-dart-detect/
├── forgeyard-dart-sdk/
├── forgeyard-dart-pub/
├── forgeyard-dart-lock/
├── forgeyard-dart-workspace/
├── forgeyard-dart-build/
├── forgeyard-dart-codegen/
├── forgeyard-dart-analysis/
├── forgeyard-dart-test/
├── forgeyard-dart-package/
├── forgeyard-flutter/
├── forgeyard-flutter-sdk/
├── forgeyard-flutter-build/
├── forgeyard-flutter-assets/
├── forgeyard-flutter-plugins/
├── forgeyard-flutter-android/
├── forgeyard-flutter-ios/
├── forgeyard-flutter-macos/
├── forgeyard-flutter-windows/
├── forgeyard-flutter-linux/
├── forgeyard-flutter-web/
├── forgeyard-flutter-test/
├── forgeyard-flutter-device-test/
├── forgeyard-flutter-package/
└── forgeyard-flutter-provenance/
```

---

# 33. Swift Ecosystem

```text
ecosystems/swift/
├── forgeyard-swift/
├── forgeyard-swift-model/
├── forgeyard-swift-detect/
├── forgeyard-swift-toolchain/
├── forgeyard-swift-swiftpm/
├── forgeyard-swift-lock/
├── forgeyard-swift-deps/
├── forgeyard-swift-macros/
├── forgeyard-swift-plugins/
├── forgeyard-swift-native/
├── forgeyard-swift-clang/
├── forgeyard-swift-objc/
├── forgeyard-swift-cxx/
├── forgeyard-swift-test/
├── forgeyard-swift-analysis/
├── forgeyard-swift-coverage/
├── forgeyard-swift-doc/
├── forgeyard-swift-linux/
├── forgeyard-swift-apple/
├── forgeyard-swift-xcode/
├── forgeyard-swift-xcframework/
├── forgeyard-swift-signing/
├── forgeyard-swift-package/
├── forgeyard-swift-publish/
└── forgeyard-swift-provenance/
```

---

# 34. Web Layer

```text
ecosystems/web/
├── forgeyard-web/
├── forgeyard-web-html/
├── forgeyard-web-css/
├── forgeyard-web-postcss/
├── forgeyard-web-tailwind/
├── forgeyard-web-sass/
├── forgeyard-web-assets/
├── forgeyard-web-browser/
└── forgeyard-web-static/
```

---

# 35. Native Toolchain Root

```text
native/
├── api/
├── assembly/
├── linker/
├── abi/
├── object/
├── sysroot/
├── libc/
├── runtime/
├── pkgconfig/
└── binary/
```

## Native API

```text
native/api/
├── forgeyard-native-api/
├── forgeyard-native-model/
├── forgeyard-native-build-request/
└── forgeyard-native-testkit/
```

High-level ecosystems consume this instead of reaching into C++/Assembly internals.

---

# 36. Assembly Subsystem

```text
native/assembly/
├── forgeyard-asm/
├── forgeyard-asm-model/
├── forgeyard-asm-detect/
├── forgeyard-asm-toolchain/
├── forgeyard-asm-preprocess/
├── forgeyard-asm-abi/
├── forgeyard-asm-object/
├── forgeyard-asm-link/
├── forgeyard-asm-layout/
├── forgeyard-asm-verify/
├── forgeyard-asm-disasm/
├── forgeyard-asm-cross/
└── forgeyard-asm-embedded/
```

`forgeyard-asm-model/src/`:

```text
lib.rs
assembler.rs
syntax.rs
architecture.rs
cpu_feature.rs
abi.rs
object_format.rs
unit.rs
action.rs
output.rs
error.rs
```

---

# 37. Linker / ABI / Object

```text
native/linker/
├── forgeyard-linker/
├── forgeyard-linker-model/
├── forgeyard-linker-bfd/
├── forgeyard-linker-lld/
├── forgeyard-linker-msvc/
├── forgeyard-linker-apple/
├── forgeyard-linker-script/
└── forgeyard-linker-verify/
```

```text
native/abi/
├── forgeyard-abi/
├── forgeyard-abi-sysv/
├── forgeyard-abi-windows/
├── forgeyard-abi-aapcs/
├── forgeyard-abi-riscv/
└── forgeyard-abi-verify/
```

```text
native/object/
├── forgeyard-object/
├── forgeyard-object-elf/
├── forgeyard-object-coff/
├── forgeyard-object-macho/
├── forgeyard-object-wasm/
├── forgeyard-object-symbol/
├── forgeyard-object-relocation/
└── forgeyard-object-disasm/
```

---

# 38. Sysroots / Runtime

```text
native/sysroot/
├── forgeyard-sysroot/
├── forgeyard-sysroot-model/
├── forgeyard-sysroot-linux/
├── forgeyard-sysroot-windows/
├── forgeyard-sysroot-android/
├── forgeyard-sysroot-apple/
└── forgeyard-sysroot-embedded/
```

```text
native/libc/
├── forgeyard-libc/
├── forgeyard-libc-glibc/
├── forgeyard-libc-musl/
├── forgeyard-libc-bionic/
└── forgeyard-libc-msvcrt/
```

```text
native/runtime/
├── forgeyard-native-runtime/
├── forgeyard-runtime-closure/
└── forgeyard-runtime-verify/
```

```text
native/binary/
├── forgeyard-binary/
├── forgeyard-binary-elf/
├── forgeyard-binary-pe/
├── forgeyard-binary-macho/
├── forgeyard-binary-wasm/
├── forgeyard-binary-dependency/
└── forgeyard-binary-symbol/
```

---

# 39. Platform Root

```text
platforms/
├── api/
├── linux/
├── windows/
├── apple/
├── android/
├── wasm/
├── embedded/
└── browser/
```

## Platform API

```text
platforms/api/
├── forgeyard-platform-api/
├── forgeyard-platform-model/
├── forgeyard-platform-sdk/
└── forgeyard-platform-testkit/
```

---

# 40. Linux Platform

```text
platforms/linux/
├── forgeyard-linux/
├── forgeyard-linux-detect/
├── forgeyard-linux-cgroup/
├── forgeyard-linux-seccomp/
├── forgeyard-linux-namespace/
├── forgeyard-linux-bwrap/
├── forgeyard-linux-ebpf/
├── forgeyard-linux-io-uring/
├── forgeyard-linux-package/
└── forgeyard-linux-runner/
```

---

# 41. Windows Platform

```text
platforms/windows/
├── forgeyard-windows/
├── forgeyard-windows-sdk/
├── forgeyard-windows-msvc/
├── forgeyard-windows-runner/
├── forgeyard-windows-sandbox/
├── forgeyard-windows-signing/
└── forgeyard-windows-package/
```

---

# 42. Apple Platform

```text
platforms/apple/
├── forgeyard-apple/
├── forgeyard-apple-xcode/
├── forgeyard-apple-sdk/
├── forgeyard-apple-simulator/
├── forgeyard-apple-device/
├── forgeyard-apple-signing/
├── forgeyard-apple-provisioning/
├── forgeyard-apple-notarization/
└── forgeyard-apple-package/
```

---

# 43. Android Platform

```text
platforms/android/
├── forgeyard-android/
├── forgeyard-android-sdk/
├── forgeyard-android-ndk/
├── forgeyard-android-emulator/
├── forgeyard-android-device/
├── forgeyard-android-signing/
└── forgeyard-android-package/
```

---

# 44. WASM / Embedded / Browser

```text
platforms/wasm/
├── forgeyard-wasm/
├── forgeyard-wasm-target/
├── forgeyard-wasm-runtime/
├── forgeyard-wasm-component/
└── forgeyard-wasm-package/
```

```text
platforms/embedded/
├── forgeyard-embedded/
├── forgeyard-embedded-target/
├── forgeyard-embedded-flash/
├── forgeyard-embedded-qemu/
├── forgeyard-embedded-probe/
├── forgeyard-embedded-serial/
└── forgeyard-embedded-device/
```

```text
platforms/browser/
├── forgeyard-browser/
├── forgeyard-browser-chromium/
├── forgeyard-browser-firefox/
├── forgeyard-browser-webkit/
└── forgeyard-browser-test/
```

---

# 45. API Layer

```text
crates/api/
├── forgeyard-api/
├── forgeyard-api-rest/
├── forgeyard-api-websocket/
├── forgeyard-api-events/
├── forgeyard-api-auth/
└── forgeyard-api-pagination/
```

`forgeyard-api-rest/src/`:

```text
lib.rs
router.rs
error.rs
auth.rs
routes/
├── projects.rs
├── pipelines.rs
├── runs.rs
├── jobs.rs
├── runners.rs
├── artifacts.rs
├── releases.rs
├── deployments.rs
├── repositories.rs
├── sources.rs
├── changes.rs
├── policies.rs
├── audit.rs
└── health.rs
```

---

# 46. Transport / Coordination / RBE

```text
crates/transport/
├── forgeyard-transport/
├── forgeyard-transport-quic/
├── forgeyard-transport-http/
├── forgeyard-transport-websocket/
└── forgeyard-transport-auth/
```

```text
crates/coordination/
├── forgeyard-coordination/
├── forgeyard-coordination-model/
├── forgeyard-coordination-local/
├── forgeyard-coordination-raft/
└── forgeyard-coordination-lock/
```

Raft only for narrow coordination state.

```text
crates/rbe/
├── forgeyard-rbe/
├── forgeyard-rbe-proto/
├── forgeyard-rbe-cas/
├── forgeyard-rbe-action/
└── forgeyard-rbe-execution/
```

RBE is interoperability, not Forgeyard's native internal protocol.

---

# 47. Supply Chain / Security

```text
crates/supply-chain/
├── forgeyard-sbom/
├── forgeyard-vex/
├── forgeyard-provenance/
├── forgeyard-attestation/
├── forgeyard-signature/
├── forgeyard-slsa/
└── forgeyard-in-toto/
```

```text
crates/security/
├── forgeyard-security/
├── forgeyard-secret-scan/
├── forgeyard-dependency-scan/
├── forgeyard-license-scan/
├── forgeyard-policy-scan/
└── forgeyard-threat-evidence/
```

---

# 48. Plugin / Extension SDK

```text
crates/plugin/
├── forgeyard-plugin-api/
├── forgeyard-plugin-host/
├── forgeyard-plugin-manifest/
├── forgeyard-plugin-permission/
└── forgeyard-plugin-sdk/
```

Do not freeze an external plugin ABI too early. First mature in-tree adapter APIs.

---

# 49. UI Detailed Tree

```text
apps/forgeyard-ui/src/
├── main.rs
├── app.rs
├── router.rs
├── state.rs
├── api.rs
├── session.rs
├── platform.rs
├── pages/
│   ├── dashboard.rs
│   ├── projects.rs
│   ├── project.rs
│   ├── pipelines.rs
│   ├── pipeline.rs
│   ├── runs.rs
│   ├── run.rs
│   ├── jobs.rs
│   ├── job.rs
│   ├── runners.rs
│   ├── runner.rs
│   ├── artifacts.rs
│   ├── releases.rs
│   ├── deployments.rs
│   ├── vcs.rs
│   ├── repository.rs
│   ├── changes.rs
│   ├── change.rs
│   ├── policies.rs
│   ├── secrets.rs
│   ├── audit.rs
│   ├── settings.rs
│   ├── admin.rs
│   └── ecosystems/
│       ├── rust.rs
│       ├── cpp.rs
│       ├── go.rs
│       ├── javascript_typescript.rs
│       ├── python.rs
│       ├── jvm.rs
│       ├── dart_flutter.rs
│       ├── swift.rs
│       └── native.rs
├── views/
│   ├── pipeline_graph.rs
│   ├── log_stream.rs
│   ├── artifact_browser.rs
│   ├── diff.rs
│   ├── review.rs
│   ├── integration_queue.rs
│   ├── dependency_graph.rs
│   ├── reproducibility.rs
│   ├── toolchain.rs
│   └── health.rs
├── components/
│   ├── button.rs
│   ├── table.rs
│   ├── tree.rs
│   ├── graph.rs
│   ├── badge.rs
│   ├── modal.rs
│   ├── drawer.rs
│   ├── code.rs
│   ├── diff_line.rs
│   ├── log_line.rs
│   ├── status.rs
│   └── empty_state.rs
├── hooks/
│   ├── use_api.rs
│   ├── use_events.rs
│   ├── use_pagination.rs
│   └── use_platform.rs
└── theme/
    ├── mod.rs
    ├── tokens.rs
    ├── typography.rs
    ├── spacing.rs
    └── responsive.rs
```

---

# 50. Protocols / Schemas / Config / Policies

```text
protocols/
├── README.md
├── internal/
│   ├── agent-daemon.md
│   ├── log-stream.md
│   ├── artifact-transfer.md
│   ├── runner-registration.md
│   └── source-resolution.md
├── public/
│   ├── rest.md
│   ├── websocket.md
│   └── events.md
└── compatibility/
    ├── versioning.md
    └── rolling-upgrades.md
```

```text
schemas/
├── config/
├── protocol/
├── policy/
├── event/
├── api/
└── migration/
```

```text
config/
├── default/
│   ├── forgeyard.ron
│   ├── daemon.ron
│   ├── agent.ron
│   ├── ui.ron
│   └── policy.ron
├── development/
├── production/
├── examples/
└── schema/
```

```text
policies/
├── default/
│   ├── build.ron
│   ├── source.ron
│   ├── change.ron
│   ├── release.ron
│   └── security.ron
├── enterprise/
└── examples/
```

---

# 51. Migrations

```text
migrations/
├── postgres/
│   ├── 0001_initial.sql
│   ├── 0002_change_proposals.sql
│   └── ...
├── stoolap/
│   ├── 0001_initial.ron
│   └── ...
└── README.md
```

Use expand-contract for distributed upgrades.

---

# 52. Packaging / Deployment / Infra

```text
packaging/
├── linux/
│   ├── deb/
│   └── rpm/
├── windows/
│   ├── msi/
│   └── msix/
├── macos/
│   ├── dmg/
│   └── pkg/
├── android/
└── oci/
```

```text
deploy/
├── standalone/
├── systemd/
├── kubernetes/
├── docker-compose/
└── examples/
```

```text
infra/
├── terraform/
├── ansible/
├── nix/
├── devcontainer/
├── observability/
└── certificates/
```

These are deployment helpers, not core runtime dependencies.

---

# 53. Fixtures

```text
fixtures/
├── source/
├── vcs/
│   ├── git/
│   ├── mercurial/
│   ├── fossil/
│   ├── breezy/
│   ├── jujutsu/
│   ├── darcs/
│   └── pijul/
├── ecosystems/
│   ├── rust/
│   ├── cpp/
│   ├── go/
│   ├── javascript-typescript/
│   ├── python/
│   ├── jvm/
│   ├── dart-flutter/
│   └── swift/
├── binaries/
├── archives/
├── policies/
├── protocols/
└── certificates/
```

Each ecosystem should have:
- minimal;
- workspace/monorepo;
- native dependency;
- failure;
- reproducibility fixture.

---

# 54. Root Tests

```text
tests/
├── integration/
├── e2e/
├── distributed/
├── failure/
├── security/
├── reproducibility/
├── upgrade/
├── migration/
├── compatibility/
├── vcs/
├── change/
├── ecosystems/
├── platforms/
└── selfhost/
```

Examples:

```text
tests/change/
├── proposal_revision.rs
├── approval_invalidation.rs
├── stale_check.rs
├── ownership.rs
├── mergeability.rs
├── candidate.rs
├── serial_queue.rs
├── speculative_queue.rs
├── batch_queue.rs
├── provider_sync.rs
└── submit_verification.rs
```

```text
tests/security/
├── sandbox_escape.rs
├── secret_leak.rs
├── webhook_spoof.rs
├── ssrf.rs
├── path_traversal.rs
├── archive_escape.rs
├── cross_tenant.rs
└── signing_boundary.rs
```

```text
tests/reproducibility/
├── rust.rs
├── cpp.rs
├── go.rs
├── python.rs
├── jvm.rs
├── flutter.rs
├── swift.rs
├── assembly.rs
└── archive.rs
```

---

# 55. Benches / Fuzz

```text
benches/
├── scheduler.rs
├── cas.rs
├── digest.rs
├── snapshot.rs
├── pipeline_parse.rs
├── dependency_graph.rs
└── diff.rs
```

```text
fuzz/
├── Cargo.toml
├── fuzz_targets/
│   ├── ron_config.rs
│   ├── postcard_envelope.rs
│   ├── canonical_path.rs
│   ├── archive_extract.rs
│   ├── vcs_event.rs
│   ├── diff_anchor.rs
│   ├── object_parser.rs
│   └── pipeline_parser.rs
└── corpus/
```

---

# 56. `xtask/`

```text
xtask/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── fmt.rs
    ├── lint.rs
    ├── test.rs
    ├── docs.rs
    ├── generate.rs
    ├── package.rs
    ├── release.rs
    ├── license.rs
    ├── fixtures.rs
    └── selfhost.rs
```

Prefer Rust `xtask` to large shell scripts.

---

# 57. Tools

```text
tools/
├── forgeyard-architecture-check/
├── forgeyard-workspace-index/
├── forgeyard-schema-gen/
├── forgeyard-fixture-gen/
├── forgeyard-protocol-inspect/
├── forgeyard-cas-inspect/
├── forgeyard-object-inspect/
├── forgeyard-snapshot-inspect/
└── forgeyard-release-verify/
```

### `forgeyard-architecture-check`
Reads Cargo metadata + `architecture.ron` and rejects forbidden dependency edges.

---

# 58. Docs

```text
docs/
├── README.md
├── getting-started/
├── architecture/
├── concepts/
├── administration/
├── security/
├── operations/
├── development/
├── ecosystems/
├── vcs/
├── change-proposals/
├── runners/
├── packaging/
├── deployment/
├── api/
├── troubleshooting/
└── reference/
```

Architecture docs:

```text
docs/architecture/
├── complete-system.md
├── hermetic-functional-packaging.md
├── vcs-neutral.md
├── change-proposal.md
├── storage.md
├── cas.md
├── scheduler.md
├── runner.md
├── sandbox.md
├── protocol.md
├── security.md
├── observability.md
├── release.md
├── deployment.md
├── ui.md
└── workspace-structure.md
```

Ecosystem docs:

```text
docs/ecosystems/
├── rust.md
├── c-cpp.md
├── go.md
├── javascript-typescript.md
├── python.md
├── java-kotlin-jvm.md
├── dart-flutter.md
├── swift.md
├── web.md
└── assembly-native.md
```

VCS docs:

```text
docs/vcs/
├── model.md
├── snapshots.md
├── provenance.md
├── git.md
├── mercurial.md
├── fossil.md
├── breezy.md
├── jujutsu.md
├── darcs.md
├── pijul.md
└── migration.md
```

Change Proposal docs:

```text
docs/change-proposals/
├── overview.md
├── review.md
├── approvals.md
├── ownership.md
├── checks.md
├── policy.md
├── integration-candidates.md
├── integration-queue.md
├── provider-sync.md
└── security.md
```

---

# 59. ADRs

```text
adr/
├── 0001-stoolap-standalone.md
├── 0002-postgres-distributed.md
├── 0003-quic-postcard-native.md
├── 0004-grpc-interop-only.md
├── 0005-cas-separate-from-metadata.md
├── 0006-iroh-not-authoritative.md
├── 0007-raft-control-state-only.md
├── 0008-kubernetes-optional.md
├── 0009-vcs-neutral-source-snapshot.md
├── 0010-change-proposal-not-pr.md
├── 0011-monorepo-modular-monolith.md
└── ...
```

Supersede ADRs; do not rewrite historical decisions.

---

# 60. RFCs

```text
rfcs/
├── README.md
├── accepted/
├── proposed/
├── rejected/
└── superseded/
```

Use for major architectural evolution.

---

# 61. Scripts / Examples / Contrib / Vendor

```text
scripts/
├── bootstrap.sh
├── bootstrap.ps1
├── install-dev-tools.sh
└── README.md
```

```text
examples/
├── standalone/
├── distributed/
├── rust/
├── cpp/
├── go/
├── python/
├── jvm/
├── flutter/
├── swift/
├── vcs/
├── change-proposal/
└── policies/
```

```text
contrib/
├── shell-completion/
├── editors/
├── systemd/
├── packaging/
└── migration/
```

```text
vendor/
├── README.md
└── ...
```

Vendored material requires origin/version/license/reason/update docs.

---

# 62. Crate Internal Pattern

A normal crate:

```text
crate/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── model/
│   ├── service/
│   ├── error.rs
│   └── ...
├── tests/
└── benches/
```

Rules:
- `lib.rs` should be small.
- avoid giant `models.rs`;
- avoid giant `service.rs`;
- prefer semantic modules;
- use `pub(crate)` by default;
- public API is intentional.

---

# 63. Toolchain Adapter Pattern

```text
src/
├── lib.rs
├── detect.rs
├── identity.rs
├── resolve.rs
├── install.rs
├── capability.rs
├── verify.rs
└── error.rs
```

# 64. Ecosystem Resolver Pattern

```text
src/
├── manifest.rs
├── lock.rs
├── resolve.rs
├── fetch.rs
├── offline.rs
├── graph.rs
└── policy.rs
```

# 65. Ecosystem Build Pattern

```text
src/
├── derivation.rs
├── build_plan.rs
├── environment.rs
├── command.rs
├── output.rs
└── diagnostics.rs
```

# 66. Ecosystem Test Pattern

```text
src/
├── plan.rs
├── discover.rs
├── execute.rs
├── parse.rs
├── report.rs
└── shard.rs
```

# 67. Ecosystem Package Pattern

```text
src/
├── package.rs
├── validate.rs
├── metadata.rs
├── publish.rs
└── provenance.rs
```

---

# 68. Architecture Layers

```text
L0 primitives
L1 domain models
L2 capability APIs
L3 generic services
L4 adapters
L5 application composition
```

### L0
`ids`, `digest`, `time`, `error`, `envelope`

### L1
VCS model, Change Proposal model, pipeline model, runner model, release model

### L2
`ForgeyardStore`, `CasBackend`, `Executor`, `SecretProvider`, `VcsBackend`, `VcsIntegrationBackend`, `EcosystemAdapter`, `ScmProvider`

### L3
scheduler, change service, pipeline planner, policy, release orchestration

### L4
Postgres/Stoolap, Git/Mercurial, S3/Iroh, ecosystems, platform adapters, GitHub/GitLab

### L5
daemon, agent, CLI, UI, standalone binary

---

# 69. Forbidden Dependency Examples

```text
forgeyard-core -> forgeyard-store-postgres        forbidden
forgeyard-vcs-model -> forgeyard-vcs-git          forbidden
forgeyard-change-model -> forgeyard-scm-github    forbidden
forgeyard-ecosystem-api -> forgeyard-rust         forbidden
forgeyard-pipeline-ir -> forgeyard-ui             forbidden
forgeyard-cas -> forgeyard-cas-s3                 forbidden
```

Allowed:

```text
forgeyard-vcs-git -> forgeyard-vcs
forgeyard-store-postgres -> forgeyard-store
forgeyard-rust -> forgeyard-ecosystem-api
forgeyard-daemon -> adapter crates
```

---

# 70. Architecture Enforcement

`architecture.ron` example:

```ron
(
    layers: [
        ("primitives", ["forgeyard-ids", "forgeyard-time"]),
        ("domain", ["forgeyard-vcs-model", "forgeyard-change-model"]),
        ("service", ["forgeyard-change-service"]),
        ("adapter", ["forgeyard-vcs-git"]),
        ("app", ["forgeyard-daemon"]),
    ],

    forbidden_edges: [
        ("domain", "adapter"),
    ],
)
```

`tools/forgeyard-architecture-check/` validates Cargo metadata against these rules.

---

# 71. Feature Strategy

Adapters are optional at composition level.

Potential features:

```text
postgres
oidc
saml
scim
rbe
kubernetes
iroh-cas
ebpf
io-uring
enclave
vcs-git
vcs-mercurial
vcs-fossil
vcs-breezy
vcs-jujutsu
vcs-darcs
vcs-pijul
ecosystem-rust
ecosystem-cpp
ecosystem-go
ecosystem-js
ecosystem-python
ecosystem-jvm
ecosystem-flutter
ecosystem-swift
```

Do not put every flag into one mega-crate. Feature-gate adapter/composition crates.

---

# 72. Platform `cfg` Rule

Platform-specific code belongs in platform crates.

Avoid `#[cfg(...)]` forests inside domain crates.

---

# 73. Build Script Rule

Avoid Forgeyard's own `build.rs` where possible.

If needed, it must be hermetic and documented.

---

# 74. Serialization Rule

- RON: human config/policy/fixtures.
- Postcard: internal binary protocol.
- JSON: public/provider interop only.
- Domain types should not accidentally become public DTOs.

---

# 75. State Modeling Rule

Use explicit enums and state transition modules.

For distributed persisted states:

```text
state.rs
transition.rs
invariant.rs
```

Use typestate for local builder/resource workflows, not as a substitute for distributed persisted-state modeling.

---

# 76. No Dumping-Ground Directories

Avoid:

```text
utils/
common/
misc/
helpers/
shared/
```

Every reusable capability needs clear semantic ownership.

---

# 77. Workspace Growth Rule

Create a new crate when there is a real:
- dependency boundary;
- runtime/platform boundary;
- security boundary;
- ownership boundary;
- test surface;
- optional heavy dependency.

Otherwise start with a module.

---

# 78. Runner Distribution Principle

Runner binaries should include only:

```text
agent protocol
CAS client
runner/sandbox/executor
required ecosystem adapters
required native/platform adapters
```

Not UI, provider APIs, or control-plane business logic.

---

# 79. Daemon Principle

Daemon includes:

```text
domain services
metadata store
scheduler
VCS/SCM
Change Proposal
policy
identity
API
coordination
```

No arbitrary user build toolchains.

---

# 80. Signing Worker Principle

Signing worker includes:

```text
artifact fetch
policy verification
restricted signer
attestation/audit
```

No compilation.

---

# 81. Device Agent Principle

Device agent includes:

```text
discovery
leases
install/flash
run
logs
cleanup
```

No policy authority.

---

# 82. Composition Bootstrap

Example daemon bootstrap:

```text
apps/forgeyard-daemon/src/bootstrap/
├── mod.rs
├── store.rs
├── cas.rs
├── vcs.rs
├── scm.rs
├── ecosystems.rs
├── policy.rs
├── identity.rs
├── secrets.rs
├── telemetry.rs
└── coordination.rs
```

Concrete adapter registries are created here.

Do not use global mutable registries.

---

# 83. Standalone Composition

```text
forgeyard
├── Stoolap
├── local CAS
├── VCS adapters
├── local scheduler
├── local runner
├── Change Proposal
├── policy
└── Dioxus UI
```

No cloud/Postgres required.

---

# 84. Distributed Composition

```text
forgeyard-daemon
├── Postgres/Neon
├── shared CAS
├── VCS/SCM
├── Change Proposal
├── scheduler
├── policy
├── identity
├── API
└── QUIC

forgeyard-agent
├── runner
├── sandbox
├── executor
├── ecosystems
├── native
└── platform toolchains
```

---

# 85. Enterprise Composition

Adds:

```text
HA coordination
OIDC/SAML/SCIM
mTLS
external secret providers
multi-region CAS
signing workers
device pools
Kubernetes operator
RBE adapter
SIEM/audit export
```

---

# 86. Self-Hosting

Eventually:

```text
trusted Forgeyard bootstrap
  ↓
Forgeyard builds Forgeyard
  ↓
new Forgeyard rebuilds Forgeyard
  ↓
independent reproducer
  ↓
release
```

Root test:

```text
tests/selfhost/forgeyard_builds_forgeyard.rs
```

Forgeyard's own Git commit resolves through the same VCS-neutral layer:

```text
Git revision
  ↓
SourceSnapshotId
  ↓
Rust derivation
```

---

# 87. Change Proposal Self-Hosting

Forgeyard's own changes should eventually use:

```text
Forgeyard ChangeProposal
  ↓
ProposalRevision
  ↓
review/check/policy
  ↓
IntegrationCandidate
  ↓
candidate CI
  ↓
IntegrationQueue
  ↓
exact tested submit
```

---

# 88. CI for the Workspace

```text
format
architecture-check
cargo check
Clippy
unit tests
integration tests
VCS conformance
Change Proposal tests
ecosystem fixture tests
platform tests
security tests
distributed failure tests
reproducibility
self-host
package
release
```

Changed-crate optimization may be used, but uncertainty must fall back to broader/full testing.

---

# 89. Final Canonical Physical Structure

```text
forgeyard/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── architecture.ron
├── README.md
├── LICENSE
├── SECURITY.md
├── CONTRIBUTING.md
│
├── .forgeyard/
├── apps/
├── crates/
│   ├── core/
│   ├── ids/
│   ├── digest/
│   ├── time/
│   ├── error/
│   ├── config/
│   ├── protocol/
│   ├── project/
│   ├── source/
│   ├── run/
│   ├── lease/
│   ├── store/
│   ├── cas/
│   ├── vcs/
│   ├── scm/
│   ├── change/
│   ├── pipeline/
│   ├── scheduler/
│   ├── runner/
│   ├── sandbox/
│   ├── executor/
│   ├── toolchain/
│   ├── hermetic/
│   ├── reproducibility/
│   ├── cache/
│   ├── artifact/
│   ├── package/
│   ├── release/
│   ├── deploy/
│   ├── policy/
│   ├── identity/
│   ├── authz/
│   ├── secrets/
│   ├── audit/
│   ├── events/
│   ├── logs/
│   ├── telemetry/
│   ├── health/
│   ├── reconciliation/
│   ├── analysis/
│   ├── device/
│   ├── notification/
│   ├── supply-chain/
│   ├── security/
│   ├── api/
│   ├── transport/
│   ├── coordination/
│   ├── rbe/
│   └── plugin/
│
├── ecosystems/
│   ├── api/
│   ├── rust/
│   ├── cpp/
│   ├── go/
│   ├── javascript-typescript/
│   ├── python/
│   ├── jvm/
│   ├── dart-flutter/
│   ├── swift/
│   └── web/
│
├── native/
│   ├── api/
│   ├── assembly/
│   ├── linker/
│   ├── abi/
│   ├── object/
│   ├── sysroot/
│   ├── libc/
│   ├── runtime/
│   ├── pkgconfig/
│   └── binary/
│
├── platforms/
│   ├── api/
│   ├── linux/
│   ├── windows/
│   ├── apple/
│   ├── android/
│   ├── wasm/
│   ├── embedded/
│   └── browser/
│
├── protocols/
├── schemas/
├── config/
├── policies/
├── migrations/
├── packaging/
├── deploy/
├── infra/
├── assets/
├── web/
├── fixtures/
├── examples/
├── tests/
├── benches/
├── fuzz/
├── tools/
├── xtask/
├── scripts/
├── docs/
├── adr/
├── rfcs/
├── contrib/
└── vendor/
```

---

# 90. Final Architecture Rule

The entire repository should obey:

```text
Core defines truth.
Capability APIs define boundaries.
Services orchestrate behavior.
Adapters implement external/platform/ecosystem behavior.
Applications compose everything.
```

And the core dependency flow is:

```text
apps
 ↓
services/orchestration
 ↓
domain + capability APIs
 ↓
adapters
 ↓
external systems/toolchains/platforms
```

Never reverse that relationship.

This structure lets Forgeyard remain one coherent Rust codebase while scaling from an offline single binary to a distributed enterprise CI/CD system with VCS-neutral source control, Change Proposals, many language ecosystems, hermetic builds, native/assembly tooling, cross-platform runners, reproducibility, supply-chain security, and self-hosting.
