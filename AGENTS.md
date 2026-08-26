# Forgeyard Workspace Rules for AI Agents

During software development, debugging, and code generation:

## Default Excluded Meta & Legal Files

Do **NOT** read, search, analyze, or process the following files unless explicitly requested by the user:

- `LICENSE` and `LICENSES/`
- `MAINTAINERS.md`
- `GOVERNANCE.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`
- `CLA.md`
- `CONTRIBUTING.md`
- `CHANGELOG.md`
- `RELEASES.md`
- `ROADMAP.md`
- Tooling metadata: `typos.toml`, `.editorconfig`, `.gitattributes`, `.dockerignore`

## Primary Focus Areas
Focus agent tools exclusively on code files, tests, Cargo manifests, and technical architecture docs (`crates/`, `apps/`, `ecosystems/`, `protocols/`, `schemas/`, `tests/`, `docs/`, `rfcs/`).
