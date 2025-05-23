use super::*;
use actix_web::http::StatusCode;

#[derive(Serialize)]
struct ErrorBody {
    foo: &'static str,
    bar: i64,
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
    assert_eq!(
        format!(
            "{}",
            HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(ErrorBody {
                    foo: "foo",
                    bar: 42
                }),
                context: Some("context".to_string())
            }
        ),
        r#"{"context":"context","http_status_code":400,"public":{"bar":42,"foo":"foo"}}"#
            .to_string()
    );
}
