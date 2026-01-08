extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use std::rc::Rc;

use helpers::*;
use mrubyedge::Error;
use mrubyedge::yamrb::helpers::mrb_define_cmethod;
use mrubyedge::yamrb::value::RObject;
use mrubyedge::yamrb::vm::VM;

#[test]
fn basic_keyword_args_test() {
    let code = "
    def greet(name, greeting: 'Hello')
      greeting + ', ' + name
    end

    greet('Bob', greeting: 'Hi')
    ";
    let binary = mrbc_compile("basic_kargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(&result_str, "Hi, Bob");
}

#[test]
fn multiple_keyword_args_test() {
    let code = "
    def destruct_it(x, foo: 42, bar: 99)
      x + foo + bar
    end

    destruct_it(10, foo: 20, bar: 30)
    ";
    let binary = mrbc_compile("multiple_kargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 10 + 20 + 30);
}

#[test]
fn keyword_args_string_symbol_test() {
    let code = "
    def format_text(text, prefix: '', suffix: '')
      prefix + text + suffix
    end

    result1 = format_text('Hello')
    result2 = format_text('Hello', prefix: '>> ')
    result3 = format_text('Hello', suffix: ' <<')
    result4 = format_text('Hello', prefix: '[', suffix: ']')
    [result1, result2, result3, result4]
    ";
    let binary = mrbc_compile("string_kargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_array: Vec<Rc<RObject>> = result.as_ref().try_into().unwrap();
    let mut expected_array = vec!["Hello", ">> Hello", "Hello <<", "[Hello]"];
    for obj in result_array {
        let s: String = obj.as_ref().try_into().unwrap();
        let expected = expected_array.remove(0);
        assert_eq!(&s, expected);
    }
}

#[test]
fn keyword_args_nested_call_test() {
    let code = "
    def inner(value, multiplier: 2)
      value * multiplier
    end

    def outer(x, factor: 3)
      result1 = inner(x)
      result2 = inner(x + 1, multiplier: factor)
      result1 + result2
    end

    [
      outer(5),
      outer(5, factor: 4)
    ]
    ";
    let binary = mrbc_compile("nested_kargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let (got1, got2) = result.as_ref().try_into().unwrap();
    assert_eq!(got1, 28); // 5 * 2 + 6 * 3
    assert_eq!(got2, 34); // 5 * 2 + 6 * 4
}

#[test]
fn keyword_args_c_definition_test() {
    fn test_mrb_multiply(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
        let a: i32 = args
            .first()
            .ok_or_else(|| Error::ArgumentError("missing positional argument 'a'".to_string()))?
            .as_ref()
            .try_into()?;
        let kwargs = vm.get_kwargs();
        match kwargs {
            Some(kargs) => {
                let b_obj = kargs.get("b").ok_or_else(|| {
                    Error::ArgumentError("missing keyword argument 'b'".to_string())
                })?;
                let c_obj = kargs.get("c").ok_or_else(|| {
                    Error::ArgumentError("missing keyword argument 'c'".to_string())
                })?;
                let b: i32 = b_obj.as_ref().try_into()?;
                let c: i32 = c_obj.as_ref().try_into()?;
                Ok(Rc::new(RObject::integer((a * b * c) as i64)))
            }
            None => Err(Error::ArgumentError(
                "missing keyword arguments".to_string(),
            )),
        }
    }

    let code = "multiply(7, b: 3, c: 11)";
    let binary = mrbc_compile("cdef_kargs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let kernel = vm.object_class.clone();
    mrb_define_cmethod(&mut vm, kernel, "multiply", Box::new(test_mrb_multiply));

    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 7 * 3 * 11);
}
