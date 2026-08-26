# Workspace Rule: Exclude Meta & Administrative Files During Active Development

During active feature development, debugging, refactoring, and code review, **DO NOT** read, analyze, search, or index project meta/administrative files unless the user explicitly mentions or requests them.

## Files and Directories to Exclude by Default

Unless explicitly requested by the user, ignore the following meta, governance, legal, and release management files:

1. **Licensing & Legal Files**:
   - `LICENSE`
   - `LICENSES/` (and all files inside)
   - `CLA.md`

2. **Governance & Community Management**:
   - `MAINTAINERS.md`
   - `GOVERNANCE.md`
   - `CODE_OF_CONDUCT.md`
   - `SECURITY.md`
   - `CONTRIBUTING.md`

3. **Release & Changelog Documents**:
   - `CHANGELOG.md`
   - `RELEASES.md`
   - `ROADMAP.md`

4. **Static Tooling / Metadata Configs** (unless debugging tooling configuration):
   - `typos.toml`
   - `.editorconfig`
   - `.gitattributes`
   - `.dockerignore`

---

## Operating Guidelines for AI Agents

1. **Focus on Codebase**: Focus search, analysis, and reading on source code (`crates/`, `apps/`, `ecosystems/`, `protocols/`, `schemas/`), tests, configuration schemas, and active technical documentation (`docs/`, `rfcs/`).
2. **Explicit Override**: Only view or modify the excluded meta files above when the user explicitly asks to edit, review, or discuss licensing, governance, contribution rules, changelogs, or maintainers.
