//! Manual `impl Config for RedisConfig` — Phase 1 reference implementation.
//!
//! This serves as both test fixture and a reference for what the derive macro
//! will generate in Phase 2.

use std::collections::HashMap;
use std::time::Duration;

use cistell_core::{
    duration_from_secs, Config, ConfigError, ConfigValue, FieldMeta, ResolvedConfig, Secret,
};

/// Example config struct with various field types:
/// - `String`, `u16` — basic scalars
/// - `Secret<String>` — redacted field
/// - `Option<String>` — optional field
/// - `Vec<String>` — list field (comma-separated from env)
/// - `Duration` — custom deserializer
#[derive(Debug)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Secret<String>,
    pub sentinel_host: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub timeout: Duration,
}

static REDIS_FIELDS: [FieldMeta; 6] = [
    FieldMeta {
        name: "host",
        config_key: "redis.host",
        env_key: "CISTELL_TEST__REDIS__HOST",
        generic_env_key: None,
        is_secret: false,
        expected_type: "String",
        has_default: true,
        deserialize_fn: None,
    },
    FieldMeta {
        name: "port",
        config_key: "redis.port",
        env_key: "CISTELL_TEST__REDIS__PORT",
        generic_env_key: None,
        is_secret: false,
        expected_type: "u16",
        has_default: true,
        deserialize_fn: None,
    },
    FieldMeta {
        name: "password",
        config_key: "redis.password",
        env_key: "CISTELL_TEST__REDIS__PASSWORD",
        generic_env_key: None,
        is_secret: true,
        expected_type: "String",
        has_default: false,
        deserialize_fn: None,
    },
    FieldMeta {
        name: "sentinel_host",
        config_key: "redis.sentinel_host",
        env_key: "CISTELL_TEST__REDIS__SENTINEL_HOST",
        generic_env_key: None,
        is_secret: false,
        expected_type: "Option<String>",
        has_default: true,
        deserialize_fn: None,
    },
    FieldMeta {
        name: "allowed_hosts",
        config_key: "redis.allowed_hosts",
        env_key: "CISTELL_TEST__REDIS__ALLOWED_HOSTS",
        generic_env_key: None,
        is_secret: false,
        expected_type: "Vec<String>",
        has_default: true,
        deserialize_fn: None,
    },
    FieldMeta {
        name: "timeout",
        config_key: "redis.timeout",
        env_key: "CISTELL_TEST__REDIS__TIMEOUT",
        generic_env_key: None,
        is_secret: false,
        expected_type: "Duration",
        has_default: true,
        deserialize_fn: Some(duration_from_secs),
    },
];

impl Config for RedisConfig {
    fn fields() -> &'static [FieldMeta] {
        &REDIS_FIELDS
    }

    fn defaults() -> Self {
        RedisConfig {
            host: "localhost".to_owned(),
            port: 6379,
            password: Secret::new(String::new()),
            sentinel_host: None,
            allowed_hosts: Vec::new(),
            timeout: Duration::from_secs(30),
        }
    }

    fn from_values(values: &HashMap<String, ConfigValue>) -> Result<Self, ConfigError> {
        let fields = Self::fields();
        let defaults = Self::defaults();

        let host = match values.get("host") {
            Some(v) => v.coerce_string(&fields[0])?,
            None => defaults.host,
        };

        let port = match values.get("port") {
            Some(v) => v.coerce::<u16>(&fields[1])?,
            None => defaults.port,
        };

        let password = match values.get("password") {
            Some(v) => Secret::new(v.coerce_string(&fields[2])?),
            None => {
                return Err(ConfigError::MissingRequired {
                    field: "password".to_owned(),
                });
            }
        };

        let sentinel_host = match values.get("sentinel_host") {
            Some(v) => Some(v.coerce_string(&fields[3])?),
            None => None,
        };

        let allowed_hosts = match values.get("allowed_hosts") {
            Some(v) => v.coerce_vec::<String>(&fields[4])?,
            None => defaults.allowed_hosts,
        };

        let timeout = match values.get("timeout") {
            Some(v) => {
                // Apply custom deserializer if present
                let converted = if let Some(f) = fields[5].deserialize_fn {
                    f(v, &fields[5])?
                } else {
                    v.clone()
                };
                let secs: u64 = converted.coerce(&fields[5])?;
                Duration::from_secs(secs)
            }
            None => defaults.timeout,
        };

        Ok(RedisConfig {
            host,
            port,
            password,
            sentinel_host,
            allowed_hosts,
            timeout,
        })
    }
}

// ─── Config Trait Tests ───

#[test]
fn test_manual_config_fields_static() {
    let fields = RedisConfig::fields();
    assert_eq!(fields.len(), 6);
    assert_eq!(fields[0].name, "host");
    assert_eq!(fields[1].name, "port");
    assert_eq!(fields[2].name, "password");
    assert!(fields[2].is_secret);
    assert_eq!(fields[3].name, "sentinel_host");
    assert_eq!(fields[4].name, "allowed_hosts");
    assert_eq!(fields[5].name, "timeout");
    assert!(fields[5].deserialize_fn.is_some());
}

#[test]
fn test_manual_config_defaults() {
    let d = RedisConfig::defaults();
    assert_eq!(d.host, "localhost");
    assert_eq!(d.port, 6379);
    assert!(d.sentinel_host.is_none());
    assert!(d.allowed_hosts.is_empty());
    assert_eq!(d.timeout, Duration::from_secs(30));
}

#[test]
fn test_manual_config_from_values() {
    let mut values = HashMap::new();
    values.insert(
        "host".to_owned(),
        ConfigValue::String("redis.example.com".into()),
    );
    values.insert("port".to_owned(), ConfigValue::Integer(6380));
    values.insert("password".to_owned(), ConfigValue::String("hunter2".into()));
    values.insert(
        "sentinel_host".to_owned(),
        ConfigValue::String("sentinel.local".into()),
    );
    values.insert(
        "allowed_hosts".to_owned(),
        ConfigValue::Array(vec![
            ConfigValue::String("h1".into()),
            ConfigValue::String("h2".into()),
        ]),
    );
    values.insert("timeout".to_owned(), ConfigValue::Integer(60));

    let config = RedisConfig::from_values(&values).unwrap();
    assert_eq!(config.host, "redis.example.com");
    assert_eq!(config.port, 6380);
    assert_eq!(*config.password.expose(), "hunter2");
    assert_eq!(config.sentinel_host, Some("sentinel.local".to_owned()));
    assert_eq!(config.allowed_hosts, vec!["h1", "h2"]);
    assert_eq!(config.timeout, Duration::from_secs(60));
}

#[test]
fn test_manual_config_from_values_missing_required() {
    // Missing password (required) should error
    let mut values = HashMap::new();
    values.insert("host".to_owned(), ConfigValue::String("localhost".into()));
    let result = RedisConfig::from_values(&values);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::MissingRequired { .. }));
    assert!(err.to_string().contains("password"));
}

#[test]
fn test_manual_config_from_values_option_missing_ok() {
    // sentinel_host is Option — should be None when missing, not error
    let mut values = HashMap::new();
    values.insert("host".to_owned(), ConfigValue::String("localhost".into()));
    values.insert("port".to_owned(), ConfigValue::Integer(6379));
    values.insert("password".to_owned(), ConfigValue::String("secret".into()));
    let config = RedisConfig::from_values(&values).unwrap();
    assert!(config.sentinel_host.is_none());
    assert!(config.allowed_hosts.is_empty());
}

// ─── Resolver::resolve::<T>() Tests ───

use cistell_core::{provenance::Source, DefaultSource, MapSource, Resolver};

fn make_redis_defaults() -> DefaultSource {
    let mut defaults = DefaultSource::new();
    defaults.insert("host", ConfigValue::String("localhost".into()));
    defaults.insert("port", ConfigValue::Integer(6379));
    defaults.insert("password", ConfigValue::String("default-pass".into()));
    defaults.insert("timeout", ConfigValue::Integer(30));
    defaults
}

#[test]
fn test_resolve_defaults_only() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    assert_eq!(resolved.host, "localhost");
    assert_eq!(resolved.port, 6379);
    assert_eq!(*resolved.password.expose(), "default-pass");
    assert!(resolved.sentinel_host.is_none());
    assert!(resolved.allowed_hosts.is_empty());
    assert_eq!(resolved.timeout, Duration::from_secs(30));

    // Check provenance for host
    let host_prov = resolved.provenance.get("host").unwrap();
    assert_eq!(host_prov.source, Source::Default);
}

#[test]
fn test_resolve_env_overrides() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_vars(
        [
            ("CISTELL_TEST__REDIS__HOST", Some("from-env")),
            ("CISTELL_TEST__REDIS__PORT", Some("9999")),
        ],
        || {
            let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
            assert_eq!(resolved.host, "from-env");
            assert_eq!(resolved.port, 9999);

            let host_prov = resolved.provenance.get("host").unwrap();
            assert!(matches!(host_prov.source, Source::EnvVar { .. }));
            assert_eq!(host_prov.rejected_sources.len(), 1);
            assert_eq!(host_prov.rejected_sources[0], Source::Default);
        },
    );
}

#[test]
fn test_resolve_map_overrides_env() {
    let mut map = MapSource::new("test-overrides");
    map.insert("host", ConfigValue::String("from-map".into()));

    let resolver = Resolver::builder()
        .add_source(map)
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__HOST", Some("from-env"), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
        assert_eq!(resolved.host, "from-map");

        let host_prov = resolved.provenance.get("host").unwrap();
        assert!(matches!(host_prov.source, Source::Map { .. }));
        // Env and Default were both rejected
        assert_eq!(host_prov.rejected_sources.len(), 2);
    });
}

#[cfg(feature = "toml")]
#[test]
fn test_resolve_file_overrides_default() {
    use cistell_core::FileSource;
    use std::io::Write;

    let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(f, "[redis]\nhost = \"from-file\"\npassword = \"file-pass\"").unwrap();
    f.flush().unwrap();

    let file_source = FileSource::from_toml(f.path()).unwrap();
    let resolver = Resolver::builder()
        .add_source(file_source)
        .defaults(make_redis_defaults())
        .build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    assert_eq!(resolved.host, "from-file");
    assert_eq!(*resolved.password.expose(), "file-pass");

    let host_prov = resolved.provenance.get("host").unwrap();
    assert!(matches!(host_prov.source, Source::File { .. }));
    assert_eq!(host_prov.rejected_sources.len(), 1);
    assert_eq!(host_prov.rejected_sources[0], Source::Default);
}

#[cfg(feature = "toml")]
#[test]
fn test_resolve_full_priority_chain() {
    use cistell_core::FileSource;
    use std::io::Write;

    let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        f,
        "[redis]\nhost = \"from-file\"\nport = 1111\npassword = \"file-pass\""
    )
    .unwrap();
    f.flush().unwrap();
    let file_source = FileSource::from_toml(f.path()).unwrap();

    let mut map = MapSource::new("overrides");
    map.insert("host", ConfigValue::String("from-map".into()));

    let resolver = Resolver::builder()
        .add_source(map)
        .env()
        .add_source(file_source)
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__PORT", Some("2222"), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();

        // host: map wins (rank 10) over env (not set), file (40), default (100)
        assert_eq!(resolved.host, "from-map");
        // port: env (rank 20) wins over file (1111, rank 40) and default (6379, rank 100)
        assert_eq!(resolved.port, 2222);
        // password: file (rank 40) wins over default (100)
        assert_eq!(*resolved.password.expose(), "file-pass");
        // timeout: default only
        assert_eq!(resolved.timeout, Duration::from_secs(30));

        // Verify provenance for port
        let port_prov = resolved.provenance.get("port").unwrap();
        assert!(matches!(port_prov.source, Source::EnvVar { .. }));
        assert!(port_prov.rejected_sources.len() >= 2); // file + default
    });
}

#[test]
fn test_resolve_missing_required_field() {
    // No password source at all and no default in DefaultSource
    let mut defaults = DefaultSource::new();
    defaults.insert("host", ConfigValue::String("localhost".into()));
    // password is NOT in defaults and has_default=false

    let resolver = Resolver::builder().defaults(defaults).build();

    temp_env::with_var_unset("CISTELL_TEST__REDIS__PASSWORD", || {
        let result = resolver.resolve::<RedisConfig>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::MissingRequired { .. }));
        assert!(err.to_string().contains("password"));
    });
}

#[test]
fn test_resolve_secret_field_provenance() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();

    let pass_prov = resolved.provenance.get("password").unwrap();
    assert!(pass_prov.is_secret);
    assert!(pass_prov.display_value.is_none()); // Never shown
}

#[test]
fn test_resolve_rejected_sources_tracked() {
    let mut map = MapSource::new("overrides");
    map.insert("host", ConfigValue::String("from-map".into()));

    let resolver = Resolver::builder()
        .add_source(map)
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__HOST", Some("from-env"), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
        let host_prov = resolved.provenance.get("host").unwrap();
        // Map wins; env + default rejected
        assert_eq!(host_prov.rejected_sources.len(), 2);
        assert!(matches!(
            host_prov.rejected_sources[0],
            Source::EnvVar { .. }
        ));
        assert_eq!(host_prov.rejected_sources[1], Source::Default);
    });
}

// ─── ResolvedConfig Tests ───

#[test]
fn test_resolved_config_deref() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    // Deref<Target = RedisConfig> — direct field access
    let host: &str = &resolved.host;
    assert_eq!(host, "localhost");
    let port: u16 = resolved.port;
    assert_eq!(port, 6379);
}

#[test]
fn test_resolved_config_explain() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    let explanation = resolved.explain();

    // Should contain field names and their sources
    assert!(explanation.contains("host"));
    assert!(explanation.contains("default"));
    // Secret field should show <secret>
    assert!(explanation.contains("<secret>"));
    // Should not contain the actual password value
    assert!(!explanation.contains("default-pass"));
}

#[cfg(feature = "tracing")]
#[test]
fn test_resolved_config_log_provenance() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    // Should not panic
    resolved.log_provenance();
}

// ─── Option<T> Resolution Tests ───

#[test]
fn test_option_field_none_when_missing() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    assert!(resolved.sentinel_host.is_none());

    // Provenance should show Source::Default for the missing Option field
    let prov = resolved.provenance.get("sentinel_host").unwrap();
    assert_eq!(prov.source, Source::Default);
}

#[test]
fn test_option_field_some_from_env() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var(
        "CISTELL_TEST__REDIS__SENTINEL_HOST",
        Some("sentinel.local"),
        || {
            let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
            assert_eq!(resolved.sentinel_host, Some("sentinel.local".to_owned()));

            let prov = resolved.provenance.get("sentinel_host").unwrap();
            assert!(matches!(prov.source, Source::EnvVar { .. }));
        },
    );
}

#[test]
fn test_option_field_provenance_none() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    let prov = resolved.provenance.get("sentinel_host").unwrap();
    assert_eq!(prov.source, Source::Default);
    assert_eq!(prov.display_value.as_deref(), Some("<default>"));
}

// ─── Vec<T> Resolution Tests ───

#[test]
fn test_vec_from_toml_array() {
    let mut map = MapSource::new("test");
    map.insert(
        "allowed_hosts",
        ConfigValue::Array(vec![
            ConfigValue::String("host1".into()),
            ConfigValue::String("host2".into()),
        ]),
    );

    let resolver = Resolver::builder()
        .add_source(map)
        .defaults(make_redis_defaults())
        .build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    assert_eq!(resolved.allowed_hosts, vec!["host1", "host2"]);
}

#[test]
fn test_vec_from_env_comma_separated() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__ALLOWED_HOSTS", Some("a,b,c"), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
        assert_eq!(resolved.allowed_hosts, vec!["a", "b", "c"]);
    });
}

#[test]
fn test_vec_from_env_with_spaces() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var(
        "CISTELL_TEST__REDIS__ALLOWED_HOSTS",
        Some("a, b , c"),
        || {
            let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
            assert_eq!(resolved.allowed_hosts, vec!["a", "b", "c"]);
        },
    );
}

#[test]
fn test_vec_from_env_empty_string() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__ALLOWED_HOSTS", Some(""), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
        assert!(resolved.allowed_hosts.is_empty());
    });
}

#[test]
fn test_vec_from_env_trailing_comma() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__ALLOWED_HOSTS", Some("a,b,"), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
        assert_eq!(resolved.allowed_hosts, vec!["a", "b"]);
    });
}

// ─── Duration Tests ───

#[test]
fn test_duration_from_env() {
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_var("CISTELL_TEST__REDIS__TIMEOUT", Some("120"), || {
        let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
        assert_eq!(resolved.timeout, Duration::from_secs(120));
    });
}

#[test]
fn test_duration_from_integer() {
    let mut map = MapSource::new("test");
    map.insert("timeout", ConfigValue::Integer(300));

    let resolver = Resolver::builder()
        .add_source(map)
        .defaults(make_redis_defaults())
        .build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    assert_eq!(resolved.timeout, Duration::from_secs(300));
}

#[test]
fn test_duration_default() {
    let resolver = Resolver::builder().defaults(make_redis_defaults()).build();

    let resolved: ResolvedConfig<RedisConfig> = resolver.resolve().unwrap();
    assert_eq!(resolved.timeout, Duration::from_secs(30));
}

// ─── Integration Tests ───

#[cfg(feature = "toml")]
#[test]
fn test_full_redis_config_resolution() {
    use cistell_core::FileSource;
    use std::io::Write;

    // Set up file source with some values
    let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        f,
        r#"[redis]
host = "file-host"
password = "file-pass"
allowed_hosts = ["h1", "h2"]
timeout = 60
"#
    )
    .unwrap();
    f.flush().unwrap();
    let file_source = FileSource::from_toml(f.path()).unwrap();

    // Map overrides host
    let mut map = MapSource::new("programmatic");
    map.insert("host", ConfigValue::String("map-host".into()));

    let resolver = Resolver::builder()
        .add_source(map)
        .env()
        .add_source(file_source)
        .defaults(make_redis_defaults())
        .build();

    temp_env::with_vars(
        [
            ("CISTELL_TEST__REDIS__PORT", Some("9999")),
            ("CISTELL_TEST__REDIS__SENTINEL_HOST", Some("sentinel.local")),
        ],
        || {
            let resolved = resolver.resolve::<RedisConfig>().unwrap();

            // map (rank 10) > file
            assert_eq!(resolved.host, "map-host");
            // env (rank 20) > default
            assert_eq!(resolved.port, 9999);
            // file (rank 40) > default
            assert_eq!(*resolved.password.expose(), "file-pass");
            // env (rank 20)
            assert_eq!(resolved.sentinel_host, Some("sentinel.local".to_owned()));
            // file (rank 40)
            assert_eq!(resolved.allowed_hosts, vec!["h1", "h2"]);
            // file (rank 40) wins with 60
            assert_eq!(resolved.timeout, Duration::from_secs(60));

            // Verify provenance
            assert!(matches!(
                resolved.provenance.get("host").unwrap().source,
                Source::Map { .. }
            ));
            assert!(matches!(
                resolved.provenance.get("port").unwrap().source,
                Source::EnvVar { .. }
            ));
            assert!(matches!(
                resolved.provenance.get("password").unwrap().source,
                Source::File { .. }
            ));
            assert!(matches!(
                resolved.provenance.get("sentinel_host").unwrap().source,
                Source::EnvVar { .. }
            ));
            assert!(matches!(
                resolved.provenance.get("allowed_hosts").unwrap().source,
                Source::File { .. }
            ));
        },
    );
}

#[test]
fn test_builder_api_ergonomics() {
    // Fluent builder pattern should read naturally
    let resolver = Resolver::builder()
        .env()
        .defaults(make_redis_defaults())
        .build();

    let resolved = resolver.resolve::<RedisConfig>().unwrap();
    assert_eq!(resolved.host, "localhost");
    assert_eq!(resolved.port, 6379);
}
