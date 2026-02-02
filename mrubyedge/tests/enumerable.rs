extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use std::rc::Rc;

use helpers::*;
use mrubyedge::yamrb::value::RObject;

#[test]
fn array_map_basic_test() {
    let code = r#"
    def test_array_map
      [1, 2, 3].map { |x| x * 2 }
    end
    "#;
    let binary = mrbc_compile("array_map_basic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_map", &args).unwrap();
    let result: (i32, i32, i32) = result.as_ref().try_into().unwrap();
    assert_eq!(result, (2, 4, 6));
}

#[test]
fn array_map_nested_test() {
    let code = r#"
    def array_map_nested
      [[1,1,1], [2,2,2], [3,3,3]].map { |arr| arr.map { |x| x * 2 } }
    end
    "#;
    let binary = mrbc_compile("array_map_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "array_map_nested", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    let r0: (i32, i32, i32) = result_array[0].as_ref().try_into().unwrap();
    let r1: (i32, i32, i32) = result_array[1].as_ref().try_into().unwrap();
    let r2: (i32, i32, i32) = result_array[2].as_ref().try_into().unwrap();
    assert_eq!(r0, (2, 2, 2));
    assert_eq!(r1, (4, 4, 4));
    assert_eq!(r2, (6, 6, 6));
}

#[test]
fn array_find_found_test() {
    let code = r#"
    def test_array_find_found
      [1, 2, 3, 4, 5].find { |x| x > 3 }
    end
    "#;
    let binary = mrbc_compile("array_find_found", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_find_found", &args).unwrap();
    let result_value: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_value, 4);
}

#[test]
fn array_find_not_found_test() {
    let code = r#"
    def test_array_find_not_found
      [1, 2, 3, 4, 5].find { |x| x > 10 }
    end
    "#;
    let binary = mrbc_compile("array_find_not_found", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_find_not_found", &args).unwrap();
    assert!(result.is_nil());
}
