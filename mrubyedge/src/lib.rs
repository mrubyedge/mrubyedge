pub mod eval;
pub mod error;
pub mod rite;
pub mod yamrb;

pub use error::Error;

/// The version of the mrubyedge crate
#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub const VERSION: &str = version!();