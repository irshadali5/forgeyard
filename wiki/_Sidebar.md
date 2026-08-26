### [Forgeyard Wiki](Home)

* [Wiki Home](Home)

---

<details open>
<summary><b>Core Foundations & Source Domain</b></summary>

- [Core Domain & Foundation](01-forgeyard-core-domain-foundation)
- [Change Proposal, Review & Integration](forgeyard-change-proposal-system-architecture)
- [VCS-Neutral Source Control System &](forgeyard-vcs-neutral-system-architecture)

</details>

<details open>
<summary><b>Storage, CAS & Data Plane</b></summary>

- [Storage & Metadata](02-forgeyard-storage-metadata)
- [CAS & Artifact Data Plane](03-forgeyard-cas-artifact-data-plane)
- [Cache Architecture, Build Acceleration, Remote Cache & Cache Correctness](38-forgeyard-cache-build-acceleration-remote-cache-correctness-system-architecture)
- [Data Lifecycle, Retention, Archival, Deletion, Legal Hold & Privacy Governance](46-forgeyard-data-lifecycle-retention-archival-deletion-privacy-governance-system-architecture)
- [Artifact Registry, Package Repository, OCI Distribution & Internal Software Distribution](52-forgeyard-artifact-registry-package-repository-oci-internal-distribution-system-architecture)
- [Database Schema Migration, Online Backfill, Data Transformation & Zero-Downtime Change Orchestration](63-forgeyard-database-schema-migration-online-backfill-data-transformation-zero-downtime-system-architecture)

</details>

<details open>
<summary><b>Pipeline Engine, IR, State Machine & Scheduling</b></summary>

- [Pipeline IR, Parsing, Normalization & Planning](04-forgeyard-pipeline-ir-parsing-planning)
- [Run / Job State Machine](05-forgeyard-run-job-state-machine)
- [Scheduler](06-forgeyard-scheduler-system-architecture)
- [Events, Event Delivery & Reconciliation](10-forgeyard-events-reconciliation-system-architecture)
- [Pipeline Triggers, Schedules, Manual Dispatch & Event-Driven Execution](44-forgeyard-pipeline-triggers-schedules-manual-dispatch-event-driven-system-architecture)
- [Workflow Concurrency, Distributed Locks, Idempotency Keys, Reservations & Exclusive Resource Coordination](60-forgeyard-workflow-concurrency-distributed-locks-idempotency-reservations-exclusive-coordination-system-architecture)
- [Job Checkpointing, Suspend/Resume, Preemption, Graceful Cancellation & Spot/Interruptible Runner Recovery](69-forgeyard-job-checkpointing-suspend-resume-preemption-graceful-cancellation-interruptible-runner-recovery-system-architecture)

</details>

<details open>
<summary><b>Runners, Sandboxing, Transport & Fleet Orchestration</b></summary>

- [Runner / Agent](07-forgeyard-runner-agent-system-architecture)
- [Sandbox & Executor](08-forgeyard-sandbox-executor-system-architecture)
- [Transport, QUIC & Internal Protocol](09-forgeyard-transport-quic-internal-protocol)
- [Device Lab](20-forgeyard-device-lab-system-architecture)
- [Runner Fleet Autoscaling, Capacity Provisioning & Infrastructure Provider](43-forgeyard-runner-fleet-autoscaling-capacity-provisioning-infrastructure-system-architecture)
- [Runner Image Factory, Golden Image, Patch Management & Fleet Baseline Attestation](58-forgeyard-runner-image-factory-golden-image-patch-management-baseline-attestation-system-architecture)
- [Network Connectivity, Private Resource Access, Egress Control, Tunneling & Zero-Trust Service Connectivity](59-forgeyard-network-connectivity-private-resource-access-egress-tunneling-zero-trust-system-architecture)

</details>

<details open>
<summary><b>Security, Policy, Secrets & Supply Chain</b></summary>

- [Policy, Authorization & Identity](11-forgeyard-policy-authorization-identity-system-architecture)
- [Secrets, Trust & Credential Security](12-forgeyard-secrets-trust-credential-security-system-architecture)
- [Supply Chain, SBOM, Provenance & Signing](13-forgeyard-supply-chain-sbom-provenance-signing-system-architecture)
- [Multi-Tenancy, Quotas, Resource Governance & Fair-Use](27-forgeyard-multi-tenancy-quotas-resource-governance-system-architecture)
- [Audit, Compliance, Evidence Retention & Security Governance](28-forgeyard-audit-compliance-security-governance-system-architecture)
- [Security Architecture, Threat Model, Hardening & Incident Response](40-forgeyard-security-threat-model-hardening-incident-response-system-architecture)

</details>

<details open>
<summary><b>Packaging, Releases, Deployment & Delivery</b></summary>

- [Packaging](14-forgeyard-packaging-system-architecture)
- [Release](15-forgeyard-release-system-architecture)
- [Deployment](16-forgeyard-deployment-system-architecture)
- [Release Distribution, Update Delivery, Installer, Channel & Client Update](41-forgeyard-release-distribution-update-delivery-installer-channel-system-architecture)
- [Environment Promotion, Progressive Delivery, Feature Rollout, Canary Analysis & Automated Rollback Governance](62-forgeyard-environment-promotion-progressive-delivery-feature-rollout-canary-rollback-system-architecture)
- [Artifact Promotion Policy, Release Train, Environment Channel & Lifecycle Governance](67-forgeyard-artifact-promotion-policy-release-train-environment-channel-lifecycle-governance-system-architecture)
- [Hermetic Build, Functional Packaging & Reproducible Distribution](forgeyard-hermetic-functional-packaging-architecture)

</details>

<details open>
<summary><b>Observability, Diagnostics, Testing & Quality</b></summary>

- [Observability, Health & Doctor](17-forgeyard-observability-health-doctor-system-architecture)
- [Search, Indexing, Query & Operational Analytics](31-forgeyard-search-indexing-query-operational-analytics-system-architecture)
- [Test Results, Quality Gates, Coverage & Flaky-Test Intelligence](32-forgeyard-test-results-quality-gates-coverage-flaky-intelligence-system-architecture)
- [Benchmarking, Performance Regression, Load-Test & Capacity Intelligence](33-forgeyard-benchmark-performance-regression-load-capacity-system-architecture)
- [Static Analysis, Code Quality, Security Scanning & Findings Management](37-forgeyard-static-analysis-code-quality-security-findings-system-architecture)
- [Failure Diagnosis, Debugging, Reproduction, Bisect & Root-Cause Intelligence](48-forgeyard-failure-diagnosis-debugging-reproduction-bisect-root-cause-system-architecture)
- [Reliability Engineering, SLO, Error Budget, Availability & Resilience Governance](50-forgeyard-reliability-slo-error-budget-availability-resilience-governance-system-architecture)
- [Test Data, Fixtures, Ephemeral Databases, Service Virtualization & Integration-Test Environment](56-forgeyard-test-data-fixtures-ephemeral-databases-service-virtualization-system-architecture)
- [Incident Management, On-Call, Escalation, Response Coordination & Postmortem](61-forgeyard-incident-management-oncall-escalation-response-postmortem-system-architecture)

</details>

<details open>
<summary><b>API, UI, DevEx & Service Portal</b></summary>

- [API / Axum](18-forgeyard-api-axum-system-architecture)
- [Dioxus UI / GUI](19-forgeyard-dioxus-ui-gui-system-architecture)
- [Developer Experience, Local Dev Environment, CLI Workflows & Reproducible Workstation](35-forgeyard-developer-experience-local-dev-cli-reproducible-workstation-system-architecture)
- [Workflow Templates, Reusable Pipelines, Organization Standards & Golden Paths](42-forgeyard-workflow-templates-reusable-pipelines-golden-paths-system-architecture)
- [Service Catalog, Component Ownership, Environment Inventory & Developer Portal](49-forgeyard-service-catalog-component-ownership-environment-inventory-developer-portal-system-architecture)
- [API/ABI/Schema/Protocol Compatibility, Contract Evolution & Breaking-Change Governance](57-forgeyard-api-abi-schema-protocol-compatibility-contract-evolution-system-architecture)
- [Remote Development Environments, Cloud Workspaces, Codespaces-Style Sessions & Developer Workspace Orchestration](64-forgeyard-remote-development-environments-cloud-workspaces-developer-workspace-orchestration-system-architecture)

</details>

<details open>
<summary><b>Distributed Coordination, Federation & Operations</b></summary>

- [SCM Provider Integrations](21-forgeyard-scm-provider-integrations-system-architecture)
- [High Availability, Coordination & Raft](22-forgeyard-ha-coordination-raft-system-architecture)
- [Remote Build Execution (RBE) Interoperability](23-forgeyard-rbe-interop-system-architecture)
- [Plugin & Extension](24-forgeyard-plugin-extension-system-architecture)
- [Operations, Backup, Upgrade & Disaster Recovery](25-forgeyard-operations-backup-upgrade-dr-system-architecture)
- [Self-Hosting, Bootstrap & Release-of-Forgeyard](26-forgeyard-self-hosting-bootstrap-release-system-architecture)
- [Notifications, Alerting & Human Workflow](29-forgeyard-notifications-alerting-human-workflow-system-architecture)
- [Entitlements, Licensing, Subscription & Commercial Access-Control](30-forgeyard-entitlements-licensing-subscription-commercial-access-system-architecture)
- [Monorepo Intelligence, Dependency Graph, Affected-Change & Incremental Execution](34-forgeyard-monorepo-dependency-graph-affected-incremental-execution-system-architecture)
- [Dependency, Package Registry, Artifact Mirror & Software-Source Governance](36-forgeyard-dependency-package-registry-artifact-mirror-source-governance-system-architecture)
- [Configuration, Feature Flags, Runtime Settings & Dynamic Configuration Governance](39-forgeyard-configuration-feature-flags-runtime-settings-governance-system-architecture)
- [Cost Accounting, FinOps, Chargeback/Showback & Resource Economics](45-forgeyard-cost-accounting-finops-chargeback-showback-resource-economics-system-architecture)
- [CI/CD Migration, Import, Compatibility & Legacy-System Interoperability](47-forgeyard-cicd-migration-import-compatibility-legacy-interoperability-system-architecture)
- [Multi-Region Federation, Edge Sites, Disconnected Operation & Cross-Site Replication](51-forgeyard-multi-region-federation-edge-disconnected-cross-site-replication-system-architecture)
- [Infrastructure-as-Code, Environment Provisioning, Preview Environments & Drift Reconciliation](53-forgeyard-infrastructure-as-code-environment-provisioning-preview-drift-system-architecture)
- [Merge Queue, Speculative Integration, Batch Validation & Protected Target Submission](54-forgeyard-merge-queue-speculative-integration-batch-validation-protected-target-system-architecture)
- [AI-Assisted CI Optimization, Engineering Copilot & Autonomous Recommendation Governance](55-forgeyard-ai-assisted-ci-optimization-engineering-copilot-autonomous-recommendation-governance-system-architecture)
- [Build Graph Replay, Historical Reproducibility, Time-Travel CI & Evidence Reconstruction](65-forgeyard-build-graph-replay-historical-reproducibility-time-travel-ci-evidence-reconstruction-system-architecture)
- [Change Risk Assessment, Preflight Simulation, Policy Preview & What-If Analysis](66-forgeyard-change-risk-assessment-preflight-simulation-policy-preview-what-if-analysis-system-architecture)
- [Configuration Drift Detection, Desired-State Convergence, Runtime Reconciliation & Environment Consistency](68-forgeyard-configuration-drift-desired-state-convergence-runtime-reconciliation-environment-consistency-system-architecture)
- [Dependency Update Automation, Version Maintenance, Vulnerability Remediation & Upgrade Campaign](70-forgeyard-dependency-update-automation-version-maintenance-vulnerability-remediation-upgrade-campaign-system-architecture)

</details>

<details open>
<summary><b>Ecosystems & Language Toolchains</b></summary>

- [Assembly & Native Object Toolchain](forgeyard-assembly-native-architecture)
- [C/C++ CI/CD System &](forgeyard-c-cpp-system-architecture)
- [Dart + Flutter CI/CD System &](forgeyard-dart-flutter-system-architecture)
- [Go CI/CD System &](forgeyard-go-system-architecture)
- [Java + Kotlin JVM CI/CD System &](forgeyard-java-kotlin-jvm-system-architecture)
- [JavaScript / TypeScript CI/CD System &](forgeyard-javascript-typescript-system-architecture)
- [Python CI/CD System &](forgeyard-python-system-architecture)
- [Rust CI/CD System &](forgeyard-rust-system-architecture)
- [Swift CI/CD System &](forgeyard-swift-system-architecture)

</details>

---

* [GitHub Repository](https://github.com/irshadali5/forgeyard)
* [System Architecture Directory](https://github.com/irshadali5/forgeyard/tree/main/sys-arch)
