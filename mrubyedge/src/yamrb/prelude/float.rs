use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::mrb_define_cmethod;

use crate::yamrb::{value::RObject, vm::VM};

pub(crate) fn initialize_float(vm: &mut VM) {
    let float_class = vm.define_standard_class("Float");
    mrb_define_cmethod(vm, float_class.clone(), "to_i", Box::new(mrb_float_to_i));
    mrb_define_cmethod(vm, float_class.clone(), "to_f", Box::new(mrb_float_to_f));
    mrb_define_cmethod(
        vm,
        float_class.clone(),
        "inspect",
        Box::new(mrb_float_inspect),
    );
    mrb_define_cmethod(vm, float_class.clone(), "to_s", Box::new(mrb_float_inspect));
}

pub fn mrb_float_to_i(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => {
            let int_value = *f as i64;
            Ok(Rc::new(RObject::integer(int_value)))
        }
        _ => Err(Error::RuntimeError(
            "Float#to_i must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_to_f(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => Ok(Rc::new(RObject::float(*f))),
        _ => Err(Error::RuntimeError(
            "Float#to_f must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => {
            let s = format!("{}", f);
            Ok(Rc::new(RObject::string(s)))
        }
        _ => Err(Error::RuntimeError(
            "Float#inspect must be called on a Float".to_string(),
        )),
    }
}
