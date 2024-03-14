# Getting Started with Cistell

Welcome to Cistell, a comprehensive configuration management library for Python applications. This guide aims to help you swiftly install Cistell and set up a foundational configuration for your projects.

## Installation

You can install Cistell using pip by executing the following command:

```bash
pip install cistell
```

Once installed, Cistell is ready to be integrated into your Python projects.

## Quick Configuration Setup

Begin by defining a simple configuration class to understand Cistell's primary functionalities:

```python
from cistell import ConfigBase, ConfigField

class MyAppConfig(ConfigBase):
    database_url = ConfigField("sqlite:///example.db")
    feature_flag = ConfigField(False)
```

To apply these settings in your application, instantiate your configuration class:

```python
config = MyAppConfig()
print(config.database_url)  # Outputs: sqlite:///example.db
```

## Environment Variables and Configuration Files

Cistell allows configuration values to be overridden using environment variables and external files, enhancing flexibility across different environments:

- **Environment Variables**: Set environment variables corresponding to your configuration fields:

  ```shell
  export CONFIG__DATABASE_URL="sqlite:///prod.db"
  export CONFIG__FEATURE_FLAG=True
  ```

- **pyproject.toml Usage**: Define configurations in `pyproject.toml` under the `[tool.config]` section:

  ```toml
  [tool.config]
  database_url = "sqlite:///prod.db"
  feature_flag = true
  ```

Refer to the {doc}`../usage_guide/index` for detailed instructions on utilizing environment variables and configuration files with Cistell.

## Summary

After installing Cistell and setting up a basic configuration, explore further by integrating environment variables and configuration files to manage different environments effectively. Dive into advanced configurations, best practices, and various application scenarios in the {doc}`../usage_guide/index`. Understanding and implementing these practices will significantly enhance your application's flexibility and maintainability.
