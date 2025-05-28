use explicit_error_exit::{prelude::*, *};
use std::{backtrace::BacktraceStatus, error::Error as StdError, process::ExitCode};

use super::OptionFault;

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

#[test]
fn with_context() {
    assert_eq!(
        Error::Fault(Fault::new())
            .with_context("context")
            .context()
            .unwrap(),
        "context"
    );

    assert_eq!(
        Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: None
        }))
        .with_context("context")
        .context()
        .unwrap(),
        "context"
    );
}

#[test]
fn context() {
    assert_eq!(
        Error::Fault(Fault::new().with_context("context"))
            .context()
            .unwrap(),
        "context"
    );

    assert_eq!(
        Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS).with_context("context"),
            source: None
        }))
        .context()
        .unwrap(),
        "context"
    );
}

#[test]
fn errors_chain_debug() {
    #[derive(Debug)]
    struct Chain1(MyError);

    impl std::fmt::Display for Chain1 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Chain1",)
        }
    }

    impl std::error::Error for Chain1 {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            Some(&self.0)
        }
    }

    #[derive(Debug)]
    struct Chain0(Chain1);

    impl std::fmt::Display for Chain0 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Chain1",)
        }
    }

    impl std::error::Error for Chain0 {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            Some(&self.0)
        }
    }

    assert_eq!(
        crate::errors_chain_debug(&Chain0(Chain1(MyError::default()))),
        "Chain0(Chain1(MyError(true)))->Chain1(MyError(true))->MyError(true)"
    );
}

#[test]
fn map_err_or_fault() {
    let closure = |e: MyError| match e.0 {
        true => Ok(ExitError::new("", ExitCode::SUCCESS)),
        false => Err(e),
    };

    assert_eq!(Ok(()).map_err_or_fault(closure).unwrap(), ());

    assert!(
        Err::<(), _>(MyError::default())
            .map_err_or_fault(closure)
            .unwrap_err()
            .is_domain()
    );

    assert!(
        Err::<(), _>(MyError(false))
            .map_err_or_fault(closure)
            .unwrap_err()
            .is_fault()
    );
}

#[test]
fn or_fault_no_source() {
    assert_eq!(
        Err::<(), _>(())
            .or_fault_no_source()
            .unwrap_err()
            .backtrace_status(),
        BacktraceStatus::Disabled
    );
    assert!(Ok::<_, ()>(()).or_fault_no_source().is_ok());
}

#[test]
fn or_fault() {
    let fault = Err::<(), _>(MyError::default()).or_fault().unwrap_err();
    assert_eq!(fault.backtrace_status(), BacktraceStatus::Disabled);
    assert_eq!(
        *fault.source.unwrap().downcast::<MyError>().unwrap(),
        MyError::default()
    );

    assert!(Ok::<_, MyError>(()).or_fault().is_ok());
}

#[test]
fn or_fault_no_source_force() {
    assert_eq!(
        Err::<(), _>(())
            .or_fault_no_source_force()
            .unwrap_err()
            .backtrace_status(),
        BacktraceStatus::Captured
    );
    assert!(Ok::<_, ()>(()).or_fault_no_source_force().is_ok());
}

#[test]
fn or_fault_force() {
    let fault = Err::<(), _>(MyError::default())
        .or_fault_force()
        .unwrap_err();
    assert_eq!(fault.backtrace_status(), BacktraceStatus::Captured);
    assert_eq!(
        *fault.source.unwrap().downcast::<MyError>().unwrap(),
        MyError::default()
    );

    assert!(Ok::<_, MyError>(()).or_fault_force().is_ok());
}

#[test]
fn try_map_on_source() {
    assert!(
        Err::<(), _>(Error::Fault(Fault::new()))
            .try_map_on_source(|_: MyError| ExitError::new("", ExitCode::SUCCESS))
            .unwrap_err()
            .is_fault()
    );

    assert!(
        Err::<(), _>(Error::Fault(Fault::new().with_source(MyError::default())))
            .try_map_on_source(|_: sqlx::Error| ExitError::new("", ExitCode::SUCCESS))
            .unwrap_err()
            .is_fault()
    );

    assert!(
        Err::<(), _>(Error::Fault(Fault::new().with_source(MyError::default())))
            .try_map_on_source(|_: MyError| ExitError::new("", ExitCode::SUCCESS))
            .unwrap_err()
            .is_domain()
    );

    assert!(
        Err::<(), _>(Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: None
        })))
        .try_map_on_source(|_: MyError| Fault::new())
        .unwrap_err()
        .is_domain()
    );

    assert!(
        Err::<(), _>(Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: Some(Box::new(MyError::default()))
        })))
        .try_map_on_source(|_: sqlx::Error| Fault::new())
        .unwrap_err()
        .is_domain()
    );

    assert!(
        Err::<(), _>(Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: Some(Box::new(MyError::default()))
        })))
        .try_map_on_source(|_: MyError| Fault::new())
        .unwrap_err()
        .is_fault()
    );
}

#[test]
fn result_with_context() {
    assert_eq!(
        Err::<(), _>(Error::Fault(Fault::new()))
            .with_context("context")
            .unwrap_err()
            .context()
            .unwrap(),
        "context"
    );

    assert_eq!(
        Err::<(), _>(Error::Domain(Box::new(DomainError {
            output: ExitError::new("", ExitCode::SUCCESS),
            source: None
        })))
        .with_context("context")
        .unwrap_err()
        .context()
        .unwrap(),
        "context"
    );

    assert!(Ok::<(), Fault>(()).with_context("context").is_ok());
}

#[test]
fn unwrap_err_source() {
    assert_eq!(
        Err::<(), _>(Error::Fault(Fault::new().with_source(MyError::default())))
            .unwrap_err()
            .downcast_source::<MyError>()
            .unwrap(),
        MyError::default()
    );
}

#[should_panic]
#[test]
fn unwrap_err_source_panic() {
    Err::<(), _>(Error::Fault(Fault::new()))
        .unwrap_err()
        .downcast_source::<MyError>()
        .unwrap();
}

#[should_panic]
#[test]
fn unwrap_err_source_panic2() {
    Err::<(), _>(Error::Fault(
        Fault::new().with_source(sqlx::Error::RowNotFound),
    ))
    .unwrap_err()
    .downcast_source::<MyError>()
    .unwrap();
}

#[test]
fn ok_or_fault() {
    assert_eq!(
        None::<()>.ok_or_fault().unwrap_err().backtrace_status(),
        BacktraceStatus::Disabled
    );
    assert_eq!(Some(()).ok_or_fault().unwrap(), ());
}

#[test]
fn ok_or_fault_force() {
    assert_eq!(
        None::<()>
            .ok_or_fault_force()
            .unwrap_err()
            .backtrace_status(),
        BacktraceStatus::Captured
    );
    assert_eq!(Some(()).ok_or_fault().unwrap(), ());
}

#[test]
fn result_fault_with_context() {
    assert_eq!(
        Err::<(), _>(Fault::new())
            .with_context("context")
            .unwrap_err()
            .context()
            .unwrap(),
        "context"
    );

    assert!(Ok::<(), Fault>(()).with_context("context").is_ok());
}
