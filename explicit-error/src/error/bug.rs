use super::HttpError;
use serde::{Serialize, Serializer};
use std::{backtrace::Backtrace, error::Error as StdError};

/// Wrapper that capture a [backtrace](std::backtrace) for errors that ends up as an http 500 internal server error.
///
/// [HttpError] implements `From<Bug>`, use `?` and `.into()` in functions and closures to convert to the [HttpError::Bug] variant.
///
/// To set up the log and http response body of a [Bug] implement [public_bug_response](crate::HandlerError::public_bug_response) of the [HandlerError](crate::HandlerError) trait.
/// # Examples
/// You can generate a [Bug] without source and context relying on the backtrace to help debugging.
/// ```rust
/// # use explicit_error::{Result, Bug};
/// # fn doc() -> Result<()> {
/// if 1 < 2 {
///     Err(Bug::new())?;
/// }
/// # Ok(())
/// # }
/// ```
/// To force the backtrace capture (see: [Backtrace::force_capture](std::backtrace::Backtrace::force_capture)) use [new_force](Bug::new_force).
/// ```rust
/// # use explicit_error::{Result, Bug};
/// # fn doc() -> Result<()> {
/// if 1 < 2 {
///     Err(Bug::new_force())?;
/// }
/// # Ok(())
/// # }
/// ```
///
/// When pattern matching on an error you can generate a [Bug] and attach the source to it.
/// Note: The display implementation print the source's errors chain.
/// ```rust
/// # use explicit_error::{prelude::*, HttpError, HttpErrorData, Bug};
/// # use problem_details::ProblemDetails;
/// # use actix_web::http::StatusCode;
/// # use sqlx::Error;
/// use explicit_error::Result;
///
/// fn fetch() -> Result<()> {
///     let sqlx_error = Error::RowNotFound;
///     Err(match sqlx_error {
///         Error::RowNotFound => HttpError::from(MyEntitysError::NotFound),
///         _ => Bug::new().with_source(sqlx_error).into()
///     })
/// }
///
/// # #[derive(Error, Debug)]
/// # #[explicit_error(StdError)]
/// # enum MyEntitysError {
/// #    NotFound,
/// # }
///
/// # impl From<&MyEntitysError> for HttpErrorData {
/// #     fn from(value: &MyEntitysError) -> Self {
/// #         match value {
/// #             MyEntitysError::NotFound => HttpErrorData {
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
///
/// You can also generate a [Bug] from a [Result] or an [Option].
/// The prelude must be imported first with `use explicit_error::prelude::*`.
/// ```rust
/// # use std::fs::File;
/// use explicit_error::{HttpError, prelude::*};
///
/// fn foo() -> Result<(), HttpError> {
///     let option: Option<u8> = None;
///     option.bug()?;
///
///     let file: Result<File, std::io::Error> = File::open("foo.conf");
///     file.bug().with_context("Configuration file foo.conf is missing.")?;
///     # Ok(())
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct Bug {
    #[serde(serialize_with = "serialize_source")]
    pub source: Option<Box<dyn StdError>>,
    #[serde(serialize_with = "serialize_backtrace")]
    backtrace: Backtrace,
    context: Option<String>,
}

impl From<Bug> for HttpError {
    fn from(value: Bug) -> Self {
        HttpError::Bug(value)
    }
}

impl StdError for Bug {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }
}

impl std::fmt::Display for Bug {
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
                Some(s) => format!("Source: {}, {}\n", crate::errors_chain_debug(s.as_ref()), s),
                None => String::new(),
            },
        )
    }
}

impl Bug {
    /// Usefull to generate a [Bug] when a predicate is not met.
    ///
    /// # Examples
    /// ```rust
    /// # use explicit_error::{Result, Bug};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Bug::new())?;
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

    /// Add an error source to a [Bug]. Usefull to generate a bug when pattern matching on an error type.
    ///
    /// On a [Result](std::result::Result) use [map_err_or_bug](crate::ResultBug::map_err_or_bug) to be more concise.
    /// # Examples
    /// ```rust
    /// # use explicit_error::{prelude::*, HttpError, HttpErrorData, Bug};
    /// # use problem_details::ProblemDetails;
    /// # use actix_web::http::StatusCode;
    /// # use sqlx::Error;
    /// use explicit_error::Result;
    ///
    /// fn fetch() -> Result<()> {
    ///     let sqlx_error = Error::RowNotFound;
    ///     Err(match sqlx_error {
    ///         Error::RowNotFound => MyEntitysError::NotFound.into(),
    ///         _ => Bug::new().with_source(sqlx_error).into()
    ///     })
    /// }
    ///
    /// # #[derive(Error, Debug)]
    /// # #[explicit_error(StdError)]
    /// # enum MyEntitysError {
    /// #    NotFound,
    /// # }
    ///
    /// # impl From<&MyEntitysError> for HttpErrorData {
    /// #     fn from(value: &MyEntitysError) -> Self {
    /// #         match value {
    /// #             MyEntitysError::NotFound => HttpErrorData {
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

    /// Add context to a [Bug], override if one was set. The context appears in display
    /// but not in the http response.
    /// # Examples
    /// ```rust
    /// # use explicit_error::{Result, Bug};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Bug::new().with_context("Some info to help debug"))?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_context<D: std::fmt::Display>(self, context: D) -> Self {
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
    /// # use explicit_error::{Result, Bug};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Bug::new_force())?;
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

impl Default for Bug {
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
            .map(|s| format!("{}: {}", crate::errors_chain_debug(s.as_ref()), s))
            .unwrap_or_default(),
    )
}

fn serialize_backtrace<S>(backtrace: &Backtrace, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&backtrace.to_string())
}
