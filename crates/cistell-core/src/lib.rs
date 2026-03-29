#[allow(unused_extern_crates)]
extern crate self as cistell_core;

pub mod config;
pub mod error;
pub mod field;
pub mod provenance;
pub mod resolver;
pub mod source;
pub mod value;

pub use cistell_macros::Config;
pub use config::{Config, ResolvedConfig};
pub use error::ConfigError;
pub use field::{DeserializeFn, FieldMeta, Secret};
pub use provenance::{FieldProvenance, ProvenanceMap, Source};
pub use resolver::{ResolvedRaw, Resolver, ResolverBuilder};
pub use source::{
    ConfigSource, DefaultSource, EnvSource, FileSource, MapSource, PyprojectTomlSource,
};
pub use value::duration_from_secs;
pub use value::ConfigValue;
