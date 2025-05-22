use std::{fmt::Display, process::ExitCode};

use crate::{DomainError, Error};

/// Representation of errors that ends a process/program.
///
/// [Error] implements `From<ExitError>`, use `?` and `.into()` in functions and closures to convert to the [Error::Domain](explicit_error::Error::Domain) variant.
///
/// Note: [ExitError] convert to [Error] by converting first to [DomainError].
/// # Examples
/// Domain errors that derive [ExitError](crate::derive::ExitError) must implement `From<#MyDomainError> for ExitError`.
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
/// ```
///
/// Domain errors cannot require to be extracted in either a struct or enum variant.
/// You can generate [Error::Domain](explicit_error::Error::Domain) variant with an [ExitError]
/// ```rust
/// use explicit_error_exit::{prelude::*, ExitError, Result, Bug};
/// use std::process::ExitCode;
///
/// fn business_logic() -> Result<()> {
///
///     Err(42).map_err(|e|
///         ExitError::new(
///             "Something went wrong because ..",
///             ExitCode::from(e)
///         )
///     )?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ExitError {
    pub message: String,
    pub exit_code: ExitCode,
    pub context: Option<String>,
}

impl ExitError {
    /// Generate an [ExitError] without a context. To add a context
    /// use [with_context](ExitError::with_context) afterwards.
    /// # Examples
    /// ```rust
    /// use explicit_error_exit::ExitError;
    /// use std::process::ExitCode;
    ///
    /// ExitError::new(
    ///     "Something went wrong because ..",
    ///     ExitCode::from(42)
    /// );
    /// ```
    pub fn new(message: impl Display, exit_code: ExitCode) -> Self {
        Self {
            message: message.to_string(),
            exit_code,
            context: None,
        }
    }

    /// Add a context to an [ExitError], override if one was set. The context appears in display
    /// but not in the [Display] implementation.
    /// # Examples
    /// ```rust
    /// use explicit_error_exit::ExitError;
    /// use std::process::ExitCode;
    ///
    /// ExitError::new(
    ///     "Something went wrong because ..",
    ///     ExitCode::from(42)
    /// ).with_context("The reason why it went wrong");
    /// ```
    pub fn with_context(mut self, context: impl Display) -> Self {
        self.context = Some(context.to_string());
        self
    }
}

impl Display for ExitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<ExitError> for Error {
    fn from(value: ExitError) -> Self {
        Error::Domain(Box::new(DomainError {
            output: value,
            source: None,
        }))
    }
}
