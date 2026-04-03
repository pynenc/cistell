use std::collections::HashMap;
use std::time::Duration;

use cistell_core::{
    Config, ConfigError, ConfigValue, FieldMeta, MapSource, Resolver, Secret, Source,
};

#[derive(Config, Debug, PartialEq)]
#[config(prefix = "CISTELL_DERIVE", group = "redis")]
struct BasicRedisConfig {
    #[config(default = "localhost")]
    host: String,

    #[config(default = 6379u16)]
    port: u16,

    #[config(secret, default = "")]
    password: Secret<String>,

    sentinel_host: Option<String>,

    #[config(default = vec!["localhost".to_owned()])]
    allowed_hosts: Vec<String>,
}

#[derive(Config, Debug, PartialEq)]
#[config(
    prefix = "CISTELL_DERIVE",
    group = "custom",
    sep = "_",
    toml_key = "service.custom"
)]
struct CustomKeysConfig {
    #[config(default = "localhost", env_key = "CUSTOM_HOST", toml_key = "hostname")]
    host: String,
}

#[derive(Config, Debug, PartialEq)]
#[config(prefix = "CISTELL_DERIVE", group = "tls")]
struct TlsConfig {
    #[config(default = false)]
    enabled: bool,

    #[config(default = "cert.pem")]
    cert_path: String,
}

#[derive(Config, Debug, PartialEq)]
#[config(prefix = "CISTELL_DERIVE", group = "redis")]
struct RedisWithFlatten {
    #[config(default = "localhost")]
    host: String,

    #[config(flatten)]
    tls: TlsConfig,
}

#[derive(Config, Debug, PartialEq)]
#[config(prefix = "CISTELL_DERIVE", group = "skip")]
struct SkipConfig {
    #[config(default = "localhost")]
    host: String,

    #[config(skip)]
    cached_count: u32,
}

#[derive(Config, Debug, PartialEq)]
#[config(prefix = "CISTELL_DERIVE", group = "timeouts")]
struct DurationConfig {
    #[config(default = ::std::time::Duration::from_secs(30), deserialize_with = "cistell_core::duration_from_secs")]
    connect_timeout: Duration,
}

#[derive(Debug, PartialEq)]
struct ManualRedisConfig {
    host: String,
    port: u16,
    sentinel_host: Option<String>,
    allowed_hosts: Vec<String>,
}

impl Config for ManualRedisConfig {
    fn fields() -> &'static [FieldMeta] {
        static FIELDS: [FieldMeta; 4] = [
            FieldMeta {
                name: "host",
                config_key: "redis.host",
                env_key: "CISTELL_DERIVE__REDIS__HOST",
                generic_env_key: None,
                is_secret: false,
                expected_type: "String",
                has_default: true,
                deserialize_fn: None,
            },
            FieldMeta {
                name: "port",
                config_key: "redis.port",
                env_key: "CISTELL_DERIVE__REDIS__PORT",
                generic_env_key: None,
                is_secret: false,
                expected_type: "u16",
                has_default: true,
                deserialize_fn: None,
            },
            FieldMeta {
                name: "sentinel_host",
                config_key: "redis.sentinel_host",
                env_key: "CISTELL_DERIVE__REDIS__SENTINEL_HOST",
                generic_env_key: None,
                is_secret: false,
                expected_type: "Option<String>",
                has_default: false,
                deserialize_fn: None,
            },
            FieldMeta {
                name: "allowed_hosts",
                config_key: "redis.allowed_hosts",
                env_key: "CISTELL_DERIVE__REDIS__ALLOWED_HOSTS",
                generic_env_key: None,
                is_secret: false,
                expected_type: "Vec<String>",
                has_default: true,
                deserialize_fn: None,
            },
        ];
        &FIELDS
    }

    fn defaults() -> Self {
        Self {
            host: "localhost".to_owned(),
            port: 6379,
            sentinel_host: None,
            allowed_hosts: vec!["localhost".to_owned()],
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

        let sentinel_host = match values.get("sentinel_host") {
            Some(v) => Some(v.coerce_string(&fields[2])?),
            None => None,
        };

        let allowed_hosts = match values.get("allowed_hosts") {
            Some(v) => v.coerce_vec::<String>(&fields[3])?,
            None => defaults.allowed_hosts,
        };

        Ok(Self {
            host,
            port,
            sentinel_host,
            allowed_hosts,
        })
    }
}

#[derive(Config, Debug, PartialEq)]
#[config(prefix = "CISTELL_DERIVE", group = "redis")]
struct DerivedRedisConfig {
    #[config(default = "localhost")]
    host: String,

    #[config(default = 6379u16)]
    port: u16,

    sentinel_host: Option<String>,

    #[config(default = vec!["localhost".to_owned()])]
    allowed_hosts: Vec<String>,
}

#[test]
fn test_derive_basic_struct() {
    let fields = BasicRedisConfig::fields();
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[0].name, "host");
    assert_eq!(fields[1].name, "port");
    assert_eq!(fields[2].name, "password");
    assert_eq!(fields[3].name, "sentinel_host");
    assert_eq!(fields[4].name, "allowed_hosts");
}

#[test]
fn test_derive_defaults() {
    let defaults = BasicRedisConfig::defaults();
    assert_eq!(defaults.host, "localhost");
    assert_eq!(defaults.port, 6379);
    assert_eq!(*defaults.password.expose(), "");
    assert_eq!(defaults.sentinel_host, None);
    assert_eq!(defaults.allowed_hosts, vec!["localhost"]);
}

#[test]
fn test_derive_from_values() {
    let mut values = HashMap::new();
    values.insert(
        "host".to_owned(),
        ConfigValue::String("redis.internal".into()),
    );
    values.insert("port".to_owned(), ConfigValue::Integer(6380));
    values.insert("password".to_owned(), ConfigValue::String("hunter2".into()));
    values.insert(
        "allowed_hosts".to_owned(),
        ConfigValue::Array(vec![
            ConfigValue::String("a".into()),
            ConfigValue::String("b".into()),
        ]),
    );

    let cfg = BasicRedisConfig::from_values(&values).unwrap();
    assert_eq!(cfg.host, "redis.internal");
    assert_eq!(cfg.port, 6380);
    assert_eq!(*cfg.password.expose(), "hunter2");
    assert_eq!(cfg.sentinel_host, None);
    assert_eq!(cfg.allowed_hosts, vec!["a", "b"]);
}

#[test]
fn test_derive_env_key_generation() {
    let fields = BasicRedisConfig::fields();
    let host = fields.iter().find(|f| f.name == "host").unwrap();
    assert_eq!(host.env_key, "CISTELL_DERIVE__REDIS__HOST");
}

#[test]
fn test_derive_config_key_generation() {
    let fields = BasicRedisConfig::fields();
    let host = fields.iter().find(|f| f.name == "host").unwrap();
    assert_eq!(host.config_key, "redis.host");
}

#[test]
fn test_derive_secret_field() {
    let fields = BasicRedisConfig::fields();
    let password = fields.iter().find(|f| f.name == "password").unwrap();
    assert!(password.is_secret);
}

#[test]
fn test_derive_option_field_no_default() {
    let fields = BasicRedisConfig::fields();
    let sentinel = fields.iter().find(|f| f.name == "sentinel_host").unwrap();
    assert!(!sentinel.has_default);

    let defaults = BasicRedisConfig::defaults();
    assert!(defaults.sentinel_host.is_none());
}

#[test]
fn test_derive_skip_field() {
    let fields = SkipConfig::fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "host");

    let defaults = SkipConfig::defaults();
    assert_eq!(defaults.host, "localhost");
    assert_eq!(defaults.cached_count, 0);
}

#[test]
fn test_derive_custom_env_key() {
    let fields = CustomKeysConfig::fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].env_key, "CUSTOM_HOST");
}

#[test]
fn test_derive_custom_toml_key() {
    let fields = CustomKeysConfig::fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].config_key, "service.custom.hostname");
}

#[test]
fn test_derive_flatten_includes_inner_fields() {
    let fields = RedisWithFlatten::fields();
    assert_eq!(fields.len(), 3);
    assert!(fields.iter().any(|f| f.name == "host"));
    assert!(fields.iter().any(|f| f.name == "tls.enabled"));
    assert!(fields.iter().any(|f| f.name == "tls.cert_path"));
}

#[test]
fn test_derive_flatten_env_key_nesting() {
    let fields = RedisWithFlatten::fields();
    let tls_enabled = fields.iter().find(|f| f.name == "tls.enabled").unwrap();
    assert_eq!(tls_enabled.env_key, "CISTELL_DERIVE__REDIS__TLS__ENABLED");
}

#[test]
fn test_derive_flatten_resolution() {
    let resolver = Resolver::builder().env().build();

    temp_env::with_var("CISTELL_DERIVE__REDIS__TLS__ENABLED", Some("true"), || {
        let resolved = resolver.resolve::<RedisWithFlatten>().unwrap();
        assert!(resolved.tls.enabled);
        assert_eq!(resolved.tls.cert_path, "cert.pem");
    });
}

#[test]
fn test_duration_deserialize_with() {
    let resolver = Resolver::builder().env().build();

    temp_env::with_var(
        "CISTELL_DERIVE__TIMEOUTS__CONNECT_TIMEOUT",
        Some("120"),
        || {
            let resolved = resolver.resolve::<DurationConfig>().unwrap();
            assert_eq!(resolved.connect_timeout, Duration::from_secs(120));
        },
    );
}

#[cfg(feature = "toml")]
#[test]
fn test_derive_and_resolve_with_file() {
    use std::io::Write;

    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        file,
        "[redis]\nhost = \"from-file\"\nport = 7000\npassword = \"file-pass\""
    )
    .unwrap();
    file.flush().unwrap();

    let resolver = Resolver::builder().file(file.path()).unwrap().build();
    let resolved = resolver.resolve::<BasicRedisConfig>().unwrap();

    assert_eq!(resolved.host, "from-file");
    assert_eq!(resolved.port, 7000);
    assert_eq!(*resolved.password.expose(), "file-pass");

    let host_prov = resolved.provenance.get("host").unwrap();
    assert!(matches!(host_prov.source, Source::File { .. }));
}

#[cfg(feature = "toml")]
#[test]
fn test_derive_and_resolve_mixed_sources() {
    use std::io::Write;

    let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(
        file,
        "[redis]\nhost = \"from-file\"\nport = 7000\npassword = \"file-pass\"\nallowed_hosts = [\"f1\", \"f2\"]"
    )
    .unwrap();
    file.flush().unwrap();

    let mut map = MapSource::new("test-overrides");
    map.insert("host", ConfigValue::String("from-map".into()));

    let resolver = Resolver::builder()
        .add_source(map)
        .env()
        .file(file.path())
        .unwrap()
        .build();

    temp_env::with_var("CISTELL_DERIVE__REDIS__PORT", Some("9001"), || {
        let resolved = resolver.resolve::<BasicRedisConfig>().unwrap();
        assert_eq!(resolved.host, "from-map");
        assert_eq!(resolved.port, 9001);
        assert_eq!(*resolved.password.expose(), "file-pass");
        assert_eq!(resolved.allowed_hosts, vec!["f1", "f2"]);

        let host_prov = resolved.provenance.get("host").unwrap();
        assert!(matches!(host_prov.source, Source::Map { .. }));

        let port_prov = resolved.provenance.get("port").unwrap();
        assert!(matches!(port_prov.source, Source::EnvVar { .. }));

        let pass_prov = resolved.provenance.get("password").unwrap();
        assert!(matches!(pass_prov.source, Source::File { .. }));
    });
}

#[test]
fn test_derive_and_resolve() {
    let resolver = Resolver::builder().env().build();

    temp_env::with_vars(
        [
            ("CISTELL_DERIVE__REDIS__HOST", None::<&str>),
            ("CISTELL_DERIVE__REDIS__PORT", None::<&str>),
            ("CISTELL_DERIVE__REDIS__PASSWORD", None::<&str>),
            ("CISTELL_DERIVE__REDIS__SENTINEL_HOST", None::<&str>),
            ("CISTELL_DERIVE__REDIS__ALLOWED_HOSTS", None::<&str>),
        ],
        || {
            let resolved = resolver.resolve::<BasicRedisConfig>().unwrap();

            assert_eq!(resolved.host, "localhost");
            assert_eq!(resolved.port, 6379);
            assert_eq!(*resolved.password.expose(), "");
            assert_eq!(resolved.allowed_hosts, vec!["localhost"]);

            let option_prov = resolved.provenance.get("sentinel_host").unwrap();
            assert_eq!(option_prov.source, Source::Default);
            assert_eq!(option_prov.display_value.as_deref(), Some("None"));
        },
    );
}

#[test]
fn test_derive_and_resolve_with_env() {
    let resolver = Resolver::builder().env().build();

    temp_env::with_vars(
        [
            ("CISTELL_DERIVE__REDIS__HOST", Some("from-env")),
            ("CISTELL_DERIVE__REDIS__PORT", Some("8000")),
            ("CISTELL_DERIVE__REDIS__ALLOWED_HOSTS", Some("a,b,c")),
        ],
        || {
            let resolved = resolver.resolve::<BasicRedisConfig>().unwrap();
            assert_eq!(resolved.host, "from-env");
            assert_eq!(resolved.port, 8000);
            assert_eq!(resolved.allowed_hosts, vec!["a", "b", "c"]);

            let host_prov = resolved.provenance.get("host").unwrap();
            assert!(matches!(host_prov.source, Source::EnvVar { .. }));
        },
    );
}

#[test]
fn test_derived_redis_matches_manual_impl() {
    let mut map = MapSource::new("manual-vs-derived");
    map.insert("port", ConfigValue::Integer(7777));

    let resolver = Resolver::builder().add_source(map).env().build();

    temp_env::with_vars(
        [
            ("CISTELL_DERIVE__REDIS__HOST", Some("env-host")),
            ("CISTELL_DERIVE__REDIS__ALLOWED_HOSTS", Some("x,y")),
            ("CISTELL_DERIVE__REDIS__SENTINEL_HOST", Some("sentinel")),
        ],
        || {
            let manual = resolver.resolve::<ManualRedisConfig>().unwrap();
            let derived = resolver.resolve::<DerivedRedisConfig>().unwrap();

            assert_eq!(manual.host, derived.host);
            assert_eq!(manual.port, derived.port);
            assert_eq!(manual.sentinel_host, derived.sentinel_host);
            assert_eq!(manual.allowed_hosts, derived.allowed_hosts);
        },
    );
}
