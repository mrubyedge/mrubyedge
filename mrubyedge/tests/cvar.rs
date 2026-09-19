extern crate mrubyedge;

mod helpers;

use helpers::*;
use mrubyedge::yamrb::value::RObject;
use std::rc::Rc;

#[test]
fn class_body_write_and_read_test() {
    let code = r#"
    class A
      @@count = 0
      def read
        @@count
      end
    end
    def test_cvar_class_body
      A.new.read
    end
    "#;
    let binary = mrbc_compile("cvar_class_body", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_cvar_class_body", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 0);
}

#[test]
fn cvar_inheritance_shares_definition_site_test() {
    let code = r#"
    class A
      @@count = 0
      def bump
        @@count += 1
      end
      def get
        @@count
      end
    end
    class B < A
    end
    def test_cvar_inheritance
      b = B.new
      b.bump
      b.bump
      b.get
    end
    "#;
    let binary = mrbc_compile("cvar_inheritance_shared", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_cvar_inheritance", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn cvar_read_from_parent_through_subclass_test() {
    let code = r#"
    class A
      @@name = "parent"
    end
    class B < A
      def name
        @@name
      end
    end
    def test_cvar_read_parent
      B.new.name
    end
    "#;
    let binary = mrbc_compile("cvar_read_parent", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_cvar_read_parent", &args).unwrap();
    let result: String = result.as_ref().try_into().unwrap();
    assert_eq!(result, "parent");
}

#[test]
fn cvar_uninitialized_raises_name_error_test() {
    let code = r#"
    class C
      @@missing
    end
    "#;
    let binary = mrbc_compile("cvar_uninitialized", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    assert!(
        vm.run().is_err(),
        "uninitialized class variable should error"
    );
}

#[test]
fn cvar_separate_tables_per_class_test() {
    let code = r#"
    class A
      @@x = 1
      def getx
        @@x
      end
    end
    class B
      @@x = 2
      def getx
        @@x
      end
    end
    def test_cvar_separate
      [A.new.getx, B.new.getx]
    end
    "#;
    let binary = mrbc_compile("cvar_separate_tables", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_cvar_separate", &args).unwrap();
    let outer: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(outer.len(), 2);
    let a: i64 = outer[0].as_ref().try_into().unwrap();
    let b: i64 = outer[1].as_ref().try_into().unwrap();
    assert_eq!((a, b), (1, 2));
}
