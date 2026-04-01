# Changelog

## 0.1.0 (unreleased)

### Added

- Rust core library (`cistell-core`) with configuration resolution engine
- `#[derive(Config)]` proc macro for zero-boilerplate config structs
- Field provenance tracking — know where every config value came from
- `Secret<T>` wrapper for sensitive fields — never leaked in logs or debug output
- 7-level priority chain: programmatic > env var > env-file > file > pyproject.toml > default
- TOML, YAML (opt-in), JSON (opt-in) file support
- Python bindings via PyO3/maturin — drop-in replacement for Python cistell
- `ConfigBase.explain()`, `.safe_dict()`, `.field_info()` provenance API
- `ConfigBase.override()` context manager for test isolation

### Changed

- Python minimum version: 3.12 (was 3.11.6)
- Build system: maturin + uv (was Poetry)
- YAML/JSON parsing now in Rust (no Python `pyyaml` dependency)

### Infrastructure

- CI/CD: GitHub Actions aligned with rustvello/pynenc conventions
  - `main.yml`: quality, tests (matrix: ubuntu/macos × py3.12/3.13), docs, publish dry-run
  - `release-rust.yml`: publish cistell-macros and cistell-core to crates.io
  - `release-python.yml`: multi-platform wheels (Linux x86_64/aarch64, macOS x86_64/arm64, Windows)
  - `labeler.yml`: auto-label PRs with conventional commit and semver labels
  - `release-drafter.yml`: auto-generated release notes
  - `pr-title-checker.yml`: enforce conventional commit PR titles
  - `publish-release-notes.yml`: auto-publish release notes and update CHANGELOG
  - `smokeshow.yml`: coverage report hosting
- Pre-commit hooks: cargo-fmt, cargo-clippy, ruff, mypy, commitlint, typos, markdownlint, prettier
- Makefile: install, check, build, test, publish-rust, publish-python, docs targets
- Docs: Sphinx + furo theme with grid cards, Rust/Python dual-ecosystem messaging
- Added dependabot config (github-actions, cargo, pip)
- Added PR template, issue templates (bug report, feature request, documentation)

### Removed

- `typing-extensions` dependency (Python 3.12 has native equivalents)
- `pyyaml` Python dependency (parsing in Rust)
