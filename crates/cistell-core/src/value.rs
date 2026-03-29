use std::fmt;

use indexmap::IndexMap;

use crate::error::ConfigError;
use crate::field::FieldMeta;

/// A loosely-typed config value, preserving the source format's native types.
/// This avoids the lossy `String` round-trip (TOML integer → String → parse again).
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    /// For list-valued fields (e.g. `allowed_hosts = ["a", "b"]`).
    Array(Vec<ConfigValue>),
    /// For nested structs or map-valued fields.
    Table(IndexMap<String, ConfigValue>),
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::String(s) => write!(f, "{s}"),
            ConfigValue::Integer(i) => write!(f, "{i}"),
            ConfigValue::Float(fl) => write!(f, "{fl}"),
            ConfigValue::Bool(b) => write!(f, "{b}"),
            ConfigValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            ConfigValue::Table(_) => write!(f, "{{...}}"),
        }
    }
}

impl ConfigValue {
    /// Coerce this value to the target Rust type.
    ///
    /// For `String` variants that need to be parsed into numeric types,
    /// this performs string parsing. For already-typed variants (`Integer`, `Float`, `Bool`),
    /// direct conversion is attempted first.
    pub fn coerce<T>(&self, field: &FieldMeta) -> Result<T, ConfigError>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        let as_string = match self {
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::Bool(b) => b.to_string(),
            ConfigValue::Array(_) | ConfigValue::Table(_) => {
                return Err(ConfigError::parse(
                    field,
                    &self.to_string(),
                    CoercionError("cannot coerce array/table to scalar".into()),
                ));
            }
        };

        as_string
            .parse::<T>()
            .map_err(|e| ConfigError::parse(field, &as_string, e))
    }

    /// Coerce to a `String` directly (no parsing needed for String variant).
    pub fn coerce_string(&self, _field: &FieldMeta) -> Result<String, ConfigError> {
        match self {
            ConfigValue::String(s) => Ok(s.clone()),
            other => Ok(other.to_string()),
        }
    }

    /// Walk a dotted key path (e.g. `"redis.host"`) into a `Table` value.
    pub fn get_by_dotted_key(&self, key: &str) -> Option<&ConfigValue> {
        let mut current = self;
        for segment in key.split('.') {
            match current {
                ConfigValue::Table(map) => {
                    current = map.get(segment)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    /// Coerce this value to a `Vec<T>`.
    ///
    /// Handles two input formats:
    /// - `ConfigValue::Array` — each element is coerced individually
    /// - `ConfigValue::String` — split on `,`, trim whitespace, filter empty entries
    ///
    /// Empty string `""` → empty vec (not `vec![""]`).
    /// Trailing commas are ignored: `"a,b,"` → `vec!["a", "b"]`.
    pub fn coerce_vec<T>(&self, field: &FieldMeta) -> Result<Vec<T>, ConfigError>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        match self {
            ConfigValue::Array(arr) => {
                let mut result = Vec::with_capacity(arr.len());
                for item in arr {
                    result.push(item.coerce::<T>(field)?);
                }
                Ok(result)
            }
            ConfigValue::String(s) => {
                if s.trim().is_empty() {
                    return Ok(Vec::new());
                }
                let mut result = Vec::new();
                for part in s.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        result.push(
                            trimmed
                                .parse::<T>()
                                .map_err(|e| ConfigError::parse(field, trimmed, e))?,
                        );
                    }
                }
                Ok(result)
            }
            other => Err(ConfigError::parse(
                field,
                &other.to_string(),
                CoercionError(format!(
                    "cannot coerce {} to Vec",
                    match other {
                        ConfigValue::Integer(_) => "integer",
                        ConfigValue::Float(_) => "float",
                        ConfigValue::Bool(_) => "bool",
                        ConfigValue::Table(_) => "table",
                        _ => "value",
                    }
                )),
            )),
        }
    }
}

/// Conversion from `toml::Value` to `ConfigValue`.
#[cfg(feature = "toml")]
impl From<toml::Value> for ConfigValue {
    fn from(val: toml::Value) -> Self {
        match val {
            toml::Value::String(s) => ConfigValue::String(s),
            toml::Value::Integer(i) => ConfigValue::Integer(i),
            toml::Value::Float(f) => ConfigValue::Float(f),
            toml::Value::Boolean(b) => ConfigValue::Bool(b),
            toml::Value::Datetime(dt) => ConfigValue::String(dt.to_string()),
            toml::Value::Array(arr) => {
                ConfigValue::Array(arr.into_iter().map(ConfigValue::from).collect())
            }
            toml::Value::Table(tbl) => {
                let map = tbl
                    .into_iter()
                    .map(|(k, v)| (k, ConfigValue::from(v)))
                    .collect();
                ConfigValue::Table(map)
            }
        }
    }
}

/// Conversion from `serde_yaml::Value` to `ConfigValue`.
#[cfg(feature = "yaml")]
impl From<serde_yaml::Value> for ConfigValue {
    fn from(val: serde_yaml::Value) -> Self {
        match val {
            serde_yaml::Value::Null => ConfigValue::String("".into()),
            serde_yaml::Value::Bool(b) => ConfigValue::Bool(b),
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ConfigValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ConfigValue::Float(f)
                } else {
                    ConfigValue::String(n.to_string())
                }
            }
            serde_yaml::Value::String(s) => ConfigValue::String(s),
            serde_yaml::Value::Sequence(seq) => {
                ConfigValue::Array(seq.into_iter().map(ConfigValue::from).collect())
            }
            serde_yaml::Value::Mapping(map) => {
                let mut tbl = IndexMap::new();
                for (k, v) in map {
                    if let Some(k_str) = k.as_str() {
                        tbl.insert(k_str.to_owned(), ConfigValue::from(v));
                    } else if let Some(i) = k.as_i64() {
                        tbl.insert(i.to_string(), ConfigValue::from(v));
                    }
                }
                ConfigValue::Table(tbl)
            }
            serde_yaml::Value::Tagged(tagged) => ConfigValue::from(tagged.value),
        }
    }
}

/// Conversion from `serde_json::Value` to `ConfigValue`.
#[cfg(feature = "json")]
impl From<serde_json::Value> for ConfigValue {
    fn from(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => ConfigValue::String("".into()),
            serde_json::Value::Bool(b) => ConfigValue::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ConfigValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ConfigValue::Float(f)
                } else {
                    ConfigValue::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => ConfigValue::String(s),
            serde_json::Value::Array(arr) => {
                ConfigValue::Array(arr.into_iter().map(ConfigValue::from).collect())
            }
            serde_json::Value::Object(map) => {
                let mut tbl = IndexMap::new();
                for (k, v) in map {
                    tbl.insert(k, ConfigValue::from(v));
                }
                ConfigValue::Table(tbl)
            }
        }
    }
}

/// A simple error type for coercion failures that aren't a standard parse error.
#[derive(Debug)]
struct CoercionError(String);

impl fmt::Display for CoercionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CoercionError {}

/// Built-in deserializer: converts a `ConfigValue` to a Duration (in seconds).
///
/// Accepts:
/// - `Integer(n)` → `Duration::from_secs(n)` (stored as `Integer(n)`)
/// - `String("300")` → parse to u64 → `Duration::from_secs(300)` (stored as `Integer(n)`)
/// - `Float(n)` → `Duration::from_secs_f64(n)` (stored as `Integer(n.round())`)
///
/// This is intended to be used as a `FieldMeta::deserialize_fn` value.
/// It converts the value *in place* to an `Integer` that `from_values()` can use.
pub fn duration_from_secs(
    value: &ConfigValue,
    field: &crate::field::FieldMeta,
) -> Result<ConfigValue, crate::error::ConfigError> {
    match value {
        ConfigValue::Integer(n) => {
            if *n < 0 {
                return Err(crate::error::ConfigError::parse(
                    field,
                    &n.to_string(),
                    CoercionError("duration cannot be negative".into()),
                ));
            }
            Ok(ConfigValue::Integer(*n))
        }
        ConfigValue::String(s) => {
            let secs: u64 = s
                .trim()
                .parse()
                .map_err(|e| crate::error::ConfigError::parse(field, s, e))?;
            Ok(ConfigValue::Integer(secs as i64))
        }
        ConfigValue::Float(f) => {
            if *f < 0.0 {
                return Err(crate::error::ConfigError::parse(
                    field,
                    &f.to_string(),
                    CoercionError("duration cannot be negative".into()),
                ));
            }
            Ok(ConfigValue::Integer(f.round() as i64))
        }
        other => Err(crate::error::ConfigError::parse(
            field,
            &other.to_string(),
            CoercionError(format!(
                "cannot convert {} to duration",
                match other {
                    ConfigValue::Bool(_) => "bool",
                    ConfigValue::Array(_) => "array",
                    ConfigValue::Table(_) => "table",
                    _ => "value",
                }
            )),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_coerce_string() {
        let meta = FieldMeta {
            name: "host",
            config_key: "redis.host",
            env_key: "RUSTVELLO__REDIS__HOST",
            generic_env_key: None,
            is_secret: false,
            expected_type: "String",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::String("hello".into());
        let result: String = val.coerce(&meta).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_config_value_coerce_integer() {
        let meta = FieldMeta {
            name: "port",
            config_key: "redis.port",
            env_key: "RUSTVELLO__REDIS__PORT",
            generic_env_key: None,
            is_secret: false,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::Integer(42);
        let result: u16 = val.coerce(&meta).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_config_value_coerce_float() {
        let meta = FieldMeta {
            name: "ratio",
            config_key: "app.ratio",
            env_key: "APP__RATIO",
            generic_env_key: None,
            is_secret: false,
            expected_type: "f64",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::Float(3.14);
        let result: f64 = val.coerce(&meta).unwrap();
        assert!((result - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_value_coerce_bool() {
        let meta = FieldMeta {
            name: "debug",
            config_key: "app.debug",
            env_key: "APP__DEBUG",
            generic_env_key: None,
            is_secret: false,
            expected_type: "bool",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::Bool(true);
        let result: bool = val.coerce(&meta).unwrap();
        assert!(result);
    }

    #[test]
    fn test_config_value_coerce_string_to_int() {
        let meta = FieldMeta {
            name: "port",
            config_key: "redis.port",
            env_key: "RUSTVELLO__REDIS__PORT",
            generic_env_key: None,
            is_secret: false,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::String("42".into());
        let result: u16 = val.coerce(&meta).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_config_value_coerce_type_mismatch() {
        let meta = FieldMeta {
            name: "port",
            config_key: "redis.port",
            env_key: "RUSTVELLO__REDIS__PORT",
            generic_env_key: None,
            is_secret: false,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::String("not_a_number".into());
        let result = val.coerce::<u16>(&meta);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("port"));
    }

    #[test]
    fn test_config_value_coerce_secret_hides_raw() {
        let meta = FieldMeta {
            name: "password",
            config_key: "redis.password",
            env_key: "RUSTVELLO__REDIS__PASSWORD",
            generic_env_key: None,
            is_secret: true,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        };
        let val = ConfigValue::String("hunter2".into());
        let result = val.coerce::<u16>(&meta);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("<secret>"));
        assert!(!err_msg.contains("hunter2"));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_config_value_from_toml() {
        let toml_val = toml::Value::Integer(42);
        let cv = ConfigValue::from(toml_val);
        assert_eq!(cv, ConfigValue::Integer(42));

        let toml_str = toml::Value::String("hello".into());
        let cv = ConfigValue::from(toml_str);
        assert_eq!(cv, ConfigValue::String("hello".into()));

        let toml_bool = toml::Value::Boolean(true);
        let cv = ConfigValue::from(toml_bool);
        assert_eq!(cv, ConfigValue::Bool(true));
    }

    #[test]
    fn test_config_value_get_by_dotted_key() {
        let mut inner = IndexMap::new();
        inner.insert("host".into(), ConfigValue::String("localhost".into()));
        inner.insert("port".into(), ConfigValue::Integer(6379));

        let mut root = IndexMap::new();
        root.insert("redis".into(), ConfigValue::Table(inner));

        let table = ConfigValue::Table(root);
        let host = table.get_by_dotted_key("redis.host");
        assert_eq!(host, Some(&ConfigValue::String("localhost".into())));

        let port = table.get_by_dotted_key("redis.port");
        assert_eq!(port, Some(&ConfigValue::Integer(6379)));

        let missing = table.get_by_dotted_key("redis.missing");
        assert!(missing.is_none());
    }
}
