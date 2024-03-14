import json
import os
import tempfile
from unittest.mock import patch

from cistell import ConfigBase, ConfigField


class LibraryConfigBase(ConfigBase):
    TOML_CONFIG_ID = "other_id"
    ENV_PREFIX = "LIBCFG"
    ENV_SEP = "<->"
    ENV_FILEPATH = "CFGFILE"
    TOML_IGNORE_CONFIG_PATTERN = "LibraryConfig"


class LibraryConfigMain(LibraryConfigBase):
    value = ConfigField("default_main")


class Secondary(LibraryConfigBase):
    value = ConfigField(3)


def test_library_config_default() -> None:
    """Test that returns the default values."""
    main_config = LibraryConfigMain()
    secondary_config = Secondary()
    assert main_config.value == "default_main"
    assert secondary_config.value == 3


def test_library_config_env_variables() -> None:
    """Test environment variables override."""
    with patch.dict(
        os.environ,
        {
            "LIBCFG<->LIBRARYCONFIGMAIN<->VALUE": "env_main",
            "LIBCFG<->SECONDARY<->VALUE": "4",
        },
    ):
        main_config = LibraryConfigMain()
        secondary_config = Secondary()
        assert main_config.value == "env_main"
        assert secondary_config.value == 4


def test_library_config_file() -> None:
    """Test configuration from a file overrides previous values."""
    with tempfile.TemporaryDirectory() as tmpdir:
        filepath = os.path.join(tmpdir, "lib_config.json")
        with open(filepath, mode="w") as _file:
            _file.write(
                json.dumps(
                    {
                        "main": {"value": "file_main"},
                        "secondary": {"value": 5},
                    }
                )
            )

        # Set up environment variable to pick config file
        with patch.dict(os.environ, {"LIBCFG<->CFGFILE": filepath}):
            main_config = LibraryConfigMain()
            secondary_config = Secondary()
            assert main_config.value == "file_main"
            assert secondary_config.value == 5


def test_library_config_specific_env_over_file() -> None:
    """Test environment variables override file configuration."""
    with tempfile.TemporaryDirectory() as tmpdir:
        filepath = os.path.join(tmpdir, "lib_config.json")
        with open(filepath, mode="w") as _file:
            _file.write(
                json.dumps(
                    {
                        "main": {"value": "file_main"},
                        "secondary": {"value": 5},
                    }
                )
            )

        with patch.dict(
            os.environ,
            {
                "LIBCFG<->CFGFILE": filepath,
                "LIBCFG<->LIBRARYCONFIGMAIN<->VALUE": "env_main",
                "LIBCFG<->SECONDARY<->VALUE": "6",
            },
        ):
            main_config = LibraryConfigMain()
            secondary_config = Secondary()
            assert main_config.value == "env_main"
            assert secondary_config.value == 6
