use super::Error;
use std::error::Error as StdError;
use std::fmt::Display;

/// To use this trait on [Result] import the prelude `use explicit_error_anyhow::prelude::*`
pub trait ResultError<T> {
    /// Try to downcast the [Error::Fault] source according to the closure's parameter type.
    /// If it is successful, execute the closure to map.
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::{prelude::*, Error, Fault};
    ///
    /// # #[derive(Debug)]
    /// # enum MyError {
    /// #     Foo,
    /// #     Bar,
    /// # }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    /// #         write!(f, "{self:?}")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # fn handler() -> Result<(), Error> {
    ///     let err: Result<(), Error> = Err(MyError::Foo).or_fault()?;
    ///
    ///     // Do the map if the source's type of the Error is MyError
    ///     err.try_downcast_fault_source_then_map(|e: MyError| -> Error {
    ///         match e {
    ///             MyError::Foo => MyError::Foo.into(), // Convert to Error::Domain
    ///             MyError::Bar => Fault::new().into(), // Convert to Error::Fault
    ///         }
    ///     })?;
    /// #     Ok(())
    /// # }
    /// ```
    fn try_downcast_fault_source_then_map<F, S, E>(self, op: F) -> Result<T, Error>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error>;

    /// Try to downcast the error typed wrapped in [Error::Domain] according to the closure's parameter type.
    /// If it is successful, execute the closure to map.
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::{prelude::*, Error, Fault};
    ///
    /// # #[derive(Debug)]
    /// # enum MyError {
    /// #     Foo,
    /// #     Bar,
    /// # }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    /// #         write!(f, "{self:?}")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// # fn handler() -> Result<(), Error> {
    ///     let err: Result<(), Error> = Err(MyError::Foo)?;
    ///
    ///     // Do the map if the source's type of the Error is MyError
    ///     err.try_downcast_domain_then_map(|e: MyError| -> Error {
    ///         match e {
    ///             MyError::Foo => MyError::Foo.into(), // Convert to Error::Domain
    ///             MyError::Bar => Fault::new().into(), // Convert to Error::Fault
    ///         }
    ///     })?;
    /// #     Ok(())
    /// # }
    /// ```
    fn try_downcast_domain_then_map<F, S, E>(self, op: F) -> Result<T, Error>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error>;

    /// Add a context to any variant of an [Error] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::{prelude::*, Fault};
    /// Err::<(), _>(Fault::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl Display) -> Result<T, Error>;
}

impl<T> ResultError<T> for Result<T, Error>
where
    T: std::fmt::Debug,
{
    fn try_downcast_domain_then_map<F, S, E>(self, op: F) -> Result<T, Error>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => match error {
                Error::Domain { inner, context } => {
                    if inner.is::<S>() {
                        return Err(op(*inner.downcast::<S>().unwrap()).into());
                    }

                    Err(Error::Domain { inner, context })
                }
                Error::Fault(_) => Err(error),
            },
        }
    }

    fn try_downcast_fault_source_then_map<F, S, E>(self, op: F) -> Result<T, Error>
    where
        F: FnOnce(S) -> E,
        S: StdError + 'static,
        E: Into<Error>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => match error {
                Error::Domain { .. } => Err(error),
                Error::Fault(b) => {
                    if let Some(s) = &b.source
                        && s.is::<S>()
                    {
                        return Err(op(*b.source.unwrap().downcast::<S>().unwrap()).into());
                    }

                    Err(Error::Fault(b))
                }
            },
        }
    }

    fn with_context(self, context: impl Display) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.with_context(context)),
        }
    }
}
