use std::rc::Rc;

use crate::{
    Error,
    yamrb::{helpers::mrb_define_cmethod, value::*, vm::VM},
};

pub(crate) fn initialize_module(vm: &mut VM) {
    let module_class = vm.define_standard_class("Module");
    mrb_define_cmethod(
        vm,
        module_class.clone(),
        "include",
        Box::new(mrb_module_include),
    );
    mrb_define_cmethod(
        vm,
        module_class,
        "ancestors",
        Box::new(mrb_module_ancestors),
    );
}

fn mrb_module_include(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::RuntimeError(
            "Module#include expects at least one module".to_string(),
        ));
    }

    let arg0 = &args[0];
    let mixin = match &arg0.value {
        RValue::Module(module) => module.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Module#include expects module arguments".to_string(),
            ));
        }
    };

    let self_obj = vm.getself()?;
    match &self_obj.value {
        RValue::Class(klass) => mrb_include_module(klass, mixin)?,
        RValue::Module(module) => mrb_include_module(module, mixin)?,
        _ => {
            return Err(Error::RuntimeError(
                "Module#include must be called on class or module".to_string(),
            ));
        }
    };

    Ok(self_obj)
}

/// Public helper.
/// Includes `mixin` module into `target`.
pub fn mrb_include_module(target: &impl AsModule, mixin: Rc<RModule>) -> Result<(), Error> {
    let target = target.as_module();
    if Rc::ptr_eq(&target, &mixin) {
        return Err(Error::RuntimeError("cannot include itself".to_string()));
    }

    let already_present = {
        let modules = target.mixed_in_modules.borrow();
        modules.iter().any(|m| Rc::ptr_eq(m, &mixin))
    };

    if already_present {
        return Err(Error::RuntimeError("module already included".to_string()));
    }

    target.mixed_in_modules.borrow_mut().insert(0, mixin);
    Ok(())
}

fn mrb_module_ancestors(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_module = vm.getself()?;
    let target_module = match &self_module.value {
        RValue::Module(module) => module.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Module#ancestors must be called on class or module".to_string(),
            ));
        }
    };
    let ancestors: Vec<Rc<RObject>> = build_module_lookup_chain(&target_module)
        .iter()
        .map(|m| RObject::module(m.clone()).to_refcount_assigned())
        .collect();
    Ok(RObject::array(ancestors).to_refcount_assigned())
}
