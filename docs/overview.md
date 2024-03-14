# Overview

## Cistell: Streamlined Configuration Management for Python Projects

Cistell is designed to simplify configuration management across various environments in Python applications, particularly emphasizing ease of integration and adaptability. This section provides a brief overview of the essential features of Cistell.

## Core Features and Design Philosophy

Cistell is developed with a focus on offering an intuitive and user-friendly interface for managing application settings, without sacrificing the flexibility needed for complex environments:

- **Hierarchical Configuration**: Enables layered configuration strategies that allow local settings to seamlessly override global defaults.

- **Environment Variable Integration**: Facilitates an intuitive method for overriding configuration values with environment variables, in alignment with the principles of twelve-factor applications.

- **File-based Configuration Support**: Provides support for multiple file formats, including JSON, YAML, and TOML, to accommodate a range of project requirements and preferences.

- **Modularity and Customization**: Built with extensibility in mind, enabling developers to implement custom field mappers and validation logic to suit their specific needs.

## Leveraging Multi-Inheritance for Flexible Configuration

Cistell supports multi-inheritance, allowing you to define nuanced, hierarchical configurations for different components of your application. This feature is particularly useful in scenarios where you may have common settings that apply across various parts of your application but also need specialized configurations for certain components.

### Example: Redis Configuration

Imagine you have an application that uses Redis in multiple capacities: as a database for one component and as a cache for another. You can define a base Redis configuration class and then extend it for specific use cases:

```python
from cistell import ConfigBase, ConfigField

class RedisConfig(ConfigBase):
    host = ConfigField("localhost")
    port = ConfigField(6379)

class RedisOrchestratorConfig(RedisConfig):
    # Additional or overridden settings specific to the orchestrator
    db_index = ConfigField(1)

class RedisCacheConfig(RedisConfig):
    # Different settings for the caching mechanism
    db_index = ConfigField(2)
    timeout = ConfigField(300)
```

In your pyproject.toml, you can define configurations applicable to all Redis instances or specific to the orchestrator or cache components:

```python
[tool.config.redis]
host = "redis.local"
port = 6380

[tool.config.redis.orchestrator]
db_index = 1

[tool.config.redis.cache]
db_index = 2
timeout = 600
```

With Cistell, the configurations are resolved through the multi-inheritance hierarchy, ensuring that each component receives the correct settings, whether they are shared across all instances or specific to a particular role.

By leveraging multi-inheritance, Cistell enables you to maintain clear and organized configuration structures, making your application's configuration logic both scalable and easy to manage.
