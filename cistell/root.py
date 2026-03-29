"""Core configuration root class and multi-inheritance conflict resolution."""

from abc import ABC, abstractmethod
from collections import defaultdict
from collections.abc import Iterator
from types import MappingProxyType
from typing import Any

from cistell import defaults
from cistell._internal import FieldProvenance
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
        config_values: dict[str, Any] | None = None,
        config_filepath: str | None = None,
    ) -> None:
        self.config_cls_to_fields: dict[str, set[str]] = defaultdict(set)
        _ = avoid_multi_inheritance_field_conflict(
            self.__class__, self.config_cls_to_fields
        )
        self._config_values: dict[ConfigField[Any], Any] = {}
        self._provenance: dict[str, str] = {}

        # on the first run, we load defaults values specified in the mapping
        # afterwards, that values will be modified by the ancestors
        # the children will have higher priority
        self._mapped_keys: set[str] = set()
        self.init_config_values(self.__class__, config_values, config_filepath)

    @classmethod
    def get_env_key(cls, field: str, config: type["ConfigRoot"] | None = None) -> str:
        """Get the key used in the environment variables."""
        if config:
            return f"{cls.ENV_PREFIX}{cls.ENV_SEP}{config.__name__.upper()}{cls.ENV_SEP}{field.upper()}"
        return f"{cls.ENV_PREFIX}{cls.ENV_SEP}{field.upper()}"

    @classmethod
    def config_fields(cls) -> list[str]:
        """Return the list of configuration field names."""
        return list(get_config_fields(cls))

    @property
    def all_fields(self) -> list[str]:
        """Return all field names across all parent config classes."""
        return list(set().union(*self.config_cls_to_fields.values()))

    def get_config_id(self, config_cls: type["ConfigRoot"]) -> str:
        """Return the configuration identifier for the given class."""
        return config_cls.__name__.replace(self.IGNORE_CLASS_NAME_SUBSTR, "").lower()

    @abstractmethod
    def init_parent_values(
        self,
        config_cls: type["ConfigRoot"],
        config_values: dict[str, Any] | None,
        config_filepath: str | None,
    ) -> None:
        """Initialize values from parent configuration classes."""
        # Initialize parent classes that are subclasses of ConfigRoot
        for parent in config_cls.__bases__:
            if issubclass(parent, ConfigRoot) and parent is not ConfigRoot:
                self.init_config_values(parent, config_values, config_filepath)

    @abstractmethod
    def init_config_values(
        self,
        config_cls: type["ConfigRoot"],
        config_values: dict[str, Any] | None,
        config_filepath: str | None,
    ) -> None:
        """Initialize the configuration values."""
        del config_cls, config_values, config_filepath

    def _get_field_descriptor(self, field_name: str) -> "ConfigField[Any] | None":
        """Walk the MRO to find the ConfigField descriptor for a field name."""
        for cls in type(self).__mro__:
            if field_name in cls.__dict__ and isinstance(
                cls.__dict__[field_name], ConfigField
            ):
                field: ConfigField[Any] = cls.__dict__[field_name]
                return field
        return None

    def field_info(self, field_name: str) -> FieldProvenance:
        """Return provenance information for a single field."""
        if field_name not in self.all_fields:
            msg = f"No field named '{field_name}'"
            raise KeyError(msg)
        field_obj = self._get_field_descriptor(field_name)
        is_secret = bool(field_obj and field_obj._secret)
        if field_name in self._provenance:
            source = self._provenance[field_name]
            is_default = False
        else:
            source = "default"
            is_default = True
        value = getattr(self, field_name)
        display_value = None if is_secret else str(value)
        return FieldProvenance(
            source=source,
            is_default=is_default,
            is_secret=is_secret,
            display_value=display_value,
        )

    def explain(self) -> str:
        """Return a human-readable provenance report for all fields."""
        lines = []
        for field_name in sorted(self.all_fields):
            info = self.field_info(field_name)
            if info.is_secret:
                lines.append(f"{field_name} = <secret> [from: {info.source}]")
            else:
                lines.append(
                    f"{field_name} = {getattr(self, field_name)} [from: {info.source}]"
                )
        return "\n".join(lines)

    def safe_dict(self) -> MappingProxyType[str, Any]:
        """Return an immutable dict of field values with secrets redacted."""
        d: dict[str, Any] = {}
        for field_name in sorted(self.all_fields):
            field_obj = self._get_field_descriptor(field_name)
            if field_obj and field_obj._secret:
                d[field_name] = "<secret>"
            else:
                d[field_name] = getattr(self, field_name)
        return MappingProxyType(d)

    @classmethod
    def override(cls, **kwargs: Any) -> "_OverrideContext":
        """Return a context manager that creates a config instance with overrides."""
        return _OverrideContext(cls, kwargs)

    def __repr__(self) -> str:
        sd = self.safe_dict()
        parts = []
        for k, v in sd.items():
            if v == "<secret>":
                parts.append(f"{k}=<secret>")
            else:
                parts.append(f"{k}={v!r}")
        return f"{self.__class__.__name__}({', '.join(parts)})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, ConfigRoot):
            return NotImplemented
        if type(self) is not type(other):
            return False
        for field_name in self.all_fields:
            if getattr(self, field_name) != getattr(other, field_name):
                return False
        return True

    def __hash__(self) -> int:
        return id(self)


class _OverrideContext:
    """Context manager for ConfigRoot.override()."""

    def __init__(self, cls: type[ConfigRoot], overrides: dict[str, Any]) -> None:
        self._cls = cls
        self._overrides = overrides
        self._instance: ConfigRoot | None = None

    def __enter__(self) -> ConfigRoot:
        self._instance = self._cls(config_values=self._overrides)
        return self._instance

    def __exit__(self, *args: Any) -> None:
        self._instance = None


def get_config_fields(cls: type) -> Iterator[str]:
    """Yield configuration field names from a class."""
    for key, value in cls.__dict__.items():
        if isinstance(value, ConfigField):
            yield key


def avoid_multi_inheritance_field_conflict(
    config_cls: type, config_cls_to_fields: dict[str, set[str]]
) -> dict[str, str]:
    """Ensure that the same configuration field is not defined in multiple parent classes of a given configuration class.

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
                msg = f"ConfigField {key} found in parent classes {parent.__name__} and {map_field_to_config_cls[key]}"
                raise ConfigMultiInheritanceError(msg)
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
