extern crate mec_mrbc_sys;
extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn return_block_simple() {
    let code = "
    def outer
      inner do
        return 5471
      end
      :unreachable
    end

    def inner
      yield
      return :unreachable
    end
    ";
    let binary = mrbc_compile("return_block_simple", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5471);
}

#[test]
fn return_block_with_c_func() {
    let code = "
    def outer2
      1.times do
        return 5472
      end
      :unreachable
    end
    ";
    let binary = mrbc_compile("return_block_with_c_func", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer2", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5472);
}

#[test]
fn return_block_nested() {
    let code = "
    def outer
      inner do
        inner do
          return 5473
        end
      end
      :unreachable
    end

    def inner
      yield
      :unreachable
    end
    ";
    let binary = mrbc_compile("return_block_nested", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5473);
}

#[test]
fn return_block_nested_with_c_func() {
    let code = "
    def outer2
      1.times do
        inner do
          return 5474
        end
      end
      :unreachable
    end

    def inner
      yield
      :unreachable
    end
    ";
    let binary = mrbc_compile("return_block_nested_with_c_func", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer2", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5474);
}

#[test]
fn return_block_nested_each_times() {
    let code = "
    def outer3
      k = 0
      [0, 1, 2].each do |i|
        k += i
        4.times do |j|
          k += j
          return k if k > 10
        end
      end
      9999
    end
    ";
    let binary = mrbc_compile("return_block_nested_each_times", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer3", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    // k = 0+0 = 0, then 0+0, 0+1, 0+2, 0+3 = 6, then 6+1 = 7, then 7+0, 7+1, 7+2, 7+3 = 13 > 10
    assert_eq!(result, 13);
}

#[test]
fn return_block_c_func_in_yield() {
    let code = "
    def outer4
      inner do
        1.times do
          return 5475
        end
      end
      :unreachable
    end

    def inner
      yield
      :unreachable
    end
    ";
    let binary = mrbc_compile("return_block_c_func_in_yield", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer4", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5475);
}

#[test]
fn return_block_deeply_nested_c_func() {
    let code = "
    def outer5
      inner do
        1.times do
          1.times do
            return 5476
          end
        end
      end
      :unreachable
    end

    def inner
      yield
      :unreachable
    end
    ";
    let binary = mrbc_compile("return_block_deeply_nested_c_func", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    vm.run().unwrap();
    // Assert
    let args = vec![];
    let result: i32 = mrb_funcall(&mut vm, None, "outer5", &args)
        .unwrap()
        .as_ref()
        .try_into()
        .unwrap();
    assert_eq!(result, 5476);
}
