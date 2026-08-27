use super::Fault;
use crate::error::Error;
use std::error::Error as StdError;

/// To use this trait on [Result] import the prelude `use explicit_error_anyhow::prelude::*`
pub trait ResultFault<T, S> {
    /// Convert with a closure any error wrapped in a [Result] to an [Error]. Returning an [Ok] convert the wrapped type to
    /// [Error::Domain].
    /// Returning an [Err] generates a [Fault] with the original error has its source.
    /// # Examples
    /// Pattern match to convert to an [Error::Domain]
    /// ```rust
    /// use explicit_error_anyhow::{Error, prelude::*};
    ///
    /// fn authz_middleware(public_identifier: &str) -> Result<(), Error> {
    ///     let entity = fetch_bar(public_identifier).map_err_or_fault(|e|
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
    /// # #[derive(Debug)]
    /// # enum NotFoundError {
    /// #     Bar(String)
    /// # }
    /// # impl std::fmt::Display for NotFoundError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    /// #         write!(f, "{self:?}")
    /// #     }
    /// # }
    /// # impl std::error::Error for NotFoundError {}
    /// ```
    fn map_err_or_fault<F, E>(self, op: F) -> Result<T, Error>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<Error>,
        S: StdError + 'static + Send + Sync;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Fault]
    /// Use [fault](ResultFault::or_fault) instead if the error implements [std::error::Error]
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_anyhow::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let file: Result<File, std::io::Error> = File::open("foo.conf");
    ///     file.or_fault_no_source().with_context("Configuration file foo.conf is missing.")?;
    ///
    ///     Err("error message").or_fault_no_source()?;
    ///     # Ok(())
    /// }
    /// ```
    fn or_fault_no_source(self) -> Result<T, Fault>;

    /// Convert any [Result::Err] wrapping an error that implements
    /// [std::error::Error] into a [Result::Err] wrapping a [Fault]
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_anyhow::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     Err(sqlx::Error::RowNotFound)
    ///         .or_fault()
    ///         .with_context("Foo not found")?;
    ///     # Ok(())
    /// }
    /// ```
    fn or_fault(self) -> Result<T, Fault>
    where
        S: StdError + 'static + Send + Sync;

    /// Convert any [Result::Err] into a [Result::Err] wrapping a [Fault] forcing backtrace capture
    /// Use [or_fault_force](ResultFault::or_fault_force) instead if the error implements [std::error::Error]
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_anyhow::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let file: Result<File, std::io::Error> = File::open("foo.conf");
    ///     file.or_fault_force().with_context("Configuration file foo.conf is missing.")?;
    ///     # Ok(())
    /// }
    /// ```
    fn or_fault_no_source_force(self) -> Result<T, Fault>;

    /// Convert any [Result::Err] wrapping an error that implements
    /// [std::error::Error] into a [Result::Err] wrapping a [Fault] forcing backtrace capture
    ///  ```rust
    /// # use std::fs::File;
    /// # use explicit_error_anyhow::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     Err(sqlx::Error::RowNotFound)
    ///         .or_fault_force()
    ///         .with_context("Foo not found.")?;
    ///     # Ok(())
    /// }
    /// ```
    fn or_fault_force(self) -> Result<T, Fault>
    where
        S: StdError + 'static + Send + Sync;
}

impl<T, S> ResultFault<T, S> for Result<T, S> {
    fn map_err_or_fault<F, E>(self, op: F) -> Result<T, Error>
    where
        F: FnOnce(S) -> Result<E, S>,
        E: Into<Error>,
        S: StdError + 'static + Send + Sync,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(match op(error) {
                Ok(d) => d.into(),
                Err(e) => Fault::new().with_source(e).into(),
            }),
        }
    }

    fn or_fault_no_source(self) -> Result<T, Fault> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Fault::new()),
        }
    }

    fn or_fault_no_source_force(self) -> Result<T, Fault> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(Fault::new_force()),
        }
    }

    fn or_fault(self) -> Result<T, Fault>
    where
        S: StdError + 'static + Send + Sync,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Fault::new().with_source(error)),
        }
    }

    fn or_fault_force(self) -> Result<T, Fault>
    where
        S: StdError + 'static + Send + Sync,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Fault::new_force().with_source(error)),
        }
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error_anyhow::prelude::*`
pub trait ResultFaultWithContext<T> {
    /// Add a context to the [Fault] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// # use explicit_error_anyhow::{prelude::*, Fault};
    /// Err::<(), _>(Fault::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl std::fmt::Display) -> Result<T, Fault>;
}

impl<T> ResultFaultWithContext<T> for Result<T, Fault> {
    fn with_context(self, context: impl std::fmt::Display) -> Result<T, Fault> {
        match self {
            Ok(ok) => Ok(ok),
            Err(b) => Err(b.with_context(context)),
        }
    }
}
