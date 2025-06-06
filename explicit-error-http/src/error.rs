use crate::Error;
use erased_serde::Serialize as DynSerialize;
use http::StatusCode;
use serde::{Serialize, Serializer};

/// Self-sufficient container to both log an error and generate its HTTP response. Regarding the web framework you use, its shape can be different.
///
/// [Error](crate::Error) implements `From<HttpError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain] variant.
///
/// Note: [HttpError] convert to [Error](crate::Error) by converting first to [DomainError](crate::DomainError).
/// # Examples
/// Domain errors that derive [HttpError](crate::derive::HttpError) must implement `From<&MyDomainError> for HttpError`.
/// ```rust
/// # use http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error_http::{derive::HttpError, prelude::*, HttpError};
///
/// #[derive(HttpError, Debug)]
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
/// Domain errors cannot require to be extracted in either a struct or enum variant (eg: middleware errors).
/// You can generate [Error::Domain] variant with an [HttpError]
/// ```rust
/// # use http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error_http::{Error, prelude::*, HttpError};
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
/// # use http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error_http::{prelude::*, HttpError, Error};
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
/// // context can be added by the caller to add information in log
/// fn business_logic() -> Result<(), Error> {
///     Err(42).map_err(|e|
///         forbidden().with_context(
///             format!("Return a forbidden instead of 500 to avoid leaking implementation details: {e}")
///     ))?;
///     # Ok(())
/// }
/// ```
#[derive(Serialize)]
pub struct HttpError {
    #[serde(skip)]
    pub http_status_code: StatusCode,
    #[serde(flatten)]
    pub public: Box<dyn DynSerialize + Send + Sync>,
    #[serde(skip)]
    pub context: Option<String>,
}

impl HttpError {
    /// Generate an [HttpError] without a context. To add a context
    /// use [with_context](HttpError::with_context) afterwards.
    /// # Examples
    /// ```rust
    /// # use explicit_error_http::{Result, HttpError};
    /// # use http::StatusCode;
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
    pub fn new<S: Serialize + 'static + Send + Sync>(
        http_status_code: StatusCode,
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
    /// # use explicit_error_http::{Result, HttpError};
    /// # use http::StatusCode;
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
    pub fn with_context(mut self, context: impl std::fmt::Display) -> Self {
        self.context = Some(context.to_string());
        self
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

impl PartialEq for HttpError {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context
            && self.http_status_code == other.http_status_code
            && serde_json::json!(self.public) == serde_json::json!(other)
    }
}

#[derive(Serialize)]
pub(crate) struct HttpErrorDisplay<'s> {
    #[serde(serialize_with = "serialize_http_status_code")]
    pub http_status_code: http::StatusCode,
    pub public: &'s dyn DynSerialize,
    pub context: Option<&'s str>,
}

impl<'s> From<&'s HttpError> for HttpErrorDisplay<'s> {
    fn from(value: &'s HttpError) -> Self {
        Self {
            http_status_code: value.http_status_code,
            public: value.public.as_ref(),
            context: value.context.as_deref(),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt<'s>(&'s self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::json!(HttpErrorDisplay::<'s>::from(self))
        )
    }
}

impl std::fmt::Debug for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpError{}", self)
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

#[cfg(test)]
mod test;
