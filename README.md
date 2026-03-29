<p align="center">
  <img src="https://raw.githubusercontent.com/pynenc/cistell/main/docs/_static/logo.png" alt="Cistell" width="300">
</p>
<h1 align="center">Cistell</h1>
<p align="center">
    <em>A powerful, transparent configuration library with provenance tracking and secret support for Rust and Python.</em>
</p>
<p align="center">
    <a href="https://crates.io/crates/cistell-core" target="_blank">
        <img src="https://img.shields.io/crates/v/cistell-core.svg?color=%23FFD21A" alt="crates.io">
    </a>
    <a href="https://pypi.org/project/cistell" target="_blank">
        <img src="https://img.shields.io/pypi/v/cistell?color=%2334D058&label=pypi" alt="PyPI">
    </a>
    <a href="https://github.com/pynenc/cistell/actions">
        <img src="https://github.com/pynenc/cistell/workflows/CI/badge.svg" alt="CI Status">
    </a>
    <a href="https://github.com/pynenc/cistell/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/pynenc/cistell" alt="GitHub license">
    </a>
</p>

---

## Overview

**Cistell** is a configuration resolution engine providing zero-boilerplate settings loading with robust accountability. Built heavily in Rust, it brings performance, type safety, and correctness across two ecosystems:

- **Rust (`cistell-core`, `cistell-macros`)**: Native library with a convenient `#[derive(Config)]` procedural macro.
- **Python (`cistell`)**: Performant bindings that serve as an idiomatic drop-in replacement for the original pure-Python cistell, extending its features with exact provenance APIs and fast resolution.

There's no more guessing where a config value came from. `cistell` tells you exactly how it resolved each field across 7 different possible priority layers, all while keeping secrets secure.

## Features & Highlights

- **Field Provenance Tracking**: Inquire exactly _where_ any configuration state was materialized from.
- **Priority Resolution Chain**: Values fall through an exact, strictly ordered 7-level priority chain.
- **Secret Management**: Built-in `Secret<T>` wrapper (Rust) and `secret=True` kwargs (Python) natively redact sensitive config properties across logs and display operations.
- **Zero-Boilerplate Derivation**: Simple attribute tagging configures fallback defaults and variable names.

## Quick Start (Rust)

Add `cistell-core` to your `Cargo.toml`:

```toml
[dependencies]
cistell-core = "0.1.0"
```

```rust
use cistell_core::{Config, Secret};

#[derive(Config, Debug)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    host: String,

    #[config(default = 8080)]
    port: u16,

    #[config(env = "API_KEY")]
    api_key: Secret<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ServerConfig::load()?;

    println!("Resolved Host: {}", cfg.host);
    // Explain the provenance
    println!("Provenance:\n{}", cistell_core::explain(&cfg));

    Ok(())
}
```

## Quick Start (Python)

Install via `pip` or `uv`:

```bash
pip install cistell
```

```python
from cistell import ConfigBase, ConfigField

class MySettings(ConfigBase):
    host = ConfigField("127.0.0.1")
    port = ConfigField(8080)
    api_key = ConfigField("default-key", secret=True)

cfg = MySettings()

print(cfg.host)
print(cfg.api_key) # prints 'default-key'

# Inspect how values were resolved
print(cfg.explain())

# Export a redacted immutable dictionary
print(cfg.safe_dict())
```

## Priority Chain

When resolving a field, Cistell checks the following layers in exact descending order of priority.

| Priority Level | Source                   | Description                                                                                                |
| -------------- | ------------------------ | ---------------------------------------------------------------------------------------------------------- |
| 1              | **Explicit/Overridden**  | Programmatically set at runtime via instantiation or context overrides.                                    |
| 2              | **Environment Variable** | Matches `PREFIX_FIELD_NAME` (or specifically configured `env` key).                                        |
| 3              | **Environment File**     | Value parsed statically from a `.env` file.                                                                |
| 4              | **Selected Config File** | Value loaded from a specifically pointed `config.toml`, `config.yaml`, or `.json`.                         |
| 5              | **pyproject.toml**       | Defaults pulled directly from Python packaging `[tool.yourapp]` namespace (if applicable).                 |
| 6              | **Default Config File**  | The default generic configuration file found in the working directory.                                     |
| 7              | **Hardcoded Defaults**   | Fallback static definition assigned via macro `#[config(default = ...)]` or python `ConfigField(default)`. |

## Feature Flags (Rust)

You can opt-in to parsing requirements in your Cargo.toml for `cistell-core`:

| Feature   | Default | Description                                 |
| --------- | ------- | ------------------------------------------- |
| `toml`    | ✅      | TOML file support                           |
| `yaml`    | ❌      | YAML file support                           |
| `json`    | ❌      | JSON file support                           |
| `serde`   | ❌      | Enable Serialize/Deserialize on `Secret<T>` |
| `zeroize` | ❌      | Enable zero-on-drop mapping for `Secret<T>` |

## Secret Fields

Keeping secrets out of crash reports and debug prints is critical.
By using `Secret<T>` (Rust) or `ConfigField(secret=True)` (Python), any underlying representation correctly implements traits or `__repr__` functions that output `"***"` rendering it harmless to standard printing vectors out of the box.

## Provenance

Need to figure out if your port was configured by `.env`, Docker env injection, or fell-back to the code default?

Use `cfg.explain()` to log the materialized source configuration on startup. Use `cfg.safe_dict()` to retrieve an inert representation of configuration states. Both functions intrinsically hide secret parameters gracefully.

## Python Compatibility

- **Minimum Version**: Python 3.12+
- **Dependencies**: No Python dependencies. All complex loading natively invokes Rust via `PyO3`.
- Available natively via pre-built wheels across Linux (manylinux), macOS (x86_64, arm64), and Windows.

## License

Cistell is distributed freely under the [MIT License](https://github.com/pynenc/cistell/blob/main/LICENSE).
CENSE).
