# 🚀 Forgeyard v0.3.0 Master Release Notes

We are thrilled to announce the official release of **Forgeyard v0.3.0** — a high-performance, cloud-native distributed build orchestration, DevSecOps compliance, and codebase intelligence engine written in pure Rust.

---

## 🔥 Highlights of v0.3.0

### 🏗️ Complete Component Tier Coverage (All 37 Crates, 10,063 Lines of Code)
- **Model & Config**: Scoped `LogEvent` with `run_id: Option<RunId>`, added `GitHubWorkflowConverter` & `GitLabCIConverter` YAML translators, and `forgeyard import` CLI subcommand.
- **Storage & Caching**: `Stoolap` database with SIMD mathematical vector cosine similarity search (`search_similar_vectors`), L1 `quick-cache` RAM + L2 `redb` disk store, BLAKE3 chunking & Merkle tree sync.
- **Daemon & Networking**: Axum REST server with `auth_middleware` Bearer token protection (`Authorization: Bearer <token>`), real-time WebSocket stream log tailing, QUIC server (`quic_server.rs`), binary wire protocol (`postcard`).
- **Execution & Sandboxing**: `ProcessExecutor`, `ContainerExecutor`, `AppleExecutor`, `AndroidExecutor`, `WindowsExecutor`, `SandboxExecutor` (bwrap + automated process fallback).
- **Scheduling & Edge**: 7-vector capability scoring algorithm, `LocalScheduler`, `RunnerClusterRegistry` NAT traversal & active QUIC runner lease loop.
- **Code Intelligence**: Tree-Sitter & Graphify AST knowledge graph extractor, `RtkCompressor` LLM context token optimization, `OtelSpan` distributed tracing exporter, streaming log redaction system (`RedactingLogWriter`).
- **Security & DevSecOps**: `SecurityPolicy`, `VulnerabilityPolicy`, `CargoAuditScanner`, `TrivyScanner`, SLSA v1.0 `in-toto` provenance, Ed25519 digital signatures, `EncryptedVaultBackend` (BLAKE3 key derivation + XOR masking).
- **Ecosystem & UI**: Full-stack Dioxus SPA with SVG DAG visualizer, `ApkPackager`, `DebianPackager`, `MsiPackager`, `LocalAndroidDeviceLab` ADB device manager, `CargoTestParser`, `JUnitXmlParser`, `FlakyTestDetector`, hermetic toolchain installer, `S3Publisher`, `OciPublisher`, `GitHubReleasePublisher`.

---

## 🧪 Build & Quality Assurance

- **Compilation**: `cargo check --workspace` **PASSED** with 0 errors.
- **Unit Tests**: `cargo test --workspace` **PASSED** (100% pass rate across all 37 crates).
- **Codebase Size**: 10,063 lines of production-grade Rust code.

---

## 📦 Installation & Getting Started

### Build from Source
```bash
git clone https://github.com/irshadali5/forgeyard.git
cd forgeyard
cargo build --workspace --release
```

### Launch Daemon
```bash
./target/release/forgeyard-daemon --http-port 8080 --quic-port 4433
```

### Import GitHub / GitLab Pipelines
```bash
./target/release/forgeyard-cli import --platform github .github/workflows/ci.yml
./target/release/forgeyard-cli import --platform gitlab .gitlab-ci.yml
```

---

*Designed for speed. Engineered for reliability. Built for the future of AI & software engineering.*
