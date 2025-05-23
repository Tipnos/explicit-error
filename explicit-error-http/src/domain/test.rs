use super::*;

#[derive(Serialize)]
struct ErrorBody {
    foo: &'static str,
    bar: i64,
}

#[test]
fn serialize() {
    assert_eq!(
        serde_json::json!(DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
                public: Box::new(ErrorBody {
                    foo: "foo",
                    bar: 42
                }),
                context: Some("context".to_string())
            },
            source: Some(Box::new(sqlx::Error::PoolClosed))
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
            DomainError {
                output: HttpError {
                    #[cfg(feature = "actix-web")]
                    http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
                    public: Box::new(ErrorBody {
                        foo: "foo",
                        bar: 42
                    }),
                    context: Some("context".to_string())
                },
                source: Some(Box::new(sqlx::Error::PoolClosed))
            }
        ),
        r#"{"context":"context","http_status_code":400,"public":{"bar":42,"foo":"foo"},"source":"PoolClosed"}"#
            .to_string()
    );
}
