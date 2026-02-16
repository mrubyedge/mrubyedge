extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn while_basic_test() {
    let code = r#"
    def test_while
      a = 0
      while a < 5
        a += 1
      end
      a
    end
    "#;
    let binary = mrbc_compile("while_basic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_while", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5);
}

#[test]
fn while_with_break_test() {
    let code = r#"
    def test_while_break
      a = 0
      while true
        a += 1
        break if a > 10
      end
      a
    end
    "#;
    let binary = mrbc_compile("while_break", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_while_break", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 11);
}

#[test]
fn while_with_ensure_test() {
    let code = r#"
    def test_while_ensure
      a = 0
      $ensure_count = 0
      while true
        begin
          a += 1
          break if a > 10
        ensure
          $ensure_count += 1
        end
      end
      a
    end
    "#;
    let binary = mrbc_compile("while_ensure", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_while_ensure", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 11);

    // Verify that ensure is executed every iteration
    let ensure_count: i32 = vm
        .globals
        .get("$ensure_count")
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(ensure_count, 11);
}

#[test]
fn while_nested_test() {
    let code = r#"
    def test_while_nested
      sum = 0
      i = 0
      while i < 3
        j = 0
        while j < 3
          sum += 1
          j += 1
        end
        i += 1
      end
      sum
    end
    "#;
    let binary = mrbc_compile("while_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_while_nested", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 9); // 3 * 3 = 9
}

#[test]
fn while_accumulate_test() {
    let code = r#"
    def test_while_accumulate
      a = 0
      sum = 0
      while a < 10
        a += 1
        sum += a
      end
      sum
    end
    "#;
    let binary = mrbc_compile("while_accumulate", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_while_accumulate", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    // 1 + 2 + 3 + ... + 10 = 55
    assert_eq!(result, 55);
}

#[test]
fn while_with_ensure_and_exception_test() {
    let code = r#"
    def test_while_ensure_exception
      a = 0
      $ensure_executed = false
      begin
        while a < 5
          begin
            a += 1
            raise "error" if a == 3
          ensure
            $ensure_executed = true
          end
        end
      rescue
        # catch the exception
      end
      a
    end
    "#;
    let binary = mrbc_compile("while_ensure_exception", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_while_ensure_exception", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 3);

    // Verify that ensure was executed
    let ensure_executed: bool = vm
        .globals
        .get("$ensure_executed")
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert!(ensure_executed);
}
