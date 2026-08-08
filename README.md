# Forgeyard

> **High-Performance, Cloud-Native Distributed Build, CI/CD Orchestration, and Codebase Intelligence Engine written in Rust.**

---

> [!IMPORTANT]
>
> ### 🤖 2–3 Year Pre-v1.0 Continuous Development & Enterprise Hardening Roadmap (2026–2029)
>
> **Product Lifecycle Statement**: Forgeyard is undergoing a **multi-year (2–3 year) active pre-v1.0 development phase**. We remain under `v0.x` to continuously iterate, battle-test, benchmark, and security-audit all 37 workspace crates across global multi-cloud and bare-metal edge workloads. Official release `v1.0.0` will be declared once long-term production stability, enterprise battle-testing, and strict API stability contracts are proven worldwide.

---

## 🚀 Overview

**Forgeyard** is an enterprise-grade, lightweight, sub-millisecond build and workflow execution engine designed for high-concurrency CI/CD pipelines, local developer environments, and automated agentic AI toolchains.

Engineered entirely in Rust, Forgeyard replaces heavy, external database dependencies with lightweight embedded storage engines (**Stoolap**, **Redb**, **QuickCache**) and provides real-time AST knowledge graph extraction (**Graphify**) to empower AI agents with token-efficient codebase understanding.

---

## ✨ Key Architectural Highlights

### ⚡ Hybrid OLAP/OLTP & Semantic Vector Storage

- **Unified Engine (`Stoolap`)**: Replaces traditional heavy SQL engines with an embedded, zero-config hybrid database supporting both transactional (OLTP) and analytical (OLAP) queries.
- **Built-in Vector & Semantic Indexing**: Enables fast vector similarity searches over build artifacts, task logs, and codebase AST relationships without third-party vector databases.

### 🚀 Linux Kernel `io_uring` Zero-Copy Engine

- **Asynchronous Submission Queue (SQ) & Completion Queue (CQ)**: Submits kernel read and write ring operations directly via `io-uring` (`IoUringCasEngine`, `IoUringLogWriter`) to bypass traditional POSIX syscall context switching overhead.
- **Workspace-Wide High Throughput**: Accelerated I/O for CAS blob streaming, file logging, BLAKE3 checksum hashing (`compute_blake3`), encrypted secret vault persistence (`persist_vault_io_uring`), and XML/JSON report parsing (`parse_file_io_uring`).
- **Cross-Platform Parity & Fallback**: Runtime kernel capability detection (`is_io_uring_supported()`) with automatic fallback to standard Tokio async file I/O on macOS, Windows, or older Linux kernels.

### 🛡️ Layered High-Performance Caching

- **L1 In-Memory Cache (`quick-cache`)**: Hyper-concurrent, low-lock-contention RAM cache for fast memory access.
- **L2 Persistent Disk Store (`redb`)**: Copy-on-Write ACID B-Tree key-value database for durable blob storage, configuration state, and task artifacts with zero crash corruption risk.

### 🧠 RTK Token Compression & CodeGraph Engine

- **Signature Extraction & Context Trimming (`RtkCompressor`)**: Extracts public function signatures, struct layouts, and cross-file `CallEdge` relationships while trimming internal logic to fit strictly within AI LLM context windows.
- **Unified CodeGraph Taxonomy**: Symbol classification across `Function`, `Method`, `Struct`, `Trait`, `Enum`, `Module`, and `Interface`.

### 📊 OpenTelemetry & Distributed Observability Tracing

- **Span Lifecycle Management (`OtelSpan` & `TelemetryExporter`)**: Generates structured distributed trace spans (`Server`, `Client`, `Internal`) with parent-child trace propagation for job scheduling, runner leasing, execution timing, and CAS transfer latency.

### 🔒 Envelope Encryption & Bearer Auth API

- **Encrypted Vault Persistence**: Encrypts pipeline secrets in-memory and on-disk using `EncryptedVaultBackend` with BLAKE3 key derivation and XOR memory masking.
- **REST & WebSocket API Auth**: Protects all `/api/v1/*` endpoints in `forgeyard-daemon` with Bearer token authentication middleware.

### 🧠 Edge AI Quantized Local Model Acceleration (Phase 19)

- **`LocalEdgeAiEngine` & Quantized Inference**: Embedded local GGUF/ONNX quantized LLM inference runtime (`GgufQ4`, `GgufQ8`, `OnnxInt8`) in `forgeyard-analyzer` for zero-latency, zero-cost offline AI code fix generation and AST graph intent reasoning.

### 🔒 Confidential Computing & Hardware Enclave Attestation (Phase 20)

- **`ConfidentialEnclaveExecutor`**: Executes sensitive build steps inside hardware-encrypted memory enclaves (AMD SEV-SNP, Intel SGX, AWS Nitro Enclaves) in `forgeyard-sandbox` with cryptographic attestation measurement hashes (`blake3`) proving isolation from host OS compromise.

### 🛠️ Autonomous Flaky Test Root Cause Synthesizer (Phase 21)

- **`FlakyRootCauseSynthesizer`**: AST delta analysis comparing passing vs failing execution traces in `forgeyard-test-report` to automatically diagnose `AsyncTimingLock`, `UnorderedMapIteration`, and `PortConflict` race conditions and synthesize code remediation patches.

### 🛡️ Enterprise eBPF XDP Firewall & DDoS Mitigation (Phase 22)

- **`EbpfXdpFirewall` & XDP Packet Filtering**: Wire-speed Linux kernel eXpress Data Path (XDP) network packet filtering in `forgeyard-daemon` providing IP rate-limiting, DDoS mitigation, and per-tenant network isolation.

### 📋 Continuous Compliance & SOC2 / ISO 27001 Audit Ledger (Phase 23)

- **`ComplianceAuditLedger`**: Immutable audit log generation exporting SLSA v1.0 attestations, eBPF telemetry, and policy findings into automated SOC2 Type II, ISO 27001, HIPAA, and SLSA Level 3 compliance reports with cryptographic BLAKE3 audit signatures.

### 🌌 Distributed P2P CAS Cache Coalescing & Swarm Seeding (Phase 24)

- **`P2pCasSeeder`**: Bit-torrent style distributed artifact blob seeding in `forgeyard-cas` across edge runner nodes with chunk deduplication, peer health scoring, and bandwidth throttling.

### 🔄 GitHub Actions & GitLab CI Pipeline Converters

- **Automated Workflow Translation**: Converts `.github/workflows/*.yml` and `.gitlab-ci.yml` configs into native `forgeyard.ron` IR using `GitHubWorkflowConverter`, `GitLabCIConverter`, and the `forgeyard import` CLI command.

### 📦 Multi-Platform Native Package Generators & Deployment

- **Native Package Generators**: Automated `.apk`, `.deb`, `.msi`, `.app`, `.tar.gz`, and `.zip` bundle creation in `forgeyard-packaging`.
- **Cloud & Registry Deployment**: Built-in deployment drivers for AWS S3, OCI/Docker container registries, SSH servers, and GitHub Releases in `forgeyard-deploy`.

---

## 🏗️ Workspace Crate Architecture

Forgeyard is structured as a clean modular Rust workspace consisting of specialized crates:

```
forgeyard/
├── crates/
│   ├── forgeyard-agent/         # Distributed build agent runner process
│   ├── forgeyard-analyzer/      # AST knowledge graph parser (Graphify integration)
│   ├── forgeyard-api/           # REST and gRPC/QUIC API data transfer objects
│   ├── forgeyard-archive/       # High-compression tar/zip/zstd archive utilities
│   ├── forgeyard-artifacts/     # Artifact CAS registration and tracking
│   ├── forgeyard-cache/         # Tiered caching engine (QuickCache + Redb)
│   ├── forgeyard-cas/           # Content-Addressable Storage engine
│   ├── forgeyard-cli/           # Main command-line interface binary
│   ├── forgeyard-config/        # Layered RON & environment configuration manager
│   ├── forgeyard-daemon/        # Central orchestration daemon & QUIC server
│   ├── forgeyard-deploy/        # Artifact publisher & deployment drivers (OCI, S3)
│   ├── forgeyard-detector/      # Parallel workspace project scanner (Rayon)
│   ├── forgeyard-device-lab/    # Real-time Android ADB & device lab manager
│   ├── forgeyard-events/        # High-throughput event bus (Stoolap-backed)
│   ├── forgeyard-executor/      # Execution drivers (Process, Container, Apple, Android, Windows)
│   ├── forgeyard-logs/          # Streaming append-only logging system
│   ├── forgeyard-model/         # Core IR, Job, Pipeline, and Run definitions
│   ├── forgeyard-packaging/     # Native package generators (APK, MSI, AppBundle)
│   ├── forgeyard-pipeline/      # Matrix expansion & DAG dependency resolver
│   ├── forgeyard-policy/        # Security policy evaluator & sandbox rules
│   ├── forgeyard-protocol/      # Binary QUIC wire protocol schemas
│   ├── forgeyard-provenance/    # SLSA build provenance record generator
│   ├── forgeyard-runner/        # Local execution worker process
│   ├── forgeyard-sandbox/       # Isolated sandbox process isolation wrappers
│   ├── forgeyard-scheduler/     # Task queue & worker allocation scheduler
│   ├── forgeyard-secrets/       # Secret broker with .env & environment resolution
│   ├── forgeyard-signing/       # Ed25519 digital signature provider
│   ├── forgeyard-storage/       # Stoolap OLAP/OLTP database storage interface
│   ├── forgeyard-test-report/   # Cargo test JSON parser and reporting
│   ├── forgeyard-toolchains/    # Automatic toolchain installer (Rustup/Cargo)
│   ├── forgeyard-ui/            # Web dashboard interface assets
│   └── forgeyard-adapter-*/     # Ecosystem adapters (Cargo, Dioxus, OCI, WASM, Xcode, Android)
├── Cargo.toml                   # Workspace manifest
└── plan.md                      # Architectural roadmap & master plan
```

---

## ⚡ Quickstart

> 📖 **Comprehensive Step-by-Step Guide**: See [`tutorial.md`](file:///home/irshad/Projects/forgeyard/tutorial.md) for full local testing instructions, daemon setup, edge worker management, log tailing, and `io_uring` verification.

### Prerequisites

- **Rust Toolchain**: `rustc` 1.85+ and `cargo`

### 1. Build the Workspace

```bash
cargo build --workspace --release
```

### 2. Start the Daemon

```bash
./target/release/forgeyard-daemon --http-port 8080 --quic-port 4433
```

### 3. Run a Pipeline via CLI

```bash
./target/release/forgeyard-cli run --config forgeyard.ron
```

---

## 🤝 Help & Contributions Needed

Development of Forgeyard is an **ongoing effort**, and we actively seek and welcome community and enterprise assistance!

We invite **help, contributions, security research, bug reports, security auditing, documentation improvements, and architectural feedback** from developers, engineers, and researchers worldwide.

- **Security Researchers**: If you audit or find vulnerabilities, please report them to help make Forgeyard more resilient.
- **Developers & Contributors**: Help us expand ecosystem adapters, optimize build runners, write tests, and refine documentation.

---

## ⚖️ Dual-License & Comprehensive Contribution Terms

Forgeyard is open-source software released under a **Dual Licensing Model**:

1. **AGPLv3 (GNU Affero General Public License v3)**:
   Free to use, inspect, and modify for non-commercial, open-source, educational, and personal projects.
2. **Commercial / Enterprise License**:
   Required for business products, commercial applications, proprietary cloud services, or closed-source enterprise deployments that modify or integrate Forgeyard without disclosing source code under AGPLv3 terms. Contact the project founder for commercial licensing inquiries.

### 📝 Founder & Maintainer Contribution Terms

As intended by the project founder and lead maintainer:
> **ANY AND ALL CONTRIBUTIONS**—including but not limited to code submissions, pull requests, patches, security research, vulnerability disclosures, bug reports, documentation, feature suggestions, or any other valuable and helpful input provided to this repository—**are automatically submitted and licensed under this Dual-License Model** (AGPLv3 + Commercial License), granting full dual-licensing rights, commercial distribution rights, and re-licensing authority to the founder and lead maintainer of Forgeyard.

---

<p align="center">
  <b>Designed for speed. Engineered for reliability. Built for the future of AI & software engineering.</b>
</p>
