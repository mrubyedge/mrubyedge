extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn assign_test() {
    let code = "
    def test_main
      ary = [10, 20, 30]
      a, b, c = *ary
      a + b + c
    end
    ";
    let binary = mrbc_compile("assign1", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 60);
}

#[test]
fn assign_post_test() {
    let code = "
    def test_main
      ary = [10, 20, 30, 40]
      a, b, *rest = *ary
      ans = a + b
      p rest
      rest.each do |v|
        ans -= v
      end
      ans
    end
    ";
    let binary = mrbc_compile_debug("assign2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, -40);
}
