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
//! ```
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
//!         debug_info: None,
//!         const_cache: Default::default(),
//!         catch_handlers: Vec::new(),
//!         send_cache: Default::default(),
//!     };
//!
//!     let mut vm = vm::VM::new_by_raw_irep(irep);
//!     let value = vm.run()?;
//!     println!("{:?}", value);
//!     Ok(())
//! }
//! ```
//!
//! # Ruby Compatibility
//!
//! mruby/edge implements a subset of the Ruby standard library.
//! The built-in classes and methods that are currently supported are listed in
//! [`COVERAGE.md`](https://github.com/mrubyedge/mrubyedge/blob/master/mrubyedge/COVERAGE.md).
//!
//! In brief, the following classes are available out of the box:
//! `Object`, `Integer`, `Float`, `String`, `Array`, `Hash`, `Range`,
//! `Symbol`, `Proc`, `NilClass`, `TrueClass`, `FalseClass`, `Module`,
//! `Class`, `Exception` (and standard subclasses), and the `Enumerable`
//! module. Additional classes such as `Random` and `Regexp` are available
//! behind Cargo feature flags (`mruby-random` and `mruby-regexp`).
//! A `SharedMemory` class unique to mruby/edge provides zero-copy access to
//! WASM linear memory.
//!
//! Loading a `*.mrb` produced by mruby 4.0's `mrbc` is also straightforward.
//! A chunk whose header does not say `RITE0400` is refused, so mruby 3.x
//! bytecode has to be recompiled. This doctest compiles a small script in
//! memory before loading and running it:
//!
//! ```
//! use mruby_compiler2_sys::MRubyCompiler2Context;
//! use mrubyedge::rite;
//! use mrubyedge::yamrb::vm;
//!
//! let script = unsafe {
//!     let mut compiler = MRubyCompiler2Context::new();
//!     compiler.compile("1 + 2").unwrap()
//! };
//! let mut rite = rite::load(&script).unwrap();
//! let mut vm = vm::VM::open(&mut rite);
//! let value = vm.run().unwrap();
//! let value: i64 = value.as_ref().try_into().unwrap();
//! assert_eq!(value, 3);
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
