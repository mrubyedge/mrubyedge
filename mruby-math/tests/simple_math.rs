extern crate mruby_math;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn test_math_pi_constant() {
    let code = "
    Math::PI
    ";
    let binary = mrbc_compile("math_pi_const", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - std::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_math_sqrt_simple() {
    let code = "
    Math.sqrt(4.0)
    ";
    let binary = mrbc_compile("math_sqrt_simple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 2.0);
}

#[test]
fn test_math_sin_simple() {
    let code = "
    Math.sin(Math::PI / 2)
    ";
    let binary = mrbc_compile("math_sin_simple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - 1.0).abs() < 1e-10);
}
