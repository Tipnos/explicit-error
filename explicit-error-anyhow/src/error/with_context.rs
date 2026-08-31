use super::Error;
use crate::fault::Fault;
use std::any::{Any, TypeId};
use std::error::Error as StdError;
use std::fmt::Display;

/// To use this trait import the prelude `use explicit_error_anyhow::prelude::*`
///
/// Implemented for [Error] itself, for [Fault], and (via a blanket impl) for any other type
/// implementing [StdError], so context can be attached the same way regardless of which of
/// the three it is: [Error] and [Fault] preserve their variant/type, anything else becomes
/// an [Error::Domain].
pub trait WithContext {
    /// Convert any type implementing [StdError] into an [Error::Domain] with the given context.
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::{Error, prelude::*};
    ///
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "{self:?}")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// let error: Error = MyError.with_context("Additional context");
    /// ```
    fn with_context(self, context: impl Display) -> Error;
}

impl WithContext for Error {
    fn with_context(self, context: impl Display) -> Error {
        Error::with_context(self, context)
    }
}

impl<E> WithContext for E
where
    E: StdError + Send + Sync + 'static,
{
    fn with_context(self, context: impl Display) -> Error {
        // Mirrors the `Fault` carve-out in `From<E> for Error` above: without this check,
        // going through this blanket impl would wrap a `Fault` in `Error::Domain` instead of
        // `Error::Fault`, erasing the fault/domain distinction the enum exists to preserve.
        if TypeId::of::<E>() == TypeId::of::<Fault>() {
            let fault = *(Box::new(self) as Box<dyn Any>)
                .downcast::<Fault>()
                .expect("TypeId check above guarantees E is Fault");
            return Error::Fault(fault.with_context(context));
        }

        Error::Domain {
            inner: Box::new(self),
            context: vec![context.to_string()],
        }
    }
}
