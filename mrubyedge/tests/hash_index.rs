// Regression tests for the Hash index fast path (OP_GETIDX/OP_SETIDX): the
// pristine Hash#[] and Hash#= run without a method send, but a redefinition,
// a subclass override or a singleton method must still be honoured.

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
fn hash_index_basic() {
    let code = r#"
    def test_main
      h = {}
      h["a"] = 1
      h["b"] = 2
      h["a"] = 3
      [h["a"], h["b"], h["missing"]]
    end
    "#;
    let arr = run_vec(code, "hx_basic");
    let a: i64 = arr[0].as_ref().try_into().unwrap();
    let b: i64 = arr[1].as_ref().try_into().unwrap();
    assert_eq!((a, b), (3, 2));
    assert!(arr[2].as_ref().is_nil());
}

#[test]
fn hash_index_mixed_keys_and_nesting() {
    let code = r#"
    def test_main
      h = { :sym => 1, 2 => "two", "three" => [3] }
      inner = { "x" => 10 }
      h["inner"] = inner
      [h[:sym], h[2], h["three"][0], h["inner"]["x"]]
    end
    "#;
    let arr = run_vec(code, "hx_mixed");
    let sym: i64 = arr[0].as_ref().try_into().unwrap();
    let two: String = arr[1].as_ref().try_into().unwrap();
    let three: i64 = arr[2].as_ref().try_into().unwrap();
    let inner: i64 = arr[3].as_ref().try_into().unwrap();
    assert_eq!(sym, 1);
    assert_eq!(two, "two");
    assert_eq!(three, 3);
    assert_eq!(inner, 10);
}

#[test]
fn redefined_hash_index_is_respected() {
    let code = r#"
    class Hash
      def [](_k)
        42
      end
    end

    def test_main
      h = { "a" => 1 }
      [h["a"]]
    end
    "#;
    let arr = run_vec(code, "hx_redefined");
    let v: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, 42);
}

#[test]
fn redefined_hash_aset_is_respected() {
    let code = r#"
    class Hash
      def []=(k, _v)
        "set #{k}"
      end
    end

    def test_main
      h = {}
      r = (h["k"] = 1)
      [r, h["k"].nil?]
    end
    "#;
    let arr = run_vec(code, "hx_aset_redefined");
    let r: String = arr[0].as_ref().try_into().unwrap();
    let still_missing: bool = arr[1].as_ref().try_into().unwrap();
    assert_eq!(r, "set k");
    assert!(still_missing);
}

#[test]
fn singleton_override_is_respected() {
    let code = r#"
    def test_main
      h = { "a" => 1 }
      def h.[](k)
        "single:#{k}"
      end
      [h["a"]]
    end
    "#;
    let arr = run_vec(code, "hx_singleton");
    let v: String = arr[0].as_ref().try_into().unwrap();
    assert_eq!(v, "single:a");
}

#[test]
fn fast_path_hits_counter_moves() {
    let code = r#"
    def test_main
      h = {}
      i = 0
      while i < 100
        h[i] = i * 2
        i += 1
      end
      sum = 0
      i = 0
      while i < 100
        sum += h[i]
        i += 1
      end
      sum
    end
    "#;
    let binary = mrbc_compile("hx_counter", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    let before = vm.hash_fast_hits.get();
    let args = vec![];
    let sum: i64 = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(sum, 9900);
    assert!(
        vm.hash_fast_hits.get() - before >= 199,
        "expected the inline path to serve the loop, got {} hits",
        vm.hash_fast_hits.get() - before
    );
}

#[test]
fn redefinition_after_warm_site_stops_fast_path() {
    // The read site is warmed at top level; the class-wide redefinition then
    // bumps the method version, so the site must go cold and see the new
    // body through the same call site inside `read`.
    let code = r#"
    def read(h, k)
      h[k]
    end

    h = { "a" => 1 }
    i = 0
    while i < 50
      read(h, "a")
      i += 1
    end

    class Hash
      def [](_k)
        7
      end
    end

    def test_main
      [read({ "a" => 1 }, "a")]
    end
    "#;
    let arr = run_vec(code, "hx_warm_redef");
    let after: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(after, 7);
}
