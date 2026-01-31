extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn basic_method_missing_test() {
    let code = "
class Foo
  def method_missing(name, *args)
    name.to_s + ':' + args.length.to_s
  end
end

foo = Foo.new
foo.bar(1, 2, 3)
    ";
    let binary = mrbc_compile("basic_method_missing", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "bar:3");
}

#[test]
fn method_missing_with_no_args_test() {
    let code = "
class Foo
  def method_missing(name)
    'missing:' + name.to_s
  end
end

foo = Foo.new
foo.unknown_method
    ";
    let binary = mrbc_compile("method_missing_no_args", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "missing:unknown_method");
}

#[test]
fn method_missing_return_value_test() {
    let code = "
class Calculator
  def method_missing(name, *args)
    if name == :add
      args[0] + args[1]
    elsif name == :multiply
      args[0] * args[1]
    else
      0
    end
  end
end

calc = Calculator.new
result1 = calc.add(10, 20)
result2 = calc.multiply(5, 6)
result1 + result2
    ";
    let binary = mrbc_compile("method_missing_return", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 60); // (10 + 20) + (5 * 6) = 30 + 30
}

#[test]
fn method_missing_with_kwargs_test() {
    let code = "
class Foo
  def method_missing(name, *args, **kwargs)
    name.to_s + ':' + kwargs.length.to_s
  end
end

foo = Foo.new
foo.test_method(1, 2, a: 3, b: 4, c: 5)
    ";
    let binary = mrbc_compile("method_missing_kwargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "test_method:3");
}

#[test]
fn method_missing_access_kwargs_test() {
    let code = "
class Config
  def method_missing(name, **options)
    if options[:default]
      options[:default]
    else
      'no default'
    end
  end
end

config = Config.new
result1 = config.setting1(default: 'value1')
result2 = config.setting2()
result1 + ',' + result2
    ";
    let binary = mrbc_compile("method_missing_access_kwargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "value1,no default");
}

#[test]
fn method_missing_multiple_calls_test() {
    let code = "
class Counter
  def initialize
    @count = 0
  end

  def method_missing(name, *args)
    @count = @count + 1
    @count
  end
end

counter = Counter.new
r1 = counter.foo()
r2 = counter.bar()
r3 = counter.baz()
r1 + r2 + r3
    ";
    let binary = mrbc_compile("method_missing_multiple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 6); // 1 + 2 + 3
}

#[test]
fn method_missing_not_defined_error_test() {
    let code = "
class Bar
end

def call_nonexistent
  bar = Bar.new
  bar.nonexistent_method
end
    ";
    let binary = mrbc_compile("no_method_missing", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "call_nonexistent", &args).err();
    let error = result.unwrap();
    let error_msg = error.message();
    assert!(
        error_msg.contains("undefined method")
            && error_msg.contains("Bar")
            && error_msg.contains("nonexistent_method")
    );
}

#[test]
fn method_missing_from_funcall_test() {
    let code = "
class Bar
  def method_missing(name, *args)
    \"handled by method_missing: #{name}\"
  end
end

Bar.new
    ";
    let binary = mrbc_compile("funcall_method_missing", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let target = vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, Some(target), "call_nonexistent", &args).unwrap();
    let msg: String = result.as_ref().try_into().unwrap();
    assert_eq!(msg, "handled by method_missing: call_nonexistent");
}
