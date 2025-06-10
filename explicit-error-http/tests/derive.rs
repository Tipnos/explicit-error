#[cfg(feature = "axum")]
mod _axum;
#[cfg(feature = "actix-web")]
mod actix;

use explicit_error::Fault;
use explicit_error_derive::HandlerErrorHelpers;
// import only derive to validate that derives work without any required import
use explicit_error_http::{DomainError, Error, HttpError, derive::HttpError};
use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(HandlerErrorHelpers)]
struct MyHandlerError(explicit_error_http::Error);

impl explicit_error_http::HandlerError for MyHandlerError {
    fn from_error(value: explicit_error_http::Error) -> Self {
        MyHandlerError(value)
    }

    fn public_fault_response(_: &explicit_error_http::Fault) -> impl Serialize {
        ErrorBody {
            foo: "fault".to_string(),
            bar: 500,
        }
    }

    fn error(&self) -> &explicit_error_http::Error {
        &self.0
    }

    fn domain_response(_: &explicit_error_http::DomainError) -> impl Serialize {
        ErrorBody {
            foo: "domain".to_string(),
            bar: 200,
        }
    }
}

#[test]
fn converts() {
    MyHandlerError::from(Fault::new()).0.unwrap_fault();
    MyHandlerError::from(DomainError {
        output: HttpError::new(StatusCode::ACCEPTED, ""),
        source: None,
    })
    .0
    .unwrap();
    MyHandlerError::from(HttpError::new(StatusCode::ACCEPTED, ""))
        .0
        .unwrap();
    MyHandlerError::from(Error::Fault(Fault::new()))
        .0
        .unwrap_fault();
}

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
            http_status_code: StatusCode::BAD_REQUEST,
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
            http_status_code: StatusCode::BAD_REQUEST,
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
