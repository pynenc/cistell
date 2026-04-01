# Changelog

For detailed information on each version, please visit the [Cistell GitHub Releases page](https://github.com/pynenc/cistell/releases).

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
- Pre-commit hooks: cargo-fmt, cargo-clippy, ruff, mypy, commitlint, typos, markdownlint, prettier
- Makefile: install, check, build, test, publish-rust, publish-python, docs targets
- Docs: Sphinx + furo theme with grid cards, Rust/Python dual-ecosystem messaging
- Added dependabot config (github-actions, cargo, pip)

### Removed

- `typing-extensions` dependency (Python 3.12 has native equivalents)
- `pyyaml` Python dependency (parsing in Rust)

## 0.0.5

- LRU cache of loaded files to avoid OsError in some environments

## 0.0.4

- ConfigRoot.get_env_key is now a class method instead of instance

## 0.0.3

- Fix issue with customization classes, should not be considered.
  Cistell will ignore any class that inherits from ConfigBase and has no fields
  Will only be considered for the class attributes

## 0.0.2

- First release to pypi

## 0.0.1

- Refactoring from Pynenc original configuration module
- Adding tests
- Workflow actions for cistell package
- Initial documentation
