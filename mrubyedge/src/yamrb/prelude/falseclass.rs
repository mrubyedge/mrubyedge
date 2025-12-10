use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::mrb_define_cmethod;

use crate::yamrb::{value::RObject, vm::VM};

pub(crate) fn initialize_falseclass(vm: &mut VM) {
    let falseclass = vm.define_standard_class("FalseClass");

    mrb_define_cmethod(
        vm,
        falseclass.clone(),
        "to_s",
        Box::new(mrb_falseclass_to_s),
    );
    mrb_define_cmethod(
        vm,
        falseclass.clone(),
        "inspect",
        Box::new(mrb_falseclass_inspect),
    );
    mrb_define_cmethod(vm, falseclass.clone(), "&", Box::new(mrb_falseclass_and));
    mrb_define_cmethod(vm, falseclass.clone(), "|", Box::new(mrb_falseclass_or));
    mrb_define_cmethod(vm, falseclass.clone(), "^", Box::new(mrb_falseclass_xor));
}

fn mrb_falseclass_to_s(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string("false".to_string())))
}

fn mrb_falseclass_inspect(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string("false".to_string())))
}

fn mrb_falseclass_and(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::boolean(false)))
}

fn mrb_falseclass_or(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let rhs = args[0].clone();
    Ok(Rc::new(RObject::boolean(rhs.is_truthy())))
}

fn mrb_falseclass_xor(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let rhs = args[0].clone();
    Ok(Rc::new(RObject::boolean(rhs.is_truthy())))
}
