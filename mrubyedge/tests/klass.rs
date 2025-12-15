extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn attr_reader_test() {
    let code = "
    class Hello
      attr_reader :world

      def update_world
        @world = 123
      end
    end

    def test_main
      w = Hello.new
      w.update_world
      w.world
    end
    ";
    let binary = mrbc_compile("attr_reader", code);
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
    assert_eq!(result, 123);
}

#[test]
fn attr_reader_2_test() {
    let code = "
    class Hello
      attr_reader :world
    end

    def test_main
      w = Hello.new
      w.world
    end
    ";
    let binary = mrbc_compile("attr_reader_2", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_main", &args).unwrap();
    assert!(result.as_ref().is_nil());
}

#[test]
fn attr_accessor_test() {
    let code = "
    class Hello
      attr_accessor :world
    end

    def test_main
      w = Hello.new
      w.world = \"Hola, attr\"
      w.world
    end
    ";
    let binary = mrbc_compile("attr_accessor", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: String = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(&result, "Hola, attr");
}

#[test]
fn class_definition_isolation_test() {
    let code = "
    class Test1
      def hello
        123
      end
    end

    class Test2
      def hello
        456
      end
    end

    def test_main1
      Test1.new.hello
    end

    def test_main2
      Test2.new.hello
    end
    ";
    let binary = mrbc_compile("class_definition_isolation", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let val1: i32 = mrb_funcall(&mut vm, None, "test_main1", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    let val2: i32 = mrb_funcall(&mut vm, None, "test_main2", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(val1, 123);
    assert_eq!(val2, 456);
}

#[test]
fn class_inheritance_super_test() {
    let code = "
    class Test1
      def hello
        123
      end
    end

    class Test3 < Test1
      def hello
        super + 1
      end
    end

    def test_main
      Test3.new.hello
    end
    ";
    let binary = mrbc_compile("class_inheritance_super", code);
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
    assert_eq!(result, 124);
}

#[test]
fn class_define_class_method_test() {
    let code = "
    class Test
      def self.hello
        123
      end
    end

    def test_main
      Test.hello
    end
    ";
    let binary = mrbc_compile("class_define_class_method", code);
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
    assert_eq!(result, 123);
}

#[test]
fn class_inheritance_class_method_test() {
    let code = "
    class Test1
      def self.hello
        123
      end
    end

    class Test2 < Test1
      def self.hello
        super + 1
      end
    end

    def test_main
      Test2.hello
    end
    ";
    let binary = mrbc_compile_debug("class_inheritance_class_method", code);
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
    assert_eq!(result, 124);
}
