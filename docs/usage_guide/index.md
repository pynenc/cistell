# Cistell Usage Guide

This guide is designed to offer comprehensive instructions and practical examples to help you leverage Cistell's configuration management in various scenarios. Whether you are new to Cistell or seeking to expand your knowledge, this guide aims to assist you through the configuration process effectively.

## Introduction to Cistell Configuration

Before diving into specific scenarios, ensure Cistell is properly installed and set up within your environment. Reference the {getting started}`../getting_started/index` section for initial setup instructions.

## Configuration Best Practices

```{important}
    Maintain a clean and hierarchical configuration structure. This practice helps in managing complex environments efficiently. Ensure that configurations are well-organized and avoid unnecessary duplications. Use environment variables for managing sensitive information such as API keys and database credentials.
```

## Prioritization of Configuration Sources

When using Cistell, the configuration sources are prioritized as follows:

1. Direct assignment in the config instance (programmatic overrides)
2. Environment variables
3. External configuration files (YAML, JSON, TOML) specified by path
4. Default values specified in the `ConfigField`

This prioritization ensures that the most specific setting is applied to your application, allowing for flexible and dynamic configuration management.

## Important Remark on Configuration Identifiers

When integrating environment variables and configuration files, it's crucial to understand the distinction in how Cistell identifies configuration classes:

- **Environment Variables**: Cistell defines environment variables using a combination of prefixe (default `CONFIG`), class names, and field names. All of them converted to uppercase. The full environment variable name combines these elements using a specific separator (e.g., `__`). For example, a setting host_name for a class named `LibraryConfigMain` would correspond to environment variables formatted as `CONFIG__LIBRARYCONFIGMAIN__HOST_NAME`.

- **Configuration Files**: File configurations (YAML, JSON, TOML), identify configuration classes using the class name converted to lowercase and without 'Config'. For instance, settings for `ConfigOther` and `ConfigMain` would be located under `config` and `main`, respectively, in a TOML file:

```toml
[tool.config]
host_name = "global config, will set any non specified value ConfigField"

[tool.config.other]
host_name = "specific value for ConfigOther.host_name"

[tool.config.main]
host_name = "specific value for ConfigMain.host_name"
```

```{note}
All these default values can be customize defining a new class that inherits from ConfigBase and specifies the class variables TOML_CONFIG_ID, ENV_PREFIX, ENV_SEP, ENV_FILEPATH and IGNORE_CLASS_NAME_SUBSTR
```

## Configuration Scenarios

Each scenario outlined in this guide represents common situations where Cistell can simplify and enhance your configuration management approach.

## Basic Usage

In this section, we demonstrate the foundational aspects of using Cistell for configuration management. At its core, Cistell allows for the easy definition of configuration parameters and provides a structured approach to access these settings within your application. This basic example serves as a stepping stone to more advanced features detailed in subsequent sections.

The following code illustrates how to define a simple configuration class using Cistell:

```python
from cistell import ConfigBase, ConfigField

class MyAppConfig(ConfigBase):
    # Define configuration fields with default values
    database_url = ConfigField("sqlite:///example.db")
    feature_flag = ConfigField(False)
```

Once you have defined your configuration class, you can instantiate and use it within your application:

```python
config = MyAppConfig()
print(config.database_url)  # Outputs: sqlite:///example.db
```

At this stage, the example might seem similar to using a module with constants due to the direct usage of default values. However, the true power of Cistell lies in its ability to override these default settings through various methods such as environment variables, configuration files, and direct assignments, which we will explore in the following sections. This initial setup establishes a structured framework for your application's configuration, paving the way for more dynamic and flexible configuration management.

## Managing Environment Overrides

Environment variables can be used to override configuration values. They follow two naming conventions:

1. `CONFIG__<CONFIG_CLASS_NAME>__<FIELD_NAME>` for setting values specific to a configuration class.
2. `CONFIG__<FIELD_NAME>` for default values that apply across all configuration classes.

**Example**:

```shell
   # Specific to a configuration class
   export CONFIG__CONFIGCHILD__TEST_FIELD="env_child_value"

   # Default value for all configuration classes
   export CONFIG__TEST_FIELD="env_default_value"
```

In the first example, `test_field` in `ConfigChild` is overridden with "env_child_value". In the second example, `test_field` is set to "env_default_value" for any configuration class that does not have a more specific value defined.

## Specifying Configuration File Path

A specific configuration file can be indicated using the `CONFIG__FILEPATH` environment variable. Additionally, a file exclusive to a particular `ConfigBase` instance can be specified, e.g., `CONFIG__SOMECONFIG__FILEPATH` for `SomeConfig`.

```{note}
   The configuration system is designed to be easily extendable, allowing users to create custom configuration classes that inherit from `ConfigBase`. This flexibility facilitates the modification of specific parts of the configuration as necessary for each system.
```

## Custom Field Mapping and Validation

`ConfigField` ensures that the type of the configuration value is preserved. Values from files or environment variables are cast to the specified type, and an exception is raised if casting is not possible.

Cistell allows for the customization of how configuration values are mapped and validated through the use of custom mappers. This is useful for scenarios where you need to enforce specific data formats, convert types, or apply custom validation logic before assigning the configuration value.

The following Python example illustrates how to define a custom mapper function and use it with a ConfigField:

```python
from typing import TypeVar, Type
from cistell import ConfigBase, ConfigField, default_config_field_mapper

T = TypeVar("T")

def other_mapper(value: Any, expected_type: Type[T]) -> T:
    """Custom mapper that converts tuples to a specific integer, otherwise uses the default mapper."""
    if isinstance(value, tuple):
        return -13  # Convert all tuples to -13
    return default_config_field_mapper(value, expected_type)

class ConfTest(ConfigBase):
    # Use the custom mapper for the 'cf' configuration field
    cf = ConfigField(0, mapper=other_mapper)
```

In this example, we define a custom mapper `other_mapper` that checks if the input value is a tuple. If so, it returns the integer `-13`; otherwise, it falls back to the default mapper provided by Cistell, `default_config_field_mapper`, which handles standard type conversions and validations.

We then use this custom mapper in a configuration class `ConfTest` for the field `cf`. Here’s how it works in practice:

```python
conf = ConfTest()

# Default value from the ConfigField definition
assert conf.cf == 0

# Standard mapping using the default mapper logic (string to int)
conf.cf = "1"
assert isinstance(conf.cf, int) and conf.cf == 1

# Custom mapping defined in 'other_mapper' (tuple to -13)
conf.cf = ("any_value", 8)
assert isinstance(conf.cf, int) and conf.cf == -13

# Attempting to set 'cf' to a list should raise a TypeError due to invalid type conversion
try:
    conf.cf = [0, "a"]
except TypeError:
    print("Caught expected TypeError.")
```

By employing custom field mappers, you can significantly enhance the robustness and flexibility of your configuration handling, ensuring that all configuration values meet your application’s specific requirements before they are utilized.

## Integrating Configuration Files

Cistell supports integrating external configuration files, allowing you to manage settings in familiar formats like YAML, JSON, and TOML. This flexibility enables seamless transitions between different environments and simplifies configuration management by externalizing parameters.

Here are examples of how you can integrate and prioritize configuration data from various sources using Cistell:

### YAML Configuration Files

YAML files are a popular choice for configuration due to their readability. Here's how you can use a YAML file to configure your application:

```python
# Define your configuration classes
class ConfigGrandpa(ConfigBase):
    test_field = ConfigField("grandpa_value")

class ConfigParent(ConfigGrandpa):
    test_field = ConfigField("parent_value")

class ConfigChild(ConfigParent):
    test_field = ConfigField("child_value")
```

YAML content representing your configuration:

```yaml
test_field: "yaml_value"
grandpa:
  test_field: "yaml_grandpa_value"
parent:
  test_field: "yaml_parent_value"
child:
  test_field: "yaml_child_value"
```

Load the YAML file in the configuration specifying it directly:

```python
config = ConfigChild(config_filepath=filepath)
assert config.test_field == "yaml_child_value"
```

Or using environment variables:

```bash
export CONFIG_FILEPATH=path/to/your/file.yaml
```

### JSON Configuration Files

JSON is another widely used format for configuration files. You can define your configuration similarly and load from a JSON file:

```json
{
  "test_field": "json_value",
  "grandpa": { "test_field": "json_grandpa_value" },
  "parent": { "test_field": "json_parent_value" },
  "child": { "test_field": "json_child_value" }
}
```

To use it with cistell specify the as the yaml file, it will use the appropiated parser based in the file extension

### TOML Configuration Files

TOML files are increasingly being used for configuration due to their clarity. Here’s how Cistell can load settings from a TOML file.

Just define variables in the pyproject.toml, cistell will pick the values from the tool.config and any config class by it's class name:

```toml
[tool.config]
value = "toml_value"

[tool.config.grandpa]
test_field = "toml_grandpa_value"

[tool.config.parent]
test_field = "toml_parent_value"

[tool.config.child]
test_field = "toml_child_value"
```

## Extending Configuration with Custom Classes

Users can extend the configuration system by creating custom configuration classes that inherit from `ConfigBase`. This flexibility allows for the easy modification of specific parts of the configuration as necessary for each system. For example, you can define a base configuration class for a library and extend it for specific functionalities or components.

Consider the following example where a base library configuration is extended for main and secondary configurations:

```python
from cistell import ConfigField, ConfigBase

class LibraryConfigBase(ConfigBase):
    TOML_CONFIG_ID = "other_id"
    ENV_PREFIX = "LIBCFG"
    ENV_SEP = "<->"
    ENV_FILEPATH = "CFGFILE"
    IGNORE_CLASS_NAME_SUBSTR = "LibraryConfig"

class LibraryConfigMain(LibraryConfigBase):
    value = ConfigField("default_main")

class Secondary(LibraryConfigBase):
    value = ConfigField(3)
```

In this example, `LibraryConfigBase` serves as the base class with common configuration settings. `LibraryConfigMain` and `Secondary` are subclasses that inherit from it and define or override their specific configurations.

Here are how different configurations can be applied:

1. **Default Values**: By default, the `LibraryConfigMain` and `Secondary` classes will use the values defined in their `ConfigField`s.

2. **Environment Variables**: Configuration values can be overridden using environment variables formatted according to the `ENV_PREFIX` and `ENV_SEP` defined in the base class.

   ```shell
   LIBCFG<->LIBRARYCONFIGMAIN<->VALUE="env_main"
   LIBCFG<->SECONDARY<->VALUE="4"
   ```

3. **Configuration Files**: A JSON configuration file can specify values for these fields. When the environment variable `LIBCFG<->CFGFILE` points to this file, the configuration system will load values from it.

   ```python
   # Assuming this content in 'lib_config.json':
   {
      "main": {"value": "file_main"},
      "secondary": {"value": 5}
   }
   ```

4. **Specific Environment Variables Over File Configurations**: If specific environment variables are set, they will override values from the configuration file.

This approach showcases the modularity and customization potential of Cistell, enabling a tailored configuration setup that meets various needs and scenarios.

```{note}
The configuration system ensures that the same configuration field is not defined in multiple parent classes, preventing conflicts and ensuring deterministic behavior.
```

## Leveraging Multi-Inheritance for Complex Configurations

Cistell supports multiple inheritance, allowing for the combination of configurations from different parent classes. This feature is particularly useful when different components of the system share common configuration options.

**Example**:

```python

   class ConfigOrchestrator(ConfigBase):
       ...

   class ConfigOrchestratorRedis(ConfigOrchestrator, ConfigRedis):
       ...
```

In this example, `ConfigOrchestratorRedis` combines the default configurations of both `ConfigOrchestrator` and `ConfigRedis`.

## Link to Real-world Example: Pynenc Configuration Module

Explore a real-world application of Cistell within the Pynenc project. The Pynenc configuration module, built using Cistell, showcases practical implementations and design patterns that can inspire and guide your configuration setup.

Visit [Pynenc Configuration Module](https://github.com/pynenc/pynenc/tree/main/pynenc/conf) for code examples and insights.

## Conclusion and Best Practices Summary

Leverage Cistell's full potential by adhering to configuration best practices, understanding the library's core principles, and applying the scenarios covered in this guide to your projects. Efficient configuration management is key to application stability and scalability.

Remember, the goal of Cistell is to provide a robust, flexible configuration framework that fits seamlessly into your Python projects, enhancing clarity, maintainability, and ease of use.
