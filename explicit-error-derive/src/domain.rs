use quote::quote;

pub fn derive(
    input: syn::DeriveInput,
    crate_name: &'static str,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let crate_name: proc_macro2::TokenStream = syn::parse_str(crate_name)?;

    //TODO: re-implement source attribute like ThisError

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as #crate_name::ToDomainError>::display(self, f)
            }
        }

        #[automatically_derived]
        impl #impl_generics #crate_name::ToDomainError for #ident #ty_generics #where_clause {
        }

        #[automatically_derived]
        impl #impl_generics From<#ident> for #crate_name::Error #ty_generics #where_clause {
            fn from(value: #ident) -> Self {
                #crate_name::Error::Domain(Box::new(<#ident as #crate_name::ToDomainError>::to_domain_error(value)))
            }
        }

        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {}
    })
}
