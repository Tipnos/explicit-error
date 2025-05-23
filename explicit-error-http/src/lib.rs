//! Built on top of [`explicit-error`](https://crates.io/crates/explicit-error), it provides idiomatic tools to manage errors that generate an HTTP response.
//! Based on the [explicit-error](explicit_error) crate, its chore tenet is to favor explicitness by inlining the error output while remaining concise.
//!
//! The key features are:
//! - Explicitly mark any error wrapped in a [Result] as a [Bug]. A backtrace is captured and a 500 Internal Server HTTP response generated.
//! - A derive macro [HttpError](derive::HttpError) to easily declare how enum or struct errors transform into an [Error], i.e. defines the generated HTTP response.
//! - Inline transformation of any errors wrapped in a [Result] into an [Error].
//! - Add context to errors to help debug.
//! - Monitor errors before they are transformed into proper HTTP responses. The implementation is different depending on the web framework used, to have more details refer to the `Web frameworks` section.
//!
//! # A tour of explicit-error-http
//!
//! The cornerstone of the library is the [Error] type. Use `Result<T, explicit_error_http::Error>`, or equivalently `explicit_error_http::Result<T>`, as the return type of any faillible function returning errors that convert to an HTTP response.
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
//!     Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
//!         .bug()?;
//!
//!     // Same behavior as bug() but the error is not captured as a source because it does not implement `[std::error::Error]`
//!     Err("error message").bug_no_source()?;
//!
//!     if 1 > 2 {
//!         Err(Bug::new()
//!             .with_context("Usefull context to help debug."))?;
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
//! To easily enable the conversion to [Error] use the [HttpError](derive::HttpError) derive and implement `From<&MyError> for HttpError`.
//!
//! ```rust
//! use actix_web::http::StatusCode;
//! use problem_details::ProblemDetails;
//! use http::Uri;
//! use explicit_error_http::{prelude::*, Result, derive::HttpError, HttpError};
//!
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
//!     Err(MyError::Foo)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: The [HttpError](derive::HttpError) derive implements the conversion to [Error], the impl of [Display](std::fmt::Display) (json format) and [std::error::Error].
//!
//! # Pattern matching
//!
//! One of the drawbacks of using one and only one return type for different domain functions is that callers loose the ability to pattern match on the returned error.
//! A solution is provided using [try_map_on_source](explicit_error::ResultError::try_map_on_source) on any `Result<T, Error>`, or equivalently `explicit_error_http::Result<T>`.
//!
//! ```rust
//! # use actix_web::http::StatusCode;
//! # use http::Uri;
//! # use problem_details::ProblemDetails;
//! # use explicit_error_http::{prelude::*, HttpError, Result, derive::HttpError};
//! #[derive(HttpError, Debug)]
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
//! Note: under the hood [try_map_on_source](explicit_error::ResultError::try_map_on_source) perform some downcasting.
//!
//! ## Web frameworks
//!
//! explicit-error-http integrates well with most popular web frameworks by providing a feature flag for each of them.
//!
//! ### Actix web
//!
//! The type [Error] cannot directly be used as handlers or middlewares returned [Err] variant. A dedicated type is required.
//! The easiest implementation is to declare a [Newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html),
//! derive it with the [HandlerError] and implement the [HandlerError] trait.
//!
//! ```rust
//! # use actix_web::{App, HttpResponse, HttpServer, get};
//! # use env_logger::Env;
//! # use explicit_error_http::{Bug, Error, HandlerError, derive::HandlerError};
//! # use log::{debug, error};
//! # use problem_details::ProblemDetails;
//! # use serde::Serialize;
//! #[derive(HandlerError)]
//! struct MyHandlerError(Error);
//!
//! impl HandlerError for MyHandlerError {
//!     // Used by the derive for conversion
//!     fn from_error(value: Error) -> Self {
//!         MyHandlerError(value)
//!     }
//!
//!     // Set-up monitoring and your custom HTTP response body for bugs
//!     fn public_bug_response(bug: &Bug) -> impl Serialize {
//!         #[cfg(debug_assertions)]
//!         error!("{bug}");
//!
//!         #[cfg(not(debug_assertions))]
//!         error!("{}", serde_json::json!(bug));
//!
//!         ProblemDetails::new()
//!             .with_type(http::Uri::from_static("/errors/internal-server-error"))
//!             .with_title("Internal server error")
//!     }
//!
//!     fn error(&self) -> &Error {
//!         &self.0
//!     }
//!
//!     // Monitor domain variant of your errors and eventually override their body
//!     fn domain_response(error: &explicit_error_http::DomainError) -> impl Serialize {
//!         if error.output.http_status_code.as_u16() < 500 {
//!             debug!("{error}");
//!         } else {
//!             error!("{error}");
//!         }
//!         error
//!     }
//! }
//!
//! #[get("/my-handler")]
//! async fn my_handler() -> Result<HttpResponse, MyHandlerError> {
//!     Ok(HttpResponse::Ok().finish())
//! }
//! ```
#[cfg(feature = "actix-web")]
mod actix;
mod domain;
mod error;
mod handler;

pub use domain::*;
pub use error::*;
pub use handler::*;

/// Re-import from [explicit_error] crate.
pub use explicit_error::Bug;

pub type Error = explicit_error::Error<DomainError>;
pub type Result<T> = std::result::Result<T, explicit_error::Error<DomainError>>;

pub mod prelude {
    pub use explicit_error::prelude::*;
}

pub mod derive {
    pub use explicit_error_derive::HandlerError;
    pub use explicit_error_derive::HttpError;
}
