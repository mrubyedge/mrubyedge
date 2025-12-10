extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use helpers::*;
// use mrubyedge::yamrb::value::RObject;
// use std::rc::Rc;

#[test]
fn string_new_test() {
    let code = "
    def test_string_new
      string = String.new
      string.size
    end
    ";
    let binary = mrbc_compile("string_new", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_string_new", &args).unwrap();
    let result: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 0);
}
