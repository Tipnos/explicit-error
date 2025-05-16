use super::HttpError;
use erased_serde::Serialize as DynSerialize;
use explicit_error_derive::JSONDisplay;
use serde::Serialize;

/// Self-sufficient container to both log an error and generate its http response.
///
/// [HttpError] implements `From<HttpErrorData>`, use `?` and `.into()` in functions and closures to convert to the [HttpError::Domain] variant.
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
/// #[derive(Error, Debug)]
/// enum MyDomainError {
///     Foo,
/// }
///
/// impl From<&MyDomainError> for HttpErrorData {
///     fn from(value: &MyDomainError) -> Self {
///         match value {
///             MyDomainError::Foo => HttpErrorData::new(
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
/// You can generate [HttpError::Domain] variant with an [HttpErrorData]
/// ```rust
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error::{HttpError, prelude::*};
///
/// fn business_logic() -> Result<(), HttpError> {
///     Err(HttpErrorData::new(
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
/// use explicit_error::{prelude::*, HttpErrorData, HttpError};
///
/// fn forbidden() -> HttpErrorData {
///     HttpErrorData::new(
///         StatusCode::FORBIDDEN,
///         ProblemDetails::new()
///             .with_type(Uri::from_static("/errors/generic#forbidden"))
///             .with_title("Forbidden."),
///     )
/// }
///
/// // context can be added by the caller to add information in log to help debugging
/// fn business_logic() -> Result<(), HttpError> {
///     Err(42).map_err(|e|
///         forbidden().with_context(
///             format!("Return a forbidden instead of 500 to avoid leaking implementation details: {e}")
///     ))?;
///     # Ok(())
/// }
/// ```
#[derive(Serialize, JSONDisplay)]
pub struct HttpErrorData {
    #[cfg(feature = "actix-web")]
    #[serde(serialize_with = "crate::actix::serialize_http_status_code")]
    pub http_status_code: actix_web::http::StatusCode,
    pub public: Box<dyn DynSerialize>,
    pub context: Option<String>,
}

impl HttpErrorData {
    /// Generate an [HttpErrorData] without a context. To add a context
    /// use [with_context](HttpErrorData::with_context) afterwards.
    /// # Examples
    /// ```rust
    /// # use explicit_error::{Result, HttpErrorData};
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// fn forbidden() -> HttpErrorData {
    ///     HttpErrorData::new(
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

    /// Add a context to an [HttpErrorData], override if one was set. The context appears in display
    /// but not in the http response.
    /// # Examples
    /// ```rust
    /// # use explicit_error::{Result, HttpErrorData};
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
    /// fn forbidden() -> HttpErrorData {
    ///     HttpErrorData::new(
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

impl From<HttpErrorData> for HttpError {
    fn from(value: HttpErrorData) -> Self {
        HttpError::Domain(Box::new(super::DomainError {
            data: value,
            source: None,
        }))
    }
}

impl std::fmt::Debug for HttpErrorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpErrorData{}", self)
    }
}
