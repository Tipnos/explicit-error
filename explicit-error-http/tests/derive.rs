#[cfg(feature = "actix-web")]
mod actix;

// import only derive to validate that derives work without any required import
use explicit_error_http::derive::HttpError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ErrorBody {
    foo: String,
    bar: i64,
}

#[derive(Debug, HttpError)]
struct MyDomainError;

impl From<&MyDomainError> for explicit_error_http::HttpError {
    fn from(_: &MyDomainError) -> Self {
        explicit_error_http::HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo".to_string(),
                bar: 42,
            }),
            context: Some("context".to_string()),
        }
    }
}

#[test]
fn http_error() {
    let error = explicit_error_http::ToDomainError::to_domain_error(MyDomainError);

    assert_eq!(
        error.output,
        explicit_error_http::HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo".to_string(),
                bar: 42,
            }),
            context: Some("context".to_string()),
        }
    );
    assert!(
        error
            .source
            .as_ref()
            .unwrap()
            .downcast_ref::<MyDomainError>()
            .is_some()
    );

    assert_eq!(
        error.to_string(),
        r#"{"context":"context","http_status_code":400,"public":{"bar":42,"foo":"foo"},"source":"MyDomainError"}"#
    );
}
