import os
from abc import ABC, abstractmethod
from collections import defaultdict
from typing import Any, Iterator, Optional, Type

from cistell import defaults
from cistell.exceptions import ConfigMultiInheritanceError
from cistell.field import ConfigField


class ConfigRoot(ABC):
    """Root class for defining configuration settings."""

    TOML_CONFIG_ID: str = defaults.TOML_CONFIG_ID
    ENV_PREFIX: str = defaults.ENV_PREFIX
    ENV_SEP: str = defaults.ENV_SEPARATOR
    ENV_FILEPATH: str = defaults.ENV_FILEPATH
    IGNORE_CLASS_NAME_SUBSTR: str = defaults.IGNORE_CLASS_NAME_SUBSTR

    def __init__(
        self,
        config_values: Optional[dict[str, Any]] = None,
        config_filepath: Optional[str] = None,
    ) -> None:
        self.config_cls_to_fields: dict[str, set[str]] = defaultdict(set)
        _ = avoid_multi_inheritance_field_conflict(
            self.__class__, self.config_cls_to_fields
        )
        self._config_values: dict[ConfigField, Any] = {}

        # on the first run, we load defaults values specified in the mapping
        # afterwards, that values will be modified by the ancestors
        # the childs will have higher priority
        self._mapped_keys: set[str] = set()
        self.init_config_values(self.__class__, config_values, config_filepath)

    def get_env_key(
        self, field: str, config: Optional[Type["ConfigRoot"]] = None
    ) -> str:
        """gets the key used in the environment variables"""
        if config:
            return f"{self.ENV_PREFIX}{self.ENV_SEP}{config.__name__.upper()}{self.ENV_SEP}{field.upper()}"
        return f"{self.ENV_PREFIX}{self.ENV_SEP}{field.upper()}"

    @classmethod
    def config_fields(cls) -> list[str]:
        return list(get_config_fields(cls))

    @property
    def all_fields(self) -> list[str]:
        return list(set().union(*self.config_cls_to_fields.values()))

    def get_config_id(self, config_cls: Type["ConfigRoot"]) -> str:
        return config_cls.__name__.replace(self.IGNORE_CLASS_NAME_SUBSTR, "").lower()

    @abstractmethod
    def init_parent_values(
        self,
        config_cls: Type["ConfigRoot"],
        config_values: Optional[dict[str, Any]],
        config_filepath: Optional[str],
    ) -> None:
        # Initialize parent classes that are subclasses of ConfigRoot
        for parent in config_cls.__bases__:
            if issubclass(parent, ConfigRoot) and parent is not ConfigRoot:
                self.init_config_values(parent, config_values, config_filepath)

    @abstractmethod
    def init_config_values(
        self,
        config_cls: Type["ConfigRoot"],
        config_values: Optional[dict[str, Any]],
        config_filepath: Optional[str],
    ) -> None:
        """Initialize the configuration values."""
        del config_cls, config_values, config_filepath

    def init_config_value_from_mapping(
        self, source: str, config_id: str, mapping: dict[str, Any]
    ) -> None:
        conf_mapping = mapping.get(config_id, {})
        conf_mapping = conf_mapping if isinstance(conf_mapping, dict) else {}
        for key in self.config_cls_to_fields.get(self.__class__.__name__, []):
            self.init_config_value_key_from_mapping(
                source, config_id, key, mapping, conf_mapping
            )

    def init_config_value_key_from_mapping(
        self, source: str, config_id: str, key: str, mapping: dict, conf_mapping: dict
    ) -> None:
        general_key = f"{source}##{key}"
        class_key = f"{source}##{config_id}##{key}"
        if general_key not in self._mapped_keys and key in mapping:
            setattr(self, key, mapping[key])
            self._mapped_keys.add(general_key)
        if class_key not in self._mapped_keys and key in conf_mapping:
            setattr(self, key, conf_mapping[key])
            self._mapped_keys.add(class_key)

    def init_config_value_from_env_vars(self, config_cls: Type["ConfigRoot"]) -> None:
        for key in self.config_cls_to_fields.get(config_cls.__name__, []):
            if self.get_env_key(key, config_cls) in os.environ:
                setattr(self, key, os.environ[self.get_env_key(key, config_cls)])
            elif self.get_env_key(key) in os.environ:
                setattr(self, key, os.environ[self.get_env_key(key)])


def get_config_fields(cls: Type) -> Iterator[str]:
    for key, value in cls.__dict__.items():
        if isinstance(value, ConfigField):
            yield key


def avoid_multi_inheritance_field_conflict(
    config_cls: Type, config_cls_to_fields: dict[str, set[str]]
) -> dict[str, str]:
    """
    Ensures that the same configuration field is not defined in multiple parent classes of a given configuration class.

    This function checks all parent classes of the provided configuration class that are subclasses of `ConfigRoot`.
    It ensures that each configuration field is defined only once among all parent classes. If a field is found in
    multiple parent classes, a `ConfigMultiInheritanceError` is raised. This check ensures deterministic behavior
    in the configuration inheritance hierarchy.

    :param Type config_cls: The configuration class to check for field conflicts.
    :return: A dictionary mapping each configuration field to the name of the parent class where it is defined.
    :raises ConfigMultiInheritanceError: If a configuration field is found in multiple parent classes.

    :example:
    ```{code-block} python
        class ParentConfig1(ConfigRoot):
            field1 = ConfigField(default_value=1)
            ...
        class ParentConfig2(ConfigRoot):
            field2 = ConfigField(default_value=2)
            ...
        class ChildConfig(ParentConfig1, ParentConfig2):
            pass

        avoid_multi_inheritance_field_conflict(ChildConfig)
        # prings: {'field1': 'ParentConfig1', 'field2': 'ParentConfig2'}
    ```
    """
    map_field_to_config_cls: dict[str, str] = {}
    cls_fields: set[str] = set()
    for parent in config_cls.__bases__:
        if not issubclass(parent, ConfigRoot) or parent is ConfigRoot:
            continue
        for key in get_config_fields(parent):
            if key in map_field_to_config_cls:
                raise ConfigMultiInheritanceError(
                    f"ConfigField {key} found in parent classes {parent.__name__} and {map_field_to_config_cls[key]}"
                )
            map_field_to_config_cls[key] = parent.__name__
            config_cls_to_fields[parent.__name__].add(key)
        # add current parent ancestor's fields that may not be specified in the current class
        map_field_to_config_cls.update(
            avoid_multi_inheritance_field_conflict(parent, config_cls_to_fields)
        )
        cls_fields = cls_fields.union(config_cls_to_fields[parent.__name__])
    config_cls_to_fields[config_cls.__name__] = cls_fields.union(
        set(get_config_fields(config_cls))
    )
    return map_field_to_config_cls
