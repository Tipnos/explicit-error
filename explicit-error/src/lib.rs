mod bug;
mod domain;
mod errors;

pub use bug::*;
pub use domain::*;
pub use errors::*;

fn unwrap_failed(msg: &str, error: &dyn std::fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}
