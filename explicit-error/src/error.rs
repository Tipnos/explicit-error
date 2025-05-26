use crate::domain::Domain;
use crate::fault::*;
use crate::unwrap_failed;
use std::{error::Error as StdError, fmt::Display};

/// Use `Result<T, explicit_error::Error>` as the return type of any binary crate
/// faillible function returning errors.
/// The [Error::Fault] variant is for errors that should not happen but cannot panic.
/// The [Error::Domain] variant is for domain errors that provide feedbacks to the user.
/// For library or functions that require the caller to pattern match on the returned error, a dedicated type is prefered.
#[derive(Debug)]
pub enum Error<D: Domain> {
    Domain(Box<D>), // Box for size: https://doc.rust-lang.org/clippy/lint_configuration.html#large-error-threshold
    Fault(Fault),
}

impl<D> StdError for Error<D>
where
    D: Domain,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Domain(explicit_error) => Some(explicit_error.as_ref()),
            Error::Fault(fault) => fault.source.as_ref().map(|e| e.as_ref()),
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
            Error::Fault(fault) => fault.fmt(f),
        }
    }
}

impl<D> Error<D>
where
    D: Domain,
{
    /// Return true if it's a [Error::Domain] variant
    pub fn is_domain(&self) -> bool {
        matches!(*self, Error::Domain(_))
    }

    /// Return true if it's a [Error::Fault] variant
    pub fn is_fault(&self) -> bool {
        !self.is_domain()
    }

    /// Unwrap the [Error::Domain] variant, panic otherwise
    pub fn unwrap(self) -> D {
        match self {
            Self::Domain(e) => *e,
            Self::Fault(f) => unwrap_failed("called `Error::unwrap()` on an `Fault` value", &f),
        }
    }

    /// Unwrap the [Error::Fault] variant, panic otherwise
    pub fn unwrap_fault(self) -> Fault {
        match self {
            Self::Fault(b) => b,
            Self::Domain(e) => {
                unwrap_failed("called `Error::unwrap_err()` on an `Domain` value", &e)
            }
        }
    }

    /// Unwrap the source of either [Error::Domain] or [Error::Fault] variant, panic otherwise
    pub fn unwrap_source(self) -> Box<dyn StdError + 'static> {
        match self {
            Error::Domain(domain) => domain.into_source().unwrap(),
            Error::Fault(fault) => fault.source.unwrap(),
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
pub trait ResultFault<T, S> {
    /// Convert with a closure any error wrapped in a [Result] to an [Error]. Returning an [Ok] convert the wrapped type to
    /// [Error::Domain].
    /// Returning an [Err] generates a [Fault] with the orginal error has its source.
    /// # Examples
    /// Pattern match to convert to an [Error::Domain]
    /// ```rust
    /// # use actix_web::http::StatusCode;
    /// # use problem_details::ProblemDetails;
    /// # use http::Uri;
    /// # use explicit_error_http::{Error, prelude::*, HttpError, derive::HttpError};
    /// fn authz_middleware(public_identifier: &str) -> Result<(), Error> {
    ///     let entity = fetch_bar(&public_identifier).map_err_or_fault(|e|
    ///         match e {
    ///             sqlx::Error::RowNotFound => Ok(
    ///                 NotFoundError::Bar(
    ///                     public_identifier.to_string())),
    ///             _ => Err(e), // Convert to Error::Fault
    ///         }
    ///     )?;
    ///
    ///     Ok(entity)
    /// }
    /// # fn fetch_bar(public_identifier: &str) -> Result<(), sqlx::Error> {
    /// #    Err(sqlx::Error::RowNotFound)
    /// # }
    /// # #[derive(HttpError, Debug)]
    /// # enum NotFoundError {
    /// #     Bar(String)
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
    /// ```
    fn map_err_or_fault<F, E, D>(self, op: F) -> Result<T, Error<D>>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<Error<D>>,
        S: StdError + 'static,
        D: Domain;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Fault]
    /// Use [fault](ResultFault::fault) instead if the error implements [std::error::Error]
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_exit::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let file: Result<File, std::io::Error> = File::open("foo.conf");
    ///     file.fault_no_source().with_context("Configuration file foo.conf is missing.")?;
    ///
    ///     Err("error message").fault_no_source()?;
    ///     # Ok(())
    /// }
    /// ```
    fn fault_no_source(self) -> Result<T, Fault>;

    /// Convert any [Result::Err] wrapping an error that implements
    /// [std::error::Error] into a [Result::Err] wrapping a [Fault]
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_exit::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     Err(sqlx::Error::RowNotFound)
    ///         .fault()
    ///         .with_context("Configuration file foo.conf is missing.")?;
    ///     # Ok(())
    /// }
    /// ```
    fn fault(self) -> Result<T, Fault>
    where
        S: StdError + 'static;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Fault] forcing backtrace capture
    /// Use [fault_force](ResultFault::fault_force) instead if the error implements [std::error::Error]
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_exit::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let file: Result<File, std::io::Error> = File::open("foo.conf");
    ///     file.fault_force().with_context("Configuration file foo.conf is missing.")?;
    ///     # Ok(())
    /// }
    /// ```
    fn fault_no_source_force(self) -> Result<T, Fault>;

    /// Convert any [Result::Err] wrapping an error that implements
    /// [std::error::Error] into a [Result::Err] wrapping a [Fault] forcing backtrace capture
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_exit::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     Err(sqlx::Error::RowNotFound)
    ///         .fault_force()
    ///         .with_context("Configuration file foo.conf is missing.")?;
    ///     # Ok(())
    /// }
    /// ```
    fn fault_force(self) -> Result<T, Fault>
    where
        S: StdError + 'static;
}

impl<T, S> ResultFault<T, S> for Result<T, S> {
    fn map_err_or_fault<F, E, D>(self, op: F) -> Result<T, Error<D>>
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
                Err(e) => Fault::new().with_source(e).into(),
            }),
        }
    }

    fn fault_no_source(self) -> Result<T, Fault> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Fault::new()),
        }
    }

    fn fault_no_source_force(self) -> Result<T, Fault> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Fault::new_force()),
        }
    }

    fn fault(self) -> Result<T, Fault>
    where
        S: StdError + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Fault::new().with_source(error)),
        }
    }

    fn fault_force(self) -> Result<T, Fault>
    where
        S: StdError + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Fault::new_force().with_source(error)),
        }
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error::prelude::*`
pub trait ResultError<T, D>
where
    D: Domain,
{
    /// Pattern match on the [Error] source from either the [Error::Fault] or [Error::Domain] variant
    /// if its type is the closure's parameter type.
    /// # Examples
    /// ```rust
    /// # use actix_web::http::StatusCode;
    /// # use http::Uri;
    /// # use problem_details::ProblemDetails;
    /// # use explicit_error_http::{prelude::*, HttpError, Result, derive::HttpError};
    /// # #[derive(HttpError, Debug)]
    /// # enum MyError {
    /// #     Foo,
    /// #     Bar,
    /// # }
    /// # impl From<&MyError> for HttpError {
    /// #    fn from(value: &MyError) -> Self {
    /// #        match value {
    /// #            MyError::Foo | MyError::Bar => HttpError::new(
    /// #                    StatusCode::BAD_REQUEST,
    /// #                    ProblemDetails::new()
    /// #                        .with_type(Uri::from_static("/errors/my-domain/foo"))
    /// #                        .with_title("Foo format incorrect.")
    /// #                ),
    /// #        }
    /// #    }
    /// # }
    /// # fn handler() -> Result<()> {
    ///     let err: Result<()> = Err(MyError::Foo)?;
    ///     
    ///     // Do the map if the source's type of the Error is MyError
    ///     err.try_map_on_source(|e| {
    ///         match e {
    ///             MyError::Foo => HttpError::new(
    ///                 StatusCode::FORBIDDEN,
    ///                 ProblemDetails::new()
    ///                     .with_type(Uri::from_static("/errors/forbidden"))
    ///                ),
    ///             MyError::Bar => HttpError::new(
    ///                 StatusCode::UNAUTHORIZED,
    ///                 ProblemDetails::new()
    ///                     .with_type(Uri::from_static("/errors/unauthorized"))
    ///                ),
    ///         }
    ///     })?;
    /// #     Ok(())
    /// # }
    /// ```
    fn try_map_on_source<F, S, E>(self, op: F) -> Result<T, Error<D>>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error<D>>;

    /// Add a context to any variant of an [Error] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error::{prelude::*, Fault};
    /// Err::<(), _>(Fault::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl Display) -> Result<T, Error<D>>;

    /// Unwrap and downcast the source of either [Error::Domain] or [Error::Fault] variant, panic otherwise.
    /// Usefull to assert_eq! in tests
    /// # Examples
    /// ```rust
    /// use explicit_error_exit::{ExitError, derive::ExitError, Error};
    /// # use std::process::ExitCode;
    /// #[test]
    /// fn test() {
    ///     asser_eq!(to_test().unwrap_err_source::<MyError>(), MyError::Foo);
    /// }
    ///
    /// #[derive(ExitError, Debug)]
    /// enum MyError {
    ///     Foo,
    /// }
    ///
    /// # impl From<&MyError> for ExitError {
    /// #     fn from(value: &MyError) -> Self {
    /// #         match value {
    /// #             MyError::Foo => ExitError::new(
    /// #                     "Something went wrong because ..",
    /// #                     ExitCode::from(42)
    /// #                 ),
    /// #         }
    /// #     }
    /// # }
    ///
    /// fn to_test() -> Result<(), Error> {
    ///     Err(MyError::Foo)?;
    ///     Ok(())
    /// }
    /// ```
    fn unwrap_err_source<E>(self) -> E
    where
        E: StdError + 'static;
}

impl<T, D> ResultError<T, D> for Result<T, Error<D>>
where
    D: Domain,
    T: std::fmt::Debug,
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
                    if d.source().is_some() && (d.source().as_ref().unwrap()).is::<S>() {
                        return Err(op(*d.into_source().unwrap().downcast::<S>().unwrap()).into());
                    }

                    Err(Error::Domain(d))
                }
                Error::Fault(b) => {
                    if let Some(s) = &b.source {
                        if s.is::<S>() {
                            return Err(op(*b.source.unwrap().downcast::<S>().unwrap()).into());
                        }
                    }

                    Err(Error::Fault(b))
                }
            },
        }
    }

    fn with_context(self, context: impl Display) -> Result<T, Error<D>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(match error {
                Error::Domain(explicit_error) => explicit_error.with_context(context).into(),
                Error::Fault(fault) => fault.with_context(context).into(),
            }),
        }
    }

    fn unwrap_err_source<E>(self) -> E
    where
        E: StdError + 'static,
    {
        *self.unwrap_err().unwrap_source().downcast::<E>().unwrap()
    }
}

/// To use this trait on [Option] import the prelude `use explicit_error::prelude::*`
pub trait OptionFault<T> {
    /// Convert an [Option::None] into a [Result::Err] wrapping a [Fault]
    /// ```rust
    /// # use std::fs::File;
    /// # use explicit_error_exit::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let option: Option<u8> = None;
    ///     option.fault().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn fault(self) -> Result<T, Fault>;

    /// Convert an [Option::None] into a [Result::Err] wrapping a [Fault] forcing backtrace capture
    /// ```rust
    /// # use std::fs::File;
    /// # use explicit_error_exit::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let option: Option<u8> = None;
    ///     option.fault_force().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn fault_force(self) -> Result<T, Fault>;
}

impl<T> OptionFault<T> for Option<T> {
    fn fault(self) -> Result<T, Fault> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Fault::new()),
        }
    }

    fn fault_force(self) -> Result<T, Fault> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Fault::new_force()),
        }
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error::prelude::*`
pub trait ResultFaultWithContext<T> {
    /// Add a context to the [Fault] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// # use explicit_error::{prelude::*, Fault};
    /// Err::<(), _>(Fault::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl Display) -> Result<T, Fault>;
}

impl<T> ResultFaultWithContext<T> for Result<T, Fault> {
    fn with_context(self, context: impl Display) -> Result<T, Fault> {
        match self {
            Ok(ok) => Ok(ok),
            Err(b) => Err(b.with_context(context)),
        }
    }
}
