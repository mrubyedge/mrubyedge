extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn constants_resolve_repeatedly_test() {
    let code = r#"
    module Cache
      X = 40
    end

    def read
      Cache::X + Cache::X + Cache::X
    end
    read
    "#;
    let binary = mrbc_compile("const_cache_repeat", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(v, 120);
}

#[test]
fn constant_redefinition_invalidates_cache_test() {
    let code = r#"
    X = 1
    a = X
    X = 2
    b = X
    [a, b]
    "#;
    let binary = mrbc_compile("const_cache_redef", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![1, 2]);
}

#[test]
fn namespaced_redefinition_invalidates_cache_test() {
    let code = r#"
    module M
      V = 1
      def self.read
        V
      end
    end
    a = M.read
    module M
      V = 99
    end
    b = M.read
    [a, b]
    "#;
    let binary = mrbc_compile("const_cache_ns_redef", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![1, 99]);
}

#[test]
fn later_class_in_namespace_invalidates_cache_test() {
    // The site inside M.read caches the top-level X on the first call
    // (M has no X yet); defining class M::X must invalidate it so the
    // second call resolves through M.
    let code = r#"
    X = 1
    module M
      def self.read
        X
      end
    end
    a = M.read
    module M
      class X
        def self.marker
          :cls
        end
      end
    end
    b = M.read
    [a, b.marker]
    "#;
    let binary = mrbc_compile("const_cache_class_shadow", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let a: i64 = arr[0].as_ref().try_into().unwrap();
    let marker: String = arr[1].as_ref().try_into().unwrap();
    assert_eq!(a, 1);
    assert_eq!(marker, "cls");
}
