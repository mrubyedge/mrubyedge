//! Entry point that groups the mruby/rite (mrb) helpers.
//! It understands the binary layout, instruction stream, and marker metadata,
//! exposing higher-level APIs via the `rite` submodule.
pub mod binfmt;
pub mod insn;
pub mod marker;
#[allow(clippy::module_inception)] // FIXME rename
pub mod rite;

pub use rite::*;

use std::error;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    General,
    TooShort,
    InvalidFormat,
    InvalidOpCode,
    TypeMismatch,
    InvalidOperand,
    NoMethod,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error nr {:?}", self)
    }
}

impl error::Error for Error {}
