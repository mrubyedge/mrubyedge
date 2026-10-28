// Regression tests for the method dispatch caches. These pin the dispatch
// semantics that must survive caching: redefinitions, reopened classes,
// runtime `include`, method_missing, super and singleton methods all have to
// invalidate or resolve correctly once lookups are cached.

extern crate mrubyedge;

mod helpers;
use helpers::*;

fn run_i32(code: &'static str, fname: &'static str) -> i32 {
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
fn func_dispatch_works() {
    let code = r#"
    class Calc
      def twice(n)
        n * 2
      end
    end

    def test_main
      c = Calc.new
      c.twice(21)
    end
    "#;
    assert_eq!(run_i32(code, "mc_func_dispatch"), 42);
}

#[test]
fn redefinition_invalidates_same_call_site() {
    // The same send site inside `read_b` runs before and after the singleton
    // method is redefined; the cached lookup must see the new body. (The
    // mrbc compiler does not emit `class Foo` reopens inside methods, so the
    // redefinition goes through a singleton def, which shares the same
    // op_def/version-bump machinery as any instance method.)
    let code = r#"
    class Foo
      def a
        1
      end
    end

    def read_b(f)
      f.b
    end

    def test_main
      foo = Foo.new
      def foo.b
        1
      end
      first = read_b(foo)
      def foo.b
        2
      end
      first + read_b(foo)
    end
    "#;
    assert_eq!(run_i32(code, "mc_redefine_site"), 3);
}

#[test]
fn method_added_after_calls() {
    let code = r#"
    class Foo
      def a
        1
      end
    end

    def read_a(f)
      f.a
    end

    def test_main
      foo = Foo.new
      first = read_a(foo)
      def foo.b
        40
      end
      first + foo.b
    end
    "#;
    assert_eq!(run_i32(code, "mc_method_added"), 41);
}

#[test]
fn include_invalidates_warmed_site() {
    // A warmed send site resolves `x` from M1; including M2 (inserted at the
    // front of the mixin chain) must invalidate the cached lookup so the same
    // site now resolves M2.
    let code = r#"
    module M1
      def x
        1
      end
    end

    module M2
      def x
        2
      end
    end

    class Foo
      include M1
    end

    def read(f)
      f.x
    end

    def test_main
      foo = Foo.new
      first = read(foo)
      Foo.include(M2)
      first + read(foo)
    end
    "#;
    assert_eq!(run_i32(code, "mc_include_warm"), 3);
}

#[test]
fn include_after_calls_resolves_new_method() {
    // The call site previously hit method_missing (not cached); after the
    // runtime include the same site must resolve the module method.
    let code = r#"
    module M
      def b
        42
      end
    end

    class Foo
      def method_missing(name, *args)
        0
      end
    end

    def read_b(f)
      f.b
    end

    def test_main
      foo = Foo.new
      before = read_b(foo)
      Foo.include(M)
      read_b(foo) + before
    end
    "#;
    assert_eq!(run_i32(code, "mc_include"), 42);
}

#[test]
fn method_missing_after_calls() {
    let code = r#"
    class Foo
      def known
        1
      end

      def method_missing(name, *args)
        "mm:" + name.to_s
      end
    end

    def read(f)
      [f.known, f.unknown].join(",")
    end

    def test_main
      read(Foo.new)
    end
    "#;
    let binary = mrbc_compile("mc_method_missing", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let result: String = mrb_funcall(&mut vm, None, "test_main", &[])
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, "1,mm:unknown");
}

#[test]
fn super_resolves_through_cached_site() {
    // B#v (cached at the read call site) calls super; the uncached next-method
    // walk must still reach A#v every time.
    let code = r#"
    class A
      def v
        1
      end
    end

    class B < A
      def v
        super + 1
      end
    end

    def read(b)
      b.v
    end

    def test_main
      b = B.new
      read(b) + read(b)
    end
    "#;
    assert_eq!(run_i32(code, "mc_super_site"), 4);
}

#[test]
fn singleton_method_after_calls() {
    let code = r#"
    class Foo
      def a
        1
      end
    end

    def read_a(f)
      f.a
    end

    def test_main
      foo = Foo.new
      read_a(foo)
      def foo.b
        2
      end
      read_a(foo) + foo.b
    end
    "#;
    assert_eq!(run_i32(code, "mc_singleton"), 3);
}

#[test]
fn polymorphic_call_site() {
    // One send site reached by two different receiver classes.
    let code = r#"
    class A
      def v
        1
      end
    end

    class B
      def v
        2
      end
    end

    def read(o)
      o.v
    end

    def test_main
      read(A.new) + read(B.new) + read(A.new)
    end
    "#;
    assert_eq!(run_i32(code, "mc_polymorphic"), 4);
}

#[test]
fn attr_accessor_after_warm() {
    let code = r#"
    class Foo
      attr_accessor :x
    end

    def test_main
      foo = Foo.new
      foo.x = 5
      foo.x
    end
    "#;
    assert_eq!(run_i32(code, "mc_attr_warm"), 5);
}
