use std::path::PathBuf;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::Source;
use crate::source::ConfigSource;
use crate::value::ConfigValue;

/// Reads config values from `[tool.{prefix}.{group}]` in a `pyproject.toml` file.
///
/// If the file does not exist, or the sections do not exist, it gracefully returns `None`
/// for all fields without producing an error.
///
/// Default rank: **50**.
#[derive(Debug, Clone)]
pub struct PyprojectTomlSource {
    path: PathBuf,
    root: Option<ConfigValue>,
    rank: u8,
    prefix: String,
    group: String,
}

impl PyprojectTomlSource {
    /// Create a new `PyprojectTomlSource` from the given `pyproject.toml` path.
    /// Prefix and group will map to `[tool.{prefix_lowercase}.{group}]`.
    #[cfg(feature = "toml")]
    pub fn new(
        path: impl AsRef<std::path::Path>,
        prefix: &str,
        group: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();
        let root = if path.exists() {
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
            Some(ConfigValue::from(table))
        } else {
            None
        };

        Ok(Self {
            path,
            root,
            rank: 50,
            prefix: prefix.to_lowercase(),
            group: group.to_lowercase(),
        })
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
    }
}

impl ConfigSource for PyprojectTomlSource {
    fn get(&self, meta: &FieldMeta) -> Result<Option<(ConfigValue, Source)>, ConfigError> {
        let Some(root) = &self.root else {
            return Ok(None);
        };

        let tool_key = format!("tool.{}.{}.{}", self.prefix, self.group, meta.name);

        match root.get_by_dotted_key(&tool_key) {
            Some(val) => Ok(Some((
                val.clone(),
                Source::File {
                    path: self.path.clone(),
                    key: tool_key,
                },
            ))),
            None => Ok(None),
        }
    }

    fn rank(&self) -> u8 {
        self.rank
    }
}

#[cfg(all(test, feature = "toml"))]
mod tests {
    use super::*;
    use std::io::Write;

    fn test_meta(name: &'static str) -> FieldMeta {
        FieldMeta {
            name,
            config_key: "doesn'tmatter",
            env_key: "DOESNT_MATTER",
            generic_env_key: None,
            is_secret: false,
            expected_type: "String",
            has_default: true,
            deserialize_fn: None,
        }
    }

    #[test]
    fn test_pyproject_reads_tool_section() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            f,
            "[tool.rustvello.redis]\nhost = \"from-pyproject\"\nport = 6380"
        )
        .unwrap();

        let source = PyprojectTomlSource::new(f.path(), "RUSTVELLO", "redis").unwrap();
        let meta = test_meta("host");

        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("from-pyproject".into()));
        assert!(
            matches!(result.1, Source::File { ref key, .. } if key == "tool.rustvello.redis.host")
        );

        let mut p_meta = test_meta("port");
        p_meta.expected_type = "u16";
        let r2 = source.get(&p_meta).unwrap().unwrap();
        assert_eq!(r2.0, ConfigValue::Integer(6380));
    }

    #[test]
    fn test_pyproject_missing_file() {
        let source =
            PyprojectTomlSource::new("/nonexistent/pyproject.toml", "rustvello", "redis").unwrap();
        let meta = test_meta("host");
        assert!(source.get(&meta).unwrap().is_none());
    }

    #[test]
    fn test_pyproject_no_tool_section() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "[some.other.section]\nhost = \"xyz\"").unwrap();

        let source = PyprojectTomlSource::new(f.path(), "rustvello", "redis").unwrap();
        let meta = test_meta("host");
        assert!(source.get(&meta).unwrap().is_none());
    }

    #[test]
    fn test_pyproject_no_group_section() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "[tool.rustvello]\nother = 1").unwrap();

        let source = PyprojectTomlSource::new(f.path(), "rustvello", "redis").unwrap();
        let meta = test_meta("host");
        assert!(source.get(&meta).unwrap().is_none());
    }

    #[test]
    fn test_pyproject_rank() {
        let f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        let source = PyprojectTomlSource::new(f.path(), "rustvello", "redis").unwrap();
        assert_eq!(source.rank(), 50);
    }

    #[test]
    fn test_pyproject_malformed() {
        let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(f, "invalid [[[").unwrap();

        let result = PyprojectTomlSource::new(f.path(), "rustvello", "redis");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::FileError { .. }));
    }
}
