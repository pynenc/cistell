use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments, Result, Type};

use crate::attrs::{parse_field_attrs, parse_struct_attrs};

pub(crate) fn expand_derive_config(input: DeriveInput) -> Result<TokenStream> {
    let struct_ident = input.ident.clone();
    let struct_attrs = parse_struct_attrs(&input.attrs, input.span())?;

    let data_struct = match input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "#[derive(Config)] only supports structs",
            ));
        }
    };

    let named_fields = match data_struct.fields {
        Fields::Named(fields) => fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                struct_ident,
                "#[derive(Config)] requires a struct with named fields",
            ));
        }
    };

    let mut direct_field_meta = Vec::new();
    let mut defaults_inits = Vec::new();
    let mut from_values_inits = Vec::new();
    let mut flatten_expanders = Vec::new();

    let prefix = struct_attrs.prefix.value();
    let group = struct_attrs.group.value();
    let sep = struct_attrs.sep.value();
    let toml_group = struct_attrs.toml_key.value();

    let prefix_upper = prefix.to_uppercase();
    let group_upper = group.to_uppercase();

    let sep_lit = LitStr::new(&sep, struct_attrs.sep.span());
    let prefix_upper_lit = LitStr::new(&prefix_upper, struct_attrs.prefix.span());
    let group_upper_lit = LitStr::new(&group_upper, struct_attrs.group.span());
    let toml_group_lit = LitStr::new(&toml_group, struct_attrs.toml_key.span());

    for field in named_fields {
        let field_ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(field.span(), "expected named field"))?;
        let field_name = field_ident.to_string();
        let field_name_lit = LitStr::new(&field_name, field_ident.span());
        let ty = field.ty.clone();

        let attrs = parse_field_attrs(&field.attrs)?;

        if attrs.skip {
            defaults_inits.push(quote! {
                #field_ident: ::std::default::Default::default()
            });
            from_values_inits.push(quote! {
                #field_ident: ::std::default::Default::default()
            });
            continue;
        }

        if attrs.flatten {
            let flatten_ident_lit = LitStr::new(&field_name, field_ident.span());
            let flatten_ty = ty.clone();

            flatten_expanders.push(quote! {
                for inner in <#flatten_ty as ::cistell_core::Config>::fields() {
                    let name = format!("{}.{}", #flatten_ident_lit, inner.name);
                    let config_key = format!("{}.{}", #toml_group_lit, inner.config_key);

                    let mut env_parts = vec![#prefix_upper_lit.to_owned(), #group_upper_lit.to_owned()];
                    let inner_segments: Vec<&str> = inner.env_key.split(#sep_lit).collect();
                    if inner_segments
                        .first()
                        .map(|s| s.eq_ignore_ascii_case(#prefix_upper_lit))
                        .unwrap_or(false)
                    {
                        env_parts.extend(inner_segments.iter().skip(1).map(|s| (*s).to_owned()));
                    } else {
                        env_parts.extend(inner_segments.iter().map(|s| (*s).to_owned()));
                    }
                    let env_key = env_parts.join(#sep_lit);

                    fields.push(::cistell_core::FieldMeta {
                        name: ::std::boxed::Box::leak(name.into_boxed_str()),
                        config_key: ::std::boxed::Box::leak(config_key.into_boxed_str()),
                        env_key: ::std::boxed::Box::leak(env_key.into_boxed_str()),
                        generic_env_key: inner.generic_env_key,
                        is_secret: inner.is_secret,
                        expected_type: inner.expected_type,
                        has_default: inner.has_default,
                        deserialize_fn: inner.deserialize_fn,
                    });
                }
            });

            defaults_inits.push(quote! {
                #field_ident: <#flatten_ty as ::cistell_core::Config>::defaults()
            });

            from_values_inits.push(quote! {
                #field_ident: {
                    let mut inner_values = ::std::collections::HashMap::new();
                    let prefix = concat!(#flatten_ident_lit, ".");
                    for (k, v) in values {
                        if let Some(stripped) = k.strip_prefix(prefix) {
                            inner_values.insert(stripped.to_owned(), v.clone());
                        }
                    }
                    <#flatten_ty as ::cistell_core::Config>::from_values(&inner_values)?
                }
            });

            continue;
        }

        let (is_secret_ty, _) = secret_inner_type(&ty);
        let is_secret = attrs.secret || is_secret_ty;
        let is_secret_lit = if is_secret {
            quote!(true)
        } else {
            quote!(false)
        };

        let has_default = attrs.default.is_some();
        let has_default_lit = if has_default {
            quote!(true)
        } else {
            quote!(false)
        };

        let field_toml_key = attrs
            .toml_key
            .as_ref()
            .map_or_else(|| field_name.clone(), LitStr::value);
        let config_key = format!("{}.{}", toml_group, field_toml_key);
        let config_key_lit = LitStr::new(&config_key, field_ident.span());

        let computed_env =
            format!("{}{}{}{}{}", prefix, sep, group, sep, field_name).to_uppercase();
        let env_key = attrs.env_key.as_ref().map_or(computed_env, LitStr::value);
        let env_key_lit = LitStr::new(&env_key, field_ident.span());

        // Generic env key: PREFIX__FIELDNAME (no group) — Python compat fallback.
        // Only generated when env_key is not manually overridden.
        let generic_env_key_expr = if attrs.env_key.is_none() {
            let generic_env = format!("{}{}{}", prefix, sep, field_name).to_uppercase();
            let generic_env_lit = LitStr::new(&generic_env, field_ident.span());
            quote!(Some(#generic_env_lit))
        } else {
            quote!(None)
        };

        let expected_type = normalize_type_name(&ty);
        let expected_type_lit = LitStr::new(&expected_type, field_ident.span());

        let deserialize_fn_expr = match attrs.deserialize_with {
            Some(path) => quote!(Some(#path)),
            None => quote!(None),
        };

        direct_field_meta.push(quote! {
            ::cistell_core::FieldMeta {
                name: #field_name_lit,
                config_key: #config_key_lit,
                env_key: #env_key_lit,
                generic_env_key: #generic_env_key_expr,
                is_secret: #is_secret_lit,
                expected_type: #expected_type_lit,
                has_default: #has_default_lit,
                deserialize_fn: #deserialize_fn_expr,
            }
        });

        let default_expr = build_default_expr(&field, &attrs.default)?;
        defaults_inits.push(quote! {
            #field_ident: #default_expr
        });

        let missing_branch = quote! {
            #default_expr
        };

        let coercion_expr = build_coercion_expr(&ty, quote!(__value), quote!(__meta));

        from_values_inits.push(quote! {
            #field_ident: match values.get(#field_name_lit) {
                Some(__raw) => {
                    let __meta = fields
                        .iter()
                        .find(|m| m.name == #field_name_lit)
                        .ok_or_else(|| ::cistell_core::ConfigError::Internal {
                            message: ::std::format!("derived field metadata missing for '{}'", #field_name_lit),
                        })?;
                    let __value = if let Some(__deserialize) = __meta.deserialize_fn {
                        __deserialize(__raw, __meta)?
                    } else {
                        __raw.clone()
                    };
                    #coercion_expr
                }
                None => #missing_branch,
            }
        });
    }

    let expanded = quote! {
        impl ::cistell_core::Config for #struct_ident {
            fn fields() -> &'static [::cistell_core::FieldMeta] {
                static FIELDS: ::std::sync::OnceLock<::std::vec::Vec<::cistell_core::FieldMeta>> =
                    ::std::sync::OnceLock::new();

                FIELDS
                    .get_or_init(|| {
                        let mut fields = ::std::vec![#(#direct_field_meta),*];
                        #(#flatten_expanders)*
                        fields
                    })
                    .as_slice()
            }

            fn defaults() -> Self {
                Self {
                    #(#defaults_inits),*
                }
            }

            fn from_values(
                values: &::std::collections::HashMap<String, ::cistell_core::ConfigValue>,
            ) -> ::std::result::Result<Self, ::cistell_core::ConfigError> {
                let fields = Self::fields();
                Ok(Self {
                    #(#from_values_inits),*
                })
            }
        }
    };

    Ok(expanded)
}

fn build_default_expr(field: &Field, default: &Option<syn::Expr>) -> Result<TokenStream> {
    let ty = &field.ty;

    if let Some(expr) = default {
        if let (true, Some(_inner)) = secret_inner_type(ty) {
            return Ok(quote! {
                ::cistell_core::Secret::new((#expr).into())
            });
        }

        if is_string_type(ty) {
            return Ok(quote! { (#expr).into() });
        }

        return Ok(quote! { #expr });
    }

    if option_inner_type(ty).is_some() {
        return Ok(quote! { None });
    }

    Err(syn::Error::new_spanned(
        field,
        "required field must have #[config(default = ...)] or be Option<T>",
    ))
}

fn build_coercion_expr(
    ty: &Type,
    value_ident: TokenStream,
    meta_ident: TokenStream,
) -> TokenStream {
    if let Some(inner) = option_inner_type(ty) {
        let inner_expr = build_coercion_expr(inner, value_ident, meta_ident);
        return quote! { Some(#inner_expr) };
    }

    if let (true, Some(inner)) = secret_inner_type(ty) {
        if let Some(vec_inner) = vec_inner_type(inner) {
            return quote! {
                ::cistell_core::Secret::new(#value_ident.coerce_vec::<#vec_inner>(#meta_ident)?)
            };
        }

        if is_duration_type(inner) {
            return quote! {
                ::cistell_core::Secret::new(::std::time::Duration::from_secs(
                    #value_ident.coerce::<u64>(#meta_ident)?
                ))
            };
        }

        return quote! {
            ::cistell_core::Secret::new(#value_ident.coerce::<#inner>(#meta_ident)?)
        };
    }

    if let Some(inner) = vec_inner_type(ty) {
        return quote! {
            #value_ident.coerce_vec::<#inner>(#meta_ident)?
        };
    }

    if is_duration_type(ty) {
        return quote! {
            ::std::time::Duration::from_secs(#value_ident.coerce::<u64>(#meta_ident)?)
        };
    }

    if is_string_type(ty) {
        return quote! {
            #value_ident.coerce_string(#meta_ident)?
        };
    }

    quote! {
        #value_ident.coerce::<#ty>(#meta_ident)?
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    type_inner(ty, "Option")
}

fn vec_inner_type(ty: &Type) -> Option<&Type> {
    type_inner(ty, "Vec")
}

fn secret_inner_type(ty: &Type) -> (bool, Option<&Type>) {
    (type_is(ty, "Secret"), type_inner(ty, "Secret"))
}

fn is_string_type(ty: &Type) -> bool {
    type_is(ty, "String")
}

fn is_duration_type(ty: &Type) -> bool {
    type_is(ty, "Duration")
}

fn type_inner<'a>(ty: &'a Type, expected: &str) -> Option<&'a Type> {
    let path = match ty {
        Type::Path(path) => &path.path,
        _ => return None,
    };

    let segment = path.segments.last()?;
    if segment.ident != expected {
        return None;
    }

    let args = match &segment.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    let first = args.args.first()?;
    match first {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

fn type_is(ty: &Type, expected: &str) -> bool {
    let path = match ty {
        Type::Path(path) => &path.path,
        _ => return false,
    };

    path.segments.last().is_some_and(|s| s.ident == expected)
}

fn normalize_type_name(ty: &Type) -> String {
    let tokens = quote!(#ty).to_string();
    tokens.chars().filter(|c| !c.is_whitespace()).collect()
}
