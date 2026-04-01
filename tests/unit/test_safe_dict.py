from types import MappingProxyType

import pytest

from cistell import ConfigBase, ConfigField


class SafeDictConfig(ConfigBase):
    host = ConfigField("localhost")
    port = ConfigField(6379)
    password = ConfigField("s3cret", secret=True)


def test_safe_dict_returns_mapping_proxy() -> None:
    cfg = SafeDictConfig()
    sd = cfg.safe_dict()
    assert isinstance(sd, MappingProxyType)


def test_safe_dict_immutable() -> None:
    cfg = SafeDictConfig()
    sd = cfg.safe_dict()
    with pytest.raises(TypeError):
        sd["host"] = "new"  # type: ignore[index]


def test_safe_dict_secrets_redacted() -> None:
    cfg = SafeDictConfig()
    sd = cfg.safe_dict()
    assert sd["password"] == "<secret>"


def test_safe_dict_non_secrets_present() -> None:
    cfg = SafeDictConfig()
    sd = cfg.safe_dict()
    assert sd["host"] == "localhost"
    assert sd["port"] == 6379


def test_safe_dict_all_fields_present() -> None:
    cfg = SafeDictConfig()
    sd = cfg.safe_dict()
    assert "host" in sd
    assert "port" in sd
    assert "password" in sd
