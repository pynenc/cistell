use std::collections::HashMap;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::Source;
use crate::source::ConfigSource;
use crate::value::ConfigValue;

/// A config source backed by an in-memory `HashMap`.
///
/// Useful for test overrides, programmatic overrides, and the Python
/// `config_values` dict API.
///
/// Lookup is by `meta.name` (the struct field name).
///
/// Default rank: **10** (highest built-in priority — overrides everything).
#[derive(Debug, Clone)]
pub struct MapSource {
    label: String,
    entries: HashMap<String, ConfigValue>,
    rank: u8,
}

impl MapSource {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            entries: HashMap::new(),
            rank: 10,
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: ConfigValue) -> &mut Self {
        self.entries.insert(key.into(), value);
        self
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
    }
}

impl ConfigSource for MapSource {
    fn get(&self, meta: &FieldMeta) -> Result<Option<(ConfigValue, Source)>, ConfigError> {
        match self.entries.get(meta.name) {
            Some(val) => Ok(Some((
                val.clone(),
                Source::Map {
                    label: self.label.clone(),
                },
            ))),
            None => Ok(None),
        }
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
            env_key: "REDIS__HOST",
            generic_env_key: None,
            is_secret: false,
            expected_type: "String",
            has_default: true,
            deserialize_fn: None,
        }
    }

    #[test]
    fn test_map_source_returns_inserted() {
        let mut source = MapSource::new("test_overrides");
        source.insert("host", ConfigValue::String("override-host".into()));

        let meta = test_meta();
        let result = source.get(&meta).unwrap().unwrap();
        assert_eq!(result.0, ConfigValue::String("override-host".into()));
        assert_eq!(
            result.1,
            Source::Map {
                label: "test_overrides".into()
            }
        );
    }

    #[test]
    fn test_map_source_missing_key() {
        let source = MapSource::new("empty");
        let meta = test_meta();
        assert!(source.get(&meta).unwrap().is_none());
    }

    #[test]
    fn test_map_source_rank() {
        let source = MapSource::new("test");
        assert_eq!(source.rank(), 10);
    }
}
