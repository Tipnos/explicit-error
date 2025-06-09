use crate::{HttpErrorDisplay, error::HttpError};
use explicit_error::{Domain, Error};
use serde::{Serialize, Serializer};
use std::{error::Error as StdError, fmt::Debug};

/// Wrapper for errors that are not a [Fault](explicit_error::Fault). It is used as the [explicit_error::Error::Domain] variant generic type.
///
/// It is highly recommended to implement the derive [HttpError](crate::derive::HttpError) which generates the boilerplate
/// for your domain errors. Otherwise you can implement the [ToDomainError] trait.
///
/// [Error](crate::Error) implements `From<DomainError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain] variant.
/// # Examples
/// [DomainError] can be generated because of a predicate
/// ```rust
/// # use http::StatusCode;
/// use explicit_error_http::{HttpError, Result, derive::HttpError};
///
/// #[derive(Debug, HttpError)]
/// enum MyError {
///     Domain,
/// }
///
/// impl From<&MyError> for HttpError {
///     fn from(value: &MyError) -> Self {
///         HttpError::new(
///             StatusCode::BAD_REQUEST,
///             "My domain error"
///         )
///     }
/// }
///
/// fn business_logic() -> Result<()> {
///     if 1 < 2 {
///         Err(MyError::Domain)?;
///     }
///     
///     if true {
///         Err(HttpError::new(StatusCode::FORBIDDEN, "")
///             .with_context("Usefull context to debug or monitor"))?;
///     }
/// #   Ok(())
/// }
/// ```
/// Or from a [Result]
/// ```rust
/// # use http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error_http::{Error, prelude::*, derive::HttpError, HttpError};
///
/// #[derive(HttpError, Debug)]
///  enum NotFoundError {
///     Bar(String)
///  }
///
///  impl From<&NotFoundError> for HttpError {
///     fn from(value: &NotFoundError) -> Self {
///         let (label, id) = match value {
///             NotFoundError::Bar(public_identifier) => ("Bar", public_identifier)
///         };
///
///         HttpError::new(
///             StatusCode::NOT_FOUND,
///             ProblemDetails::new()
///                 .with_type(Uri::from_static("/errors/not-found"))
///                 .with_title("Not found")
///                 .with_detail(format!("Unknown {label} with identifier {id}."))
///         )
///     }
/// }
///
/// fn business_logic(public_identifier: &str) -> Result<(), Error> {     
///     let entity = fetch_bar(&public_identifier).map_err_or_fault(|e|
///         match e {
///             sqlx::Error::RowNotFound => Ok(
///                 NotFoundError::Bar(public_identifier.to_string())),
///             _ => Err(e),
///         }
///     )?;
///
///     Ok(entity)
/// }
///
/// fn fetch_bar(public_identifier: &str) -> Result<(), sqlx::Error> {
///     Err(sqlx::Error::RowNotFound)
/// }
/// ```
/// Or an [Option]
/// ```rust
/// # use explicit_error_http::HttpError;
/// # use http::StatusCode;
/// Some(12).ok_or(HttpError::new(StatusCode::FORBIDDEN, ""))?;
/// # Ok::<(), HttpError>(())
/// ```
#[derive(Debug, Serialize)]
pub struct DomainError {
    #[serde(flatten)]
    pub output: HttpError,
    #[serde(skip)]
    pub source: Option<Box<dyn StdError + Send + Sync>>,
}

impl Domain for DomainError {
    fn into_source(self) -> Option<Box<dyn std::error::Error + Send + Sync>> {
        self.source
    }

    fn context(&self) -> Option<&str> {
        self.output.context.as_deref()
    }

    fn with_context(mut self, context: impl std::fmt::Display) -> Self {
        self.output = self.output.with_context(context);
        self
    }
}

impl From<DomainError> for Error<DomainError> {
    fn from(value: DomainError) -> Self {
        Self::Domain(Box::new(value))
    }
}

impl StdError for DomainError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_deref().map(|o| o as _)
    }
}

#[derive(Serialize)]
struct DomainErrorDisplay<'s> {
    #[serde(flatten)]
    output: HttpErrorDisplay<'s>,
    #[serde(serialize_with = "serialize_option_source_dyn")]
    pub source: Option<&'s (dyn StdError + 's + Send + Sync)>,
}

impl<'s> From<&'s DomainError> for DomainErrorDisplay<'s> {
    fn from(value: &'s DomainError) -> Self {
        Self {
            output: HttpErrorDisplay::<'s>::from(&value.output),
            source: value.source.as_deref(),
        }
    }
}

impl std::fmt::Display for DomainError {
    fn fmt<'s>(&'s self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::json!(DomainErrorDisplay::<'s>::from(self))
        )
    }
}

/// Internally used by [HttpError](crate::derive::HttpError) derive.
pub trait ToDomainError
where
    Self: Sized + StdError + 'static + Into<Error<DomainError>> + Send + Sync,
    for<'a> &'a Self: Into<HttpError>,
{
    fn to_domain_error(self) -> DomainError {
        DomainError {
            output: (&self).into(),
            source: Some(Box::new(self)),
        }
    }

    fn display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Serialize)]
        struct S<'s> {
            #[serde(flatten)]
            output: HttpError,
            #[serde(serialize_with = "serialize_source_dyn")]
            pub source: &'s dyn StdError,
        }

        write!(
            f,
            r#"{{"output":"{}","source":}}{}"#,
            Into::<HttpError>::into(self),
            serde_json::json!(S {
                output: self.into(),
                source: self,
            })
        )
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error_http::prelude::*`
pub trait ResultDomainWithContext<T, D>
where
    D: ToDomainError,
    for<'a> &'a D: Into<HttpError>,
{
    /// Add a context to an error that convert to [Error] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error::{prelude::*, Fault};
    /// Err::<(), _>(Fault::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl std::fmt::Display) -> std::result::Result<T, DomainError>;
}

impl<T, D> ResultDomainWithContext<T, D> for Result<T, D>
where
    D: ToDomainError,
    for<'a> &'a D: Into<HttpError>,
{
    fn with_context(self, context: impl std::fmt::Display) -> std::result::Result<T, DomainError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.to_domain_error().with_context(context)),
        }
    }
}

fn serialize_source_dyn<S>(source: &dyn StdError, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&explicit_error::errors_chain_debug(source))
}

fn serialize_option_source_dyn<S>(
    source: &Option<&(dyn StdError + Send + Sync)>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match source {
        Some(source) => serialize_source_dyn(source, s),
        None => s.serialize_none(),
    }
}

#[cfg(test)]
mod test;
