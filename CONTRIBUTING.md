# Contributing to Forgeyard

Welcome to the **Forgeyard** developer community! Forgeyard is an open-source, hermetic, polyglot CI/CD, build orchestration, and software delivery platform built in Rust.

We welcome contributions of all kinds—bug fixes, performance enhancements, documentation improvements, test suites, architecture proposals, and new ecosystem adapters.

---

## 1. Governance & Licensing Philosophy

Forgeyard operates under a **Dual-Licensing Strategy** to ensure software freedom for the community while funding long-term engineering development:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Forgeyard Repository                              │
├──────────────────────────────────────┬──────────────────────────────────────┤
│  Shared Libraries & Ecosystem Crates │  Applications, Servers & Binaries   │
│  (`crates/*`, `ecosystems/*`, etc.)  │  (`apps/*`, `deploy/*`, `infra/*`)   │
│                                      │                                      │
│  License: MIT OR Apache-2.0          │  License: GNU AGPLv3                 │
│  (Permissive for open & closed code) │  (Strong Copyleft & Network Disclosure)│
└──────────────────────────────────────┴──────────────────────────────────────┘
```

### Contributor License Agreement (CLA) & DCO Requirement
All pull requests require a commit sign-off complying with our **[Contributor License Agreement (`CLA.md`)](file:///home/irshad/Projects/forgeyard/CLA.md)**.

To sign off on your commits automatically, use the `-s` flag:
```bash
git commit -s -m "feat(runner): implement isolated sandbox environment"
```

---

## 2. Setting Up Your Local Environment

### Toolchain Requirements
* **Rust**: `1.85+` (installed via `rustup`).
* **Cargo Tools**: `cargo-clippy`, `rustfmt`, `cargo-deny`, `cargo-xtask`.
* **Container Runtime** (Optional): Docker or Podman (for running container runner integration tests).

### Quickstart Setup
```bash
# Clone the repository
git clone https://github.com/forgeyard/forgeyard.git
cd forgeyard

# Verify toolchain and environment
cargo --version
rustup component add clippy rustfmt

# Run workspace check
cargo check --workspace --all-targets
```

---

## 3. Recommended Development Workflow

### Workspace Commands

| Task | Command | Description |
| :--- | :--- | :--- |
| **Check Compilation** | `cargo check --workspace --all-targets` | Validates typechecking across all crates and binaries |
| **Format Code** | `cargo fmt --all` | Enforces standard Rust formatting rules |
| **Run Linter** | `cargo clippy --workspace --all-targets -- -D warnings` | Runs strict clippy linter checks |
| **Run Unit Tests** | `cargo test --workspace` | Runs all workspace unit and doc tests |
| **Run Integration Tests** | `cargo test --test '*' -- --nocapture` | Runs workspace integration test suites |

---

## 4. Git Commit Guidelines & Conventional Commits

We follow the **[Conventional Commits](https://www.conventionalcommits.org/)** specification for clean commit history and automated release changelog generation.

### Commit Format:
```text
<type>(<scope>): <short summary>

[optional body description]

Signed-off-by: Your Name <your.email@example.com>
```

### Supported Types:
* `feat`: A new user-facing feature or API capability.
* `fix`: A bug fix or runtime patch.
* `docs`: Documentation, README, or RFC updates.
* `refactor`: Code reorganization without functional changes.
* `perf`: Performance improvement or optimization.
* `test`: Adding or updating test suites.
* `ci`: Infrastructure, release, or GitHub Actions pipeline changes.
* `chore`: Maintenance, dependencies, or toolchain updates.

### Example Commit:
```bash
git commit -s -m "fix(cas): resolve race condition in parallel blob ingestion"
```

---

## 5. Submitting a Pull Request (PR)

1. **Fork & Branch**: Fork the repo and create a focused topic branch from `main`:
   ```bash
   git checkout -b feat/custom-runner-adapter
   ```
2. **Implement & Test**: Write your code, add corresponding unit/integration tests, and format:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
3. **Commit with Sign-Off**: Ensure every commit has the DCO sign-off (`git commit -s`).
4. **Push & Create PR**: Open a Pull Request on GitHub against `main`. Provide a detailed description of the changes, architectural decisions, and issue references.
5. **CI & Review**: Respond to automated CI feedback and maintainer code reviews promptly.

---

## 6. Architecture RFCs & Proposal Process

For major architectural changes, crate additions, wire protocol revisions, or breaking schema changes, please submit an **RFC (Request for Comments)** in the `rfcs/` directory prior to writing code.

---

## 7. Code of Conduct & Community Guidelines

We are committed to providing a welcoming, inclusive, and harassment-free community. Please read and abide by our **[`CODE_OF_CONDUCT.md`](file:///home/irshad/Projects/forgeyard/CODE_OF_CONDUCT.md)** in all project channels.
