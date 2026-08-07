# 🚀 Forgeyard v0.2.0 Release Notes

We are thrilled to announce the official release of **Forgeyard v0.2.0** — a high-performance, pure-Rust distributed build orchestration and codebase intelligence engine built for high-concurrency CI/CD pipelines, edge runner clusters, and AI agentic workflows.

---

## 🔥 What's New in v0.2.0

### 🧠 RTK Token Compression & Hybrid CodeGraph Engine
- **Token Efficiency**: Introduced `RtkCompressor` in `forgeyard-analyzer` to prune internal function bodies and extract public API signatures, achieving 60-80% token reduction for LLM agent context windows.
- **CodeGraph Taxonomy**: Implemented `CodeGraph`, `SymbolInfo`, `SymbolKind`, and `CallEdge` in `forgeyard-model` to represent cross-file symbol definitions and call dependency graphs.

### 📊 OpenTelemetry Distributed Observability
- **Trace Exporting**: Added `OtelSpan`, `SpanKind`, and `TelemetryExporter` in `forgeyard-events` to record distributed trace spans (`Server`, `Client`, `Internal`) with parent-child propagation.
- **Daemon Instrumentation**: Integrated tracing spans into `forgeyard-daemon` for job scheduling, runner leasing, execution timing, and CAS transfer latency.

### 🛡️ DevSecOps Vulnerability & Compliance Scanning
- **Vulnerability Policy Gate**: Added `VulnerabilityPolicy`, `SeverityLevel`, and `VulnerabilityReport` in `forgeyard-policy` to parse `trivy` and `cargo-audit` reports.
- **Build Enforcement**: Automatically generates `PolicyFindingStatus::Fail` for Critical/High CVEs or unpatched vulnerabilities.

### 🌐 Multi-Cloud Remote Worker Cluster & NAT Traversal
- **Edge Worker Management**: Added `RunnerClusterRegistry` in `forgeyard-scheduler` for tracking active worker descriptors, assigned jobs, and UNIX epoch heartbeats.
- **Heartbeat Timeout Eviction**: Automatically evicts stale edge runners that miss heartbeat windows over QUIC tunnels.

### 📜 SLSA v1.0 Provenance Attestations & Cryptographic Signing
- Standard `in-toto` SLSA v1.0 predicate generation with Ed25519 cryptographic signing (`ed25519-dalek`).

### ✨ Pure Rust & Zero Compiler Warnings
- Cleaned up all compiler warnings across all 32 workspace crates (`cargo check --workspace` with 0 warnings).
- 100% unit test suite pass rate (`cargo test --workspace`).

---

## 📦 Installation & Getting Started

### Building from Source
```bash
git clone https://github.com/irshadali5/forgeyard.git
cd forgeyard
cargo build --workspace --release
```

### Launch Daemon
```bash
./target/release/forgeyard-daemon --http-port 8080 --quic-port 4433
```

---

*Designed for speed. Engineered for reliability. Built for the future of AI & software engineering.*
