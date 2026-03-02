extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use std::rc::Rc;

use helpers::*;
use mrubyedge::yamrb::value::RObject;

#[test]
fn symbol_to_proc_direct() {
    let code = r#"
    def test_to_proc_direct
      sym = :upcase
      proc = sym.to_proc
      proc.call("hello")
    end
    "#;
    let binary = mrbc_compile("to_proc_direct", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let result = mrb_funcall(&mut vm, None, "test_to_proc_direct", &[]).unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "HELLO");
}

#[test]
fn symbol_to_proc_map_to_s() {
    let code = r#"
    def test_to_proc_map_to_s
      [1, 2, 3].map(&:to_s)
    end
    "#;
    let binary = mrbc_compile_debug("to_proc_map_to_s", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let result = mrb_funcall(&mut vm, None, "test_to_proc_map_to_s", &[]).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    let r0: String = result_array[0].as_ref().try_into().unwrap();
    let r1: String = result_array[1].as_ref().try_into().unwrap();
    let r2: String = result_array[2].as_ref().try_into().unwrap();
    assert_eq!(r0, "1");
    assert_eq!(r1, "2");
    assert_eq!(r2, "3");
}

#[test]
fn symbol_to_proc_select() {
    let code = r#"
    def test_to_proc_select
      [nil, 1, nil, 2, 3].select(&:nil?)
    end
    "#;
    let binary = mrbc_compile("to_proc_select", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let result = mrb_funcall(&mut vm, None, "test_to_proc_select", &[]).unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(result_array.len(), 2);
    assert!(result_array[0].is_nil());
    assert!(result_array[1].is_nil());
}
