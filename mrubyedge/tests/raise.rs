extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn raise_test() {
    let code = "
    def test_raise
      raise \"Intentional Error\"
    end
    ";
    let binary = mrbc_compile("raise", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_raise", &args).err();
    assert_eq!(&result.unwrap().message(), "Intentional Error");
}

#[test]
fn raise_nest_test() {
    let code = "
    def do_raise
      raise \"Intentional Error 2\"
      p :HOGE
    end

    def test_raise
      do_raise
      p :NG
    end
    ";
    let binary = mrbc_compile("raise_nest", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_raise", &args).err();
    assert_eq!(&result.unwrap().message(), "Intentional Error 2");
}

#[test]
fn raise_nest_test_toplevel() {
    let code = "
    def do_raise
      raise \"Intentional Error 0\"
      p :HOGE
    end

    def shim
      do_raise
      p :NG_NG
    end

    shim
    p :NG
    ";
    let binary = mrbc_compile("raise_nest_toplevel", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);

    // Assert
    let result = vm.run().err();
    assert_eq!(
        &result
            .unwrap()
            .downcast_ref::<mrubyedge::Error>()
            .unwrap()
            .message(),
        "Intentional Error 0",
    );
}

#[test]
fn raise_nest_nest_test() {
    let code = "
    def do_raise
      raise \"Intentional Error 2b\"
      p :HOGE
    end

    def shim
      do_raise
      p :NG_1
    end

    def test_raise
      shim
      p :NG_2
    end
    ";
    let binary = mrbc_compile("raise_nest_nest", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_raise", &args).err();
    assert_eq!(&result.unwrap().message(), "Intentional Error 2b");
}

#[test]
fn rescue_test() {
    let code = "
    def test_raise
      begin
        raise \"Intentional Error 3\"
      rescue => e
        puts \"rescue: #{e.message}\"
        \"rescue: #{e.message}\"
      end
    end
    ";
    let binary = mrbc_compile("rescue", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: String = mrb_funcall(&mut vm, None, "test_raise", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(&result, "rescue: Intentional Error 3");
}

#[test]
fn rescue_nest_test() {
    let code = "
    def test_raise
      raise \"Intentional Error 4\"
    end

    def test_raise_parent
      begin
        test_raise
        \"NG\"
      rescue => e
        puts \"rescue: #{e.message}\"
        \"rescue: #{e.message}\"
      end
    end
    ";
    let binary = mrbc_compile("rescue_nest", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: String = mrb_funcall(&mut vm, None, "test_raise_parent", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(&result, "rescue: Intentional Error 4");
}

#[test]
fn rescue_nest_nest_test() {
    let code = "
    def test_raise
      raise \"Intentional Error 4b\"
    end

    def shim
      test_raise
      \"NG_1\"
    end

    def test_raise_parent
      begin
        shim
        \"NG_2\"
      rescue => e
        puts \"rescue: #{e.message}\"
        \"rescue: #{e.message}\"
      end
    end
    ";
    let binary = mrbc_compile("rescue_nest_nest", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: String = mrb_funcall(&mut vm, None, "test_raise_parent", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(&result, "rescue: Intentional Error 4b");
}

// A bare `rescue => e` matches StandardError via is_a?, which is exactly
// what op_rescue checks; NoMatchingPatternError must answer true to it.
#[test]
fn no_matching_pattern_error_is_a_standard_error_test() {
    let code = "
    def test_no_matching_pattern_error
      NoMatchingPatternError.new(\"boom\").is_a?(StandardError)
    end
    ";
    let binary = mrbc_compile("no_matching_pattern_error", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result = mrb_funcall(&mut vm, None, "test_no_matching_pattern_error", &args).unwrap();
    let is_standard_error: bool = result.as_ref().try_into().unwrap();
    assert!(is_standard_error);
}

#[test]
fn raise_user_defined_class_test() {
    let code = "
    class MyError < StandardError
    end

    def test_raise_class
      begin
        raise MyError, \"custom boom\"
      rescue MyError => e
        e.message
      end
    end
    ";
    let binary = mrbc_compile("raise_class", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    let args = vec![];
    let result: String = mrb_funcall(&mut vm, None, "test_raise_class", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert!(result.contains("custom boom"));
}
