use crate::{DomainError, Error};
use explicit_error::Bug;
use serde::Serialize;

/// The type [Error] cannot directly be used as handlers or middlewares returned [Err] variant. A dedicated type is required.
/// The easiest implementation is to declare a [Newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html),
/// derive it with the [HandlerError](crate::derive::HandlerError) and implement the [HandlerError] trait.
///
/// ```rust
/// # use actix_web::{App, HttpResponse, HttpServer, get};
/// # use explicit_error_http::{Bug, Error, HandlerError, derive::HandlerError};
/// # use log::{debug, error};
/// # use problem_details::ProblemDetails;
/// # use serde::Serialize;
/// #[derive(HandlerError)]
/// struct MyHandlerError(Error);
///
/// impl HandlerError for MyHandlerError {
///     // Used by the derive for conversion
///     fn from_http_error(value: Error) -> Self {
///         MyHandlerError(value)
///     }
///
///     // Set-up monitoring and your custom HTTP response body for bugs
///     fn public_bug_response(bug: &Bug) -> impl Serialize {
///         #[cfg(debug_assertions)]
///         error!("{bug}");
///
///         #[cfg(not(debug_assertions))]
///         error!("{}", serde_json::json!(bug));
///
///         ProblemDetails::new()
///             .with_type(http::Uri::from_static("/errors/internal-server-error"))
///             .with_title("Internal server error")
///     }
///
///     fn http_error(&self) -> &Error {
///         &self.0
///     }
///
///     // Monitor domain variant of your errors
///     fn on_domain_response(error: &explicit_error_http::DomainError) {
///         if error.output.http_status_code.as_u16() < 500 {
///             debug!("{error}");
///         } else {
///             error!("{error}");
///         }
///     }
/// }
///
/// #[get("/my-handler")]
/// async fn my_handler() -> Result<HttpResponse, MyHandlerError> {
///     Ok(HttpResponse::Ok().finish())
/// }
/// ```
pub trait HandlerError {
    /// Accessor required by [HandlerError](crate::derive::HandlerError)
    fn http_error(&self) -> &Error;

    /// Set-up monitoring and your custom HTTP response body for bugs
    /// # Examples
    /// ```rust
    /// # use explicit_error_http::Bug;
    /// # use log::{debug, error};
    /// # use problem_details::ProblemDetails;
    /// # use serde::Serialize;
    /// fn public_bug_response(bug: &Bug) -> impl Serialize {
    ///     #[cfg(debug_assertions)]
    ///     error!("{bug}");
    ///
    ///     #[cfg(not(debug_assertions))]
    ///     error!("{}", serde_json::json!(bug));
    ///
    ///     ProblemDetails::new()
    ///         .with_type(http::Uri::from_static("/errors/internal-server-error"))
    ///         .with_title("Internal server error")
    /// }
    /// ```
    fn public_bug_response(bug: &Bug) -> impl Serialize;

    /// Monitor domain variant of your errors
    /// # Examples
    /// ```rust
    /// # use log::{debug, error};
    /// fn on_domain_response(error: &explicit_error_http::DomainError) {
    ///     if error.output.http_status_code.as_u16() < 500 {
    ///         debug!("{error}");
    ///     } else {
    ///         error!("{error}");
    ///     }
    /// }
    /// ```
    fn on_domain_response(error: &DomainError);

    /// Used by the derive for conversion
    fn from_http_error(value: Error) -> Self;
}
