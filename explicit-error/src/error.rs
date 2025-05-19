use crate::bug::*;
use crate::domain::Domain;
use crate::unwrap_failed;
use std::{error::Error as StdError, fmt::Display};

#[derive(Debug)]
pub enum Error<D: Domain> {
    Domain(Box<D>), // Box for size: https://doc.rust-lang.org/clippy/lint_configuration.html#large-error-threshold
    Bug(Bug),
}

impl<D> StdError for Error<D>
where
    D: Domain,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Domain(explicit_error) => Some(explicit_error.as_ref()),
            Error::Bug(bug) => bug.source.as_ref().map(|e| e.as_ref()),
        }
    }
}

impl<D> Display for Error<D>
where
    D: Domain,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Domain(explicit_error) => Display::fmt(&explicit_error, f),
            Error::Bug(bug) => bug.fmt(f),
        }
    }
}

impl<D> Error<D>
where
    D: Domain,
{
    pub fn is_domain(&self) -> bool {
        matches!(*self, Error::Domain(_))
    }

    pub fn is_bug(&self) -> bool {
        !self.is_domain()
    }
    pub fn unwrap(self) -> D {
        match self {
            Self::Domain(e) => *e,
            Self::Bug(b) => unwrap_failed("called `Error::unwrap()` on an `Bug` value", &b),
        }
    }

    pub fn unwrap_bug(self) -> Bug {
        match self {
            Self::Bug(b) => b,
            Self::Domain(e) => {
                unwrap_failed("called `Error::unwrap_err()` on an `Domain` value", &e)
            }
        }
    }
}

pub fn errors_chain_debug(source: &dyn StdError) -> String {
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
    /// Convert with a closure any error wrapped in a [Result] to an [Error]. Returning an [Ok] convert the wrapped type to
    /// an [Error] usually it is either a [DomainError] or [HttpError] ending as [Error::Domain].
    /// Returning an [Err] generates a [Bug] with the orginal error has its source.
    /// # Examples
    /// Pattern match to convert to an [Error::Domain]
    /// ```rust
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// use explicit_error::{Error, prelude::*};
    /// fn authz_middleware(public_identifier: &str) -> Result<(), Error> {
    ///     let entity = fetch_bar(&public_identifier).map_err_or_bug(|e|
    ///         match e {
    ///             sqlx::Error::RowNotFound => Ok(
    ///                 NotFoundError::Bar(
    ///                     public_identifier.to_string())),
    ///             _ => Err(e), // Convert to Error::Bug
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
    /// #[derive(HttpErrorDerive, Debug)]
    /// # #[explicit_error(StdError)]
    /// enum NotFoundError {
    ///     Bar(String)
    /// }
    /// # impl From<&NotFoundError> for HttpError {
    /// #   fn from(value: &NotFoundError) -> Self {
    /// #       let (label, id) = match value {
    /// #           NotFoundError::Bar(public_identifier) => ("Bar", public_identifier)
    /// #       };
    /// #       HttpError::new(
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
    /// use explicit_error::{Error, prelude::*};
    /// fn authz_middleware(public_identifier: &str) -> Result<(), Error> {
    ///     let entity = fetch_bar(&public_identifier).map_err_or_bug(|e|
    ///         match e {
    ///             sqlx::Error::RowNotFound => Ok(
    ///                 forbidden()
    ///                 .with_context(NotFoundError::Bar(
    ///                     public_identifier.to_string()).to_string())),
    ///             _ => Err(e), // Convert to Error::Bug
    ///         }
    ///     )?;
    ///
    ///     Ok(entity)
    /// }
    ///
    /// # fn fetch_bar(public_identifier: &str) -> Result<(), sqlx::Error> {
    /// #    Err(sqlx::Error::RowNotFound)
    /// # }
    /// # #[derive(HttpErrorDerive, Debug)]
    /// # #[explicit_error(StdError)]
    /// # enum NotFoundError {
    /// #    Bar(String)
    /// # }
    /// # impl From<&NotFoundError> for HttpError {
    /// #   fn from(value: &NotFoundError) -> Self {
    /// #       let (label, id) = match value {
    /// #           NotFoundError::Bar(public_identifier) => ("Bar", public_identifier)
    /// #       };
    /// #       HttpError::new(
    /// #           StatusCode::NOT_FOUND,
    /// #           ProblemDetails::new()
    /// #               .with_type(Uri::from_static("/errors/not-found"))
    /// #               .with_title("Not found")
    /// #               .with_detail(format!("Unknown {label} with identifier {id}."))
    /// #       )
    /// #   }
    /// # }
    /// fn forbidden() -> HttpError {
    ///     HttpError::new(
    ///         StatusCode::FORBIDDEN,
    ///         ProblemDetails::new()
    ///             .with_type(Uri::from_static("/errors/generic#forbidden"))
    ///             .with_title("Forbidden."),
    ///     )
    /// }
    /// ```
    fn map_err_or_bug<F, E, D>(self, op: F) -> Result<T, Error<D>>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<Error<D>>,
        S: StdError + 'static,
        D: Domain;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Bug]
    ///  ```rust
    /// # use std::fs::File;
    /// use explicit_error::{Error, prelude::*};
    ///
    /// fn foo() -> Result<(), Error> {
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
    /// use explicit_error::{Error, prelude::*};
    ///
    /// fn foo() -> Result<(), Error> {
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
    fn map_err_or_bug<F, E, D>(self, op: F) -> Result<T, Error<D>>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<Error<D>>,
        S: StdError + 'static,
        D: Domain,
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
pub trait ResultError<T, D>
where
    D: Domain,
{
    /// TODO
    fn try_map_on_source<F, S, E>(self, op: F) -> Result<T, Error<D>>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error<D>>;

    /// Add a context to any variant of an [Error] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error::{prelude::*, Bug};
    /// Err::<(), _>(Bug::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl Display) -> Result<T, Error<D>>;
}

impl<T, D> ResultError<T, D> for Result<T, Error<D>>
where
    D: Domain,
{
    fn try_map_on_source<F, S, E>(self, op: F) -> Result<T, Error<D>>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error<D>>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => match error {
                Error::Domain(d) => {
                    if d.source().is_some() {
                        if (d.source().as_ref().unwrap()).is::<S>() {
                            return Err(
                                op(*d.into_source().unwrap().downcast::<S>().unwrap()).into()
                            );
                        }
                    }

                    Err(Error::Domain(d))
                }
                Error::Bug(b) => {
                    if let Some(s) = &b.source {
                        if s.is::<S>() {
                            return Err(op(*b.source.unwrap().downcast::<S>().unwrap()).into());
                        }
                    }

                    Err(Error::Bug(b))
                }
            },
        }
    }

    fn with_context(self, context: impl Display) -> Result<T, Error<D>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(match error {
                Error::Domain(explicit_error) => explicit_error.with_context(context).into(),
                Error::Bug(bug) => bug.with_context(context).into(),
            }),
        }
    }
}

/// To use this trait on [Option] import the prelude `use explicit_error::prelude::*`
pub trait OptionBug<T> {
    /// Convert an [Option::None] into a [Result::Err] wrapping a [Bug]
    /// ```rust
    /// # use std::fs::File;
    /// use explicit_error::{Error, prelude::*};
    ///
    /// fn foo() -> Result<(), Error> {
    ///     let option: Option<u8> = None;
    ///     option.bug().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn bug(self) -> Result<T, Bug>;

    /// Convert an [Option::None] into a [Result::Err] wrapping a [Bug] forcing backtrace capture
    /// ```rust
    /// # use std::fs::File;
    /// use explicit_error::{Error, prelude::*};
    ///
    /// fn foo() -> Result<(), Error> {
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
    fn with_context(self, context: impl Display) -> Result<T, Bug>;
}

impl<T> ResultBugWithContext<T> for Result<T, Bug> {
    fn with_context(self, context: impl Display) -> Result<T, Bug> {
        match self {
            Ok(ok) => Ok(ok),
            Err(b) => Err(b.with_context(context)),
        }
    }
}
