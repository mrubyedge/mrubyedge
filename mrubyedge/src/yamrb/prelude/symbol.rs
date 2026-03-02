use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::{mrb_define_cmethod, mrb_funcall};

use crate::yamrb::{
    value::{RFn, RObject, RProc},
    vm::VM,
};

pub(crate) fn initialize_symbol(vm: &mut VM) {
    let symbol_class = vm.define_standard_class("Symbol");
    mrb_define_cmethod(vm, symbol_class.clone(), "to_s", Box::new(mrb_symbol_to_s));
    mrb_define_cmethod(
        vm,
        symbol_class.clone(),
        "inspect",
        Box::new(mrb_symbol_inspect),
    );
    mrb_define_cmethod(
        vm,
        symbol_class.clone(),
        "to_proc",
        Box::new(mrb_symbol_to_proc),
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

fn mrb_symbol_to_proc(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let method_name: String = vm.getself()?.as_ref().try_into()?;
    let rfn: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let recv = args
            .first()
            .cloned()
            .ok_or_else(|| Error::ArgumentError("no receiver given".to_string()))?;
        let method_args = if args.len() > 1 { &args[1..] } else { &[] };
        mrb_funcall(vm, Some(recv), &method_name, method_args)
    });
    vm.push_fnblock(Rc::new(rfn))?;
    let block = RProc {
        is_rb_func: false,
        is_fnblock: true,
        sym_id: None,
        next: None,
        irep: None,
        func: None,
        environ: None,
        block_self: vm.getself().ok(),
    };
    Ok(RObject::proc(block).to_refcount_assigned())
}
