use crate::{DomainError, error::HttpError};
use actix_web::{HttpResponse, http::StatusCode};
use serde::Serializer;

impl From<&DomainError> for HttpResponse {
    fn from(value: &DomainError) -> Self {
        HttpResponse::build(value.output.http_status_code).json(&value.output.public)
    }
}

impl From<&HttpError> for HttpResponse {
    fn from(value: &HttpError) -> Self {
        HttpResponse::build(value.http_status_code).json(&value.public)
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
