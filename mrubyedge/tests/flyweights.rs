extern crate mrubyedge;

mod helpers;
use helpers::*;

#[test]
fn flyweights_reset_between_vms_on_the_same_thread() {
    // VM 1 plants state on the shared immediates: @leak on nil via a
    // NilClass method (op_setiv with self = the shared nil instance) and
    // @mark on a small integer.
    let code1 = r#"
    class NilClass
      def plant
        @leak = 1
      end
    end
    class Integer
      def plant
        @mark = 2
      end
    end
    nil.plant
    7.plant
    "#;
    let binary1 = mrbc_compile("flyweight_plant", code1);
    let mut rite1 = mrubyedge::rite::load(&binary1).unwrap();
    let mut vm1 = mrubyedge::yamrb::vm::VM::open(&mut rite1);
    vm1.run().unwrap();

    // VM 2 runs on the same thread and must not see either value.
    let code2 = r#"
    class NilClass
      def read
        @leak
      end
    end
    class Integer
      def read
        @mark
      end
    end
    [nil.read, 7.read]
    "#;
    let binary2 = mrbc_compile("flyweight_read", code2);
    let mut rite2 = mrubyedge::rite::load(&binary2).unwrap();
    let mut vm2 = mrubyedge::yamrb::vm::VM::open(&mut rite2);
    let result = vm2.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    assert!(
        arr[0].as_ref().is_nil(),
        "ivar planted on nil must not leak into a later VM"
    );
    assert!(
        arr[1].as_ref().is_nil(),
        "ivar planted on a small integer must not leak into a later VM"
    );
}

#[test]
fn flyweights_shared_within_one_vm() {
    // Within a single VM the instances stay shared: identity holds for the
    // whole run.
    let code = r#"
    a = nil
    b = nil
    [a.object_id == b.object_id, 5.object_id == 5.object_id]
    "#;
    let binary = mrbc_compile("flyweight_identity", code);
    let mut rite = mrubyedge::rite::load(&binary).unwrap();
    let mut vm = mrubyedge::yamrb::vm::VM::open(&mut rite);
    let result = vm.run().unwrap();
    let arr: Vec<std::rc::Rc<mrubyedge::yamrb::value::RObject>> =
        result.as_ref().try_into().unwrap();
    let nil_same: bool = arr[0].as_ref().try_into().unwrap();
    let int_same: bool = arr[1].as_ref().try_into().unwrap();
    assert!(nil_same);
    assert!(int_same);
}
