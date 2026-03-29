# Cistell Documentation

**Cistell** is a configuration resolution library with a Rust core (`cistell-core`, `cistell-macros`) and Python bindings via PyO3. It was built to serve [pynenc](https://github.com/pynenc/pynenc) and [rustvello](https://github.com/pynenc/rustvello), and later generalised for standalone use.

---

::::{grid} 1 2 2 3
:gutter: 3

:::{grid-item-card} Rust Core
:link: overview
:link-type: doc

Configuration resolution engine. Use `cistell-core` and `cistell-macros` as native Rust crates.
:::

:::{grid-item-card} Python Bindings
:link: getting_started/index
:link-type: doc

Python integration via PyO3 and maturin. Drop-in replacement for the pure-Python API.
:::

:::{grid-item-card} Ecosystem
:link: overview
:link-type: doc

Designed to work with **rustvello** (Rust task engine) and **pynenc** (Python distributed task framework).
:::

::::

## Features

- **Multi-source resolution** — environment variables, config files (JSON, YAML, TOML), direct assignment, and Redis
- **Provenance tracking** — see where each configuration value came from
- **Hierarchical configs** — layered resolution with multi-inheritance support
- **Secret fields** — mark sensitive values to prevent accidental logging
- **Rust and Python** — use from either language with the same semantics

## Installation

::::{tab-set}

:::{tab-item} Python
```bash
pip install cistell
```
:::

:::{tab-item} Rust
```toml
[dependencies]
cistell-core = "0.1"
cistell-macros = "0.1"
```
:::

::::

## Quick Start

```python
from cistell import ConfigBase, ConfigField

class AppConfig(ConfigBase):
    log_level = ConfigField("INFO")
    port = ConfigField(8080)

config = AppConfig()
print(config.log_level)  # "INFO" (or overridden by env/file)
print(config.port)       # 8080
```

Override via environment variables:

```bash
export CONFIG__APPCONFIG___LOG_LEVEL="DEBUG"
export CONFIG__APPCONFIG___PORT=9090
```

See the {doc}`getting_started/index` for more examples.

```{toctree}
:hidden:
:maxdepth: 2
:caption: Table of Contents

overview
getting_started/index
usage_guide/index
migration
apidocs/index.rst
contributing/index
faq
changelog
license
```

## Support

- [GitHub Issues](https://github.com/pynenc/cistell/issues)
- [GitHub Discussions](https://github.com/pynenc/cistell/discussions)

## License

Cistell is released under the MIT License. See {doc}`license`.
