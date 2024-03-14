import os
from typing import Any, Optional, Type

from cistell import util_files
from cistell.root import ConfigRoot


class ConfigBase(ConfigRoot):
    """
    Base class for defining configuration settings.

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
        config_cls: Type["ConfigRoot"],
        config_values: Optional[dict[str, Any]],
        config_filepath: Optional[str],
    ) -> None:
        # Initialize parent classes that are subclasses of ConfigBase
        for parent in config_cls.__bases__:
            if issubclass(parent, ConfigBase) and parent not in (
                ConfigBase,
                ConfigRoot,
            ):
                self.init_config_values(parent, config_values, config_filepath)

    def init_config_values(
        self,
        config_cls: Type["ConfigRoot"],
        config_values: Optional[dict[str, Any]],
        config_filepath: Optional[str],
    ) -> None:
        config_id = self.get_config_id(config_cls)
        self.init_parent_values(config_cls, config_values, config_filepath)
        # 5.- User specifies the config by values (dict[str: Any])
        if config_values:
            self.init_config_value_from_mapping(
                "config_values", config_id, config_values
            )
        # 4.- User specifies config values in pyproject.toml
        if os.path.isfile("pyproject.toml"):
            self.init_config_value_from_mapping(
                "pyproject.toml",
                config_id,
                util_files.load_file(self.TOML_CONFIG_ID, "pyproject.toml"),
            )
        # 3.- User specifies the config filepath(ref to a yml, toml or json…)
        if config_filepath:
            self.init_config_value_from_mapping(
                "config_filepath",
                config_id,
                util_files.load_file(self.TOML_CONFIG_ID, config_filepath),
            )
        # 2.- User specifies the location of the config file by env vars
        # 2.1 Global config filepath specify by env var
        if filepath := os.environ.get(self.get_env_key(self.ENV_FILEPATH)):
            self.init_config_value_from_mapping(
                "ENV_FILEPATH",
                config_id,
                util_files.load_file(self.TOML_CONFIG_ID, filepath),
            )
        # 2.2 Specific class config filepath specify by env var
        if filepath := os.environ.get(self.get_env_key(self.ENV_FILEPATH, config_cls)):
            self.init_config_value_from_mapping(
                "ENV_CLASS_FILEPATH",
                config_id,
                util_files.load_file(self.TOML_CONFIG_ID, filepath),
            )
        # 1.- User specifies environment variables
        self.init_config_value_from_env_vars(config_cls)
