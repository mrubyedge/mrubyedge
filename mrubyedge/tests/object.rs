extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use std::rc::Rc;

use helpers::*;
use mrubyedge::yamrb::value::RObject;

#[test]
fn object_test() {
    let code = "
    class Hello
      def world
        puts \"hello world\"
        1
      end
    end

    def test_main
      Hello.new.world
    end
    ";
    let binary = mrbc_compile("add", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 1);
}

#[test]
fn object_extend_test() {
    let code = r#"
    module Greeter
      def greet
        "Hello from module"
      end
    end

    module Farewell
      def bye
        "Goodbye from module"
      end
    end

    def test_extend
      obj = Object.new
      obj.extend(Greeter)
      result1 = obj.greet

      obj.extend(Farewell)
      result2 = obj.bye

      [result1, result2]
    end
    "#;
    let binary = mrbc_compile("extend", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_extend", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(
        TryInto::<String>::try_into(arr[0].as_ref()).unwrap(),
        "Hello from module"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[1].as_ref()).unwrap(),
        "Goodbye from module"
    );
}

#[test]
fn object_extend_multiple_modules_test() {
    let code = r#"
    module M1
      def m1_method
        "from M1"
      end
    end

    module M2
      def m2_method
        "from M2"
      end
    end

    def test_extend_multiple
      obj = Object.new
      obj.extend(M1, M2)
      [obj.m1_method, obj.m2_method]
    end
    "#;
    let binary = mrbc_compile("extend_multiple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_extend_multiple", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(
        TryInto::<String>::try_into(arr[0].as_ref()).unwrap(),
        "from M1"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[1].as_ref()).unwrap(),
        "from M2"
    );
}
