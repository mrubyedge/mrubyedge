extern crate mrubyedge;

mod helpers;
use helpers::*;

use std::rc::Rc;

use mrubyedge::yamrb::helpers::{mrb_define_singleton_cmethod, mrb_funcall};
use mrubyedge::yamrb::value::{RObject, RValue};
use mrubyedge::yamrb::vm::VM;

/// Case 1: `def self.m` inside a module body must define a callable
/// singleton method on the module.
#[test]
fn def_self_in_module_defines_callable_singleton() {
    let code = r#"
module M1
  def self.answer
    42
  end
end

M1.answer
"#;
    let binary = mrbc_compile("singleton_mod_self", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: i64 = result.as_ref().try_into().expect("answer must be integer");
    assert_eq!(value, 42);
}

/// Case 2: explicit receiver variant `def M.m`.
#[test]
fn def_explicit_module_receiver_singleton() {
    let code = r#"
module M2
end

def M2.answer
  7
end

M2.answer
"#;
    let binary = mrbc_compile("singleton_mod_explicit", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: i64 = result.as_ref().try_into().expect("answer must be integer");
    assert_eq!(value, 7);
}

/// Case 3: `class << self` inside a module body.
#[test]
fn singleton_class_sugar_inside_module() {
    let code = r#"
module M3
  class << self
    def answer
      9
    end
  end
end

M3.answer
"#;
    let binary = mrbc_compile("singleton_mod_sclass", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: i64 = result.as_ref().try_into().expect("answer must be integer");
    assert_eq!(value, 9);
}

/// Case 4: native singleton registered first (as a host boot sequence does),
/// then a Ruby-level `def self.`; both must stay callable.
#[test]
fn native_singleton_then_ruby_singleton_coexist() {
    // Blob 1: define the module.
    let binary1 = mrbc_compile("smod_native_1", "module Q\nend\n");
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm = VM::open(&mut rite1);
    vm.run().unwrap();

    // Native registration on the module wrapper.
    let q_wrapper = vm.get_const_by_name("Q").expect("Q must be defined");
    mrb_define_singleton_cmethod(
        &mut vm,
        q_wrapper,
        "native_answer",
        Box::new(|_vm, _args| Ok(Rc::new(RObject::integer(5)))),
    );

    // Blob 2: Ruby-level singleton definition.
    let binary2 = mrbc_compile(
        "smod_native_2",
        r#"
module Q
  def self.ruby_answer
    native_answer * 2
  end
end
"#,
    );
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    vm.eval_rite(&mut rite2).unwrap();

    // Both reachable through the canonical wrapper.
    let q2 = vm.get_const_by_name("Q").expect("Q still defined");
    let n = mrb_funcall(&mut vm, Some(q2.clone()), "native_answer", &[]).unwrap();
    let n: i64 = n.as_ref().try_into().expect("native answer integer");
    assert_eq!(n, 5);

    let r = mrb_funcall(&mut vm, Some(q2), "ruby_answer", &[]).unwrap();
    let r: i64 = r.as_ref().try_into().expect("ruby answer integer");
    assert_eq!(r, 10);
}

/// Case 5: reopening the module keeps Ruby singletons and allows adding more.
#[test]
fn module_reopen_preserves_ruby_singletons_and_adds_more() {
    let binary1 = mrbc_compile(
        "smod_reopen_1",
        r#"
module R
  def self.alpha
    1
  end
end
"#,
    );
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm = VM::open(&mut rite1);
    vm.run().unwrap();

    let binary2 = mrbc_compile(
        "smod_reopen_2",
        r#"
module R
  def self.beta
    alpha + 1
  end
end
"#,
    );
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    vm.eval_rite(&mut rite2).unwrap();

    let r = vm.get_const_by_name("R").expect("R defined");
    let a = mrb_funcall(&mut vm, Some(r.clone()), "alpha", &[]).unwrap();
    let a: i64 = a.as_ref().try_into().expect("alpha integer");
    assert_eq!(a, 1);

    let b = mrb_funcall(&mut vm, Some(r), "beta", &[]).unwrap();
    let b: i64 = b.as_ref().try_into().expect("beta integer");
    assert_eq!(b, 2);
}

/// Case 6 (root disease): two wrappers over the same underlying RModule.
/// A singleton defined through one wrapper must be visible from the other,
/// like op_tclass building fresh wrappers at runtime.
#[test]
fn second_wrapper_shares_singleton_state() {
    let binary = mrbc_compile("smod_dual", "module W\nend\n");
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = VM::open(&mut rite);
    vm.run().unwrap();

    // Define a singleton via the canonical wrapper.
    let w1 = vm.get_const_by_name("W").expect("W defined");
    mrb_define_singleton_cmethod(
        &mut vm,
        w1,
        "tag",
        Box::new(|_vm, _args| Ok(Rc::new(RObject::integer(3)))),
    );

    // Build a second wrapper over the same RModule identity.
    let canonical = vm.get_const_by_name("W").expect("W defined");
    let inner: Rc<mrubyedge::yamrb::value::RModule> = match &canonical.value {
        RValue::Module(m) => m.clone(),
        _ => panic!("expected module value"),
    };
    let w2: Rc<RObject> = Rc::new(RObject::module(inner));

    // The singleton must resolve through the second wrapper too.
    let res = mrb_funcall(&mut vm, Some(w2), "tag", &[]).unwrap();
    let v: i64 = res.as_ref().try_into().expect("tag integer");
    assert_eq!(v, 3);
}

/// Case 7: regression guard - `def self.` in class bodies keeps working.
#[test]
fn class_singleton_regression() {
    let code = r#"
class C1
  def self.answer
    11
  end
end

C1.answer
"#;
    let binary = mrbc_compile("singleton_class_reg", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: i64 = result.as_ref().try_into().expect("answer must be integer");
    assert_eq!(value, 11);
}

/// Case 8: regression guard - extend on plain instances is untouched.
#[test]
fn instance_extend_regression() {
    let code = r#"
module E1
  def hello
    "hi"
  end
end

obj = Object.new
obj.extend(E1)
obj.hello
"#;
    let binary = mrbc_compile("singleton_extend_reg", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = VM::open(&mut rite);
    let result = vm.run().unwrap();
    let value: String = result.as_ref().try_into().expect("hello returns string");
    assert_eq!(value, "hi");
}
