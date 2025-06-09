use proc_macro2::TokenStream;
use quote::quote;

pub fn derive(input: syn::DeriveInput) -> TokenStream {
    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let actix = if cfg!(feature = "actix-web") {
        quote! {
            #[automatically_derived]
            impl #impl_generics actix_web::ResponseError for #ident #ty_generics #where_clause {
                fn error_response(&self) -> actix_web::HttpResponse {
                    match <Self as explicit_error_http::HandlerError>::error(self) {
                        explicit_error_http::Error::Domain(d) => actix_web::HttpResponse::build(
                            actix_web::http::StatusCode::from_u16(d.output.http_status_code.as_u16()).unwrap())
                            .json(<Self as explicit_error_http::HandlerError>::domain_response(d)),
                        explicit_error_http::Error::Fault(b) => actix_web::HttpResponse::InternalServerError()
                            .json(<Self as explicit_error_http::HandlerError>::public_fault_response(b)),
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let axum = if cfg!(feature = "axum") {
        quote! {
            #[automatically_derived]
            impl #impl_generics axum::response::IntoResponse for #ident #ty_generics #where_clause {
                fn into_response(self) -> axum::response::Response {
                    match <Self as explicit_error_http::HandlerError>::error(&self) {
                        explicit_error_http::Error::Domain(d) => axum::response::IntoResponse::into_response((
                            axum::http::StatusCode::from_u16(d.output.http_status_code.as_u16()).unwrap(),
                            axum::Json(<Self as explicit_error_http::HandlerError>::domain_response(d)),
                        )),
                        explicit_error_http::Error::Fault(b) => axum::response::IntoResponse::into_response((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(<Self as explicit_error_http::HandlerError>::public_fault_response(b)),
                        )),
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    //TODO: re-implement source attribute like ThisError

    quote! {
        #axum

        #actix

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
        impl #impl_generics From<explicit_error_http::DomainError> for #ident #ty_generics #where_clause {
            fn from(value: explicit_error_http::DomainError) -> Self {
                <Self as explicit_error_http::HandlerError>::from_error(value.into())
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Display for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(<Self as explicit_error_http::HandlerError>::error(self), f)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(<Self as explicit_error_http::HandlerError>::error(self), f)
            }
        }

        #[automatically_derived]
        impl #impl_generics std::error::Error for #ident #ty_generics #where_clause {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(<Self as explicit_error_http::HandlerError>::error(self))
            }
        }
    }
}
