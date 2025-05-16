use super::Error;
use erased_serde::Serialize as DynSerialize;
use explicit_error_derive::JSONDisplay;
use serde::Serialize;

/// Self-sufficient container to both log an error and generate its http response.
///
/// [Error] implements `From<HttpError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain] variant.
///
/// Regarding the web framework you use, its shape can be different.
/// # Examples
/// Its principal usage is for domain errors that derive [Error](crate::Error) or implement [ToDomainError](crate::ToDomainError).
///```rust
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error::prelude::*;
///
/// #[derive(HttpErrorDerive, Debug)]
/// enum MyDomainError {
///     Foo,
/// }
///
/// impl From<&MyDomainError> for HttpError {
///     fn from(value: &MyDomainError) -> Self {
///         match value {
///             MyDomainError::Foo => HttpError::new(
///                     StatusCode::BAD_REQUEST,
///                     ProblemDetails::new()
///                         .with_type(Uri::from_static("/errors/my-domain/foo"))
///                         .with_title("Foo format incorrect.")
///                 ),
///         }
///     }
/// }
/// ```
///
/// Domain errors sometimes don't require to be extracted in either a struct or enum variant (eg: middleware errors).
/// You can generate [Error::Domain] variant with an [HttpError]
/// ```rust
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error::{Error, prelude::*};
///
/// fn business_logic() -> Result<(), Error> {
///     Err(HttpError::new(
///         StatusCode::FORBIDDEN,
///         ProblemDetails::new()
///             .with_type(Uri::from_static("/errors/generic#forbidden"))
///             .with_title("Forbidden."),
///     ))?;
///     # Ok(())
/// }
/// ```
///
/// Usually to avoid boilerplate and having consistency in error responses web applications
/// implement helpers for frequent http error codes.
/// ```rust
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error::{prelude::*, HttpError, Error};
///
/// fn forbidden() -> HttpError {
///     HttpError::new(
///         StatusCode::FORBIDDEN,
///         ProblemDetails::new()
///             .with_type(Uri::from_static("/errors/generic#forbidden"))
///             .with_title("Forbidden."),
///     )
/// }
///
/// // context can be added by the caller to add information in log to help debugging
/// fn business_logic() -> Result<(), Error> {
///     Err(42).map_err(|e|
///         forbidden().with_context(
///             format!("Return a forbidden instead of 500 to avoid leaking implementation details: {e}")
///     ))?;
///     # Ok(())
/// }
/// ```
#[derive(Serialize, JSONDisplay)]
pub struct HttpError {
    #[cfg(feature = "actix-web")]
    #[serde(serialize_with = "crate::actix::serialize_http_status_code")]
    pub http_status_code: actix_web::http::StatusCode,
    pub public: Box<dyn DynSerialize>,
    pub context: Option<String>,
}

impl HttpError {
    /// Generate an [HttpError] without a context. To add a context
    /// use [with_context](HttpError::with_context) afterwards.
    /// # Examples
    /// ```rust
    /// # use explicit_error::{Result, HttpError};
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// fn forbidden() -> HttpError {
    ///     HttpError::new(
    ///         StatusCode::UNAUTHORIZED,
    ///         ProblemDetails::new()
    ///             .with_type(Uri::from_static("/errors/forbidden"))
    ///             .with_title("Forbidden"),
    ///     )
    /// }
    /// ```
    #[cfg(feature = "actix-web")]
    pub fn new<S: Serialize + 'static>(
        http_status_code: actix_web::http::StatusCode,
        public: S,
    ) -> Self {
        Self {
            http_status_code,
            public: Box::new(public),
            context: None,
        }
    }

    /// Add a context to an [HttpError], override if one was set. The context appears in display
    /// but not in the http response.
    /// # Examples
    /// ```rust
    /// # use explicit_error::{Result, HttpError};
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// fn check_authz() -> Result<()> {
    ///     if !false {
    ///         Err(forbidden().with_context("Some info to help debug"))?;
    ///     }
    ///     Ok(())
    /// }
    ///
    /// fn forbidden() -> HttpError {
    ///     HttpError::new(
    ///         StatusCode::UNAUTHORIZED,
    ///         ProblemDetails::new()
    ///             .with_type(Uri::from_static("/errors/forbidden"))
    ///             .with_title("Forbidden"),
    ///     )
    /// }
    /// ```
    pub fn with_context<D: std::fmt::Display>(self, context: D) -> Self {
        Self {
            #[cfg(feature = "actix-web")]
            http_status_code: self.http_status_code,
            public: self.public,
            context: Some(context.to_string()),
        }
    }
}

impl From<HttpError> for Error {
    fn from(value: HttpError) -> Self {
        Error::Domain(Box::new(super::DomainError {
            output: value,
            source: None,
        }))
    }
}

impl std::fmt::Debug for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpError{}", self)
    }
}
