"""Concrete configuration base class with full source resolution."""

import os
import pathlib

from typing import Any

from cistell._internal import load_config_file, resolve_field
from cistell.root import ConfigRoot


class ConfigBase(ConfigRoot):
    """Base class for defining configuration settings.

    This class serves as the base for creating configuration classes. It supports
    hierarchical and flexible configuration from various sources, including
    environment variables, configuration files, and default values.

    :param Optional[dict[str, Any]] config_values:
        A dictionary of configuration values to use.
    :param Optional[str] config_filepath:
        The path to a configuration file to use.

    Configuration values are determined based on the following priority (highest to lowest):
    1. Direct assignment in the config instance (not recommended)
    2. Environment variables
    3. Configuration file path specified by environment variables
    4. Configuration file path (YAML, TOML, JSON) by config_filepath parameter
    5. `pyproject.toml`
    6. Default values specified in the `ConfigField`
    7. Previous steps for any Parent config class
    8. User does not specify anything (default values)

    ## Examples
    Define a configuration class for a Redis client:

    ```{code-block} python
        class ConfigRedis(ConfigRoot):
            redis_host = ConfigField("localhost")
            redis_port = ConfigField(6379)
            redis_db = ConfigField(0)
    ```

    Define a main configuration class for orchestrator components:

    ```{code-block} python
        class ConfigOrchestrator(ConfigRoot):
            cycle_control = ConfigField(True)
            blocking_control = ConfigField(True)
            auto_final_invocation_purge_hours = ConfigField(24.0)
    ```

    Combine configurations using multiple inheritance:

    ```{code-block} python
        class ConfigOrchestratorRedis(ConfigOrchestrator, ConfigRedis):
            pass
    ```

    The `ConfigOrchestratorRedis` class now includes settings from both `ConfigOrchestrator`
    and `ConfigRedis`.
    """

    def init_parent_values(
        self,
        config_cls: type["ConfigRoot"],
        config_values: dict[str, Any] | None,
        config_filepath: str | None,
    ) -> None:
        """Initialize values from parent configuration classes."""
        # Initialize parent classes that are subclasses of ConfigBase
        for parent in config_cls.__bases__:
            if issubclass(parent, ConfigBase) and parent not in (
                ConfigBase,
                ConfigRoot,
            ):
                if not parent.config_fields() and ConfigBase in parent.__bases__:
                    # Skip this parent as it's just for customization without fields
                    continue
                self.init_config_values(parent, config_values, config_filepath)

    def init_config_values(
        self,
        config_cls: type["ConfigRoot"],
        config_values: dict[str, Any] | None,
        config_filepath: str | None,
    ) -> None:
        """Initialize configuration values for a specific class."""
        config_id = self.get_config_id(config_cls)
        self.init_parent_values(config_cls, config_values, config_filepath)

        # Build mapping sources: lowest → highest priority
        mappings: list[tuple[str, Any]] = []
        if config_values:
            mappings.append(("config_values", config_values))
        if pathlib.Path("pyproject.toml").is_file():
            mappings.append((
                "pyproject.toml",
                load_config_file("pyproject.toml", self.TOML_CONFIG_ID),
            ))
        if config_filepath:
            mappings.append((
                "config_filepath",
                load_config_file(config_filepath, self.TOML_CONFIG_ID),
            ))
        if filepath := os.environ.get(self.get_env_key(self.ENV_FILEPATH)):
            mappings.append((
                "ENV_FILEPATH",
                load_config_file(filepath, self.TOML_CONFIG_ID),
            ))
        if filepath := os.environ.get(self.get_env_key(self.ENV_FILEPATH, config_cls)):
            mappings.append((
                "ENV_CLASS_FILEPATH",
                load_config_file(filepath, self.TOML_CONFIG_ID),
            ))

        # Resolve each field via Rust
        for field_name in self.config_cls_to_fields.get(config_cls.__name__, set()):
            field = self._get_field_descriptor(field_name)
            if field is None:
                continue
            result = resolve_field(
                field_name,
                config_id,
                self.get_env_key(field_name, config_cls),
                self.get_env_key(field_name),
                secret=field._secret,
                mappings=mappings,
                mapped_keys=self._mapped_keys,
            )
            if result is not None:
                value, prov = result
                value = field._mapper(value, type(field._default_value))
                self._config_values[field] = value
                self._provenance[field_name] = prov.source
