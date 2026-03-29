from cistell import ConfigBase, ConfigField


class ReprEqConfig(ConfigBase):
    host = ConfigField("localhost")
    port = ConfigField(6379)
    password = ConfigField("s3cret", secret=True)


def test_repr_redacts_secrets() -> None:
    cfg = ReprEqConfig()
    r = repr(cfg)
    assert "<secret>" in r
    assert "s3cret" not in r


def test_repr_shows_values() -> None:
    cfg = ReprEqConfig()
    r = repr(cfg)
    assert "localhost" in r
    assert "6379" in r
    assert "ReprEqConfig(" in r


def test_eq_same_values() -> None:
    cfg1 = ReprEqConfig()
    cfg2 = ReprEqConfig()
    assert cfg1 == cfg2


def test_eq_different_values() -> None:
    cfg1 = ReprEqConfig(config_values={"host": "a"})
    cfg2 = ReprEqConfig(config_values={"host": "b"})
    assert cfg1 != cfg2


def test_eq_different_provenance_same_values() -> None:
    cfg1 = ReprEqConfig()
    cfg2 = ReprEqConfig(config_values={"host": "localhost", "port": 6379})
    # same values, different sources -> should still be equal
    assert cfg1 == cfg2


def test_eq_different_type() -> None:
    cfg = ReprEqConfig()
    assert cfg != "not a config"
    assert cfg != 42
    assert cfg != None  # noqa: E711
