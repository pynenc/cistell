use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::{FieldProvenance, ProvenanceMap, Source};
use crate::value::ConfigValue;

/// The core trait that config structs implement.
///
/// In Phase 2, `#[derive(Config)]` generates this automatically.
/// For now, implement it manually.
pub trait Config: Sized {
    /// Static metadata for every field in this config struct.
    fn fields() -> &'static [FieldMeta];

    /// Construct the struct with all default values.
    /// Fields without defaults should use the type's `Default` (e.g., `None` for `Option<T>`).
    fn defaults() -> Self;

    /// Build a concrete instance from a map of field_name → ConfigValue.
    ///
    /// Each field calls `ConfigValue::coerce::<FieldType>()` or applies a custom
    /// `deserialize_fn` if one is set on the `FieldMeta`.
    fn from_values(values: &HashMap<String, ConfigValue>) -> Result<Self, ConfigError>;
}

/// A resolved config struct bundled with its full provenance.
///
/// Implements `Deref<Target = T>` so fields can be accessed directly:
/// ```ignore
/// let resolved = resolver.resolve::<MyConfig>()?;
/// println!("{}", resolved.host); // Deref to MyConfig
/// ```
pub struct ResolvedConfig<T> {
    pub value: T,
    pub provenance: ProvenanceMap,
}

impl<T> Deref for ResolvedConfig<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> ResolvedConfig<T> {
    /// Human-readable provenance report for every resolved field.
    ///
    /// Secret fields show `<secret>` instead of the actual value.
    pub fn explain(&self) -> String {
        let mut lines = Vec::with_capacity(self.provenance.len());
        for (_, prov) in &self.provenance {
            lines.push(prov.to_string());
        }
        lines.join("\n")
    }

    /// Emit a `tracing::debug!` event for each resolved field.
    ///
    /// Secret fields are suppressed: the value is shown as `<secret>`.
    #[cfg(feature = "tracing")]
    pub fn log_provenance(&self) {
        for (_, prov) in &self.provenance {
            let val = if prov.is_secret {
                "<secret>"
            } else {
                prov.display_value.as_deref().unwrap_or("<none>")
            };
            ::tracing::debug!(
                field = %prov.field_name,
                value = %val,
                source = %prov.source,
                "config field resolved"
            );
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for ResolvedConfig<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedConfig")
            .field("value", &self.value)
            .field("provenance", &self.provenance)
            .finish()
    }
}

/// Helper: build a `FieldProvenance` entry for a field that kept its default value
/// (no source provided a value).
pub(crate) fn default_provenance(
    field: &FieldMeta,
    display_value: Option<String>,
) -> FieldProvenance {
    FieldProvenance {
        field_name: field.name.to_owned(),
        source: Source::Default,
        is_secret: field.is_secret,
        display_value,
        rejected_sources: Vec::new(),
    }
}
