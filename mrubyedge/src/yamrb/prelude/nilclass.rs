use std::rc::Rc;

use crate::yamrb::helpers::mrb_define_cmethod;
use crate::Error;

use crate::yamrb::{value::RObject, vm::VM};

pub(crate) fn initialize_nilclass(vm: &mut VM) {
    let nilclass = vm.define_standard_class("NilClass");

    mrb_define_cmethod(vm, nilclass.clone(), "to_s", Box::new(mrb_nilclass_to_s));
    mrb_define_cmethod(vm, nilclass.clone(), "inspect", Box::new(mrb_nilclass_inspect));
    mrb_define_cmethod(vm, nilclass.clone(), "nil?", Box::new(mrb_nilclass_nil_p));
}

fn mrb_nilclass_to_s(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string("".to_string())))
}

fn mrb_nilclass_inspect(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string("nil".to_string())))
}

fn mrb_nilclass_nil_p(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::boolean(true)))
}
