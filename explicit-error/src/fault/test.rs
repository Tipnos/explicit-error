use super::*;

#[test]
fn from_for_error() {
    assert!(explicit_error_exit::Error::from(explicit_error_exit::Fault::new()).is_fault());
}

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
    assert_eq!(fault.context.as_ref().unwrap(), "context");
    assert_eq!(
        fault.with_context("context 2").context.unwrap(),
        "context 2"
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
