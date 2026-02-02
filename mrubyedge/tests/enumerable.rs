extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use helpers::*;

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
