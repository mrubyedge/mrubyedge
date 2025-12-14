#![cfg(feature = "mruby-regexp")]
extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;

use helpers::*;

#[test]
fn regexp_match_operator_test() {
    let code = r#"
    def test_regexp_match
      re = /ruby/
      target = "mrubyedge"
      result = target =~ re
      result
    end
    "#;
    let binary = mrbc_compile("regexp_match", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_regexp_match", &args).unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();
    assert_eq!(result, 1); // "ruby" starts at index 1 in "mrubyedge"
}

#[test]
fn regexp_not_match_operator_test() {
    let code = r#"
    def test_regexp_not_match
      re = /ruby/
      target = "micropython"
      result = re !~ target
      result
    end
    "#;
    let binary = mrbc_compile("regexp_not_match", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_regexp_not_match", &args).unwrap();
    assert!(result.is_truthy());
}

#[test]
fn regexp_match_method_test() {
    let code = r#"
    def test_regexp_match_method
      re = /(m?ruby).*?(m?ruby).*?(m?ruby(?:ists)?)/
      target = "mruby/edge is a mruby for embedded systems, built for rubyists."
      matched = re.match(target)
      matched
    end
    "#;
    let binary = mrbc_compile("regexp_match_method", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_regexp_match_method", &args).unwrap();

    // Check that we got a match object (not nil)
    assert!(!matches!(
        &result.value,
        mrubyedge::yamrb::value::RValue::Nil
    ));
}

#[test]
fn regexp_match_captures_test() {
    let code = r#"
    def test_regexp_captures
      re = /(m?ruby).*?(m?ruby).*?(m?ruby(?:ists)?)/
      target = "mruby/edge is a mruby for embedded systems, built for rubyists."
      matched = re.match(target)
      [matched[0], matched[1], matched[2], matched[3]]
    end
    "#;
    let binary = mrbc_compile("regexp_captures", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_regexp_captures", &args).unwrap();

    // Verify it's an array
    if let mrubyedge::yamrb::value::RValue::Array(arr) = &result.value {
        assert_eq!(arr.borrow().len(), 4);

        // Check first capture (full match)
        let capture0: String = arr.borrow()[0].as_ref().try_into().unwrap();
        assert_eq!(
            capture0,
            "mruby/edge is a mruby for embedded systems, built for rubyists"
        );

        // Check first group
        let capture1: String = arr.borrow()[1].as_ref().try_into().unwrap();
        assert_eq!(capture1, "mruby");

        // Check second group
        let capture2: String = arr.borrow()[2].as_ref().try_into().unwrap();
        assert_eq!(capture2, "mruby");

        // Check third group
        let capture3: String = arr.borrow()[3].as_ref().try_into().unwrap();
        assert_eq!(capture3, "rubyists");
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn regexp_no_match_test() {
    let code = r#"
    def test_regexp_no_match
      re = /python/
      target = "mrubyedge"
      result = target =~ re
      result
    end
    "#;
    let binary = mrbc_compile("regexp_no_match", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_regexp_no_match", &args).unwrap();

    // Should return nil when no match
    assert!(matches!(
        &result.value,
        mrubyedge::yamrb::value::RValue::Nil
    ));
}
