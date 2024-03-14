import json
import tomllib
from typing import Any

import yaml


def load_pyproject_toml(config_id: str, file_path: str) -> dict:
    """
    Loads configuration from a TOML file.

    :param str file_path: The path to the TOML file.
    :return: A dictionary containing the configuration data.
    """
    with open(file_path, "rb") as toml_file:
        if config_data := tomllib.load(toml_file):
            if "pyproject.toml" in file_path:
                return config_data.get("tool", {}).get(config_id, {})
        return config_data or {}


def load_file(config_id: str, filepath: str) -> dict[str, Any]:
    """
    Loads data from a file based on its extension (YAML, JSON, TOML).

    :param str filepath: The path to the file.
    :return: A dictionary containing the file's data.
    """
    with open(filepath) as _file:
        if filepath.lower().endswith(".yaml") or filepath.lower().endswith(".yml"):
            # TODO fix yaml
            return yaml.load(_file, Loader=yaml.SafeLoader)
        if filepath.lower().endswith(".json"):
            return json.load(_file)
        if filepath.lower().endswith(".toml"):
            return load_pyproject_toml(config_id, filepath)
    raise ValueError(f"Unexpected file extension {filepath=}")
