#![allow(clippy::bool_assert_comparison)]
extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn is_a_with_inheritance_and_modules_test() {
    let script = r#"
    class X 
    end

    module Z
    end

    module W
    include Z
    end

    class Y < X
    include W
    end

    class V
    end

    def test_main
    o = Y.new
    [
        o.is_a?(X),
        o.is_a?(Y),
        o.is_a?(Z),
        o.is_a?(W),
        o.is_a?(V)
    ]
    end

    test_main
    "#;

    let binary = mrbc_compile("is_a_test", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let values: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    // o.is_a?(X) => true
    let val1: bool = values[0].as_ref().try_into().unwrap();
    assert_eq!(val1, true);

    // o.is_a?(Y) => true
    let val2: bool = values[1].as_ref().try_into().unwrap();
    assert_eq!(val2, true);

    // o.is_a?(Z) => true
    let val3: bool = values[2].as_ref().try_into().unwrap();
    assert_eq!(val3, true);

    // o.is_a?(W) => true
    let val4: bool = values[3].as_ref().try_into().unwrap();
    assert_eq!(val4, true);

    // o.is_a?(V) => false
    let val5: bool = values[4].as_ref().try_into().unwrap();
    assert_eq!(val5, false);
}

#[test]
fn is_a_with_ancestors_test() {
    let script = r#"
    module Z
    end

    module W
    include Z
    end

    class Y
    include W
    end

    def test_ancestors
    Y.ancestors
    end

    test_ancestors
    "#;

    let binary = mrbc_compile("is_a_ancestors", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    // Just verify it runs successfully and returns an array
    let values: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    // Should have at least Y, W, Z, Object, etc.
    assert!(values.len() >= 3);
}

#[test]
fn is_a_basic_types_test() {
    let script = r#"
    def test_basic
    [
        1.is_a?(Integer),
        "hello".is_a?(String),
        [1,2,3].is_a?(Array),
        true.is_a?(TrueClass),
        false.is_a?(FalseClass),
        nil.is_a?(NilClass)
    ]
    end

    test_basic
    "#;

    let binary = mrbc_compile("is_a_basic", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let values: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    // All should be true
    for (i, val) in values.iter().enumerate() {
        let bool_val: bool = val.as_ref().try_into().unwrap();
        assert_eq!(bool_val, true, "Test case {} failed", i);
    }
}

#[test]
fn is_a_object_superclass_test() {
    let script = r#"
    class MyClass
    end

    def test_object
    o = MyClass.new
    [
        o.is_a?(MyClass),
        o.is_a?(Object)
    ]
    end

    test_object
    "#;

    let binary = mrbc_compile("is_a_object", script);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();

    let values: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    // Both should be true
    let val1: bool = values[0].as_ref().try_into().unwrap();
    assert_eq!(val1, true);

    let val2: bool = values[1].as_ref().try_into().unwrap();
    assert_eq!(val2, true);
}
