#![allow(clippy::approx_constant)]
extern crate mrubyedge;
extern crate mrubyedge_serde_json;

mod helpers;
use helpers::*;

#[test]
fn test_json_load_integer() {
    let code = r#"
    JSON.load("42")
    "#;
    let binary = mrbc_compile("json_load_integer", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let value: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_json_load_string() {
    let code = r#"
    JSON.load('"hello"')
    "#;
    let binary = mrbc_compile("json_load_string", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let value: String = result.as_ref().try_into().unwrap();
    assert_eq!(value, "hello");
}

#[test]
fn test_json_load_array() {
    let code = r#"
    result = JSON.load('[
      1,
      2,
      3
    ]')
    result
    "#;
    let binary = mrbc_compile("json_load_array", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    if let mrubyedge::yamrb::value::RValue::Array(arr) = &result.value {
        assert_eq!(arr.borrow().len(), 3);
        let v0: i64 = arr.borrow()[0].as_ref().try_into().unwrap();
        let v1: i64 = arr.borrow()[1].as_ref().try_into().unwrap();
        let v2: i64 = arr.borrow()[2].as_ref().try_into().unwrap();
        assert_eq!(v0, 1);
        assert_eq!(v1, 2);
        assert_eq!(v2, 3);
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_json_load_hash() {
    let code = r#"
    result = JSON.load('{
      "name": "Alice",
      "age": 30
    }')
    result["name"]
    "#;
    let binary = mrbc_compile("json_load_hash_name", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);
    let result = vm.run().unwrap();
    let name: String = result.as_ref().try_into().unwrap();
    assert_eq!(name, "Alice");

    let code2 = r#"
    result = JSON.load('{
      "name": "Alice",
      "age": 30
    }')
    result["age"]
    "#;
    let binary2 = mrbc_compile("json_load_hash_age", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    let mut vm2 = mrubyedge::yamrb::vm::VM::open(&mut rite2);
    mrubyedge_serde_json::init_json(&mut vm2);
    let result2 = vm2.run().unwrap();
    let age: i64 = result2.as_ref().try_into().unwrap();
    assert_eq!(age, 30);
}

#[test]
fn test_json_load_nested_structure() {
    let code = r#"
    result = JSON.load('{
      "users": [
        {
          "name": "Bob",
          "age": 25
        },
        {
          "name": "Carol",
          "age": 28
        }
      ]
    }')
    result["users"][0]["name"]
    "#;
    let binary = mrbc_compile("json_load_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let name: String = result.as_ref().try_into().unwrap();
    assert_eq!(name, "Bob");

    let code2 = r#"
    result = JSON.load('{
      "users": [
        {
          "name": "Bob",
          "age": 25
        },
        {
          "name": "Carol",
          "age": 28
        }
      ]
    }')
    result["users"][1]["name"]
    "#;
    let binary2 = mrbc_compile("json_load_nested2", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    let mut vm2 = mrubyedge::yamrb::vm::VM::open(&mut rite2);
    mrubyedge_serde_json::init_json(&mut vm2);
    let result2 = vm2.run().unwrap();
    let name2: String = result2.as_ref().try_into().unwrap();
    assert_eq!(name2, "Carol");
}

#[test]
fn test_json_load_boolean() {
    let code = r#"
    JSON.load("true")
    "#;
    let binary = mrbc_compile("json_load_bool", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let value: bool = result.as_ref().try_into().unwrap();
    assert_eq!(value, true);
}

#[test]
fn test_json_load_boolean_2() {
    let code = r#"
    JSON.load("false")
    "#;
    let binary = mrbc_compile("json_load_bool_2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let value: bool = result.as_ref().try_into().unwrap();
    assert_eq!(value, false);
}

#[test]
fn test_json_load_nil() {
    let code = r#"
    JSON.load("null")
    "#;
    let binary = mrbc_compile("json_load_nil", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    assert!(result.is_nil());
}

#[test]
fn test_json_load_float() {
    let code = r#"
    JSON.load("3.1415")
    "#;
    let binary = mrbc_compile("json_load_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let value: f64 = result.as_ref().try_into().unwrap();
    assert_eq!(value, 3.1415);
}

#[test]
fn test_json_load_key() {
    let code = r#"
    result = JSON.load('{
      "status": "ok"
    }')
    result["status"]
    "#;
    let binary = mrbc_compile("json_load_key", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let value: String = result.as_ref().try_into().unwrap();
    assert_eq!(value, "ok");
}
