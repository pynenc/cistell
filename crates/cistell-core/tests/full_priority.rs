use cistell_core::config::Config;
use cistell_core::resolver::Resolver;
use cistell_macros::Config;

#[derive(Debug, Config)]
#[config(prefix = "RUSTVELLO", group = "redis")]
#[allow(dead_code)]
struct RedisConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = 6379)]
    port: u16,
    #[config(default = 1)]
    db: u8,
}

#[cfg(feature = "toml")]
#[test]
fn test_env_file_class_level() {
    use std::io::Write;

    let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(f, "host = \"class-level-file\"").unwrap();
    f.flush().unwrap();

    temp_env::with_var("RUSTVELLO__REDIS__FILEPATH", Some(f.path()), || {
        let resolver = Resolver::builder()
            .env_file_class("RUSTVELLO", "__", "redis")
            .unwrap()
            .build();
        let cfg = resolver.resolve::<RedisConfig>().unwrap();
        assert_eq!(cfg.host, "class-level-file");
    });
}

#[cfg(feature = "toml")]
#[test]
fn test_env_file_class_level_missing_file() {
    use cistell_core::error::ConfigError;

    temp_env::with_var(
        "RUSTVELLO__REDIS__FILEPATH",
        Some("/no/such/file.toml"),
        || {
            let res = Resolver::builder().env_file_class("RUSTVELLO", "__", "redis");
            let err = match res {
                Ok(_) => panic!("expected err"),
                Err(e) => e,
            };

            assert!(matches!(err, ConfigError::FileError { .. }));
        },
    );
}

#[test]
fn test_env_file_class_level_unset() {
    temp_env::with_vars(vec![("RUSTVELLO__REDIS__FILEPATH", None::<&str>)], || {
        let resolver = Resolver::builder()
            .env_file_class("RUSTVELLO", "__", "redis")
            .unwrap()
            .build();
        let cfg = resolver.resolve::<RedisConfig>().unwrap();
        assert_eq!(cfg.host, "localhost");
    });
}

#[cfg(feature = "toml")]
#[test]
fn test_env_file_global() {
    use std::io::Write;

    let mut f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(f, "[redis]\nhost = \"global-file\"").unwrap();
    f.flush().unwrap();

    temp_env::with_var("RUSTVELLO__FILEPATH", Some(f.path()), || {
        let resolver = Resolver::builder()
            .env_file_global("RUSTVELLO", "__")
            .unwrap()
            .build();
        let cfg = resolver.resolve::<RedisConfig>().unwrap();
        assert_eq!(cfg.host, "global-file");
    });
}

#[cfg(feature = "toml")]
#[test]
fn test_env_file_global_rank() {
    use std::io::Write;

    let mut class_f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(class_f, "host = \"class-file\"").unwrap();
    class_f.flush().unwrap();

    let mut global_f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(global_f, "[redis]\nhost = \"global-file\"").unwrap();
    global_f.flush().unwrap();

    temp_env::with_vars(
        [
            (
                "RUSTVELLO__REDIS__FILEPATH",
                Some(class_f.path().to_str().unwrap()),
            ),
            (
                "RUSTVELLO__FILEPATH",
                Some(global_f.path().to_str().unwrap()),
            ),
        ],
        || {
            let resolver = Resolver::builder()
                .env_file_global("RUSTVELLO", "__")
                .unwrap()
                .env_file_class("RUSTVELLO", "__", "redis")
                .unwrap()
                .build();
            let cfg = resolver.resolve::<RedisConfig>().unwrap();
            // class_file should win (30 vs 35)
            assert_eq!(cfg.host, "class-file");
        },
    );
}

#[cfg(feature = "toml")]
#[test]
fn test_full_priority_chain_7_levels() {
    use cistell_core::source::MapSource;
    use cistell_core::value::ConfigValue;
    use std::env::set_current_dir;
    use tempfile::tempdir;

    // We will test the priority chain
    // 1. MapSource (rank 5)
    // 2. EnvVar direct (rank 20)
    // 3. EnvVar->class file (rank 30)
    // 4. EnvVar->global file (rank 35)
    // 5. Explicit file (rank 40)
    // 6. pyproject.toml (rank 50)
    // 7. Default (rank 100) -> structure default

    // Let's create these
    // Map = host: map-host
    // Env = port: 9999
    // Class= db: 5
    // Global= host: global-host (loses to Map)
    // Explicit= db: 99 (loses to Class)
    // Pyproject= host: pyproj-host (loses to Map)

    let dir = tempdir().unwrap();

    let class_f = dir.path().join("class.toml");
    std::fs::write(&class_f, "db = 5").unwrap();

    let global_f = dir.path().join("global.toml");
    std::fs::write(&global_f, "[redis]\nhost = \"global-host\"").unwrap();

    let explicit_f = dir.path().join("explicit.toml");
    std::fs::write(&explicit_f, "[redis]\ndb = 99").unwrap();

    let pyproj_f = dir.path().join("pyproject.toml");
    std::fs::write(
        &pyproj_f,
        "[tool.rustvello.redis]\nhost = \"pyproj-host\"\nport = 8888",
    )
    .unwrap();

    let old_cwd = std::env::current_dir().unwrap();
    set_current_dir(dir.path()).unwrap();

    temp_env::with_vars(
        [
            ("RUSTVELLO__REDIS__PORT", Some("9999")),
            (
                "RUSTVELLO__REDIS__FILEPATH",
                Some(class_f.to_str().unwrap()),
            ),
            ("RUSTVELLO__FILEPATH", Some(global_f.to_str().unwrap())),
        ],
        || {
            let mut map = MapSource::new("test");
            map.insert("host", ConfigValue::String("map-host".into()));

            let resolver = Resolver::builder()
                .add_source(map.with_rank(5))
                .env()
                .env_file_class("RUSTVELLO", "__", "redis")
                .unwrap()
                .env_file_global("RUSTVELLO", "__")
                .unwrap()
                .file(&explicit_f)
                .unwrap()
                .pyproject_toml("rustvello", "redis")
                .unwrap()
                .build();

            let cfg = resolver.resolve::<RedisConfig>().unwrap();

            assert_eq!(cfg.host, "map-host");
            assert_eq!(cfg.port, 9999);
            assert_eq!(cfg.db, 5);
        },
    );

    set_current_dir(old_cwd).unwrap();
}

#[cfg(all(feature = "yaml", feature = "toml"))]
#[test]
fn test_mixed_sources_per_field() {
    use std::io::Write;

    let mut yaml_f = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
    writeln!(yaml_f, "redis:\n  host: yaml-host\n  port: 2222").unwrap();
    yaml_f.flush().unwrap();

    let mut toml_f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(toml_f, "[redis]\ndb = 42\nhost = \"toml-host\"").unwrap();
    toml_f.flush().unwrap();

    let resolver = Resolver::builder()
        .add_source(
            cistell_core::source::FileSource::load(yaml_f.path())
                .unwrap()
                .with_rank(30),
        )
        .add_source(
            cistell_core::source::FileSource::load(toml_f.path())
                .unwrap()
                .with_rank(40),
        )
        .build();

    let cfg = resolver.resolve::<RedisConfig>().unwrap();

    // host and port from yaml (rank 30)
    assert_eq!(cfg.host, "yaml-host");
    assert_eq!(cfg.port, 2222);
    // db from toml (rank 40), because it's missing in yaml
    assert_eq!(cfg.db, 42);
}

#[cfg(all(feature = "yaml", feature = "toml"))]
#[test]
fn test_yaml_and_toml_together() {
    // Just simple resolve to assure it compiles and links correctly.
    test_mixed_sources_per_field()
}

// ============================================================================
// Python pynenc compatibility tests
// ============================================================================

/// Python behavior: `PREFIX__FIELDNAME` (generic env var, no group) serves as
/// fallback when `PREFIX__GROUP__FIELDNAME` (class-specific) is not set.
///
/// Equivalent Python test:
/// ```python
/// class SomeConfig(ConfigBase):
///     field = ConfigField(0)
///
/// with patch.dict(os.environ, {"CONFIG__FIELD": "7"}):
///     conf = SomeConfig()
/// assert conf.field == 7
/// ```
#[derive(Debug, Config)]
#[config(prefix = "PYCOMPAT", group = "some")]
#[allow(dead_code)]
struct SomeConfig {
    #[config(default = 0)]
    field: i32,
}

#[test]
fn test_generic_env_var_fallback() {
    // Generic env var (no group) should work as fallback
    temp_env::with_vars(
        [
            ("PYCOMPAT__SOME__FIELD", None::<&str>),
            ("PYCOMPAT__FIELD", Some("7")),
        ],
        || {
            let resolver = Resolver::builder().env().build();
            let cfg = resolver.resolve::<SomeConfig>().unwrap();
            assert_eq!(cfg.field, 7);
        },
    );
}

#[test]
fn test_class_specific_env_overrides_generic() {
    // Class-specific env var should override generic
    // Python equivalent: CONFIG__SOMECONFIG__FIELD overrides CONFIG__FIELD
    temp_env::with_vars(
        [
            ("PYCOMPAT__SOME__FIELD", Some("8")),
            ("PYCOMPAT__FIELD", Some("7")),
        ],
        || {
            let resolver = Resolver::builder().env().build();
            let cfg = resolver.resolve::<SomeConfig>().unwrap();
            assert_eq!(cfg.field, 8);
        },
    );
}

#[test]
fn test_env_var_highest_priority() {
    // Env vars should override all file sources (Python priority #1)
    use cistell_core::source::MapSource;
    use cistell_core::value::ConfigValue;

    let mut map = MapSource::new("config_values");
    map.insert("field", ConfigValue::Integer(1));

    temp_env::with_vars([("PYCOMPAT__SOME__FIELD", Some("8"))], || {
        let resolver = Resolver::builder()
            .env()
            .add_source(map.with_rank(40)) // simulating file source priority
            .build();
        let cfg = resolver.resolve::<SomeConfig>().unwrap();
        assert_eq!(cfg.field, 8);
    });
}

/// Python behavior: files with flat keys (no group section) should work
/// as a fallback when the group-specific key is not present.
///
/// Equivalent Python:
/// ```python
/// class ConfigChild(ConfigParent):
///     test_field = ConfigField("child_value")
///
/// # File: {"test_field": "file_default_value"}
/// config = ConfigChild(config_filepath=filepath)
/// assert config.test_field == "file_default_value"
/// ```
#[cfg(feature = "json")]
#[test]
fn test_file_flat_key_fallback() {
    use std::io::Write;

    // File with flat key only (no group section)
    let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    writeln!(f, r#"{{"field": 42}}"#).unwrap();
    f.flush().unwrap();

    let source = cistell_core::source::FileSource::load(f.path()).unwrap();
    let resolver = Resolver::builder().add_source(source).build();
    let cfg = resolver.resolve::<SomeConfig>().unwrap();
    assert_eq!(cfg.field, 42);
}

/// Python behavior: class-specific keys in files override flat keys.
///
/// Equivalent Python test:
/// ```python
/// conf = SomeConfig(config_values={"field": 1, "some": {"field": 2}})
/// assert conf.field == 2
/// ```
#[cfg(feature = "json")]
#[test]
fn test_file_class_specific_overrides_flat() {
    use std::io::Write;

    // File with both flat and group-specific keys
    let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    writeln!(f, r#"{{"field": 1, "some": {{"field": 2}}}}"#).unwrap();
    f.flush().unwrap();

    let source = cistell_core::source::FileSource::load(f.path()).unwrap();
    let resolver = Resolver::builder().add_source(source).build();
    let cfg = resolver.resolve::<SomeConfig>().unwrap();
    // Group-specific "some.field" should win over flat "field"
    assert_eq!(cfg.field, 2);
}

/// Python behavior: env_file_class uses double underscore as separator.
///
/// Equivalent Python:
/// ```python
/// # ENV: CONFIG__SOMECONFIG__FILEPATH=/path/to/file.json
/// conf = SomeConfig()
/// ```
#[cfg(feature = "json")]
#[test]
fn test_env_file_class_uses_double_underscore_separator() {
    use std::io::Write;

    let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    writeln!(f, r#"{{"field": 99}}"#).unwrap();
    f.flush().unwrap();

    // The env var should be PREFIX__GROUP__FILEPATH (double underscore)
    temp_env::with_var("PYCOMPAT__SOME__FILEPATH", Some(f.path()), || {
        let resolver = Resolver::builder()
            .env_file_class("PYCOMPAT", "__", "some")
            .unwrap()
            .build();
        let cfg = resolver.resolve::<SomeConfig>().unwrap();
        assert_eq!(cfg.field, 99);
    });
}

/// Python behavior: custom separator is respected in filepath env vars.
///
/// Equivalent Python:
/// ```python
/// class LibraryConfigBase(ConfigBase):
///     ENV_SEP = "<->"
/// # ENV: LIBCFG<->LIBRARYCONFIGMAIN<->CFGFILE=/path
/// ```
#[cfg(feature = "json")]
#[test]
fn test_custom_separator_env_file() {
    use std::io::Write;

    #[derive(Debug, Config)]
    #[config(prefix = "LIBCFG", group = "main", sep = "<->")]
    #[allow(dead_code)]
    struct CustomSepConfig {
        #[config(default = "default_val")]
        name: String,
    }

    let mut f = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    writeln!(f, r#"{{"name": "from_file"}}"#).unwrap();
    f.flush().unwrap();

    // Custom sep: LIBCFG<->MAIN<->FILEPATH
    temp_env::with_var("LIBCFG<->MAIN<->FILEPATH", Some(f.path()), || {
        let resolver = Resolver::builder()
            .env_file_class("LIBCFG", "<->", "main")
            .unwrap()
            .build();
        let cfg = resolver.resolve::<CustomSepConfig>().unwrap();
        assert_eq!(cfg.name, "from_file");
    });
}

/// Verify the derive macro generates correct generic_env_key.
#[test]
fn test_derive_generates_generic_env_key() {
    let fields = SomeConfig::fields();
    let field = &fields[0];
    assert_eq!(field.name, "field");
    assert_eq!(field.env_key, "PYCOMPAT__SOME__FIELD");
    assert_eq!(field.generic_env_key, Some("PYCOMPAT__FIELD"));
}

/// Custom env_key disables generic fallback.
#[test]
fn test_custom_env_key_no_generic_fallback() {
    #[derive(Debug, Config)]
    #[config(prefix = "TEST", group = "grp")]
    #[allow(dead_code)]
    struct CustomEnvKeyConfig {
        #[config(default = "default", env_key = "MY_CUSTOM_KEY")]
        value: String,
    }

    let fields = CustomEnvKeyConfig::fields();
    assert_eq!(fields[0].env_key, "MY_CUSTOM_KEY");
    assert_eq!(fields[0].generic_env_key, None);
}

/// Python behavior: full priority chain matches Python's ordering.
///
/// Python priority (highest to lowest):
/// 1. Class-specific env var (PREFIX__CLASSNAME__FIELD)
/// 2. Generic env var (PREFIX__FIELD)
/// 3. Class-specific env file (PREFIX__CLASSNAME__FILEPATH)
/// 4. Global env file (PREFIX__FILEPATH)
/// 5. Config file parameter
/// 6. pyproject.toml
/// 7. config_values dict
/// 8. Default
#[cfg(feature = "toml")]
#[test]
fn test_python_compat_full_priority() {
    use std::io::Write;

    // Setup: generic env var sets field=42
    // Class-specific file sets field=99
    // Generic env should win (higher priority than files)
    let mut class_f = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
    writeln!(class_f, "field = 99").unwrap();
    class_f.flush().unwrap();

    temp_env::with_vars(
        [
            ("PYCOMPAT__FIELD", Some("42")),
            (
                "PYCOMPAT__SOME__FILEPATH",
                Some(class_f.path().to_str().unwrap()),
            ),
        ],
        || {
            let resolver = Resolver::builder()
                .env()
                .env_file_class("PYCOMPAT", "__", "some")
                .unwrap()
                .build();
            let cfg = resolver.resolve::<SomeConfig>().unwrap();
            // Generic env var (rank 20) should win over class file (rank 30)
            assert_eq!(cfg.field, 42);
        },
    );
}
