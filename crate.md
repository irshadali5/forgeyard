# 📜 Forgeyard License & Crate Compliance Audit (`crate.md`)

## ⚖️ Project License Architecture

Forgeyard is dual-licensed under:
1. **AGPL-3.0-or-later**: GNU Affero General Public License v3.0 (Open-Source Edition)
2. **Commercial License**: Enterprise Proprietary License for proprietary cloud deployments & OEM redistribution without AGPL copyleft obligations.

---

## 🚫 License Restriction & Enforcement Policy

To safeguard the commercial and open-source dual-license model of Forgeyard:
- ❌ **Forbidden Licenses**: Copyleft **GPL-2.0**, **GPL-3.0**, **LGPL-3.0** (unless dual-licensed under permissive MIT/Apache-2.0), and restrictive **Business Source Licenses** (**BSL 1.1**, **BUSL**, **SSPL**).
- ✅ **Permitted Licenses**: **AGPL-3.0-or-later**, **Apache-2.0**, **MIT**, **BSD-2-Clause**, **BSD-3-Clause**, **ISC**, **CC0-1.0**, **Unlicense**, and **Boost Software License 1.0 (BSL-1.0)**.

---

## 📦 Workspace Crates License Inventory

All **37 workspace crates** in the Forgeyard monorepo are governed strictly under the **AGPL-3.0-or-later OR Commercial** dual license:

| Crate Name | Version | License | Status |
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

## 🛡️ Dependency License Audit & Compatibility Matrix

All direct and transitive open-source dependencies have been audited via `cargo metadata`. Zero forbidden GPL/BSL crates exist in the dependency tree.

| Category / Subsystem | Primary External Crates | License(s) | Compliance Verification |
| :--- | :--- | :--- | :---: |
| **Cryptography & AEAD** | `chacha20poly1305`, `blake3`, `ed25519-dalek`, `sha2`, `hex` | `MIT OR Apache-2.0`, `CC0-1.0` | ✅ Permissive |
| **Graph & Dependency Engine** | `guppy`, `petgraph`, `itertools` | `MIT OR Apache-2.0` | ✅ Permissive |
| **Git Repository Inspection** | `gix` (gitoxide) | `MIT OR Apache-2.0` | ✅ Permissive |
| **System & Isolation** | `nix`, `io-uring`, `tempfile` | `MIT OR Apache-2.0` | ✅ Permissive |
| **Parsing & Utilities** | `quick-xml`, `regex`, `serde`, `serde_json`, `thiserror`, `tokio` | `MIT OR Apache-2.0` | ✅ Permissive |
| **Archive Builders** | `tar`, `flate2`, `zip` | `MIT OR Apache-2.0` | ✅ Permissive |

---

## 🔒 Automated License Gate Assertion

The Forgeyard policy gate validates that no non-compliant GPL or Business Source Licensed code enters the build pipeline. All crates comply 100% with the AGPLv3 / Commercial dual-license structure.
