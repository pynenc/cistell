use proc_macro2::Span;
use syn::{Attribute, Expr, LitStr, Path, Result};

#[derive(Debug, Clone)]
pub(crate) struct StructAttrs {
    pub prefix: LitStr,
    pub group: LitStr,
    pub sep: LitStr,
    pub toml_key: LitStr,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FieldAttrs {
    pub default: Option<Expr>,
    pub env_key: Option<LitStr>,
    pub toml_key: Option<LitStr>,
    pub secret: bool,
    pub skip: bool,
    pub flatten: bool,
    pub deserialize_with: Option<Path>,
}

pub(crate) fn parse_struct_attrs(attrs: &[Attribute], span: Span) -> Result<StructAttrs> {
    let mut prefix: Option<LitStr> = None;
    let mut group: Option<LitStr> = None;
    let mut sep: Option<LitStr> = None;
    let mut toml_key: Option<LitStr> = None;

    for attr in attrs {
        if !attr.path().is_ident("config") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                let value: LitStr = meta.value()?.parse()?;
                prefix = Some(value);
                return Ok(());
            }
            if meta.path.is_ident("group") {
                let value: LitStr = meta.value()?.parse()?;
                group = Some(value);
                return Ok(());
            }
            if meta.path.is_ident("sep") {
                let value: LitStr = meta.value()?.parse()?;
                sep = Some(value);
                return Ok(());
            }
            if meta.path.is_ident("toml_key") {
                let value: LitStr = meta.value()?.parse()?;
                toml_key = Some(value);
                return Ok(());
            }

            Err(meta.error(
                "unknown struct config attribute; expected one of: prefix, group, sep, toml_key",
            ))
        })?;
    }

    let prefix = prefix.ok_or_else(|| {
        syn::Error::new(
            span,
            "#[config(prefix = \"...\")] is required on the struct",
        )
    })?;
    let group = group.ok_or_else(|| {
        syn::Error::new(span, "#[config(group = \"...\")] is required on the struct")
    })?;

    let sep = sep.unwrap_or_else(|| LitStr::new("__", span));
    let toml_key = toml_key.unwrap_or_else(|| LitStr::new(&group.value(), group.span()));

    Ok(StructAttrs {
        prefix,
        group,
        sep,
        toml_key,
    })
}

pub(crate) fn parse_field_attrs(attrs: &[Attribute]) -> Result<FieldAttrs> {
    let mut out = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("config") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                let expr: Expr = meta.value()?.parse()?;
                out.default = Some(expr);
                return Ok(());
            }
            if meta.path.is_ident("env_key") {
                let value: LitStr = meta.value()?.parse()?;
                out.env_key = Some(value);
                return Ok(());
            }
            if meta.path.is_ident("toml_key") {
                let value: LitStr = meta.value()?.parse()?;
                out.toml_key = Some(value);
                return Ok(());
            }
            if meta.path.is_ident("secret") {
                out.secret = true;
                return Ok(());
            }
            if meta.path.is_ident("skip") {
                out.skip = true;
                return Ok(());
            }
            if meta.path.is_ident("flatten") {
                out.flatten = true;
                return Ok(());
            }
            if meta.path.is_ident("deserialize_with") {
                let value: LitStr = meta.value()?.parse()?;
                let parsed = syn::parse_str::<Path>(&value.value()).map_err(|e| {
                    syn::Error::new(value.span(), format!("invalid path for deserialize_with: {e}"))
                })?;
                out.deserialize_with = Some(parsed);
                return Ok(());
            }

            Err(meta.error(
                "unknown field config attribute; expected one of: default, env_key, toml_key, secret, skip, flatten, deserialize_with",
            ))
        })?;
    }

    if out.skip && out.default.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[config(skip)] and #[config(default = ...)] are mutually exclusive",
        ));
    }

    if out.flatten && (out.secret || out.default.is_some() || out.env_key.is_some()) {
        return Err(syn::Error::new(
            Span::call_site(),
            "#[config(flatten)] cannot be combined with secret/default/env_key",
        ));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    use syn::spanned::Spanned;

    #[test]
    fn test_parse_struct_attrs() {
        let input: syn::DeriveInput = parse_quote! {
            #[config(prefix = "X", group = "redis")]
            struct Demo { a: String }
        };

        let parsed = parse_struct_attrs(&input.attrs, input.span()).unwrap();
        assert_eq!(parsed.prefix.value(), "X");
        assert_eq!(parsed.group.value(), "redis");
        assert_eq!(parsed.sep.value(), "__");
        assert_eq!(parsed.toml_key.value(), "redis");
    }

    #[test]
    fn test_parse_struct_attrs_with_sep() {
        let input: syn::DeriveInput = parse_quote! {
            #[config(prefix = "X", group = "redis", sep = "_")]
            struct Demo { a: String }
        };

        let parsed = parse_struct_attrs(&input.attrs, input.span()).unwrap();
        assert_eq!(parsed.sep.value(), "_");
    }

    #[test]
    fn test_parse_field_default_string() {
        let input: syn::Field = parse_quote! {
            #[config(default = "localhost")]
            host: String
        };

        let parsed = parse_field_attrs(&input.attrs).unwrap();
        assert!(parsed.default.is_some());
    }

    #[test]
    fn test_parse_field_default_numeric() {
        let input: syn::Field = parse_quote! {
            #[config(default = 6379u16)]
            port: u16
        };

        let parsed = parse_field_attrs(&input.attrs).unwrap();
        assert!(parsed.default.is_some());
    }

    #[test]
    fn test_parse_field_secret() {
        let input: syn::Field = parse_quote! {
            #[config(secret)]
            password: String
        };

        let parsed = parse_field_attrs(&input.attrs).unwrap();
        assert!(parsed.secret);
    }

    #[test]
    fn test_parse_field_skip() {
        let input: syn::Field = parse_quote! {
            #[config(skip)]
            cache: String
        };

        let parsed = parse_field_attrs(&input.attrs).unwrap();
        assert!(parsed.skip);
    }

    #[test]
    fn test_parse_field_flatten() {
        let input: syn::Field = parse_quote! {
            #[config(flatten)]
            tls: TlsConfig
        };

        let parsed = parse_field_attrs(&input.attrs).unwrap();
        assert!(parsed.flatten);
    }
}
