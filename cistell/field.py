from typing import TYPE_CHECKING, Any, Callable, Generic, Optional, Type, TypeVar, cast

if TYPE_CHECKING:
    from cistell.root import ConfigRoot

T = TypeVar("T")

ConfigLoader = Callable[[str], dict[str, str]]
ConfigFieldMapper = Callable[[Any, Type[T]], T]


def default_config_field_mapper(value: Any, expected_type: Type[T]) -> T:
    """Generic type conversion function for a config field."""
    if isinstance(value, expected_type):
        return value
    try:
        callable_type = cast(Callable[[Any], T], expected_type)
        return callable_type(value)  # type conversion
    except (ValueError, TypeError) as ex:
        raise TypeError(
            f"Invalid type. Expected {expected_type} instead {type(value)}."
        ) from ex


class ConfigField(Generic[T]):
    """
    Define each typed field from a ConfigBase instance.

    This class is used to define typed configuration fields within a ConfigBase
    subclass. It ensures type consistency and supports value validation and casting.

    :param T default_value:
        The default value for the configuration field.
    :param Optional[ConfigFieldMapper] mapper:
        An optional function to map or transform the value.
    """

    def __init__(
        self, default_value: T, mapper: Optional[ConfigFieldMapper] = None
    ) -> None:
        self._default_value: T = default_value
        self._mapper = mapper or default_config_field_mapper

    def __get__(self, instance: Optional["ConfigRoot"], owner: Type[object]) -> T:
        del owner
        if instance is None:
            return self._default_value
        return instance._config_values.get(self, self._default_value)

    def __set__(self, instance: "ConfigRoot", value: Any) -> None:
        instance._config_values[self] = self._mapper(value, type(self._default_value))
