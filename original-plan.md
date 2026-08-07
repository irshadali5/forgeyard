# Forgeyard

## Local-First, Cross-Platform CI/CD Platform in Pure Rust

## 1. Vision

Forgeyard is a self-hosted CI/CD platform designed to run primarily on a developer workstation, build server, homelab, or private LAN.

The user provides:

* a local source directory;
* a Git repository;
* a source archive;
* or a repository URL.

Forgeyard then:

1. inspects the codebase;
2. identifies languages, frameworks, packages, workspaces, build tools, tests, and intended platforms;
3. generates or loads a build matrix;
4. prepares reproducible build environments;
5. runs validation and tests;
6. compiles for supported target platforms;
7. packages distributable artifacts;
8. signs artifacts where credentials are available;
9. generates checksums, manifests, provenance, and reports;
10. optionally deploys or publishes the resulting artifacts.

The platform itself is implemented in Rust. External compilers, SDKs and platform tools are invoked as isolated tools rather than reimplemented.

---

# 2. Critical Platform Reality

A universal cross-platform compiler cannot reliably build and test every target from one operating system.

Forgeyard must distinguish among:

* cross-compilable targets;
* cross-linkable targets;
* emulatable targets;
* natively testable targets;
* platform-restricted targets.

Rust categorises compilation targets into support tiers. A target being accepted by `rustc` does not necessarily mean that host tools, runtime execution, standard-library builds, or automated testing are fully supported. Forgeyard must therefore maintain explicit capabilities for every host-target combination rather than assuming that installation of a Rust target is sufficient.

## 2.1 Apple restriction

Production-quality macOS and iOS builds require an Apple build environment containing the appropriate Xcode and Apple SDK versions. Xcode supplies the compilers, SDKs, simulators, archive tooling, signing infrastructure, and distribution tooling for Apple platforms.

Therefore:

* Linux cannot be treated as a complete macOS/iOS release builder.
* A Mac runner is required for signed macOS applications.
* A Mac runner is required for iOS simulator tests, device tests, archives, signing, notarisation, and App Store packaging.
* Cross-compilation may validate portions of portable Rust code, but it does not replace the native Apple release pipeline.

## 2.2 Windows restriction

Linux can compile many Windows GNU targets with MinGW, but this is not equivalent to a native MSVC build.

A complete Windows matrix should normally include:

* `x86_64-pc-windows-msvc`;
* `aarch64-pc-windows-msvc`;
* optionally `x86_64-pc-windows-gnu`.

Native Windows runners are needed for:

* MSVC-specific builds;
* Windows UI integration tests;
* MSI or MSIX packaging;
* code signing;
* Windows service installation tests;
* registry-related tests;
* DirectX and Windows API behaviour tests.

## 2.3 Android restriction

Rust native libraries can be built for Android ABIs using the Android NDK. Android distributes separate ABI targets, so Forgeyard must build and package each requested architecture explicitly.

Typical Android targets are:

* `aarch64-linux-android`;
* `armv7-linux-androideabi`;
* `x86_64-linux-android`;
* `i686-linux-android`, where still required.

## 2.4 Practical architecture consequence

Forgeyard is local-first but not necessarily single-machine.

It consists of:

* one local controller;
* one or more local or LAN build runners;
* optional native Windows and macOS runners;
* optional Android devices;
* optional iPhones or iPads connected to a Mac runner.

A single Linux workstation can still handle:

* source analysis;
* web builds;
* Linux builds;
* Linux tests;
* server builds;
* many Windows GNU cross-builds;
* Android native compilation;
* static analysis;
* dependency auditing;
* package generation;
* orchestration of native Windows and Apple runners.

---

# 3. Design Principles

## 3.1 Local-first

All repositories, logs, caches, secrets, reports, and artifacts remain local by default.

Remote operation is optional.

## 3.2 Reproducible

Every execution records:

* source revision;
* source-tree digest;
* pipeline definition digest;
* toolchain versions;
* build environment;
* environment-variable allowlist;
* dependency lockfiles;
* build commands;
* artifact hashes;
* runner identity;
* timestamps;
* signing identity metadata.

## 3.3 Explicit over magical

Automatic detection proposes a plan, but the durable source of truth is a version-controlled pipeline configuration.

Automatic detection must never silently invent deployment or signing behaviour.

## 3.4 Capability-driven

Jobs are scheduled according to capabilities, not merely operating-system names.

For example:

```text
os = macos
arch = aarch64
xcode = 26
ios_sdk = installed
codesign = enabled
notarisation = enabled
hardware_virtualisation = true
devices = ["iphone"]
```

## 3.5 Hermetic where possible

Build steps receive only:

* declared source inputs;
* declared toolchains;
* declared environment variables;
* declared secrets;
* declared network access;
* declared writable directories.

## 3.6 Incremental

Forgeyard avoids repeating work through:

* content-addressed source snapshots;
* dependency caches;
* compiler caches;
* build graph fingerprints;
* artifact reuse;
* test-result reuse where safe;
* pipeline-level deduplication.

## 3.7 Secure by default

Untrusted source code is treated as hostile.

Build scripts can execute arbitrary code, so repository inspection and job execution must not run with unrestricted access to the host.

---

# 4. Non-Goals

Forgeyard should not initially attempt to:

* replace Cargo, Clang, Gradle, Xcode or platform SDKs;
* emulate all operating systems;
* provide a public multi-tenant SaaS;
* dynamically translate arbitrary applications between frameworks;
* guarantee that compilation means runtime correctness;
* infer signing or production deployment without explicit configuration;
* support every programming language in the first release.

The initial product should deeply support Rust projects while providing a generic process runner for other ecosystems.

---

# 5. High-Level Architecture

```text
                         ┌────────────────────────┐
                         │ CLI / TUI / Local Web  │
                         └───────────┬────────────┘
                                     │
                         Local IPC / HTTP / QUIC
                                     │
                   ┌─────────────────▼─────────────────┐
                   │           Forgeyard Daemon        │
                   │                                   │
                   │ Repository Intake                 │
                   │ Project Detection                 │
                   │ Pipeline Compiler                 │
                   │ Scheduler                         │
                   │ State Machine                     │
                   │ Cache Coordinator                 │
                   │ Artifact Manager                  │
                   │ Secret Broker                     │
                   │ Report Generator                  │
                   └──────┬───────────────┬────────────┘
                          │               │
                     Job leases       Metadata/events
                          │               │
             ┌────────────▼───────┐   ┌───▼────────────────┐
             │ Runner Coordinator │   │ Local State Store   │
             └──────┬───────┬─────┘   │ Metadata + Journal  │
                    │       │         └────────────────────┘
          ┌─────────▼─┐ ┌───▼─────────┐
          │Linux Runner│ │Windows Runner│
          └─────────┬─┘ └───┬─────────┘
                    │       │
              ┌─────▼───────▼─────┐
              │    macOS Runner    │
              └─────────┬──────────┘
                        │
          ┌─────────────▼──────────────────┐
          │ Execution Backends             │
          │ Process / Sandbox / Container  │
          │ VM / Emulator / Physical Device│
          └─────────────┬──────────────────┘
                        │
       ┌────────────────▼───────────────────┐
       │ CAS / Cache / Logs / Artifacts     │
       └────────────────────────────────────┘
```

---

# 6. Main Components

## 6.1 `forgeyard` CLI

The CLI is the primary interface.

Example commands:

```bash
forgeyard init
forgeyard inspect .
forgeyard plan .
forgeyard run
forgeyard run test
forgeyard run release
forgeyard build --target linux-x86_64
forgeyard matrix
forgeyard status
forgeyard logs <run-id>
forgeyard artifact list <run-id>
forgeyard runner list
forgeyard runner doctor
forgeyard cache gc
forgeyard verify <artifact>
```

Responsibilities:

* communicate with the daemon;
* render progress;
* stream structured logs;
* display the dependency graph;
* inspect generated plans;
* manage runners;
* retrieve artifacts;
* verify checksums and provenance.

Recommended crates:

* `clap` for commands;
* `anstream` and `anstyle` for terminal output;
* `indicatif` for simple progress;
* `ratatui` for an optional TUI;
* `miette` for diagnostics.

The CLI should not contain orchestration logic. It is a client of the daemon and shares only protocol and model crates.

---

## 6.2 Local web interface

A Dioxus desktop/web interface can be added after the CLI is stable.

Views:

* repositories;
* pipeline runs;
* build graph;
* live job logs;
* test results;
* target matrix;
* artifacts;
* runners;
* cache usage;
* secrets metadata;
* settings;
* deployment history.

The UI must consume the same typed API as the CLI.

Do not place scheduling, build detection, or pipeline semantics inside UI code.

---

## 6.3 Forgeyard daemon

The daemon is the authoritative control plane.

Responsibilities:

* repository registration;
* run creation;
* pipeline compilation;
* job-state transitions;
* runner discovery;
* scheduling;
* lease management;
* cancellation;
* retries;
* event recording;
* cache coordination;
* artifact indexing;
* secret delivery;
* log ingestion.

Suggested runtime:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    forgeyard_daemon::bootstrap().await
}
```

The daemon should run as:

* a foreground process during development;
* a user-level system service;
* or a system service on a dedicated build host.

---

# 7. Repository Intake

Forgeyard supports four source modes:

```rust
pub enum SourceInput {
    WorkingDirectory(PathBuf),
    GitRepository {
        url: String,
        revision: Option<String>,
    },
    Archive(PathBuf),
    Snapshot {
        digest: Digest,
    },
}
```

## 7.1 Intake pipeline

```text
Input
  ↓
Validate source
  ↓
Create immutable source snapshot
  ↓
Apply ignore rules
  ↓
Calculate Merkle tree
  ↓
Store snapshot in CAS
  ↓
Run static project inspection
```

## 7.2 Ignore rules

Respect:

* `.gitignore`;
* `.forgeyardignore`;
* generated build directories;
* editor caches;
* optional globally ignored paths.

Do not automatically exclude lockfiles.

Files such as these are critical inputs:

* `Cargo.lock`;
* `package-lock.json`;
* `pnpm-lock.yaml`;
* `yarn.lock`;
* `gradle.lockfile`;
* `Podfile.lock`;
* `Package.resolved`.

## 7.3 Safe inspection

Initial inspection must be non-executing.

It may parse:

* filenames;
* manifests;
* lockfiles;
* workspace metadata;
* build configuration;
* source imports;
* target-specific modules.

It must not run:

* `build.rs`;
* package lifecycle scripts;
* arbitrary shell scripts;
* compiler plugins;
* project-provided executables.

---

# 8. Project Detection Engine

The detector produces evidence, not final truth.

```rust
pub struct DetectionReport {
    pub ecosystems: Vec<EcosystemDetection>,
    pub applications: Vec<ApplicationDetection>,
    pub platforms: Vec<PlatformIntent>,
    pub test_suites: Vec<TestSuiteDetection>,
    pub package_managers: Vec<PackageManagerDetection>,
    pub confidence: DetectionConfidence,
    pub warnings: Vec<DetectionWarning>,
}
```

## 8.1 Rust detection

Inspect:

* `Cargo.toml`;
* workspace members;
* crate types;
* target-specific dependencies;
* features;
* binaries;
* libraries;
* examples;
* benches;
* tests;
* `build.rs`;
* `.cargo/config.toml`;
* `rust-toolchain.toml`;
* Dioxus configuration;
* Tauri configuration;
* Android Gradle projects;
* Xcode projects.

## 8.2 Platform inference

Examples:

| Evidence                        | Likely target                             |
| ------------------------------- | ----------------------------------------- |
| Axum, Actix Web, Poem, Rocket   | Server                                    |
| Dioxus web feature              | Web/WASM                                  |
| Dioxus desktop                  | Linux, Windows, macOS                     |
| Dioxus mobile files             | Android, iOS                              |
| Tauri configuration             | Desktop/mobile depending on configuration |
| `cdylib` plus JNI glue          | Android                                   |
| Xcode project                   | macOS/iOS                                 |
| `wasm-bindgen`                  | Web/WASM                                  |
| systemd unit                    | Linux server                              |
| Dockerfile                      | Container/server                          |
| Windows manifest                | Windows desktop                           |
| `.desktop` file/AppImage config | Linux desktop                             |

## 8.3 Confidence model

```rust
pub enum Confidence {
    Certain,
    Strong,
    Probable,
    Weak,
}
```

A weak inference must be reported but not automatically added to release builds.

Example:

```text
Detected:
  Rust workspace                    certain
  Axum server                       strong
  Dioxus desktop                    strong
  Dioxus web                        strong
  Android application               probable
  iOS application                   weak

Suggested matrix generated with iOS disabled pending confirmation.
```

---

# 9. Configuration Format

Use RON as the human-authored configuration format.

File:

```text
forgeyard.ron
```

Use Postcard for internal binary messages, persisted event records, job envelopes, and runner communication where schema compatibility is controlled.

## 9.1 Example configuration

```ron
(
    version: 1,

    project: (
        name: "aequos",
        source: WorkingTree,
    ),

    toolchains: {
        "rust-stable": (
            kind: Rust,
            channel: "stable",
            profile: "minimal",
            components: ["rustfmt", "clippy"],
        ),
    },

    pipelines: {
        "validate": (
            triggers: [Manual, SourceChanged],
            stages: [
                "inspect",
                "format",
                "lint",
                "unit-test",
            ],
        ),

        "release": (
            triggers: [Manual, Tag(pattern: "v*")],
            stages: [
                "validate",
                "build-web",
                "build-server",
                "build-desktop",
                "build-android",
                "build-apple",
                "package",
                "sign",
                "provenance",
            ],
        ),
    },

    targets: {
        "web": (
            platform: Web,
            runner: (os: Any),
            commands: [
                Cargo(args: ["build", "--release", "--target", "wasm32-unknown-unknown"]),
            ],
            outputs: ["target/wasm32-unknown-unknown/release/*.wasm"],
        ),

        "linux-x86_64": (
            platform: Linux,
            arch: X86_64,
            runner: (os: Linux),
            rust_target: "x86_64-unknown-linux-gnu",
            package: TarZst,
        ),

        "windows-x86_64": (
            platform: Windows,
            arch: X86_64,
            runner: (os: Windows),
            rust_target: "x86_64-pc-windows-msvc",
            package: Zip,
        ),

        "macos-universal": (
            platform: MacOS,
            arch: Universal,
            runner: (os: MacOS),
            members: [
                "aarch64-apple-darwin",
                "x86_64-apple-darwin",
            ],
            package: Dmg,
            signing: "apple-developer-id",
        ),

        "android": (
            platform: Android,
            runner: (capabilities: ["android-sdk", "android-ndk"]),
            abis: ["arm64-v8a", "x86_64"],
            package: Aab,
            signing: "android-release",
        ),

        "ios": (
            platform: IOS,
            runner: (os: MacOS, capabilities: ["xcode", "ios-sdk"]),
            destinations: [
                Simulator(name: "iPhone"),
                Archive,
            ],
            package: Ipa,
            signing: "apple-distribution",
        ),
    },

    policy: (
        network_default: Deny,
        untrusted_source: SandboxRequired,
        maximum_parallel_jobs: 8,
        maximum_run_time_seconds: 7200,
    ),
)
```

## 9.2 Configuration layers

Resolve configuration in this order:

```text
Built-in defaults
    ↓
System configuration
    ↓
User configuration
    ↓
Repository forgeyard.ron
    ↓
Local uncommitted override
    ↓
CLI overrides
```

A local override may hold machine-specific runner selectors, but never plaintext secrets.

---

# 10. Pipeline Intermediate Representation

Do not execute configuration directly.

Compile it into a validated pipeline IR.

```rust
pub struct PipelineIr {
    pub pipeline_id: PipelineId,
    pub stages: Vec<StageIr>,
    pub jobs: BTreeMap<JobId, JobIr>,
    pub edges: Vec<JobDependency>,
    pub outputs: Vec<DeclaredOutput>,
    pub policy: EffectivePolicy,
}
```

Each job must be immutable after pipeline compilation.

```rust
pub struct JobIr {
    pub id: JobId,
    pub name: String,
    pub dependencies: Vec<JobId>,
    pub runner_requirements: CapabilityExpression,
    pub execution: ExecutionSpec,
    pub inputs: Vec<InputSpec>,
    pub outputs: Vec<OutputSpec>,
    pub cache: CachePolicy,
    pub secrets: Vec<SecretReference>,
    pub retry: RetryPolicy,
    pub timeout: Duration,
}
```

## 10.1 Compilation phases

```text
Parse configuration
  ↓
Validate schema
  ↓
Resolve imports/templates
  ↓
Expand target matrices
  ↓
Resolve dependencies
  ↓
Check cycles
  ↓
Evaluate policies
  ↓
Calculate job fingerprints
  ↓
Emit immutable pipeline IR
```

---

# 11. Directed Acyclic Build Graph

A pipeline is represented as a DAG.

Example:

```text
snapshot
   │
   ├────────────┐
   ▼            ▼
metadata      secret-scan
   │            │
   ├──────┬─────┘
   ▼      ▼
format   lint
   │      │
   └──┬───┘
      ▼
 unit-test
      │
 ┌────┼───────────────┬──────────────┐
 ▼    ▼               ▼              ▼
web  server        desktop        mobile
                    │              │
             ┌──────┼─────┐   ┌────┴────┐
             ▼      ▼     ▼   ▼         ▼
           Linux Windows macOS Android  iOS
             └──────┴─────┴────┴─────────┘
                           │
                           ▼
                       provenance
```

Benefits:

* independent jobs run concurrently;
* failed dependencies block only dependent jobs;
* unchanged branches can reuse cached outputs;
* cancellation propagates correctly;
* the UI can show precise execution state.

---

# 12. Job State Machine

```rust
pub enum JobState {
    Created,
    Ready,
    WaitingForRunner,
    Leased,
    Preparing,
    Running,
    UploadingOutputs,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    Lost,
    Skipped,
    Cached,
}
```

Valid transitions must be enforced centrally.

Example:

```text
Created
  → Ready
  → WaitingForRunner
  → Leased
  → Preparing
  → Running
  → UploadingOutputs
  → Succeeded
```

A runner losing contact causes:

```text
Leased | Preparing | Running
  → Lost
  → Ready
```

This should happen only when the operation is safe to retry.

Signing and deployment jobs may require explicit idempotency keys.

---

# 13. Scheduler

The scheduler matches ready jobs with compatible runners.

## 13.1 Runner description

```rust
pub struct RunnerDescriptor {
    pub id: RunnerId,
    pub host: HostPlatform,
    pub capabilities: BTreeSet<Capability>,
    pub resources: ResourceCapacity,
    pub trust_level: TrustLevel,
    pub labels: BTreeMap<String, String>,
    pub health: RunnerHealth,
}
```

## 13.2 Capability examples

```text
os:linux
os:windows
os:macos
arch:x86_64
arch:aarch64
rust:stable
rust:nightly
docker
podman
kvm
wine
qemu-user
android-sdk
android-ndk
android-emulator
android-device
xcode
ios-simulator
ios-device
apple-codesign
apple-notary
windows-signing
gpu:nvidia
gpu:intel
```

## 13.3 Scheduling score

```text
score =
    exact_host_match
  + warm_toolchain_score
  + cache_locality_score
  + available_cpu_score
  + available_memory_score
  + trusted_runner_score
  - queue_penalty
  - estimated_transfer_cost
```

Hard requirements are evaluated before scoring.

## 13.4 Resource scheduling

Each job requests resources:

```rust
pub struct ResourceRequest {
    pub cpu_millis: u32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub gpu: Option<GpuRequest>,
    pub exclusive_devices: Vec<DeviceSelector>,
}
```

Avoid mapping one logical job directly to one operating-system thread.

Use:

* Tokio for orchestration and I/O;
* bounded worker pools for CPU-intensive hashing and compression;
* OS processes for compilers;
* semaphores for host-level resource control.

---

# 14. Runner Architecture

```text
Runner Agent
  ├── Registration client
  ├── Heartbeat service
  ├── Capability detector
  ├── Job lease client
  ├── Workspace manager
  ├── Execution backend manager
  ├── Toolchain manager
  ├── Log streamer
  ├── Artifact uploader
  ├── Local cache
  └── Device manager
```

## 14.1 Runner modes

### Local embedded runner

Runs on the same host as the daemon.

Best for:

* initial implementation;
* trusted repositories;
* personal development.

### Local isolated runner

Runs as a separate service account.

Best for:

* stronger filesystem isolation;
* persistent build machines.

### LAN runner

Connects through authenticated QUIC or mutual TLS.

Best for:

* Mac mini build node;
* Windows workstation;
* high-core Linux server;
* physical device lab.

### Ephemeral runner

Starts for one run, executes jobs, then destroys its workspace.

Best for:

* reproducible release builds;
* stronger isolation;
* clean-room validation.

---

# 15. Communication Protocol

Recommended transport:

* Unix domain sockets for same-host communication;
* named pipes on Windows;
* QUIC with mutual authentication for LAN runners;
* HTTP only for the local web API when convenient.

Use Postcard payloads over framed transport.

```rust
pub enum ControlMessage {
    RegisterRunner(RegisterRunner),
    Heartbeat(Heartbeat),
    RequestLease(RequestLease),
    AcceptLease(AcceptLease),
    RejectLease(RejectLease),
    JobEvent(JobEvent),
    LogBatch(LogBatch),
    ArtifactDeclared(ArtifactDeclared),
    JobCompleted(JobCompleted),
}
```

## 15.1 Protocol requirements

* version negotiation;
* maximum frame size;
* request IDs;
* idempotency keys;
* monotonic per-job sequence numbers;
* explicit cancellation;
* resumable artifact upload;
* heartbeat expiry;
* replay protection;
* authenticated runner identity.

Postcard is suitable for controlled Rust-to-Rust communication, but stored records need a migration/versioning strategy.

Every persisted envelope should include:

```rust
pub struct Envelope<T> {
    pub protocol_version: u16,
    pub schema_version: u16,
    pub message_id: MessageId,
    pub body: T,
}
```

---

# 16. Execution Backends

Forgeyard requires a common executor interface.

```rust
#[async_trait]
pub trait Executor: Send + Sync {
    async fn prepare(&self, context: &JobContext) -> Result<PreparedJob>;
    async fn execute(
        &self,
        job: PreparedJob,
        events: EventSink,
    ) -> Result<ExecutionResult>;
    async fn cancel(&self, execution: ExecutionHandle) -> Result<()>;
    async fn cleanup(&self, job: PreparedJob) -> Result<()>;
}
```

## 16.1 Native process executor

Runs commands directly under a restricted OS account.

Advantages:

* lowest overhead;
* best compiler performance;
* easiest access to platform SDKs;
* required for some Xcode and Windows tooling.

Disadvantages:

* weakest isolation;
* host toolchain drift;
* harder cleanup;
* potentially unsafe for untrusted code.

Use it for trusted repositories and platform-native release jobs.

## 16.2 Linux sandbox executor

Preferred Linux mechanisms:

* namespaces;
* user namespaces;
* mount namespaces;
* PID namespaces;
* network namespaces;
* cgroups v2;
* seccomp;
* capability dropping;
* read-only root filesystem;
* tmpfs workspace where appropriate.

A practical first release can integrate:

* Bubblewrap;
* systemd transient units;
* Podman;
* Docker.

The control plane remains Rust even when an external isolation runtime is used.

## 16.3 OCI container executor

Use OCI images for repeatable Linux and Android build environments.

BuildKit can reuse build layers and export caches, which makes it useful as an optional image-building backend rather than the central scheduler.

Forgeyard should not make Docker mandatory.

Support:

* Podman;
* Docker;
* native execution;
* future direct OCI runtime integration.

## 16.4 Virtual-machine executor

Use VMs when:

* strong isolation is required;
* OS-level tests must run;
* Windows builds require a Windows environment;
* kernel or installer behaviour must be tested;
* untrusted code must not share the host kernel.

Potential backends:

* QEMU/KVM on Linux;
* Hyper-V on Windows;
* Apple Virtualization.framework through a helper integration;
* libvirt as an optional provider.

## 16.5 Emulator executor

Used for:

* Android emulator tests;
* iOS simulator tests on macOS;
* QEMU user-mode tests;
* architecture smoke tests.

## 16.6 Physical-device executor

Used for:

* Android instrumentation tests;
* iOS device tests;
* hardware-specific tests;
* camera, sensor, GPU, Bluetooth, or notification testing.

Devices must be treated as exclusive resources during a job.

---

# 17. Toolchain Management

Toolchains are immutable, versioned installations.

```rust
pub struct ToolchainDescriptor {
    pub kind: ToolchainKind,
    pub version: VersionConstraint,
    pub digest: Digest,
    pub host: HostPlatform,
    pub installed_components: BTreeSet<String>,
}
```

## 17.1 Rust toolchain

Manage:

* stable/beta/nightly channels;
* pinned version;
* components;
* target standard libraries;
* Cargo version;
* linker configuration;
* `rust-src`;
* target-specific environment.

Relevant commands:

```bash
rustup toolchain install <version>
rustup component add rustfmt clippy
rustup target add <target>
cargo metadata --format-version 1
```

## 17.2 External SDKs

Forgeyard manages metadata and validation for:

* Android SDK;
* Android NDK;
* JDK;
* Gradle;
* Xcode;
* Apple SDKs;
* Visual Studio Build Tools;
* Windows SDK;
* LLVM/Clang;
* MinGW;
* WebAssembly tools;
* Node.js only when a project requires it.

It should not redistribute SDKs whose licences prohibit redistribution.

## 17.3 Toolchain lock

Generate:

```text
.forgeyard/toolchains.lock
```

Example:

```ron
(
    rust: (
        channel: "1.96.0",
        host: "x86_64-unknown-linux-gnu",
        components: ["rustfmt", "clippy"],
    ),
    android: (
        sdk: 36,
        ndk: "r29",
        build_tools: "36.0.0",
        jdk: "21",
    ),
    apple: (
        xcode: "26.6",
    ),
)
```

The values are examples; the actual lock should reflect installed and selected toolchains.

---

# 18. Build Adapter Architecture

Build systems are supported through adapters.

```rust
#[async_trait]
pub trait BuildAdapter {
    fn detect(&self, repository: &RepositoryView) -> DetectionResult;
    fn inspect(&self, repository: &RepositoryView) -> Result<ProjectModel>;
    fn generate_jobs(
        &self,
        project: &ProjectModel,
        request: &BuildRequest,
    ) -> Result<Vec<JobTemplate>>;
}
```

Initial adapters:

* Cargo;
* generic command;
* Dioxus;
* WebAssembly;
* Android Gradle;
* Xcode;
* container image;
* static site.

Later:

* CMake;
* Meson;
* npm/pnpm;
* Flutter;
* .NET;
* Go.

The platform remains pure Rust because adapters are Rust orchestration modules, even though they invoke ecosystem-native tools.

---

# 19. Rust Build Pipeline

A strong default Rust validation pipeline is:

```text
manifest validation
  ↓
cargo metadata
  ↓
format check
  ↓
cargo check
  ↓
clippy
  ↓
unit tests
  ↓
integration tests
  ↓
documentation tests
  ↓
feature matrix
  ↓
target matrix
  ↓
release build
```

Suggested commands:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --doc
cargo build --workspace --release
```

Do not always enable all features automatically. Mutually exclusive Cargo features are common.

Support feature strategies:

```rust
pub enum FeatureStrategy {
    Default,
    All,
    NoDefault,
    Explicit(Vec<String>),
    Powerset {
        maximum_combinations: usize,
    },
    EachFeatureSeparately,
}
```

---

# 20. Web Build Architecture

## 20.1 Rust/WASM path

Typical stages:

```text
cargo check
  ↓
wasm target build
  ↓
wasm-bindgen or framework bundling
  ↓
asset fingerprinting
  ↓
WASM optimisation
  ↓
static test server
  ↓
browser smoke tests
  ↓
static artifact bundle
```

Example target:

```text
wasm32-unknown-unknown
```

Possible output:

```text
dist/
├── index.html
├── app.js
├── app_bg.wasm
├── assets/
├── manifest.json
└── checksums.txt
```

## 20.2 Browser testing

Provide adapters for browser automation, but do not embed a complete browser engine.

A browser test job can use:

* Chromium;
* Firefox;
* WebKit, when available;
* project-provided Playwright tests;
* WebDriver.

The job should produce:

* screenshots on failure;
* browser console logs;
* network errors;
* trace archive;
* test result report.

---

# 21. Server Build Architecture

Server targets may produce:

* native binaries;
* static or mostly static binaries;
* OCI images;
* system packages;
* systemd service bundles;
* deployment archives.

## 21.1 Linux server targets

Common Rust targets:

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
```

Do not assume MUSL is compatible with every dependency.

OpenSSL, glibc assumptions, dynamic plugins, native database drivers, and C libraries require explicit validation.

## 21.2 Server tests

Layers:

1. unit tests;
2. integration tests;
3. database migration tests;
4. API contract tests;
5. service startup smoke test;
6. load test;
7. fault-injection test;
8. upgrade and rollback test.

Services such as PostgreSQL should be declared as job dependencies:

```ron
services: [
    (
        name: "postgres",
        image: "postgres:<pinned-version>",
        health_check: Tcp(port: 5432),
    ),
]
```

Never expose service credentials outside that job's isolated environment.

---

# 22. Desktop Build Architecture

## 22.1 Linux

Targets:

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
```

Packaging options:

* `.tar.zst`;
* AppImage;
* Flatpak;
* `.deb`;
* `.rpm`.

Recommended order:

1. portable archive;
2. AppImage or Flatpak;
3. native distribution packages.

Linux UI tests should run under:

* a nested compositor;
* virtual X server for X11 applications;
* headless Wayland where supported;
* VM for release-level verification.

## 22.2 Windows

Build:

* x86-64 MSVC;
* ARM64 MSVC where required;
* optional GNU compatibility target.

Package:

* `.zip`;
* MSI;
* MSIX;
* portable executable bundle.

Test:

* launch test;
* window creation;
* filesystem paths;
* Unicode paths;
* service installation;
* update behaviour;
* clean uninstall.

## 22.3 macOS

Build separately:

```text
aarch64-apple-darwin
x86_64-apple-darwin
```

Then optionally create a universal binary:

```text
lipo -create ...
```

Package:

* `.app`;
* `.dmg`;
* `.pkg`;
* `.zip`.

Release flow:

```text
build
  ↓
assemble .app
  ↓
sign nested components
  ↓
sign application
  ↓
verify signature
  ↓
package
  ↓
notarise
  ↓
staple
  ↓
final verification
```

---

# 23. Android Build Architecture

## 23.1 Rust native layer

Compile Rust for configured Android ABIs.

The Android NDK provides architecture-specific toolchains and APIs; each ABI must be compiled separately and then included in the Android package.

Example layout:

```text
app/src/main/jniLibs/
├── arm64-v8a/libapplication.so
├── armeabi-v7a/libapplication.so
├── x86_64/libapplication.so
└── x86/libapplication.so
```

## 23.2 Packaging pipeline

```text
Rust checks/tests
  ↓
Build native libraries per ABI
  ↓
Copy native outputs into Android project
  ↓
Gradle compile
  ↓
Android lint
  ↓
JVM/unit tests
  ↓
Emulator/device tests
  ↓
Build APK or AAB
  ↓
Sign
  ↓
Verify signature
```

## 23.3 Android test tiers

* host-side Rust unit tests;
* Android target compile check;
* emulator smoke tests;
* instrumentation tests;
* physical-device tests;
* multiple API levels;
* multiple architectures where practical.

## 23.4 Outputs

```text
application-debug.apk
application-release.apk
application-release.aab
mapping.txt
native-debug-symbols.zip
checksums.txt
provenance.json
```

---

# 24. iOS Build Architecture

iOS jobs must be assigned to macOS runners with Xcode.

## 24.1 Build variants

* simulator ARM64;
* simulator x86-64 where supported and needed;
* physical-device ARM64;
* archive;
* signed IPA.

## 24.2 Pipeline

```text
Rust portable tests
  ↓
Build Rust library for simulator
  ↓
Run simulator integration tests
  ↓
Build Rust library for device
  ↓
Create XCFramework if required
  ↓
Build Xcode workspace
  ↓
Run XCTest
  ↓
Archive
  ↓
Codesign
  ↓
Export IPA
  ↓
Verify archive
```

Xcode supports building and running against simulators and connected Apple devices, so Forgeyard should model simulators and devices as runner capabilities.

## 24.3 Signing modes

```rust
pub enum AppleSigningMode {
    Disabled,
    Development,
    AdHoc,
    DeveloperId,
    AppStore,
    Enterprise,
}
```

Signing must be separated from compilation.

Pull-request validation should usually perform unsigned or development builds.

Production signing should occur only in protected release pipelines.

---

# 25. Cross-Compilation Strategy

Use a layered strategy rather than one universal cross tool.

## Layer 1: Native Cargo cross-target compilation

Use when host linkers and SDKs are available.

```bash
cargo build --target <triple>
```

## Layer 2: Containerised cross compilation

Use target-specific Linux environments and linkers.

This is suitable for many:

* Linux targets;
* embedded targets;
* Windows GNU targets.

## Layer 3: Cross helper integration

Forgeyard may integrate with tools such as `cross` or `cargo-cross` as optional adapters rather than coupling its entire architecture to them. Current `cargo-cross` documentation advertises support across several desktop and mobile target families, but Forgeyard must still verify actual linker, SDK, runtime-test, and packaging capabilities per target.

## Layer 4: Native remote runner

Use for:

* MSVC Windows;
* macOS;
* iOS;
* signed installers;
* OS-specific tests.

## Layer 5: Emulator or physical device

Use when compilation alone is insufficient.

---

# 26. Test Model

Tests are first-class records, not merely console output.

```rust
pub struct TestReport {
    pub suite: TestSuiteId,
    pub target: TargetId,
    pub started_at: Timestamp,
    pub duration: Duration,
    pub cases: Vec<TestCaseResult>,
    pub coverage: Option<CoverageReport>,
    pub attachments: Vec<ArtifactRef>,
}
```

## 26.1 Test categories

```rust
pub enum TestKind {
    Format,
    StaticAnalysis,
    Unit,
    Integration,
    Documentation,
    UI,
    EndToEnd,
    Performance,
    Security,
    Compatibility,
    Installation,
    Upgrade,
    Smoke,
}
```

## 26.2 Test sharding

Large test suites may be split by:

* test binary;
* package;
* module;
* historical duration;
* deterministic hash partition.

Example:

```text
shard = hash(test_name) mod shard_count
```

Historical-duration balancing is more efficient:

```text
sort tests by estimated duration descending
assign each test to currently lightest shard
```

## 26.3 Flaky tests

Do not silently rerun until green.

Record:

* initial failure;
* retry count;
* retry outcomes;
* failure signature;
* flakiness history.

A retried success should be classified as:

```text
SucceededWithFlakyTests
```

---

# 27. Content-Addressed Storage

Use a content-addressed store for:

* source snapshots;
* dependency archives;
* intermediate outputs;
* final artifacts;
* logs;
* test attachments;
* provenance documents.

```text
storage/
├── blobs/
│   ├── ab/
│   │   └── abcdef...
├── manifests/
├── refs/
├── temp/
└── quarantine/
```

Digest:

```rust
pub struct Digest {
    pub algorithm: DigestAlgorithm,
    pub bytes: [u8; 32],
}
```

A suitable default is BLAKE3 for internal identity and SHA-256 for external compatibility manifests.

## 27.1 Blob properties

* immutable;
* verified after write;
* written through temporary files;
* atomically renamed;
* optionally compressed;
* reference-counted or traced for garbage collection.

## 27.2 Chunking

Large artifacts should support chunked storage and resumable transfer.

```rust
pub struct ChunkManifest {
    pub size: u64,
    pub chunks: Vec<ChunkRef>,
    pub root_digest: Digest,
}
```

---

# 28. Cache Architecture

Forgeyard needs separate cache classes.

## 28.1 Source cache

Immutable source snapshots keyed by repository-tree digest.

## 28.2 Dependency cache

Examples:

* Cargo registry index;
* crate archives;
* Git dependencies;
* Android Gradle dependencies;
* SDK downloads where redistribution permits;
* JavaScript package stores.

## 28.3 Compiler cache

Support `sccache` as an optional adapter.

The platform may later implement remote compilation cache protocols, but should not reinvent compiler caching in the first release.

## 28.4 Build output cache

Cache job outputs using an input fingerprint.

```text
job fingerprint =
    pipeline schema version
  + job definition
  + source input digests
  + declared dependency artifact digests
  + toolchain digest
  + runner platform ABI
  + relevant environment variables
  + secret version identifiers where applicable
```

## 28.5 Test cache

Use conservatively.

Tests may be cached only when:

* all inputs are declared;
* network is disabled or deterministically mocked;
* time and randomness are controlled;
* no external mutable system is accessed;
* test framework and binary hashes match.

## 28.6 Cache policy

```rust
pub enum CachePolicy {
    Disabled,
    ReadOnly,
    ReadWrite,
    PullRequestSafe,
    ReleaseIsolated,
}
```

Release pipelines should not automatically trust mutable caches generated by untrusted branches.

---

# 29. Persistent State

Use separate stores for metadata and blobs.

## 29.1 Metadata store

Recommended initial approach:

* SQLite through `rusqlite` for simplicity and reliability;
* or an embedded Rust-native transactional database after careful durability evaluation.

Metadata includes:

* repositories;
* source snapshots;
* runs;
* stages;
* jobs;
* events;
* runner registrations;
* job leases;
* artifact references;
* test summaries;
* cache entries;
* secret metadata;
* deployment records.

Do not store large logs or artifacts as database blobs.

## 29.2 Event journal

Every important state transition creates an append-only event.

```rust
pub enum DomainEvent {
    RunCreated,
    PipelineCompiled,
    JobReady,
    JobLeased,
    JobStarted,
    LogChunkStored,
    ArtifactUploaded,
    JobSucceeded,
    JobFailed,
    RunCompleted,
}
```

Use projections for efficient queries.

Benefits:

* crash recovery;
* auditability;
* debugging;
* reproducible state transitions;
* future distributed operation.

## 29.3 Transaction model

State transition and event insertion must be atomic.

Pseudo-flow:

```text
BEGIN
  verify current state
  update entity state
  insert event
  update projection
COMMIT
```

---

# 30. Logging

Logs should be structured internally.

```rust
pub struct LogEvent {
    pub run_id: RunId,
    pub job_id: JobId,
    pub sequence: u64,
    pub stream: LogStream,
    pub timestamp: Timestamp,
    pub level: Option<LogLevel>,
    pub message: Bytes,
    pub fields: BTreeMap<String, Value>,
}
```

Support:

* stdout;
* stderr;
* Forgeyard system logs;
* compiler diagnostics;
* test events;
* progress events.

## 30.1 Secret redaction

Redact:

* exact secret values;
* encoded variants where practical;
* common token patterns;
* explicitly registered sensitive strings.

Redaction is defence-in-depth, not a guarantee.

Secrets should preferably be delivered through:

* temporary files;
* inherited file descriptors;
* OS keychain handles;
* short-lived tokens.

Avoid environment variables for high-value secrets when tools support safer mechanisms.

---

# 31. Secret Management

## 31.1 Secret backends

```rust
pub trait SecretBackend {
    async fn get(&self, reference: &SecretReference)
        -> Result<SecretMaterial>;
}
```

Supported backends:

* Linux Secret Service;
* macOS Keychain;
* Windows Credential Manager;
* encrypted local vault;
* hardware-backed keys;
* external command provider.

## 31.2 Secret model

```rust
pub struct SecretReference {
    pub name: String,
    pub version: Option<String>,
    pub scope: SecretScope,
    pub delivery: SecretDelivery,
}
```

Never store secret values in:

* pipeline configuration;
* logs;
* event journal;
* cache keys;
* artifact metadata;
* provenance reports.

## 31.3 Pipeline protection

Production secrets should require:

* trusted source revision;
* protected branch or signed tag;
* trusted runner;
* approved pipeline;
* optional local user confirmation;
* no unreviewed pipeline changes.

---

# 32. Supply-Chain Security

Each release should produce:

* artifact checksum manifest;
* dependency inventory;
* SBOM;
* build provenance;
* toolchain inventory;
* source-tree digest;
* signature;
* vulnerability scan report where configured.

## 32.1 Provenance record

```rust
pub struct Provenance {
    pub source_digest: Digest,
    pub revision: Option<String>,
    pub pipeline_digest: Digest,
    pub toolchains: Vec<ToolchainRecord>,
    pub runner: RunnerAttestation,
    pub commands: Vec<SanitisedCommand>,
    pub materials: Vec<Material>,
    pub artifacts: Vec<ProducedArtifact>,
}
```

## 32.2 Dependency policy

Policies may enforce:

* lockfile required;
* licence allowlist;
* licence denylist;
* no Git dependencies;
* no unpinned Git revisions;
* no yanked dependencies;
* vulnerability threshold;
* duplicated dependency warnings;
* unsafe-code budget;
* source provenance requirements.

---

# 33. Security Boundaries

Forgeyard should have explicit trust zones.

```text
Trusted control plane
    ├── daemon
    ├── metadata store
    ├── secret broker
    └── signing service

Semi-trusted runners
    ├── platform toolchains
    ├── SDKs
    └── execution backends

Untrusted workload
    ├── repository source
    ├── build scripts
    ├── tests
    └── downloaded dependencies
```

## 33.1 Threats

Assume repository code may try to:

* read SSH keys;
* read browser profiles;
* steal signing credentials;
* modify host binaries;
* contact remote servers;
* poison caches;
* spoof test results;
* escape containers;
* consume all CPU, memory, disk, or processes;
* modify another build;
* replace generated artifacts after signing.

## 33.2 Defences

* dedicated service account;
* sandbox or VM;
* read-only source snapshot;
* per-job writable workspace;
* network denied by default;
* cgroup resource limits;
* process limits;
* mount allowlists;
* no host Docker socket exposure;
* immutable artifacts;
* cache namespaces by trust level;
* signing separated from compilation;
* digest verification between stages;
* runner attestation;
* short-lived leases.

---

# 34. Network Policy

```rust
pub enum NetworkPolicy {
    Deny,
    DependencyFetchOnly,
    AllowHosts(Vec<HostPattern>),
    AllowAll,
}
```

Suggested phases:

```text
dependency resolution: restricted network
build: no network
test: no network unless declared
deployment: destination allowlist
```

A proxy can enforce allowed domains and record dependency-fetch metadata.

---

# 35. Artifact Model

```rust
pub struct ArtifactRecord {
    pub id: ArtifactId,
    pub kind: ArtifactKind,
    pub target: TargetId,
    pub digest: Digest,
    pub size: u64,
    pub media_type: String,
    pub signed: bool,
    pub provenance: Option<ArtifactId>,
}
```

Artifact kinds:

```text
binary
dynamic-library
static-library
web-bundle
container-image
apk
aab
ipa
app-bundle
dmg
pkg
msi
msix
appimage
deb
rpm
tar-zst
symbols
coverage
test-report
sbom
provenance
checksum-manifest
```

## 35.1 Atomic release bundle

A release is a manifest referencing immutable artifacts.

```ron
(
    release: "1.4.0",
    revision: "abc123",
    artifacts: [
        (
            target: "linux-x86_64",
            digest: "sha256:...",
        ),
        (
            target: "windows-x86_64",
            digest: "sha256:...",
        ),
    ],
)
```

A release should be published only after all required artifacts are available and verified.

---

# 36. Packaging and Signing

Packaging is platform-specific and should be adapter-based.

```rust
#[async_trait]
pub trait Packager {
    fn supports(&self, target: &Target) -> bool;
    async fn package(
        &self,
        context: &PackageContext,
    ) -> Result<Vec<ProducedArtifact>>;
}
```

Signing is separate:

```rust
#[async_trait]
pub trait Signer {
    async fn sign(
        &self,
        artifact: ArtifactRef,
        identity: SigningIdentityRef,
    ) -> Result<SignedArtifact>;
}
```

This prevents build jobs from receiving long-lived signing credentials.

---

# 37. Deployment and Publication

Deployments are optional pipeline stages.

Possible targets:

* local directory;
* SSH/SFTP server;
* private binary repository;
* OCI registry;
* static web server;
* Android internal distribution;
* App Store Connect;
* release page;
* LAN device.

```rust
pub trait Publisher {
    async fn prepare(&self, release: &ReleaseManifest)
        -> Result<PublishPlan>;

    async fn publish(
        &self,
        plan: PublishPlan,
        idempotency_key: IdempotencyKey,
    ) -> Result<PublishResult>;
}
```

A deployment must support:

* dry run;
* idempotency;
* approval gates;
* rollback metadata;
* audit trail;
* destination allowlist.

---

# 38. Observability

Use `tracing` throughout.

Metrics:

* queue duration;
* job execution duration;
* cache hit rate;
* source upload bytes;
* artifact upload bytes;
* runner utilisation;
* test pass rate;
* flaky-test rate;
* failure categories;
* toolchain preparation time;
* scheduler decision latency;
* CAS size;
* garbage-collection reclaim rate.

Local exporters:

* structured log files;
* Prometheus endpoint;
* OpenTelemetry endpoint;
* JSON run report.

---

# 39. Performance Design

## 39.1 Avoid unnecessary copies

Use:

* memory-mapped reads for large immutable files where appropriate;
* streaming hashing;
* streaming compression;
* zero-copy byte slices in protocol decoding where lifetime-safe;
* hardlinks or reflinks for local immutable blobs;
* chunked artifact transfer.

## 39.2 Parallelism

Use three layers of concurrency:

### Pipeline concurrency

Independent DAG jobs run together.

### Build-tool concurrency

Cargo and compilers manage internal parallel compilation.

### Data concurrency

Hashing, compression, transfer, and report parsing use bounded worker pools.

Avoid oversubscription.

If four Cargo jobs each use all host cores, performance may be worse than executing fewer jobs.

The scheduler should calculate:

```text
effective job CPU =
    requested compiler threads
  + auxiliary process allowance
```

Set:

```text
CARGO_BUILD_JOBS
RAYON_NUM_THREADS
MAKEFLAGS
```

only when appropriate and declared.

## 39.3 Filesystem performance

Workspace strategies:

* copy-on-write snapshots;
* filesystem reflinks;
* hardlinked immutable inputs;
* per-job writable output directories;
* local CAS on the fastest available disk.

Do not use network filesystems for compiler working directories unless unavoidable.

## 39.4 Build timing

Cargo can emit build timing information, which Forgeyard can collect to identify expensive crates and critical build paths.

## 39.5 Priority scheduling

Priority order:

1. interactive validation;
2. manually requested builds;
3. release builds;
4. automatic background validation;
5. cache warming;
6. garbage collection.

---

# 40. Failure Classification

Do not expose every failure merely as “command exited with code 1.”

```rust
pub enum FailureCategory {
    Configuration,
    SourceCheckout,
    ToolchainUnavailable,
    DependencyResolution,
    Compilation,
    Linking,
    TestFailure,
    TestCrash,
    TestTimeout,
    Packaging,
    Signing,
    Notarisation,
    Deployment,
    Infrastructure,
    RunnerLost,
    ResourceExhausted,
    PolicyViolation,
    SecurityViolation,
}
```

Diagnostics should include:

* failed phase;
* exact sanitised command;
* working directory;
* exit status;
* relevant compiler messages;
* runner;
* toolchain;
* reproduction command;
* likely remediation.

---

# 41. Reproduction

Every failed job should generate a reproduction descriptor.

```bash
forgeyard reproduce <run-id> <job-id>
```

It reconstructs:

* source snapshot;
* toolchain;
* environment;
* command;
* declared services;
* target;
* network policy.

Secrets are not automatically reproduced.

---

# 42. Workspace Layout

```text
forgeyard/
├── Cargo.toml
├── rust-toolchain.toml
├── deny.toml
├── clippy.toml
├── crates/
│   ├── forgeyard-cli/
│   ├── forgeyard-daemon/
│   ├── forgeyard-runner/
│   ├── forgeyard-api/
│   ├── forgeyard-protocol/
│   ├── forgeyard-model/
│   ├── forgeyard-config/
│   ├── forgeyard-pipeline/
│   ├── forgeyard-scheduler/
│   ├── forgeyard-executor/
│   ├── forgeyard-sandbox/
│   ├── forgeyard-cas/
│   ├── forgeyard-cache/
│   ├── forgeyard-storage/
│   ├── forgeyard-events/
│   ├── forgeyard-logs/
│   ├── forgeyard-secrets/
│   ├── forgeyard-artifacts/
│   ├── forgeyard-provenance/
│   ├── forgeyard-policy/
│   ├── forgeyard-toolchains/
│   ├── forgeyard-detector/
│   ├── forgeyard-adapter-cargo/
│   ├── forgeyard-adapter-dioxus/
│   ├── forgeyard-adapter-wasm/
│   ├── forgeyard-adapter-android/
│   ├── forgeyard-adapter-xcode/
│   ├── forgeyard-adapter-oci/
│   ├── forgeyard-packaging/
│   ├── forgeyard-signing/
│   ├── forgeyard-device-lab/
│   ├── forgeyard-test-report/
│   └── forgeyard-ui/
├── schemas/
├── fixtures/
├── test-projects/
├── toolchain-images/
├── packaging/
└── docs/
```

---

# 43. Crate Responsibilities

## `forgeyard-model`

Pure domain types.

Must not depend on:

* Tokio;
* database implementation;
* HTTP framework;
* UI framework;
* OS-specific runner code.

## `forgeyard-config`

* RON parsing;
* validation;
* defaults;
* layered configuration;
* migration;
* diagnostics.

## `forgeyard-pipeline`

* matrix expansion;
* DAG construction;
* cycle detection;
* pipeline IR;
* fingerprints;
* policy binding.

## `forgeyard-scheduler`

* ready queue;
* runner matching;
* fairness;
* priorities;
* resource allocation;
* leases.

## `forgeyard-executor`

* execution traits;
* command model;
* cancellation;
* process collection;
* output streaming.

## `forgeyard-sandbox`

Platform-specific isolation implementations.

## `forgeyard-cas`

* streaming writes;
* digest verification;
* manifests;
* chunking;
* garbage collection.

## `forgeyard-storage`

* repositories;
* transactions;
* schema migrations;
* projections.

## `forgeyard-runner`

* registration;
* capability probing;
* workspace lifecycle;
* executor selection;
* heartbeat;
* upload/download.

## Adapter crates

Each adapter must depend on stable traits, not on daemon internals.

---

# 44. Domain Types

Prefer strongly typed IDs.

```rust
#[repr(transparent)]
pub struct RunId(Uuid);

#[repr(transparent)]
pub struct JobId(Uuid);

#[repr(transparent)]
pub struct RunnerId(Uuid);

#[repr(transparent)]
pub struct ArtifactId(Uuid);
```

Avoid raw strings for:

* platforms;
* architectures;
* target triples;
* artifact types;
* secret scopes;
* states;
* trust levels.

---

# 45. Platform Abstraction

```rust
pub enum OperatingSystem {
    Linux,
    Windows,
    MacOS,
    Android,
    IOS,
    Web,
}

pub enum Architecture {
    X86,
    X86_64,
    ArmV7,
    Aarch64,
    Wasm32,
    Universal,
}

pub struct TargetPlatform {
    pub os: OperatingSystem,
    pub arch: Architecture,
    pub abi: Option<Abi>,
    pub rust_target: Option<RustTargetTriple>,
}
```

Separate product platform from Rust compilation triple.

Android `arm64-v8a`, Rust `aarch64-linux-android`, Gradle ABI configuration, and device architecture are related but not interchangeable values.

---

# 46. Command Model

Avoid representing commands as shell strings.

```rust
pub struct CommandSpec {
    pub program: ToolReference,
    pub arguments: Vec<OsString>,
    pub working_directory: WorkspacePath,
    pub environment: BTreeMap<OsString, EnvironmentValue>,
    pub stdin: StdinPolicy,
    pub timeout: Duration,
    pub network: NetworkPolicy,
}
```

This prevents:

* accidental shell injection;
* quoting differences;
* platform-specific shell behaviour;
* ambiguous logging.

Shell scripts may be supported explicitly:

```rust
ExecutionSpec::ShellScript {
    shell: ShellKind,
    script: String,
}
```

They should not be the default representation.

---

# 47. Path Safety

Repository paths are untrusted.

Enforce:

* no absolute output paths;
* no `..` traversal;
* symlink validation;
* no output escaping the workspace;
* no artifact collection through unsafe symlinks;
* platform-safe path conversion.

Use logical workspace paths internally:

```rust
pub struct WorkspacePath(Utf8PathBuf);
```

Keep raw `OsString` at operating-system boundaries to avoid corrupting non-UTF-8 paths.

---

# 48. Build Matrix

A build matrix should be explicit and inspectable.

```rust
pub struct MatrixAxis {
    pub name: String,
    pub values: Vec<MatrixValue>,
}
```

Example:

```ron
matrix: (
    axes: {
        "os": ["linux", "windows", "macos"],
        "arch": ["x86_64", "aarch64"],
        "profile": ["debug", "release"],
    },
    exclude: [
        {"os": "windows", "arch": "aarch64", "profile": "debug"},
    ],
)
```

Matrix expansion must impose limits to prevent accidental job explosions.

```text
maximum matrix jobs: 128
```

Require explicit override above the limit.

---

# 49. Incremental Change Analysis

A future optimisation can identify affected packages.

Inputs:

* changed files;
* Cargo workspace graph;
* package dependencies;
* test ownership;
* target-specific files;
* build script dependencies.

Example:

```text
changed:
  crates/accounting-engine/src/ledger.rs

affected:
  accounting-engine
  reporting
  server
  desktop
  integration-tests

unaffected:
  static documentation
  unrelated CLI examples
```

This optimisation must be conservative.

When uncertain, run more jobs rather than risk skipping required validation.

---

# 50. Local Triggers

Supported triggers:

```rust
pub enum Trigger {
    Manual,
    SourceChanged,
    GitCommit,
    GitTag { pattern: String },
    Schedule { expression: String },
    LocalHook,
    Api,
}
```

## 50.1 Filesystem watch

Use `notify` to observe source changes.

Debounce events and calculate actual content differences before starting a run.

## 50.2 Git hook

Install optional hooks:

```text
pre-commit
pre-push
post-commit
```

Hooks should call Forgeyard, not duplicate pipeline logic.

## 50.3 Scheduled local builds

Use an internal scheduler or integrate with:

* systemd timers;
* Windows Task Scheduler;
* launchd.

---

# 51. Cancellation

Cancellation must be hierarchical.

```text
Cancel run
  ↓
cancel stages
  ↓
cancel jobs
  ↓
cancel execution backend
  ↓
terminate child process tree
  ↓
release devices
  ↓
clean temporary workspace
```

On Unix:

1. send graceful termination;
2. wait for bounded grace period;
3. kill process group;
4. clean descendants.

On Windows, use Job Objects to control process trees.

---

# 52. Recovery

After daemon restart:

1. replay event journal;
2. rebuild projections;
3. mark expired leases;
4. query reachable runners;
5. reconcile running executions;
6. retry safe jobs;
7. mark unknown non-idempotent jobs for manual review;
8. resume artifact uploads where possible.

The daemon must never infer that a signing or deployment job failed safely without checking its idempotency record.

---

# 53. Garbage Collection

CAS garbage collection uses mark-and-sweep.

Roots:

* retained runs;
* pinned releases;
* current cache entries;
* active leases;
* manually pinned artifacts;
* toolchain manifests.

Process:

```text
acquire GC generation
  ↓
mark reachable manifests
  ↓
mark referenced blobs
  ↓
quarantine unreferenced blobs
  ↓
wait safety interval
  ↓
delete
```

Quarantine prevents race-related immediate deletion.

---

# 54. API Design

Local API modules:

```text
/api/v1/repositories
/api/v1/runs
/api/v1/jobs
/api/v1/runners
/api/v1/artifacts
/api/v1/caches
/api/v1/secrets
/api/v1/events
```

Use typed request and response structures.

For live updates:

* Server-Sent Events for browser UI simplicity;
* WebSocket if bidirectional interactive functionality is needed;
* native framed protocol for runners.

---

# 55. Policy Engine

Policies are deterministic predicates over pipeline and execution context.

```rust
pub trait Policy {
    fn evaluate(
        &self,
        input: &PolicyInput,
    ) -> Vec<PolicyFinding>;
}
```

Examples:

* reject unpinned toolchains;
* reject network access during compilation;
* require lockfile;
* require tests before signing;
* disallow production secrets on untrusted runners;
* require native test for release targets;
* require provenance for release;
* prevent deployment from a dirty working tree;
* require signed Git tag;
* prevent release if flaky tests exist.

---

# 56. Release Quality Gates

A release pipeline should require:

```text
source tree clean
AND source revision recorded
AND pipeline definition committed
AND all required validation jobs passed
AND platform-native builds passed
AND package verification passed
AND vulnerability policy passed
AND artefact hashes recorded
AND signing completed
AND provenance generated
```

Compilation on Linux for a Windows or Apple target must not automatically satisfy a native-release test requirement.

---

# 57. Recommended Technology Stack

Core:

* `tokio` — asynchronous runtime;
* `serde` — serialisation model;
* `ron` — human configuration;
* `postcard` — compact internal protocol;
* `bytes` — byte buffers;
* `blake3` — internal content hashing;
* `sha2` — compatible published checksums;
* `clap` — CLI;
* `tracing` — observability;
* `miette` — diagnostics;
* `thiserror` — library errors;
* `anyhow` — executable-boundary aggregation;
* `rusqlite` — initial metadata database;
* `quinn` — QUIC runner transport;
* `rustls` — transport security;
* `notify` — filesystem watching;
* `ignore` — Git-compatible file traversal;
* `camino` — UTF-8 project paths where appropriate;
* `cap-std` — capability-oriented filesystem access;
* `tempfile` — temporary resources;
* `zstd` — compression;
* `tar` — archive generation;
* `uuid` or UUID-compatible typed identifiers;
* `semver` — toolchain and adapter versions.

Use dependency features carefully to limit binary size and compile time.

---

# 58. Error Architecture

Library crates expose typed errors.

```rust
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("pipeline contains dependency cycle: {0}")]
    CycleDetected(String),

    #[error("unknown target: {0}")]
    UnknownTarget(String),

    #[error("no runner satisfies requirements: {0}")]
    UnsatisfiedCapabilities(String),

    #[error("policy rejected job: {0}")]
    PolicyRejected(String),
}
```

Executable boundaries attach contextual diagnostics using `miette` or `anyhow`.

Do not erase meaningful domain errors too early.

---

# 59. Testing Forgeyard Itself

## 59.1 Unit tests

Test:

* matrix expansion;
* DAG validation;
* state transitions;
* capability matching;
* cache fingerprinting;
* configuration merging;
* path validation;
* secret redaction.

## 59.2 Property tests

Use property testing for:

* DAG operations;
* serialisation round trips;
* path containment;
* scheduler invariants;
* event replay;
* CAS chunk reconstruction.

## 59.3 Integration tests

Create fixture repositories:

```text
test-projects/
├── rust-library/
├── rust-workspace/
├── axum-server/
├── dioxus-web/
├── dioxus-desktop/
├── android-rust/
├── ios-rust/
├── failing-tests/
├── malicious-paths/
└── cache-behaviour/
```

## 59.4 End-to-end tests

Test:

* daemon plus local runner;
* runner disconnect;
* daemon restart;
* cancellation;
* cache hit;
* artifact corruption;
* interrupted upload;
* unavailable target;
* expired lease;
* denied network access.

## 59.5 Security tests

Include malicious fixtures attempting:

* path traversal;
* symlink escape;
* environment exfiltration;
* secret printing;
* fork bombs;
* excessive disk writes;
* cache poisoning;
* malformed protocol frames.

---

# 60. Implementation Phases

## Phase 1: Local Rust CI core

Deliver:

* CLI;
* daemon;
* embedded local runner;
* RON configuration;
* Cargo adapter;
* native process executor;
* pipeline DAG;
* job state machine;
* local SQLite metadata;
* CAS;
* logs;
* Linux host support;
* format, check, Clippy, test, build.

This phase should already be usable.

## Phase 2: Isolation and caching

Deliver:

* Linux sandbox;
* container executor;
* cache fingerprints;
* dependency cache;
* artifact reuse;
* resource limits;
* network policy;
* secret redaction;
* crash recovery.

## Phase 3: Cross-target Linux and web

Deliver:

* Rust target matrix;
* WASM builds;
* web packaging;
* MUSL and GNU server targets;
* QEMU smoke tests;
* package archives;
* SBOM and provenance.

## Phase 4: Multi-runner protocol

Deliver:

* separate runner agent;
* QUIC transport;
* mutual authentication;
* capabilities;
* leases;
* heartbeats;
* resumable transfer;
* LAN runner discovery.

## Phase 5: Windows

Deliver:

* native Windows runner;
* MSVC builds;
* Windows process isolation;
* MSI/MSIX integration;
* Windows signing;
* Windows runtime tests.

## Phase 6: Android

Deliver:

* Android SDK/NDK detector;
* ABI matrix;
* Gradle adapter;
* emulator management;
* device management;
* APK/AAB packaging;
* Android signing.

## Phase 7: Apple

Deliver:

* macOS runner;
* Xcode capability detection;
* macOS universal applications;
* `.app`, `.dmg`, and `.pkg`;
* iOS simulator tests;
* device builds;
* archive and IPA export;
* codesigning and notarisation.

## Phase 8: Local UI and advanced analytics

Deliver:

* Dioxus UI;
* pipeline visualisation;
* historical timing;
* flaky-test tracking;
* cache analytics;
* runner utilisation;
* build comparison.

## Phase 9: Deployment

Deliver:

* local publication;
* SSH deployment;
* OCI registry;
* static-site upload;
* release channels;
* approval gates;
* rollback records.

---

# 61. Minimal Viable Configuration

The first usable version should support:

```ron
(
    version: 1,
    pipeline: (
        jobs: {
            "format": (
                command: ["cargo", "fmt", "--all", "--", "--check"],
            ),
            "check": (
                needs: ["format"],
                command: [
                    "cargo", "check",
                    "--workspace",
                    "--all-targets",
                ],
            ),
            "clippy": (
                needs: ["format"],
                command: [
                    "cargo", "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D", "warnings",
                ],
            ),
            "test": (
                needs: ["check", "clippy"],
                command: [
                    "cargo", "test",
                    "--workspace",
                ],
            ),
            "release": (
                needs: ["test"],
                command: [
                    "cargo", "build",
                    "--workspace",
                    "--release",
                ],
                outputs: [
                    "target/release/application",
                ],
            ),
        },
    ),
)
```

Do not begin with an enormous dynamic plugin system. Start with typed built-in adapters and stable internal traits.

---

# 62. Recommended First Deployment Topology

For a serious personal setup:

```text
Linux workstation/server
  ├── Forgeyard daemon
  ├── Linux runner
  ├── Web/WASM runner
  ├── Android SDK/NDK runner
  ├── CAS
  └── metadata database

Windows machine or VM
  └── Native Windows runner

Mac mini
  ├── Native macOS runner
  ├── iOS simulator runner
  ├── signing service
  └── optional connected iPhone
```

This topology covers all major release platforms without pretending that Apple and Windows-native tooling can be completely replaced by Linux cross-compilation.

---

# 63. Final Architectural Decisions

## Use RON for

* repository pipeline configuration;
* toolchain declarations;
* local policies;
* target matrices;
* runner configuration.

## Use Postcard for

* daemon-runner protocol;
* durable internal event envelopes;
* compact cache manifests;
* job leases;
* local IPC messages.

## Use SQLite initially for

* metadata;
* job states;
* event indexes;
* artifact references;
* runner records;
* cache records.

## Use filesystem CAS for

* source snapshots;
* logs;
* artifacts;
* reports;
* large intermediate files.

## Use Tokio for

* orchestration;
* transport;
* log streaming;
* process supervision;
* concurrent artifact transfer.

## Use external platform SDKs for

* Android packaging;
* Windows native compilation;
* Xcode builds;
* signing;
* notarisation;
* installers.

## Use native runners for final assurance

Cross-compilation proves that code can often be compiled for a target. It does not prove that the application installs, launches, renders, integrates with the operating system, signs correctly, or passes runtime tests.

The release policy must therefore distinguish:

```text
cross-compiled
native-compiled
emulator-tested
simulator-tested
physical-device-tested
signed
distribution-verified
```

That distinction is the foundation of a reliable multi-platform CI/CD system.
