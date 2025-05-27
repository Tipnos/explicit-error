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

#[test]
fn is_domain() {
    assert!(!Error::Fault(Fault::new()).is_domain());
    assert!(Error::from(ExitError::new("", ExitCode::SUCCESS)).is_domain());
}

#[test]
fn is_fault() {
    assert!(Error::Fault(Fault::new()).is_fault());
    assert!(!Error::from(ExitError::new("", ExitCode::SUCCESS)).is_fault());
}

#[should_panic]
#[test]
fn unwrap_panic() {
    Error::Fault(Fault::new()).unwrap();
}

#[test]
fn unwrap() {
    Error::from(ExitError::new("", ExitCode::SUCCESS)).unwrap();
}

#[should_panic]
#[test]
fn unwrap_fault_panic() {
    Error::from(ExitError::new("", ExitCode::SUCCESS)).unwrap_fault();
}

#[test]
fn unwrap_fault() {
    Error::Fault(Fault::new()).unwrap_fault();
}

#[test]
fn downcast_source() {
    assert!(
        Error::Fault(Fault::new())
            .downcast_source::<Fault>()
            .is_ok()
    );
    assert!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .downcast_source::<MyError>()
            .is_ok()
    );
    assert!(
        Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: None
        }))
        .downcast_source::<DomainError>()
        .is_ok()
    );
    assert!(
        Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: Some(Box::new(MyError::default()))
        }))
        .downcast_source::<MyError>()
        .is_ok()
    );

    assert!(
        Error::Fault(Fault::new())
            .downcast_source::<MyError>()
            .is_err()
    );
}
