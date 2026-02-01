extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use helpers::*;
use mrubyedge::yamrb::value::RObject;
use std::rc::Rc;

#[test]
fn hash_new_test() {
    let code = "
    def test_hash_new
      hash = Hash.new
      hash.size
    end
    ";
    let binary = mrbc_compile("hash_new", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_new", &args).unwrap();
    let result: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 0);
}

#[test]
fn hash_test() {
    let code = "
    def test_hash
      foo = {}
      foo[\"bar\"] = 42
      foo[\"bar\"]
    end
    ";
    let binary = mrbc_compile("hash", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_hash", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 42);
}

#[test]
fn hash_2_test() {
    let code = "
  $hash = {}

  def test_hash_set(key, value)
    $hash[key] = value
  end

  def test_hash_get(key)
    $hash[key]
  end
  ";
    let binary = mrbc_compile("hash_2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![
        Rc::new(RObject::symbol("bar".into())),
        Rc::new(RObject::integer(54)),
    ];
    let result: i32 = mrb_funcall(&mut vm, None, "test_hash_set", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 54);

    let args = vec![Rc::new(RObject::symbol("bar".into()))];
    let result: i32 = mrb_funcall(&mut vm, None, "test_hash_get", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 54);
}

#[test]
fn hash_each_test() {
    let code = "
    def test_hash_1
      hash = {
        \"foo\" => 1,
        \"bar\" => 2,
        \"baz\" => 3,
      }
      res = 0
      hash.each do |key, value|
        puts \"key: #{key}, value: #{value}\"
        res += value
      end
      res
    end
    ";
    let binary = mrbc_compile("hash_each", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let value = mrb_funcall(&mut vm, None, "test_hash_1", &args).unwrap();
    let value: i64 = value.as_ref().try_into().unwrap();
    assert_eq!(value, 6);
}

#[test]
fn hash_each_test_2() {
    let code = "
    def test_hash_1
      hash = {
        \"foo\" => 1,
        \"bar\" => 2,
        \"baz\" => 3,
      }
      res = \"\"
      hash.each do |key, value|
        res += key
      end
      res
    end
    ";
    let binary = mrbc_compile("hash_each_2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let value = mrb_funcall(&mut vm, None, "test_hash_1", &args).unwrap();
    let value: String = value.as_ref().try_into().unwrap();
    assert!(value.contains("foo"));
    assert!(value.contains("bar"));
    assert!(value.contains("baz"));
}

#[test]
fn hash_clear_test() {
    let code = r#"
    def test_hash_clear
      h = {"a" => 1, "b" => 2}
      h.clear
      h.size
    end
    "#;
    let binary = mrbc_compile("hash_clear", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_clear", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 0);
}

#[test]
fn hash_dup_test() {
    let code = r#"
    def test_hash_dup
      h = {"a" => 1}
      h2 = h.dup
      h2["a"]
    end
    "#;
    let binary = mrbc_compile("hash_dup", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_dup", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn hash_empty_test() {
    let code = r#"
    def test_hash_empty
      {}.empty?
    end
    "#;
    let binary = mrbc_compile("hash_empty", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_empty", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn hash_has_key_test() {
    let code = r#"
    def test_hash_has_key
      h = {"a" => 1, "b" => 2}
      h.has_key?("a")
    end
    "#;
    let binary = mrbc_compile("hash_has_key", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_has_key", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn hash_has_value_test() {
    let code = r#"
    def test_hash_has_value
      h = {"a" => 1, "b" => 2}
      h.has_value?(2)
    end
    "#;
    let binary = mrbc_compile("hash_has_value", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_has_value", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn hash_key_test() {
    let code = r#"
    def test_hash_key
      h = {"a" => 1, "b" => 2}
      h.key(2)
    end
    "#;
    let binary = mrbc_compile("hash_key", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_key", &args).unwrap();
    let result: String = result.as_ref().try_into().unwrap();
    assert_eq!(result, "b");
}

#[test]
fn hash_keys_test() {
    let code = r#"
    def test_hash_keys
      h = {"a" => 1, "b" => 2}
      h.keys.size
    end
    "#;
    let binary = mrbc_compile("hash_keys", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_keys", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn hash_values_test() {
    let code = r#"
    def test_hash_values
      h = {"a" => 1, "b" => 2}
      h.values.size
    end
    "#;
    let binary = mrbc_compile("hash_values", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_values", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn hash_merge_test() {
    let code = r#"
    def test_hash_merge
      h1 = {"a" => 1, "b" => 2}
      h2 = {"b" => 3, "c" => 4}
      h3 = h1.merge(h2)
      h3.size
    end
    "#;
    let binary = mrbc_compile("hash_merge", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_merge", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn hash_merge_self_test() {
    let code = r#"
    def test_hash_merge_self
      h1 = {"a" => 1, "b" => 2}
      h2 = {"b" => 3, "c" => 4}
      h1.merge!(h2)
      h1.size
    end
    "#;
    let binary = mrbc_compile("hash_merge_self", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_merge_self", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn hash_to_h_test() {
    let code = r#"
    def test_hash_to_h
      h = {"a" => 1}
      h2 = h.to_h
      h2.object_id == h.object_id
    end
    "#;
    let binary = mrbc_compile("hash_to_h", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_to_h", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}
