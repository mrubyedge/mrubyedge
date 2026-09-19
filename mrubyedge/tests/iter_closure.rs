extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn times_test_c() {
    let code = "
    def test_times
      i = 0
      3.times do |j|
        i += j * 2
      end
      i
    end
    ";
    let binary = mrbc_compile("times_c", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_times", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 6);
}

#[test]
fn range_each_test_c() {
    let code = "
    def test_each
      i = 0
      (1..10).each do |j|
        i += j * 2
      end
      i
    end
    ";
    let binary = mrbc_compile("range_each_c", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_each", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 110);
}

#[test]
fn array_each_test_c() {
    let code = "
    def test_each
      i = 0
      [1, 10, 100, 1000, 10000].each do |j|
        i += j * 2
      end
      i
    end
    ";
    let binary = mrbc_compile("array_each_c", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "test_each", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 22222);
}

#[test]
fn array_each_nested_test_c() {
    let code = "
    def do_times_nest
      result = 200
      3.times do |i|
        3.times do |j|
          result += 200
        end
      end
      
      result
    end
    ";
    let binary = mrbc_compile("array_each_nested_c", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "do_times_nest", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 2000);
}

#[test]
fn expired_closure_test() {
    let code = "
    def do_time_block
      result = 0
      ->(i) {
        result += 100
        puts \"result = #{result}\"
      }
    end

    def do_times
      3.times(&do_time_block)
    end
    ";
    let binary = mrbc_compile("expired_closure", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();

    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "do_times", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 3);
}

#[test]
fn cross_file_irep_ids_do_not_collide() {
    // Two blobs loaded into one VM must not reuse irep ids. Per-file numbering
    // used to collide, so a method returning from one file captured its
    // registers into another file's live block environment (the outer env of
    // the nested each in build_layer). That shrank the captured window below
    // the upvar index and panicked on the read.
    let build = "
    def build_layer
      a = 0; b = 1; c = 2; d = 3; e = 4; f = 5
      draws = []
      [1, 2].each do |x|
        foreign_return
        [3, 4].each { |arr| draws << arr }
      end
      draws.size
    end
    ";
    let foreign = "
    def foreign_return
      q = 1
      q
    end
    ";
    let mut vm = mrubyedge::yamrb::vm::VM::empty();
    for (name, code) in [("cross_file_build", build), ("cross_file_foreign", foreign)] {
        let binary = mrbc_compile(name, code);
        let mut rite = mrubyedge::rite::load(&binary).unwrap();
        vm.eval_rite(&mut rite).unwrap();
    }

    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "build_layer", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 4);
}
