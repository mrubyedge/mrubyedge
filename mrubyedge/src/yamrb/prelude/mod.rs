//! Prelude that wires up the built-in Ruby-like standard library for yamrb.
//! Each submodule exposes initializers that install core classes and constants
//! into a [`VM`] so user bytecode starts with the expected environment.

use super::vm::VM;

pub mod array;
pub mod class;
pub mod exception;
pub mod falseclass;
pub mod hash;
pub mod integer;
pub mod module;
pub mod nilclass;
pub mod object;
pub mod range;
pub mod shared_memory;
pub mod string;
pub mod symbol;
pub mod trueclass;

#[cfg(feature = "mruby-regexp")]
pub mod regexp;

pub fn prelude(vm: &mut VM) {
    object::initialize_object(vm);
    exception::initialize_exception(vm);
    module::initialize_module(vm);
    class::initialize_class(vm);
    integer::initialize_integer(vm);
    nilclass::initialize_nilclass(vm);
    trueclass::initialize_trueclass(vm);
    falseclass::initialize_falseclass(vm);
    symbol::initialize_symbol(vm);
    string::initialize_string(vm);
    array::initialize_array(vm);
    hash::initialize_hash(vm);
    range::initialize_range(vm);
    shared_memory::initialize_shared_memory(vm);

    #[cfg(feature = "mruby-regexp")]
    regexp::initialize_regexp(vm);
}
