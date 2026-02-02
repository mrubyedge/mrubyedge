extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn float_clamp_test() {
    let code = "
result1 = 100.5.clamp(50.0, 150.0)
result2 = 25.5.clamp(50.0, 150.0)
result3 = 200.5.clamp(50.0, 150.0)
result1 + result2 + result3
    ";
    let binary = mrbc_compile("float_clamp", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 300.5); // 100.5 + 50.0 + 150.0
}

#[test]
fn float_clamp_with_integer_bounds_test() {
    let code = "
result1 = 100.5.clamp(50, 150)
result2 = 25.5.clamp(50, 150)
result3 = 200.5.clamp(50, 150)
result1 + result2 + result3
    ";
    let binary = mrbc_compile("float_clamp_int", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 300.5); // 100.5 + 50.0 + 150.0
}
