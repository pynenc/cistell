"""Tests for extra_qualifiers and extra_env_keys hooks in ConfigBase."""

from __future__ import annotations

import os

from typing import TYPE_CHECKING

from cistell import ConfigBase, ConfigField

if TYPE_CHECKING:
    from cistell.root import ConfigRoot


class TaskConfig(ConfigBase):
    """A config that uses extra qualifiers and env keys (like pynenc's ConfigTask)."""

    max_retries = ConfigField(3)
    timeout = ConfigField(30)

    def __init__(
        self,
        task_key: str | None = None,
        **kwargs,
    ) -> None:
        self._task_key = task_key
        super().__init__(**kwargs)

    def get_extra_qualifiers(self, config_cls: type[ConfigRoot]) -> list[str] | None:
        if self._task_key:
            return [self._task_key]
        return None

    def get_extra_env_keys(
        self, field_name: str, config_cls: type[ConfigRoot]
    ) -> list[str] | None:
        if self._task_key:
            return [f"TASK_{self._task_key.upper()}__{field_name.upper()}"]
        return None


class TestExtraQualifiers:
    def test_no_qualifiers_uses_default(self) -> None:
        cfg = TaskConfig()
        assert cfg.max_retries == 3

    def test_qualifier_picks_up_subsection(self) -> None:
        config_values = {
            "task": {
                "max_retries": 10,
                "my_module.my_task": {
                    "max_retries": 42,
                },
            },
        }
        cfg = TaskConfig(task_key="my_module.my_task", config_values=config_values)
        assert cfg.max_retries == 42

    def test_qualifier_without_match_falls_back_to_class(self) -> None:
        config_values = {
            "task": {
                "max_retries": 10,
            },
        }
        cfg = TaskConfig(task_key="my_module.my_task", config_values=config_values)
        assert cfg.max_retries == 10

    def test_qualifier_only_affects_matched_fields(self) -> None:
        config_values = {
            "task": {
                "max_retries": 10,
                "timeout": 60,
                "my_module.my_task": {
                    "max_retries": 42,
                },
            },
        }
        cfg = TaskConfig(task_key="my_module.my_task", config_values=config_values)
        assert cfg.max_retries == 42
        assert cfg.timeout == 60


class TestExtraEnvKeys:
    def test_extra_env_key_overrides(self) -> None:
        env_key = "TASK_MY_MODULE.MY_TASK__MAX_RETRIES"
        os.environ[env_key] = "99"
        try:
            cfg = TaskConfig(task_key="my_module.my_task")
            assert cfg.max_retries == 99
        finally:
            del os.environ[env_key]

    def test_extra_env_key_beats_class_env_key(self) -> None:
        class_key = TaskConfig.get_env_key("max_retries", TaskConfig)
        task_key = "TASK_MY_MODULE.MY_TASK__MAX_RETRIES"
        os.environ[class_key] = "50"
        os.environ[task_key] = "99"
        try:
            cfg = TaskConfig(task_key="my_module.my_task")
            assert cfg.max_retries == 99
        finally:
            del os.environ[class_key]
            del os.environ[task_key]

    def test_extra_env_key_beats_qualifier(self) -> None:
        env_key = "TASK_MY_MODULE.MY_TASK__MAX_RETRIES"
        os.environ[env_key] = "99"
        config_values = {
            "task": {
                "my_module.my_task": {
                    "max_retries": 42,
                },
            },
        }
        try:
            cfg = TaskConfig(task_key="my_module.my_task", config_values=config_values)
            assert cfg.max_retries == 99
        finally:
            del os.environ[env_key]

    def test_no_task_key_ignores_extra_env(self) -> None:
        cfg = TaskConfig()
        assert cfg.max_retries == 3
