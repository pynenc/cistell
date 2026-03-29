//! Procedural macros for cistell.

mod attrs;
mod derive_config;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Derive `cistell_core::Config` for a struct.
#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    match derive_config::expand_derive_config(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
