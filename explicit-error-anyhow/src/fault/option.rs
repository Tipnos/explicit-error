use super::Fault;

/// To use this trait on [Option] import the prelude `use explicit_error_anyhow::prelude::*`
pub trait OptionFault<T> {
    /// Transforms the `Option<T>` into a `Result<T, Fault>`, mapping Some(v) to Ok(v) and None to Err(Fault)
    /// ```rust
    /// # use std::fs::File;
    /// # use explicit_error_anyhow::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let option: Option<u8> = None;
    ///     option.ok_or_fault().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn ok_or_fault(self) -> Result<T, Fault>;

    /// Transforms the `Option<T>` into a `Result<T, Fault>`, mapping Some(v) to Ok(v) and None to Err(Fault)
    /// forcing backtrace capture
    /// ```rust
    /// # use std::fs::File;
    /// # use explicit_error_anyhow::{Error, prelude::*};
    /// fn foo() -> Result<(), Error> {
    ///     let option: Option<u8> = None;
    ///     option.ok_or_fault_force().with_context("Help debugging")?;
    ///     # Ok(())
    /// }
    /// ```
    fn ok_or_fault_force(self) -> Result<T, Fault>;
}

impl<T> OptionFault<T> for Option<T> {
    fn ok_or_fault(self) -> Result<T, Fault> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Fault::new()),
        }
    }

    fn ok_or_fault_force(self) -> Result<T, Fault> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Fault::new_force()),
        }
    }
}
