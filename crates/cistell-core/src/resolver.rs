use std::collections::HashMap;

use crate::config::{default_provenance, Config, ResolvedConfig};
use crate::error::ConfigError;
use crate::field::FieldMeta;
use crate::provenance::{FieldProvenance, Source};
#[cfg(feature = "toml")]
use crate::source::pyproject::PyprojectTomlSource;
use crate::source::FileSource;
use crate::source::{ConfigSource, DefaultSource, EnvSource};
use crate::value::ConfigValue;

/// The result of resolving a set of fields: maps field name
/// to the winning `ConfigValue` plus its `FieldProvenance`.
pub type ResolvedRaw = HashMap<String, (ConfigValue, FieldProvenance)>;

/// Builds a `Resolver` by adding config sources.
pub struct ResolverBuilder {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ResolverBuilder {
    fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a config source. Sources are iterated in rank order (lowest rank wins).
    #[must_use]
    pub fn add_source(mut self, source: impl ConfigSource + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    /// Add an `EnvSource` with default rank (20).
    #[must_use]
    pub fn env(self) -> Self {
        self.add_source(EnvSource::new())
    }

    /// Add a `FileSource` from a file path specified by a class-level environment variable.
    ///
    /// Env var: `{PREFIX}{SEP}{GROUP}{SEP}FILEPATH` (e.g., `PYNENC__BROKER__FILEPATH`).
    /// Default rank: 30.
    pub fn env_file_class(self, prefix: &str, sep: &str, group: &str) -> Result<Self, ConfigError> {
        let env_key = format!(
            "{}{}{}{}FILEPATH",
            prefix.to_uppercase(),
            sep,
            group.to_uppercase(),
            sep,
        );
        if let Ok(path) = std::env::var(&env_key) {
            let source = FileSource::load(path)?.with_rank(30).with_group(group);
            return Ok(self.add_source(source));
        }
        Ok(self)
    }

    /// Add a `FileSource` from a file path specified by a global environment variable.
    ///
    /// Env var: `{PREFIX}{SEP}FILEPATH` (e.g., `PYNENC__FILEPATH`).
    /// Default rank: 35.
    pub fn env_file_global(self, prefix: &str, sep: &str) -> Result<Self, ConfigError> {
        let env_key = format!("{}{}FILEPATH", prefix.to_uppercase(), sep);
        if let Ok(path) = std::env::var(&env_key) {
            let source = FileSource::load(path)?.with_rank(35);
            return Ok(self.add_source(source));
        }
        Ok(self)
    }

    /// Add a `FileSource` using format auto-detection. Default rank (40).
    pub fn file(self, path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let source = FileSource::load(path)?;
        Ok(self.add_source(source))
    }

    /// Add a `PyprojectTomlSource` that looks for `[tool.{prefix_lowercase}.{group_lowercase}]`.
    /// Reads from 'pyproject.toml' in the current working directory.
    /// Default rank (50).
    #[cfg(feature = "toml")]
    pub fn pyproject_toml(self, prefix: &str, group: &str) -> Result<Self, ConfigError> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let path = cwd.join("pyproject.toml");
        let source = PyprojectTomlSource::new(path, prefix, group)?;
        Ok(self.add_source(source))
    }

    /// Add a `DefaultSource` with default rank (100).
    #[must_use]
    pub fn defaults(self, defaults: DefaultSource) -> Self {
        self.add_source(defaults)
    }

    pub fn build(mut self) -> Resolver {
        // Sort by rank ascending (lowest rank = highest priority)
        self.sources.sort_by_key(|s| s.rank());
        Resolver {
            sources: self.sources,
        }
    }
}

/// Resolves config fields from multiple sources, ordered by rank.
///
/// The first source (lowest rank number) that returns a value wins.
/// All other sources that also provide a value are tracked as `rejected_sources`
/// in the provenance record.
pub struct Resolver {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl Resolver {
    pub fn builder() -> ResolverBuilder {
        ResolverBuilder::new()
    }

    /// Creates a complete resolver with standard sources in priority order:
    /// 1. Environment variables (rank 20)
    /// 2. Class-specific file from env (rank 30)
    /// 3. Global file from env (rank 35)
    /// 4. pyproject.toml (rank 50) - if 'toml' feature is enabled
    /// 5. Defaults (rank 100)
    pub fn from_env_and_defaults<T: Config>(
        prefix: &str,
        sep: &str,
        group: &str,
    ) -> Result<Self, ConfigError> {
        #[allow(unused_mut)]
        let mut builder = Self::builder()
            .env()
            .env_file_class(prefix, sep, group)?
            .env_file_global(prefix, sep)?;

        #[cfg(feature = "toml")]
        {
            builder = builder.pyproject_toml(prefix, group)?;
        }

        Ok(builder.build())
    }

    /// Resolve a typed config struct from all configured sources.
    ///
    /// 1. Get field metadata from `T::fields()`
    /// 2. Call `resolve_raw()` to get winning values + provenance
    /// 3. For fields missing from raw results but with defaults (or Option fields), fill via `T::from_values()`
    /// 4. Build the typed struct via `T::from_values()`
    /// 5. Return `ResolvedConfig { value, provenance }`
    pub fn resolve<T: Config>(&self) -> Result<ResolvedConfig<T>, ConfigError> {
        let fields = T::fields();
        let raw = self.resolve_raw(fields)?;

        let mut values = HashMap::with_capacity(fields.len());
        let mut provenance = indexmap::IndexMap::with_capacity(fields.len());

        for field in fields {
            if let Some((val, prov)) = raw.get(field.name) {
                values.insert(field.name.to_owned(), val.clone());
                provenance.insert(field.name.to_owned(), prov.clone());
            } else if field.has_default || field_can_be_missing(field) {
                // Field missing from all sources — use default.
                // Provenance records it as Source::Default.
                let display = if field_can_be_missing(field) && !field.has_default {
                    Some("None".to_owned())
                } else {
                    Some("<default>".to_owned())
                };
                provenance.insert(field.name.to_owned(), default_provenance(field, display));
                // Don't insert into values — from_values() handles missing keys
                // for Option<T> and defaulted fields.
            }
            // If required and missing, resolve_raw already errored with MissingRequired.
        }

        let value = T::from_values(&values)?;
        Ok(ResolvedConfig { value, provenance })
    }

    /// Resolve all fields in the given slice.
    ///
    /// For each field:
    /// 1. Iterate sources in rank order (lowest = highest priority)
    /// 2. The first `Some(value, source)` wins
    /// 3. Remaining sources that also return values go into `rejected_sources`
    /// 4. If no source provides a value and field is required → `MissingRequired`
    pub fn resolve_raw(&self, fields: &[FieldMeta]) -> Result<ResolvedRaw, ConfigError> {
        let mut results = HashMap::with_capacity(fields.len());

        for field in fields {
            let mut winner: Option<(ConfigValue, Source)> = None;
            let mut rejected = Vec::new();

            for source in &self.sources {
                if let Some((val, src)) = source.get(field)? {
                    if winner.is_some() {
                        rejected.push(src);
                    } else {
                        winner = Some((val, src));
                    }
                }
            }

            match winner {
                Some((value, source)) => {
                    let display_value = if field.is_secret {
                        None
                    } else {
                        Some(value.to_string())
                    };

                    let provenance = FieldProvenance {
                        field_name: field.name.to_owned(),
                        source,
                        is_secret: field.is_secret,
                        display_value,
                        rejected_sources: rejected,
                    };

                    results.insert(field.name.to_owned(), (value, provenance));
                }
                None => {
                    if !field.has_default && !field_can_be_missing(field) {
                        return Err(ConfigError::MissingRequired {
                            field: field.name.to_owned(),
                        });
                    }
                    // Field has a default but no source provided a value — that's fine.
                    // The derive macro will use the struct's Default value.
                }
            }
        }

        Ok(results)
    }
}

fn field_can_be_missing(field: &FieldMeta) -> bool {
    let ty: String = field
        .expected_type
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    ty.starts_with("Option<")
        || ty.starts_with("std::option::Option<")
        || ty.starts_with("core::option::Option<")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{DefaultSource, EnvSource, MapSource};

    fn meta_host() -> FieldMeta {
        FieldMeta {
            name: "host",
            config_key: "redis.host",
            env_key: "CISTELL_RESOLVER_TEST__HOST",
            generic_env_key: None,
            is_secret: false,
            expected_type: "String",
            has_default: true,
            deserialize_fn: None,
        }
    }

    fn meta_port() -> FieldMeta {
        FieldMeta {
            name: "port",
            config_key: "redis.port",
            env_key: "CISTELL_RESOLVER_TEST__PORT",
            generic_env_key: None,
            is_secret: false,
            expected_type: "u16",
            has_default: true,
            deserialize_fn: None,
        }
    }

    fn meta_password() -> FieldMeta {
        FieldMeta {
            name: "password",
            config_key: "redis.password",
            env_key: "CISTELL_RESOLVER_TEST__PASSWORD",
            generic_env_key: None,
            is_secret: true,
            expected_type: "String",
            has_default: false,
            deserialize_fn: None,
        }
    }

    #[test]
    fn test_resolve_raw_default_only() {
        let mut defaults = DefaultSource::new();
        defaults.insert("host", ConfigValue::String("localhost".into()));

        let resolver = Resolver::builder().add_source(defaults).build();

        let fields = [meta_host()];
        let result = resolver.resolve_raw(&fields).unwrap();

        let (val, prov) = result.get("host").unwrap();
        assert_eq!(*val, ConfigValue::String("localhost".into()));
        assert_eq!(prov.source, Source::Default);
        assert!(prov.rejected_sources.is_empty());
    }

    #[test]
    fn test_resolve_raw_env_overrides_default() {
        let mut defaults = DefaultSource::new();
        defaults.insert("host", ConfigValue::String("localhost".into()));

        let env = EnvSource::new();

        let resolver = Resolver::builder()
            .add_source(env)
            .add_source(defaults)
            .build();

        let fields = [meta_host()];

        temp_env::with_var("CISTELL_RESOLVER_TEST__HOST", Some("from-env"), || {
            let result = resolver.resolve_raw(&fields).unwrap();
            let (val, prov) = result.get("host").unwrap();
            assert_eq!(*val, ConfigValue::String("from-env".into()));
            assert!(matches!(prov.source, Source::EnvVar { .. }));
            // Default was rejected
            assert_eq!(prov.rejected_sources.len(), 1);
            assert_eq!(prov.rejected_sources[0], Source::Default);
        });
    }

    #[test]
    fn test_resolve_raw_map_overrides_env() {
        let mut map = MapSource::new("overrides");
        map.insert("host", ConfigValue::String("from-map".into()));

        let env = EnvSource::new();

        let resolver = Resolver::builder().add_source(env).add_source(map).build();

        let fields = [meta_host()];

        temp_env::with_var("CISTELL_RESOLVER_TEST__HOST", Some("from-env"), || {
            let result = resolver.resolve_raw(&fields).unwrap();
            let (val, prov) = result.get("host").unwrap();
            assert_eq!(*val, ConfigValue::String("from-map".into()));
            assert!(matches!(prov.source, Source::Map { .. }));
            // Env was rejected
            assert_eq!(prov.rejected_sources.len(), 1);
            assert!(matches!(prov.rejected_sources[0], Source::EnvVar { .. }));
        });
    }

    #[cfg(feature = "toml")]
    #[test]
    fn test_resolve_raw_file_overrides_default() {
        use crate::source::FileSource;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[redis]\nhost = \"from-file\"").unwrap();
        f.flush().unwrap();

        let file_source = FileSource::from_toml(f.path()).unwrap();
        let mut defaults = DefaultSource::new();
        defaults.insert("host", ConfigValue::String("localhost".into()));

        let resolver = Resolver::builder()
            .add_source(file_source)
            .add_source(defaults)
            .build();

        let fields = [meta_host()];
        let result = resolver.resolve_raw(&fields).unwrap();
        let (val, prov) = result.get("host").unwrap();
        assert_eq!(*val, ConfigValue::String("from-file".into()));
        assert!(matches!(prov.source, Source::File { .. }));
        assert_eq!(prov.rejected_sources.len(), 1);
        assert_eq!(prov.rejected_sources[0], Source::Default);
    }

    #[test]
    fn test_resolve_raw_missing_required_errors() {
        let resolver = Resolver::builder().build();

        let fields = [meta_password()];
        temp_env::with_var_unset("CISTELL_RESOLVER_TEST__PASSWORD", || {
            let result = resolver.resolve_raw(&fields);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, ConfigError::MissingRequired { .. }));
            assert!(err.to_string().contains("password"));
        });
    }

    #[test]
    fn test_resolve_raw_provenance_tracks_rejected() {
        let mut map = MapSource::new("overrides");
        map.insert("host", ConfigValue::String("from-map".into()));

        let env = EnvSource::new();

        let mut defaults = DefaultSource::new();
        defaults.insert("host", ConfigValue::String("default-host".into()));

        let resolver = Resolver::builder()
            .add_source(map)
            .add_source(env)
            .add_source(defaults)
            .build();

        let fields = [meta_host()];

        temp_env::with_var("CISTELL_RESOLVER_TEST__HOST", Some("from-env"), || {
            let result = resolver.resolve_raw(&fields).unwrap();
            let (_, prov) = result.get("host").unwrap();
            // Map (rank 10) wins; env (rank 20) and default (rank 100) are rejected
            assert_eq!(prov.rejected_sources.len(), 2);
        });
    }

    #[test]
    fn test_resolve_raw_secret_field_provenance() {
        let mut defaults = DefaultSource::new();
        defaults.insert("password", ConfigValue::String("hunter2".into()));

        let resolver = Resolver::builder().add_source(defaults).build();

        let fields = [meta_password()];
        let result = resolver.resolve_raw(&fields).unwrap();
        let (_, prov) = result.get("password").unwrap();
        assert!(prov.is_secret);
        assert!(prov.display_value.is_none());
    }

    #[test]
    fn test_resolve_raw_multiple_fields() {
        let mut defaults = DefaultSource::new();
        defaults.insert("host", ConfigValue::String("localhost".into()));
        defaults.insert("port", ConfigValue::Integer(6379));

        let resolver = Resolver::builder().add_source(defaults).build();

        let fields = [meta_host(), meta_port()];
        let result = resolver.resolve_raw(&fields).unwrap();

        assert!(result.contains_key("host"));
        assert!(result.contains_key("port"));
        assert_eq!(
            result.get("host").unwrap().0,
            ConfigValue::String("localhost".into())
        );
        assert_eq!(result.get("port").unwrap().0, ConfigValue::Integer(6379));
    }
}
