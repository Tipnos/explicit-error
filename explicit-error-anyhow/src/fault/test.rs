use super::*;

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
        Fault::new()
            .with_source(sqlx::Error::RowNotFound)
            .source()
            .unwrap()
            .downcast_ref::<sqlx::Error>()
            .is_some()
    );

    assert!(Fault::new().source().is_none());
}

#[test]
fn new() {
    let fault = Fault::new();
    assert!(fault.context().is_none());
    assert!(fault.source.is_none());
    assert_eq!(fault.backtrace.status(), BacktraceStatus::Disabled);
}

#[test]
fn with_source() {
    assert!(
        Fault::new()
            .with_source(sqlx::Error::RowNotFound)
            .source
            .unwrap()
            .downcast::<sqlx::Error>()
            .is_ok()
    );
}

#[test]
fn with_context() {
    let fault = Fault::new().with_context("context");
    assert_eq!(fault.context, vec!["context".to_string()]);

    let fault = fault.with_context("context 2");
    assert_eq!(
        fault.context,
        vec!["context".to_string(), "context 2".to_string()]
    );
    // Most recently added context wins for `context()`.
    assert_eq!(fault.context().unwrap(), "context 2");
    // But nothing is lost: `contexts()` exposes the full stack, in call order.
    assert_eq!(
        fault.contexts(),
        &["context".to_string(), "context 2".to_string()]
    );
}

#[test]
fn new_force() {
    let fault = Fault::new_force();
    assert!(fault.context().is_none());
    assert!(fault.source.is_none());
    assert_eq!(fault.backtrace.status(), BacktraceStatus::Captured);
}

#[test]
fn backtrace_status() {
    assert_eq!(Fault::new().backtrace_status(), BacktraceStatus::Disabled);
    assert_eq!(
        Fault::new_force().backtrace.status(),
        BacktraceStatus::Captured
    );
}

#[test]
fn context() {
    assert_eq!(
        Fault::new().with_context("context").context().unwrap(),
        "context"
    );
}

#[test]
fn display_prints_context_and_source_most_recent_first() {
    struct RootCause;
    impl std::fmt::Debug for RootCause {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "ROOT_CAUSE")
        }
    }
    impl std::fmt::Display for RootCause {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "root cause display")
        }
    }
    impl std::error::Error for RootCause {}

    struct TopError(RootCause);
    impl std::fmt::Debug for TopError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TOP_ERROR")
        }
    }
    impl std::fmt::Display for TopError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "top error display")
        }
    }
    impl std::error::Error for TopError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    let fault = Fault::new()
        .with_context("first context")
        .with_context("second context")
        .with_context("third context")
        .with_source(TopError(RootCause));

    assert_eq!(
        fault.to_string(),
        "Context: third context->second context->first context\n\
         Source: TOP_ERROR->ROOT_CAUSE, top error display\n"
    );
}

#[test]
fn map_err_or_fault() {
    let closure = |e: MyError| match e.0 {
        true => Ok(sqlx::Error::RowNotFound),
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
    assert_eq!(Some(()).ok_or_fault_force().unwrap(), ());
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
