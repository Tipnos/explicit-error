use quote::quote;

pub fn derive(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    //TODO: re-implement source attribute like ThisError

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as explicit_error::ToDomainError>::display(self, f)
            }
        }

        #[automatically_derived]
        impl #impl_generics explicit_error::ToDomainError for #ident #ty_generics #where_clause {
        }

        #[automatically_derived]
        impl #impl_generics From<#ident> for explicit_error::HttpError #ty_generics #where_clause {
            fn from(value: #ident) -> Self {
                explicit_error::HttpError::Domain(Box::new(<#ident as explicit_error::ToDomainError>::to_domain_error(value)))
            }
        }

        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {}
    })
}
