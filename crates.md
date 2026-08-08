# 📦 Forgeyard Crate Audit & Recommended Open-Source Replacements

This document provides a comprehensive architectural audit of all manual and custom implementations across Forgeyard's 37 workspace crates, identifying production-ready standard Rust crates available on **crates.io** for replacement or integration.

---

## 📊 Executive Summary Matrix

| Forgeyard Component / Crate | Current Manual Implementation | Recommended Rust Crate | Primary Use Case & Architectural Benefit |
| :--- | :--- | :--- | :--- |
| **`forgeyard-secrets`** | Custom byte XOR cipher loop (`byte ^ key[i % 32]`) | [`chacha20poly1305`](https://crates.io/crates/chacha20poly1305) or [`aes-gcm`](https://crates.io/crates/aes-gcm) | Authenticated AEAD encryption with tamper prevention and hardware acceleration. |
| **`forgeyard-signing`** | Mock ML-DSA-87 signature formatting (`format!("mldsa-87-{}", hash)`) | [`pqcrypto-mldsa`](https://crates.io/crates/pqcrypto-mldsa) / [`pqcrypto-dilithium`](https://crates.io/crates/pqcrypto-dilithium) | Real NIST FIPS 204 Post-Quantum ML-DSA (Dilithium) signature generation & verification. |
| **`forgeyard-provenance`** | Subprocess invocation of `git` binary (`std::process::Command::new("git")`) | [`gix`](https://crates.io/crates/gix) (gitoxide) or [`git2`](https://crates.io/crates/git2) | In-process, zero-overhead Git repository query engine without spawning external binaries. |
| **`forgeyard-provenance`** | Mock STARK proof formatting (`format!("zk-stark-{}", hash)`) | [`winterfell`](https://crates.io/crates/winterfell) or [`sp1`](https://crates.io/crates/sp1) | Production-grade Zero-Knowledge STARK proof generation and verification. |
| **`forgeyard-policy`** | Custom string matching rules (`command.contains("rm -rf")`) | [`cedar-policy`](https://crates.io/crates/cedar-policy) or [`rhai`](https://crates.io/crates/rhai) | High-performance, formal policy evaluation engine with fine-grained access control semantics. |
| **`forgeyard-sandbox`** | Custom raw Linux syscall wrappers & manual `unshare` logic | [`landlock`](https://crates.io/crates/landlock) & [`nix`](https://crates.io/crates/nix) | Safe, idiomatic Rust abstraction for Linux Landlock LSM sandboxing, namespaces, and cgroups. |
| **`forgeyard-analyzer`** | Mock string output generator simulating quantized LLM code fixes | [`candle-core`](https://crates.io/crates/candle-core) / [`candle-transformers`](https://crates.io/crates/candle-transformers) | High-performance local GGUF/ONNX quantized LLM inference runtime in pure Rust (HuggingFace Candle). |
| **`forgeyard-pipeline`** | Manual in-degree / DFS graph topological sorting algorithm | [`petgraph`](https://crates.io/crates/petgraph) | Production-grade DAG dependency resolution, cycle detection, and graph traversal. |
| **`forgeyard-pipeline`** | Custom nested loop implementation for matrix combination generation | [`itertools`](https://crates.io/crates/itertools) | Efficient `multi_cartesian_product` for high-dimensional matrix expansion. |
| **`forgeyard-cas`** | Custom in-memory peer tracking struct (`P2pCasSeeder`) | [`libp2p`](https://crates.io/crates/libp2p) | Enterprise-grade peer discovery, Kademlia DHT routing, and bit-torrent style swarm blob transfers. |
| **`forgeyard-scheduler`** | Manual static string metrics estimation for GPU VRAM & Tensor Cores | [`nvml-wrapper`](https://crates.io/crates/nvml-wrapper) | Real-time NVIDIA GPU VRAM profiling, CUDA core utilization, and temperature telemetry. |
| **`forgeyard-logs`** | Custom substring pattern matching and string replace loop | [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) & [`regex`](https://crates.io/crates/regex) | High-throughput zero-allocation log streaming, structured filtering, and regex-based redaction. |
| **`forgeyard-packaging`** | Subprocess invocation of `cargo-deb`, `cargo-wix`, and `cargo-bundle` binaries | [`msi`](https://crates.io/crates/msi) / [`deb-rs`](https://crates.io/crates/deb-rs) | Direct in-memory native package generation without requiring pre-installed CLI binaries. |
| **`forgeyard-deploy`** | Custom REST call implementations for GitHub Releases and SSH commands | [`octocrab`](https://crates.io/crates/octocrab) & [`russh`](https://crates.io/crates/russh) | Type-safe GitHub API client and pure-Rust async SSH/SFTP protocol driver. |
| **`forgeyard-device-lab`** | Subprocess invocation of `adb` command-line executable | [`mozdevice`](https://crates.io/crates/mozdevice) or [`adb_client`](https://crates.io/crates/adb_client) | Direct TCP protocol connection to Android Debug Bridge daemon for device automation. |
| **`forgeyard-test-report`** | Custom line-by-line text parser for Cargo test outputs | [`cargo_metadata`](https://crates.io/crates/cargo_metadata) & [`quick-xml`](https://crates.io/crates/quick-xml) | Structured JSON event stream parsing for Cargo test runs and high-speed JUnit XML parsing. |

---

## 🔍 Detailed Component Audit & Architectural Analysis

### 1. Cryptography, Secrets & Memory Protection (`forgeyard-secrets`)
- **Current Implementation**: `EncryptedVaultBackend` uses a manual XOR loop (`byte ^ key[i % 32]`) for secret encryption.
- **Problem**: XOR encryption is vulnerable to key reuse attacks and offers zero authenticity guarantees (tampering is undetectable).
- **Recommended Crate**: [`chacha20poly1305`](https://crates.io/crates/chacha20poly1305) or [`aes-gcm`](https://crates.io/crates/aes-gcm)
- **Use Case**: Encrypting sensitive pipeline secrets (tokens, SSH keys, registry credentials) with Authenticated Encryption with Associated Data (AEAD). Provides authenticated confidentiality where any tampered ciphertext is rejected automatically.

---

### 2. Post-Quantum Signatures & SLSA Provenance (`forgeyard-signing`)
- **Current Implementation**: `PostQuantumSigner` constructs mock signature strings (`format!("mldsa-87-{}", hex::encode(hash))`).
- **Problem**: The system claims post-quantum security guarantees without actual post-quantum cryptographic primitives.
- **Recommended Crate**: [`pqcrypto-mldsa`](https://crates.io/crates/pqcrypto-mldsa) (FIPS 204 ML-DSA) or [`pqcrypto-dilithium`](https://crates.io/crates/pqcrypto-dilithium)
- **Use Case**: Real quantum-resistant digital signatures for SLSA v1.0 attestations, ensuring build provenance signatures remain cryptographically valid even against future quantum decryption.

---

### 3. Git Repository Querying & Subprocess Removal (`forgeyard-provenance`)
- **Current Implementation**: `BasicProvenanceGenerator` executes `std::process::Command::new("git")` to retrieve HEAD commit hashes and remote origin URLs.
- **Problem**: Relying on external shell binaries introduces latency, depends on `git` being installed in the execution environment, and fails inside minimal container images.
- **Recommended Crate**: [`gix`](https://crates.io/crates/gix) (gitoxide) or [`git2`](https://crates.io/crates/git2)
- **Use Case**: Pure Rust git repository inspection. Queries commit hashes, branch heads, tags, and origin URLs directly from filesystem objects without invoking subprocesses.

---

### 4. Zero-Knowledge Proof Synthesizer (`forgeyard-provenance`)
- **Current Implementation**: `ZkProofGenerator` returns mock proof strings (`format!("zk-stark-{}", hash)`).
- **Problem**: Does not generate mathematical zero-knowledge proof traces.
- **Recommended Crate**: [`winterfell`](https://crates.io/crates/winterfell) (STARK prover) or [`sp1`](https://crates.io/crates/sp1) / [`risc0-zkvm`](https://crates.io/crates/risc0-zkvm)
- **Use Case**: Generating mathematical STARK execution proofs that verify a binary target was compiled from specific source code without revealing confidential source code files.

---

### 5. Dynamic Policy Gate Evaluation (`forgeyard-policy`)
- **Current Implementation**: `SecurityPolicy` uses manual `if command.contains("rm -rf")` string matching.
- **Problem**: Simple substring matching can be easily bypassed using shell escaping, variable expansion, or aliasing (e.g. `rm -r -f /` or `eval $CMD`).
- **Recommended Crate**: [`cedar-policy`](https://crates.io/crates/cedar-policy) (AWS Cedar) or [`rhai`](https://crates.io/crates/rhai)
- **Use Case**: Expressive, sandboxed security policy definitions. Allows security teams to write formal authorization and build compliance policies evaluated safely at runtime.

---

### 6. Linux Process Isolation & Sandboxing (`forgeyard-sandbox`)
- **Current Implementation**: Manual unsafe raw syscall invocations and manual unshare calls.
- **Problem**: Unsafe raw syscall blocks increase maintenance risk and potential undefined behavior across different kernel architectures.
- **Recommended Crate**: [`landlock`](https://crates.io/crates/landlock) & [`nix`](https://crates.io/crates/nix)
- **Use Case**: Safe, high-level Rust wrappers for Landlock filesystem unshare, Seccomp system call filtering, and Linux namespace isolation.

---

### 7. Edge AI Quantized Offline Remediation (`forgeyard-analyzer`)
- **Current Implementation**: `LocalEdgeAiEngine` generates hardcoded mock fix strings (`format!("Local Edge AI (GgufQ4) Fix Proposer...")`).
- **Problem**: Unable to perform genuine AI intent reasoning or generate real code patches offline.
- **Recommended Crate**: [`candle-core`](https://crates.io/crates/candle-core) & [`candle-transformers`](https://crates.io/crates/candle-transformers)
- **Use Case**: Native Rust LLM inference engine by HuggingFace. Loads local GGUF/ONNX quantized models (CodeLlama, Qwen, DeepSeek) for zero-latency, offline code fix generation.

---

### 8. DAG Dependency Graph & Matrix Resolution (`forgeyard-pipeline`)
- **Current Implementation**: Custom in-degree topological sorting algorithm in `dag.rs` and nested loops for matrix expansion in `matrix.rs`.
- **Problem**: Reinventing graph algorithms increases the risk of edge-case bugs (e.g., complex multi-cycle resolution or graph mutations).
- **Recommended Crate**: [`petgraph`](https://crates.io/crates/petgraph) & [`itertools`](https://crates.io/crates/itertools)
- **Use Case**: Utilizing `petgraph::graph::DiGraph` for DAG pipeline resolution and `itertools::multi_cartesian_product` for combinatorial build matrix expansion.

---

### 9. Distributed P2P CAS Swarm Seeding (`forgeyard-cas`)
- **Current Implementation**: Custom in-memory struct `P2pCasSeeder` maintaining peer node lists.
- **Problem**: Lacks real network protocol transport, peer discovery, or chunk re-transmission mechanisms.
- **Recommended Crate**: [`libp2p`](https://crates.io/crates/libp2p)
- **Use Case**: Standard peer-to-peer networking protocol supporting Kademlia DHT peer discovery, Noise protocol transport encryption, and bit-torrent style swarm artifact blob transfers across distributed edge runner nodes.

---

### 10. Hardware GPU Metric Profiling (`forgeyard-scheduler`)
- **Current Implementation**: Static score calculations without inspecting physical hardware metrics.
- **Problem**: Cannot make accurate work-stealing decisions based on actual VRAM availability or GPU temperature.
- **Recommended Crate**: [`nvml-wrapper`](https://crates.io/crates/nvml-wrapper)
- **Use Case**: Queries NVIDIA Management Library (NVML) for real-time GPU VRAM usage, CUDA core utilization, and memory pressure to optimize GPU task scheduling.

---

### 11. Multi-Cloud Deployment Publishers (`forgeyard-deploy`)
- **Current Implementation**: Subprocess calls to `ssh` / `scp` and custom HTTP requests for GitHub API endpoints.
- **Problem**: Prone to environment-specific binary mismatches and authentication failures.
- **Recommended Crate**: [`octocrab`](https://crates.io/crates/octocrab) & [`russh`](https://crates.io/crates/russh)
- **Use Case**: Asynchronous, strongly-typed API client for GitHub Releases and native SSH/SFTP protocol transport in Rust without external dependencies.

---

## 🛠️ Prioritized Refactoring & Migration Roadmap

### Phase 1: High-Priority Security & Infrastructure Wins (Immediate)
1. **Secrets Security**: Migrate `EncryptedVaultBackend` from XOR loop to [`chacha20poly1305`](https://crates.io/crates/chacha20poly1305).
2. **Git Query Engine**: Replace `std::process::Command::new("git")` in `forgeyard-provenance` with [`gix`](https://crates.io/crates/gix).
3. **Graph Resolution**: Standardize `forgeyard-pipeline` DAG scheduling on [`petgraph`](https://crates.io/crates/petgraph).

### Phase 2: Production Capabilities & Network Upgrades
1. **P2P CAS Swarm**: Replace `P2pCasSeeder` with [`libp2p`](https://crates.io/crates/libp2p) for real peer discovery and blob chunk streaming.
2. **Post-Quantum Signing**: Replace mock ML-DSA strings in `forgeyard-signing` with [`pqcrypto-mldsa`](https://crates.io/crates/pqcrypto-mldsa).
3. **Policy Engine**: Upgrade `forgeyard-policy` to evaluate formal [`cedar-policy`](https://crates.io/crates/cedar-policy) rules.

### Phase 3: Advanced Intelligence & Native Packaging
1. **Edge AI Runtime**: Integrate [`candle-core`](https://crates.io/crates/candle-core) into `forgeyard-analyzer` for offline GGUF LLM execution.
2. **Native Packaging**: Replace `cargo-deb`/`cargo-wix` subprocess invocations in `forgeyard-packaging` with direct library crates ([`msi`](https://crates.io/crates/msi), [`deb-rs`](https://crates.io/crates/deb-rs)).
3. **Hardware GPU Profiler**: Integrate [`nvml-wrapper`](https://crates.io/crates/nvml-wrapper) in `forgeyard-scheduler` for real-time VRAM allocation.

---

*Document compiled for Forgeyard architecture optimization and refactoring plan.*
