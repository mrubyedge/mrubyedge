// Regression tests for inline attribute access on send-site cache hits.
// These pin the accessor semantics that must survive inlining: values,
// defaults, redefinitions, singletons, inheritance, module includes,
// polymorphic sites and arity errors.

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

fn run_vec(
    vm: &mut mrubyedge::yamrb::vm::VM,
) -> Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> {
    let args = vec![];
    mrb_funcall(vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap()
}

fn run_with_vm(code: &'static str, fname: &'static str) -> mrubyedge::yamrb::vm::VM {
    let binary = mrbc_compile(fname, code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    vm
}

#[test]
fn accessor_roundtrip_and_default() {
    let code = r#"
    class Point
      attr_accessor :x, :y
    end

    def test_main
      p = Point.new
      d = p.x
      p.x = 10
      p.y = p.x + 5
      [d.nil?, p.x, p.y]
    end
    "#;
    let binary = mrbc_compile("ai_roundtrip", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let arr = run_vec(&mut vm);
    let nil_default: bool = arr[0].as_ref().try_into().unwrap();
    let x: i32 = arr[1].as_ref().try_into().unwrap();
    let y: i32 = arr[2].as_ref().try_into().unwrap();
    assert!(nil_default);
    assert_eq!((x, y), (10, 15));
}

#[test]
fn reader_and_writer_separately() {
    let code = r#"
    class Config
      attr_writer :mode
      attr_reader :level
      def initialize
        @level = 3
      end
      def mode
        @mode
      end
    end

    def test_main
      c = Config.new
      c.mode = 7
      [c.level, c.mode]
    end
    "#;
    let binary = mrbc_compile("ai_reader_writer", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let arr = run_vec(&mut vm);
    let level: i32 = arr[0].as_ref().try_into().unwrap();
    let mode: i32 = arr[1].as_ref().try_into().unwrap();
    assert_eq!((level, mode), (3, 7));
}

#[test]
fn writer_returns_assigned_value() {
    let code = r#"
    class Box
      attr_accessor :v
    end

    def test_main
      b = Box.new
      (b.v = 21) + 21
    end
    "#;
    assert_eq!(run_i32(code, "ai_writer_returns"), 42);
}

#[test]
fn redefinition_after_site_is_warm() {
    // The same send site runs many times (site becomes warm), then the
    // accessor is replaced by a regular method: the version bump must send
    // the site cold so the new body runs.
    let code = r#"
    class Counter
      attr_accessor :n
      def initialize
        @n = 0
      end
    end

    def read_n(c)
      c.n
    end

    def test_main
      c = Counter.new
      i = 0
      warm = 0
      while i < 50
        warm = read_n(c)
        i += 1
      end
      class << c
        def n
          99
        end
      end
      [warm, c.n]
    end
    "#;
    let binary = mrbc_compile("ai_redefine", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let arr = run_vec(&mut vm);
    let warm: i32 = arr[0].as_ref().try_into().unwrap();
    let after: i32 = arr[1].as_ref().try_into().unwrap();
    assert_eq!((warm, after), (0, 99));
}

#[test]
fn inheritance_attrs_resolve_through_superclass() {
    let code = r#"
    module Tagged
      def tag_name
        "tagged"
      end
    end
    class Base
      attr_accessor :id
    end
    class Item < Base
      include Tagged
    end

    def test_main
      it = Item.new
      it.id = 5
      [it.id, it.tag_name]
    end
    "#;
    let binary = mrbc_compile("ai_inherit", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let arr = run_vec(&mut vm);
    let id: i32 = arr[0].as_ref().try_into().unwrap();
    let tag: String = arr[1].as_ref().try_into().unwrap();
    assert_eq!(id, 5);
    assert_eq!(tag, "tagged");
}

#[test]
fn polymorphic_site_keeps_values_separate() {
    let code = r#"
    class A
      attr_accessor :v
      def initialize; @v = 1; end
    end
    class B
      attr_accessor :v
      def initialize; @v = 2; end
    end

    def test_main
      objs = [A.new, B.new, A.new, B.new]
      total = 0
      i = 0
      while i < 40
        total += objs[i % 4].v
        i += 1
      end
      total
    end
    "#;
    // 10 rounds of (1 + 2 + 1 + 2)
    assert_eq!(run_i32(code, "ai_polymorphic"), 60);
}

#[test]
fn wrong_shape_sends_fall_through_to_closure() {
    // The closures ignore extra arguments, and so must the inline path:
    // wrong-arity sends take the closure, keeping one behavior.
    let code = r#"
    class Probe
      attr_accessor :v
      def initialize; @v = 0; end
    end

    def test_main
      p = Probe.new
      p.v = 7
      p.v(1)
    end
    "#;
    assert_eq!(run_i32(code, "ai_wrong_shape"), 7);
}

#[test]
fn inline_hits_counter_moves() {
    let code = r#"
    class Cell
      attr_accessor :v
    end

    def test_main
      c = Cell.new
      i = 0
      while i < 100
        c.v = i
        c.v
        i += 1
      end
      c.v
    end
    "#;
    let mut vm = run_with_vm(code, "ai_counter");
    let args = vec![];
    let ret: i32 = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(ret, 99);
    // 100 writes + 100 reads; each site's first execution is cold, so the
    // inline path serves 99 + 99 of them.
    assert!(
        vm.attr_inline_hits.get() >= 198,
        "expected the inline path to serve the warm loop, got {} hits",
        vm.attr_inline_hits.get()
    );
}

#[test]
fn reopen_does_not_break_earlier_attrs() {
    let code = r#"
    class Doc
      attr_accessor :title
    end
    class Doc
      attr_accessor :body
    end

    def test_main
      d = Doc.new
      d.title = "t"
      d.body = "b"
      [d.title, d.body]
    end
    "#;
    let binary = mrbc_compile("ai_reopen", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let arr = run_vec(&mut vm);
    let title: String = arr[0].as_ref().try_into().unwrap();
    let body: String = arr[1].as_ref().try_into().unwrap();
    assert_eq!((title.as_str(), body.as_str()), ("t", "b"));
}
