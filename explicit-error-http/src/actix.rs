use actix_web::http::StatusCode;
use serde::Serializer;

pub(crate) fn serialize_http_status_code<S>(
    status_code: &StatusCode,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(status_code.as_u16())
}
