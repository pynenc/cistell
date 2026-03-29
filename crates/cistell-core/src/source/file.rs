use std::path::PathBuf;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::Source;
use crate::source::ConfigSource;
use crate::value::ConfigValue;

/// Reads config values from a parsed config file.
///
/// The file is parsed at construction time (eager I/O).
/// Field lookup walks the dotted `meta.config_key` (e.g. `"redis.host"`)
/// into the parsed table.
///
/// Default rank: **40**.
///
/// Currently supports TOML only. YAML / JSON support will arrive via feature flags.
#[derive(Debug, Clone)]
pub struct FileSource {
    path: PathBuf,
    root: ConfigValue,
    rank: u8,
    /// When set, strip `{group}.` from the config_key before lookup.
    /// Used for class-level file-from-env-var sources.
    group: Option<String>,
}

impl FileSource {
    /// Load a file, autodetecting the format based on the extension.
    ///
    /// Fails with `ConfigError::FileError` if the file cannot be read, parsed,
    /// or if the required feature flag is not enabled for its extension.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "toml" => {
                #[cfg(feature = "toml")]
                return Self::from_toml(path);
                #[cfg(not(feature = "toml"))]
                return Err(ConfigError::FileError {
                    path: path.clone(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "TOML support requires the 'toml' feature flag",
                    )),
                });
            }
            "yaml" | "yml" => {
                #[cfg(feature = "yaml")]
                return Self::from_yaml(path);
                #[cfg(not(feature = "yaml"))]
                return Err(ConfigError::FileError {
                    path: path.clone(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "YAML support requires the 'yaml' feature flag",
                    )),
                });
            }
            "json" => {
                #[cfg(feature = "json")]
                return Self::from_json(path);
                #[cfg(not(feature = "json"))]
                return Err(ConfigError::FileError {
                    path: path.clone(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "JSON support requires the 'json' feature flag",
                    )),
                });
            }
            _ => Err(ConfigError::FileError {
                path: path.clone(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "unsupported format",
                )),
            }),
        }
    }

    /// Parse a TOML config file.
    ///
    /// Fails with `ConfigError::FileError` if the file cannot be read or parsed.
    #[cfg(feature = "toml")]
    pub fn from_toml(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();
        let content = std::fs::read_to_string(&path).map_err(|e| ConfigError::FileError {
            path: path.clone(),
            source: Box::new(e),
        })?;
        let table: toml::Value =
            content
                .parse::<toml::Value>()
                .map_err(|e| ConfigError::FileError {
                    path: path.clone(),
                    source: Box::new(e),
                })?;
        Ok(Self {
            path,
            root: ConfigValue::from(table),
            rank: 40,
            group: None,
        })
    }

    /// Parse a YAML config file.
    #[cfg(feature = "yaml")]
    pub fn from_yaml(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();
        let content = std::fs::read_to_string(&path).map_err(|e| ConfigError::FileError {
            path: path.clone(),
            source: Box::new(e),
        })?;
        let table: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| ConfigError::FileError {
                path: path.clone(),
                source: Box::new(e),
            })?;
        Ok(Self {
            path,
            root: ConfigValue::from(table),
            rank: 40,
            group: None,
        })
    }

    /// Parse a JSON config file.
    #[cfg(feature = "json")]
    pub fn from_json(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();
        let content = std::fs::read_to_string(&path).map_err(|e| ConfigError::FileError {
            path: path.clone(),
            source: Box::new(e),
        })?;
        let table: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| ConfigError::FileError {
                path: path.clone(),
                source: Box::new(e),
            })?;
        Ok(Self {
            path,
            root: ConfigValue::from(table),
            rank: 40,
            group: None,
        })
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
    }

    /// Scope lookups to a specific group.
    ///
    /// When set, `{group}.` is stripped from the config_key before lookup.
    /// This is used for class-level file-from-env-var, where the file contains
    /// flat keys for a single group (e.g., `host = "..."` instead of `[redis]\nhost = "..."`).
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into().to_lowercase());
        self
    }
}

impl ConfigSource for FileSource {
    fn get(&self, meta: &FieldMeta) -> Result<Option<(ConfigValue, Source)>, ConfigError> {
        let lookup_key = if let Some(ref group) = self.group {
            let prefix = format!("{}.", group);
            meta.config_key
                .strip_prefix(&prefix)
                .unwrap_or(meta.config_key)
        } else {
            meta.config_key
        };

        // Try the full config_key (group-specific nested key, e.g. "redis.host")
        if let Some(val) = self.root.get_by_dotted_key(lookup_key) {
            return Ok(Some((
                val.clone(),
                Source::File {
                    path: self.path.clone(),
                    key: meta.config_key.to_owned(),
                },
            )));
        }

        // For global files (no group set), try flat field name as fallback
        // when the config_key is a dotted path (e.g. "redis.host" → try "host").
        // This matches Python cistell behavior where files with flat keys
        // like {"host": "value"} work for any config class.
        if self.group.is_none() && lookup_key.contains('.') {
            if let Some(val) = self.root.get_by_dotted_key(meta.name) {
                return Ok(Some((
                    val.clone(),
                    Source::File {
                        path: self.path.clone(),
                        key: meta.name.to_owned(),
                    },
                )));
            }
        }

        Ok(None)
    }

    fn rank(&self) -> u8 {
        self.rank
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "toml", feature = "yaml", feature = "json"))]
    use std::io::Write;

    #[cfg(any(feature = "toml", feature = "yaml", feature = "json"))]
    fn test_meta(config_key: &'static str) -> FieldMeta {
        FieldMeta {
            name: "host",
            config_key,
            env_key: "REDIS__HOST",
            generic_env_key: None,
            is_secret: false,
            expected_type: "String",
            has_default: true,
            deserialize_fn: None,
        }
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_file_source_toml_reads_key() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "host = \"localhost\"").unwrap();

        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_file_source_toml_nested_key() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            f,
            "[redis]
host = \"127.0.0.1\"
port = 6379"
        )
        .unwrap();

        let source = FileSource::load(f.path()).unwrap();

        let meta = test_meta("redis.host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("127.0.0.1".into()));

        let meta_port = FieldMeta {
            name: "port",
            config_key: "redis.port",
            env_key: "REDIS__PORT",
            generic_env_key: None,
            is_secret: false,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        };
        let result_port = source.get(&meta_port).unwrap().unwrap();
        assert_eq!(result_port.0, ConfigValue::Integer(6379));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_file_source_toml_missing_key() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "host = \"localhost\"").unwrap();

        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("nonexistent");
        assert!(source.get(&meta).unwrap().is_none());
    }

    #[test]
    fn test_file_source_missing_file() {
        let result = FileSource::load("/nonexistent/path.toml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::FileError { .. }));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_file_source_invalid_toml() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "not valid toml [[[").unwrap();

        let result = FileSource::load(f.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::FileError { .. }));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_file_source_rank() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "x = 1").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        assert_eq!(source.rank(), 40);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml_file_source_reads_key() {
        let mut f = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(f, "host: localhost").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml_file_source_nested_key() {
        let mut f = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
        writeln!(
            f,
            "redis:
  host: '127.0.0.1'
  port: 6379"
        )
        .unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("redis.host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("127.0.0.1".into()));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml_file_source_array() {
        let mut f = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
        writeln!(
            f,
            "hosts:
  - a
  - b"
        )
        .unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("hosts");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(
            result.0,
            ConfigValue::Array(vec![
                ConfigValue::String("a".into()),
                ConfigValue::String("b".into())
            ])
        );
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml_file_source_anchor() {
        let mut f = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
        writeln!(
            f,
            "template: &T
  host: a
redis: *T"
        )
        .unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("redis.host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("a".into()));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_file_source_reads_key() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(f, "{{\"host\": \"localhost\"}}").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_file_source_nested_key() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(
            f,
            "{{\"redis\": {{\"host\": \"127.0.0.1\", \"port\": 6379}}}}"
        )
        .unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("redis.host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("127.0.0.1".into()));

        let mut m2 = test_meta("redis.port");
        m2.expected_type = "u16";
        let r2 = source.get(&m2).unwrap().unwrap();
        assert_eq!(r2.0, ConfigValue::Integer(6379));
    }

    #[cfg(feature = "json")]
    #[test]
    #[allow(clippy::approx_constant)]
    fn test_json_file_source_float() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(f, "{{\"val\": 3.14}}").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let mut meta = test_meta("val");
        meta.expected_type = "f64";
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::Float(3.14));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_file_source_integer_preserved() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(f, "{{\"count\": 42}}").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let mut meta = test_meta("count");
        meta.expected_type = "i64";
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::Integer(42));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_file_source_array() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(f, "{{\"hosts\": [\"a\", \"b\"]}}").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("hosts");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(
            result.0,
            ConfigValue::Array(vec![
                ConfigValue::String("a".into()),
                ConfigValue::String("b".into())
            ])
        );
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_json_file_source_invalid() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(f, "not valid json {{{{").unwrap();
        let result = FileSource::load(f.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::FileError { .. }));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml_file_source_missing_key() {
        let mut f = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(f, "host: localhost").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("nonexistent");
        assert!(source.get(&meta).unwrap().is_none());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_yaml_file_source_invalid() {
        let mut f = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(f, "  bad:\nyaml: [[[").unwrap();
        let result = FileSource::load(f.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::FileError { .. }));
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_auto_detect_toml() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "host = \"localhost\"").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_auto_detect_yaml() {
        let mut f = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(f, "host: localhost").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_auto_detect_yml() {
        let mut f = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
        writeln!(f, "host: localhost").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_auto_detect_json() {
        let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(f, "{{\"host\": \"localhost\"}}").unwrap();
        let source = FileSource::load(f.path()).unwrap();
        let meta = test_meta("host");
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("localhost".into()));
    }

    #[test]
    fn test_auto_detect_unsupported() {
        let f = tempfile::Builder::new().suffix(".xml").tempfile().unwrap();
        let result = FileSource::load(f.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unsupported format"));
    }

    #[test]
    fn test_auto_detect_no_extension() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let result = FileSource::load(f.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unsupported format"));
    }

    #[cfg(not(feature = "yaml"))]
    #[test]
    fn test_feature_gate_yaml_disabled() {
        use std::io::Write;

        let mut f = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(f, "host: localhost").unwrap();
        let result = FileSource::load(f.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("YAML support requires the 'yaml' feature flag"),
            "got: {err}"
        );
    }
}
