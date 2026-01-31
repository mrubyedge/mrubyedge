extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn basic_splat_args_test() {
    let code = "
def sum(*args)
  total = 0
  args.each do |arg|
    total = total + arg
  end
  total
end

sum(1, 2, 3, 4, 5)
    ";
    let binary = mrbc_compile("basic_splat", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 15);
}

#[test]
fn splat_args_with_regular_args_test() {
    let code = "
def splat_it(x, y, *args)
  total = x + y
  args.each do |arg|
    total = total + arg
  end
  total
end

splat_it(10, 20, 30, 40, 50)
    ";
    let binary = mrbc_compile("splat_with_regular", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 150);
}

#[test]
fn splat_args_empty_test() {
    let code = "
def splat_it(x, y, *args)
  args.size
end

splat_it(10, 20)
    ";
    let binary = mrbc_compile("splat_empty", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 0);
}
