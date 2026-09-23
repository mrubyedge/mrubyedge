use crate::helpers::*;
use mrubyedge::Error;

#[test]
fn matcherr_passes_when_a_pattern_matched_test() {
    let code = "
x = 1
case x
in 1
  7
end
";
    let binary = mrbc_compile("compiled", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let result: i64 = result.as_ref().try_into().unwrap();

    // Assert
    assert_eq!(result, 7);
}

#[test]
fn matcherr_raises_when_nothing_matched_test() {
    let code = "
x = 1
case x
in 2
  7
end
";
    let binary = mrbc_compile("compiled", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let err = vm.run().unwrap_err();
    let err = err.downcast_ref::<Error>().expect("a VM error");

    // Assert
    assert!(
        matches!(err, Error::TaggedError(tag, msg) if tag == "NoMatchingPatternError" && msg == "pattern not matched"),
        "{:?}",
        err
    );
}
