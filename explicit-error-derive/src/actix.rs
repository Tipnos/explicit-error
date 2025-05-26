use quote::quote;

pub fn derive(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics actix_web::ResponseError for #ident #ty_generics #where_clause {
            fn error_response(&self) -> actix_web::HttpResponse {
                match self.error() {
                    explicit_error_http::Error::Domain(d) => {
                        let status_code = d.output.http_status_code;
                        actix_web::HttpResponse::build(status_code).json(<Self as HandlerError>::domain_response(d))
                    },
                    explicit_error_http::Error::Fault(b) => actix_web::HttpResponse::InternalServerError().json(<Self as explicit_error_http::HandlerError>::public_fault_response(b)),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<explicit_error_http::Fault> for #ident #ty_generics #where_clause {
            fn from(value: explicit_error_http::Fault) -> Self {
                <Self as explicit_error_http::HandlerError>::from_error(value.into())
            }
        }

        #[automatically_derived]
        impl #impl_generics From<explicit_error_http::Error> for #ident #ty_generics #where_clause {
            fn from(value: explicit_error_http::Error) -> Self {
                <Self as explicit_error_http::HandlerError>::from_error(value)
            }
        }

        #[automatically_derived]
        impl #impl_generics From<explicit_error_http::HttpError> for #ident #ty_generics #where_clause {
            fn from(value: explicit_error_http::HttpError) -> Self {
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
