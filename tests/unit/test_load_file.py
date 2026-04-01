import os
import pathlib
import tempfile

import pytest

from cistell.util_files import load_file  # Adjust import path as needed


def create_temp_file(content: str, extension: str) -> str:
    fd, path = tempfile.mkstemp(suffix=extension)
    with os.fdopen(fd, "w") as tmp:
        tmp.write(content)
    return path


@pytest.mark.parametrize(
    ("extension", "content", "expected"),
    [
        (".yaml", "key: value", {"key": "value"}),
        (".json", '{"key": "value"}', {"key": "value"}),
        (".toml", 'key = "value"', {"key": "value"}),
        (".pyproject.toml", '[tool.test]\nkey = "value"', {"key": "value"}),
    ],
)
def test_load_file_valid(extension: str, content: str, expected: dict) -> None:
    filepath = create_temp_file(content, extension)
    result = load_file("test", filepath)
    assert result == expected
    pathlib.Path(filepath).unlink()


def test_load_file_invalid_extension() -> None:
    filepath = create_temp_file("content", ".txt")
    with pytest.raises(ValueError, match="Unsupported file extension"):
        load_file("test", filepath)
    pathlib.Path(filepath).unlink()


def test_load_file_missing_file() -> None:
    with pytest.raises(FileNotFoundError):
        load_file("test", "nonexistent.yaml")


def test_load_file_invalid_format() -> None:
    for ext, invalid_content in [
        (".yaml", "key: value:"),
        (".json", "{key: 'value'}"),
        (".toml", "key = 'value"),
    ]:
        filepath = create_temp_file(invalid_content, ext)
        with pytest.raises(ValueError, match=r"."):
            load_file("test", filepath)
        pathlib.Path(filepath).unlink()
