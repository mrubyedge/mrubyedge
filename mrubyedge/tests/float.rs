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

#[test]
fn float_add_method_test() {
    let code = r#"
    def test_add
      a = 5.5
      b = 3.2
      a.+(b)
    end
    "#;
    let binary = mrbc_compile("float_add_method", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_add", &args).unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 8.7);
}

#[test]
fn float_sub_method_test() {
    let code = r#"
    def test_sub
      a = 10.5
      b = 3.2
      a.-(b)
    end
    "#;
    let binary = mrbc_compile("float_sub_method", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_sub", &args).unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert!((result_float - 7.3).abs() < 0.0001); // Floating point comparison
}

#[test]
fn float_add_integer_test() {
    let code = r#"
    def test_add_int
      a = 5.5
      i = 3
      a.+(i)
    end
    "#;
    let binary = mrbc_compile("float_add_int", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_add_int", &args).unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 8.5);
}
