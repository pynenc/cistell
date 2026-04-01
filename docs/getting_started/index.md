# Getting Started

## Installation

```bash
pip install cistell
```

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

Refer to the {doc}`../usage_guide/index` for details on environment variables and configuration files.

## Next Steps

See the {doc}`../usage_guide/index` for advanced configuration patterns including multi-inheritance, custom mappers, and provenance tracking.
