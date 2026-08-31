//! `Error::Domain` for errors the user caused, `Error::Fault` for bugs -- both flow
//! through one `Result`, and `?` converts any `std::error::Error` for free, like anyhow.
//!
//! Try: `get 1` (ok), `get 99` (Domain: not found), `get 0` (Fault: bug), `get abc` (Domain: bad input)
//! Or: `tour` for a guided run through the rest of the API.

use explicit_error_anyhow::{Fault, Result, prelude::*};
use std::process::ExitCode;

#[derive(Debug, PartialEq)]
struct NotFound(u32);

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "note #{} not found", self.0)
    }
}
impl std::error::Error for NotFound {}

// Stands in for a real DB driver: same foreign, fallible `sqlx::Error` surface.
fn fetch(id: u32) -> std::result::Result<&'static str, sqlx::Error> {
    match id {
        0 => Err(sqlx::Error::PoolTimedOut), // unexpected driver failure
        1 => Ok("Buy milk"),
        _ => Err(sqlx::Error::RowNotFound),
    }
}

fn get_note(id: u32) -> Result<&'static str> {
    fetch(id).map_err_or_fault(|e| match e {
        sqlx::Error::RowNotFound => Ok(NotFound(id)), // expected -> Error::Domain
        e => Err(e),                                  // unexpected -> Error::Fault
    })
}

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("tour") {
        api_tour();
        return ExitCode::SUCCESS;
    }

    let arg = std::env::args().nth(2).unwrap_or_default();

    // `WithContext`: turn a foreign error into a domain `Error` with a message,
    // just like `anyhow::Context`.
    let result = arg
        .parse()
        .map_err(|e: std::num::ParseIntError| e.with_context(format!("'{arg}' is not a valid id")))
        .and_then(get_note);

    match result {
        Ok(note) => {
            println!("{note}");
            ExitCode::SUCCESS
        }
        Err(error) if error.is_domain() => {
            eprint!("error: {error}");
            if let Some(context) = error.context() {
                eprint!(" ({context})");
            }
            eprintln!();
            ExitCode::from(1)
        }
        Err(error) => {
            // Fault carries the source error + a backtrace (RUST_BACKTRACE=1): never show this to a user.
            eprintln!("internal error:\n{}", error.unwrap_fault());
            ExitCode::from(70)
        }
    }
}

// Advanced: a guided tour of the rest of the API, run live instead of only
// read. `cargo run --example cli -p explicit-error-anyhow -- tour`
fn api_tour() {
    println!("explicit-error-anyhow API tour\n");
    // `or_fault`/`or_fault_no_source`: turn any `Result::Err` into a `Fault`,
    // capturing the error as its source only when it implements `Error`.
    let with_source = Err::<(), _>(sqlx::Error::PoolTimedOut)
        .or_fault()
        .unwrap_err();
    let no_source = Err::<(), _>("raw failure")
        .or_fault_no_source()
        .unwrap_err();
    println!(
        "[or_fault] source: {} / [or_fault_no_source] source: {}",
        with_source.source.is_some(),
        no_source.source.is_some()
    );

    // `ok_or_fault_force`: same idea for `Option::None`; the `_force` suffix
    // (available on every adapter above too) always captures a backtrace,
    // even without `RUST_BACKTRACE=1`.
    let forced = None::<()>.ok_or_fault_force().unwrap_err();
    println!(
        "[ok_or_fault_force] backtrace: {:?}",
        forced.backtrace_status()
    );

    // `Fault` can be raised directly too, exactly like `anyhow::bail!(...)`.
    let fault = Fault::new_force().with_context("invariant broken");
    println!("[Fault::new_force] context: {:?}", fault.context());

    // `with_context` cumulates, like `anyhow::Context` -- calling it twice stacks
    // both messages instead of the second replacing the first. `context()` returns
    // the most recently added one; `contexts()` returns the full stack, in the
    // order the messages were added.
    let error = Err::<(), _>(sqlx::Error::PoolTimedOut)
        .or_fault()
        .with_context("first")
        .with_context("second")
        .unwrap_err();
    println!(
        "[with_context] latest context: {:?} / full stack: {:?}",
        error.context(),
        error.contexts()
    );

    // `Error::unwrap`: mirrors `Result::unwrap` for the Domain variant, and
    // panics instead of silently treating a bug as an expected outcome.
    println!("[Error::unwrap] {}", get_note(99).unwrap_err().unwrap());

    println!("\ntour complete");
}
