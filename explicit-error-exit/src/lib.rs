//! Built on top of [`explicit-error`](https://crates.io/crates/explicit-error), it provides idiomatic tools to manage errors that ends a process/program.
//! Based on the [explicit-error](explicit_error) crate, its chore tenet is to favor explicitness by inlining the error output while remaining concise.
//!
//! The key features are:
//! - Explicitly mark any error wrapped in a [Result] as a [Bug], a backtrace is captured.
//! - Inline transformation of any errors wrapped in a [Result] into an [Error].
//! - A derive macro [ExitError](derive::ExitError) to easily declare how enum or struct errors transform into an [Error].
//! - Add context to errors to help debug.
//!
//! # A tour of explicit-error-bin
//!
//! The cornerstone of the library is the [Error] type. Use `Result<T, explicit_error_http::Error>`, or equivalently `explicit_error_bin::Result<T>`, as the return type of any faillible function returning errors that can end the program.
//!
//! ## Inline
//!
//! In the body of the function you can explicitly turn errors as exit errors using [ExitError] or marking them as [Bug].
//! ```rust
//! use explicit_error_exit::{prelude::*, ExitError, Result, Bug};
//! use std::process::ExitCode;
//! // Import the prelude to enable functions on std::result::Result
//!
//! fn business_logic() -> Result<()> {
//!     Err("error message").bug_no_source()?;
//!
//!     Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
//!         .bug()?; // Same behavior as bug() but capture the wrapped std::error::Error as a source
//!
//!     if 1 > 2 {
//!         Err(Bug::new()
//!             .with_context("Usefull context to help debug."))?;
//!     }
//!
//!     Err(42).map_err(|e|
//!         ExitError::new(
//!             "Something went wrong because ..",
//!             ExitCode::from(e)
//!         )
//!     )?;
//!
//!     Ok(())
//! }
//!```
//!
//! ## Enum and struct
//!
//! Domain errors are often represented as enum or struct as they are raised in different places.
//! To easily enable the conversion to [Error] use the [ExitError](derive::ExitError) derive and implement `From<&MyError> for ExitError`.
//!
//! ```rust
//! use explicit_error_exit::{prelude::*, ExitError, Result, derive::ExitError};
//! use std::process::ExitCode;
//!
//! #[derive(ExitError, Debug)]
//! enum MyError {
//!     Foo,
//! }
//!
//! impl From<&MyError> for ExitError {
//!     fn from(value: &MyError) -> Self {
//!         match value {
//!             MyError::Foo => ExitError::new(
//!                     "Something went wrong because ..",
//!                     ExitCode::from(42)
//!                 ),
//!         }
//!     }
//! }
//!
//! fn business_logic() -> Result<()> {
//!     Err(MyError::Foo)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: The [ExitError](derive::ExitError) derive implements the conversion to [Error], the impl of [Display](std::fmt::Display) and [std::error::Error].
//!
//! # Pattern matching
//!
//! One of the drawbacks of using one and only one return type for different domain functions is that callers loose the ability to pattern match on the returned error.
//! A solution is provided using [try_map_on_source](explicit_error::ResultError::try_map_on_source) on any `Result<T, Error>`, or equivalently `explicit_error_exit::Result<T>`.
//!
//! ```rust
//! use explicit_error_exit::{prelude::*, ExitError, Result, derive::ExitError};
//! use std::process::ExitCode;
//!
//! #[derive(ExitError, Debug)]
//! enum MyError {
//!     Foo,
//!     Bar
//! }
//!
//! # impl From<&MyError> for ExitError {
//! #     fn from(value: &MyError) -> Self {
//! #         ExitError::new(
//! #           "Something went wrong because ..",
//! #           ExitCode::from(42))
//! #     }
//! # }
//! fn business_logic() -> Result<()> {
//!     let err: Result<()> = Err(MyError::Foo)?;
//!
//!     // Do the map if the source's type of the Error is MyError
//!     err.try_map_on_source(|e| {
//!         match e {
//!             MyError::Foo => ExitError::new(
//!                 "Foo",
//!                 ExitCode::SUCCESS),
//!             MyError::Bar => ExitError::new(
//!                 "Bar",
//!                 ExitCode::FAILURE),
//!         }
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: under the hood [try_map_on_source](explicit_error::ResultError::try_map_on_source) perform some downcasting.
mod domain;
mod error;

pub use domain::*;
pub use error::*;

pub type Error = explicit_error::Error<DomainError>;
pub type Result<T> = std::result::Result<T, Error>;

/// Re-import from [explicit_error] crate.
pub use explicit_error::Bug;

pub mod prelude {
    pub use crate::ResultDomainWithContext;
    pub use explicit_error::prelude::*;
}

pub mod derive {
    pub use explicit_error_derive::ExitError;
}
