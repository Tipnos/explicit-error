use quote::quote;

pub fn derive(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut from_impl_generics = input.generics.clone();
    from_impl_generics.params.push(syn::parse_str("EE")?);
    let explicit_error_http_where = "EE: Into<explicit_error_http::Error>";
    let from_where_clause = where_clause.map_or(
        syn::parse_str(&format!("where {explicit_error_http_where}"))?,
        |w| {
            let mut c = w.clone();

            c.predicates
                .push(syn::parse_str(explicit_error_http_where).unwrap());

            c
        },
    );

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics actix_web::ResponseError for #ident #ty_generics #where_clause {
            fn error_response(&self) -> actix_web::HttpResponse {
                match self.error() {
                    explicit_error_http::Error::Domain(d) => {
                        let status_code = d.output.http_status_code;
                        actix_web::HttpResponse::build(status_code).json(<Self as HandlerError>::domain_response(d))
                    },
                    explicit_error_http::Error::Bug(b) => actix_web::HttpResponse::InternalServerError().json(<Self as explicit_error_http::HandlerError>::public_bug_response(b)),
                }
            }
        }

        #[automatically_derived]
        impl #from_impl_generics From<EE> for #ident #ty_generics #from_where_clause
        {
            fn from(value: EE) -> Self {
                <Self as explicit_error_http::HandlerError>::from_error(value.into())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(self.error(), f)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(self.error(), f)
            }
        }
    })
}
