use explicit_error_exit::*;
use std::{error::Error as StdError, process::ExitCode};

#[derive(Debug, PartialEq)]
struct MyError(bool);

impl Default for MyError {
    fn default() -> Self {
        Self(true)
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for MyError {}

#[test]
fn source() {
    assert!(
        Error::Fault(Fault::new())
            .source()
            .unwrap()
            .downcast_ref::<Fault>()
            .is_some()
    );
    assert_eq!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .source()
            .unwrap()
            .downcast_ref::<MyError>()
            .unwrap(),
        &MyError::default()
    );
    assert!(
        Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: None
        }))
        .source()
        .unwrap()
        .downcast_ref::<DomainError>()
        .is_some()
    );
    assert_eq!(
        Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: Some(Box::new(MyError::default()))
        }))
        .source()
        .unwrap()
        .downcast_ref::<MyError>()
        .unwrap(),
        &MyError::default()
    );
}
