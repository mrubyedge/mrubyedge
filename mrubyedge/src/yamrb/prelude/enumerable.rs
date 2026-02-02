use std::{cell::Cell, rc::Rc};

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_call_block, mrb_define_module_cmethod, mrb_funcall},
        value::{RFn, RObject, RProc},
        vm::VM,
    },
};

pub(crate) fn initialize_enumerable(vm: &mut VM) {
    let enumerable_module = vm.define_module("Enumerable", None);

    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "map",
        Box::new(mrb_enumerable_map),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "find",
        Box::new(mrb_enumerable_find),
    );
}

fn rproc_from_rust_block(vm: &mut VM, rfn: RFn) -> Result<Rc<RObject>, Error> {
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

// Enumerable#map: Returns a new array with the results of running block once for every element
fn mrb_enumerable_map(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let results: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let results_ref = results.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        mrb_funcall(
            vm,
            Some(results_ref.clone()),
            "push",
            std::slice::from_ref(&result),
        )?;
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(results)
}

// Enumerable#find: Returns the first element for which the block returns true
fn mrb_enumerable_find(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let found = Cell::new(false);
    let result_box: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let result_box_ref = result_box.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        if found.get() {
            return Ok(Rc::new(RObject::nil()));
        }

        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        if result.is_truthy() {
            mrb_funcall(
                vm,
                Some(result_box_ref.clone()),
                "push",
                std::slice::from_ref(&args[0]),
            )?;
            found.set(true);
        }
        Ok(result)
    });
    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    let found = mrb_funcall(vm, result_box.into(), "pop", &[])?;
    Ok(found)
}
