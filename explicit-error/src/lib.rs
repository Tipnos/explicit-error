mod bug;
mod domain;
mod error;

pub use bug::*;
pub use domain::*;
pub use error::*;

pub mod prelude {
    pub use crate::error::{OptionBug, ResultBug, ResultBugWithContext, ResultError};
}

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}
