//! mruby/edge is a pure-Rust reimplementation of the mruby VM that keeps its
//! core execution engine `no_std`-friendly while striving for behavioral
//! compatibility with upstream mruby. It primarily targets WebAssembly
//! deployments, yet remains embeddable inside ordinary Rust binaries for host
//! tooling or native experimentation.
//!
//! Key goals:
//! - Written in idiomatic Rust with a `no_std` core so it can run in
//!   constrained environments.
//! - Behavior compatible with the mruby VM so existing bytecode executes as-is.
//! - First-class WebAssembly target support.
//! - Ergonomic embedding in general Rust applications.
//!
//! Basic initialization follows the pattern shown in `examples/newvm.rs`:
//!
//! ```no_run
//! use mrubyedge::yamrb::{op, vm, value::RSym};
//! use mrubyedge::rite::insn::{Fetched, OpCode};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let irep = vm::IREP {
//!         __id: 0,
//!         nlocals: 0,
//!         nregs: 7,
//!         rlen: 0,
//!         code: vec![
//!             op::Op { code: OpCode::LOADI_1, operand: Fetched::B(1), pos: 0, len: 2 },
//!             op::Op { code: OpCode::LOADI_2, operand: Fetched::B(2), pos: 2, len: 2 },
//!             op::Op { code: OpCode::MOVE, operand: Fetched::BB(4, 1), pos: 4, len: 3 },
//!             op::Op { code: OpCode::MOVE, operand: Fetched::BB(5, 2), pos: 7, len: 3 },
//!             op::Op { code: OpCode::ADD, operand: Fetched::B(4), pos: 10, len: 2 },
//!             op::Op { code: OpCode::SSEND, operand: Fetched::BBB(3, 0, 1), pos: 12, len: 4 },
//!             op::Op { code: OpCode::RETURN, operand: Fetched::B(3), pos: 16, len: 2 },
//!             op::Op { code: OpCode::STOP, operand: Fetched::Z, pos: 18, len: 1 },
//!         ],
//!         syms: vec![RSym::new("puts".to_string())],
//!         pool: Vec::new(),
//!         reps: Vec::new(),
//!         lv: None,
//!         catch_target_pos: Vec::new(),
//!     };
//!
//!     let mut vm = vm::VM::new_by_raw_irep(irep);
//!     let value = vm.run()?;
//!     println!("{:?}", value);
//!     Ok(())
//! }
//! ```
//!
//! Loading a precompiled `*.mrb` produced by `mrbc` is also straightforward
//! using `include_bytes!`:
//!
//! ```no_run
//! use mrubyedge::rite;
//! use mrubyedge::yamrb::vm;
//!
//! // Bundle the compiled script at build time.
//! const SCRIPT: &[u8] = include_bytes!("../examples/simple.mrb");
//!
//! fn run_embedded() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut rite = rite::load(SCRIPT)?;
//!     let mut vm = vm::VM::open(&mut rite);
//!     let value = vm.run()?;
//!     println!("{:?}", value);
//!     Ok(())
//! }
//! ```
pub mod error;
pub mod eval;
pub mod rite;
pub mod yamrb;

// re-exports for easier access
pub use error::Error;
pub use rite::{Rite, load};
pub use yamrb::value::RObject;
pub use yamrb::vm::VM;

/// The version of the mrubyedge crate
#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub const VERSION: &str = version!();
