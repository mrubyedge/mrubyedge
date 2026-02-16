#![cfg(feature = "insn-limit")]
extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn insn_limit_basic_test() {
    let code = r#"
    def test_simple
      a = 1
      b = 2
      a + b
    end
    "#;
    let binary = mrbc_compile("insn_limit_basic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);

    // Simple function should complete within limit
    let result = vm.run();
    assert!(result.is_ok());

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_simple", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 3);
}

#[test]
fn insn_limit_exceeded_test() {
    let code = r#"
    def test_infinite_loop
      i = 0
      loop do
        i += 1
      end
    end
    "#;
    let binary = mrbc_compile("insn_limit_exceeded", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_infinite_loop", &args);

    // Should fail due to instruction limit
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains("instruction limit exceeded"));
}

#[test]
fn insn_limit_reset_test() {
    let code = r#"
    def test_count
      sum = 0
      10.times do |i|
        sum += i
      end
      sum
    end
    "#;
    let binary = mrbc_compile("insn_limit_reset", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // First call
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_count", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 45);

    let count_before_reset = vm.get_insn_count();
    assert!(count_before_reset > 0);

    vm.reset_insn_count();
    assert_eq!(vm.get_insn_count(), 0);

    // Second call should work after reset
    let result: i32 = mrb_funcall(&mut vm, None, "test_count", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 45);
}

#[test]
fn insn_limit_while_loop_test() {
    let code = r#"
    def test_while
      i = 0
      sum = 0
      while i < 100000
        i += 1
        sum += i
      end
      sum
    end
    "#;
    let binary = mrbc_compile("insn_limit_while", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_while", &args);

    assert!(result.is_err());
    assert!(format!("{:?}", result.unwrap_err()).contains("instruction limit exceeded"));
}

#[test]
fn insn_limit_counter_increments_test() {
    let code = r#"
    def test_increment
      a = 1
      b = 2
      c = 3
      a + b + c
    end
    "#;
    let binary = mrbc_compile("insn_limit_increment", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);

    let initial_count = vm.get_insn_count();
    assert_eq!(initial_count, 0);

    vm.run().unwrap();

    let count_after_run = vm.get_insn_count();
    assert!(count_after_run > 0);

    let args = vec![];
    mrb_funcall(&mut vm, None, "test_increment", &args).unwrap();

    let count_after_call = vm.get_insn_count();
    assert!(count_after_call > count_after_run);
}
