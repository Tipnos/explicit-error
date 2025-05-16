extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

#[cfg(feature = "actix-web")]
mod actix;
mod error;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(HttpError, attributes(explicit_error))]
pub fn derive_explicit_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    error::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature = "actix-web")]
#[proc_macro_derive(DeriveHandlerError)]
pub fn derive_actix_handler_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    actix::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(JSONDisplay)]
pub fn json_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", serde_json::json!(self))
            }
        }
    };

    TokenStream::from(expanded)
}
