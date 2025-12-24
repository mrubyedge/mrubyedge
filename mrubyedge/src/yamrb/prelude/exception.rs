use std::rc::Rc;

use crate::{
    Error,
    yamrb::{helpers::mrb_define_cmethod, value::*, vm::VM},
};

pub(crate) fn initialize_exception(vm: &mut VM) {
    let exp_class: Rc<RClass> = vm.define_standard_class("Exception");
    let _ = vm.define_standard_class_with_superclass("InternalError", exp_class.clone());

    // fill in ruby's standard exceptions:
    let std_exp_class: Rc<RClass> =
        vm.define_standard_class_with_superclass("StandardError", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("RuntimeError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("TypeError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("ArgumentError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("NoMemoryError", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("ScriptError", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("LoadError", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("NotImplementedError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("SyntaxError", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("SecurityError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("SignalException", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("Interrupt", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("SystemExit", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("SystemStackError", exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("SystemCallError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("NoMethodError", std_exp_class.clone());
    let _ = vm.define_standard_class_with_superclass("NameError", std_exp_class.clone());

    // Dummy class for 'break' control flow
    let _ = vm.define_standard_class("_Break");

    mrb_define_cmethod(vm, exp_class, "message", Box::new(mrb_exception_message));
}

pub fn mrb_exception_message(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let exp = vm.getself()?;
    match &exp.value {
        RValue::Exception(e) => {
            let message = e.as_ref().message.clone();
            Ok(RObject::string(message).to_refcount_assigned())
        }
        _ => Err(Error::RuntimeError(
            "Exception#message must be called on an Exception".to_string(),
        )),
    }
}
