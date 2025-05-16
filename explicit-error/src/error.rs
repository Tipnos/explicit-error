mod bug;
mod data;
mod domain;

use crate::unwrap_failed;
pub use bug::*;
pub use data::*;
pub use domain::*;
use std::error::Error as StdError;

/// # Examples
/// [HttpError::Domain] can be generated from result
/// ```rust
/// # use actix_web::http::StatusCode;
/// # use problem_details::ProblemDetails;
/// # use http::Uri;
/// use explicit_error::{HttpError, prelude::*};
/// fn authz_middleware(public_identifier: &str) -> Result<(), HttpError> {
///     let entity = fetch_bar(&public_identifier).map_err_or_bug(|e|
///         match e {
///             sqlx::Error::RowNotFound => Ok(
///                 forbidden()
///                 .with_context(NotFoundError::Bar(
///                     public_identifier.to_string()).to_string())),
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
///
/// #[derive(Error, Debug)]
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
/// # fn forbidden() -> HttpErrorData {
/// #    HttpErrorData::new(
/// #        StatusCode::FORBIDDEN,
/// #        ProblemDetails::new()
/// #            .with_type(Uri::from_static("/errors/generic#forbidden"))
/// #            .with_title("Forbidden."),
/// #    )
/// # }
/// ```
#[derive(Debug)]
pub enum HttpError {
    Domain(Box<DomainError>), // Box for size: https://doc.rust-lang.org/clippy/lint_configuration.html#large-error-threshold
    Bug(Bug),
}

impl StdError for HttpError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            HttpError::Domain(explicit_error) => Some(explicit_error.as_ref()),
            HttpError::Bug(bug) => bug.source.as_ref().map(|e| e.as_ref()),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::Domain(explicit_error) => explicit_error.fmt(f),
            HttpError::Bug(bug) => bug.fmt(f),
        }
    }
}

impl HttpError {
    pub fn is_domain(&self) -> bool {
        matches!(*self, HttpError::Domain(_))
    }

    pub fn is_bug(&self) -> bool {
        !self.is_domain()
    }
    pub fn unwrap(self) -> DomainError {
        match self {
            Self::Domain(e) => *e,
            Self::Bug(b) => unwrap_failed("called `HttpError::unwrap()` on an `Bug` value", &b),
        }
    }

    pub fn unwrap_bug(self) -> Bug {
        match self {
            Self::Bug(b) => b,
            Self::Domain(e) => {
                unwrap_failed("called `HttpError::unwrap_err()` on an `Domain` value", &e)
            }
        }
    }
}

#[doc(hidden)]
pub(crate) fn errors_chain_debug(source: &dyn StdError) -> String {
    use std::fmt::Write;
    let mut source = source;
    let mut str = format!("{:?}", source);

    while source.source().is_some() {
        source = source.source().unwrap();
        let _ = write!(&mut str, "->{:?}", source);
    }

    str
}

/// To use this trait on [Result] import the prelude `use explicit_error::prelude::*`
pub trait ResultBug<T, S> {
    /// Convert with a closure any error wrapped in a [Result] to an [HttpError]. Returning an [Ok] convert the wrapped type to
    /// an [HttpError] usually it is either a [DomainError] or [HttpErrorData] ending as [HttpError::Domain].
    /// Returning an [Err] generates a [Bug] with the orginal error has its source.
    /// # Examples
    /// Pattern match to convert to an [HttpError::Domain]
    /// ```rust
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// use explicit_error::{HttpError, prelude::*};
    /// fn authz_middleware(public_identifier: &str) -> Result<(), HttpError> {
    ///     let entity = fetch_bar(&public_identifier).map_err_or_bug(|e|
    ///         match e {
    ///             sqlx::Error::RowNotFound => Ok(
    ///                 NotFoundError::Bar(
    ///                     public_identifier.to_string())),
    ///             _ => Err(e), // Convert to HttpError::Bug
    ///         }
    ///     )?;
    ///
    ///     Ok(entity)
    /// }
    ///
    /// # fn fetch_bar(public_identifier: &str) -> Result<(), sqlx::Error> {
    /// #    Err(sqlx::Error::RowNotFound)
    /// # }
    ///
    /// #[derive(Error, Debug)]
    /// # #[explicit_error(StdError)]
    /// enum NotFoundError {
    ///     Bar(String)
    /// }
    /// # impl From<&NotFoundError> for HttpErrorData {
    /// #   fn from(value: &NotFoundError) -> Self {
    /// #       let (label, id) = match value {
    /// #           NotFoundError::Bar(public_identifier) => ("Bar", public_identifier)
    /// #       };
    /// #       HttpErrorData::new(
    /// #           StatusCode::NOT_FOUND,
    /// #           ProblemDetails::new()
    /// #               .with_type(Uri::from_static("/errors/not-found"))
    /// #               .with_title("Not found")
    /// #               .with_detail(format!("Unknown {label} with identifier {id}."))
    /// #       )
    /// #   }
    /// # }
    /// ```
    /// ```rust
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// use explicit_error::{HttpError, prelude::*};
    /// fn authz_middleware(public_identifier: &str) -> Result<(), HttpError> {
    ///     let entity = fetch_bar(&public_identifier).map_err_or_bug(|e|
    ///         match e {
    ///             sqlx::Error::RowNotFound => Ok(
    ///                 forbidden()
    ///                 .with_context(NotFoundError::Bar(
    ///                     public_identifier.to_string()).to_string())),
    ///             _ => Err(e), // Convert to HttpError::Bug
    ///         }
    ///     )?;
    ///
    ///     Ok(entity)
    /// }
    ///
    /// # fn fetch_bar(public_identifier: &str) -> Result<(), sqlx::Error> {
    /// #    Err(sqlx::Error::RowNotFound)
    /// # }
    /// # #[derive(Error, Debug)]
    /// # #[explicit_error(StdError)]
    /// # enum NotFoundError {
    /// #    Bar(String)
    /// # }
    /// # impl From<&NotFoundError> for HttpErrorData {
    /// #   fn from(value: &NotFoundError) -> Self {
    /// #       let (label, id) = match value {
    /// #           NotFoundError::Bar(public_identifier) => ("Bar", public_identifier)
    /// #       };
    /// #       HttpErrorData::new(
    /// #           StatusCode::NOT_FOUND,
    /// #           ProblemDetails::new()
    /// #               .with_type(Uri::from_static("/errors/not-found"))
    /// #               .with_title("Not found")
    /// #               .with_detail(format!("Unknown {label} with identifier {id}."))
    /// #       )
    /// #   }
    /// # }
    /// fn forbidden() -> HttpErrorData {
    ///     HttpErrorData::new(
    ///         StatusCode::FORBIDDEN,
    ///         ProblemDetails::new()
    ///             .with_type(Uri::from_static("/errors/generic#forbidden"))
    ///             .with_title("Forbidden."),
    ///     )
    /// }
    /// ```
    fn map_err_or_bug<F, E>(self, op: F) -> Result<T, HttpError>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<HttpError>,
        S: StdError + 'static;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Bug]
    ///  ```rust
    /// # use std::fs::File;
    /// use explicit_error::{HttpError, prelude::*};
    ///
    /// fn foo() -> Result<(), HttpError> {
    ///     let file: Result<File, std::io::Error> = File::open("foo.conf");
    ///     file.bug().with_context("Configuration file foo.conf is missing.")?;
    ///
    ///     Err("error message").bug()?;
    ///     # Ok(())
    /// }
    /// ```
    fn bug(self) -> Result<T, Bug>;

    /// Same behavior as `bug` but capture the original error as a source.
    /// Only applicable to wrapped [std::errror::Error](std::error::Error)
    fn bug_with_source(self) -> Result<T, Bug>
    where
        S: StdError + 'static;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Bug] forcing backtrace capture
    ///  ```rust
    /// # use std::fs::File;
    /// use explicit_error::{HttpError, prelude::*};
    ///
    /// fn foo() -> Result<(), HttpError> {
    ///
    ///     let file: Result<File, std::io::Error> = File::open("foo.conf");
    ///     file.bug_force().with_context("Configuration file foo.conf is missing.")?;
    ///     # Ok(())
    /// }
    /// ```
    fn bug_force(self) -> Result<T, Bug>;

    /// Same behavior as `bug_force` but capture the original error as a source.
    /// Only applicable to wrapped [std::errror::Error](std::error::Error)
    fn bug_force_with_source(self) -> Result<T, Bug>
    where
        S: StdError + 'static;
}

impl<T, S> ResultBug<T, S> for Result<T, S> {
    fn map_err_or_bug<F, E>(self, op: F) -> Result<T, HttpError>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<HttpError>,
        S: StdError + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(match op(error) {
                Ok(d) => d.into(),
                Err(e) => Bug::new().with_source(e).into(),
            }),
        }
    }

    fn bug(self) -> Result<T, Bug> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Bug::new()),
        }
    }

    fn bug_force(self) -> Result<T, Bug> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Bug::new_force()),
        }
    }

    fn bug_with_source(self) -> Result<T, Bug>
    where
        S: StdError + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Bug::new().with_source(error)),
        }
    }

    fn bug_force_with_source(self) -> Result<T, Bug>
    where
        S: StdError + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Bug::new_force().with_source(error)),
        }
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error::prelude::*`
pub trait ResultHttpError<T> {
    /// TODO
    fn try_map_on_source<F, S, E>(self, op: F) -> Result<T, HttpError>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<HttpError>;

    /// Add a context to any variant of an [HttpError] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error::{prelude::*, Bug};
    /// Err::<(), _>(Bug::new()).with_context("Foo bar");
    /// ```
    fn with_context<D: std::fmt::Display>(self, context: D) -> Result<T, HttpError>;
}

impl<T> ResultHttpError<T> for Result<T, HttpError> {
    fn try_map_on_source<F, S, E>(self, op: F) -> Result<T, HttpError>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<HttpError>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => match error {
                HttpError::Domain(d) => {
                    if d.source.is_some() {
                        if (d.source.as_ref().unwrap()).is::<S>() {
                            let (_, e) = d.split();
                            return Err(op(*e.unwrap().downcast::<S>().unwrap()).into());
                        }
                    }

                    Err(HttpError::Domain(d))
                }
                HttpError::Bug(b) => {
                    if let Some(s) = &b.source {
                        if s.is::<S>() {
                            return Err(op(*b.source.unwrap().downcast::<S>().unwrap()).into());
                        }
                    }

                    Err(HttpError::Bug(b))
                }
            },
        }
    }

    fn with_context<D: std::fmt::Display>(self, context: D) -> Result<T, HttpError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(match error {
                HttpError::Domain(explicit_error) => (*explicit_error).with_context(context).into(),
                HttpError::Bug(bug) => bug.with_context(context).into(),
            }),
        }
    }
}

/// To use this trait on [Option] import the prelude `use explicit_error::prelude::*`
pub trait OptionBug<T> {
    /// Convert an [Option::None] into a [Result::Err] wrapping a [Bug]
    /// ```rust
    /// # use std::fs::File;
    /// use explicit_error::{HttpError, prelude::*};
    ///
    /// fn foo() -> Result<(), HttpError> {
    ///     let option: Option<u8> = None;
    ///     option.bug().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn bug(self) -> Result<T, Bug>;

    /// Convert an [Option::None] into a [Result::Err] wrapping a [Bug] forcing backtrace capture
    /// ```rust
    /// # use std::fs::File;
    /// use explicit_error::{HttpError, prelude::*};
    ///
    /// fn foo() -> Result<(), HttpError> {
    ///     let option: Option<u8> = None;
    ///     option.bug_force().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn bug_force(self) -> Result<T, Bug>;
}

impl<T> OptionBug<T> for Option<T> {
    fn bug(self) -> Result<T, Bug> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Bug::new().into()),
        }
    }

    fn bug_force(self) -> Result<T, Bug> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Bug::new_force().into()),
        }
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error::prelude::*`
pub trait ResultBugWithContext<T> {
    /// Add a context to the [Bug] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error::{prelude::*, Bug};
    /// Err::<(), _>(Bug::new()).with_context("Foo bar");
    /// ```
    fn with_context<D: std::fmt::Display>(self, context: D) -> Result<T, Bug>;
}

impl<T> ResultBugWithContext<T> for Result<T, Bug> {
    fn with_context<D: std::fmt::Display>(self, context: D) -> Result<T, Bug> {
        match self {
            Ok(ok) => Ok(ok),
            Err(b) => Err(b.with_context(context)),
        }
    }
}
