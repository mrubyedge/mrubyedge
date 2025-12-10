//! Yet Another mruby (yamrb) runtime layer.
//! Provides value representation, opcode tables, helpers, and the VM itself
//! so mruby bytecode can execute inside Rust.
pub mod optable;
pub mod value;
pub mod shared_memory;
pub mod vm;
pub mod op;
pub mod helpers;

pub mod prelude;
