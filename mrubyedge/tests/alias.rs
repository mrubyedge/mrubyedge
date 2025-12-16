extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn alias_test() {
    let code = "
    class Hello
      def sample
        42
      end
      alias alias_sample sample
    end
    def test_main
      w = Hello.new
      w.alias_sample
    end
    ";
    let binary = mrbc_compile("alias", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_main", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 42);
}

#[test]
fn undef_test() {
    let code = "
    class Hello
      def sample
        \"Hola\"
      end
      alias alias_sample sample
      undef sample
    end
    def test_main_1
      w = Hello.new
      w.alias_sample
    end
    def test_main_2
      w = Hello.new
      begin
        w.sample
      rescue NoMethodError => e
        e.message
      end
    end
    ";
    let binary = mrbc_compile("undef", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: String = mrb_funcall(&mut vm, None, "test_main_1", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, "Hola");

    let result: String = mrb_funcall(&mut vm, None, "test_main_2", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert!(result.contains("Method not found: sample"));
}
