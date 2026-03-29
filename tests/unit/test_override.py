from cistell import ConfigBase, ConfigField


class OverrideConfig(ConfigBase):
    host = ConfigField("localhost")
    port = ConfigField(6379)


def test_override_changes_value() -> None:
    with OverrideConfig.override(host="test-host") as cfg:
        assert cfg.host == "test-host"


def test_override_reverts_on_exit() -> None:
    with OverrideConfig.override(host="test-host") as cfg:
        assert cfg.host == "test-host"
    # Outside: original resolution restored
    normal = OverrideConfig()
    assert normal.host == "localhost"


def test_override_multiple_fields() -> None:
    with OverrideConfig.override(host="x", port=1234) as cfg:
        assert cfg.host == "x"
        assert cfg.port == 1234


def test_override_nested() -> None:
    with OverrideConfig.override(host="a") as cfg1:
        assert cfg1.host == "a"
        with OverrideConfig.override(host="b") as cfg2:
            assert cfg2.host == "b"
        assert cfg1.host == "a"


def test_override_exception_reverts() -> None:
    try:
        with OverrideConfig.override(host="err-host") as cfg:
            assert cfg.host == "err-host"
            msg = "test error"
            raise ValueError(msg)
    except ValueError:
        pass
    normal = OverrideConfig()
    assert normal.host == "localhost"
