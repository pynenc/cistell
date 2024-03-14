# Welcome to Cistell's Documentation

**Cistell: Streamlined Configuration Management for Python Applications**

## Introduction

Cistell is a Python library designed to simplify configuration management for applications in diverse environments. It facilitates flexible and hierarchical configuration through an easy-to-use, class-based system. By leveraging Cistell, developers can define, access, and manage configurations seamlessly across files, environment variables, and direct settings, making it ideal for projects ranging from simple scripts to complex distributed systems.

```{toctree}
:hidden:
:maxdepth: 2
:caption: Table of Contents

overview
getting_started/index
usage_guide/index
apidocs/index.rst
contributing/index
faq
changelog
license
```

## Key Features

- Hierarchical and flexible configuration management
- Support for environment variables, configuration files (JSON, YAML, TOML), and direct assignment
- Easy integration with Python projects
- Advanced type checking and error handling for configuration values
- Extensible through custom field mappers and validators

For more details on these features, refer to the {doc}`usage_guide/index`.

## Installation

Cistell can be easily installed using pip:

```bash
pip install cistell
```

Refer to the {doc}`getting_started/index` section for more detailed installation instructions.

## Quick Start

To demonstrate the flexibility of Cistell, we'll create a simple configuration for a hypothetical library. Define a configuration class and set initial values through ConfigField:

```python
from cistell import ConfigBase, ConfigField

class MainConfig(ConfigBase):
    log_level = ConfigField("INFO")  # Default logging level
    port = ConfigField(8080)         # Default port for the library server
```

You can then override these defaults using environment variables or a configuration file:

1. Override using environment variables:

   ```bash
   export CONFIG__MAINCONFIG___LOG_LEVEL="DEBUG"
   export CONFIG__MAINCONFIG___PORT=9090
   ```

2. Override using pyproject.toml:

   ```toml
   [tool.config]
   log_level = "ERROR"
   port = 6060

   [tool.config.main]
   log_level = "WARNING"
   ```

In your application, load and use the configuration like this:

```python
config = LibraryConfig()
print(f"Log Level: {config.log_level}")
print(f"Server Port: {config.port}")
```

This setup allows you to maintain flexibility and scalability in your application's configuration. Explore more in the {doc}`getting_started/index` section.

## Contact or Support

Need help or want to discuss Cistell? Check out our [GitHub Issues](https://github.com/pynenc/cistell/issues) and [GitHub Discussions](https://github.com/pynenc/cistell/discussions).

## License

Cistell is released under the MIT License. For more information, see {doc}`license`.
