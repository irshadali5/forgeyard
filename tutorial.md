# 🛠️ Forgeyard Complete Local Testing & Operations Guide

Welcome to the **Forgeyard Local Testing Guide**. This comprehensive tutorial walks you step-by-step through building, configuring, importing, orchestrating, and executing build pipelines using **Forgeyard** on your local machine.

---

## 📋 Table of Contents
1. [Prerequisites & System Requirements](#-prerequisites--system-requirements)
2. [Step 1: Building Workspace Binaries](#-step-1-building-workspace-binaries)
3. [Step 2: Initializing and Inspecting a Project](#-step-2-initializing-and-inspecting-a-project)
4. [Step 3: Importing CI/CD Pipelines (GitHub & GitLab)](#-step-3-importing-cicd-pipelines-github--gitlab)
5. [Step 4: Launching the Orchestration Daemon](#-step-4-launching-the-orchestration-daemon)
6. [Step 5: Connecting Edge Build Agents](#-step-5-connecting-edge-build-agents)
7. [Step 6: Executing Pipelines & Tracking Build Status](#-step-6-executing-pipelines--tracking-build-status)
8. [Step 7: Real-Time Log Tailing & Secret Redaction](#-step-7-real-time-log-tailing--secret-redaction)
9. [Step 8: Verifying Linux Kernel io_uring Acceleration](#-step-8-verifying-linux-kernel-io_uring-acceleration)
10. [Step 9: DevSecOps Vulnerability & Policy Gate Testing](#-step-9-devsecops-vulnerability--policy-gate-testing)
11. [Step 10: Package Generation & Release Artifact Publishing](#-step-10-package-generation--release-artifact-publishing)

---

## ⚙️ Prerequisites & System Requirements

Before testing Forgeyard locally, ensure your environment meets the following prerequisites:

- **Operating System**: Linux (Kernel 5.1+ recommended for `io_uring` zero-copy acceleration), macOS, or Windows.
- **Rust Toolchain**: `rustc` 1.85+ and `cargo` installed.
- **Optional Tools**: `docker` / `podman` (for container execution), `adb` (for Android device lab testing).

Verify your Rust toolchain version:
```bash
rustc --version
cargo --version
```

---

## 🔨 Step 1: Building Workspace Binaries

Forgeyard is structured as a modular 37-crate Rust workspace. Build the entire workspace and generate the release binaries:

```bash
cargo build --workspace --release
```

After compilation finishes, three core binaries will be produced in `./target/release/`:

- **`forgeyard-cli`**: The primary developer command-line interface.
- **`forgeyard-daemon`**: The central orchestration daemon, REST/WebSocket server, and QUIC edge worker hub.
- **`forgeyard-agent`**: The distributed edge worker agent process.

Verify that the CLI binary operates properly:
```bash
./target/release/forgeyard-cli --version
./target/release/forgeyard-cli --help
```

---

## 📂 Step 2: Initializing and Inspecting a Project

### Initialize a New Project Configuration
To create a default pipeline configuration (`forgeyard.ron`) in your project directory:

```bash
./target/release/forgeyard-cli init
```

### Inspect Project Toolchains & Workspaces
Run parallel workspace project scanner (`forgeyard-detector`) to discover active project types (Cargo, npm, Go, Gradle, Dockerfile):

```bash
./target/release/forgeyard-cli inspect
```

---

## 🔄 Step 3: Importing CI/CD Pipelines (GitHub & GitLab)

Forgeyard includes native AST converters (`GitHubWorkflowConverter` and `GitLabCIConverter`) to translate existing CI/CD YAML configurations into native `forgeyard.ron` Intermediate Representation (IR).

### Option A: Import a GitHub Actions Workflow
```bash
./target/release/forgeyard-cli import --platform github .github/workflows/ci.yml
```

### Option B: Import a GitLab CI Pipeline
```bash
./target/release/forgeyard-cli import --platform gitlab .gitlab-ci.yml
```

### Inspect the Generated `forgeyard.ron`
Open the generated `forgeyard.ron` to review translated pipeline triggers, DAG stages, dependency graphs, and job execution commands:

```ron
(
    version: 1,
    project: (
        name: "imported-project",
    ),
    pipelines: {
        "default": (
            triggers: [ GitCommit ],
            stages: [ "build", "test" ],
            jobs: {
                "build": (
                    needs: [],
                    command: [ "cargo check --workspace" ],
                    matrix: None,
                ),
                "test": (
                    needs: [ "build" ],
                    command: [ "cargo test --workspace" ],
                    matrix: None,
                ),
            },
        ),
    },
)
```

---

## 🌐 Step 4: Launching the Orchestration Daemon

The central daemon manages job scheduling, edge worker cluster leases, vector similarity indexing (`Stoolap`), and REST/WebSocket API endpoints.

Start the daemon in a dedicated terminal window:

```bash
./target/release/forgeyard-daemon --http-port 8080 --quic-port 4433
```

- **HTTP REST & WebSocket API**: `http://localhost:8080`
- **QUIC Edge Tunnel Port**: `4433`

### REST API Authentication
All `/api/v1/*` endpoints are protected by Bearer token authentication middleware (`auth_middleware`). Pass the authentication header when sending HTTP requests:

```bash
curl -H "Authorization: Bearer forgeyard-default-secret-token" http://localhost:8080/api/v1/status
```

---

## 🤖 Step 5: Connecting Edge Build Agents

Forgeyard edge build agents connect to the central daemon via persistent QUIC tunnels (`quic_server.rs`). Agents register their system capabilities (CPU, RAM, GPU, NDK/JDK/Rust toolchains) using a 7-vector capability scoring algorithm (`forgeyard-scheduler`).

In a separate terminal, launch a local build agent:

```bash
./target/release/forgeyard-agent --daemon-url http://localhost:8080
```

The daemon automatically registers the agent in `RunnerClusterRegistry` and monitors heartbeats to ensure high-availability worker allocation.

---

## ▶️ Step 6: Executing Pipelines & Tracking Build Status

### View Pipeline DAG Plan
Preview the topologically sorted Execution DAG before triggering a run:

```bash
./target/release/forgeyard-cli plan
```

### Execute the Pipeline
Trigger pipeline execution:

```bash
./target/release/forgeyard-cli run
```

### Stream Pipeline Execution with Watch Mode
To trigger execution and actively tail status updates until completion:

```bash
./target/release/forgeyard-cli run --watch
```

### Check Active Run Status
Query pipeline execution metrics and job completion states:

```bash
./target/release/forgeyard-cli status
```

---

## 📜 Step 7: Real-Time Log Tailing & Secret Redaction

### Tail Logs via CLI
Retrieve streamed execution logs for a specific run ID:

```bash
./target/release/forgeyard-cli logs --run-id <run_id>
```

### WebSocket Live Log Streaming
Connect directly to the daemon's WebSocket stream to receive live log lines as jobs execute:

```
ws://localhost:8080/api/v1/logs/stream/<run_id>
```

### Automatic Secret Masking (`RedactingLogWriter`)
Forgeyard automatically filters sensitive credentials, tokens, and vault secrets in memory before writing to disk or streaming over WebSockets:

- Original line: `Connecting to AWS with secret AKIAIOSFODNN7EXAMPLE`
- Redacted line: `Connecting to AWS with secret [REDACTED_SECRET]`

---

## ⚡ Step 8: Verifying Linux Kernel io_uring Acceleration

Forgeyard leverages Linux kernel `io_uring` Submission Queue (SQ) and Completion Queue (CQ) ring buffers to bypass traditional POSIX context switching latency.

### Implemented Zero-Copy Drivers
1. **`IoUringCasEngine`**: High-concurrency Content-Addressable Storage blob reader.
2. **`IoUringLogWriter`**: Direct kernel submission ring file appender for log ingestion.
3. **`compute_blake3`**: Asynchronous submission ring file chunk reader for hash calculations.
4. **`persist_vault_io_uring`**: Direct kernel ring write driver for encrypted secret vault persistence.
5. **`parse_file_io_uring`**: Kernel ring file reader for JUnit XML and Trivy JSON report parsing.

### Runtime Capability Check & Automated Fallback
Check kernel support programmatically:

```rust
if IoUringCasEngine::is_io_uring_supported() {
    println!("⚡ Linux io_uring zero-copy kernel acceleration ACTIVE!");
} else {
    println!("🛡️ Automated Tokio async file I/O fallback ACTIVE!");
}
```

---

## 🛡️ Step 9: DevSecOps Vulnerability & Policy Gate Testing

Forgeyard evaluates `cargo-audit` dependency advisories and `trivy` container scan reports against security policy gates (`VulnerabilityPolicy`).

### Local Policy Scanner Verification
Create a test security scan check:

```rust
use forgeyard_policy::{VulnerabilityPolicy, TrivyScanner, PolicyFindingStatus};

let report = TrivyScanner::parse_file_io_uring("app:latest", std::path::Path::new("trivy_report.json"));
let policy = VulnerabilityPolicy::default();
let status = policy.evaluate(&report);

assert_ne!(status, PolicyFindingStatus::Fail);
```

If Critical or High severity CVEs are present, Forgeyard automatically halts pipeline progression and logs policy violations.

---

## 📦 Step 10: Package Generation & Release Artifact Publishing

Forgeyard builds native distribution bundles and publishes release artifacts across cloud providers.

### 1. Generate Native Distribution Packages (`forgeyard-packaging`)
- **Debian Linux (`.deb`)**: `DebianPackager::package(...)`
- **Android APK (`.apk`)**: `ApkPackager::package(...)`
- **Windows Installer (`.msi`)**: `MsiPackager::package(...)`
- **Tarball / Zip Archives**: `TarPackager::package(...)` / `ZipPackager::package(...)`

### 2. Publish Release Artifacts (`forgeyard-deploy`)
Publish target release binaries to distribution endpoints:

- **AWS S3**: `S3Publisher` (`s3://<bucket>/<prefix>/<channel>`)
- **OCI / Docker Registry**: `OciPublisher` (`oci://<registry>/<image>:<tag>`)
- **GitHub Release API**: `GitHubReleasePublisher` (Uploads assets directly to GitHub Release tags)
- **SSH Deployment**: `SshPublisher` (SCP transfer + remote service restart)

---

## ✅ Verification Checklist

Verify that your local setup is fully functional:

- [x] All 37 workspace crates build cleanly (`cargo check --workspace`).
- [x] All unit test suites pass (`cargo test --workspace`).
- [x] `forgeyard-cli import` converts GitHub/GitLab YAML into `forgeyard.ron`.
- [x] `forgeyard-daemon` accepts REST requests and streams WebSockets.
- [x] `forgeyard-agent` registers capabilities via QUIC tunnels.
- [x] `io_uring` ring drivers execute with automatic Tokio fallback.

---

*Designed for speed. Engineered for reliability. Built for the future of AI & software engineering.*
