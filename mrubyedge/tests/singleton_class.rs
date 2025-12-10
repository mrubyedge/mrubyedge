extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn test_singleton_class() {
    let code = "
    obj = Object.new
    def obj.my_singleton_method
      123
    end

    obj.my_singleton_method
    ";
    let binary = mrbc_compile("singleton_class", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 123);
}
