# Forgeyard Project Governance

This document describes the governance structure and decision-making processes for the **Forgeyard** open-source project.

---

## 1. Governance Model: BDFL + Core Steering Committee

Forgeyard follows a **Benevolent Dictator for Life (BDFL) + Core Steering Committee** governance model:

* **Lead Maintainer / BDFL**: Has final authority on architectural direction, release approval, breaking changes, license enforcement, and commercial agreements.
* **Core Steering Committee**: Composed of maintainers responsible for domain areas (runner engines, CAS infrastructure, language ecosystems, protocol definitions).

---

## 2. Decision Making Process

1. **Consensus-Seeking**: Technical decisions are discussed openly in GitHub Issues, Pull Requests, and RFC proposals (`rfcs/`). Maintainers strive for consensus.
2. **RFC Requirement**: Major architectural changes, wire protocol updates, or new top-level crates require an RFC proposal passed with approval from at least two core maintainers.
3. **Tie-Breaking**: If consensus cannot be reached after thorough discussion, the BDFL makes the final determination.

---

## 3. Commercial & Dual-Licensing Governance

* All open-source developments in `crates/*` remain permissive under **MIT OR Apache-2.0**.
* Platform binaries in `apps/*` remain strong copyleft under **GNU AGPLv3**.
* Commercial AGPL exemptions, enterprise support SLAs, and custom managed deployments are governed exclusively by the Core Maintainers to fund ongoing engineering and open-source development.
