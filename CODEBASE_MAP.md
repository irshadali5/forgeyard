# Forgeyard Codebase Map & Directory Architecture Guide

Welcome to the **Forgeyard** codebase. This document is a comprehensive, structured map of every top-level directory, subproject, crate category, and system configuration file in the repository.

---

## 1. Architectural Overview & Design Principles

Forgeyard is an open-source, next-generation, hermetic, polyglot CI/CD, build orchestration, and software delivery platform. It is structured as a **modular monolith** with the following foundational principles:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                          Modular Monolith                              │
│                                                                        │
│   One Canonical Git Repo   +   One Cargo Workspace                     │
│   Layered Capability Crates +   Strict Unidirectional Dependencies     │
│   Pluggable Ecosystems     +   Cross-Platform Targets                  │
└────────────────────────────────────────────────────────────────────────┘
```

1. **VCS Neutrality**: Core pipelines and builds operate on content-addressed `SourceSnapshotId` rather than being tightly coupled to a single VCS (Git, Jujutsu, Mercurial, Pijul, Darcs, Breezy, Fossil).
2. **Hermetic & Content-Addressed Execution**: Actions, toolchains, outputs, and intermediate states are indexed via Content-Addressable Storage (CAS) with cryptographic hashing (BLAKE3 / SHA-256).
3. **Polyglot First-Class Support**: Dedicated ecosystem subsystems handle native toolchain detection, dependency resolution, lockfiles, compilation, testing, and packaging across 9+ language ecosystems.
4. **Reproducibility & Provenance**: First-class SLSA Level 3/4 provenance, in-toto attestations, SBOM generation (SPDX/CycloneDX), bit-for-bit diffing, and environment normalization.
5. **Flexible Topologies**: Single-binary local execution (`apps/forgeyard` / CLI) scaling up to horizontally partitioned distributed runners, remote build execution (RBE), isolated signing workers, and device test agents.

---

## 2. Repository Directory Tree

```text
forgeyard/
├── .forgeyard/          # Workspace-level self-hosting configuration, pipelines, and policies
├── apps/                # Standalone runnable binary applications and daemons
├── crates/              # Core capability-oriented shared Rust crates (52+ domain areas)
├── ecosystems/          # Language ecosystem adapters (Rust, C/C++, Go, Python, etc.)
├── native/              # Low-level native tooling (ABI, linkers, assembly, sysroots, objects)
├── platforms/           # Target platform support (Linux, Windows, Apple, Android, WASM, Embedded)
├── protocols/           # Wire protocols, envelope definitions, and compatibility suites
├── schemas/             # JSON/RON schemas for APIs, manifests, configs, and events
├── config/              # Default configuration templates and runtime defaults
├── policies/            # Declarative security, approval, and compliance policies
├── migrations/          # Database schema migrations for PostgreSQL and Stoolap
├── packaging/           # Packaging definitions (deb, rpm, pkg, msi, apk, oci, fypkg)
├── deploy/              # Deployment manifests (docker-compose, kubernetes, systemd)
├── infra/               # Infrastructure as Code (Terraform, Ansible, Nix, devcontainers)
├── assets/              # Static assets, branding, and icons
├── web/                 # Web assets and frontend resources
├── fixtures/            # Static test fixtures, mock certificates, and sample inputs
├── examples/            # Example pipelines, change proposals, and ecosystem configs
├── tests/               # Workspace-wide integration, e2e, security, and reproducibility tests
├── benches/             # Criterion/benchmarking suites for performance testing
├── fuzz/                # Cargo-fuzz / libFuzzer fuzzing targets
├── tools/               # Internal developer inspection, verification, and schema tools
├── xtask/               # Workspace automation runner (`cargo xtask`)
├── scripts/             # Shell and automation scripts
├── docs/                # Developer, administrative, reference, and operational documentation
├── adr/                 # Architecture Decision Records (ADRs)
├── rfcs/                # Request for Comments (RFCs)
├── sys-arch/            # 70 detailed technical system architecture specifications
├── contrib/             # Community contributions, shell completions, and editor plugins
├── vendor/              # Vendored dependencies and third-party references
└── LICENSES/            # SPDX licenses and legal notices
```

---

## 3. Root Files Reference

| File | Purpose |
|---|---|
| `Cargo.toml` | Root workspace manifest declaring members, workspace dependencies, linting profiles, and compilation settings. |
| `Cargo.lock` | Exact pinned dependency lockfile for reproducible workspace builds. |
| `rust-toolchain.toml` | Declares the pinned Rust compiler channel, version, components, and target triples. |
| `rustfmt.toml` | Canonical Rust code formatting rules enforced across all crates. |
| `clippy.toml` | Static analysis, strict linting levels, and forbidden pattern rules. |
| `deny.toml` | `cargo-deny` configuration for verifying licenses, bans, advisories, and dependency duplicates. |
| `typos.toml` | Automated spelling and typo checker configuration. |
| `architecture.ron` | Machine-readable workspace architecture contract defining allowed/forbidden crate dependency edges. |
| `.editorconfig` | Cross-editor formatting rules (indentation, charset, line endings, trim trailing whitespace). |
| `.gitignore` / `.gitattributes` | Git repository ignore filters and line ending / binary file attributes. |
| `.dockerignore` | Build context exclusions for container image builds. |
| `README.md` | Primary entrypoint, introduction, quick-start, and project vision. |
| `CONTRIBUTING.md` | Contribution guidelines, coding standards, PR workflow, and testing requirements. |
| `CODE_OF_CONDUCT.md` | Community standards and behavioral expectations. |
| `GOVERNANCE.md` | Project stewardship, leadership structure, and decision-making model. |
| `MAINTAINERS.md` | List of subsystem maintainers, CODEOWNERS, and domain responsibilities. |
| `SECURITY.md` | Vulnerability disclosure policy, reporting instructions, and security model. |
| `RELEASES.md` | Release channels (nightly, beta, stable), cadence, and versioning scheme. |
| `CHANGELOG.md` | Historical log of notable changes, additions, fixes, and deprecations. |
| `ROADMAP.md` | High-level development milestones, vision, and roadmap deliverables. |
| `forgeyard-complete-workspace-structure.md` | Definitive internal reference guide detailing the complete workspace architecture. |

---

## 4. Self-Hosting Configuration (`.forgeyard/`)

Contains Forgeyard's own pipeline, toolchain, policy, and release definitions used when Forgeyard builds, tests, and releases itself:

```text
.forgeyard/
├── forgeyard.ron       # Primary workspace configuration (roots, storage, caching)
├── pipeline.ron        # Canonical CI/CD pipeline definitions for Forgeyard
├── policy.ron          # Quality gates, security rules, and required checks
├── ownership.ron       # Component and subsystem code ownership definitions
├── release.ron         # Artifact promotion, signing, and distribution rules
├── toolchains.ron      # Declared hermetic toolchain requirements (Rust, LLVM, etc.)
├── lock/               # Locked toolchain hashes and dependency snapshots
└── templates/          # Reusable pipeline and step templates
```

---

## 5. Runnable Applications (`apps/`)

The `apps/` directory houses the standalone executable binaries that comprise Forgeyard's runtime fleet:

```text
apps/
├── forgeyard/              # Unified monolithic server / single-process all-in-one daemon
├── forgeyard-cli/          # Developer CLI (pipeline trigger, local builds, inspections, CP management)
├── forgeyard-daemon/       # Background local service for workspace indexing & local execution
├── forgeyard-agent/        # Remote runner agent connecting to the scheduler for job execution
├── forgeyard-worker/       # Generic distributed task execution and build worker
├── forgeyard-device-agent/ # Specialized runner for connected hardware, mobile, & embedded devices
├── forgeyard-signing-worker/# Air-gapped, isolated cryptographic signing daemon for release assets
├── forgeyard-ui/           # Interactive GUI client built with Dioxus (desktop / web)
└── forgeyard-migration/    # CLI tool for database schema and storage backend migrations
```

---

## 6. Core Crates Layer (`crates/`)

The `crates/` directory contains 50+ domain capability packages organized logically:

### Primitives & Foundation
- `crates/ids/forgeyard-ids`: Strongly-typed entity identifiers (`RunId`, `JobId`, `ArtifactId`, `SourceSnapshotId`).
- `crates/core/forgeyard-core`: Shared core primitives, common traits, error types, and utilities.
- `crates/time/forgeyard-time`: Deterministic time mocking, monotonic clocks, and duration types.
- `crates/crypto/`: Cryptographic hashing, key generation, and signing primitives.

### Protocol, Transport & Wire Format
- `crates/protocol/`: Postcard serialization models, wire envelope encodings, API versioning.
- `crates/transport/`: Network communication layer supporting HTTP/2, WebSockets, and QUIC.
- `crates/coordination/`: Distributed locks, leader election, and raft/paxos primitives.
- `crates/rbe/`: Remote Build Execution (RBE) protocol, action cache, and execution service.

### VCS & Source Management
- `crates/vcs/`: VCS-neutral source snapshots, diffing, graph inspection, and drivers for Git, Jujutsu, Mercurial, Pijul, Darcs, Fossil, and Breezy.
- `crates/scm/`: SCM forge integrations (GitHub, GitLab, Gitea, Forgejo, SourceHut) and webhooks.
- `crates/change/`: Change Proposal (CP) workflow, speculative merge queues, and approval tracking.

### Hermeticity, CAS & Toolchains
- `crates/hermetic/`: Pure functional derivations, store paths, impure detection, and environment sandboxing.
- `crates/toolchain/`: Toolchain resolution, trust verification, caching, and hermetic downloads.
- `crates/store/`: Persistence abstraction layer with backends for PostgreSQL, Neon, and Stoolap.
- `crates/cache/`: Multi-tier caching (local CAS, distributed remote cache, HTTP cache).

### Pipelines, Jobs & Scheduling
- `crates/pipeline/`: Pipeline parsing (RON/YAML), DAG validation, matrix generation, IR compiler.
- `crates/run/`: Execution state machine, run/job lifecycle tracking, attempt retry logic.
- `crates/scheduler/`: Priority-based job placement, resource scoring, and lease management.
- `crates/lease/`: Distributed resource leases for execution slots, devices, and licenses.

### Execution, Runners & Sandboxing
- `crates/runner/`: Runner capability detection, heartbeats, log streaming, and workspace cleanup.
- `crates/executor/`: Process executors for Linux, Windows, macOS, container, and confidential VMs.
- `crates/sandbox/`: OS-level isolation (Linux bubblewrap/namespaces/seccomp, Windows AppContainer, Apple sandbox).
- `crates/device/`: Hardware device discovery, serial communication, and mobile/embedded target management.

### Security, Secrets, Policy & Identity
- `crates/identity/`: User identity, authentication, OIDC, SAML, SCIM, and session management.
- `crates/secrets/`: Zeroizing secret storage, cloud KMS integrations (AWS, GCP, Vault), and masking.
- `crates/policy/`: Declarative policy engine, approval gates, compliance rules, and exception tracking.
- `crates/security/`: Secret leak detection, vulnerability scanning, and static security checks.
- `crates/supply-chain/`: SLSA provenance, in-toto attestations, SBOM generation, and VEX statements.

### Packaging, Artifacts & Release
- `crates/artifact/`: Content-addressed artifact collection, hashing, indexing, and storage.
- `crates/package/`: Native package format generators (Debian `.deb`, RPM `.rpm`, macOS `.pkg`/`.dmg`, Windows `.msi`/`.msix`, Android `.apk`/`.aab`, OCI images, and native `.fypkg`).
- `crates/release/`: Release promotion, version bumps, release candidate validation, and cryptographic signing.
- `crates/reproducibility/`: Bitwise artifact diffing, build normalization, and reproducibility verification.

### Observability, Health & Telemetry
- `crates/telemetry/`: Distributed tracing, metrics export (OpenTelemetry/Prometheus), and structured logging.
- `crates/health/`: System health probes, readiness checks, and diagnostic "doctor" inspections.
- `crates/notification/`: Alerting and notifications (Email, Slack, Webhooks, in-app notifications).
- `crates/audit/`: Append-only tamper-evident audit logs and governance event streams.
- `crates/reconciliation/`: Background reconciliation loops ensuring desired-state convergence.

---

## 7. Ecosystems Layer (`ecosystems/`)

Contains polyglot language toolchain adapters. Each ecosystem provides detection, lockfile parsing, package modeling, compilation, testing, coverage, and publishing:

```text
ecosystems/
├── rust/           # Cargo workspace detection, rustc flags, miri, clippy, doc, cross, publishing
├── c-cpp/          # CMake, Meson, Ninja, compile_commands.json, clang-tidy, header dependencies
├── go/             # Go modules, go build, vet, test, cover, cgo integration, toolchain downloads
├── js-ts/          # npm, pnpm, yarn, bun, biome, oxc, vite, package.json parsing, node-gyp
├── python/         # PEP 517/518, pip, poetry, uv, wheel, sdist, venv isolation, cython, pyo3
├── jvm/            # Maven, Gradle, javac, kotlinc, jar/war packaging, JUnit test runners
├── dart/           # Flutter SDK, pubspec, dart analyzer, test, compilation to native/web
├── swift/          # SwiftPM, Package.swift, Xcode, clang integration, Apple notarization/signing
└── web/            # Tailwind, PostCSS, Sass, static assets, HTML/CSS bundling, browser tests
```

Each ecosystem adheres to a standard crate structure:
`forgeyard-<lang>`, `forgeyard-<lang>-detect`, `forgeyard-<lang>-model`, `forgeyard-<lang>-lock`, `forgeyard-<lang>-package`, `forgeyard-<lang>-test`, `forgeyard-<lang>-publish`.

---

## 8. Native Toolchain Subsystems (`native/`)

Low-level cross-compilation, binary parsing, and platform ABI toolchains:

```text
native/
├── abi/            # Platform ABI models (SysV, Windows x64, AAPCS/ARM, RISC-V, WebAssembly)
├── api/            # Generic native build request schemas and model definitions
├── assembly/       # Assembly toolchains (nasm, yasm, gas, llvm-as), disassembly, and relocations
├── binary/         # Binary format inspectors and parsers (ELF, Mach-O, PE/COFF, WASM)
├── libc/           # C runtime bindings and compatibility layers (glibc, musl, bionic, msvcrt)
├── linker/         # Linker integration and model drivers (lld, mold, bfd, apple ld, msvc link)
├── object/         # Object file inspection, symbol tables, relocation fixups, and disassemblers
├── pkgconfig/      # `pkg-config` parser and native library search path resolver
├── runtime/        # Runtime closure resolution and dynamic dependency graph verification
└── sysroot/        # Hermetic cross-compilation sysroot generators (Linux, Android, Apple, Windows)
```

---

## 9. Platforms Subsystems (`platforms/`)

Platform-specific runner implementations, sandbox drivers, and device interfaces:

```text
platforms/
├── api/            # Platform abstraction traits (`Platform`, `Sandbox`, `Device`, `Sdk`)
├── linux/          # Linux namespaces, cgroups v2, Bubblewrap, seccomp-bpf, io_uring, eBPF
├── windows/        # Windows sandboxes, job objects, AppContainer, MSVC SDKs, code signing
├── apple/          # macOS/iOS sandboxes, Xcode integration, provisioning profiles, notarization
├── android/        # Android NDK/SDK, ADB device bridge, emulator management, APK signing
├── browser/        # Headless browser testing drivers (Chromium, Firefox, WebKit via CDP/WebDriver)
├── wasm/           # WebAssembly components, WASI runtimes (Wasmtime, Wasmer), and targets
└── embedded/       # Embedded toolchains, QEMU target simulation, flashing tools, and serial probes
```

---

## 10. Protocols, Schemas, Config & Policies

- **`protocols/`**: Defines internal and public wire protocols, postcard envelope specifications, and cross-version compatibility test suites.
- **`schemas/`**: Formal RON, JSON Schema, and OpenAPI/AsyncAPI specifications for configuration, events, pipeline definitions, and API requests.
- **`config/`**: Default and production configuration files (`daemon.ron`, `runner.ron`, `server.ron`).
- **`policies/`**: Declarative security rules (default security baselines, enterprise approval requirements, air-gapped policies).

---

## 11. Database Migrations (`migrations/`)

Managed database migration scripts:
- `migrations/postgres/`: SQL migrations for PostgreSQL / Neon distributed metadata stores.
- `migrations/stoolap/`: Embedded transactional migration definitions for Stoolap storage engines.

---

## 12. Packaging, Deployment & Infrastructure

- **`packaging/`**: Build recipes for distributing Forgeyard binaries across Linux (`deb`, `rpm`, `arch`), macOS (`dmg`, `pkg`, `brew`), Windows (`msi`, `msix`, `winget`), Android, and OCI containers.
- **`deploy/`**: Turnkey deployment definitions for `docker-compose`, Kubernetes (Helm charts & manifests), standalone single-node installs, and systemd units.
- **`infra/`**: Infrastructure as Code assets including Terraform modules, Ansible playbooks, Nix expressions, and `.devcontainer` configurations.

---

## 13. Internal Developer Tools (`tools/` & `xtask/`)

- **`xtask/`**: Rust workspace automation tool (`cargo xtask`):
  - `cargo xtask test`: Runs full polyglot test suite.
  - `cargo xtask lint`: Runs lints, clippy, typos, and architecture check.
  - `cargo xtask fmt`: Formats the entire workspace.
  - `cargo xtask selfhost`: Executes the self-hosting build pipeline.
  - `cargo xtask release`: Validates and packages release artifacts.
  - `cargo xtask docs`: Builds and validates all documentation.
- **`tools/`**: Specialized diagnostics and developer utilities:
  - `forgeyard-architecture-check`: Validates crate dependency directions against `architecture.ron`.
  - `forgeyard-cas-inspect`: Inspects and debugs local and remote CAS objects.
  - `forgeyard-object-inspect`: Disassembles and inspects object files and ELF/Mach-O/PE headers.
  - `forgeyard-protocol-inspect`: Inspects and decodes postcard wire protocol envelopes.
  - `forgeyard-schema-gen`: Generates JSON/RON schemas from Rust data structures.
  - `forgeyard-snapshot-inspect`: Inspects VCS-neutral source tree snapshots.
  - `forgeyard-workspace-index`: Indexes workspace crates, dependencies, and affected graphs.

---

## 14. Testing & Verification Suites

- **`tests/`**: High-level integration, end-to-end, and domain verification suites:
  - `tests/change/`: Change proposal, speculative queue, and mergeability tests.
  - `tests/reproducibility/`: Bitwise reproducibility tests across all supported languages.
  - `tests/security/`: Security boundary, sandbox escape, secret leak, and SSRF tests.
  - `tests/selfhost/`: End-to-end tests validating that Forgeyard can build and verify itself.
  - `tests/vcs/`, `tests/distributed/`, `tests/platforms/`, `tests/ecosystems/`.
- **`benches/`**: Benchmark suites for CAS hashing, pipeline parsing, scheduling throughput, and execution latencies.
- **`fuzz/`**: Fuzzing targets testing wire protocol parsers, envelope decoders, pipeline parsers, and archive extractors.
- **`fixtures/`**: Static fixtures (sample repos, certificates, invalid payloads, lockfiles).
- **`examples/`**: Real-world pipeline examples, change proposals, and multi-language workspaces.

---

## 15. Documentation & System Architecture Specs

- **`docs/`**: General documentation covering getting started, concepts, administration, operations, security, API references, and troubleshooting.
- **`adr/`**: Architecture Decision Records capturing fundamental design choices.
- **`rfcs/`**: Formal Request for Comments tracking proposed, accepted, and superseded system enhancements.
- **`sys-arch/`**: 70 exhaustive, standalone technical architecture specifications detailing every subsystem in Forgeyard (from CAS, RBE, VCS neutrality, to FinOps, multi-region federation, merge queues, and AI-assisted CI).

---

## 16. Dependency Layers & Architecture Rules

Forgeyard strictly enforces an 8-tier unidirectional dependency hierarchy. Higher layers may depend on lower layers; lower layers **never** depend on higher layers:

```text
Layer 7: Apps & Daemons
  ▲      apps/forgeyard, apps/forgeyard-cli, apps/forgeyard-ui, apps/forgeyard-agent
  │
Layer 6: Platform & Ecosystem Adapters
  ▲      platforms/* (linux, windows, apple), ecosystems/* (rust, python, go, etc.)
  │
Layer 5: Orchestration & Workflow Engine
  ▲      crates/pipeline, crates/scheduler, crates/change, crates/release
  │
Layer 4: Execution & Sandboxing
  ▲      crates/executor, crates/sandbox, crates/runner, crates/device
  │
Layer 3: Hermetic Foundation, CAS & Storage
  ▲      crates/hermetic, crates/store, crates/cache, crates/toolchain
  │
Layer 2: Source, VCS & SCM
  ▲      crates/vcs, crates/scm, crates/diff
  │
Layer 1: Protocols, Identity, Policy & Telemetry
  ▲      crates/protocol, crates/transport, crates/policy, crates/identity, crates/telemetry
  │
Layer 0: Primitives & Foundation
         crates/ids, crates/core, crates/time, crates/crypto
```

---

## 17. Navigation Cheat Sheet

- **Looking for an executable entry point?** → Check `apps/`.
- **Looking for core CI/CD logic (scheduling, runs, pipelines)?** → Check `crates/run/`, `crates/pipeline/`, `crates/scheduler/`.
- **Looking for language-specific build & test logic?** → Check `ecosystems/<language>/`.
- **Looking for OS sandboxing (Bubblewrap, AppContainer)?** → Check `platforms/<os>/` and `crates/sandbox/`.
- **Looking for CAS or hermetic store handling?** → Check `crates/hermetic/` and `crates/rbe/`.
- **Looking for how Forgeyard tests itself?** → Check `.forgeyard/` and `tests/selfhost/`.
- **Looking for deep architectural specs?** → Check `sys-arch/`.
- **Need to run tests, formatting, or checks?** → Run `cargo xtask [test|lint|fmt|docs]`.
