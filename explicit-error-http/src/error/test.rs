use super::*;
#[cfg(feature = "actix-web")]
use actix_web::http::StatusCode;

#[derive(Serialize)]
struct ErrorBody {
    foo: &'static str,
    bar: i64,
}

#[cfg(feature = "actix-web")]
#[test]
fn new() {
    let error = HttpError::new(
        StatusCode::BAD_REQUEST,
        ErrorBody {
            foo: "foo",
            bar: 42,
        },
    );
    assert!(error.context.is_none());
    assert_eq!(error.http_status_code, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::json!(error).to_string(),
        r#"{"bar":42,"foo":"foo"}"#
    );
}

#[test]
fn with_context() {
    let error = HttpError {
        #[cfg(feature = "actix-web")]
        http_status_code: StatusCode::BAD_REQUEST,
        public: Box::new(ErrorBody {
            foo: "foo",
            bar: 42,
        }),
        context: None,
    }
    .with_context("context");
    assert_eq!(error.context.as_deref().unwrap(), "context");
    assert_eq!(
        error.with_context("context 2").context.unwrap(),
        "context 2"
    )
}

#[test]
fn from_http_error_for_error() {
    let domain_error = crate::Error::from(HttpError {
        #[cfg(feature = "actix-web")]
        http_status_code: StatusCode::BAD_REQUEST,
        public: Box::new(ErrorBody {
            foo: "foo",
            bar: 42,
        }),
        context: None,
    })
    .unwrap();
    assert_eq!(
        HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: None,
        },
        domain_error.output
    );
    assert!(domain_error.source.is_none());
}

#[test]
fn serialize() {
    assert_eq!(
        serde_json::json!(HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42
            }),
            context: Some("context".to_string())
        })
        .to_string(),
        r#"{"bar":42,"foo":"foo"}"#.to_string()
    );
}

#[test]
fn display() {
    let error = HttpError {
        #[cfg(feature = "actix-web")]
        http_status_code: StatusCode::BAD_REQUEST,
        public: Box::new(ErrorBody {
            foo: "foo",
            bar: 42,
        }),
        context: Some("context".to_string()),
    }
    .to_string();

    #[cfg(feature = "actix-web")]
    assert_eq!(
        error,
        r#"{"context":"context","http_status_code":400,"public":{"bar":42,"foo":"foo"}}"#
            .to_string()
    );

    #[cfg(not(feature = "actix-web"))]
    assert_eq!(
        error,
        r#"{"context":"context","public":{"bar":42,"foo":"foo"}}"#.to_string()
    );
}
