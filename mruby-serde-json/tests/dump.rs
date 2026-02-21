extern crate mrubyedge;
extern crate mrubyedge_serde_json;

mod helpers;
use helpers::*;

#[test]
fn test_json_dump_integer() {
    let code = "
    JSON.dump(42)
    ";
    let binary = mrbc_compile("json_dump_integer", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(json_str, "42");
}

#[test]
fn test_json_dump_string() {
    let code = r#"
    JSON.dump("hello")
    "#;
    let binary = mrbc_compile("json_dump_string", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(json_str, r#""hello""#);
}

#[test]
fn test_json_dump_array() {
    let code = "
    JSON.dump([1, 2, 3])
    ";
    let binary = mrbc_compile("json_dump_array", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    println!("json_str: {}", json_str);
    assert_eq!(json_str, "[1,2,3]");
}

#[test]
fn test_json_dump_hash() {
    let code = r#"
    JSON.dump({
      "name" => "Alice",
      "age" => 30
    })
    "#;
    let binary = mrbc_compile("json_dump_hash", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    println!("json_str: {}", json_str);
    assert!(json_str.contains(r#""name":"Alice""#));
    assert!(json_str.contains(r#""age":30"#));
}

#[test]
fn test_json_dump_nested_structure() {
    let code = r#"
    JSON.dump({
      "users" => [{
        "name" => "Bob",
        "age" => 25
      }, {
        "name" => "Carol",
        "age" => 28
      }]
    })
    "#;
    let binary = mrbc_compile("json_dump_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    println!("json_str: {}", json_str);
    assert!(json_str.contains(r#""users""#));
    assert!(json_str.contains(r#""name":"Bob""#));
    assert!(json_str.contains(r#""age":25"#));
    assert!(json_str.contains(r#""name":"Carol""#));
    assert!(json_str.contains(r#""age":28"#));
}

#[test]
fn test_json_dump_boolean() {
    let code = "
    JSON.dump(true)
    ";
    let binary = mrbc_compile("json_dump_bool", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(json_str, "true");
}

#[test]
fn test_json_dump_nil() {
    let code = "
    JSON.dump(nil)
    ";
    let binary = mrbc_compile("json_dump_nil", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(json_str, "null");
}

#[test]
fn test_json_dump_float() {
    let code = "
    JSON.dump(3.14)
    ";
    let binary = mrbc_compile("json_dump_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(json_str, "3.14");
}

#[test]
fn test_json_dump_symbol_key() {
    let code = "
    JSON.dump({
      status: :ok
    })
    ";
    let binary = mrbc_compile("json_dump_symbol", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    println!("json_str: {}", json_str);
    assert!(json_str.contains(r#""status":"ok""#));
}

#[test]
fn test_json_dump_to_json() {
    let code = r#"
    class User
      def initialize(name, age)
        @name = name
        @age = age
      end

      def to_json(*_args)
        {
          name: @name,
          age: @age
        }
      end
    end
    JSON.dump(User.new("Dave", 40))
    "#;
    let binary = mrbc_compile("json_dump_to_json", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);

    let result = vm.run().unwrap();
    let json_str: String = result.as_ref().try_into().unwrap();
    println!("json_str: {}", json_str);
    assert!(json_str.contains(r#""name":"Dave""#));
    assert!(json_str.contains(r#""age":40"#));
}

// FIXME: panic!s when to_json is not defined in user-defined class
