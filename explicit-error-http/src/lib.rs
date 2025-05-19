//! This crate provides idiomatic tools to manage errors that generate an HTTP response.
//! Its chore tenet is to favor inline explicitness while remaining concise.
//!
//! The key features are:
//! - Explicitly mark any error wrapped in a [Result] as a [Bug]. A backtrace is captured and a 500 Internal Server HTTP response generated.
//! - A derive macro to easily declare how enum or struct errors transform into an [Error], i.e. defines the generated HTTP response.
//! - Inline transformation of any errors wrapped in a [Result] into an [Error].
//! - Add context to errors to help debugging.
//! - Monitor errors before they are transformed into proper HTTP responses. The implementation is different depending on the web framework used, to have more details refer to the `Web frameworks` section.
//!
//! # A tour of explicit_error_http
//!
//! The cornerstone of the library is the [Error] type. Use `Result<T, explicit_error_http::Error>`, or equivalently `explicit_error_http::Result<T>`, as the return type of any faillible function returning errors that convert to HTTP response.
//! Usually, it is mostly functions either called by handlers or middlewares.
//!
//! ## Inline
//!
//! In the body of the function you can explicitly turn errors into HTTP response using [HttpError] or marking them as [Bug].
//!
//! ```rust
//! use actix_web::http::StatusCode;
//! use problem_details::ProblemDetails;
//! use http::Uri;
//! use explicit_error_http::{prelude::*, HttpError, Result, Bug};
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
//! Note: The crate [problem_details] is used as an example for the HTTP response body. You can, of course, use whatever you would like that implements [Serialize](serde::Serialize).
//!
//! ## Enum and struct
//!
//! Domain errors are often represented as enum or struct as they are raised in different places.
//! To easily enable the conversion to [Error] use the [Error] derive and implement `From<YourError> for HttpError`.
//!
//! ```rust
//! use actix_web::http::StatusCode;
//! use problem_details::ProblemDetails;
//! use http::Uri;
//! use explicit_error_http::{prelude::*, Result};
//!
//! #[derive(HttpErrorDerive, Debug)]
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
//!     Err(MyError::Foo)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: The [Error] derive implements the conversion to [Error], the impl of [Display](std::fmt::Display) (json format) and [std::error::Error].
//!
//! # Pattern matching
//!
//! One of the drawbacks of using one and only one return type for different domain functions is that callers loose the ability to pattern match on the returned error.
//! A solution is provided using [try_map_on_source](ResultHttpError::try_map_on_source) on any `Result<T, Error>`, or equivalently `explicit_error_http::Result<T>`.
//!
//!
//! ```rust
//! # use actix_web::http::StatusCode;
//! # use http::Uri;
//! # use problem_details::ProblemDetails;
//! # use explicit_error_http::{prelude::*, HttpError, Result};
//! #[derive(HttpErrorDerive, Debug)]
//! enum MyError {
//!     Foo,
//!     Bar,
//! }
//!
//! # impl From<&MyError> for HttpError {
//! #    fn from(value: &MyError) -> Self {
//! #        match value {
//! #            MyError::Foo | MyError::Bar => HttpError::new(
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
//!     // Do the map if the source's type of the Error is MyError
//!     err.try_map_on_source(|e| {
//!         match e {
//!             MyError::Foo => HttpError::new(
//!                 StatusCode::FORBIDDEN,
//!                 ProblemDetails::new()
//!                     .with_type(Uri::from_static("/errors/forbidden"))
//!                ),
//!             MyError::Bar => HttpError::new(
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
//! explicit_error_http integrates well with most popular web frameworks by providing a feature flag for each of them.
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
mod http;

pub use error::*;
pub use handler::*;
pub use http::*;

#[cfg(feature = "actix-web")]
pub use explicit_error_http_derive::HandlerErrorDerive;
pub use explicit_error_http_derive::HttpErrorDerive;

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use crate::{
        HandlerError, OptionBug, ResultBug, ResultBugWithContext, ResultHttpError, ToDomainError,
        http::HttpError,
    };
    pub use explicit_error_http_derive::HttpErrorDerive;
}
