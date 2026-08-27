//! Provide `anyhow`-like error ergonomics for binary crates while keeping errors that
//! should not happen but cannot panic distinguishable from expected domain errors.
//!
//! It provides [`Error`], a non-generic enum with two variants:
//! [`Fault`] for errors that should not happen, and `Domain` for expected errors that
//! carry an informative message. Like [`anyhow::Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html),
//! `?` and [`Into`] work out of the box for any error that implements
//! `std::error::Error` in any function returning
//! `Result<T, explicit_error_anyhow::Error>` (aliased as [`Result`]) — no derive or
//! per-type boilerplate required. `explicit-error-anyhow` also provides traits implemented
//! for [`std::result::Result`] and [`std::option::Option`] to generate an idiomatic syntax.
//!
//! ```rust
//! use explicit_error_anyhow::{prelude::*, Result};
//!
//! #[derive(Debug)]
//! enum MyError {
//!     Foo,
//!     Bar,
//! }
//!
//! impl std::fmt::Display for MyError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "{self:?}")
//!     }
//! }
//!
//! impl std::error::Error for MyError {}
//!
//! fn business_logic() -> Result<()> {
//!     let one = Ok::<_, MyError>(())
//!         .or_fault()
//!         .with_context("Usefull context to help debug.")?;
//!
//!     let two = Some(2).ok_or_fault()?;
//!
//!     if 1 < 2 {
//!         // No derive needed: any `std::error::Error` converts to `Error::Domain`.
//!         Err(MyError::Foo)?;
//!     }
//!
//!     Err(MyError::Bar).map_err_or_fault(|e| {
//!         match e {
//!             MyError::Foo => Ok(MyError::Foo), // Convert to `Error::Domain`
//!             _ => Err(e), // Convert to a `Fault` with the original error as its source
//!         }
//!     })?;
//!
//!     Ok(())
//! }
//! ```
//!
//! [`Error`] is an enum with two variants:
//!
//! ```rust
//! # use explicit_error_anyhow::Fault;
//! # use std::error::Error as StdError;
//! pub enum Error {
//!     Domain {
//!         inner: Box<dyn StdError + Send + Sync>, // Any error, no wrapping needed
//!         context: Option<String>,
//!     },
//!     Fault(Fault), // Can be generated from any `Result::Err`, `Option::None` or out of the box
//! }
//! ```
//!
//! A single blanket `impl<E: std::error::Error + Send + Sync + 'static> From<E> for Error`
//! makes `?`/`.into()` work for any error type.
//! [`Fault`] itself also satisfies that bound, so it is carved out of the blanket impl (via a
//! runtime type check, since trait coherence forbids a second, non-generic `impl From<Fault>
//! for Error`) and always converts to [`Error::Fault`] instead of
//! `Error::Domain`, so the two stay distinguishable at the call site with
//! `is_domain()`/`is_fault()`.
//!
//! This crate is the anyhow-flavored counterpart to [`explicit-error`](https://crates.io/crates/explicit-error),
//! which favors an explicit, per-error-type `D` generic (with derive macros in
//! [`explicit-error-http`](https://crates.io/crates/explicit-error-http) and
//! [`explicit-error-exit`](https://crates.io/crates/explicit-error-exit)) enforced at
//! compile time, over the blanket, zero-boilerplate conversion used here. Reach for
//! `explicit-error-anyhow` when you want `anyhow`'s effortless `?` but still need to tell
//! "this is a bug" apart from "this is an expected error" without downcasting.
//!
//! When you have to pattern match on the source error of an [`Error`],
//! [`try_map_on_source`](crate::error::ResultError::try_map_on_source) can be used.
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
//! - represent stuff that should not happen (aka Fault) and cannot panic.
//!
//! Finally, it is also painfull to have consistency in error output format and monitoring:
//! - Without a type to unified errors the implementation are spread
//! - With one type to unify errors (eg: a big enum called AppError), cohesion is discreased with more boilerplate
mod error;
mod fault;

pub use error::*;
pub use fault::*;

pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use crate::error::{ResultError, WithContext};
    pub use crate::fault::{OptionFault, ResultFault, ResultFaultWithContext};
}

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}
