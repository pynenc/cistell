use std::fmt;
use std::path::PathBuf;

use indexmap::IndexMap;

/// Where a config value came from.
///
/// `#[non_exhaustive]` allows adding new source types in minor versions
/// without breaking downstream `match` arms (semver safety).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Source {
    /// Hardcoded default — declared in the field definition.
    Default,

    /// Loaded from a TOML, YAML, or JSON file.
    File { path: PathBuf, key: String },

    /// Loaded from an environment variable.
    EnvVar { name: String },

    /// Set explicitly in code (programmatic).
    Programmatic { location: Option<String> },

    /// Injected via a `HashMap` (test overrides, `config_values` dict in Python API).
    Map { label: String },
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Default => write!(f, "default"),
            Source::File { path, key } => {
                write!(f, "file '{}' key '{key}'", path.display())
            }
            Source::EnvVar { name } => write!(f, "env var '{name}'"),
            Source::Programmatic {
                location: Some(loc),
            } => write!(f, "code ({loc})"),
            Source::Programmatic { location: None } => write!(f, "code"),
            Source::Map { label } => write!(f, "map '{label}'"),
        }
    }
}

/// The resolved value for one field, bundled with its provenance.
#[derive(Debug, Clone)]
pub struct FieldProvenance {
    pub field_name: String,
    pub source: Source,
    pub is_secret: bool,
    /// String representation of the value — `None` if secret.
    pub display_value: Option<String>,
    /// All sources that were considered but lost to the winner, ordered by priority.
    pub rejected_sources: Vec<Source>,
}

impl fmt::Display for FieldProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = if self.is_secret {
            "<secret>"
        } else {
            self.display_value.as_deref().unwrap_or("<none>")
        };
        write!(f, "{} = {} [from: {}]", self.field_name, val, self.source)
    }
}

/// The complete provenance map for one config struct instance.
/// Uses `IndexMap` to preserve field declaration order.
pub type ProvenanceMap = IndexMap<String, FieldProvenance>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_source_display_default() {
        assert_eq!(Source::Default.to_string(), "default");
    }

    #[test]
    fn test_source_display_file() {
        let s = Source::File {
            path: PathBuf::from("/etc/app.toml"),
            key: "redis.host".into(),
        };
        assert_eq!(s.to_string(), "file '/etc/app.toml' key 'redis.host'");
    }

    #[test]
    fn test_source_display_env() {
        let s = Source::EnvVar {
            name: "MY_VAR".into(),
        };
        assert_eq!(s.to_string(), "env var 'MY_VAR'");
    }

    #[test]
    fn test_source_display_programmatic() {
        let s = Source::Programmatic {
            location: Some("main.rs:42".into()),
        };
        assert_eq!(s.to_string(), "code (main.rs:42)");

        let s2 = Source::Programmatic { location: None };
        assert_eq!(s2.to_string(), "code");
    }

    #[test]
    fn test_source_display_map() {
        let s = Source::Map {
            label: "test_overrides".into(),
        };
        assert_eq!(s.to_string(), "map 'test_overrides'");
    }

    #[test]
    fn test_source_eq_hash() {
        let a = Source::EnvVar {
            name: "MY_VAR".into(),
        };
        let b = Source::EnvVar {
            name: "MY_VAR".into(),
        };
        assert_eq!(a, b);

        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn test_field_provenance_display_normal() {
        let prov = FieldProvenance {
            field_name: "host".into(),
            source: Source::Default,
            is_secret: false,
            display_value: Some("localhost".into()),
            rejected_sources: vec![],
        };
        assert_eq!(prov.to_string(), "host = localhost [from: default]");
    }

    #[test]
    fn test_field_provenance_display_secret() {
        let prov = FieldProvenance {
            field_name: "password".into(),
            source: Source::EnvVar {
                name: "DB_PASS".into(),
            },
            is_secret: true,
            display_value: None,
            rejected_sources: vec![],
        };
        assert_eq!(
            prov.to_string(),
            "password = <secret> [from: env var 'DB_PASS']"
        );
    }
}
