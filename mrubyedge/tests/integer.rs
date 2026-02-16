extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn integer_bitref_test() {
    let code = "
n = 0b1010
n[0] + n[1] + n[2] + n[3]
    ";
    let binary = mrbc_compile("bitref", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 2); // 0 + 1 + 0 + 1 = 2
}

#[test]
fn integer_negative_test() {
    let code = "
a = 42
-a
    ";
    let binary = mrbc_compile("negative", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, -42);
}

#[test]
fn integer_power_test() {
    let code = "
2 ** 10
    ";
    let binary = mrbc_compile("power", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 1024);
}

#[test]
fn integer_power_float_test() {
    let code = "
4 ** 0.5
    ";
    let binary = mrbc_compile("power_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 2.0);
}

#[test]
fn integer_mod_test() {
    let code = "
17 % 5
    ";
    let binary = mrbc_compile("mod", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 2);
}

#[test]
fn integer_and_test() {
    let code = "
0b1100 & 0b1010
    ";
    let binary = mrbc_compile("and", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 0b1000);
}

#[test]
fn integer_or_test() {
    let code = "
0b1100 | 0b1010
    ";
    let binary = mrbc_compile("or", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 0b1110);
}

#[test]
fn integer_xor_test() {
    let code = "
0b1100 ^ 0b1010
    ";
    let binary = mrbc_compile("xor", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 0b0110);
}

#[test]
fn integer_not_test() {
    let code = "
~5
    ";
    let binary = mrbc_compile("not", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, -6);
}

#[test]
fn integer_lshift_test() {
    let code = "
5 << 2
    ";
    let binary = mrbc_compile("lshift", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 20);
}

#[test]
fn integer_rshift_test() {
    let code = "
20 >> 2
    ";
    let binary = mrbc_compile("rshift", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 5);
}

#[test]
fn integer_abs_test() {
    let code = "
result1 = (-42).abs
result2 = 42.abs
result1 + result2
    ";
    let binary = mrbc_compile("abs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 84);
}

#[test]
fn integer_to_i_test() {
    let code = "
42.to_i
    ";
    let binary = mrbc_compile("to_i", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 42);
}

#[test]
fn integer_to_f_test() {
    let code = "
42.to_f
    ";
    let binary = mrbc_compile("to_f", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 42.0);
}

#[test]
fn integer_chr_test() {
    let code = "
65.chr
    ";
    let binary = mrbc_compile("chr", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "A");
}

#[test]
fn integer_to_s_test() {
    let code = "
123.to_s
    ";
    let binary = mrbc_compile("to_s", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "123");
}

#[test]
fn integer_inspect_test() {
    let code = "
456.inspect
    ";
    let binary = mrbc_compile("inspect", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "456");
}

#[test]
fn integer_clamp_test() {
    let code = "
100.clamp(50, 150) + 25.clamp(50, 150) + 200.clamp(50, 150)
    ";
    let binary = mrbc_compile("integer_clamp", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 300); // 100 + 50 + 150
}

#[test]
fn integer_add_method_test() {
    let code = r#"
    def test_add
      a = 5
      b = 3
      a.+(b)
    end
    "#;
    let binary = mrbc_compile("integer_add_method", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_add", &args).unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 8);
}

#[test]
fn integer_sub_method_test() {
    let code = r#"
    def test_sub
      a = 10
      b = 3
      a.-(b)
    end
    "#;
    let binary = mrbc_compile("integer_sub_method", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_sub", &args).unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 7);
}

#[test]
fn integer_add_float_test() {
    let code = r#"
    def test_add_float
      a = 5
      f = 2.5
      a.+(f)
    end
    "#;
    let binary = mrbc_compile("integer_add_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_add_float", &args).unwrap();
    let result_float: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(result_float, 7.5);
}
