# Migration Guide (0.1.0)

For existing `cistell` Python users, transitioning to version 0.1.0 is designed to be seamless while bringing major under-the-hood improvements and new features.

## What's Changed?

| Before                                        | After                                  |
| --------------------------------------------- | -------------------------------------- | ----- |
| `pip install cistell`                         | `pip install cistell` (unchanged)      |
| `from cistell import ConfigBase, ConfigField` | Same (unchanged)                       |
| Python ≥ 3.11.6                               | Python ≥ 3.12                          |
| `typing.Optional[X]`                          | `X                                     | None` |
| `typing-extensions` required                  | Not needed                             |
| `pyyaml` required                             | Not needed (YAML parsing done in Rust) |
| Poetry development                            | uv development                         |
| `poetry install`                              | `uv sync`                              |
| `poetry build`                                | `maturin build`                        |

## New Features Available

- **`cfg.explain()`** — Prints a detailed summary of all fields and where their current values were loaded from (provenance).
- **`cfg.safe_dict()`** — Returns an immutable dictionary of the configuration with secret values properly redacted (`***`).
- **`cfg.field_info(name)`** — Returns a `FieldProvenance` object for a specific field, showing its value, source, and priority.
- **`ConfigField(secret=True)`** — Marks a field as a secret. Secret fields are automatically redacted in `__repr__` and `safe_dict()`.
- **`ConfigBase.override()`** — A context manager meant for testing that isolates configuration overriding. Overrides revert when the context exits.
- **`repr(cfg)`** — Secrets are redacted automatically when the configuration object is printed.

## Breaking Changes

- **Python 3.12+ required**: The minimum supported Python version has been bumped to 3.12.
- **`typing-extensions` dependency**: No longer required.
- **`pyyaml` Python dependency**: Removed; parsing is now handled by the native Rust backend.
- **Rust Implementation**: The internal implementation is now fully powered by Rust (`cistell-core` via PyO3). Users implementing highly deeply customized configuration metaclasses or low-level hooks that relied directly on the old pure-Python internals may need to adjust to the new streamlined logic.

## Usage Guide Updates

Refer to our quickstart and README to test out the new provenance API. Your existing configurations deriving from `ConfigBase` and instantiating `ConfigField` should work completely unmodified out-of-the-box.
