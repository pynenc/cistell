import json
import tomllib
from typing import Any

import yaml


def load_pyproject_toml(config_id: str, file_path: str) -> dict[str, Any]:
    """
    Loads configuration from a TOML file.

    :param str file_path: The path to the TOML file.
    :return: A dictionary containing the configuration data.
    """
    try:
        with open(file_path) as toml_file:
            config_data = tomllib.loads(toml_file.read())
            if "pyproject.toml" in file_path.lower():
                return config_data.get("tool", {}).get(config_id, {})
            return config_data or {}
    except tomllib.TOMLDecodeError as e:
        raise ValueError(f"Invalid TOML file {file_path}: {str(e)}") from e


def load_file(config_id: str, filepath: str) -> dict[str, Any]:
    """
    Loads data from a file based on its extension (YAML, JSON, TOML).

    :param str filepath: The path to the file.
    :return: A dictionary containing the file's data.
    """
    try:
        with open(filepath) as _file:
            if filepath.lower().endswith((".yaml", ".yml")):
                try:
                    data = yaml.load(_file, Loader=yaml.SafeLoader)
                    return data if isinstance(data, dict) else {}
                except yaml.YAMLError as e:
                    raise ValueError(f"Invalid YAML file {filepath}: {str(e)}") from e
            elif filepath.lower().endswith(".json"):
                try:
                    data = json.load(_file)
                    return data if isinstance(data, dict) else {}
                except json.JSONDecodeError as e:
                    raise ValueError(f"Invalid JSON file {filepath}: {str(e)}") from e
            elif filepath.lower().endswith(".toml"):
                return load_pyproject_toml(config_id, filepath)
            else:
                raise ValueError(f"Unsupported file extension in {filepath}")
    except FileNotFoundError as e:
        raise FileNotFoundError(f"Configuration file not found: {filepath}") from e
