use crate::{DomainError, HttpError, HttpErrorData};
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serializer;

impl ResponseError for HttpError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            HttpError::Domain(handler_explicit_error) => handler_explicit_error.as_ref().into(),
            HttpError::Bug(_) => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl Into<HttpResponse> for &DomainError {
    fn into(self) -> HttpResponse {
        HttpResponse::build(self.data.http_status_code).json(&self.data.public)
    }
}

impl Into<HttpResponse> for &HttpErrorData {
    fn into(self) -> HttpResponse {
        HttpResponse::build(self.http_status_code).json(&self.public)
    }
}

pub(crate) fn serialize_http_status_code<S>(
    status_code: &StatusCode,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(status_code.as_u16())
}
