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

#[test]
fn hash_flatten_test() {
    let code = r#"
    def test_hash_flatten
      h = {"a" => 1, "b" => 2}
      h.flatten
    end
    "#;
    let binary = mrbc_compile("hash_flatten", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_flatten", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();

    // Hash#flatten returns an array of [key1, value1, key2, value2, ...]
    assert_eq!(arr.len(), 4);

    // Verify the array contains the keys and values
    let strings: Vec<String> = arr
        .iter()
        .filter_map(|obj| {
            let s: Result<String, _> = obj.as_ref().try_into();
            s.ok()
        })
        .collect();
    let ints: Vec<i64> = arr
        .iter()
        .filter_map(|obj| {
            let i: Result<i64, _> = obj.as_ref().try_into();
            i.ok()
        })
        .collect();

    assert!(strings.contains(&"a".to_string()));
    assert!(strings.contains(&"b".to_string()));
    assert!(ints.contains(&1));
    assert!(ints.contains(&2));
}

#[test]
fn hash_splat_literal_test() {
    let code = r#"
    def test_hash_splat_literal
      h = { a: 1 }
      { **h }.size
    end
    "#;
    let binary = mrbc_compile("hash_splat_literal", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_splat_literal", &args).unwrap();
    let result: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn hash_splat_literal_with_extra_test() {
    let code = r#"
    def test_hash_splat_with_extra
      h = { a: 1 }
      { **h, b: 2 }.size
    end
    "#;
    let binary = mrbc_compile("hash_splat_with_extra", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_splat_with_extra", &args).unwrap();
    let result: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn hash_literal_over_stack_flush_test() {
    let code = r#"
    def test_big_hash_literal
      { k1: 1, k2: 2, k3: 3, k4: 4, k5: 5, k6: 6, k7: 7, k8: 8, k9: 9, k10: 10, k11: 11, k12: 12, k13: 13, k14: 14, k15: 15, k16: 16, k17: 17, k18: 18, k19: 19, k20: 20, k21: 21, k22: 22, k23: 23, k24: 24, k25: 25, k26: 26, k27: 27, k28: 28, k29: 29, k30: 30, k31: 31, k32: 32, k33: 33, k34: 34, k35: 35, k36: 36, k37: 37, k38: 38, k39: 39, k40: 40, k41: 41, k42: 42, k43: 43, k44: 44, k45: 45, k46: 46, k47: 47, k48: 48, k49: 49, k50: 50, k51: 51, k52: 52, k53: 53, k54: 54, k55: 55, k56: 56, k57: 57, k58: 58, k59: 59, k60: 60 }.size
    end
    "#;
    let binary = mrbc_compile("big_hash_literal", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_big_hash_literal", &args).unwrap();
    let result: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 60);
}

#[test]
fn hash_splat_literal_overrides_test() {
    let code = r#"
    def test_hash_splat_overrides
      h = { a: 1, b: 2 }
      [{ **h, a: 9 }[:a], { **h, a: 9 }[:b]]
    end
    "#;
    let binary = mrbc_compile("hash_splat_overrides", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_hash_splat_overrides", &args).unwrap();
    let outer: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let a: i32 = outer[0].as_ref().try_into().unwrap();
    let b: i32 = outer[1].as_ref().try_into().unwrap();
    assert_eq!((a, b), (9, 2));
}
