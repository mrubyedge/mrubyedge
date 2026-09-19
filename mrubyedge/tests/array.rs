extern crate mrubyedge;

mod helpers;

use helpers::*;

#[test]
fn array_add_test() {
    let code = r#"
    def test_array_add
      [1, 2] + [3, 4]
    end
    "#;
    let binary = mrbc_compile("array_add", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_add", &args).unwrap();
    let result = result.as_vec_owned().unwrap();
    assert_eq!(result.len(), 4);
}

#[test]
fn array_push_test() {
    let code = r#"
    def test_array_push
      a = [1, 2]
      a << 3
      a
    end
    "#;
    let binary = mrbc_compile("array_push", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_push", &args).unwrap();
    let result = result.as_vec_owned().unwrap();
    assert_eq!(result.len(), 3);
}

#[test]
fn array_at_test() {
    let code = r#"
    def test_array_at
      [1, 2, 3].at(1)
    end
    "#;
    let binary = mrbc_compile("array_at", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_at", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn array_negative_index_test() {
    let code = r#"
    def test_array_negative_index
      arr = [1, 2, 3]
      [arr[-1], arr[-2], arr[-3]]
    end
    "#;
    let binary = mrbc_compile("array_negative_index", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_negative_index", &args).unwrap();
    let result: (i32, i32, i32) = result.as_ref().try_into().unwrap();
    assert_eq!(result, (3, 2, 1));
}

#[test]
fn array_set_negative_index_test() {
    let code = r#"
    def test_array_set_negative_index
      arr = [1, 2, 3]
      arr[-1] = 4
      arr[-2] = 5
      arr[-3] = 6
      arr
    end
    "#;
    let binary = mrbc_compile("array_set_negative_index", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_set_negative_index", &args).unwrap();
    let result: (i32, i32, i32) = result.as_ref().try_into().unwrap();
    assert_eq!(result, (6, 5, 4));
}

#[test]
fn array_clear_test() {
    let code = r#"
    def test_array_clear
      a = [1, 2, 3]
      a.clear
      a.size
    end
    "#;
    let binary = mrbc_compile("array_clear", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_clear", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 0);
}

#[test]
fn array_delete_at_test() {
    let code = r#"
    def test_array_delete_at
      a = [1, 2, 3]
      a.delete_at(1)
    end
    "#;
    let binary = mrbc_compile("array_delete_at", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_delete_at", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn array_empty_test() {
    let code = r#"
    def test_array_empty
      [].empty?
    end
    "#;
    let binary = mrbc_compile("array_empty", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_empty", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn array_include_test() {
    let code = r#"
    def test_array_include
      [1, 2, 3].include?(2)
    end
    "#;
    let binary = mrbc_compile("array_include", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_include", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn array_and_test() {
    let code = r#"
    def test_array_and
      ([1, 2, 3] & [2, 3, 4]).size
    end
    "#;
    let binary = mrbc_compile("array_and", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_and", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 2);
}

#[test]
fn array_or_test() {
    let code = r#"
    def test_array_or
      ([1, 2] | [2, 3]).size
    end
    "#;
    let binary = mrbc_compile("array_or", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_or", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_first_test() {
    let code = r#"
    def test_array_first
      [1, 2, 3].first
    end
    "#;
    let binary = mrbc_compile("array_first", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_first", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_last_test() {
    let code = r#"
    def test_array_last
      [1, 2, 3].last
    end
    "#;
    let binary = mrbc_compile("array_last", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_last", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_pop_test() {
    let code = r#"
    def test_array_pop
      a = [1, 2, 3]
      a.pop
    end
    "#;
    let binary = mrbc_compile("array_pop", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_pop", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_shift_test() {
    let code = r#"
    def test_array_shift
      a = [1, 2, 3]
      a.shift
    end
    "#;
    let binary = mrbc_compile("array_shift", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_shift", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_unshift_test() {
    let code = r#"
    def test_array_unshift
      a = [2, 3]
      a.unshift(1)
      a.first
    end
    "#;
    let binary = mrbc_compile("array_unshift", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_unshift", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1);
}

#[test]
fn array_dup_test() {
    let code = r#"
    def test_array_dup
      a = [1, 2, 3]
      b = a.dup
      b == a
    end
    "#;
    let binary = mrbc_compile("array_dup", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_dup", &args).unwrap();
    let result: bool = result.as_ref().try_into().unwrap();
    assert!(result);
}

#[test]
fn array_uniq_self_test() {
    let code = r#"
    def test_array_uniq_self
      a = [1, 2, 2, 3]
      a.uniq!
      a.size
    end
    "#;
    let binary = mrbc_compile("array_uniq_self", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_uniq_self", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_join_test() {
    let code = r#"
    def test_array_join
      [1, 2, 3].join(",")
    end
    "#;
    let binary = mrbc_compile("array_join", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_join", &args).unwrap();
    let result: String = result.as_ref().try_into().unwrap();
    assert_eq!(result, "1,2,3");
}

#[test]
fn array_reference_mutation_test() {
    let code = r#"
    def incr(times, state)
      return state if times == 0
      state[0] += 1
      incr(times - 1, state)
    end
    
    def test_array_reference
      arr = [0]
      incr(3, arr)
      arr[0]
    end
    "#;
    let binary = mrbc_compile("array_reference_mutation", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_array_reference", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn array_reference_mutation_recursive_test() {
    let code = r#"
    def incr_recursive(times, state, results)
      return results if times == 0
      state[0] += 1
      results << state[0]
      incr_recursive(times - 1, state, results)
    end
    
    def test_recursive_mutation
      arr = [0]
      results = []
      incr_recursive(5, arr, results)
      [arr[0], results]
    end
    "#;
    let binary = mrbc_compile("array_reference_recursive", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_recursive_mutation", &args).unwrap();
    let outer: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    // arr[0] should be 5
    let final_count: i64 = outer[0].as_ref().try_into().unwrap();
    assert_eq!(final_count, 5);

    // results should be [1, 2, 3, 4, 5]
    let results: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        outer[1].as_ref().try_into().unwrap();
    assert_eq!(results.len(), 5);

    for (i, item) in results.iter().enumerate() {
        let val: i64 = item.as_ref().try_into().unwrap();
        assert_eq!(val, (i + 1) as i64);
    }
}

#[test]
fn array_flatten_basic_test() {
    let code = r#"
    def test_flatten_basic
      [1, 2, [3, 4]].flatten
    end
    "#;
    let binary = mrbc_compile("flatten_basic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_basic", &args).unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    assert_eq!(arr.len(), 4);
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![1, 2, 3, 4]);
}

#[test]
fn array_flatten_nested_test() {
    let code = r#"
    def test_flatten_nested
      [1, [2, [3, 4]], 5].flatten
    end
    "#;
    let binary = mrbc_compile("flatten_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_nested", &args).unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    assert_eq!(arr.len(), 5);
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![1, 2, 3, 4, 5]);
}

#[test]
fn array_flatten_empty_test() {
    let code = r#"
    def test_flatten_empty
      [].flatten
    end
    "#;
    let binary = mrbc_compile("flatten_empty", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_empty", &args).unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    assert_eq!(arr.len(), 0);
}

#[test]
fn array_flatten_no_nested_test() {
    let code = r#"
    def test_flatten_no_nested
      [1, 2, 3].flatten
    end
    "#;
    let binary = mrbc_compile("flatten_no_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_no_nested", &args).unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    assert_eq!(arr.len(), 3);
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![1, 2, 3]);
}

#[test]
fn array_flatten_self_basic_test() {
    let code = r#"
    def test_flatten_self_basic
      a = [1, 2, [3, 4]]
      a.flatten!
      a
    end
    "#;
    let binary = mrbc_compile("flatten_self_basic", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_self_basic", &args).unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    assert_eq!(arr.len(), 4);
    let vals: Vec<i64> = arr.iter().map(|r| r.as_ref().try_into().unwrap()).collect();
    assert_eq!(vals, vec![1, 2, 3, 4]);
}

#[test]
fn array_flatten_self_returns_nil_if_no_change_test() {
    let code = r#"
    def test_flatten_self_no_change
      a = [1, 2, 3]
      a.flatten!
    end
    "#;
    let binary = mrbc_compile("flatten_self_no_change", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_self_no_change", &args).unwrap();

    // Should return nil if no changes were made
    assert!(result.as_ref().is_nil());
}

#[test]
fn array_flatten_self_returns_self_if_changed_test() {
    let code = r#"
    def test_flatten_self_changed
      a = [1, [2], 3]
      result = a.flatten!
      [result, a]
    end
    "#;
    let binary = mrbc_compile("flatten_self_changed", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_flatten_self_changed", &args).unwrap();
    let outer: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();

    // result (outer[0]) should be the same as a (outer[1])
    let result_arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        outer[0].as_ref().try_into().unwrap();
    let a_arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        outer[1].as_ref().try_into().unwrap();

    assert_eq!(result_arr.len(), 3);
    assert_eq!(a_arr.len(), 3);

    let result_vals: Vec<i64> = result_arr
        .iter()
        .map(|r| r.as_ref().try_into().unwrap())
        .collect();
    let a_vals: Vec<i64> = a_arr
        .iter()
        .map(|r| r.as_ref().try_into().unwrap())
        .collect();

    assert_eq!(result_vals, vec![1, 2, 3]);
    assert_eq!(a_vals, vec![1, 2, 3]);
}

#[test]
fn array_literal_over_stack_flush_test() {
    // Literals larger than the OP_ARRAY2 operand range are built in batches and
    // completed with OP_ARYPUSH, which upstream leaves unimplemented.
    let code = r#"
    def test_big_literal
      a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144]
      [a.size, a[0], a[72], a[143]]
    end
    "#;
    let binary = mrbc_compile("big_literal", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_big_literal", &args).unwrap();
    let outer: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let size: i64 = outer[0].as_ref().try_into().unwrap();
    let first: i64 = outer[1].as_ref().try_into().unwrap();
    let middle: i64 = outer[2].as_ref().try_into().unwrap();
    let last: i64 = outer[3].as_ref().try_into().unwrap();
    assert_eq!((size, first, middle, last), (144, 1, 73, 144));
}
