# Overview

## What Cistell Does

Cistell resolves configuration values from multiple sources (defaults, environment variables, config files, programmatic overrides) and tracks where each value came from. It was originally built for the [pynenc](https://github.com/pynenc/pynenc) distributed task framework and later extracted as a standalone library.

## Core Concepts

- **Hierarchical configuration** — local settings override global defaults through class inheritance.
- **Environment variable integration** — override any field via environment variables, following twelve-factor conventions.
- **File-based configuration** — supports JSON, YAML, and TOML config files.
- **Extensibility** — custom field mappers and validation logic can be plugged in.

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
