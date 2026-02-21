extern crate mrubyedge;
extern crate mrubyedge_math as mruby_math;

mod helpers;
use helpers::*;

#[test]
fn test_math_sin() {
    let code = "
    Math.sin(Math::PI / 6.0)
    ";
    let binary = mrbc_compile("math_sin", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - 0.5).abs() < 1e-10);
}

#[test]
fn test_math_cos() {
    let code = "
    Math.cos(Math::PI)
    ";
    let binary = mrbc_compile("math_cos", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value + 1.0).abs() < 1e-10);
}

#[test]
fn test_math_sqrt() {
    let code = "
    Math.sqrt(16)
    ";
    let binary = mrbc_compile("math_sqrt", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 4.0);
}

#[test]
fn test_math_log() {
    let code = "
    Math.log(Math::E)
    ";
    let binary = mrbc_compile("math_log", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - 1.0).abs() < 1e-10);
}

#[test]
fn test_math_log_with_base() {
    let code = "
    Math.log(8, 2)
    ";
    let binary = mrbc_compile("math_log_base", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - 3.0).abs() < 1e-10);
}

#[test]
fn test_math_atan2() {
    let code = "
    Math.atan2(1, 1)
    ";
    let binary = mrbc_compile("math_atan2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - std::f64::consts::PI / 4.0).abs() < 1e-10);
}

#[test]
fn test_math_hypot() {
    let code = "
    Math.hypot(3, 4)
    ";
    let binary = mrbc_compile("math_hypot", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - 5.0).abs() < 1e-10);
}

#[test]
fn test_math_exp() {
    let code = "
    Math.exp(1)
    ";
    let binary = mrbc_compile("math_exp", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mruby_math::init_math(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert!((value - std::f64::consts::E).abs() < 1e-10);
}
