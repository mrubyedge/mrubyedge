extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use std::rc::Rc;

use helpers::*;
use mrubyedge::yamrb::{prelude::array::mrb_array_get_index, value::RObject};

#[test]
fn pack_unpack_test() {
    let code = "
def pack_unpack
  data = [100, 150, 200, 250].pack('C C C C')
  result = data.unpack('C C C C')
  result
end";
    let binary = mrbc_compile("pack_unpack", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "pack_unpack", &args).unwrap();
    for (i, expected) in [100, 150, 200, 250].iter().enumerate() {
        let args = vec![Rc::new(RObject::integer(i as i64))];
        let value = mrb_array_get_index(result.clone(), &args).expect("getting index failed");
        let value: i64 = value.as_ref().try_into().expect("value is not integer");
        assert_eq!(value, *expected);
    }
}

#[test]
fn unpack_test() {
    let code = "
def sum_unpack
  data = \"\\x01\\x02\\x03\\x04\"
  result = data.unpack('c c c c')
  result[0] + result[1] + result[2] + result[3]
end";
    let binary = mrbc_compile("shared_memory", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "sum_unpack", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 10);
}
