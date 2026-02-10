extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn default_arg_array_reused() {
    let code = r#"
def incr(times, state=[0])
  return state if times == 0
  state[0] += 1
  incr(times - 1, state)
end

result = incr(3)
result[0]
    "#;
    let binary = mrbc_compile("default_arg_array_reused", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 3);
}

#[test]
fn default_arg_simple() {
    let code = r##"
def greet(name, greeting="Hello")
  "#{greeting}, #{name}!"
end

greet("Alice")
    "##;
    let binary = mrbc_compile("default_arg_simple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "Hello, Alice!");
}

#[test]
fn default_arg_multiple() {
    let code = r#"
def create_point(x=0, y=3, z=6)
  [x, y, z]
end

result = create_point()
result[0] + result[1] + result[2]
    "#;
    let binary = mrbc_compile("default_arg_multiple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 9);
}

#[test]
fn default_arg_override() {
    let code = r#"
def add(a, b=10)
  a + b
end

add(5, 20)
    "#;
    let binary = mrbc_compile("default_arg_override", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 25);
}

#[test]
fn default_arg_partial_override() {
    let code = r#"
def calc(a, b=2, c=3)
  a * b + c
end

calc(5, 4)
    "#;
    let binary = mrbc_compile("default_arg_partial_override", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 23);
}

#[test]
fn default_arg_hash_reused() {
    let code = r#"
def update_counter(times, state={count: 0, other: 2})
  return state if times == 0
  state[:count] += 1
  update_counter(times - 1, state)
end

result = update_counter(5)
result[:count] + result[:other]
    "#;
    let binary = mrbc_compile("default_arg_hash_reused", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_int: i32 = result.as_ref().try_into().unwrap();
    assert_eq!(result_int, 5 + 2);
}

#[test]
fn default_arg_mixed_types() {
    let code = r##"
def format_message(msg, prefix="Info", level=1, enabled=true)
  return "disabled" unless enabled
  "[#{prefix}:#{level}] #{msg}"
end

format_message("Test")
    "##;
    let binary = mrbc_compile("default_arg_mixed_types", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result_str: String = result.as_ref().try_into().unwrap();
    assert_eq!(result_str, "[Info:1] Test");
}
