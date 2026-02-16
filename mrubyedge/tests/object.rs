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

#[test]
fn object_extend_overrides_class_method_test() {
    let code = r#"
    class MyClass
      def greet
        "from class"
      end
    end

    module MyModule
      def greet
        "from module"
      end
    end

    def test_extend_override
      obj = MyClass.new
      obj.extend(MyModule)
      obj.greet
    end
    "#;
    let binary = mrbc_compile("extend_override_class", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_extend_override", &args).unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "from module");
}

#[test]
fn object_extend_singleton_method_priority_test() {
    let code = r#"
    module MyModule
      def greet
        "from module"
      end
    end

    def test_singleton_priority
      obj = Object.new
      obj.extend(MyModule)
      
      def obj.greet
        "from singleton"
      end
      
      obj.greet
    end
    "#;
    let binary = mrbc_compile("extend_singleton_priority", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_singleton_priority", &args).unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "from singleton");
}

#[test]
fn object_extend_multiple_priority_test() {
    let code = r#"
    module M1
      def greet
        "from M1"
      end
    end

    module M2
      def greet
        "from M2"
      end
    end

    module M3
      def greet
        "from M3"
      end
    end

    def test_multiple_priority
      obj = Object.new
      obj.extend(M1)
      result1 = obj.greet
      
      obj.extend(M2)
      result2 = obj.greet
      
      obj.extend(M3)
      result3 = obj.greet
      
      [result1, result2, result3]
    end
    "#;
    let binary = mrbc_compile("extend_multiple_priority", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_multiple_priority", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(
        TryInto::<String>::try_into(arr[0].as_ref()).unwrap(),
        "from M1"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[1].as_ref()).unwrap(),
        "from M2"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[2].as_ref()).unwrap(),
        "from M3"
    );
}

#[test]
fn object_extend_multiple_arguments_priority_test() {
    let code = r#"
    module M1
      def greet
        "from M1"
      end
      
      def m1_only
        "M1 only"
      end
    end

    module M2
      def greet
        "from M2"
      end
      
      def m2_only
        "M2 only"
      end
    end

    def test_args_priority
      obj = Object.new
      # extend(M1, M2) extends in order: M2, then M1, so M1 has priority
      obj.extend(M1, M2)
      [obj.greet, obj.m1_only, obj.m2_only]
    end
    "#;
    let binary = mrbc_compile("extend_args_priority", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_args_priority", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    // M1 is extended last, so greet calls M1's method
    assert_eq!(
        TryInto::<String>::try_into(arr[0].as_ref()).unwrap(),
        "from M1"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[1].as_ref()).unwrap(),
        "M1 only"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[2].as_ref()).unwrap(),
        "M2 only"
    );
}

#[test]
fn object_loop_basic_test() {
    let code = r#"
    def test_loop
      i = 0
      loop do
        i += 1
        break if i >= 5
      end
      i
    end
    "#;
    let binary = mrbc_compile_debug("loop_basic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_loop", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5);
}
