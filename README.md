# Forgeyard

> **High-Performance, Cloud-Native Distributed Build, CI/CD Orchestration, and Codebase Intelligence Engine written in Rust.**

---

> [!IMPORTANT]
> ### 🤖 AI-Engineered & Human-Directed Development Notice
> **Transparency Statement**: All code within this repository was generated and written with AI agent collaboration, but strictly guided, engineered, audited, and driven by **human architectural oversight, technical decision-making, security research, and domain expertise**. Every component, crate dependency, storage model, and control flow decision reflects deliberate human design choices for maximum performance, memory safety, and cross-platform reliability.

---

## 🚀 Overview

**Forgeyard** is an enterprise-grade, lightweight, sub-millisecond build and workflow execution engine designed for high-concurrency CI/CD pipelines, local developer environments, and automated agentic AI toolchains. 

Engineered entirely in Rust, Forgeyard replaces heavy, external database dependencies with lightweight embedded storage engines (**Stoolap**, **Redb**, **QuickCache**) and provides real-time AST knowledge graph extraction (**Graphify**) to empower AI agents with token-efficient codebase understanding.

---

## ✨ Key Architectural Highlights

### ⚡ Hybrid OLAP/OLTP & Semantic Vector Storage
- **Unified Engine (`Stoolap`)**: Replaces traditional heavy SQL engines with an embedded, zero-config hybrid database supporting both transactional (OLTP) and analytical (OLAP) queries.
- **Built-in Vector & Semantic Indexing**: Enables fast vector similarity searches over build artifacts, task logs, and codebase AST relationships without third-party vector databases.

### 🛡️ Layered High-Performance Caching
- **L1 In-Memory Cache (`quick-cache`)**: Hyper-concurrent, low-lock-contention RAM cache for fast memory access.
- **L2 Persistent Disk Store (`redb`)**: Copy-on-Write ACID B-Tree key-value database for durable blob storage, configuration state, and task artifacts with zero crash corruption risk.

### 🧠 AST Knowledge Graph Extraction (`Graphify`)
- Built-in AST analysis using `graphify-core` and `graphify-extract` powered by Tree-sitter parsers.
- Dynamically extracts workspace relationships, call graphs, import dependencies, and symbol definitions into a minimal token footprint tailored for Large Language Model (LLM) context windows.

### 🔀 Concurrency & Data Parallelism
- Integrated **`rayon`** thread pools wrapped in Tokio async tasks for multi-threaded file detection, parallel code parsing, and high-throughput job dependency scheduling.

### 🔒 Enterprise Security, Signing & SLSA Provenance
- **Ed25519 Cryptographic Signing (`ed25519-dalek`)**: Digitally signs build provenance and execution records (`SignedProvenance`).
- **Security Policy Engine (`forgeyard-policy`)**: Analyzes job execution specifications against command safety rules, privilege constraints, and forbidden pattern filters.
- **Log Sanitization & Redaction**: Built-in streaming log redactor preventing token and secret leaks.

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

## 🛡️ Security & Bug Reporting

Security research and vulnerability disclosure are essential to the Forgeyard ecosystem.

- **Vulnerability Disclosures**: If you discover a security vulnerability or security flaw, please do **NOT** open a public issue. Send a detailed report directly to the core maintainers.
- **Bug Reports & Contributions**: Bug fixes, performance enhancements, and security improvements are welcome via pull requests following our contribution guidelines.

---

## ⚖️ Dual-License & Contribution Model

Forgeyard is open-source software released under a **Dual Licensing Model**:

1. **AGPLv3 (GNU Affero General Public License v3)**: 
   Free to use, inspect, and modify for non-commercial, open-source, educational, and personal projects.
2. **Commercial / Enterprise License**: 
   Required for commercial products, proprietary cloud services, or closed-source enterprise deployments that modify or integrate Forgeyard without disclosing source code under AGPLv3 terms. Contact the project founder for commercial license acquisition.

### 📝 Contributor License Terms
By contributing to this repository (via Pull Requests, Code Submissions, or Patches), **all contributors agree that their contributions are automatically licensed under this same Dual-License Model** (AGPLv3 + Commercial License), granting full dual-licensing authorization to the founder and lead maintainer of Forgeyard.

---

<p align="center">
  <b>Designed for speed. Engineered for reliability. Built for the future of AI & software engineering.</b>
</p>
