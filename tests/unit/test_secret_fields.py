from cistell import ConfigBase, ConfigField


class SecretConfig(ConfigBase):
    host = ConfigField("localhost")
    password = ConfigField("s3cret", secret=True)


def test_secret_field_access() -> None:
    """Secret field should return actual value when accessed for use in code."""
    cfg = SecretConfig()
    assert cfg.password == "s3cret"


def test_secret_field_repr() -> None:
    """repr() should show <secret> for secret fields."""
    cfg = SecretConfig()
    r = repr(cfg)
    assert "<secret>" in r
    assert "s3cret" not in r


def test_secret_field_safe_dict() -> None:
    """safe_dict() should redact secret field values."""
    cfg = SecretConfig()
    sd = cfg.safe_dict()
    assert sd["password"] == "<secret>"


def test_secret_field_explain() -> None:
    """explain() should redact secret field values."""
    cfg = SecretConfig()
    output = cfg.explain()
    assert "<secret>" in output
    assert "s3cret" not in output
