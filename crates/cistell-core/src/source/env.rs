use std::env;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::Source;
use crate::source::ConfigSource;
use crate::value::ConfigValue;

/// Reads config values from environment variables.
///
/// Uses `meta.env_key` as the variable name (e.g. `"RUSTVELLO__REDIS__HOST"`).
/// All environment values are returned as `ConfigValue::String`.
///
/// Default rank: **20**.
#[derive(Debug, Clone)]
pub struct EnvSource {
    rank: u8,
}

impl EnvSource {
    pub fn new() -> Self {
        Self { rank: 20 }
    }

    pub fn with_rank(rank: u8) -> Self {
        Self { rank }
    }
}

impl Default for EnvSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigSource for EnvSource {
    fn get(&self, meta: &FieldMeta) -> Result<Option<(ConfigValue, Source)>, ConfigError> {
        // Try class-specific env var first (e.g., PREFIX__GROUP__FIELD)
        match env::var(meta.env_key) {
            Ok(val) => {
                return Ok(Some((
                    ConfigValue::String(val),
                    Source::EnvVar {
                        name: meta.env_key.to_owned(),
                    },
                )))
            }
            Err(env::VarError::NotUnicode(_)) => {
                return Err(ConfigError::EnvVar {
                    key: meta.env_key.to_owned(),
                    cause: "value is not valid UTF-8".to_owned(),
                })
            }
            Err(env::VarError::NotPresent) => {}
        }

        // Fallback to generic env var (e.g., PREFIX__FIELD) — matches Python cistell behavior
        if let Some(generic_key) = meta.generic_env_key {
            match env::var(generic_key) {
                Ok(val) => {
                    return Ok(Some((
                        ConfigValue::String(val),
                        Source::EnvVar {
                            name: generic_key.to_owned(),
                        },
                    )))
                }
                Err(env::VarError::NotUnicode(_)) => {
                    return Err(ConfigError::EnvVar {
                        key: generic_key.to_owned(),
                        cause: "value is not valid UTF-8".to_owned(),
                    })
                }
                Err(env::VarError::NotPresent) => {}
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

    fn test_meta() -> FieldMeta {
        FieldMeta {
            name: "host",
            config_key: "redis.host",
            env_key: "CISTELL_TEST__REDIS__HOST",
            generic_env_key: None,
            is_secret: false,
            expected_type: "String",
            has_default: true,
            deserialize_fn: None,
        }
    }

    #[test]
    fn test_env_source_reads_var() {
        let source = EnvSource::new();
        let meta = test_meta();
        temp_env::with_var("CISTELL_TEST__REDIS__HOST", Some("localhost"), || {
            let result = source.get(&meta).unwrap();
            assert!(result.is_some());
            let (value, provenance) = result.unwrap();
            assert_eq!(value, ConfigValue::String("localhost".into()));
            assert_eq!(
                provenance,
                Source::EnvVar {
                    name: "CISTELL_TEST__REDIS__HOST".into()
                }
            );
        });
    }

    #[test]
    fn test_env_source_missing_returns_none() {
        let source = EnvSource::new();
        let meta = test_meta();
        temp_env::with_var_unset("CISTELL_TEST__REDIS__HOST", || {
            let result = source.get(&meta).unwrap();
            assert!(result.is_none());
        });
    }

    #[test]
    fn test_env_source_rank() {
        let source = EnvSource::new();
        assert_eq!(source.rank(), 20);
    }
}
