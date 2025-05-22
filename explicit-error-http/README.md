Explicit error http
==============

[<img alt="crates.io" src="https://img.shields.io/crates/v/explicit-error-http.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/explicit-error-http)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-explicit-error-http-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/explicit-error-http)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Tipnos/explicit-error/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/Tipnos/explicit-error/actions?query=branch%3Amain)

<!-- cargo-rdme start -->

Built on top of [`explicit-error`](https://crates.io/crates/explicit-error), it provides idiomatic tools to manage errors that generate an HTTP response.
Based on the [explicit-error](explicit_error) crate, its chore tenet is to favor explicitness by inlining the error output while remaining concise.

The key features are:
- Explicitly mark any error wrapped in a [Result] as a [Bug]. A backtrace is captured and a 500 Internal Server HTTP response generated.
- A derive macro [HttpError](derive::HttpError) to easily declare how enum or struct errors transform into an [Error], i.e. defines the generated HTTP response.
- Inline transformation of any errors wrapped in a [Result] into an [Error].
- Add context to errors to help debug.
- Monitor errors before they are transformed into proper HTTP responses. The implementation is different depending on the web framework used, to have more details refer to the `Web frameworks` section.

## A tour of explicit-error-http

The cornerstone of the library is the [Error] type. Use `Result<T, explicit_error_http::Error>`, or equivalently `explicit_error_http::Result<T>`, as the return type of any faillible function returning errors that convert to an HTTP response.
Usually, it is mostly functions either called by handlers or middlewares.

### Inline

In the body of the function you can explicitly turn errors into HTTP response using [HttpError] or marking them as [Bug].

```rust
use actix_web::http::StatusCode;
use problem_details::ProblemDetails;
use http::Uri;
use explicit_error_http::{prelude::*, HttpError, Result, Bug};
// Import the prelude to enable functions on std::result::Result

fn business_logic() -> Result<()> {
    Err("error message").bug()?;

    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        .bug_with_source()?; // Same behavior as bug() but capture the wrapped std::error::Error as a source

    if 1 > 2 {
        Err(Bug::new()
            .with_context("Usefull context to help debug."))?;
    }

    Err(42).map_err(|_|
        HttpError::new(
            StatusCode::BAD_REQUEST,
            ProblemDetails::new()
                .with_type(Uri::from_static("/errors/business-logic"))
                .with_title("Informative feedback for the user."),
        )
    )?;

    Ok(())
}
```

Note: The crate [problem_details] is used as an example for the HTTP response body. You can, of course, use whatever you would like that implements [Serialize](serde::Serialize).

### Enum and struct

Domain errors are often represented as enum or struct as they are raised in different places.
To easily enable the conversion to [Error] use the [HttpError](derive::HttpError) derive and implement `From<&MyError> for HttpError`.

```rust
use actix_web::http::StatusCode;
use problem_details::ProblemDetails;
use http::Uri;
use explicit_error_http::{prelude::*, Result, derive::HttpError, HttpError};

#[derive(HttpError, Debug)]
enum MyError {
    Foo,
}

impl From<&MyError> for HttpError {
    fn from(value: &MyError) -> Self {
        match value {
            MyError::Foo => HttpError::new(
                    StatusCode::BAD_REQUEST,
                    ProblemDetails::new()
                        .with_type(Uri::from_static("/errors/my-domain/foo"))
                        .with_title("Foo format incorrect.")
                ),
        }
    }
}

fn business_logic() -> Result<()> {
    Err(MyError::Foo)?;

    Ok(())
}
```

Note: The [HttpError](derive::HttpError) derive implements the conversion to [Error], the impl of [Display](std::fmt::Display) (json format) and [std::error::Error].

## Pattern matching

One of the drawbacks of using one and only one return type for different domain functions is that callers loose the ability to pattern match on the returned error.
A solution is provided using [try_map_on_source](explicit_error::ResultError::try_map_on_source) on any `Result<T, Error>`, or equivalently `explicit_error_http::Result<T>`.

```rust
#[derive(HttpError, Debug)]
enum MyError {
    Foo,
    Bar,
}


fn handler() -> Result<()> {
    let err: Result<()> = Err(MyError::Foo)?;
    
    // Do the map if the source's type of the Error is MyError
    err.try_map_on_source(|e| {
        match e {
            MyError::Foo => HttpError::new(
                StatusCode::FORBIDDEN,
                ProblemDetails::new()
                    .with_type(Uri::from_static("/errors/forbidden"))
               ),
            MyError::Bar => HttpError::new(
                StatusCode::UNAUTHORIZED,
                ProblemDetails::new()
                    .with_type(Uri::from_static("/errors/unauthorized"))
               ),
        }
    })?;

    Ok(())
}
```

Note: under the hood [try_map_on_source](explicit_error::ResultError::try_map_on_source) perform some downcasting.

### Web frameworks

explicit-error-http integrates well with most popular web frameworks by providing a feature flag for each of them.

#### Actix web

The type [Error] cannot directly be used as handlers or middlewares returned [Err] variant. A dedicated type is required.
The easiest implementation is to declare a [Newtype](https://doc.rust-lang.org/rust-by-example/generics/new_types.html),
derive it with the [HandlerError] and implement the [HandlerError] trait.

```rust
#[derive(HandlerError)]
struct MyHandlerError(Error);

impl HandlerError for MyHandlerError {
    // Used by the derive for conversion
    fn from_http_error(value: Error) -> Self {
        MyHandlerError(value)
    }

    // Set-up monitoring and your custom HTTP response body for bugs
    fn public_bug_response(bug: &Bug) -> impl Serialize {
        #[cfg(debug_assertions)]
        error!("{bug}");

        #[cfg(not(debug_assertions))]
        error!("{}", serde_json::json!(bug));

        ProblemDetails::new()
            .with_type(http::Uri::from_static("/errors/internal-server-error"))
            .with_title("Internal server error")
    }

    fn http_error(&self) -> &Error {
        &self.0
    }

    // Monitor domain variant of your errors
    fn on_domain_response(error: &explicit_error_http::DomainError) {
        if error.output.http_status_code.as_u16() < 500 {
            debug!("{error}");
        } else {
            error!("{error}");
        }
    }
}

#[get("/my-handler")]
async fn my_handler() -> Result<HttpResponse, MyHandlerError> {
    Ok(HttpResponse::Ok().finish())
}
```

<!-- cargo-rdme end -->
