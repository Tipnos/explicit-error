mod result;
mod with_context;

pub use result::ResultError;
pub use with_context::WithContext;

use crate::fault::*;
use crate::unwrap_failed;
use std::ops::Deref;
use std::{
    any::{Any, TypeId},
    error::Error as StdError,
    fmt::Display,
};

/// Use `Result<T, explicit_error_anyhow::Error>` as the return type of any binary crate
/// faillible function returning errors.
/// The [Error::Fault] variant is for errors that should not happen but cannot panic.
/// The [Error::Domain] variant is for domain errors that provide feedback to the user.
/// For library or functions that require the caller to pattern match on the returned error, a dedicated type is prefered.
#[derive(Debug)]
pub enum Error {
    Domain {
        inner: Box<dyn StdError + Send + Sync>,
        context: Vec<String>,
    },
    Fault(Fault),
}

impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        // `Fault` itself satisfies `StdError + Send + Sync + 'static`, so without this
        // check every `Fault` would silently collapse into `Error::Domain` here,
        // erasing the fault/domain distinction the enum exists to preserve. There is
        // no way to carve `Fault` out of the bound at the trait-coherence level (a
        // second, non-generic `impl From<Fault> for Error` would conflict with this
        // blanket impl), so the exclusion is done with a runtime `TypeId` check instead.
        if TypeId::of::<E>() == TypeId::of::<Fault>() {
            let fault = *(Box::new(value) as Box<dyn Any>)
                .downcast::<Fault>()
                .expect("TypeId check above guarantees E is Fault");
            return Error::Fault(fault);
        }

        Error::Domain {
            inner: Box::new(value),
            context: Vec::new(),
        }
    }
}

impl Error {
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Domain { inner, .. } => Some(inner.deref() as &(dyn StdError + 'static)),
            Error::Fault(fault) => fault.source().or(Some(fault)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Domain { inner, .. } => Display::fmt(&inner, f),
            Error::Fault(fault) => fault.fmt(f),
        }
    }
}

impl Error {
    /// Return true if it's a [Error::Domain] variant
    pub fn is_domain(&self) -> bool {
        matches!(*self, Error::Domain { .. })
    }

    /// Return true if it's a [Error::Fault] variant
    pub fn is_fault(&self) -> bool {
        !self.is_domain()
    }

    /// Unwrap the [Error::Domain] variant, panic otherwise
    pub fn unwrap(self) -> Box<dyn StdError> {
        match self {
            Self::Domain { inner, .. } => inner,
            Self::Fault(f) => unwrap_failed("called `Error::unwrap()` on an `Fault` value", &f),
        }
    }

    /// Unwrap the [Error::Fault] variant, panic otherwise
    pub fn unwrap_fault(self) -> Fault {
        match self {
            Self::Fault(b) => b,
            Self::Domain { inner, .. } => {
                unwrap_failed("called `Error::unwrap_err()` on an `Domain` value", &inner)
            }
        }
    }

    /// Try to downcast the type wrapped in [Error::Domain]
    /// Useful to assert_eq! in tests
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::Error;
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum MyError {
    ///     Foo,
    /// }
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "{self:?}")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// fn to_test() -> Result<(), Error> {
    ///     Err(MyError::Foo)?;
    ///     Ok(())
    /// }
    ///
    /// assert_eq!(
    ///     to_test().unwrap_err().downcast_domain_ref::<MyError>().unwrap(),
    ///     &MyError::Foo
    /// );
    /// ```
    pub fn downcast_domain_ref<E>(&self) -> Option<&E>
    where
        E: StdError + 'static,
    {
        match self {
            Error::Domain { inner, .. } => inner.as_ref() as &dyn StdError,
            Error::Fault(_) => return None,
        }
        .downcast_ref::<E>()
    }

    /// Try to downcast the type wrapped in [Error::Domain]
    /// Useful to assert_eq! in tests
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::Error;
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum MyError {
    ///     Foo,
    /// }
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "{self:?}")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// fn to_test() -> Result<(), Error> {
    ///     Err(MyError::Foo)?;
    ///     Ok(())
    /// }
    ///
    /// assert_eq!(to_test().unwrap_err().downcast_domain::<MyError>().unwrap(), MyError::Foo);
    /// ```
    pub fn downcast_domain<E>(self) -> Result<E, Self>
    where
        E: StdError + 'static,
    {
        match self {
            Error::Domain { inner, context } => {
                if inner.is::<E>() {
                    Ok(*inner.downcast::<E>().unwrap())
                } else {
                    Err(Error::Domain { inner, context })
                }
            }
            Error::Fault(_) => Err(self),
        }
    }

    /// Try to downcast the [Error::Fault] source if any.
    /// Useful to assert_eq! in tests
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::Error;
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum MyError {
    ///     Foo,
    /// }
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "{self:?}")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// fn to_test() -> Result<(), Error> {
    ///     Err(MyError::Foo).or_fault()?;
    ///     Ok(())
    /// }
    ///
    /// assert_eq!(
    ///     to_test().unwrap_err().downcast_fault_source_ref::<MyError>().unwrap(),
    ///     &MyError::Foo
    /// );
    /// ```
    pub fn downcast_fault_source_ref<E>(&self) -> Option<&E>
    where
        E: StdError + 'static,
    {
        if let Error::Fault(f) = self
            && let Some(source) = &f.source
        {
            source.downcast_ref::<E>()
        } else {
            None
        }
    }

    /// Try to downcast the [Error::Fault] source if any.
    /// Useful to assert_eq! in tests
    /// # Examples
    /// ```rust
    /// use explicit_error_anyhow::Error;
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum MyError {
    ///     Foo,
    /// }
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "{self:?}")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// fn to_test() -> Result<(), Error> {
    ///     Err(MyError::Foo).or_fault()?;
    ///     Ok(())
    /// }
    ///
    /// assert_eq!(to_test().unwrap_err().downcast_fault_source::<MyError>().unwrap(), MyError::Foo);
    /// ```
    pub fn downcast_fault_source<E>(self) -> Result<E, Self>
    where
        E: StdError + 'static,
    {
        match self {
            Error::Domain { .. } => Err(self),
            Error::Fault(mut fault) => match fault.source.take_if(|source| source.is::<E>()) {
                Some(source) => Ok(*source.downcast::<E>().unwrap()),
                None => Err(Error::Fault(fault)),
            },
        }
    }

    /// Add context of either [Error::Domain] or [Error::Fault] variant. Cumulates:
    /// each call stacks a new message on top of any previous ones
    pub fn with_context(self, context: impl std::fmt::Display) -> Self {
        match self {
            Error::Domain {
                inner,
                context: mut existing,
            } => {
                existing.push(context.to_string());
                Error::Domain {
                    inner,
                    context: existing,
                }
            }
            Error::Fault(fault) => Error::Fault(fault.with_context(context)),
        }
    }

    /// Return the most recently added context of either [Error::Domain] or
    /// [Error::Fault] variant, if any.
    pub fn context(&self) -> Option<&str> {
        match self {
            Error::Domain { context, .. } => context.last().map(String::as_str),
            Error::Fault(fault) => fault.context(),
        }
    }

    /// Return every context message added to either [Error::Domain] or
    /// [Error::Fault] variant, in the order they were added.
    pub fn contexts(&self) -> &[String] {
        match self {
            Error::Domain { context, .. } => context,
            Error::Fault(fault) => fault.contexts(),
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

#[cfg(test)]
mod test;
