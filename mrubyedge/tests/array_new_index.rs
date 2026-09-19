extern crate mrubyedge;

mod helpers;

use helpers::*;

#[test]
fn array_index_out_of_range_returns_nil_test() {
    let code = r#"
    def test_array_index_oob
      a = [1, 2, 3]
      [a[5], a[-5], a[-1]]
    end
    "#;
    let binary = mrbc_compile("array_index_oob", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_index_oob", &args).unwrap();
    let elems: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    assert!(elems[0].as_ref().is_nil());
    assert!(elems[1].as_ref().is_nil());
    let last: i64 = elems[2].as_ref().try_into().unwrap();
    assert_eq!(last, 3);
}

#[test]
fn array_index_missing_argument_errors_test() {
    let code = r#"
    def test_array_index_missing
      [1, 2, 3][]
    end
    "#;
    let binary = mrbc_compile("array_index_missing", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_index_missing", &args);
    assert!(result.is_err());
}

#[test]
fn array_new_with_block_test() {
    let code = r#"
    def test_array_new_block
      Array.new(3) { |i| i * 2 }
    end
    "#;
    let binary = mrbc_compile("array_new_block", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_new_block", &args).unwrap();
    let elems: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = elems
        .iter()
        .map(|e| e.as_ref().try_into().unwrap())
        .collect();
    assert_eq!(vals, vec![0, 2, 4]);
}

#[test]
fn array_new_with_default_value_test() {
    let code = r#"
    def test_array_new_default
      Array.new(2, 7)
    end
    "#;
    let binary = mrbc_compile("array_new_default", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_new_default", &args).unwrap();
    let elems: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = elems
        .iter()
        .map(|e| e.as_ref().try_into().unwrap())
        .collect();
    assert_eq!(vals, vec![7, 7]);
}
