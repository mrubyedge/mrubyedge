extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn range_inclusive_each_test() {
    let code = r#"
    def test_range_each
      sum = 0
      (0..10).each do |i|
        sum += i
      end
      sum
    end
    "#;
    let binary = mrbc_compile("range_inclusive_each", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_range_each", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 55);
}

#[test]
fn range_exclusive_each_test() {
    let code = r#"
    def test_range_each
      sum = 0
      (0...10).each do |i|
        sum += i
      end
      sum
    end
    "#;
    let binary = mrbc_compile("range_exclusive_each", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_range_each", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 45);
}

#[test]
fn range_map_test() {
    let code = r#"
    def test_range_map
      (1..3).map do |i|
        i * 2
      end
    end
    "#;
    let binary = mrbc_compile("range_map", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_range_map", &args).unwrap();
    let result: (i32, i32, i32) = result.as_ref().try_into().unwrap();
    assert_eq!(result, (2, 4, 6));
}
