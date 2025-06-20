Explicit error exit
==============

<!-- cargo-rdme start -->

Built on top of [`explicit-error`](https://crates.io/crates/explicit-error), it provides idiomatic tools to manage errors that ends a process/program.
Based on the [explicit-error](explicit_error) crate, its chore tenet is to favor explicitness by inlining the error output while remaining concise.

The key features are:
- Provide [MainResult] as a returned type of crate's main function to have well formated error.
- Explicitly mark any error wrapped in a [Result] as a [Fault], a backtrace is captured.
- Inline transformation of any errors wrapped in a [Result] into an [Error].
- A derive macro [ExitError](derive::ExitError) to easily declare how enum or struct errors transform into an [Error].
- Add context to errors to help debug.

## A tour of explicit-error-bin

The cornerstone of the library is the [Error] type. Use `Result<T, explicit_error_http::Error>`, or equivalently `explicit_error_bin::Result<T>`, as the return type of any faillible function returning errors that can end the program.

### Inline

In the body of the function you can explicitly turn errors as exit errors using [ExitError] or marking them as [Fault].
```rust
use explicit_error_exit::{prelude::*, ExitError, Result, Fault, MainResult};
use std::process::ExitCode;
// Import the prelude to enable functions on std::result::Result

fn main() -> MainResult { // Error message returned: "Error: Something went wrong because .."
    business_logic()?;
    Ok(())
}

fn business_logic() -> Result<()> {
    Err(42).map_err(|e|
        ExitError::new(
            "Something went wrong because ..",
            ExitCode::from(e)
        )
    )?;

    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        .or_fault()?;
    
    // Same behavior as or_fault() but the error is not captured as a source because it does not implement `[std::error::Error]`
    Err("error message").or_fault_no_source()?;

    if 1 > 2 {
        Err(Fault::new()
            .with_context("Usefull context to help debug."))?;
    }

    Ok(())
}
```

### Enum and struct

Domain errors are often represented as enum or struct as they are raised in different places.
To easily enable the conversion to [Error] use the [ExitError](derive::ExitError) derive and implement `From<&MyError> for ExitError`.

```rust
use explicit_error_exit::{prelude::*, ExitError, Result, derive::ExitError};
use std::process::ExitCode;

#[derive(ExitError, Debug)]
enum MyError {
    Foo,
}

impl From<&MyError> for ExitError {
    fn from(value: &MyError) -> Self {
        match value {
            MyError::Foo => ExitError::new(
                    "Something went wrong because ..",
                    ExitCode::from(42)
                ),
        }
    }
}

fn business_logic() -> Result<()> {
    Err(MyError::Foo)?;

    Ok(())
}
```

Note: The [ExitError](derive::ExitError) derive implements the conversion to [Error], the impl of [Display](std::fmt::Display) and [std::error::Error].

## Pattern matching

One of the drawbacks of using one and only one return type for different domain functions is that callers loose the ability to pattern match on the returned error.
A solution is provided using [try_map_on_source](explicit_error::ResultError::try_map_on_source) on any `Result<T, Error>`, or equivalently `explicit_error_exit::Result<T>`.

```rust
use explicit_error_exit::{prelude::*, ExitError, Result, derive::ExitError};
use std::process::ExitCode;

#[derive(ExitError, Debug)]
enum MyError {
    Foo,
    Bar
}

fn business_logic() -> Result<()> {
    let err: Result<()> = Err(MyError::Foo)?;

    // Do the map if the source's type of the Error is MyError
    err.try_map_on_source(|e| {
        match e {
            MyError::Foo => ExitError::new(
                "Foo",
                ExitCode::SUCCESS),
            MyError::Bar => ExitError::new(
                "Bar",
                ExitCode::FAILURE),
        }
    })?;

    Ok(())
}
```

Note: under the hood [try_map_on_source](explicit_error::ResultError::try_map_on_source) perform some downcasting.

<!-- cargo-rdme end -->
