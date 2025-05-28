use actix_web::{App, HttpResponse, body, get, http::StatusCode, test};
// import only derive to validate that derives work without any required import
use super::{ErrorBody, MyDomainError};
use explicit_error_http::derive::HandlerErrorHelpers;
use serde::Serialize;

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
#[actix_web::test]
async fn handler_derive() {
    let app = test::init_service(
        App::new()
            .service(domain_error)
            .service(domain_error2)
            .service(fault_error),
    )
    .await;

    let resp = test::call_service(&app, test::TestRequest::get().uri("/domain").to_request()).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let resp = serde_json::from_str::<ErrorBody>(
        std::str::from_utf8(&body::to_bytes(resp.into_body()).await.unwrap_or_default()).unwrap(),
    )
    .unwrap();
    assert_eq!(resp.foo, "domain");
    assert_eq!(resp.bar, 200);

    let resp =
        test::call_service(&app, test::TestRequest::get().uri("/domain2").to_request()).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let resp = serde_json::from_str::<ErrorBody>(
        std::str::from_utf8(&body::to_bytes(resp.into_body()).await.unwrap_or_default()).unwrap(),
    )
    .unwrap();
    assert_eq!(resp.foo, "domain");
    assert_eq!(resp.bar, 200);

    let resp = test::call_service(&app, test::TestRequest::get().uri("/fault").to_request()).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let resp = serde_json::from_str::<ErrorBody>(
        std::str::from_utf8(&body::to_bytes(resp.into_body()).await.unwrap_or_default()).unwrap(),
    )
    .unwrap();
    assert_eq!(resp.foo, "fault");
    assert_eq!(resp.bar, 500);
}

#[get("/domain")]
async fn domain_error() -> Result<HttpResponse, MyHandlerError> {
    Err(explicit_error_http::HttpError {
        http_status_code: StatusCode::FORBIDDEN,
        public: Box::new(""),
        context: None,
    })?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/domain2")]
async fn domain_error2() -> Result<HttpResponse, MyHandlerError> {
    Err(explicit_error_http::Error::from(MyDomainError))?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/fault")]
async fn fault_error() -> Result<HttpResponse, MyHandlerError> {
    Err(explicit_error_http::Fault::new())?;

    Ok(HttpResponse::Ok().finish())
}
