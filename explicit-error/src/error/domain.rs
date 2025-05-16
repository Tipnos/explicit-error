use super::HttpErrorData;
use crate::Error;
use explicit_error_derive::JSONDisplay;
use serde::{Serialize, Serializer};
use std::{error::Error as StdError, fmt::Debug};

/// Wrapper for errors of your domain that return informative feedback to the users.
///
/// It is highly recommended to implement the derive [Error](crate::Error) which generates the boilerplate
/// for your domain errors. Otherwise you can implement the [ToDomainError] trait.
///
/// [Error] implements `From<DomainError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain] variant.
/// # Examples
/// [DomainError] can be generated because of a predicate
/// ```rust
///
/// ```
/// Or from a [Result]
/// ```rust
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error::{Error, prelude::*};
///
/// #[derive(HttpError, Debug)]
/// # #[explicit_error(StdError)]
///  enum NotFoundError {
///     Bar(String)
///  }
///
///  impl From<&NotFoundError> for HttpErrorData {
///     fn from(value: &NotFoundError) -> Self {
///         let (label, id) = match value {
///             NotFoundError::Bar(public_identifier) => ("Bar", public_identifier)
///         };
///
///         HttpErrorData::new(
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
/// TODO
#[derive(Debug, Serialize, JSONDisplay)]
pub struct DomainError {
    #[serde(flatten)]
    pub(crate) output: HttpErrorData,
    #[serde(serialize_with = "serialize_source_box")]
    pub source: Option<Box<dyn StdError>>,
}

impl From<DomainError> for Error {
    fn from(value: DomainError) -> Self {
        Self::Domain(Box::new(value))
    }
}

impl DomainError {
    pub fn output(&self) -> &HttpErrorData {
        &self.output
    }

    pub fn split(self) -> (HttpErrorData, Option<Box<dyn StdError>>) {
        (self.output, self.source)
    }

    pub fn with_context<D: std::fmt::Display>(self, context: D) -> Self {
        Self {
            output: self.output.with_context(context),
            source: self.source,
        }
    }
}

impl StdError for DomainError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|o| o.as_ref())
    }
}

pub trait ToDomainError
where
    Self: Sized + StdError + 'static + Into<Error>,
    for<'a> &'a Self: Into<HttpErrorData>,
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
            output: HttpErrorData,
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
    s.serialize_str(&crate::errors_chain_debug(source))
}
