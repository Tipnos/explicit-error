use crate::{DomainError, error::HttpError};
use actix_web::{HttpResponse, http::StatusCode};
use serde::Serializer;

impl Into<HttpResponse> for &DomainError {
    fn into(self) -> HttpResponse {
        HttpResponse::build(self.output.http_status_code).json(&self.output.public)
    }
}

impl Into<HttpResponse> for &HttpError {
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
