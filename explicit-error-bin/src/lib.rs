use explicit_error::{Domain, Error as ExplicitError};
use std::{error::Error as StdError, fmt::Display, process::ExitCode};

pub type Error = ExplicitError<DomainError>;
pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use explicit_error::prelude::*;
}

#[derive(Debug)]
pub struct DomainError {
    pub output: BinError,
    pub source: Option<Box<dyn StdError>>,
}

impl Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.output)
    }
}

impl StdError for DomainError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|o| o.as_ref())
    }
}

impl From<DomainError> for ExplicitError<DomainError> {
    fn from(value: DomainError) -> Self {
        Error::Domain(Box::new(value))
    }
}

impl Domain for DomainError {
    fn into_source(self) -> Option<Box<dyn std::error::Error>> {
        self.source
    }

    fn with_context(mut self, context: impl Display) -> Self {
        self.output = self.output.with_context(context);
        self
    }
}

#[derive(Debug)]
pub struct BinError {
    pub message: String,
    pub exit_code: ExitCode,
    pub context: Option<String>,
}

impl BinError {
    pub fn new(message: impl Display, exit_code: ExitCode) -> Self {
        Self {
            message: message.to_string(),
            exit_code,
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Display) -> Self {
        self.context = Some(context.to_string());
        self
    }
}

impl Display for BinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
