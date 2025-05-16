use crate::{Bug, DomainError, HttpError};
use serde::Serialize;

pub trait HandlerError {
    fn http_error(&self) -> &HttpError;

    fn public_bug_response(bug: &Bug) -> impl Serialize;

    fn public_domain_response(error: &DomainError);

    fn from_http_error(value: HttpError) -> Self;
}
