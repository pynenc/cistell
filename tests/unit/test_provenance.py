import os

from unittest.mock import patch

import pytest

from cistell import ConfigBase, ConfigField, FieldProvenance


class SimpleConfig(ConfigBase):
    host = ConfigField("localhost")
    port = ConfigField(6379)


def test_explain_output_format() -> None:
    cfg = SimpleConfig()
    output = cfg.explain()
    assert "host = localhost [from: default]" in output
    assert "port = 6379 [from: default]" in output


def test_explain_secret_redacted() -> None:
    class SecretConfig(ConfigBase):
        password = ConfigField("s3cret", secret=True)

    cfg = SecretConfig()
    output = cfg.explain()
    assert "<secret>" in output
    assert "s3cret" not in output


def test_explain_env_source() -> None:
    with patch.dict(os.environ, {"CONFIG__SIMPLECONFIG__HOST": "from-env"}):
        cfg = SimpleConfig()
    output = cfg.explain()
    assert "host = from-env [from: env var 'CONFIG__SIMPLECONFIG__HOST']" in output


def test_field_info_returns_provenance() -> None:
    cfg = SimpleConfig()
    info = cfg.field_info("host")
    assert isinstance(info, FieldProvenance)
    assert info.source == "default"
    assert info.is_default is True
    assert info.is_secret is False
    assert info.display_value == "localhost"


def test_field_info_default_source() -> None:
    cfg = SimpleConfig()
    info = cfg.field_info("port")
    assert info.is_default is True
    assert info.source == "default"


def test_field_info_env_source() -> None:
    with patch.dict(os.environ, {"CONFIG__SIMPLECONFIG__HOST": "env-host"}):
        cfg = SimpleConfig()
    info = cfg.field_info("host")
    assert info.is_default is False
    assert "env var" in info.source


def test_field_info_secret() -> None:
    class SecretConfig(ConfigBase):
        password = ConfigField("secret-val", secret=True)

    cfg = SecretConfig()
    info = cfg.field_info("password")
    assert info.is_secret is True
    assert info.display_value is None


def test_field_info_invalid_name() -> None:
    cfg = SimpleConfig()
    with pytest.raises(KeyError):
        cfg.field_info("nonexistent")
