# 🚀 Forgeyard Release v0.9.0-dev (Pre-v1.0 Preview)

> **High-Performance, Cloud-Native Distributed Build, CI/CD Orchestration, and Codebase Intelligence Engine**

---

## 🌟 Release v0.9.0-dev Highlights

Forgeyard v0.9.0-dev is a pre-v1.0 active development release featuring **24 master architectural implementation phases** built entirely in Rust across 37 modular workspace crates. We remain under v1.0 to iterate rapidly and battle-test features before v1.0 enterprise readiness.

---

## ✨ Key Features & Architecture Summary

### 1. ⚡ Embedded Database & CAS Core
- **`Stoolap` Hybrid Database**: Embedded zero-config OLAP/OLTP engine backed by SQLite dialect with vector similarity search capabilities.
- **Linux Kernel `io_uring` Ring I/O**: Zero-copy submission/completion queue processing (`IoUringCasEngine`, `IoUringLogWriter`) for ultra-low latency CAS blob transfers and logging.
- **Tiered Cache Engine**: L1 `quick-cache` in-memory RAM cache paired with L2 `redb` ACID persistent key-value store.

### 2. 🧠 Codebase Intelligence & Edge AI
- **`RtkCompressor` Signature Extraction**: Extracts public AST API surfaces and dependency call graphs (`Graphify`) for token-efficient LLM context windows.
- **`LocalEdgeAiEngine` (Phase 19)**: Offline GGUF/ONNX quantized LLM inference runtime (`GgufQ4`, `GgufQ8`, `OnnxInt8`) for zero-cost, zero-latency code fix generation and AST graph intent reasoning.

### 3. 🔒 Zero-Trust Security & Confidential Computing
- **`ConfidentialEnclaveExecutor` (Phase 20)**: Hardware-encrypted memory enclave execution (AMD SEV-SNP, Intel SGX, AWS Nitro Enclaves) with BLAKE3 cryptographic measurement attestation.
- **Encrypted Secret Vault**: In-memory and disk secret encryption (`EncryptedVaultBackend`) with XOR memory masking and BLAKE3 key derivation.
- **Hybrid Post-Quantum Signing (Phase 13)**: Dual Ed25519 + ML-DSA-87 signatures for SLSA v1.0 attestations.
- **Zero-Knowledge STARK Proofs (Phase 15)**: `ZkProofGenerator` STARK statements proving build output integrity without revealing confidential source code.

### 4. 🌐 Distributed Scheduling & Networking
- **Multi-Region Cloud Failover (Phase 17)**: Automated cross-cloud region latency probing and dynamic failover across AWS, GCP, Azure, and edge clusters.
- **GPU Resource Scheduling (Phase 14)**: CUDA/Vulkan VRAM profiling and Tensor Core work-stealing score allocation.
- **Enterprise eBPF XDP Firewall (Phase 22)**: Wire-speed Linux kernel eXpress Data Path (XDP) network packet filtering, IP rate-limiting, and DDoS protection.
- **Distributed P2P CAS Swarm (Phase 24)**: Bit-torrent style P2P artifact blob seeding (`P2pCasSeeder`) across edge runner nodes.

### 5. 🛠️ DevSecOps, Compliance & Observability
- **Autonomous Flaky Test Synthesizer (Phase 21)**: AST delta analysis diagnosing async timing locks, port conflicts, and race conditions with auto-fix patch generation.
- **Continuous Compliance Audit Ledger (Phase 23)**: `ComplianceAuditLedger` exporting SLSA v1.0 attestations into automated SOC2 Type II, ISO 27001, HIPAA, and SLSA Level 3 compliance audit logs.
- **Interactive Teleport Shell (Phase 18)**: Interactive PTY bash shell sessions (`create_pty_session`) for real-time container debugging.
- **OpenTelemetry Tracing**: Spans with parent-child propagation for scheduler, runner, daemon, and CAS transfers.

---

## 📦 Workspace Package Matrix (37 Crates)

| Crate Name | Description |
| :--- | :--- |
| `forgeyard-agent` | Distributed QUIC build agent runner process |
| `forgeyard-analyzer` | AST knowledge graph & quantized Edge AI engine |
| `forgeyard-api` | REST & WebSocket DTO definitions |
| `forgeyard-archive` | High-compression tar/zip/zstd utilities |
| `forgeyard-artifacts` | Artifact CAS registration & tracking |
| `forgeyard-cache` | QuickCache + Redb L1/L2 tiered caching |
| `forgeyard-cas` | Content-Addressable Storage & P2P swarm seeder |
| `forgeyard-cli` | Main CLI binary (`forgeyard`) |
| `forgeyard-config` | RON & env configuration manager |
| `forgeyard-daemon` | QUIC/mDNS orchestration daemon & eBPF firewall |
| `forgeyard-deploy` | AWS S3, OCI, SSH, & GitHub Release publishers |
| `forgeyard-detector` | Parallel workspace scanner (Rayon) |
| `forgeyard-device-lab` | Android ADB device lab manager |
| `forgeyard-events` | Stoolap event bus |
| `forgeyard-executor` | Execution drivers (Process, Container, Apple, Android, Windows) |
| `forgeyard-logs` | `io_uring` append-only logging system |
| `forgeyard-model` | Core IR, Job, Pipeline, and Run definitions |
| `forgeyard-packaging` | Native package generators (.deb, .apk, .msi, .app) |
| `forgeyard-pipeline` | Matrix expansion, DAG resolver, & AST fingerprinter |
| `forgeyard-policy` | Security policy engine & SOC2 compliance ledger |
| `forgeyard-protocol` | QUIC wire protocol schemas |
| `forgeyard-provenance` | SLSA provenance & STARK ZK proof generator |
| `forgeyard-runner` | Local execution worker process |
| `forgeyard-sandbox` | Sandbox wrappers & Confidential Enclave executor |
| `forgeyard-scheduler` | Task queue, GPU profiler, & multi-region failover |
| `forgeyard-secrets` | Encrypted vault secret broker |
| `forgeyard-signing` | Hybrid Ed25519 + ML-DSA-87 signer |
| `forgeyard-storage` | Stoolap OLAP/OLTP database storage interface |
| `forgeyard-test-report` | Cargo test JSON parser & flaky test synthesizer |
| `forgeyard-toolchains` | Hermetic toolchain resolver (Rust, Node, Go, Java, NDK) |
| `forgeyard-ui` | Dioxus Web dashboard UI |
| `forgeyard-adapter-*` | Ecosystem adapters (Cargo, Dioxus, OCI, WASM, Xcode, Android) |

---

## 🔨 Building & Verification Instructions

```bash
# Clone the repository
git clone https://github.com/irshadali5/forgeyard.git
cd forgeyard

# Build the release binaries across all 37 crates
cargo build --workspace --release

# Run the workspace unit test suite
cargo test --workspace

# Run strict clippy verification
cargo clippy --workspace --all-targets -- -D warnings
```

---

## ⚖️ Dual License

Forgeyard is dual-licensed under AGPLv3 (Open Source) and Commercial License (Enterprise). See [`README.md`](file:///home/irshad/Projects/forgeyard/README.md) for contribution terms and licensing details.

*Designed for speed. Engineered for reliability. Built for the future of AI & software engineering.*
