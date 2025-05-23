extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

#[cfg(feature = "actix-web")]
mod actix;
#[cfg(any(feature = "exit", feature = "actix-web"))]
mod domain;

#[cfg(any(feature = "http", feature = "exit", feature = "actix-web"))]
use proc_macro::TokenStream;
#[cfg(any(feature = "http", feature = "exit", feature = "actix-web"))]
use syn::{DeriveInput, parse_macro_input};

#[cfg(feature = "http")]
#[proc_macro_derive(HttpError)]
pub fn derive_http_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    domain::derive(input, "explicit_error_http")
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature = "exit")]
#[proc_macro_derive(ExitError)]
pub fn derive_bin_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    domain::derive(input, "explicit_error_exit")
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature = "actix-web")]
#[proc_macro_derive(HandlerErrorHelpers)]
pub fn derive_actix_handler_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    actix::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
