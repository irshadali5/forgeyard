# 📜 Forgeyard License & Dependency Compliance Policy (`crate.md`)

## ⚖️ Dual-Licensing Architecture & Copyright Ownership

Forgeyard uses a **Dual-Licensing Model**:

1. **Open-Source Edition**: Licensed under **GNU AGPLv3** (`AGPL-3.0-or-later`).
2. **Commercial Edition**: Proprietary Commercial License for enterprise customers requiring closed-source deployment, OEM embedding, or SaaS redistribution without AGPL copyleft obligations.

> [!IMPORTANT]
> **Why Third-Party AGPL / GPL Dependencies are Strictly Forbidden**:
> Copyright ownership of Forgeyard code belongs to the project maintainers, enabling dual-licensing under both AGPLv3 and Commercial terms. However, if Forgeyard incorporates **third-party** AGPL or GPL dependencies, those external authors' copyleft terms would infect the codebase, making it legally impossible to grant closed-source Commercial licenses to enterprise buyers.

---

## 🚫 Third-Party Dependency License Rules

To guarantee that commercial customers receive un-contaminated proprietary rights, all **third-party external crates** in `Cargo.toml` / `Cargo.lock` are governed by strict license gates:

### ❌ Forbidden Third-Party Licenses

- **AGPL-1.0 / AGPL-3.0** (Prevents commercial proprietary re-licensing)
- **GPL-1.0 / GPL-2.0 / GPL-3.0** (Strong copyleft contamination)
- **LGPL-2.1 / LGPL-3.0** (Unless dual-licensed under MIT/Apache-2.0)
- **BSL 1.1 / BUSL / SSPL** (Restrictive Business Source Licenses)

### ✅ Allowed Third-Party Dependency Licenses

Only permissive open-source licenses (or dual-licensed crates with a permissive option) are permitted for external crates:

- **Apache-2.0**
- **MIT**
- **BSD-2-Clause / BSD-3-Clause**
- **ISC**
- **CC0-1.0 / Unlicense**
- **Boost Software License 1.0 (BSL-1.0)** (Permissive Boost C++ license)

---

## 📦 Workspace Crates License Table (First-Party)

All **37 first-party workspace crates** in Forgeyard are copyright-owned and dual-licensed under `AGPL-3.0-or-later OR Commercial`:

| Crate Name | Version | First-Party License | Status |
| :--- | :--- | :--- | :---: |
| `forgeyard-model` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-config` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-cas` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-storage` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-pipeline` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-executor` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-runner` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-daemon` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-cli` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-scheduler` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-adapter-cargo` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-sandbox` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-archive` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-protocol` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-agent` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-ui` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-toolchains` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-deploy` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-secrets` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-signing` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-api` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-cache` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-events` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-artifacts` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-provenance` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-policy` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-analyzer` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-test-report` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-detector` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-device-lab` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-packaging` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-logs` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-adapter-oci` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-adapter-wasm` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-adapter-android` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-adapter-xcode` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |
| `forgeyard-adapter-dioxus` | 0.1.0 | `AGPL-3.0-or-later OR Commercial` | ✅ Compliant |

---

## 🔍 Third-Party Dependency Audit Verification

Automated inspection via `cargo metadata` verifies:

- **0 Third-Party AGPL / GPL Crates**
- **0 Restrictive BSL 1.1 / BUSL / SSPL Crates**
- **100% Permissive Dependency Tree** (`Apache-2.0`, `MIT`, `BSD`, `ISC`, `CC0-1.0`)

Commercial customers purchasing an enterprise license receive a clean, un-contaminated proprietary distribution.
