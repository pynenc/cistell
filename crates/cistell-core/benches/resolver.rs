use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cistell_core::field::FieldMeta;
use cistell_core::resolver::Resolver;
use cistell_core::source::{DefaultSource, EnvSource, MapSource};
use cistell_core::value::ConfigValue;

fn field_host() -> FieldMeta {
    FieldMeta {
        name: "host",
        config_key: "redis.host",
        env_key: "BENCH_CISTELL__HOST",
        generic_env_key: None,
        is_secret: false,
        expected_type: "String",
        has_default: true,
        deserialize_fn: None,
    }
}

fn field_port() -> FieldMeta {
    FieldMeta {
        name: "port",
        config_key: "redis.port",
        env_key: "BENCH_CISTELL__PORT",
        generic_env_key: None,
        is_secret: false,
        expected_type: "u16",
        has_default: true,
        deserialize_fn: None,
    }
}

fn field_secret() -> FieldMeta {
    FieldMeta {
        name: "password",
        config_key: "redis.password",
        env_key: "BENCH_CISTELL__PASSWORD",
        generic_env_key: None,
        is_secret: true,
        expected_type: "String",
        has_default: true,
        deserialize_fn: None,
    }
}

fn bench_resolve_raw_defaults(c: &mut Criterion) {
    let mut defaults = DefaultSource::new();
    defaults.insert("host", ConfigValue::String("localhost".into()));
    defaults.insert("port", ConfigValue::Integer(6379));
    defaults.insert("password", ConfigValue::String("secret".into()));

    let resolver = Resolver::builder().add_source(defaults).build();
    let fields = [field_host(), field_port(), field_secret()];

    c.bench_function("resolve_raw_defaults_only", |b| {
        b.iter(|| resolver.resolve_raw(black_box(&fields)).unwrap());
    });
}

fn bench_resolve_raw_with_env(c: &mut Criterion) {
    let mut defaults = DefaultSource::new();
    defaults.insert("host", ConfigValue::String("localhost".into()));
    defaults.insert("port", ConfigValue::Integer(6379));

    let env = EnvSource::new();
    let resolver = Resolver::builder()
        .add_source(env)
        .add_source(defaults)
        .build();

    let fields = [field_host(), field_port()];

    c.bench_function("resolve_raw_env_miss_fallback_defaults", |b| {
        b.iter(|| resolver.resolve_raw(black_box(&fields)).unwrap());
    });
}

fn bench_resolve_raw_multi_source(c: &mut Criterion) {
    let mut map = MapSource::new("overrides");
    map.insert("host", ConfigValue::String("from-map".into()));

    let env = EnvSource::new();

    let mut defaults = DefaultSource::new();
    defaults.insert("host", ConfigValue::String("localhost".into()));
    defaults.insert("port", ConfigValue::Integer(6379));
    defaults.insert("password", ConfigValue::String("secret".into()));

    let resolver = Resolver::builder()
        .add_source(map)
        .add_source(env)
        .add_source(defaults)
        .build();

    let fields = [field_host(), field_port(), field_secret()];

    c.bench_function("resolve_raw_multi_source", |b| {
        b.iter(|| resolver.resolve_raw(black_box(&fields)).unwrap());
    });
}

fn bench_config_value_coerce_string(c: &mut Criterion) {
    let meta = field_host();
    let val = ConfigValue::String("hello-world".into());

    c.bench_function("config_value_coerce_string", |b| {
        b.iter(|| val.coerce::<String>(black_box(&meta)).unwrap());
    });
}

fn bench_config_value_coerce_integer(c: &mut Criterion) {
    let meta = field_port();
    let val = ConfigValue::Integer(8080);

    c.bench_function("config_value_coerce_integer", |b| {
        b.iter(|| val.coerce::<u16>(black_box(&meta)).unwrap());
    });
}

fn bench_config_value_coerce_string_to_int(c: &mut Criterion) {
    let meta = field_port();
    let val = ConfigValue::String("8080".into());

    c.bench_function("config_value_coerce_string_to_int", |b| {
        b.iter(|| val.coerce::<u16>(black_box(&meta)).unwrap());
    });
}

fn bench_config_value_dotted_key_lookup(c: &mut Criterion) {
    use indexmap::IndexMap;

    let mut inner = IndexMap::new();
    inner.insert("host".into(), ConfigValue::String("localhost".into()));
    inner.insert("port".into(), ConfigValue::Integer(6379));

    let mut mid = IndexMap::new();
    mid.insert("redis".into(), ConfigValue::Table(inner));

    let mut root = IndexMap::new();
    root.insert("app".into(), ConfigValue::Table(mid));
    let table = ConfigValue::Table(root);

    c.bench_function("config_value_dotted_key_lookup", |b| {
        b.iter(|| table.get_by_dotted_key(black_box("app.redis.host")));
    });
}

fn bench_config_value_coerce_vec(c: &mut Criterion) {
    let meta = field_host();
    let val = ConfigValue::String("alpha, bravo, charlie, delta, echo".into());

    c.bench_function("config_value_coerce_vec_from_csv", |b| {
        b.iter(|| val.coerce_vec::<String>(black_box(&meta)).unwrap());
    });

    let arr = ConfigValue::Array(vec![
        ConfigValue::String("alpha".into()),
        ConfigValue::String("bravo".into()),
        ConfigValue::String("charlie".into()),
        ConfigValue::String("delta".into()),
        ConfigValue::String("echo".into()),
    ]);

    c.bench_function("config_value_coerce_vec_from_array", |b| {
        b.iter(|| arr.coerce_vec::<String>(black_box(&meta)).unwrap());
    });
}

criterion_group!(
    benches,
    bench_resolve_raw_defaults,
    bench_resolve_raw_with_env,
    bench_resolve_raw_multi_source,
    bench_config_value_coerce_string,
    bench_config_value_coerce_integer,
    bench_config_value_coerce_string_to_int,
    bench_config_value_dotted_key_lookup,
    bench_config_value_coerce_vec,
);
criterion_main!(benches);
