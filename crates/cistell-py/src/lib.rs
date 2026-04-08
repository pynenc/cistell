// PyO3 macro expansion generates useless_conversion clippy warnings.
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySet, PyTuple};

use cistell_core::value::ConfigValue;

/// Tracks the source and metadata of a resolved configuration field value.
#[pyclass(frozen, from_py_object)]
#[derive(Clone, Debug)]
pub struct FieldProvenance {
    #[pyo3(get)]
    pub source: String,
    #[pyo3(get)]
    pub is_default: bool,
    #[pyo3(get)]
    pub is_secret: bool,
    #[pyo3(get)]
    pub display_value: Option<String>,
}

#[pymethods]
impl FieldProvenance {
    #[new]
    #[pyo3(signature = (*, source, is_default, is_secret, display_value=None))]
    fn new(
        source: String,
        is_default: bool,
        is_secret: bool,
        display_value: Option<String>,
    ) -> Self {
        Self {
            source,
            is_default,
            is_secret,
            display_value,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "FieldProvenance(source='{}', is_secret={})",
            self.source,
            if self.is_secret { "True" } else { "False" }
        )
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a [`ConfigValue`] to a Python object recursively.
fn config_value_to_py(py: Python<'_>, val: &ConfigValue) -> PyResult<Py<PyAny>> {
    Ok(match val {
        ConfigValue::String(s) => s.into_pyobject(py)?.into_any().unbind(),
        ConfigValue::Integer(i) => i.into_pyobject(py)?.into_any().unbind(),
        ConfigValue::Float(f) => f.into_pyobject(py)?.into_any().unbind(),
        ConfigValue::Bool(b) => b.into_pyobject(py)?.to_owned().into_any().unbind(),
        ConfigValue::Array(arr) => {
            let items: Vec<Py<PyAny>> = arr
                .iter()
                .map(|item| config_value_to_py(py, item))
                .collect::<PyResult<_>>()?;
            PyList::new(py, items)?.into_any().unbind()
        }
        ConfigValue::Table(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map.iter() {
                dict.set_item(k, config_value_to_py(py, v)?)?;
            }
            dict.into_any().unbind()
        }
    })
}

// ---------------------------------------------------------------------------
// load_config_file
// ---------------------------------------------------------------------------

/// Parse a configuration file and return a Python `dict`.
///
/// Supports TOML, YAML and JSON (auto-detected by extension).
/// When `config_id` is given **and** the path contains `"pyproject.toml"`,
/// the `[tool.{config_id}]` section is extracted.
#[pyfunction]
#[pyo3(signature = (path, config_id=None))]
fn load_config_file(py: Python<'_>, path: &str, config_id: Option<&str>) -> PyResult<Py<PyAny>> {
    let p = std::path::Path::new(path);
    let content = std::fs::read_to_string(p).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            pyo3::exceptions::PyFileNotFoundError::new_err(format!(
                "Configuration file not found: {path}"
            ))
        } else {
            pyo3::exceptions::PyOSError::new_err(format!("Error reading file {path}: {e}"))
        }
    })?;

    let extension = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let root = match extension.as_str() {
        "toml" => {
            let table: toml::Table = content.parse().map_err(|e: toml::de::Error| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid TOML file {path}: {e}"))
            })?;
            ConfigValue::from(toml::Value::Table(table))
        }
        "yaml" | "yml" => {
            let val: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid YAML file {path}: {e}"))
            })?;
            ConfigValue::from(val)
        }
        "json" => {
            let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON file {path}: {e}"))
            })?;
            ConfigValue::from(val)
        }
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unsupported file extension in {path}"
            )));
        }
    };

    // Non-table root → empty dict (matches Python pyyaml / json behaviour).
    if !matches!(&root, ConfigValue::Table(_)) {
        return Ok(PyDict::new(py).into_any().unbind());
    }

    // For pyproject.toml, extract [tool.{config_id}].
    let is_pyproject = path.to_lowercase().contains("pyproject.toml");
    if is_pyproject {
        if let Some(cid) = config_id {
            if let ConfigValue::Table(ref table) = root {
                if let Some(ConfigValue::Table(tool)) = table.get("tool") {
                    if let Some(section) = tool.get(cid) {
                        return config_value_to_py(py, section);
                    }
                }
            }
            // Section not found → empty dict.
            return Ok(PyDict::new(py).into_any().unbind());
        }
    }

    config_value_to_py(py, &root)
}

// ---------------------------------------------------------------------------
// resolve_field – helpers
// ---------------------------------------------------------------------------

/// Walk `dict[keys[0]][keys[1]]…`, casting each intermediate value to [`PyDict`].
///
/// Returns `None` when any key is absent or any intermediate is not a dict.
fn dict_chain_get<'py>(
    dict: &Bound<'py, PyDict>,
    keys: &[&str],
) -> PyResult<Option<Bound<'py, PyAny>>> {
    let Some((&first, rest)) = keys.split_first() else {
        return Ok(None);
    };
    let Some(mut current) = dict.get_item(first)? else {
        return Ok(None);
    };
    for key in rest {
        let next = {
            let Ok(sub) = current.cast::<PyDict>() else {
                return Ok(None);
            };
            sub.get_item(key)?
        };
        match next {
            Some(val) => current = val,
            None => return Ok(None),
        }
    }
    Ok(Some(current))
}

/// Look up a value in `mapping` via a chain of dict keys, guarded by `mapped_keys`.
///
/// If the composite key is already in `mapped_keys` the lookup is skipped
/// (returns `None`).  On a successful find the composite key is recorded in
/// `mapped_keys` to prevent duplicate application in multi-inheritance.
fn try_mapping_lookup<'py>(
    mapping: &Bound<'py, PyDict>,
    keys: &[&str],
    composite_key: &str,
    mapped_keys: Option<&Bound<'py, PySet>>,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    if let Some(mk) = mapped_keys {
        if mk.contains(composite_key)? {
            return Ok(None);
        }
    }
    let Some(val) = dict_chain_get(mapping, keys)? else {
        return Ok(None);
    };
    if let Some(mk) = mapped_keys {
        mk.add(composite_key)?;
    }
    Ok(Some(val))
}

/// Try to read an environment variable and wrap it as a Python string with provenance.
fn resolve_env_var(
    py: Python<'_>,
    key: &str,
    secret: bool,
) -> PyResult<Option<(Py<PyAny>, FieldProvenance)>> {
    let Ok(val) = std::env::var(key) else {
        return Ok(None);
    };
    Ok(Some((
        val.into_pyobject(py)?.into_any().unbind(),
        FieldProvenance {
            source: format!("env var '{key}'"),
            is_default: false,
            is_secret: secret,
            display_value: None,
        },
    )))
}

// ---------------------------------------------------------------------------
// resolve_field
// ---------------------------------------------------------------------------

/// Resolve a single configuration field through all sources.
///
/// **Mapping sources** are processed low → high priority (each may overwrite
/// the previous). Within a single source, the class-specific subsection
/// `mapping[config_id][field_name]` beats the generic `mapping[field_name]`.
/// The `mapped_keys` set prevents the same `"{source}##{field}"` pair from
/// being applied twice (critical for multi-inheritance).
///
/// **Qualifier subsections** (`extra_qualifiers`) are checked after the
/// class-specific subsection with even higher priority.  For example, pynenc
/// passes `["module_name.task_name"]` so that a YAML section like
/// `task: { module_name.task_name: { max_retries: 5 } }` overrides the
/// class-level `task: { max_retries: 10 }`.
///
/// **Env vars** (highest priority) always override; they are not gated by
/// `mapped_keys`.  When `extra_env_keys` is provided, those keys are checked
/// **after** `class_env_key` and `generic_env_key`, giving them the highest
/// possible priority.
///
/// Returns `Some((value, FieldProvenance))` when a source provides a value,
/// or `None` when nothing was found (caller should keep any previously-set
/// value / field default).
#[pyfunction]
#[pyo3(signature = (
    field_name,
    config_id,
    class_env_key,
    generic_env_key,
    *,
    secret = false,
    mappings = None,
    mapped_keys = None,
    extra_qualifiers = None,
    extra_env_keys = None,
))]
#[allow(clippy::too_many_arguments)]
fn resolve_field<'py>(
    py: Python<'py>,
    field_name: &str,
    config_id: &str,
    class_env_key: &str,
    generic_env_key: &str,
    secret: bool,
    mappings: Option<&Bound<'py, PyList>>,
    mapped_keys: Option<&Bound<'py, PySet>>,
    extra_qualifiers: Option<&Bound<'py, PyList>>,
    extra_env_keys: Option<&Bound<'py, PyList>>,
) -> PyResult<Option<(Py<PyAny>, FieldProvenance)>> {
    let mut current_value: Option<Py<PyAny>> = None;
    let mut current_source = String::new();

    // 1. Mapping sources (lowest → highest priority).
    if let Some(mappings) = mappings {
        for item in mappings.iter() {
            let py_tuple = item.cast::<PyTuple>()?;
            let source_name: String = py_tuple.get_item(0)?.extract()?;
            let mapping_any = py_tuple.get_item(1)?;
            let mapping = mapping_any.cast::<PyDict>()?;

            // --- generic: mapping[field_name] ---
            let general_key = format!("{source_name}##{field_name}");
            if let Some(val) =
                try_mapping_lookup(mapping, &[field_name], &general_key, mapped_keys)?
            {
                current_value = Some(val.unbind());
                current_source.clone_from(&source_name);
            }

            // --- class-specific: mapping[config_id][field_name] ---
            let class_key = format!("{source_name}##{config_id}##{field_name}");
            if let Some(val) =
                try_mapping_lookup(mapping, &[config_id, field_name], &class_key, mapped_keys)?
            {
                current_value = Some(val.unbind());
                current_source.clone_from(&source_name);
            }

            // --- qualifier subsections: mapping[config_id][qualifier][field_name] ---
            // Higher priority than class-specific; later qualifiers win.
            if let Some(qualifiers) = extra_qualifiers {
                for qual_item in qualifiers.iter() {
                    let qualifier: String = qual_item.extract()?;
                    let qual_key = format!("{source_name}##{config_id}##{qualifier}##{field_name}");
                    if let Some(val) = try_mapping_lookup(
                        mapping,
                        &[config_id, &qualifier, field_name],
                        &qual_key,
                        mapped_keys,
                    )? {
                        current_value = Some(val.unbind());
                        current_source.clone_from(&source_name);
                    }
                }
            }
        }
    }

    // 2. Env vars (highest priority, always override).
    // Extra env keys have the highest priority; checked first so they win.
    if let Some(extra_keys) = extra_env_keys {
        // Iterate in reverse so the last key in the list has highest priority.
        for key_item in extra_keys.iter().rev() {
            let env_key: String = key_item.extract()?;
            if let Some(result) = resolve_env_var(py, &env_key, secret)? {
                return Ok(Some(result));
            }
        }
    }
    if let Some(result) = resolve_env_var(py, class_env_key, secret)? {
        return Ok(Some(result));
    }
    if let Some(result) = resolve_env_var(py, generic_env_key, secret)? {
        return Ok(Some(result));
    }

    // 3. Return mapping result or None.
    match current_value {
        Some(val) => Ok(Some((
            val,
            FieldProvenance {
                source: current_source,
                is_default: false,
                is_secret: secret,
                display_value: None,
            },
        ))),
        None => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

#[pymodule(name = "_internal")]
fn cistell_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FieldProvenance>()?;
    m.add_function(wrap_pyfunction!(load_config_file, m)?)?;
    m.add_function(wrap_pyfunction!(resolve_field, m)?)?;
    Ok(())
}
