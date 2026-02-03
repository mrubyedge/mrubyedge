pub mod json_value;

use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{helpers::mrb_define_class_cmethod, value::RObject, vm::VM},
};

pub fn init_json(vm: &mut VM) {
    let json_class = vm.define_class("JSON", None, None);
    mrb_define_class_cmethod(vm, json_class, "dump", Box::new(mrb_json_class_dump));
}

pub fn mrb_json_class_dump(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = if args[args.len() - 1].is_nil() {
        &args[0..args.len() - 1]
    } else {
        args
    };
    if args.len() != 1 {
        return Err(Error::ArgumentError(
            "wrong number of arguments".to_string(),
        ));
    }
    let result = json_value::mrb_json_dump(vm, args[0].clone())?;
    Ok(result)
}
