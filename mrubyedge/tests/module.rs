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

#[test]
fn nested_define_module_preserves_existing_methods() {
    use mrubyedge::yamrb::helpers::mrb_define_module_cmethod;
    use mrubyedge::yamrb::value::RObject;
    use std::rc::Rc;

    let mut vm = mrubyedge::yamrb::vm::VM::empty();

    // First: define Outer module
    let outer = vm.define_module("Outer", None);

    // Define a cmethod on Outer
    mrb_define_module_cmethod(
        &mut vm,
        outer.clone(),
        "foo",
        Box::new(|_vm, _args| Ok(RObject::integer(42).to_refcount_assigned())),
    );

    // Define Inner nested under Outer
    let inner = vm.define_module("Inner", Some(outer.clone()));

    mrb_define_module_cmethod(
        &mut vm,
        inner.clone(),
        "bar",
        Box::new(|_vm, _args| Ok(RObject::integer(99).to_refcount_assigned())),
    );

    // Re-open Outer via define_module — should return the same module
    let outer2 = vm.define_module("Outer", None);
    assert!(
        Rc::ptr_eq(&outer, &outer2),
        "define_module should return the existing module"
    );

    // foo should still be defined on re-opened Outer
    assert!(
        outer2.procs.borrow().contains_key("foo"),
        "existing cmethod 'foo' should be preserved after re-opening"
    );

    // Re-open Inner nested under Outer — should return the same module
    let inner2 = vm.define_module("Inner", Some(outer.clone()));
    assert!(
        Rc::ptr_eq(&inner, &inner2),
        "nested define_module should return the existing module"
    );

    // bar should still be defined on re-opened Inner
    assert!(
        inner2.procs.borrow().contains_key("bar"),
        "existing cmethod 'bar' should be preserved after re-opening nested module"
    );
}

#[test]
fn include_nested_module_and_call_method() {
    use mrubyedge::yamrb::helpers::mrb_define_module_cmethod;
    use mrubyedge::yamrb::value::RObject;

    let mut vm = mrubyedge::yamrb::vm::VM::empty();
    let outer = vm.define_module("Outer", None);
    mrb_define_module_cmethod(
        &mut vm,
        outer.clone(),
        "foo",
        Box::new(|_vm, _args| Ok(RObject::integer(42).to_refcount_assigned())),
    );
    let inner = vm.define_module("Inner", Some(outer.clone()));
    mrb_define_module_cmethod(
        &mut vm,
        inner.clone(),
        "greet",
        Box::new(|_vm, _args| {
            Ok(RObject::string("hello from Inner".to_string()).to_refcount_assigned())
        }),
    );

    let script = r#"
class MyClass
  include Outer::Inner
end

MyClass.new.greet
"#;
    let binary = mrbc_compile("include_nested_module", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let result = vm.eval_rite(&mut rite).unwrap();

    let value: String = result
        .as_ref()
        .try_into()
        .expect("greet should return string");
    assert_eq!(value, "hello from Inner");
}
