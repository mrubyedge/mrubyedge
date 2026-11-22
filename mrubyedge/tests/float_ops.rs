extern crate mrubyedge;

use mrubyedge::rite::insn::OpCode;
use mrubyedge::yamrb::value::{RObject, RValue};
use std::rc::Rc;

mod helpers;
use helpers::*;

#[test]
fn test_float_mul() {
    let code = "
    result = 3.5 * 2.0
    result
    ";
    let binary = mrbc_compile("float_mul", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 7.0);
}

#[test]
fn test_float_div() {
    let code = "
    result = 10.0 / 2.0
    result
    ";
    let binary = mrbc_compile("float_div", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 5.0);
}

#[test]
fn test_float_positive() {
    let code = "
    result = +3.5
    result
    ";
    let binary = mrbc_compile("float_positive", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 3.5);
}

#[test]
fn test_float_negative() {
    let code = "
    result = -3.5
    result
    ";
    let binary = mrbc_compile("float_negative", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, -3.5);
}

#[test]
fn test_float_power() {
    let code = "
    result = 2.0 ** 3.0
    result
    ";
    let binary = mrbc_compile("float_power", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 8.0);
}

#[test]
fn test_float_abs() {
    let code = "
    result = -3.5.abs
    result
    ";
    let binary = mrbc_compile("float_abs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 3.5);
}

#[test]
fn test_float_mul_with_integer() {
    let code = "
    result = 3.5 * 2
    result
    ";
    let binary = mrbc_compile("float_mul_int", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 7.0);
}

#[test]
fn test_float_div_with_integer() {
    let code = "
    result = 10.0 / 2
    result
    ";
    let binary = mrbc_compile("float_div_int", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 5.0);
}

#[test]
fn test_float_add() {
    let code = "
    result = 3.5 + 2.5
    result
    ";
    let binary = mrbc_compile("float_add", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 6.0);
}

#[test]
fn test_float_sub() {
    let code = "
    result = 10.0 - 3.5
    result
    ";
    let binary = mrbc_compile("float_sub", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 6.5);
}

#[test]
fn test_float_add_with_integer() {
    let code = "
    x = 2.5
    y = x + 1
    y
    ";
    let result = run_with_opcode(code, OpCode::ADDI);
    assert!(matches!(result.value, RValue::Float(3.5)), "{result:?}");
}

#[test]
fn test_custom_add_with_integer() {
    let code = "
    class Counter
      def +(amount)
        10 + amount
      end
    end
    counter = Counter.new
    result = counter + 1
    result
    ";
    let result = run_with_opcode(code, OpCode::ADDI);
    assert!(matches!(result.value, RValue::Integer(11)), "{result:?}");
}

#[test]
fn test_float_sub_with_integer() {
    let code = "
    x = 2.5
    y = x - 1
    y
    ";
    let result = run_with_opcode(code, OpCode::SUBI);
    assert!(matches!(result.value, RValue::Float(1.5)), "{result:?}");
}

#[test]
fn test_custom_sub_with_integer() {
    let code = "
    class Counter
      def -(amount)
        10 - amount
      end
    end
    counter = Counter.new
    result = counter - 1
    result
    ";
    let result = run_with_opcode(code, OpCode::SUBI);
    assert!(matches!(result.value, RValue::Integer(9)), "{result:?}");
}

fn run_with_opcode(code: &'static str, opcode: OpCode) -> Rc<RObject> {
    let binary = mrbc_compile("immediate_arithmetic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    assert!(
        vm.current_irep
            .code
            .iter()
            .any(|op| op.code as u8 == opcode as u8),
        "expected {opcode:?} in the compiled bytecode"
    );
    vm.run().unwrap()
}
