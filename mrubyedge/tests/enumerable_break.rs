extern crate mrubyedge;

mod helpers;
use helpers::*;

// Every Enumerable builtin that yields a block returns the break value,
// discarding any partial accumulation (Ruby semantics).

#[test]
fn enum_map_break_returns_value() {
    let code = r#"
[1, 2, 3].map { |x| break x * 10 if x == 2 }
"#;
    let binary = mrbc_compile("enum_break_map", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().expect("map break integer");
    assert_eq!(v, 20);
}

#[test]
fn enum_select_break_returns_value() {
    let code = r#"
r = [1, 2, 3].select { |x| break :stop if x == 2 }
r
"#;
    let binary = mrbc_compile("enum_break_select", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: String = result.as_ref().try_into().expect("select break symbol");
    assert_eq!(v, "stop");
}

#[test]
fn enum_find_break_returns_value() {
    let code = r#"
[1, 2, 3].find { |x| break 99 if x == 1 }
"#;
    let binary = mrbc_compile("enum_break_find", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().expect("find break integer");
    assert_eq!(v, 99);
}

#[test]
fn enum_all_p_break_returns_value() {
    // all? short-circuits on the first falsy block result, so the break must
    // fire before any falsy iteration (x == 1 here).
    let code = r#"
[1, 2, 3].all? { |x| break "no" if x == 1 }
"#;
    let binary = mrbc_compile("enum_break_all", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: String = result.as_ref().try_into().expect("all? break string");
    assert_eq!(v, "no");
}

#[test]
fn enum_any_p_break_returns_value() {
    let code = r#"
[1, 2, 3].any? { |x| break 7 if x == 1 }
"#;
    let binary = mrbc_compile("enum_break_any", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().expect("any? break integer");
    assert_eq!(v, 7);
}

#[test]
fn enum_delete_if_break_returns_value() {
    let code = r#"
[1, 2, 3].delete_if { |x| break :halt if x == 2 }
"#;
    let binary = mrbc_compile("enum_break_delete_if", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: String = result.as_ref().try_into().expect("delete_if break symbol");
    assert_eq!(v, "halt");
}

#[test]
fn enum_each_with_index_break_returns_index() {
    let code = r#"
[10, 20, 30].each_with_index { |elem, i| break i if elem == 20 }
"#;
    let binary = mrbc_compile("enum_break_ewi", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result
        .as_ref()
        .try_into()
        .expect("each_with_index break index");
    assert_eq!(v, 1);
}

#[test]
fn enum_sort_by_break_returns_value() {
    let code = r#"
[3, 1, 2].sort_by { |x| break :sorted_off if x == 1 }
"#;
    let binary = mrbc_compile("enum_break_sort_by", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: String = result.as_ref().try_into().expect("sort_by break symbol");
    assert_eq!(v, "sorted_off");
}

#[test]
fn enum_reduce_break_returns_value() {
    let code = r#"
[1, 2, 3].reduce(0) { |acc, x| break -1 if x == 2 }
"#;
    let binary = mrbc_compile("enum_break_reduce", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().expect("reduce break integer");
    assert_eq!(v, -1);
}

#[test]
fn enum_count_break_returns_value() {
    let code = r#"
[1, 2, 3].count { |x| break 5 if x == 1 }
"#;
    let binary = mrbc_compile("enum_break_count", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let v: i64 = result.as_ref().try_into().expect("count break integer");
    assert_eq!(v, 5);
}

#[test]
fn enum_times_wrapped_break_still_works() {
    // Plain iterator break (no Enumerable) keeps working.
    let code = r#"
n = 0
10.times { |i| n = i; break i * 2 if i >= 4 }
[n, 10.times { |i| break i if i >= 3 }]
"#;
    let binary = mrbc_compile("enum_break_plain_guard", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().expect("array result");
    let n: i64 = arr[0].as_ref().try_into().unwrap();
    let t: i64 = arr[1].as_ref().try_into().unwrap();
    assert_eq!(n, 4);
    assert_eq!(t, 3);
}
