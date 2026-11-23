// Regression tests for bare `super` argument forwarding (ARGARY + SUPER
// CALL_MAXARGS): the call forwards the method's own arguments, rest included,
// through both bytecode sends and native funcalls (Class#new), while arity
// errors still come from the callee.

extern crate mrubyedge;

mod helpers;
use helpers::*;
use std::rc::Rc;

fn run_vec(code: &'static str, fname: &'static str) -> Vec<Rc<mrubyedge::yamrb::value::RObject>> {
    let binary = mrbc_compile(fname, code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let args = vec![];
    mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap()
}

#[test]
fn bare_super_forwards_required_and_rest() {
    let code = r#"
    class A
      def m(a, b)
        [a, b]
      end
    end
    class B < A
      def m(a, *rest)
        super
      end
    end

    def test_main
      B.new.m(1, 2)
    end
    "#;
    let arr = run_vec(code, "sa_req_rest");
    let a: i64 = arr[0].as_ref().try_into().unwrap();
    let b: i64 = arr[1].as_ref().try_into().unwrap();
    assert_eq!((a, b), (1, 2));
}

#[test]
fn bare_super_with_empty_rest_forwards_required_only() {
    let code = r#"
    class A
      def m(a)
        a * 10
      end
    end
    class B < A
      def m(a, *rest)
        super
      end
    end

    def test_main
      [B.new.m(4)]
    end
    "#;
    let arr = run_vec(code, "sa_empty_rest");
    let v: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, 40);
}

#[test]
fn bare_super_with_post_required() {
    let code = r#"
    class A
      def m(first, *middle, last)
        [first, middle.join(","), last].join("-")
      end
    end
    class B < A
      def m(first, *middle, last)
        super
      end
    end

    def test_main
      [B.new.m("a", "b", "c", "d")]
    end
    "#;
    let arr = run_vec(code, "sa_post");
    let v: String = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, "a-b,c-d");
}

#[test]
fn bare_super_from_initialize_via_class_new() {
    // Class#new enters initialize through a native funcall frame; the
    // suspended callinfo must stay reachable for `super`.
    let code = r#"
    class Base
      def initialize(a, b)
        @sum = a + b
      end
      def sum
        @sum
      end
    end
    class Child < Base
      def initialize(x, *rest)
        super
      end
    end

    def test_main
      [Child.new(1, 2).sum]
    end
    "#;
    let arr = run_vec(code, "sa_funcall");
    let v: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, 3);
}

#[test]
fn bare_super_arity_error_comes_from_callee() {
    let code = r#"
    class A
      def m(a)
        a
      end
    end
    class B < A
      def m(a, *rest)
        super
      end
    end

    def test_main
      begin
        B.new.m(1, 2)
        [0]
      rescue ArgumentError
        [1]
      end
    end
    "#;
    let arr = run_vec(code, "sa_arity");
    let v: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, 1);
}

#[test]
fn explicit_super_still_works() {
    let code = r#"
    class A
      def m(a, b)
        a + b
      end
    end
    class B < A
      def m(x, y)
        super(x * 2, y * 3)
      end
    end

    def test_main
      [B.new.m(1, 2)]
    end
    "#;
    let arr = run_vec(code, "sa_explicit");
    let v: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, 8);
}

#[test]
fn bare_super_in_block_forwards_enclosing_method_args() {
    let code = r#"
    class A
      def m(a, b)
        [a, b]
      end
    end
    class B < A
      def m(a, *rest)
        r = nil
        [1].each do
          r = super
        end
        r
      end
    end

    def test_main
      [B.new.m(3, 4)]
    end
    "#;
    let arr = run_vec(code, "sa_block");
    let inner: Vec<Rc<mrubyedge::yamrb::value::RObject>> = arr[0].as_ref().try_into().unwrap();
    let a: i64 = inner[0].as_ref().try_into().unwrap();
    let b: i64 = inner[1].as_ref().try_into().unwrap();
    assert_eq!((a, b), (3, 4));
}

#[test]
fn bare_super_to_native_parent() {
    let code = r#"
    class CountingArray < Array
      def push(x)
        super
        1
      end
    end

    def test_main
      arr = CountingArray.new
      arr.push(7)
      [arr.length]
    end
    "#;
    let arr = run_vec(code, "sa_native");
    let v: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, 1);
}
