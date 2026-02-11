extern crate mec_mrbc_sys;
extern crate mrubyedge;

use std::rc::Rc;

use super::helpers::*;
use mrubyedge::yamrb::value::RObject;

#[test]
fn test_eval_multiple_rites_with_classes() {
    // First code: define Foo class
    let code1 = r#"
    class Foo
      def bar
        "Hello from Foo"
      end
    end
    "#;
    let binary1 = mrbc_compile("code1", code1);
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite1);
    vm.run().unwrap();

    // Second code: define Bar class
    let code2 = r#"
    class Bar
      def baz
        "Hello from Bar"
      end
    end
    "#;
    let binary2 = mrbc_compile("code2", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    vm.eval_rite(&mut rite2).unwrap();

    // Third code: use both Foo and Bar classes
    let code3 = r#"
    def test_both
      foo = Foo.new
      bar = Bar.new
      [foo.bar, bar.baz]
    end
    "#;
    let binary3 = mrbc_compile("code3", code3);
    let mut rite3 = mrubyedge::rite::load(&binary3).unwrap();
    vm.eval_rite(&mut rite3).unwrap();

    // Call the method that uses both classes
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_both", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(
        TryInto::<String>::try_into(arr[0].as_ref()).unwrap(),
        "Hello from Foo"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[1].as_ref()).unwrap(),
        "Hello from Bar"
    );
}

#[test]
fn test_eval_multiple_rites_accumulate_methods() {
    // First code: define initial method
    let code1 = r#"
    def greet(name)
      "Hello, #{name}!"
    end
    "#;
    let binary1 = mrbc_compile("greet", code1);
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite1);
    vm.run().unwrap();

    // Second code: define another method
    let code2 = r#"
    def farewell(name)
      "Goodbye, #{name}!"
    end
    "#;
    let binary2 = mrbc_compile("farewell", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    vm.eval_rite(&mut rite2).unwrap();

    // Third code: use both methods
    let code3 = r#"
    def test_methods
      [greet("Alice"), farewell("Bob")]
    end
    "#;
    let binary3 = mrbc_compile("test_methods", code3);
    let mut rite3 = mrubyedge::rite::load(&binary3).unwrap();
    vm.eval_rite(&mut rite3).unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_methods", &args).unwrap();
    let arr: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    assert_eq!(
        TryInto::<String>::try_into(arr[0].as_ref()).unwrap(),
        "Hello, Alice!"
    );
    assert_eq!(
        TryInto::<String>::try_into(arr[1].as_ref()).unwrap(),
        "Goodbye, Bob!"
    );
}

#[test]
fn test_eval_multiple_rites_with_inheritance() {
    // First code: define base class
    let code1 = r#"
    class Animal
      def speak
        "Some sound"
      end
    end
    "#;
    let binary1 = mrbc_compile("animal", code1);
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite1);
    vm.run().unwrap();

    // Second code: define subclass that inherits from Animal
    let code2 = r#"
    class Dog < Animal
      def speak
        "Woof!"
      end
    end
    "#;
    let binary2 = mrbc_compile("dog", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    vm.eval_rite(&mut rite2).unwrap();

    // Third code: use the subclass
    let code3 = r#"
    def test_inheritance
      dog = Dog.new
      dog.speak
    end
    "#;
    let binary3 = mrbc_compile("test_inheritance", code3);
    let mut rite3 = mrubyedge::rite::load(&binary3).unwrap();
    vm.eval_rite(&mut rite3).unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_inheritance", &args).unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "Woof!");
}

#[test]
fn test_eval_multiple_rites_with_modules() {
    // First code: define module
    let code1 = r#"
    module Greeter
      def greet
        "Hello from #{@name}"
      end
    end
    "#;
    let binary1 = mrbc_compile("module", code1);
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite1);
    vm.run().unwrap();

    // Second code: define class that includes the module
    let code2 = r#"
    class Person
      include Greeter
      
      def initialize(name)
        @name = name
      end
    end
    "#;
    let binary2 = mrbc_compile("person", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    vm.eval_rite(&mut rite2).unwrap();

    // Third code: use the class with included module
    let code3 = r#"
    def test_module_include
      person = Person.new("Alice")
      person.greet
    end
    "#;
    let binary3 = mrbc_compile("test_include", code3);
    let mut rite3 = mrubyedge::rite::load(&binary3).unwrap();
    vm.eval_rite(&mut rite3).unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_module_include", &args).unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "Hello from Alice");
}
