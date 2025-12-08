use std::rc::Rc;

use crate::{yamrb::{value::*, vm::VM}, Error};

pub(crate) fn initialize_module(vm: &mut VM) {
    let _module_class = vm.define_standard_class("Module");
}
