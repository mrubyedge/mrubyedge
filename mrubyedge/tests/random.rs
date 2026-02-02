extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn random_new_test() {
    let code = "
r = Random.new
r.class.inspect
    ";
    let binary = mrbc_compile("random_new", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "Random");
}

#[test]
fn random_new_with_seed_test() {
    let code = "
r = Random.new(12345)
r.seed
    ";
    let binary = mrbc_compile("random_new_seed", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 12345);
}

#[test]
fn random_rand_no_args_test() {
    let code = "
r = Random.new(42)
val = r.rand
puts val.to_s
val >= 0.0 && val < 1.0
    ";
    let binary = mrbc_compile("random_rand_no_args", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn random_rand_with_int_test() {
    let code = "
r = Random.new(100)
val = r.rand(10)
puts val.to_s
val >= 0 && val < 10
    ";
    let binary = mrbc_compile("random_rand_int", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn random_rand_with_float_test() {
    let code = "
r = Random.new(200)
val = r.rand(5.0)
puts val.to_s
val >= 0.0 && val < 5.0
    ";
    let binary = mrbc_compile("random_rand_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn random_class_rand_test() {
    let code = "
val = Random.rand
puts val.to_s
val >= 0.0 && val < 1.0
    ";
    let binary = mrbc_compile("random_class_rand", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn random_class_rand_with_arg_test() {
    let code = "
val = Random.rand(100)
puts val.to_s
val >= 0 && val < 100
    ";
    let binary = mrbc_compile("random_class_rand_arg", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn kernel_rand_test() {
    let code = "
val = rand
puts val.to_s
val >= 0.0 && val < 1.0
    ";
    let binary = mrbc_compile("kernel_rand", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn kernel_rand_with_arg_test() {
    let code = "
val = rand(50)
puts val.to_s
val >= 0 && val < 50
    ";
    let binary = mrbc_compile("kernel_rand_arg", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn random_srand_test() {
    let code = "
Random.srand(777)
old = Random.srand(888)
old
    ";
    let binary = mrbc_compile("random_srand", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 777);
}

#[test]
fn random_deterministic_test() {
    let code = "
r1 = Random.new(12345)
r2 = Random.new(12345)
a = r1.rand(100)
b = r2.rand(100)
puts \"a = #{a}, b = #{b}\"
a == b
    ";
    let binary = mrbc_compile("random_deterministic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}

#[test]
fn random_deterministic_test_2() {
    let code = "
r1 = Random.new(12345)
r2 = Random.new(123456)
a = r1.rand(100)
b = r2.rand(100)
puts \"a = #{a}, b = #{b}\"
a != b
    ";
    let binary = mrbc_compile("random_deterministic_2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    assert!(result.as_ref().is_truthy());
}
