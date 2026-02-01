extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use helpers::*;

#[test]
fn array_add_test() {
    let code = r#"
    def test_array_add
      [1, 2] + [3, 4]
    end
    "#;
    let binary = mrbc_compile("array_add", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_add", &args).unwrap();
    let result = result.as_vec_owned().unwrap();
    assert_eq!(result.len(), 4);
}

#[test]
fn array_push_test() {
    let code = r#"
    def test_array_push
      a = [1, 2]
      a << 3
      a
    end
    "#;
    let binary = mrbc_compile("array_push", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_push", &args).unwrap();
    let result = result.as_vec_owned().unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn array_at_test() {
    let code = r#"
    def test_array_at
      [1, 2, 3].at(1)
    end
    "#;
    let binary = mrbc_compile("array_at", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_at", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn array_clear_test() {
    let code = r#"
    def test_array_clear
      a = [1, 2, 3]
      a.clear
      a.size
    end
    "#;
    let binary = mrbc_compile("array_clear", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_clear", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 0);
}

#[test]
fn array_delete_at_test() {
    let code = r#"
    def test_array_delete_at
      a = [1, 2, 3]
      a.delete_at(1)
    end
    "#;
    let binary = mrbc_compile("array_delete_at", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_delete_at", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn array_empty_test() {
    let code = r#"
    def test_array_empty
      [].empty?
    end
    "#;
    let binary = mrbc_compile("array_empty", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_empty", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn array_include_test() {
    let code = r#"
    def test_array_include
      [1, 2, 3].include?(2)
    end
    "#;
    let binary = mrbc_compile("array_include", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_include", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn array_and_test() {
    let code = r#"
    def test_array_and
      ([1, 2, 3] & [2, 3, 4]).size
    end
    "#;
    let binary = mrbc_compile("array_and", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_and", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn array_or_test() {
    let code = r#"
    def test_array_or
      ([1, 2] | [2, 3]).size
    end
    "#;
    let binary = mrbc_compile("array_or", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_or", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_first_test() {
    let code = r#"
    def test_array_first
      [1, 2, 3].first
    end
    "#;
    let binary = mrbc_compile("array_first", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_first", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_last_test() {
    let code = r#"
    def test_array_last
      [1, 2, 3].last
    end
    "#;
    let binary = mrbc_compile("array_last", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_last", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_pop_test() {
    let code = r#"
    def test_array_pop
      a = [1, 2, 3]
      a.pop
    end
    "#;
    let binary = mrbc_compile("array_pop", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_pop", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_shift_test() {
    let code = r#"
    def test_array_shift
      a = [1, 2, 3]
      a.shift
    end
    "#;
    let binary = mrbc_compile("array_shift", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_shift", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_unshift_test() {
    let code = r#"
    def test_array_unshift
      a = [2, 3]
      a.unshift(1)
      a.first
    end
    "#;
    let binary = mrbc_compile("array_unshift", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_unshift", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_dup_test() {
    let code = r#"
    def test_array_dup
      a = [1, 2, 3]
      b = a.dup
      b == a
    end
    "#;
    let binary = mrbc_compile("array_dup", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_dup", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn array_min_test() {
    let code = r#"
    def test_array_min
      [3, 1, 2].min
    end
    "#;
    let binary = mrbc_compile("array_min", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_min", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_max_test() {
    let code = r#"
    def test_array_max
      [3, 1, 2].max
    end
    "#;
    let binary = mrbc_compile("array_max", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_max", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_minmax_test() {
    let code = r#"
    def test_array_minmax
      [3, 1, 2].minmax
    end
    "#;
    let binary = mrbc_compile("array_minmax", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_minmax", &args).unwrap();
    let result = result.as_vec_owned().unwrap();
    assert_eq!(result.len(), 2);
    let min: i64 = result[0].as_ref().try_into().unwrap();
    let max: i64 = result[1].as_ref().try_into().unwrap();
    assert_eq!(min, 1);
    assert_eq!(max, 3);
}

#[test]
fn array_uniq_test() {
    let code = r#"
    def test_array_uniq
      [1, 2, 2, 3].uniq
    end
    "#;
    let binary = mrbc_compile("array_uniq", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_uniq", &args).unwrap();
    let result = result.as_vec_owned().unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn array_uniq_self_test() {
    let code = r#"
    def test_array_uniq_self
      a = [1, 2, 2, 3]
      a.uniq!
      a.size
    end
    "#;
    let binary = mrbc_compile("array_uniq_self", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_uniq_self", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_join_test() {
    let code = r#"
    def test_array_join
      [1, 2, 3].join(",")
    end
    "#;
    let binary = mrbc_compile("array_join", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_join", &args).unwrap();
    let result: String = result.as_ref().try_into().unwrap();
    assert_eq!(result, "1,2,3");
}
