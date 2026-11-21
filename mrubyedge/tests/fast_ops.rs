extern crate mrubyedge;

mod helpers;
use helpers::*;

// NOTE: expected values here mirror the EXISTING native implementations
// (e.g. Integer#% is Rust's truncated i64 remainder, not floored modulo);
// the fast path must be indistinguishable from the natives it replaces.

#[test]
fn fast_int_mod_matches_native_test() {
    let code = "
    [(-10 % 3), (10 % -3), (10 % 3), (0 % 5)]
    ";
    let binary = mrbc_compile("fast_int_mod", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    // truncated (Rust i64) remainder, same as the native mrb_integer_mod
    assert_eq!(vals, vec![-1, 1, 1, 0]);
}

#[test]
fn fast_int_pow_matches_native_test() {
    let code = "
    [(2 ** 8), (2 ** 0), (2 ** 62), ((2 ** 8).class == Integer)]
    ";
    let binary = mrbc_compile("fast_int_pow", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let i: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(i, 256);
    let i: i64 = arr[1].as_ref().try_into().unwrap();
    assert_eq!(i, 1);
    let i: i64 = arr[2].as_ref().try_into().unwrap();
    assert_eq!(i, 1 << 62);
    let b: bool = arr[3].as_ref().try_into().unwrap();
    assert!(b, "Integer ** Integer must stay Integer");
}

#[test]
fn fast_int_pow_promotes_to_float_test() {
    let code = "
    [(2 ** -1), (2 ** 1.5)]
    ";
    let binary = mrbc_compile("fast_int_pow_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<f64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![0.5, 2f64.powf(1.5)]);
}

#[test]
fn fast_float_pow_matches_native_test() {
    let code = "
    [(2.0 ** 3.0), (2.0 ** 3), (9.0 ** 0.5), ((2.0 ** 3).class == Float)]
    ";
    let binary = mrbc_compile("fast_float_pow", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<f64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![8.0, 8.0, 3.0, 1.0]);
}

#[test]
fn fast_spaceship_matches_native_test() {
    let code = "
    [(1 <=> 2), (2 <=> 1), (2 <=> 2),
     (1.5 <=> 2.5), (2 <=> 2.0), (1 <=> 1.5), (2.5 <=> 2)]
    ";
    let binary = mrbc_compile("fast_spaceship", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![-1, 1, 0, -1, 0, -1, 1]);
}

#[test]
fn fast_ne_matches_native_test() {
    let code = "
    [(1 != 2), (1 != 1), (1.5 != 1.5), (1.5 != 2.5)]
    ";
    let binary = mrbc_compile("fast_ne", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<bool> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![true, false, false, true]);
}

#[test]
fn fast_ne_mixed_kinds_follow_structural_rule_test() {
    // Object#!= compares ValueEquality, where Integer and Float are never
    // equal across kinds: 1 != 1.0 is true even though the values match.
    // Mixed pairs are NOT fast-pathed, so the native produces them.
    let code = "
    [(1 != 1.0), (1.0 != 1), (1 != 1.5)]
    ";
    let binary = mrbc_compile("fast_ne_mixed", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<bool> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![true, true, true]);
}

#[test]
fn fast_ops_work_inside_methods_and_chains_test() {
    let code = "
    def f(x)
      y = x % 7
      y % 3
    end
    def g
      2 ** 3 ** 2
    end
    [f(20), g, (10 % 4) != 1]
    ";
    let binary = mrbc_compile("fast_ops_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let i: i64 = arr[0].as_ref().try_into().unwrap();
    assert_eq!(i, 0);
    let i: i64 = arr[1].as_ref().try_into().unwrap();
    assert_eq!(i, 512);
    let b: bool = arr[2].as_ref().try_into().unwrap();
    assert!(b);
}

#[test]
fn fast_path_hits_counter_test() {
    // The send-site cache must be warm before the fast path fires, so each
    // tagged op runs twice; the second round is served inline. The counter
    // moving proves the fast path (not the native) produced those results.
    let code = "
    def ops
      [10 % 3, 2 ** 8, 2.0 ** 3.0, 1 <=> 2, 1 != 2]
    end
    a = ops
    b = ops
    [a, b]
    ";
    let binary = mrbc_compile("fast_hits", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let before = vm.fast_native_hits.get();
    vm.run().unwrap();
    assert_eq!(vm.fast_native_hits.get() - before, 5);
}

#[test]
fn mixed_kind_ne_does_not_hit_fast_path_test() {
    let code = "1 != 1.0";
    let binary = mrbc_compile("fast_hits_mixed_ne", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let before = vm.fast_native_hits.get();
    let result = vm.run().unwrap();
    let b: bool = result.as_ref().try_into().unwrap();
    assert!(b);
    // mixed kinds fall back to the native Object#!=
    assert_eq!(vm.fast_native_hits.get() - before, 0);
}

#[test]
fn fast_op_fallthrough_non_numeric_test() {
    let code = "1 % 'a'";
    let binary = mrbc_compile("fast_fallthrough_mod_str", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    assert!(
        vm.run().is_err(),
        "1 % 'a' must keep its native TypeMismatch"
    );
}

#[test]
fn fast_pow_fallthrough_non_numeric_test() {
    let code = "1 ** 'a'";
    let binary = mrbc_compile("fast_fallthrough_pow_str", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    assert!(
        vm.run().is_err(),
        "1 ** 'a' must keep its native TypeMismatch"
    );
}

#[test]
fn fast_spaceship_fallthrough_incompatible_test() {
    let code = "1 <=> 'a'";
    let binary = mrbc_compile("fast_fallthrough_cmp_str", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    assert!(
        vm.run().is_err(),
        "1 <=> 'a' must keep its native ArgumentError"
    );
}

#[test]
fn fast_mod_float_divisor_keeps_native_coercion_test() {
    // A Float divisor is not fast-pathed; the native truncates it to i64
    // (3.5 -> 3) exactly as before.
    let code = "10 % 3.5";
    let binary = mrbc_compile("fast_fallthrough_mod_float", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let before = vm.fast_native_hits.get();
    let result = vm.run().unwrap();
    let i: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(i, 1);
    assert_eq!(vm.fast_native_hits.get() - before, 0);
}

#[test]
fn fast_ne_fallthrough_non_numeric_test() {
    let code = "['a' != 'a', 'a' != 'b', 1 != nil]";
    let binary = mrbc_compile("fast_fallthrough_ne_str", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<bool> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![false, true, true]);
}

#[test]
#[should_panic(expected = "divisor of zero")]
fn fast_mod_by_zero_behaves_as_before_test() {
    // The fast arm declines a zero divisor so the native path panics on
    // `%0` exactly as it always did.
    let code = "1 % 0";
    let binary = mrbc_compile("fast_mod_zero", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
}

#[test]
fn fast_spaceship_fallthrough_strings_test() {
    // String pairs are not numeric: they fall through to Object#<=>'s
    // String arm.
    let code = "['a' <=> 'b', 'b' <=> 'a', 'a' <=> 'a']";
    let binary = mrbc_compile("fast_fallthrough_cmp_strs", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![-1, 1, 0]);
}

#[test]
fn redefinition_kills_fast_tag_mod_test() {
    // Redefining Integer#% replaces the RProc and its FastOp tag together;
    // the send must run the Ruby method, not the inline math.
    let code = r#"
    class Integer
      def %(o)
        :redefined
      end
    end
    10 % 3
    "#;
    let binary = mrbc_compile("fast_redefine_mod", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let before = vm.fast_native_hits.get();
    let result = vm.run().unwrap();
    let sym: String = result.as_ref().try_into().unwrap();
    assert_eq!(sym, "redefined");
    assert_eq!(
        vm.fast_native_hits.get() - before,
        0,
        "redefined method must not fast-path"
    );
}

#[test]
fn redefinition_kills_fast_tag_ne_test() {
    let code = r#"
    class Object
      def !=(o)
        :nope
      end
    end
    1 != 2
    "#;
    let binary = mrbc_compile("fast_redefine_ne", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let before = vm.fast_native_hits.get();
    let result = vm.run().unwrap();
    let sym: String = result.as_ref().try_into().unwrap();
    assert_eq!(sym, "nope");
    assert_eq!(
        vm.fast_native_hits.get() - before,
        0,
        "redefined method must not fast-path"
    );
}

#[test]
fn fast_results_preserve_native_identity() {
    let code = "
    def results
      [10 % 3, 2 ** 3, 2 <=> 1, 1 != 2, 1.0 != 1.0, (2 ** 40).object_id]
    end
    [results, results]
    ";
    let binary = mrbc_compile("fast_identity", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    type Objects = Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>>;
    let rounds: Objects = result.as_ref().try_into().unwrap();
    let cold: Objects = rounds[0].as_ref().try_into().unwrap();
    let warm: Objects = rounds[1].as_ref().try_into().unwrap();
    assert_eq!(vm.fast_native_hits.get(), 6);
    for i in 0..5 {
        assert!(
            std::rc::Rc::ptr_eq(&cold[i], &warm[i]),
            "result {i} lost its shared instance"
        );
    }
    assert_eq!(cold[5].as_eq_value(), warm[5].as_eq_value());
}
