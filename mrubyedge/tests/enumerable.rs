extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use std::rc::Rc;

use helpers::*;
use mrubyedge::yamrb::value::RObject;

#[test]
fn enumerable_map_basic_test() {
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
fn enumerable_map_nested_test() {
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
fn enumerable_find_found_test() {
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
fn enumerable_find_not_found_test() {
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

#[test]
fn enumerable_min_test() {
    let code = r#"
    def test_array_min
      [3, 1, 2].min
    end
    "#;
    let binary = mrbc_compile("array_min", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_min", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn enumerable_max_test() {
    let code = r#"
    def test_array_max
      [3, 1, 2].max
    end
    "#;
    let binary = mrbc_compile("array_max", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_max", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn enumerable_minmax_test() {
    let code = r#"
    def test_array_minmax
      [3, 1, 2].minmax
    end
    "#;
    let binary = mrbc_compile("array_minmax", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_minmax", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 2);
    let min: i64 = result_array[0].as_ref().try_into().unwrap();
    let max: i64 = result_array[1].as_ref().try_into().unwrap();
    assert_eq!(min, 1);
    assert_eq!(max, 3);
}

#[test]
fn enumerable_uniq_test() {
    let code = r#"
    def test_array_uniq
      [1, 2, 2, 3].uniq
    end
    "#;
    let binary = mrbc_compile("array_uniq", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_uniq", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 3);
}

#[test]
fn enumerable_select_test() {
    let code = r#"
    def test_select
      [1, 2, 3, 4, 5].select { |x| x > 3 }
    end
    "#;
    let binary = mrbc_compile("enumerable_select", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_select", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 2);
}

#[test]
fn enumerable_all_test() {
    let code = r#"
    def test_all
      [2, 4, 6].all? { |x| x % 2 == 0 }
    end
    "#;
    let binary = mrbc_compile("enumerable_all", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_all", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn enumerable_any_test() {
    let code = r#"
    def test_any
      [1, 2, 3].any? { |x| x > 2 }
    end
    "#;
    let binary = mrbc_compile("enumerable_any", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_any", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn enumerable_delete_if_test() {
    let code = r#"
    def test_delete_if
      [1, 2, 3, 4, 5].delete_if { |x| x % 2 == 0 }
    end
    "#;
    let binary = mrbc_compile("enumerable_delete_if", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_delete_if", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 3);
}

#[test]
fn enumerable_each_with_index_test() {
    let code = r#"
    def test_each_with_index
      result = []
      [10, 20, 30].each_with_index { |x, i| result.push(x + i) }
      result
    end
    "#;
    let binary = mrbc_compile("enumerable_each_with_index", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_each_with_index", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 3);
    let r0: i64 = result_array[0].as_ref().try_into().unwrap();
    let r1: i64 = result_array[1].as_ref().try_into().unwrap();
    let r2: i64 = result_array[2].as_ref().try_into().unwrap();
    assert_eq!(r0, 10);
    assert_eq!(r1, 21);
    assert_eq!(r2, 32);
}

#[test]
fn enumerable_sort_test() {
    let code = r#"
    def test_sort
      [3, 1, 4, 1, 5].sort
    end
    "#;
    let binary = mrbc_compile("enumerable_sort", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_sort", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    let r0: i64 = result_array[0].as_ref().try_into().unwrap();
    assert_eq!(r0, 1);
}

#[test]
fn enumerable_sort_by_test() {
    let code = r#"
    def test_sort_by
      [3, 1, 4, 1, 5].sort_by { |x| -x }
    end
    "#;
    let binary = mrbc_compile("enumerable_sort_by", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_sort_by", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    let r0: i64 = result_array[0].as_ref().try_into().unwrap();
    assert_eq!(r0, 5);
}

#[test]
fn enumerable_compact_test() {
    let code = r#"
    def test_compact
      [1, nil, 2, nil, 3].compact
    end
    "#;
    let binary = mrbc_compile("enumerable_compact", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_compact", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 3);
}

#[test]
fn enumerable_count_test() {
    let code = r#"
    def test_count
      [1, 2, 3, 4, 5].count { |x| x % 2 == 0 }
    end
    "#;
    let binary = mrbc_compile("enumerable_count", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_count", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn enumerable_to_a_test() {
    let code = r#"
    def test_to_a
      [1, 2, 3].to_a
    end
    "#;
    let binary = mrbc_compile("enumerable_to_a", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_to_a", &args).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 3);
}
