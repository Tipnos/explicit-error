use crate::{Error, ExitError};
use explicit_error::{Domain, Error as ExplicitError};
use std::{error::Error as StdError, fmt::Display};

/// Wrapper for errors that are not a [Fault](explicit_error::Fault). It is used as the [explicit_error::Error::Domain] variant generic type.
///
/// It is highly recommended to implement the derive [ExitError](crate::derive::ExitError) which generates the boilerplate
/// for your domain errors. Otherwise you can implement the [ToDomainError] trait.
///
/// [Error] implements `From<DomainError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain](explicit_error::Error::Domain) variant.
/// # Examples
/// [DomainError] can be generated because of a predicate
/// ```rust
/// use explicit_error_exit::{prelude::*, ExitError, Result, derive::ExitError};
/// use std::process::ExitCode;
///
/// #[derive(ExitError, Debug)]
/// enum MyError {
///     Foo,
/// }
///
/// impl From<&MyError> for ExitError {
///     fn from(value: &MyError) -> Self {
///         match value {
///             MyError::Foo => ExitError::new(
///                     "Something went wrong because ..",
///                     ExitCode::from(42)
///                 ),
///         }
///     }
/// }
///
/// fn business_logic() -> Result<()> {
///     if 1 < 2 {
///         Err(MyError::Foo)
///             .with_context("Usefull context to debug or monitor")?;
///     }
/// #   Ok(())
/// }
/// ```
/// Or from a [Result]
/// ```rust
/// # use explicit_error_exit::{prelude::*, ExitError, Result, derive::ExitError};
/// # use std::process::ExitCode;
/// # #[derive(Debug)]
/// # enum ErrorKind {
/// #     Foo,
/// #     Bar,
/// # }
/// # impl std::fmt::Display for ErrorKind {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
/// #       write!(f, "ErrorKind")
/// #    }
/// # }
/// # impl std::error::Error for ErrorKind {}
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
/// fn business_logic() -> Result<()> {   
///    Err(ErrorKind::Foo).map_err_or_fault(|e|
///         match e {
///             ErrorKind::Foo => Ok(MyError::Foo),
///             _ => Err(e)
///         }
///     )?;
/// # Ok(())
/// }
/// ```
/// Or an [Option]
/// ```rust
/// # use explicit_error_exit::ExitError;
/// # use std::process::ExitCode;
/// Some(12).ok_or(ExitError::new("Something went wrong because ..", ExitCode::from(42)))?;
/// # Ok::<(), ExitError>(())
/// ```
#[derive(Debug)]
pub struct DomainError {
    pub output: ExitError,
    pub source: Option<Box<dyn StdError>>,
}

impl Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.output)
    }
}

impl StdError for DomainError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|o| o.as_ref())
    }
}

impl From<DomainError> for ExplicitError<DomainError> {
    fn from(value: DomainError) -> Self {
        Error::Domain(Box::new(value))
    }
}

impl Domain for DomainError {
    fn into_source(self) -> Option<Box<dyn std::error::Error>> {
        self.source
    }

    fn with_context(mut self, context: impl Display) -> Self {
        self.output = self.output.with_context(context);
        self
    }
}

/// Internally used by [ExitError](crate::derive::ExitError) derive.
pub trait ToDomainError
where
    Self: StdError + 'static + Into<Error>,
    for<'a> &'a Self: Into<ExitError>,
{
    fn to_domain_error(self) -> DomainError {
        DomainError {
            output: (&self).into(),
            source: Some(Box::new(self)),
        }
    }

    fn display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bin_error: ExitError = self.into();
        write!(f, "{}", bin_error)
    }
}

/// To use this trait on [Result] import the prelude `use explicit_error_exit::prelude::*`
pub trait ResultDomainWithContext<T, D>
where
    D: ToDomainError,
    for<'a> &'a D: Into<ExitError>,
{
    /// Add a context to an error that convert to [Error] wrapped in a [Result::Err]
    /// # Examples
    /// ```rust
    /// use explicit_error::{prelude::*, Fault};
    /// Err::<(), _>(Fault::new()).with_context("Foo bar");
    /// ```
    fn with_context(self, context: impl std::fmt::Display) -> std::result::Result<T, DomainError>;
}

impl<T, D> ResultDomainWithContext<T, D> for std::result::Result<T, D>
where
    D: ToDomainError,
    for<'a> &'a D: Into<ExitError>,
{
    fn with_context(self, context: impl std::fmt::Display) -> std::result::Result<T, DomainError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.to_domain_error().with_context(context)),
        }
    }
}
