use axum::{
    Router,
    body::Body,
    http::{self, Request, StatusCode},
    routing::get,
};
// import only derive to validate that derives work without any required import
use super::{ErrorBody, MyDomainError};
use explicit_error_http::derive::HandlerErrorHelpers;
use http_body_util::BodyExt;
use serde::Serialize;
use tower::util::ServiceExt;

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

fn app() -> Router {
    Router::new()
        .route("/domain", get(domain_error))
        .route("/domain2", get(domain_error2))
        .route("/fault", get(fault_error))
}

#[tokio::test]
async fn handler_derive() {
    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/domain")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let resp = serde_json::from_str::<ErrorBody>(
        std::str::from_utf8(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap(),
    )
    .unwrap();
    assert_eq!(resp.foo, "domain");
    assert_eq!(resp.bar, 200);

    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/domain2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let resp = serde_json::from_str::<ErrorBody>(
        std::str::from_utf8(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap(),
    )
    .unwrap();
    assert_eq!(resp.foo, "domain");
    assert_eq!(resp.bar, 200);

    let resp = app()
        .oneshot(
            Request::builder()
                .uri("/fault")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let resp = serde_json::from_str::<ErrorBody>(
        std::str::from_utf8(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap(),
    )
    .unwrap();
    assert_eq!(resp.foo, "fault");
    assert_eq!(resp.bar, 500);
}

async fn domain_error() -> Result<StatusCode, MyHandlerError> {
    Err(explicit_error_http::HttpError {
        http_status_code: http::StatusCode::FORBIDDEN,
        public: Box::new(""),
        context: None,
    })?;

    Ok(StatusCode::OK)
}

async fn domain_error2() -> Result<StatusCode, MyHandlerError> {
    Err(explicit_error_http::Error::from(MyDomainError))?;

    Ok(StatusCode::OK)
}

async fn fault_error() -> Result<StatusCode, MyHandlerError> {
    Err(explicit_error_http::Fault::new())?;

    Ok(StatusCode::OK)
}
