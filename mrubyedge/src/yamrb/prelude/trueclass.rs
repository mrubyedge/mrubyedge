use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::mrb_define_cmethod;

use crate::yamrb::{value::RObject, vm::VM};

pub(crate) fn initialize_trueclass(vm: &mut VM) {
    let trueclass = vm.define_standard_class("TrueClass");

    mrb_define_cmethod(vm, trueclass.clone(), "to_s", Box::new(mrb_trueclass_to_s));
    mrb_define_cmethod(
        vm,
        trueclass.clone(),
        "inspect",
        Box::new(mrb_trueclass_inspect),
    );
    mrb_define_cmethod(vm, trueclass.clone(), "&", Box::new(mrb_trueclass_and));
    mrb_define_cmethod(vm, trueclass.clone(), "|", Box::new(mrb_trueclass_or));
    mrb_define_cmethod(vm, trueclass.clone(), "^", Box::new(mrb_trueclass_xor));
}

fn mrb_trueclass_to_s(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string("true".to_string())))
}

fn mrb_trueclass_inspect(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string("true".to_string())))
}

fn mrb_trueclass_and(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let rhs = args[0].clone();
    Ok(Rc::new(RObject::boolean(rhs.is_truthy())))
}

fn mrb_trueclass_or(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::boolean(true)))
}

fn mrb_trueclass_xor(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let rhs = args[0].clone();
    Ok(Rc::new(RObject::boolean(!rhs.is_truthy())))
}
