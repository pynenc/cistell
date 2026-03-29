use std::path::PathBuf;

use crate::field::FieldMeta;
use crate::provenance::Source;

/// All errors that cistell-core can produce.
///
/// `#[non_exhaustive]` so we can add variants in minor versions
/// without breaking downstream `match` arms.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConfigError {
    /// A value could not be parsed into the expected type.
    ParseError {
        field: String,
        /// The raw value (or `"<secret>"` if the field is secret).
        raw_display: String,
        expected_type: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// A config file could not be read or parsed.
    FileError {
        path: PathBuf,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// An environment variable exists but is not valid UTF-8.
    EnvVar { key: String, cause: String },

    /// Two sources at the same rank provide conflicting values.
    FieldConflict { field: String, a: Source, b: Source },

    /// A required field has no default and no source provided a value.
    MissingRequired { field: String },

    /// An internal invariant was violated (should never happen in normal operation).
    Internal { message: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ParseError {
                field,
                raw_display,
                expected_type,
                source,
            } => write!(
                f,
                "field '{field}': cannot parse '{raw_display}' as {expected_type}: {source}"
            ),
            ConfigError::FileError { path, source } => {
                write!(f, "config file '{}': {source}", path.display())
            }
            ConfigError::EnvVar { key, cause } => {
                write!(f, "env var '{key}': {cause}")
            }
            ConfigError::FieldConflict { field, a, b } => {
                write!(f, "field '{field}': conflicting values from {a} and {b}")
            }
            ConfigError::MissingRequired { field } => {
                write!(f, "field '{field}': required but no value provided")
            }
            ConfigError::Internal { message } => {
                write!(f, "internal error: {message}")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::ParseError { source, .. } => Some(source.as_ref()),
            ConfigError::FileError { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl ConfigError {
    /// Create a `ParseError`, automatically redacting the raw value when `meta.is_secret`.
    pub fn parse<E>(meta: &FieldMeta, raw: &str, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ConfigError::ParseError {
            field: meta.name.to_owned(),
            raw_display: if meta.is_secret {
                "<secret>".to_owned()
            } else {
                raw.to_owned()
            },
            expected_type: meta.expected_type.to_owned(),
            source: Box::new(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::num::ParseIntError;

    fn test_meta(is_secret: bool) -> FieldMeta {
        FieldMeta {
            name: "port",
            config_key: "redis.port",
            env_key: "REDIS__PORT",
            generic_env_key: None,
            is_secret,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        }
    }

    #[test]
    fn test_error_parse_redacts_secret() {
        let meta = test_meta(true);
        let parse_err: ParseIntError = "abc".parse::<u16>().unwrap_err();
        let err = ConfigError::parse(&meta, "hunter2", parse_err);
        let msg = err.to_string();
        assert!(msg.contains("<secret>"), "should contain <secret>: {msg}");
        assert!(!msg.contains("hunter2"), "should NOT leak raw: {msg}");
    }

    #[test]
    fn test_error_parse_shows_raw_for_non_secret() {
        let meta = test_meta(false);
        let parse_err: ParseIntError = "abc".parse::<u16>().unwrap_err();
        let err = ConfigError::parse(&meta, "bad_value", parse_err);
        let msg = err.to_string();
        assert!(msg.contains("bad_value"), "should show raw: {msg}");
    }

    #[test]
    fn test_error_display_messages() {
        let meta = test_meta(false);
        let parse_err: ParseIntError = "abc".parse::<u16>().unwrap_err();

        // ParseError
        let err = ConfigError::parse(&meta, "abc", parse_err);
        let msg = err.to_string();
        assert!(msg.contains("port"));
        assert!(msg.contains("abc"));
        assert!(msg.contains("u16"));

        // FileError
        let err = ConfigError::FileError {
            path: PathBuf::from("/etc/config.toml"),
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )),
        };
        assert!(err.to_string().contains("/etc/config.toml"));

        // EnvVar
        let err = ConfigError::EnvVar {
            key: "MY_VAR".into(),
            cause: "not valid unicode".into(),
        };
        assert!(err.to_string().contains("MY_VAR"));

        // FieldConflict
        let err = ConfigError::FieldConflict {
            field: "host".into(),
            a: Source::Default,
            b: Source::EnvVar {
                name: "HOST".into(),
            },
        };
        assert!(err.to_string().contains("host"));

        // MissingRequired
        let err = ConfigError::MissingRequired {
            field: "port".into(),
        };
        assert!(err.to_string().contains("port"));
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn test_error_source_chain() {
        let meta = test_meta(false);
        let parse_err: ParseIntError = "abc".parse::<u16>().unwrap_err();
        let err = ConfigError::parse(&meta, "abc", parse_err);
        assert!(err.source().is_some());

        let err = ConfigError::MissingRequired {
            field: "port".into(),
        };
        assert!(err.source().is_none());
    }
}
