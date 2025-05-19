use crate::error::HttpError;
use explicit_error::{Domain, Error};
use explicit_error_http_derive::JSONDisplay;
use serde::{Serialize, Serializer};
use std::{error::Error as StdError, fmt::Debug};

/// Wrapper for errors of your domain that return informative feedback to the users. It is used as the [explicit_error::Error::Domain] variant generic type.
///
/// It is highly recommended to implement the derive [HttpError](crate::derive::HttpError) which generates the boilerplate
/// for your domain errors. Otherwise you can implement the [ToDomainError] trait.
///
/// [Error](crate::Error) implements `From<DomainError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain] variant.
/// # Examples
/// [DomainError] can be generated because of a predicate
/// ```rust
/// # use actix_web::http::StatusCode;
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
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error_http::{Error, prelude::*, derive::HttpError};
///
/// #[derive(HttpError, Debug)]
/// # #[explicit_error_http(StdError)]
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
///     let entity = fetch_bar(&public_identifier).map_err_or_bug(|e|
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
/// # use actix_web::http::StatusCode;
/// Some(12).ok_or(HttpError::new(StatusCode::FORBIDDEN, ""))?;
/// # Ok::<(), HttpError>(())
/// ```
#[derive(Debug, Serialize, JSONDisplay)]
pub struct DomainError {
    #[serde(flatten)]
    pub output: HttpError,
    #[serde(serialize_with = "serialize_source_box")]
    pub source: Option<Box<dyn StdError>>,
}

impl Domain for DomainError {
    fn into_source(self) -> Option<Box<dyn std::error::Error>> {
        self.source
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
        self.source.as_ref().map(|o| o.as_ref())
    }
}

/// Internally used by [HttpError](crate::derive::HttpError) derive.
pub trait ToDomainError
where
    Self: Sized + StdError + 'static + Into<Error<DomainError>>,
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
            "{}",
            serde_json::json!(S {
                output: self.into(),
                source: self,
            })
        )
    }
}

fn serialize_source_box<S>(source: &Option<Box<dyn StdError>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(source) = source {
        serialize_source_dyn(source.as_ref(), s)
    } else {
        s.serialize_none()
    }
}

fn serialize_source_dyn<S>(source: &dyn StdError, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&explicit_error::errors_chain_debug(source))
}
