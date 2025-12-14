#[cfg(feature = "mruby-regexp")]
use std::rc::Rc;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use crate::{
    Error,
    yamrb::{
        helpers::mrb_define_class_cmethod,
        value::{RData, RObject, RValue},
        vm::VM,
    },
};

pub(crate) fn initialize_regexp(vm: &mut VM) {
    let regexp_class = vm.define_standard_class("Regexp");

    mrb_define_class_cmethod(vm, regexp_class, "new", Box::new(mrb_regexp_new));
}

pub struct Regexp {
    pattern: String,
}

pub fn mrb_regexp_new(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let pattern_obj = args[0].clone();
    match &pattern_obj.value {
        RValue::String(pattern) => {
            // For simplicity, we only support literal patterns without options.
            let pattern_str = pattern.clone().borrow().to_owned();
            let regexp = Regexp {
                pattern: String::from_utf8(pattern_str).map_err(|e| {
                    Error::RuntimeError(format!("Invalid regexp expression: {:?}", e))
                })?,
            };
            let regexp_data = Rc::new(RData {
                class: vm.get_class_by_name("Regexp"),
                ivar: RefCell::new(HashMap::new()),
                data: RefCell::new(Some(Rc::new(Box::new(regexp) as Box<dyn std::any::Any>))),
                ref_count: 1,
            });
            Ok(RObject {
                tt: crate::yamrb::value::RType::Data,
                value: RValue::Data(regexp_data),
                object_id: Cell::new(0),
                singleton_class: RefCell::new(None),
            }
            .to_refcount_assigned())
        }
        _ => Err(Error::RuntimeError(
            "Regexp.new requires a string pattern".to_string(),
        )),
    }
}
