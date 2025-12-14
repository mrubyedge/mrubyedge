use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::mrb_define_cmethod;

use crate::yamrb::{value::RObject, vm::VM};

pub(crate) fn initialize_symbol(vm: &mut VM) {
    let symbol_class = vm.define_standard_class("Symbol");
    mrb_define_cmethod(vm, symbol_class.clone(), "to_s", Box::new(mrb_symbol_to_s));
    mrb_define_cmethod(
        vm,
        symbol_class.clone(),
        "inspect",
        Box::new(mrb_symbol_inspect),
    );
}

fn mrb_symbol_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(format!(":{}", this))))
}

fn mrb_symbol_to_s(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let symbol: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(symbol)))
}
