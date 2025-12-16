extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn lambda_test() {
    let code = r#"
    def test_lambda
      test = lambda { 42 }
      test.call
    end
    "#;
    let binary = mrbc_compile("lambda", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i64 = mrb_funcall(&mut vm, None, "test_lambda", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 42);
}

#[test]
fn lambda_with_arg_test() {
    let code = r#"
    def test_lambda_arg
      test = ->(a) { a * 2 }
      test.call(21)
    end
    "#;
    let binary = mrbc_compile("lambda_arg", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i64 = mrb_funcall(&mut vm, None, "test_lambda_arg", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 42);
}

#[test]
fn proc_new_test() {
    let code = r#"
    def test_proc_new
      test = Proc.new { |a, b| a + b }
      test.call(10, 32)
    end
    "#;
    let binary = mrbc_compile("proc_new", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i64 = mrb_funcall(&mut vm, None, "test_proc_new", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 42);
}

#[test]
fn proc_closure_test() {
    let code = r#"
    def test_proc_closure
      value = 10
      incrementer = Proc.new { |x| value = x + value }
      result1 = incrementer.call(5)
      result2 = incrementer.call(10)
      result3 = incrementer.call(100)
      [result1, result2, result3]
    end
    "#;
    let binary = mrbc_compile("proc_closure", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_proc_closure", &args).unwrap();
    
    if let mrubyedge::yamrb::value::RValue::Array(arr) = &result.value {
        let arr = arr.borrow();
        assert_eq!(arr.len(), 3);
        
        let r1: i64 = arr[0].as_ref().try_into().unwrap();
        let r2: i64 = arr[1].as_ref().try_into().unwrap();
        let r3: i64 = arr[2].as_ref().try_into().unwrap();
        
        assert_eq!(r1, 15);  // 10 + 5
        assert_eq!(r2, 25);  // 15 + 10
        assert_eq!(r3, 125); // 25 + 100
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn proc_block_test() {
    let code = r#"
    def accept_block(&block)
      block.call("test value")
    end

    def test_proc_block
      accept_block do |x|
        x.size
      end
    end
    "#;
    let binary = mrbc_compile("proc_block", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i64 = mrb_funcall(&mut vm, None, "test_proc_block", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 10);
}

#[test]
fn proc_class_variable_test() {
    let code = r#"
    class Router
      def self.get path, &block
        @routes ||= {}
        @routes[path] = block
      end

      def self.request path
        if @routes && @routes[path]
          @routes[path].call(path)
        else
          nil
        end
      end
    end

    def test_router
      Router.get "/home" do |path|
        path.size
      end
      
      result1 = Router.request "/home"
      result2 = Router.request "/about"
      [result1, result2]
    end
    "#;
    let binary = mrbc_compile("router", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_router", &args).unwrap();
    
    if let mrubyedge::yamrb::value::RValue::Array(arr) = &result.value {
        let arr = arr.borrow();
        assert_eq!(arr.len(), 2);
        
        // First result should be 5 (length of "/home")
        let r1: i64 = arr[0].as_ref().try_into().unwrap();
        assert_eq!(r1, 5);
        
        // Second result should be nil
        assert!(matches!(&arr[1].value, mrubyedge::yamrb::value::RValue::Nil));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn proc_call_method_test() {
    let code = r#"
    def test_proc_call
      my_proc = Proc.new { |x| x * 2 }
      my_proc.call(21)
    end
    "#;
    let binary = mrbc_compile("proc_call", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i64 = mrb_funcall(&mut vm, None, "test_proc_call", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 42);
}
