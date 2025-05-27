use crate::{domain::Domain, error::Error};
use serde::{Serialize, Serializer};
use std::{backtrace::Backtrace, error::Error as StdError};

/// Wrapper for errors that should not happen but cannot panic.
/// It is wrapped in the [Error::Fault] variant.
///
/// To generate it from predicates use [Fault::new], from [Result] or [Option]
/// import the prelude and use either [or_fault()](crate::error::ResultFault::or_fault),
/// [or_fault_no_source()](crate::error::ResultFault::or_fault_no_source),
/// [or_fault_force()](crate::error::ResultFault::or_fault_force),
/// [or_fault_no_source_force()](crate::error::ResultFault::or_fault_no_source_force)
#[derive(Debug, Serialize)]
pub struct Fault {
    #[serde(serialize_with = "serialize_source")]
    pub source: Option<Box<dyn StdError>>,
    #[serde(serialize_with = "serialize_backtrace")]
    backtrace: Backtrace,
    context: Option<String>,
}

impl<D> From<Fault> for Error<D>
where
    D: Domain,
{
    fn from(value: Fault) -> Self {
        Error::Fault(value)
    }
}

impl StdError for Fault {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            match self.backtrace.status() {
                std::backtrace::BacktraceStatus::Captured =>
                    format!("{}\n ----------------------- \n\n", self.backtrace),
                _ => String::new(),
            },
            match &self.context {
                Some(c) => format!("Context: {}\n", c),
                None => String::new(),
            },
            match &self.source {
                Some(s) => format!(
                    "Source: {}, {}\n",
                    crate::error::errors_chain_debug(s.as_ref()),
                    s
                ),
                None => String::new(),
            },
        )
    }
}

impl Fault {
    /// Usefull to generate a [Fault] when a predicate is not met.
    ///
    /// # Examples
    /// ```rust
    /// # use explicit_error_http::{Result, Fault};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Fault::new())?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            source: None,
            backtrace: Backtrace::capture(),
            context: None,
        }
    }

    /// Add an error source to a [Fault]. Usefull to generate a fault when pattern matching on an error type.
    ///
    /// On a [Result](std::result::Result) use [map_err_or_fault](crate::ResultFault::map_err_or_fault) to be more concise.
    /// # Examples
    /// ```rust
    /// # use explicit_error_http::{prelude::*, Error, HttpError, Fault, derive::HttpError};
    /// # use problem_details::ProblemDetails;
    /// # use actix_web::http::StatusCode;
    /// # use explicit_error_http::Result;
    /// fn fetch() -> Result<()> {
    ///     let sqlx_error = sqlx::Error::RowNotFound;
    ///     Err(match sqlx_error {
    ///         sqlx::Error::RowNotFound => MyEntitysError::NotFound.into(),
    ///         _ => Fault::new().with_source(sqlx_error).into()
    ///     })
    /// }
    /// # #[derive(HttpError, Debug)]
    /// # enum MyEntitysError {
    /// #    NotFound,
    /// # }
    /// # impl From<&MyEntitysError> for HttpError {
    /// #     fn from(value: &MyEntitysError) -> Self {
    /// #         match value {
    /// #             MyEntitysError::NotFound => HttpError {
    /// #               http_status_code: StatusCode::NOT_FOUND,
    /// #                 public: Box::new(
    /// #                     ProblemDetails::new()
    /// #                         .with_type(http::Uri::from_static("/errors/my-entity/not-found"))
    /// #                         .with_title("Not found"),
    /// #                     ),
    /// #                 context: Some("Some usefull info to debug".to_string()),
    /// #             },
    /// #         }
    /// #     }
    /// # }
    /// ```
    pub fn with_source<E: StdError + 'static>(self, error: E) -> Self {
        Self {
            source: Some(Box::new(error)),
            backtrace: self.backtrace,
            context: self.context,
        }
    }

    /// Add context to a [Fault], override if one was set. The context appears in display
    /// but not in the http response.
    /// # Examples
    /// ```rust
    /// # use explicit_error_http::{Result, Fault};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Fault::new().with_context("Some info to help debug"))?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_context(self, context: impl std::fmt::Display) -> Self {
        Self {
            source: self.source,
            backtrace: self.backtrace,
            context: Some(context.to_string()),
        }
    }

    /// Force backtrace capture using [force_capture](std::backtrace::Backtrace::force_capture)
    ///
    /// # Examples
    /// ```rust
    /// # use explicit_error_http::{Result, Fault};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Fault::new_force())?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_force() -> Self {
        Self {
            source: None,
            backtrace: Backtrace::force_capture(),
            context: None,
        }
    }
}

impl Default for Fault {
    fn default() -> Self {
        Self::new()
    }
}

fn serialize_source<S>(source: &Option<Box<dyn StdError>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(
        &source
            .as_ref()
            .map(|s| format!("{}: {}", crate::error::errors_chain_debug(s.as_ref()), s))
            .unwrap_or_default(),
    )
}

fn serialize_backtrace<S>(backtrace: &Backtrace, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&backtrace.to_string())
}
