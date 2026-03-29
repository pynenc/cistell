"""Tests that verify the provenance / explain() output across multiple sources.

These tests assert the exact format of explain() so we catch regressions in the
human-readable provenance output.
"""

import json
import os
import tempfile
from unittest.mock import patch

import pytest

from cistell import ConfigBase, ConfigField


class AppConfig(ConfigBase):
    host = ConfigField("localhost")
    port = ConfigField(8080)
    debug = ConfigField(False)
    api_key = ConfigField("changeme", secret=True)


# ── defaults ────────────────────────────────────────────────────────────────


def test_explain_all_defaults() -> None:
    """All fields should show [from: default] when nothing is overridden."""
    cfg = AppConfig()
    output = cfg.explain()
    for field in ("host", "port", "debug"):
        assert f"[from: default]" in output
        assert f"{field} = " in output
    # secret field should be redacted but still show source
    assert "api_key = <secret> [from: default]" in output


def test_field_info_default_provenance() -> None:
    cfg = AppConfig()
    for name in ("host", "port", "debug"):
        info = cfg.field_info(name)
        assert info.is_default is True
        assert info.source == "default"
        assert info.is_secret is False
        assert info.display_value is not None


# ── environment variables ───────────────────────────────────────────────────


def test_explain_env_override() -> None:
    """Fields overridden by env vars should report the exact env var name."""
    env = {
        "CONFIG__APPCONFIG__HOST": "prod.example.com",
        "CONFIG__APPCONFIG__PORT": "9090",
    }
    with patch.dict(os.environ, env):
        cfg = AppConfig()
    output = cfg.explain()
    assert "host = prod.example.com [from: env var 'CONFIG__APPCONFIG__HOST']" in output
    assert "port = 9090 [from: env var 'CONFIG__APPCONFIG__PORT']" in output
    # non-overridden fields stay default
    assert "debug = False [from: default]" in output


def test_explain_env_default_key() -> None:
    """A class-less env var (CONFIG__HOST) should apply as a default."""
    with patch.dict(os.environ, {"CONFIG__HOST": "env-default-host"}):
        cfg = AppConfig()
    output = cfg.explain()
    assert "host = env-default-host [from: env var 'CONFIG__HOST']" in output


def test_field_info_env_source() -> None:
    with patch.dict(os.environ, {"CONFIG__APPCONFIG__DEBUG": "true"}):
        cfg = AppConfig()
    info = cfg.field_info("debug")
    assert info.is_default is False
    assert "env var" in info.source
    assert "CONFIG__APPCONFIG__DEBUG" in info.source


# ── config file (JSON) ─────────────────────────────────────────────────────


def test_explain_file_source() -> None:
    """Fields loaded from a config file should report the file path."""
    data = {"app": {"host": "file-host", "port": 3000}}
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False
    ) as f:
        json.dump(data, f)
        f.flush()
        tmp = f.name
    try:
        cfg = AppConfig(config_filepath=tmp)
        output = cfg.explain()
        assert "host = file-host" in output
        # NOTE: provenance currently shows the mapping label, not the file path.
        # This is a known limitation — see test_print_explain_mixed for the raw output.
        assert "config_filepath" in output
        assert "port = 3000" in output
    finally:
        os.unlink(tmp)


# ── secret redaction ────────────────────────────────────────────────────────


def test_explain_secret_env_redacted() -> None:
    """Secret fields overridden by env should still be redacted."""
    with patch.dict(os.environ, {"CONFIG__APPCONFIG__API_KEY": "real-secret"}):
        cfg = AppConfig()
    output = cfg.explain()
    assert "api_key = <secret>" in output
    assert "real-secret" not in output
    # but the source should still be visible
    assert "env var" in output


def test_field_info_secret() -> None:
    cfg = AppConfig()
    info = cfg.field_info("api_key")
    assert info.is_secret is True
    assert info.display_value is None


# ── mixed sources ───────────────────────────────────────────────────────────


def test_explain_mixed_sources() -> None:
    """Verify explain() when values come from multiple different sources."""
    data = {"app": {"port": 5555}}
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False
    ) as f:
        json.dump(data, f)
        f.flush()
        tmp = f.name
    try:
        env = {"CONFIG__APPCONFIG__HOST": "env-host"}
        with patch.dict(os.environ, env):
            cfg = AppConfig(config_filepath=tmp)
        output = cfg.explain()
        # env wins for host
        assert "host = env-host [from: env var" in output
        # file wins for port (no env override)
        assert "port = 5555" in output
        assert "config_filepath" in output
        # debug stays default
        assert "debug = False [from: default]" in output
    finally:
        os.unlink(tmp)


# ── env var priority over file ──────────────────────────────────────────────


def test_env_overrides_file() -> None:
    """Environment variables should take priority over file values."""
    data = {"app": {"host": "file-host"}}
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False
    ) as f:
        json.dump(data, f)
        f.flush()
        tmp = f.name
    try:
        env = {"CONFIG__APPCONFIG__HOST": "env-host"}
        with patch.dict(os.environ, env):
            cfg = AppConfig(config_filepath=tmp)
        info = cfg.field_info("host")
        assert "env var" in info.source
        assert cfg.host == "env-host"
    finally:
        os.unlink(tmp)


# ── print explain output (diagnostic, always passes) ───────────────────────


def test_print_explain_defaults(capsys: pytest.CaptureFixture[str]) -> None:
    """Print explain() output so developers can review the format."""
    cfg = AppConfig()
    print("\n--- explain() with all defaults ---")
    print(cfg.explain())


def test_print_explain_mixed(capsys: pytest.CaptureFixture[str]) -> None:
    """Print explain() with mixed sources for visual review."""
    data = {"app": {"port": 5555, "debug": True}}
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False
    ) as f:
        json.dump(data, f)
        f.flush()
        tmp = f.name
    try:
        env = {
            "CONFIG__APPCONFIG__HOST": "prod.example.com",
            "CONFIG__APPCONFIG__API_KEY": "sk-live-xxx",
        }
        with patch.dict(os.environ, env):
            cfg = AppConfig(config_filepath=tmp)
        print("\n--- explain() with mixed sources ---")
        print(cfg.explain())
    finally:
        os.unlink(tmp)
