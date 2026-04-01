pub mod default;
pub mod env;
pub mod file;
pub mod map;
pub mod pyproject;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::Source;
use crate::value::ConfigValue;

/// A pluggable source of config values.
///
/// Each source has a **rank** (lower = higher priority).
/// The resolver iterates sources in rank order and the first
/// source that returns `Some(...)` wins.
pub trait ConfigSource: Send + Sync {
    /// Look up the value for one field.
    ///
    /// Returns `Ok(None)` when the source has no opinion about this field.
    fn get(&self, meta: &FieldMeta) -> Result<Option<(ConfigValue, Source)>, ConfigError>;

    /// Priority rank: lower values win over higher values.
    fn rank(&self) -> u8;
}

pub use default::DefaultSource;
pub use env::EnvSource;
pub use file::FileSource;
pub use map::MapSource;
pub use pyproject::PyprojectTomlSource;
