mod option;
mod result;

pub use option::OptionFault;
pub use result::{ResultFault, ResultFaultWithContext};

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    error::Error as StdError,
};

/// Wrapper for errors that should not happen but cannot panic.
/// It is wrapped in the [Error::Fault](crate::error::Error::Fault) variant.
///
/// To generate it from predicates use [Fault::new], from [Result] or [Option]
/// import the prelude and use either [or_fault()](crate::fault::ResultFault::or_fault),
/// [or_fault_no_source()](crate::fault::ResultFault::or_fault_no_source),
/// [or_fault_force()](crate::fault::ResultFault::or_fault_force),
/// [or_fault_no_source_force()](crate::fault::ResultFault::or_fault_no_source_force),
/// [ok_or_fault()](crate::fault::OptionFault::ok_or_fault)
/// [ok_or_fault_force()](crate::fault::OptionFault::ok_or_fault_force)
#[derive(Debug)]
pub struct Fault {
    pub source: Option<Box<dyn StdError + Send + Sync>>,
    backtrace: Backtrace,
    context: Vec<String>,
}

impl StdError for Fault {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_deref().map(|s| s as _)
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.backtrace.status() == BacktraceStatus::Captured {
            write!(f, "{}\n ----------------------- \n\n", self.backtrace)?;
        }

        // reverse the vector to print most recent contexts first like the error's sources
        for (i, context) in self.contexts().iter().rev().enumerate() {
            if i == 0 {
                write!(f, "Context: {}", context)?;
            } else {
                write!(f, "->{}", context)?;
            }
        }

        if let Some(s) = &self.source {
            write!(
                f,
                "\nSource: {}, {}",
                crate::error::errors_chain_debug(s.as_ref()),
                s
            )?;
        }

        writeln!(f)
    }
}

impl Fault {
    /// Usefull to generate a [Fault] when a predicate is not met.
    ///
    /// # Examples
    /// ```rust
    /// # use explicit_error_anyhow::{Result, Fault};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Fault::new())?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            source: None,
            backtrace: Backtrace::capture(),
            context: Vec::new(),
        }
    }

    /// Add an error source to a [Fault]. Usefull to generate a fault when pattern matching on an error type.
    ///
    /// On a [Result](std::result::Result) use [map_err_or_fault](crate::ResultFault::map_err_or_fault) to be more concise.
    /// # Examples
    /// ```rust
    /// # use explicit_error_anyhow::{Fault, Result};
    /// fn fetch() -> Result<()> {
    ///     let sqlx_error = sqlx::Error::RowNotFound;
    ///     Err(match sqlx_error {
    ///         sqlx::Error::RowNotFound => MyEntitiesError::NotFound.into(),
    ///         _ => Fault::new().with_source(sqlx_error).into()
    ///     })
    /// }
    /// # #[derive(Debug)]
    /// # enum MyEntitiesError {
    /// #    NotFound,
    /// # }
    /// # impl std::fmt::Display for MyEntitiesError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    /// #         write!(f, "{self:?}")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyEntitiesError {}
    /// ```
    pub fn with_source<E: StdError + 'static + Send + Sync>(self, error: E) -> Self {
        Self {
            source: Some(Box::new(error)),
            backtrace: self.backtrace,
            context: self.context,
        }
    }

    /// Add context to a [Fault]. Cumulates: each call stacks a new message on top of
    /// any previous ones.
    /// # Examples
    /// ```rust
    /// # use explicit_error_anyhow::{Result, Fault, prelude::*};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Fault::new()
    ///         .with_context("Some info to help debug")
    ///         .with_context("Even more info"))?;
    /// }
    ///
    /// # Ok(())
    /// # }
    ///
    /// # let err = doc();
    /// err.with_context("More context added during unstacking");
    /// ```
    pub fn with_context(mut self, context: impl std::fmt::Display) -> Self {
        self.context.push(context.to_string());
        self
    }

    /// Force backtrace capture using [force_capture](std::backtrace::Backtrace::force_capture)
    ///
    /// # Examples
    /// ```rust
    /// # use explicit_error_anyhow::{Result, Fault};
    /// # fn doc() -> Result<()> {
    /// if 1 < 2 {
    ///     Err(Fault::new_force())?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_force() -> Self {
        Self {
            source: None,
            backtrace: Backtrace::force_capture(),
            context: Vec::new(),
        }
    }

    /// Return the status of the backtrace
    pub fn backtrace_status(&self) -> BacktraceStatus {
        self.backtrace.status()
    }

    /// Return the most recently added context, if any.
    pub fn context(&self) -> Option<&str> {
        self.context.last().map(String::as_str)
    }

    /// Return every context message added, in the order they were added.
    pub fn contexts(&self) -> &[String] {
        &self.context
    }
}

impl Default for Fault {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test;
