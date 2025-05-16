//! This crate provides idiomatic tools to manage errors that generate an HTTP response.
//! Its chore tenet is to favor inline explicitness while remaining concise.
//!
//! The key features are:
//! - Explicitly mark any error wrapped in a [Result] as a [Bug]. A backtrace is captured and a 500 Internal Server HTTP response generated.
//! - A derive macro to easily declare how enum or struct errors transform into an [HttpError], i.e. defines the generated HTTP response.
//! - Inline transformation of any errors wrapped in a [Result] into an [HttpError].
//! - Add context to errors to help debugging.
//! - Monitor errors before they are transformed into proper HTTP responses. The implementation is different depending on the web framework used, to have more details refer to the `Web frameworks` section.
//!
//! # A tour of explicit_error
//!
//! The cornerstone of the library is the [HttpError] type. Use `Result<T, explicit_error::HttpError>`, or equivalently `explicit_error::Result<T>`, as the return type of any faillible function returning errors that convert to HTTP response.
//! Usually, it is mostly functions either called by handlers or middlewares.
//!
//! ## Inline
//!
//! In the body of the function you can explicitly turn errors into HTTP response using [HttpErrorData] or marking them as [Bug].
//!
//! ```rust
//! use actix_web::http::StatusCode;
//! use problem_details::ProblemDetails;
//! use http::Uri;
//! use explicit_error::{prelude::*, HttpErrorData, Result, Bug};
//! // Import the prelude to enable functions on std::result::Result
//!
//! fn business_logic() -> Result<()> {
//!     Err("error message").bug()?;
//!
//!     Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
//!         .bug_with_source()?; // Same behavior as bug() but capture the wrapped std::error::Error as a source
//!
//!     if 1 > 2 {
//!         Err(Bug::new()
//!             .with_context("Usefull context to help debugging."))?;
//!     }
//!
//!     Err(42).map_err(|_|
//!         HttpErrorData::new(
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
//! Note: The crate [problem_details] is used as an example for the HTTP response body. You can, of course, use whatever you would like that implements [Serialize](serde::Serialize).
//!
//! ## Enum and struct
//!
//! Domain errors are often represented as enum or struct as they are raised in different places.
//! To easily enable the conversion to [HttpError] use the [Error] derive and implement `From<YourError> for HttpErrorData`.
//!
//! ```rust
//! use actix_web::http::StatusCode;
//! use problem_details::ProblemDetails;
//! use http::Uri;
//! use explicit_error::{prelude::*, Result};
//!
//! #[derive(Error, Debug)]
//! enum MyError {
//!     Foo,
//! }
//!
//! impl From<&MyError> for HttpErrorData {
//!     fn from(value: &MyError) -> Self {
//!         match value {
//!             MyError::Foo => HttpErrorData::new(
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
//!     Err(MyError::Foo)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: The [Error] derive implements the conversion to [HttpError], the impl of [Display](std::fmt::Display) (json format) and [std::error::Error].
//!
//! # Pattern matching
//!
//! One of the drawbacks of using one and only one return type for different domain functions is that callers loose the ability to pattern match on the returned error.
//! A solution is provided using [try_map_on_source](ResultHttpError::try_map_on_source) on any `Result<T, HttpError>`, or equivalently `explicit_error::Result<T>`.
//!
//!
//! ```rust
//! # use actix_web::http::StatusCode;
//! # use http::Uri;
//! # use problem_details::ProblemDetails;
//! # use explicit_error::{prelude::*, HttpErrorData, Result};
//! #[derive(Error, Debug)]
//! enum MyError {
//!     Foo,
//!     Bar,
//! }
//!
//! # impl From<&MyError> for HttpErrorData {
//! #    fn from(value: &MyError) -> Self {
//! #        match value {
//! #            MyError::Foo | MyError::Bar => HttpErrorData::new(
//! #                    StatusCode::BAD_REQUEST,
//! #                    ProblemDetails::new()
//! #                        .with_type(Uri::from_static("/errors/my-domain/foo"))
//! #                        .with_title("Foo format incorrect.")
//! #                ),
//! #        }
//! #    }
//! # }
//!
//! fn handler() -> Result<()> {
//!     let err: Result<()> = Err(MyError::Foo)?;
//!     
//!     // Do the map if the source's type of the HttpError is MyError
//!     err.try_map_on_source(|e| {
//!         match e {
//!             MyError::Foo => HttpErrorData::new(
//!                 StatusCode::FORBIDDEN,
//!                 ProblemDetails::new()
//!                     .with_type(Uri::from_static("/errors/forbidden"))
//!                ),
//!             MyError::Bar => HttpErrorData::new(
//!                 StatusCode::UNAUTHORIZED,
//!                 ProblemDetails::new()
//!                     .with_type(Uri::from_static("/errors/unauthorized"))
//!                ),
//!         }
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: under the hood [try_map_on_source](ResultHttpError::try_map_on_source) perform some downcasting.
//!
//! ## Web frameworks
//!
//! explicit_error integrates well with most popular web frameworks by providing a feature flag for each of them.
//!
//! ### Actix web
//!
//! ### Axum
//!
//! work in progress
//!
#[cfg(feature = "actix-web")]
mod actix;
mod error;
mod handler;

pub use error::*;
pub use handler::*;

#[cfg(feature = "actix-web")]
pub use explicit_error_derive::DeriveHandlerError;
pub use explicit_error_derive::Error;

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}

pub type Result<T> = std::result::Result<T, HttpError>;

pub mod prelude {
    pub use crate::{
        HandlerError, HttpErrorData, OptionBug, ResultBug, ResultBugWithContext, ResultHttpError,
        ToDomainError,
    };
    pub use explicit_error_derive::Error;
}
