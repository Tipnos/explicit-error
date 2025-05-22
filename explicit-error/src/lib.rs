//! Provide tools to have an explicit and concise error syntax for binary crates.
//!
//! To achieve this goal it provides [`explicit_error::Error`](crate::error::Error) an enum to explicitly differentiates
//! [`Bug`] errors that cannot panic from `Domain` errors that return informative feedbacks
//! to users. To generate an idiomatic syntax, `explicit-error` also provides traits implemented for [`std::result::Result`]
//! and [`std::option::Option`] .
//!
//! ```rust
//! use explicit_error_exit::{prelude::*, ExitError, derive::ExitError, Result, Bug};
//! use std::process::ExitCode;
//! # #[derive(ExitError, Debug)]
//! # enum MyError {
//! #   Foo,
//! #   Bar,
//! # }
//! # impl From<&MyError> for ExitError {
//! #   fn from(value: &MyError) -> Self {
//! #       ExitError::new(
//! #           "", ExitCode::SUCCESS
//! #       )
//! #   }
//! # }
//!
//! fn business_logic() -> Result<()> {
//!     let one = Ok::<_, MyError>(())
//!         .bug()
//!         .with_context("Usefull context to help debug.")?;
//!
//!     let two = Some(2).bug()?;
//!
//!     if 1 < 2 {
//!         Err(MyError::Foo)?;
//!     }
//!     
//!     Err(MyError::Bar).map_err_or_bug(|e| {
//!         match e {
//!             MyError::Foo => Ok(ExitError::new(
//!                 "Informative feedback",
//!                 ExitCode::FAILURE
//!             )),
//!             _ => Err(e) // Convert to a Bug with the original error as its std::error::Error source
//!         }
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! [`explicit_error::Error`](crate::error::Error) is not an opaque type because its [`Domain`](crate::error::Error::Domain) variant wraps generic.
//! Usually wrapped types are containers of one error output format that optionnaly carries the underlying error as an `std::error::Error` source.
//!
//! Two crates are built on top of `explicit-error`:
//! - [`explicit-error-http`](https://crates.io/crates/explicit-error-http) provides tools and derives to idiomatically manage and monitor errors that generate an HTTP response. It has dedicated feature flag to integrate well with most populars web frameworks (actix-web, axum WIP).
//! - [`explicit-error-exit`](https://crates.io/crates/explicit-error-exit) to manage errors that end a process/program.
//!
//! If you want to have more examples to understand the benefits have a look at [`explicit-error-http`](https://crates.io/crates/explicit-error-http) doc and examples.
//!
//! If you want to use this crate to build you own tooling for your custom error output format. Having a look at how [
//! `explicit-error-exit`](https://crates.io/crates/explicit-error-exit) is implemented is a good starting point.
//!
//! If you want to understand the crate's genesis and why it brings something new to the table, read the next two sections.
//!
//! ## Comparaison to Anyhow
//!
//! Anyhow and Explicit both aims to help error management in binary crates but they have opposite trade-offs.
//! The former favor maximum flexibility for implicitness while `explicit-error`
//! favor explicitness and output format enforcement for less flexibility.
//!
//! With Anyhow the `?` operator can be used on any error that implements `std::error::Error` in any function
//! returning `Result<_, anyhow::Error>`. There is no meaningfull information required about what the error means
//! exactly: caller must match on it? domain error? bug?
//!
//! On the contrary `explicit-error::Error` is not an opaque type. It is an enum with two variants:
//!
//! To illustrate, below an example from the `explicit-error-http` crate.
//!
//! ```rust
//! # use explicit_error::{Bug, Domain};
//! pub enum Error<D: Domain> {
//!     Domain(Box<D>), // Box for size: https://doc.rust-lang.org/clippy/lint_configuration.html#large-error-threshold
//!     Bug(Bug), // Can be generated from any `Result::Err`, `Option::None` or out of the box
//! }
//! ```
//!
//! The chore principle is that the `?` operator can be used on errors in functions
//! that return a `Result<T, explicit_error::Error<D>>` if they are either:
//! - marked as `Bug`
//! - convertible to D. Usually D represents the error output format.
//!
//! For example in the `explicit-error-http` crate, `D` is the type `HttpError`. Any faillible function
//! returning errors that convert to an HTTP response can have as a return type `Result<T, explicit_error::Error<HttpError>`.
//! To help application's domain errors represented as enums or structs to be convertible to `D`, crates provide derives to reduce boilerplate.
//!
//! Below an example from the `explicit-error-http` crate to show what the syntax looks like.
//!
//! ```rust
//! # use actix_web::http::StatusCode;
//! # use problem_details::ProblemDetails;
//! # use http::Uri;
//! # use explicit_error_http::{prelude::*, HttpError, Result, Bug, derive::HttpError};
//! #[derive(HttpError, Debug)]
//! enum MyError {
//!     Foo,
//! }
//!
//! impl From<&MyError> for HttpError {
//!     fn from(value: &MyError) -> Self {
//!         match value {
//!             MyError::Foo => HttpError::new(
//!                     StatusCode::BAD_REQUEST,
//!                     ProblemDetails::new()
//!                         .with_type(Uri::from_static("/errors/my-domain/foo"))
//!                         .with_title("Foo format incorrect.")
//!                 ),
//!         }
//!     }
//! }
//!
//! fn business_logic() -> Result<()> {
//!     // Error from a library that should not happen
//!     Err(sqlx::Error::RowNotFound)
//!         .bug()?;
//!
//!     // Application error
//!     if 1 > 2 {
//!         Err(MyError::Foo)?;
//!     }
//!
//!     // Inline error
//!     Err(42).map_err(|_|
//!         HttpError::new(
//!             StatusCode::BAD_REQUEST,
//!             ProblemDetails::new()
//!                 .with_type(Uri::from_static("/errors/business-logic"))
//!                 .with_title("Informative feedback for the user."),
//!         )
//!     )?;
//!
//!     Ok(())
//! }
//!```
//!
//! As you can see, function's error flow logic is straightforward, explicit and remain concise!
//!
//! Most of the time, for "lib functions", relying on the caller to generate a proper domain error,
//! the best implementation is to return a dedicated error type for idiomatic pattern matching.
//! For rare cases when you have to pattern match on the source error of an `explict_error::Error`,
//! [`explict_error::try_map_on_source`](crate::error::ResultError::try_map_on_source) can be used.
//!
//! ## Comparaison to ThisError
//!
//! Thiserror is a great tool for library errors. That's why it supplements well with `explicit-error`
//! which is designed for binary crates.
//!
//! _Why only relying on it for application can be a footgun?_
//!
//! When using ThisError you naturally tend to use enums as errors everywhere and heavily rely on the derive `#[from]`
//! to have conversion between types giving the ability to use the `?` operator almost everywhere without thinking.
//!
//! It is fine in small applications as the combinatory between errors remains limited. But as the code base grows
//! everything becomes more and more implicit. Understand the error logic flow starts to be really painfull as you
//! have to read multiple implementations spread in different places.
//!
//! Moreover boilerplate and coupling increase (also maintenance cost) because enums have multiple meaningless variants
//! from a domain point of view:
//! - encapsulate types from dependencies
//! - represent stuff that should not happen (aka Bug) and cannot panic.
//!
//! Finally, it is also painfull to have consistency in error output format and monitoring:
//! - Without a type to unified errors the implementation are spread
//! - With one type to unify errors (eg: a big enum called AppError), cohesion is discreased with more boilerplate
mod bug;
mod domain;
mod error;

pub use bug::*;
pub use domain::*;
pub use error::*;

pub mod prelude {
    pub use crate::error::{OptionBug, ResultBug, ResultBugWithContext, ResultError};
}

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}
