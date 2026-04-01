"""File-loading utilities — delegates to Rust ``_internal.load_config_file``."""

from typing import Any

from cistell._internal import load_config_file as _rust_load


def load_pyproject_toml(config_id: str, file_path: str) -> dict[str, Any]:
    """Load configuration from a TOML file.

    :param str file_path: The path to the TOML file.
    :return: A dictionary containing the configuration data.
    """
    return _rust_load(file_path, config_id)


def load_file(config_id: str, filepath: str) -> dict[str, Any]:
    """Load data from a file based on its extension (YAML, JSON, TOML).

    :param str filepath: The path to the file.
    :return: A dictionary containing the file's data.
    """
    return _rust_load(filepath, config_id)
