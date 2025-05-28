use super::*;
#[cfg(feature = "actix-web")]
use actix_web::http::StatusCode;

#[derive(Serialize)]
struct ErrorBody {
    foo: &'static str,
    bar: i64,
}

#[test]
fn into_source() {
    assert!(
        DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(""),
                context: None,
            },
            source: None,
        }
        .into_source()
        .is_none()
    );

    assert!(
        DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(""),
                context: None,
            },
            source: Some(Box::new(sqlx::Error::RowNotFound)),
        }
        .into_source()
        .unwrap()
        .downcast::<sqlx::Error>()
        .is_ok()
    );
}

#[test]
fn with_context() {
    let domain = DomainError {
        output: HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: StatusCode::BAD_REQUEST,
            public: Box::new(""),
            context: None,
        },
        source: None,
    }
    .with_context("context");

    assert_eq!(domain.output.context.as_ref().unwrap(), "context");
    assert_eq!(
        domain.with_context("context 2").output.context.unwrap(),
        "context 2"
    );
}

#[test]
fn context() {
    assert!(
        DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(""),
                context: None,
            },
            source: None,
        }
        .context()
        .is_none(),
    );

    assert_eq!(
        DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(""),
                context: Some("context".to_string()),
            },
            source: None,
        }
        .context()
        .unwrap(),
        "context"
    );
}

#[test]
fn source() {
    assert!(
        DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(""),
                context: None,
            },
            source: None,
        }
        .source()
        .is_none()
    );

    assert!(
        DomainError {
            output: HttpError {
                #[cfg(feature = "actix-web")]
                http_status_code: StatusCode::BAD_REQUEST,
                public: Box::new(""),
                context: None,
            },
            source: Some(Box::new(sqlx::Error::RowNotFound)),
        }
        .source()
        .unwrap()
        .downcast_ref::<sqlx::Error>()
        .is_some()
    );
}

#[test]
fn from_domain_for_error() {
    let domain = Error::from(DomainError {
        output: HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: None,
        },
        source: None,
    })
    .unwrap();

    assert_eq!(
        domain.output,
        HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: None,
        }
    );
    assert!(domain.source.is_none());
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
    let domain = DomainError {
        output: HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: Some("context".to_string()),
        },
        source: Some(Box::new(sqlx::Error::PoolClosed)),
    }
    .to_string();

    #[cfg(feature = "actix-web")]
    assert_eq!(
            domain,
        r#"{"context":"context","http_status_code":400,"public":{"bar":42,"foo":"foo"},"source":"PoolClosed"}"#
            .to_string()
    );

    #[cfg(not(feature = "actix-web"))]
    assert_eq!(
        domain,
        r#"{"context":"context","public":{"bar":42,"foo":"foo"},"source":"PoolClosed"}"#
            .to_string()
    );
}

#[derive(Debug)]
struct MyDomainError;

impl From<&MyDomainError> for HttpError {
    fn from(_: &MyDomainError) -> Self {
        HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: Some("context".to_string()),
        }
    }
}

impl StdError for MyDomainError {}

impl std::fmt::Display for MyDomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as ToDomainError>::display(self, f)
    }
}

impl From<MyDomainError> for crate::Error {
    fn from(value: MyDomainError) -> Self {
        Error::from(value.to_domain_error())
    }
}

impl ToDomainError for MyDomainError {}

#[test]
fn to_domain_error() {
    let domain_error = MyDomainError.to_domain_error();

    assert_eq!(
        domain_error.output,
        HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: Some("context".to_string()),
        }
    );
    assert!(
        domain_error
            .source
            .as_ref()
            .unwrap()
            .downcast_ref::<MyDomainError>()
            .is_some()
    );

    #[cfg(feature = "actix-web")]
    assert_eq!(
        domain_error.to_string(),
        r#"{"context":"context","http_status_code":400,"public":{"bar":42,"foo":"foo"},"source":"MyDomainError"}"#
    );

    #[cfg(not(feature = "actix-web"))]
    assert_eq!(
        domain_error.to_string(),
        r#"{"context":"context","public":{"bar":42,"foo":"foo"},"source":"MyDomainError"}"#
    );
}

#[test]
fn result_domain_with_context() {
    let domain_error = Err::<(), _>(MyDomainError)
        .with_context("context 2")
        .unwrap_err();

    assert_eq!(
        domain_error.output,
        HttpError {
            #[cfg(feature = "actix-web")]
            http_status_code: actix_web::http::StatusCode::BAD_REQUEST,
            public: Box::new(ErrorBody {
                foo: "foo",
                bar: 42,
            }),
            context: Some("context 2".to_string()),
        }
    );
    assert!(
        domain_error
            .source
            .as_ref()
            .unwrap()
            .downcast_ref::<MyDomainError>()
            .is_some()
    );

    #[cfg(feature = "actix-web")]
    assert_eq!(
        domain_error.to_string(),
        r#"{"context":"context 2","http_status_code":400,"public":{"bar":42,"foo":"foo"},"source":"MyDomainError"}"#
    );

    #[cfg(not(feature = "actix-web"))]
    assert_eq!(
        domain_error.to_string(),
        r#"{"context":"context 2","public":{"bar":42,"foo":"foo"},"source":"MyDomainError"}"#
    );
}
