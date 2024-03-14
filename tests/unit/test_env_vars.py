import os
from unittest.mock import patch

from cistell import base, field


class SomeConfig(base.ConfigBase):
    test_value = field.ConfigField(0)


def test_env_var() -> None:
    """Test that returns the default"""
    with patch.dict(
        os.environ,
        {
            "CONFIG__SOMECONFIG__TEST_VALUE": "13",
        },
    ):
        conf = SomeConfig()
    assert conf.test_value == 13
