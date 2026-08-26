#!/usr/bin/env python3
"""
Forgeyard System Architecture Wiki Index Generator
Generates Home.md, _Sidebar.md, and _Footer.md for GitHub Wiki.
"""

import os
import glob
import re

def main():
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    sys_arch_dir = os.path.join(root_dir, "sys-arch")
    wiki_dir = os.path.join(root_dir, "wiki")
    os.makedirs(wiki_dir, exist_ok=True)

    file_info = {}
    for path in glob.glob(os.path.join(sys_arch_dir, "*.md")):
        fname = os.path.basename(path)
        with open(path, "r", encoding="utf-8") as f:
            content = f.read(5000)
        lines = content.splitlines()
        h1 = lines[0].strip().lstrip("#").strip() if lines else fname

        # Extract clean short title
        short_title = h1
        short_title = re.sub(r"^(\d+\s*[\u2014\u2013\-]\s*Forgeyard\s*|\d+\s*[\u2014\u2013\-]\s*|Forgeyard\s*)", "", short_title)
        short_title = re.sub(r"\s*System Architecture$", "", short_title)
        short_title = re.sub(r"\s*Architecture$", "", short_title)
        short_title = re.sub(r"\s*CI/CD System & Architecture$", "", short_title)
        short_title = short_title.strip()

        # Extract scope
        scope = ""
        for line in lines[1:25]:
            if "Scope:" in line or "**Scope:**" in line:
                scope = line.split("Scope:", 1)[1].replace("**", "").strip()
                break
        if not scope:
            for line in lines[10:35]:
                if line.strip() and not line.startswith("#") and not line.startswith("**") and not line.startswith("---") and not line.startswith("```"):
                    scope = line.strip()
                    break
        if len(scope) > 130:
            scope = scope[:127] + "..."

        file_info[fname] = {
            "h1": h1,
            "short_title": short_title,
            "scope": scope.strip(),
            "slug": fname.replace(".md", "")
        }

    groups = [
        (
            "1. Core Foundations & Source Domain",
            "Foundational domain models, typed identities, invariants, VCS neutrality, and Change Proposals.",
            [
                "01-forgeyard-core-domain-foundation.md",
                "forgeyard-change-proposal-system-architecture.md",
                "forgeyard-vcs-neutral-system-architecture.md"
            ]
        ),
        (
            "2. Storage, CAS & Data Plane",
            "Metadata persistence, Content-Addressable Storage (CAS), remote caching, lifecycle policies, and schema evolution.",
            [
                "02-forgeyard-storage-metadata.md",
                "03-forgeyard-cas-artifact-data-plane.md",
                "38-forgeyard-cache-build-acceleration-remote-cache-correctness-system-architecture.md",
                "46-forgeyard-data-lifecycle-retention-archival-deletion-privacy-governance-system-architecture.md",
                "52-forgeyard-artifact-registry-package-repository-oci-internal-distribution-system-architecture.md",
                "63-forgeyard-database-schema-migration-online-backfill-data-transformation-zero-downtime-system-architecture.md"
            ]
        ),
        (
            "3. Pipeline Engine, IR, State Machine & Scheduling",
            "Intermediate Representation (IR), DAG evaluation, run/job state transitions, distributed scheduling, concurrency locks, and checkpointing.",
            [
                "04-forgeyard-pipeline-ir-parsing-planning.md",
                "05-forgeyard-run-job-state-machine.md",
                "06-forgeyard-scheduler-system-architecture.md",
                "10-forgeyard-events-reconciliation-system-architecture.md",
                "44-forgeyard-pipeline-triggers-schedules-manual-dispatch-event-driven-system-architecture.md",
                "60-forgeyard-workflow-concurrency-distributed-locks-idempotency-reservations-exclusive-coordination-system-architecture.md",
                "69-forgeyard-job-checkpointing-suspend-resume-preemption-graceful-cancellation-interruptible-runner-recovery-system-architecture.md"
            ]
        ),
        (
            "4. Runners, Sandboxing, Transport & Fleet Orchestration",
            "Runner daemon agents, sandbox execution environments (microVM/container/chroot), internal QUIC transport, device testbeds, autoscaling, and zero-trust tunneling.",
            [
                "07-forgeyard-runner-agent-system-architecture.md",
                "08-forgeyard-sandbox-executor-system-architecture.md",
                "09-forgeyard-transport-quic-internal-protocol.md",
                "20-forgeyard-device-lab-system-architecture.md",
                "43-forgeyard-runner-fleet-autoscaling-capacity-provisioning-infrastructure-system-architecture.md",
                "58-forgeyard-runner-image-factory-golden-image-patch-management-baseline-attestation-system-architecture.md",
                "59-forgeyard-network-connectivity-private-resource-access-egress-tunneling-zero-trust-system-architecture.md"
            ]
        ),
        (
            "5. Security, Policy, Secrets & Supply Chain",
            "Fine-grained ABAC/RBAC authorization, secrets vaults, cryptographic provenance/SBOM signing, multi-tenancy quotas, audit compliance, and threat mitigation.",
            [
                "11-forgeyard-policy-authorization-identity-system-architecture.md",
                "12-forgeyard-secrets-trust-credential-security-system-architecture.md",
                "13-forgeyard-supply-chain-sbom-provenance-signing-system-architecture.md",
                "27-forgeyard-multi-tenancy-quotas-resource-governance-system-architecture.md",
                "28-forgeyard-audit-compliance-security-governance-system-architecture.md",
                "40-forgeyard-security-threat-model-hardening-incident-response-system-architecture.md"
            ]
        ),
        (
            "6. Packaging, Releases, Deployment & Delivery",
            "Hermetic functional packaging, reproducible distribution, deployment orchestration, automated release channels, progressive delivery, and artifact promotion.",
            [
                "14-forgeyard-packaging-system-architecture.md",
                "15-forgeyard-release-system-architecture.md",
                "16-forgeyard-deployment-system-architecture.md",
                "41-forgeyard-release-distribution-update-delivery-installer-channel-system-architecture.md",
                "62-forgeyard-environment-promotion-progressive-delivery-feature-rollout-canary-rollback-system-architecture.md",
                "67-forgeyard-artifact-promotion-policy-release-train-environment-channel-lifecycle-governance-system-architecture.md",
                "forgeyard-hermetic-functional-packaging-architecture.md"
            ]
        ),
        (
            "7. Observability, Diagnostics, Testing & Quality",
            "Telemetry & doctor diagnostics, operational search analytics, quality gates & flaky test intelligence, performance benchmarking, static analysis, failure bisecting, SLO error budgets, and incident postmortems.",
            [
                "17-forgeyard-observability-health-doctor-system-architecture.md",
                "31-forgeyard-search-indexing-query-operational-analytics-system-architecture.md",
                "32-forgeyard-test-results-quality-gates-coverage-flaky-intelligence-system-architecture.md",
                "33-forgeyard-benchmark-performance-regression-load-capacity-system-architecture.md",
                "37-forgeyard-static-analysis-code-quality-security-findings-system-architecture.md",
                "48-forgeyard-failure-diagnosis-debugging-reproduction-bisect-root-cause-system-architecture.md",
                "50-forgeyard-reliability-slo-error-budget-availability-resilience-governance-system-architecture.md",
                "56-forgeyard-test-data-fixtures-ephemeral-databases-service-virtualization-system-architecture.md",
                "61-forgeyard-incident-management-oncall-escalation-response-postmortem-system-architecture.md"
            ]
        ),
        (
            "8. API, UI, DevEx & Service Portal",
            "Axum HTTP/gRPC gateway, Dioxus Web/Desktop GUI, local CLI / workstation ergonomics, golden path workflow templates, developer service catalog, API/ABI compatibility, and cloud workspace orchestration.",
            [
                "18-forgeyard-api-axum-system-architecture.md",
                "19-forgeyard-dioxus-ui-gui-system-architecture.md",
                "35-forgeyard-developer-experience-local-dev-cli-reproducible-workstation-system-architecture.md",
                "42-forgeyard-workflow-templates-reusable-pipelines-golden-paths-system-architecture.md",
                "49-forgeyard-service-catalog-component-ownership-environment-inventory-developer-portal-system-architecture.md",
                "57-forgeyard-api-abi-schema-protocol-compatibility-contract-evolution-system-architecture.md",
                "64-forgeyard-remote-development-environments-cloud-workspaces-developer-workspace-orchestration-system-architecture.md"
            ]
        ),
        (
            "9. Distributed Coordination, Federation & Operations",
            "SCM provider integrations, Raft consensus high availability, Remote Build Execution (RBE) interop, WASM plugin extensions, disaster recovery, air-gapped self-hosting, human approval workflows, subscription licensing, monorepo graph analysis, dependency mirroring, configuration drift detection, merge queues, and AI-assisted CI optimization.",
            [
                "21-forgeyard-scm-provider-integrations-system-architecture.md",
                "22-forgeyard-ha-coordination-raft-system-architecture.md",
                "23-forgeyard-rbe-interop-system-architecture.md",
                "24-forgeyard-plugin-extension-system-architecture.md",
                "25-forgeyard-operations-backup-upgrade-dr-system-architecture.md",
                "26-forgeyard-self-hosting-bootstrap-release-system-architecture.md",
                "29-forgeyard-notifications-alerting-human-workflow-system-architecture.md",
                "30-forgeyard-entitlements-licensing-subscription-commercial-access-system-architecture.md",
                "34-forgeyard-monorepo-dependency-graph-affected-incremental-execution-system-architecture.md",
                "36-forgeyard-dependency-package-registry-artifact-mirror-source-governance-system-architecture.md",
                "39-forgeyard-configuration-feature-flags-runtime-settings-governance-system-architecture.md",
                "45-forgeyard-cost-accounting-finops-chargeback-showback-resource-economics-system-architecture.md",
                "47-forgeyard-cicd-migration-import-compatibility-legacy-interoperability-system-architecture.md",
                "51-forgeyard-multi-region-federation-edge-disconnected-cross-site-replication-system-architecture.md",
                "53-forgeyard-infrastructure-as-code-environment-provisioning-preview-drift-system-architecture.md",
                "54-forgeyard-merge-queue-speculative-integration-batch-validation-protected-target-system-architecture.md",
                "55-forgeyard-ai-assisted-ci-optimization-engineering-copilot-autonomous-recommendation-governance-system-architecture.md",
                "65-forgeyard-build-graph-replay-historical-reproducibility-time-travel-ci-evidence-reconstruction-system-architecture.md",
                "66-forgeyard-change-risk-assessment-preflight-simulation-policy-preview-what-if-analysis-system-architecture.md",
                "68-forgeyard-configuration-drift-desired-state-convergence-runtime-reconciliation-environment-consistency-system-architecture.md",
                "70-forgeyard-dependency-update-automation-version-maintenance-vulnerability-remediation-upgrade-campaign-system-architecture.md"
            ]
        ),
        (
            "10. Ecosystems & Language Toolchains",
            "Targeted architecture specifications for hermetic multi-language compilation, dependency management, toolchain isolation, and test runners.",
            [
                "forgeyard-assembly-native-architecture.md",
                "forgeyard-c-cpp-system-architecture.md",
                "forgeyard-dart-flutter-system-architecture.md",
                "forgeyard-go-system-architecture.md",
                "forgeyard-java-kotlin-jvm-system-architecture.md",
                "forgeyard-javascript-typescript-system-architecture.md",
                "forgeyard-python-system-architecture.md",
                "forgeyard-rust-system-architecture.md",
                "forgeyard-swift-system-architecture.md"
            ]
        )
    ]

    # Generate Home.md
    home_lines = []
    home_lines.append("# Forgeyard System Architecture Wiki\n")
    home_lines.append("Welcome to the official **Forgeyard Architecture Wiki**. This knowledge base contains the complete architectural specifications, execution models, protocol contracts, domain abstractions, and toolchain definitions for Forgeyard.\n")
    home_lines.append("Forgeyard is a next-generation, high-performance, reproducible, and hermetic CI/CD and software delivery platform engineered as a modular Rust workspace.\n")
    home_lines.append("---\n")
    home_lines.append("## High-Level System Topology\n")
    home_lines.append("The diagram below illustrates the architectural layers and inter-subsystem data flows across the Forgeyard platform:\n")
    home_lines.append("```mermaid")
    home_lines.append("flowchart TB")
    home_lines.append("    subgraph ClientLayer[\"User & Interface Layer\"]")
    home_lines.append("        CLI[\"Forgeyard CLI\"]")
    home_lines.append("        GUI[\"Dioxus UI / GUI (Web & Desktop)\"]")
    home_lines.append("        SCMHook[\"SCM Webhooks (GitHub / GitLab / Gitea)\"]")
    home_lines.append("        IDE[\"Dev Workspace & IDE Bridge\"]")
    home_lines.append("    end")
    home_lines.append("")
    home_lines.append("    subgraph GatewayLayer[\"API & Control Ingress\"]")
    home_lines.append("        AxumGW[\"Axum API Gateway (REST / gRPC / WebSocket)\"]")
    home_lines.append("        AuthN[\"Identity & Token Verification\"]")
    home_lines.append("        PolicyEngine[\"Policy & RBAC/ABAC Evaluator\"]")
    home_lines.append("    end")
    home_lines.append("")
    home_lines.append("    subgraph CorePlane[\"Core Orchestration & Planning Plane\"]")
    home_lines.append("        IRPlanner[\"Pipeline IR Parser & DAG Planner\"]")
    home_lines.append("        StateMachine[\"Run & Job State Machine\"]")
    home_lines.append("        Scheduler[\"Distributed Scheduler & Resource Matcher\"]")
    home_lines.append("        Reconciliation[\"Event Bus & State Reconciliation\"]")
    home_lines.append("        LockCoord[\"Distributed Lock & Idempotency Engine\"]")
    home_lines.append("    end")
    home_lines.append("")
    home_lines.append("    subgraph StoragePlane[\"Storage & Content-Addressable Plane\"]")
    home_lines.append("        MetaDB[\"Metadata & State Database\"]")
    home_lines.append("        CAS[\"CAS (Content-Addressable Storage) Engine\"]")
    home_lines.append("        RemoteCache[\"Remote Build Cache & Acceleration\"]")
    home_lines.append("        ArtifactReg[\"Artifact Registry & OCI Distribution\"]")
    home_lines.append("    end")
    home_lines.append("")
    home_lines.append("    subgraph ExecutionPlane[\"Execution & Runner Plane\"]")
    home_lines.append("        QUIC[\"QUIC Secure Internal Transport\"]")
    home_lines.append("        RunnerAgent[\"Runner Daemon Agent\"]")
    home_lines.append("        SandboxMicroVM[\"MicroVM / Container Sandbox Executor\"]")
    home_lines.append("        DeviceLab[\"Hardware & Device Testbed\"]")
    home_lines.append("        FleetAutoscaler[\"Runner Fleet Autoscaler\"]")
    home_lines.append("    end")
    home_lines.append("")
    home_lines.append("    subgraph SecurityPlane[\"Security, Trust & Supply Chain\"]")
    home_lines.append("        SecretsVault[\"Secrets Vault & Zero-Trust Ephemeral Creds\"]")
    home_lines.append("        SBOMSign[\"SBOM & Cryptographic Provenance (Cosign/In-Toto)\"]")
    home_lines.append("        AuditCompliance[\"Audit Log & Compliance Governance\"]")
    home_lines.append("    end")
    home_lines.append("")
    home_lines.append("    CLI --> AxumGW")
    home_lines.append("    GUI --> AxumGW")
    home_lines.append("    SCMHook --> AxumGW")
    home_lines.append("    IDE --> AxumGW")
    home_lines.append("")
    home_lines.append("    AxumGW --> AuthN --> PolicyEngine --> IRPlanner")
    home_lines.append("    IRPlanner --> StateMachine --> Scheduler")
    home_lines.append("    Scheduler --> Reconciliation")
    home_lines.append("    StateMachine --> LockCoord")
    home_lines.append("")
    home_lines.append("    StateMachine --> MetaDB")
    home_lines.append("    IRPlanner --> CAS")
    home_lines.append("    Scheduler --> QUIC")
    home_lines.append("")
    home_lines.append("    QUIC --> RunnerAgent")
    home_lines.append("    RunnerAgent --> SandboxMicroVM")
    home_lines.append("    RunnerAgent --> DeviceLab")
    home_lines.append("    FleetAutoscaler --> RunnerAgent")
    home_lines.append("")
    home_lines.append("    SandboxMicroVM --> RemoteCache")
    home_lines.append("    SandboxMicroVM --> CAS")
    home_lines.append("    SandboxMicroVM --> ArtifactReg")
    home_lines.append("")
    home_lines.append("    SandboxMicroVM --> SecretsVault")
    home_lines.append("    SandboxMicroVM --> SBOMSign")
    home_lines.append("    Reconciliation --> AuditCompliance")
    home_lines.append("```\n")
    home_lines.append("---\n")
    home_lines.append("## Architectural Taxonomy (82 Specifications)\n")
    home_lines.append("The 82 system architecture specifications are organized into 10 cohesive architectural domains:\n")

    for gtitle, gdesc, flist in groups:
        home_lines.append(f"### {gtitle}\n")
        home_lines.append(f"*{gdesc}*\n")
        home_lines.append("| Specification | Document | Summary Scope |")
        home_lines.append("| :--- | :--- | :--- |")
        for fname in flist:
            info = file_info[fname]
            slug = info["slug"]
            short_t = info["short_title"]
            scope = info["scope"]
            home_lines.append(f"| **{short_t}** | [`{slug}`]({slug}) | {scope} |")
        home_lines.append("\n---\n")

    home_lines.append("## Global Design Principles\n")
    home_lines.append("1. **Strict Type Safety & Strong Invariants:** All identities (runs, jobs, artifacts, runners, nodes) are strongly typed UUID/ULID primitives.")
    home_lines.append("2. **Hermeticity & Reproducibility:** Every build and test step runs in content-addressed, hermetic sandboxes with explicit inputs and outputs.")
    home_lines.append("3. **Transport Efficiency:** Control plane and runner nodes communicate via multiplexed QUIC streams with mutual TLS.")
    home_lines.append("4. **VCS Neutrality:** Universal Git/Jujutsu/Mercurial/Pijul source abstraction layer decoupling Forgeyard from any single SCM provider.")
    home_lines.append("5. **Supply Chain Integrity:** Built-in cryptographic provenance attestation, SBOM generation, and SLSA Level 3+ compliance by default.\n")
    home_lines.append("---\n")
    home_lines.append("*This Wiki is automatically synchronized from the [`sys-arch/`](https://github.com/irshadali5/forgeyard/tree/main/sys-arch) directory in the Forgeyard repository.*\n")

    with open(os.path.join(wiki_dir, "Home.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(home_lines))

    # Generate _Sidebar.md
    sidebar_lines = []
    sidebar_lines.append("### [Forgeyard Wiki](Home)\n")
    sidebar_lines.append("* [Wiki Home](Home)\n")
    sidebar_lines.append("---\n")

    for gtitle, gdesc, flist in groups:
        sidebar_title = re.sub(r"^\d+\.\s*", "", gtitle)
        sidebar_lines.append("<details open>")
        sidebar_lines.append(f"<summary><b>{sidebar_title}</b></summary>\n")
        for fname in flist:
            info = file_info[fname]
            slug = info["slug"]
            short = info["short_title"]
            sidebar_lines.append(f"- [{short}]({slug})")
        sidebar_lines.append("\n</details>\n")

    sidebar_lines.append("---\n")
    sidebar_lines.append("* [GitHub Repository](https://github.com/irshadali5/forgeyard)")
    sidebar_lines.append("* [System Architecture Directory](https://github.com/irshadali5/forgeyard/tree/main/sys-arch)\n")

    with open(os.path.join(wiki_dir, "_Sidebar.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(sidebar_lines))

    # Generate _Footer.md
    footer_content = """---
<div align="center">

**Forgeyard CI/CD & Build Platform** · [Wiki Index](Home) · [GitHub Repository](https://github.com/irshadali5/forgeyard) · [Architecture Specifications](https://github.com/irshadali5/forgeyard/tree/main/sys-arch)

*All architecture specifications are released under project governance guidelines.*

</div>
"""
    with open(os.path.join(wiki_dir, "_Footer.md"), "w", encoding="utf-8") as f:
        f.write(footer_content)

    print(f"Successfully generated Home.md, _Sidebar.md, and _Footer.md in {wiki_dir}!")

if __name__ == "__main__":
    main()
