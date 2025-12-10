extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn module_definition_test() {
    let script = r#"
module TestModule
  def module_method
    42
  end
end

TestModule
"#;

    let binary = mrbc_compile("module_def", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    // Result should be the module itself
    assert!(matches!(result.tt, mrubyedge::yamrb::value::RType::Module));
}

#[test]
fn class_can_include_module_method() {
    let script = r#"
module Printable
  def greet
    "hello"
  end
end

class User
  include Printable
end

User.new.greet
"#;

    let binary = mrbc_compile("module_include", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let value: String = result
        .as_ref()
        .try_into()
        .expect("greet should return string");
    assert_eq!(value, "hello");
}

#[test]
fn modules_can_be_used_as_namespace() {
    let script = r#"
module Outer
  module Printable
    def greet
      "hello"
    end
  end

  class User
    include Printable
  end
end

Outer::User.new.greet
"#;

    let binary = mrbc_compile("module_include_ns", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let value: String = result
        .as_ref()
        .try_into()
        .expect("greet should return string");
    assert_eq!(value, "hello");
}

#[test]
fn module_can_include_module_method() {
    let script = r#"
module Core
  def core_value
    123
  end
end

module Superset
  include Core
end

class Wrapper
  include Superset
end

Wrapper.new.core_value
"#;

    let binary = mrbc_compile("module_include_module", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let value: i64 = result
        .as_ref()
        .try_into()
        .expect("core_value should return integer");
    assert_eq!(value, 123);
}

#[test]
fn module_can_include_module_method_2() {
    let script = r#"
module Core
  def core_value
    123
  end
end

module Superset
  include Core

  def core_value
    super + 1
  end
end

class Wrapper
  include Superset
end

Wrapper.new.core_value
"#;

    let binary = mrbc_compile("module_include_module_2", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let value: i64 = result
        .as_ref()
        .try_into()
        .expect("core_value should return integer");
    assert_eq!(value, 124);
}
