use std::rc::Rc;

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_call_block, mrb_define_class_cmethod, mrb_define_cmethod},
        value::*,
        vm::{Breadcrumb, VM},
    },
};

pub(crate) fn initialize_proc(vm: &mut VM) {
    let proc_class = vm.define_standard_class("Proc");

    mrb_define_class_cmethod(vm, proc_class.clone(), "new", Box::new(mrb_proc_new));

    mrb_define_cmethod(vm, proc_class.clone(), "call", Box::new(mrb_proc_call));
}

fn mrb_proc_new(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let block = args[0].clone();
    Ok(block)
}

pub fn mrb_proc_call(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    // handle Proc#call as special
    let cur = vm
        .current_breadcrumb
        .take()
        .expect("empty breadcrumb on call");
    let new_breadcrumb = Rc::new(Breadcrumb {
        upper: cur.upper.clone(),
        caller: Some("Proc#call".to_string()),
        event: "_proc_call_via_method",
        return_reg: cur.return_reg,
    });
    vm.current_breadcrumb.replace(new_breadcrumb);

    let this = vm.getself()?;
    mrb_call_block(vm, this.clone(), None, args, 0)
}
