use super::*;
use std::error::Error as StdError;

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
fn from_fault() {
    assert!(Error::from(Fault::new()).is_fault());
}

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
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .source()
        .unwrap()
        .downcast_ref::<sqlx::Error>()
        .is_some()
    );
    assert_eq!(
        Error::Domain {
            inner: Box::new(MyError::default()),
            context: Vec::new()
        }
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
    assert!(Error::from(sqlx::Error::RowNotFound).is_domain());
}

#[test]
fn is_fault() {
    assert!(Error::Fault(Fault::new()).is_fault());
    assert!(!Error::from(sqlx::Error::RowNotFound).is_fault());
}

#[should_panic]
#[test]
fn unwrap_panic() {
    Error::Fault(Fault::new()).unwrap();
}

#[test]
fn unwrap() {
    Error::from(sqlx::Error::RowNotFound).unwrap();
}

#[test]
fn unwrap_fault() {
    Error::Fault(Fault::new()).unwrap_fault();
}

#[should_panic]
#[test]
fn unwrap_fault_panic() {
    Error::from(sqlx::Error::RowNotFound).unwrap_fault();
}

#[test]
fn error_source() {
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
}

#[test]
fn downcast_domain() {
    assert!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .downcast_domain::<sqlx::Error>()
        .is_ok()
    );

    assert!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .downcast_domain::<MyError>()
        .is_err()
    );

    assert!(
        Error::Fault(Fault::new())
            .downcast_domain::<MyError>()
            .is_err()
    );

    assert!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .downcast_domain_ref::<sqlx::Error>()
        .is_some()
    );

    assert!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .downcast_domain_ref::<MyError>()
        .is_none()
    );

    assert!(
        Error::Fault(Fault::new())
            .downcast_domain_ref::<MyError>()
            .is_none()
    );
}

#[test]
fn downcast_fault() {
    assert!(
        Error::Fault(Fault::new())
            .downcast_fault_source::<Fault>()
            .is_err()
    );
    assert!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .downcast_fault_source::<MyError>()
            .is_ok()
    );

    assert!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .downcast_fault_source::<Fault>()
            .is_err()
    );

    assert!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .downcast_fault_source::<sqlx::Error>()
        .is_err()
    );

    assert!(
        Error::Fault(Fault::new())
            .downcast_fault_source_ref::<Fault>()
            .is_none()
    );
    assert!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .downcast_fault_source_ref::<MyError>()
            .is_some()
    );

    assert!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .downcast_fault_source_ref::<Fault>()
            .is_none()
    );

    assert!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .downcast_fault_source_ref::<sqlx::Error>()
        .is_none()
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
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        }
        .with_context("context")
        .context()
        .unwrap(),
        "context"
    );

    let error = <Fault as WithContext>::with_context(Fault::new(), "context");
    assert!(error.is_fault());
    assert_eq!(error.context().unwrap(), "context");

    assert!(
        Error::Fault(Fault::new())
            .downcast_domain::<Fault>()
            .is_ok()
    );
    assert!(
        Error::Fault(Fault::new().with_source(MyError::default()))
            .downcast_domain::<MyError>()
            .is_ok()
    );

    assert!(
        Error::Fault(Fault::new())
            .downcast_domain::<MyError>()
            .is_err()
    );
    let error = MyError::default().with_context("context");
    assert!(error.is_domain());
    assert_eq!(error.context().unwrap(), "context");
    assert_eq!(
        error.downcast_domain::<MyError>().unwrap(),
        MyError::default()
    );
}

#[test]
fn with_context_cumulates() {
    let error = Error::Fault(Fault::new())
        .with_context("first")
        .with_context("second");
    assert_eq!(error.context().unwrap(), "second");
    assert_eq!(
        error.contexts(),
        &["first".to_string(), "second".to_string()]
    );

    let error = Error::Domain {
        inner: Box::new(sqlx::Error::RowNotFound),
        context: Vec::new(),
    }
    .with_context("first")
    .with_context("second");
    assert_eq!(error.context().unwrap(), "second");
    assert_eq!(
        error.contexts(),
        &["first".to_string(), "second".to_string()]
    );
}

#[test]
fn context() {
    assert_eq!(
        Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: vec!["context".to_string()]
        }
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
fn try_downcast_fault() {
    assert!(
        Err::<(), _>(Error::Fault(Fault::new()))
            .try_downcast_fault_source_then_map(|_: MyError| sqlx::Error::RowNotFound)
            .unwrap_err()
            .is_fault()
    );

    assert!(
        Err::<(), _>(Error::Fault(Fault::new().with_source(MyError::default())))
            .try_downcast_fault_source_then_map(|_: sqlx::Error| sqlx::Error::RowNotFound)
            .unwrap_err()
            .is_fault()
    );

    assert!(
        Err::<(), _>(Error::Fault(Fault::new().with_source(MyError::default())))
            .try_downcast_fault_source_then_map(|_: MyError| sqlx::Error::RowNotFound)
            .unwrap_err()
            .is_domain()
    );

    assert!(
        Err::<(), _>(Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        })
        .try_downcast_domain_then_map(|_: MyError| Fault::new())
        .unwrap_err()
        .is_domain()
    );

    assert!(
        Err::<(), _>(Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        })
        .try_downcast_domain_then_map(|_: MyError| Fault::new())
        .unwrap_err()
        .is_domain()
    );

    assert!(
        Err::<(), _>(Error::Domain {
            inner: Box::new(MyError::default()),
            context: Vec::new()
        })
        .try_downcast_domain_then_map(|_: MyError| Fault::new())
        .unwrap_err()
        .is_fault()
    );
}

#[test]
fn try_downcast_domain() {}

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
        Err::<(), _>(Error::Domain {
            inner: Box::new(sqlx::Error::RowNotFound),
            context: Vec::new()
        })
        .with_context("context")
        .unwrap_err()
        .context()
        .unwrap(),
        "context"
    );
}

#[test]
fn unwrap_err_source() {
    assert_eq!(
        Err::<(), _>(Error::Fault(Fault::new().with_source(MyError::default())))
            .unwrap_err()
            .downcast_domain::<MyError>()
            .unwrap(),
        MyError::default()
    );
}

#[should_panic]
#[test]
fn unwrap_err_source_panic() {
    Err::<(), _>(Error::Fault(Fault::new()))
        .unwrap_err()
        .downcast_domain::<MyError>()
        .unwrap();
}

#[should_panic]
#[test]
fn unwrap_err_source_panic2() {
    Err::<(), _>(Error::Fault(
        Fault::new().with_source(sqlx::Error::RowNotFound),
    ))
    .unwrap_err()
    .downcast_domain::<MyError>()
    .unwrap();
}
