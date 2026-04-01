"""Type stubs for cistell._internal (Rust extension module)."""

from __future__ import annotations

from typing import Any

class FieldProvenance:
    """Tracks the source and metadata of a resolved configuration field value."""

    @property
    def source(self) -> str: ...
    @property
    def is_default(self) -> bool: ...
    @property
    def is_secret(self) -> bool: ...
    @property
    def display_value(self) -> str | None: ...
    def __init__(
        self,
        *,
        source: str,
        is_default: bool,
        is_secret: bool,
        display_value: str | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

def load_config_file(path: str, config_id: str | None = None) -> dict[str, Any]: ...
def resolve_field(
    field_name: str,
    config_id: str,
    class_env_key: str,
    generic_env_key: str,
    *,
    secret: bool = False,
    mappings: list[tuple[str, dict[str, Any]]] | None = None,
    mapped_keys: set[str] | None = None,
) -> tuple[Any, FieldProvenance] | None: ...
