"""Tests for resolve_field helper functions (dict_chain_get, try_mapping_lookup, resolve_env_var).

These exercise the internal Rust helpers through the public ``resolve_field``
FFI boundary, covering edge-cases that higher-level ConfigBase tests don't
reach.
"""

from __future__ import annotations

import os
import uuid

from cistell._internal import resolve_field

# ---------------------------------------------------------------------------
# Fixtures / constants
# ---------------------------------------------------------------------------

FIELD = "timeout"
CONFIG_ID = "myapp"
CLASS_ENV = f"MYAPP__{FIELD}".upper()
GENERIC_ENV = FIELD.upper()


def _mapping(source_name: str, data: dict) -> list[tuple[str, dict]]:
    """Wrap *data* as a single-source mapping list expected by resolve_field."""
    return [(source_name, data)]


# ---------------------------------------------------------------------------
# dict_chain_get — tested via mapping resolution depth
# ---------------------------------------------------------------------------


class TestDictChainGet:
    """Exercise ``dict_chain_get`` through mapping lookups at different depths."""

    def test_single_key_generic_lookup(self) -> None:
        """mapping[field_name] — single key chain."""
        mappings = _mapping("src", {FIELD: 42})
        val, prov = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert val == 42
        assert prov.source == "src"

    def test_two_key_class_specific_lookup(self) -> None:
        """mapping[config_id][field_name] — two-key chain."""
        mappings = _mapping("src", {CONFIG_ID: {FIELD: 99}})
        val, _prov = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert val == 99

    def test_three_key_qualifier_lookup(self) -> None:
        """mapping[config_id][qualifier][field_name] — three-key chain."""
        mappings = _mapping("src", {CONFIG_ID: {"qual.key": {FIELD: 7}}})
        result = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            extra_qualifiers=["qual.key"],
        )
        assert result is not None
        val, _ = result
        assert val == 7

    def test_missing_intermediate_key_returns_none(self) -> None:
        """mapping[config_id] missing → class-specific lookup yields nothing."""
        mappings = _mapping("src", {"other_id": {FIELD: 1}})
        result = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        # generic key not present either → None
        assert result is None

    def test_non_dict_intermediate_is_skipped(self) -> None:
        """mapping[config_id] is a non-dict value → treated as absent."""
        mappings = _mapping("src", {CONFIG_ID: "not-a-dict"})
        result = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert result is None

    def test_qualifier_non_dict_intermediate_is_skipped(self) -> None:
        """mapping[config_id][qualifier] is a non-dict → qualifier lookup skipped."""
        mappings = _mapping("src", {CONFIG_ID: {"qual": "oops", FIELD: 5}})
        result = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            extra_qualifiers=["qual"],
        )
        # Falls back to class-specific lookup for field_name
        assert result is not None
        val, _ = result
        assert val == 5

    def test_empty_mapping_dict_returns_none(self) -> None:
        mappings = _mapping("src", {})
        result = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert result is None

    def test_no_mappings_returns_none(self) -> None:
        result = resolve_field(FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV)
        assert result is None


# ---------------------------------------------------------------------------
# try_mapping_lookup — mapped_keys dedup behaviour
# ---------------------------------------------------------------------------


class TestTryMappingLookup:
    """Exercise ``try_mapping_lookup`` mapped_keys dedup logic."""

    def test_mapped_keys_prevents_duplicate_generic(self) -> None:
        """A generic key already in mapped_keys is not applied again."""
        mapped_keys: set[str] = set()
        mappings = _mapping("s1", {FIELD: 10})

        # First call — records the key
        val1, _ = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            mapped_keys=mapped_keys,
        )
        assert val1 == 10
        assert f"s1##{FIELD}" in mapped_keys

        # Second call with the same source — should be skipped
        mappings2 = _mapping("s1", {FIELD: 999})
        result = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings2,
            mapped_keys=mapped_keys,
        )
        assert result is None

    def test_mapped_keys_prevents_duplicate_class_specific(self) -> None:
        mapped_keys: set[str] = set()
        mappings = _mapping("s1", {CONFIG_ID: {FIELD: 20}})

        val1, _ = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            mapped_keys=mapped_keys,
        )
        assert val1 == 20
        assert f"s1##{CONFIG_ID}##{FIELD}" in mapped_keys

        # Duplicate skipped
        result = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            mapped_keys=mapped_keys,
        )
        assert result is None

    def test_mapped_keys_prevents_duplicate_qualifier(self) -> None:
        mapped_keys: set[str] = set()
        qual = "my.qual"
        mappings = _mapping("s1", {CONFIG_ID: {qual: {FIELD: 77}}})

        val1, _ = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            mapped_keys=mapped_keys,
            extra_qualifiers=[qual],
        )
        assert val1 == 77
        assert f"s1##{CONFIG_ID}##{qual}##{FIELD}" in mapped_keys

        # Duplicate skipped — only the generic key (not present) is re-checked
        result = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            mapped_keys=mapped_keys,
            extra_qualifiers=[qual],
        )
        assert result is None

    def test_different_sources_not_deduped(self) -> None:
        """mapped_keys are source-scoped — same field from a different source is applied."""
        mapped_keys: set[str] = set()
        m1 = _mapping("s1", {FIELD: 10})
        m2 = _mapping("s2", {FIELD: 20})

        resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=m1,
            mapped_keys=mapped_keys,
        )
        val, prov = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=m2,
            mapped_keys=mapped_keys,
        )
        assert val == 20
        assert prov.source == "s2"

    def test_none_mapped_keys_allows_all(self) -> None:
        """Without mapped_keys, every call applies (no dedup)."""
        mappings = _mapping("s1", {FIELD: 10})
        v1, _ = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        v2, _ = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert v1 == v2 == 10


# ---------------------------------------------------------------------------
# resolve_env_var — env-var resolution and provenance
# ---------------------------------------------------------------------------


class TestResolveEnvVar:
    """Exercise ``resolve_env_var`` through env-var paths in ``resolve_field``."""

    def _unique_key(self) -> str:
        return f"CISTELL_TEST_{uuid.uuid4().hex[:8].upper()}"

    def test_class_env_key_resolved(self) -> None:
        key = self._unique_key()
        os.environ[key] = "hello"
        try:
            val, prov = resolve_field(FIELD, CONFIG_ID, key, GENERIC_ENV)
            assert val == "hello"
            assert prov.source == f"env var '{key}'"
            assert prov.is_default is False
        finally:
            del os.environ[key]

    def test_generic_env_key_resolved(self) -> None:
        key = self._unique_key()
        os.environ[key] = "world"
        try:
            dummy_class = self._unique_key()  # not set
            val, prov = resolve_field(FIELD, CONFIG_ID, dummy_class, key)
            assert val == "world"
            assert prov.source == f"env var '{key}'"
        finally:
            del os.environ[key]

    def test_class_env_beats_generic(self) -> None:
        class_key = self._unique_key()
        generic_key = self._unique_key()
        os.environ[class_key] = "class"
        os.environ[generic_key] = "generic"
        try:
            val, prov = resolve_field(FIELD, CONFIG_ID, class_key, generic_key)
            assert val == "class"
            assert class_key in prov.source
        finally:
            del os.environ[class_key]
            del os.environ[generic_key]

    def test_env_var_beats_mapping(self) -> None:
        key = self._unique_key()
        os.environ[key] = "from_env"
        mappings = _mapping("file", {FIELD: "from_file"})
        try:
            val, prov = resolve_field(
                FIELD, CONFIG_ID, key, GENERIC_ENV, mappings=mappings
            )
            assert val == "from_env"
            assert "env var" in prov.source
        finally:
            del os.environ[key]

    def test_missing_env_var_falls_through(self) -> None:
        missing = self._unique_key()
        result = resolve_field(FIELD, CONFIG_ID, missing, missing)
        assert result is None

    def test_secret_flag_propagated(self) -> None:
        key = self._unique_key()
        os.environ[key] = "s3cr3t"
        try:
            _, prov = resolve_field(FIELD, CONFIG_ID, key, GENERIC_ENV, secret=True)
            assert prov.is_secret is True
        finally:
            del os.environ[key]

    def test_extra_env_keys_highest_priority(self) -> None:
        """extra_env_keys beat class and generic env keys."""
        class_key = self._unique_key()
        extra_key = self._unique_key()
        os.environ[class_key] = "class"
        os.environ[extra_key] = "extra"
        try:
            val, prov = resolve_field(
                FIELD,
                CONFIG_ID,
                class_key,
                GENERIC_ENV,
                extra_env_keys=[extra_key],
            )
            assert val == "extra"
            assert extra_key in prov.source
        finally:
            del os.environ[class_key]
            del os.environ[extra_key]

    def test_extra_env_keys_last_wins(self) -> None:
        """When multiple extra env keys are set, the last one in the list wins."""
        k1 = self._unique_key()
        k2 = self._unique_key()
        os.environ[k1] = "first"
        os.environ[k2] = "second"
        try:
            val, prov = resolve_field(
                FIELD,
                CONFIG_ID,
                CLASS_ENV,
                GENERIC_ENV,
                extra_env_keys=[k1, k2],
            )
            assert val == "second"
            assert k2 in prov.source
        finally:
            del os.environ[k1]
            del os.environ[k2]

    def test_extra_env_keys_missing_falls_to_class(self) -> None:
        """If extra env keys are not set, class env key is used."""
        class_key = self._unique_key()
        os.environ[class_key] = "class"
        missing = self._unique_key()
        try:
            val, _ = resolve_field(
                FIELD,
                CONFIG_ID,
                class_key,
                GENERIC_ENV,
                extra_env_keys=[missing],
            )
            assert val == "class"
        finally:
            del os.environ[class_key]


# ---------------------------------------------------------------------------
# Priority ordering — end-to-end through all helpers
# ---------------------------------------------------------------------------


class TestPriorityOrdering:
    """Verify the full priority chain: generic < class < qualifier < env."""

    def test_class_specific_beats_generic(self) -> None:
        mappings = _mapping("src", {FIELD: 1, CONFIG_ID: {FIELD: 2}})
        val, _ = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert val == 2

    def test_qualifier_beats_class_specific(self) -> None:
        mappings = _mapping(
            "src",
            {CONFIG_ID: {FIELD: 2, "q": {FIELD: 3}}},
        )
        val, _ = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            extra_qualifiers=["q"],
        )
        assert val == 3

    def test_later_qualifier_beats_earlier(self) -> None:
        mappings = _mapping(
            "src",
            {CONFIG_ID: {"q1": {FIELD: 10}, "q2": {FIELD: 20}}},
        )
        val, _ = resolve_field(
            FIELD,
            CONFIG_ID,
            CLASS_ENV,
            GENERIC_ENV,
            mappings=mappings,
            extra_qualifiers=["q1", "q2"],
        )
        assert val == 20

    def test_later_source_beats_earlier(self) -> None:
        mappings = [("low", {FIELD: 1}), ("high", {FIELD: 2})]
        val, prov = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert val == 2
        assert prov.source == "high"

    def test_provenance_source_tracks_winning_mapping(self) -> None:
        mappings = [("fileA", {FIELD: 1}), ("fileB", {CONFIG_ID: {FIELD: 2}})]
        _, prov = resolve_field(
            FIELD, CONFIG_ID, CLASS_ENV, GENERIC_ENV, mappings=mappings
        )
        assert prov.source == "fileB"
