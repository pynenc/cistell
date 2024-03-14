<p align="center">
  <img src="https://raw.githubusercontent.com/pynenc/cistell/main/docs/_static/logo.png" alt="Cistell" width="300">
</p>
<h1 align="center">Cistell</h1>
<p align="center">
    <em>A comprehensive configuration management library for Python applications</em>
</p>
<p align="center">
    <a href="https://pypi.org/project/cistell" target="_blank">
        <img src="https://img.shields.io/pypi/v/cistell?color=%2334D058&label=pypi%20package" alt="Package version">
    </a>
    <a href="https://pypi.org/project/cistell" target="_blank">
        <img src="https://img.shields.io/pypi/pyversions/cistell.svg?color=%2334D058" alt="Supported Python versions">
    </a>
    <a href="https://github.com/pynenc/cistell/commits/main">
        <img src="https://img.shields.io/github/last-commit/pynenc/cistell" alt="GitHub last commit">
    </a>
    <a href="https://github.com/pynenc/cistell/graphs/contributors">
        <img src="https://img.shields.io/github/contributors/pynenc/cistell" alt="GitHub contributors">
    </a>
    <a href="https://github.com/pynenc/cistell/issues">
        <img src="https://img.shields.io/github/issues/pynenc/cistell" alt="GitHub issues">
    </a>
    <a href="https://github.com/pynenc/cistell/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/pynenc/cistell" alt="GitHub license">
    </a>
    <a href="https://github.com/pynenc/cistell/stargazers">
        <img src="https://img.shields.io/github/stars/pynenc/cistell?style=social" alt="GitHub Repo stars">
    </a>
    <a href="https://github.com/pynenc/cistell/network/members">
        <img src="https://img.shields.io/github/forks/pynenc/cistell?style=social" alt="GitHub forks">
    </a>
</p>

---

**Documentation**: <a href="https://docs.cistell.pynenc.org" target="_blank">https://docs.cistell.pynenc.org</a>

**Source Code**: <a href="https://github.com/pynenc/cistell" target="_blank">https://github.com/pynenc/cistell</a>

---

Cistell is a powerful library aimed at simplifying configuration management for Python projects. It allows developers to manage settings across different environments seamlessly, using a structured yet flexible approach. With support for environment variables, configuration files, and direct settings, Cistell makes it easy to maintain a clear and consistent configuration strategy for any Python application.

## Key Features

- **Flexible Configuration**: Supports various sources, including environment variables, configuration files (JSON, YAML, TOML), and direct assignment.
- **Hierarchical Configuration**: Allows for layered configuration strategies, accommodating different environments and scenarios.
- **Type Checking and Error Handling**: Offers robust type checking and error handling for secure and reliable configuration management.
- **Extensible Framework**: Easily extendable with custom field mappers and validators to fit unique project needs.

## Installation

Cistell can be quickly installed using pip. Run the following command in your terminal:

```bash
pip install cistell
```

This command will download and install Cistell along with its dependencies. Once installed, Cistell is ready to enhance your Python project with efficient configuration management.

For further installation details and advanced options, refer to the [Cistell Documentation](https://docs.cistell.pynenc.org/).

## Quick Start Example

Here's a basic example to get you started with Cistell:

1. **Define a Configuration Class**:

   Create a file named `config.py` and define your configuration settings:

   ```python
   from cistell import ConfigBase, ConfigField

   class MyAppConfig(ConfigBase):
       database_url = ConfigField("sqlite:///example.db")
       feature_flag = ConfigField(False)
   ```

2. **Instantiate and Use Your Configuration**:

   In your application, create an instance of your configuration class and use the settings:

   ```python
   from config import MyAppConfig

   config = MyAppConfig()
   print(config.database_url)  # Outputs: sqlite:///example.db
   ```

For detailed examples and further setup, visit the [Cistell Quick Start Guide](https://docs.cistell.pynenc.org/getting_started/index).

## Requirements

Cistell requires Python 3.7 or newer. Additional requirements may vary based on your setup and the features you use.

## Contact or Support

For assistance with Cistell, suggestions, or general discussion, please use the following resources:

- **[GitHub Issues](https://github.com/pynenc/cistell/issues)**: For bug reports, feature requests, or other technical concerns.
- **[GitHub Discussions](https://github.com/pynenc/cistell/discussions)**: For questions, ideas exchange, or discussions about Cistell usage and development.

Your contributions and feedback are invaluable in helping Cistell improve and evolve.

## License

Cistell is available under the [MIT License](https://github.com/pynenc/cistell/blob/main/LICENSE).
