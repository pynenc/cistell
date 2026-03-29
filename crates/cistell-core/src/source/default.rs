use std::collections::HashMap;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::Source;
use crate::source::ConfigSource;
use crate::value::ConfigValue;

/// Provides default values as the lowest-priority fallback.
///
/// Stores a `HashMap<field_name, ConfigValue>`. Looked up by `meta.name`.
///
/// Default rank: **100** (lowest priority among built-in sources).
#[derive(Debug, Clone)]
pub struct DefaultSource {
    defaults: HashMap<String, ConfigValue>,
    rank: u8,
}

impl DefaultSource {
    pub fn new() -> Self {
        Self {
            defaults: HashMap::new(),
            rank: 100,
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: ConfigValue) -> &mut Self {
        self.defaults.insert(key.into(), value);
        self
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
    }
}

impl Default for DefaultSource {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigSource for DefaultSource {
    fn get(&self, meta: &FieldMeta) -> Result<Option<(ConfigValue, Source)>, ConfigError> {
        match self.defaults.get(meta.name) {
            Some(val) => Ok(Some((val.clone(), Source::Default))),
            None => Ok(None),
        }
    }

    fn rank(&self) -> u8 {
        self.rank
    }
}
