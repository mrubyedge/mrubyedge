use std::rc::Rc;

use crate::Error;

use super::{
    optable::push_callinfo,
    value::{RClass, RFn, RModule, RObject, RProc, RSym, RValue, resolve_method},
    vm::VM,
};

fn call_block(
    vm: &mut VM,
    block: RProc,
    recv: Rc<RObject>,
    args: &[Rc<RObject>],
    method_info: Option<(RSym, Rc<RModule>)>,
) -> Result<Rc<RObject>, Error> {
    let (method_id, method_owner) = match method_info {
        Some((id, owner)) => (id, Some(owner)),
        None => (RSym::new("<block>".to_string()), None),
    };
    push_callinfo(vm, method_id, args.len(), method_owner);

    let old_callinfo = vm.current_callinfo.take();

    // Since call_block does not move the registers offset,
    // keep the state before the call.
    let prev_self = vm.current_regs()[0].replace(recv);

    let mut prev_args = vec![];
    for (i, arg) in args.iter().enumerate() {
        let old = vm.current_regs()[i + 1].replace(arg.clone());
        prev_args.push(old);
    }

    vm.pc.set(0);
    vm.current_irep = block
        .irep
        .as_ref()
        .ok_or_else(|| Error::RuntimeError("No IREP".to_string()))?
        .clone();
    vm.upper = block.environ;

    let res = vm.run();

    if let Some(prev) = prev_self {
        vm.current_regs()[0].replace(prev);
    } else {
        vm.current_regs()[0].take();
    }
    for (i, prev_arg) in prev_args.into_iter().enumerate() {
        if let Some(prev) = prev_arg {
            vm.current_regs()[i + 1].replace(prev);
        } else {
            vm.current_regs()[i + 1].take();
        }
    }

    if let Some(ci) = old_callinfo {
        if let Some(prev) = &ci.prev {
            vm.current_callinfo.replace(prev.clone());
        }
        vm.current_irep = ci.pc_irep.clone();
        vm.pc.set(ci.pc);
        vm.current_regs_offset = ci.current_regs_offset;
        vm.target_class = ci.target_class.clone();
    }
    if let Some(upper) = vm.upper.take()
        && let Some(upper) = &upper.as_ref().upper
    {
        vm.upper.replace(upper.clone());
    }

    match &res {
        Ok(res) => Ok(res.clone()),
        Err(e) => {
            let err = if let Some(e) = e.downcast_ref::<Error>() {
                e.clone()
            } else {
                // TODO: Rust level error
                Error::RuntimeError(format!("{:?}", e.as_ref()))
            };
            Err(err)
        }
    }
}

/// Calls a Ruby block (Proc) with the given receiver and arguments.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `block` - The block object to call (must be a Proc)
/// * `recv` - Optional receiver object. If None, uses the block's self
/// * `args` - Array of arguments to pass to the block
///
/// # Returns
///
/// Returns the result of the block execution or an error if the call fails.
pub fn mrb_call_block(
    vm: &mut VM,
    block: Rc<RObject>,
    recv: Option<Rc<RObject>>,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    let block = match &block.value {
        RValue::Proc(p) => p.clone(),
        _ => panic!("Not a block"),
    };
    let recv = match recv {
        Some(r) => r,
        None => block
            .block_self
            .clone()
            .ok_or_else(|| Error::RuntimeError("No block self assigned".to_string()))?,
    };
    call_block(vm, block, recv, args, None)
}

/// Calls a method on an object by name with the given arguments.
///
/// This is the main function call interface for invoking Ruby methods from Rust code.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `top_self` - Optional receiver object. If None, uses the "top self"
/// * `name` - The name of the method to call
/// * `args` - Array of arguments to pass to the method
///
/// # Returns
///
/// Returns the result of the method call or an error if the method is not found or execution fails.
pub fn mrb_funcall(
    vm: &mut VM,
    top_self: Option<Rc<RObject>>,
    name: &str,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    let recv: Rc<RObject> = match top_self {
        Some(obj) => obj,
        None => vm.getself()?,
    };
    let binding = recv.initialize_or_get_singleton_class(vm);
    let (owner_module, method) =
        resolve_method(&binding, name).ok_or_else(|| Error::NoMethodError(name.to_string()))?;

    if method.is_rb_func {
        let method_id = method
            .sym_id
            .clone()
            .unwrap_or_else(|| RSym::new(name.to_string()));
        call_block(
            vm,
            method,
            recv.clone(),
            args,
            Some((method_id, owner_module)),
        )
    } else {
        vm.current_regs_offset += 2; // FIXME: magick number?
        vm.current_regs()[0].replace(recv.clone());

        let func = vm.fn_table[method.func.unwrap()].clone();
        let res = func(vm, args);
        vm.current_regs_offset -= 2;

        res
    }
}

pub fn mrb_call_inspect(vm: &mut VM, recv: Rc<RObject>) -> Result<Rc<RObject>, Error> {
    let binding = recv.get_class(vm);
    let (owner_module, method) = resolve_method(&binding, "inspect")
        .ok_or_else(|| Error::NoMethodError("inspect".to_string()))?;
    if method.is_rb_func {
        let method_id = method
            .sym_id
            .clone()
            .unwrap_or_else(|| RSym::new("inspect".to_string()));
        call_block(
            vm,
            method,
            recv.clone(),
            &[],
            Some((method_id, owner_module)),
        )
    } else {
        vm.current_regs_offset += 2; // FIXME: magick number?
        vm.current_regs()[0].replace(recv.clone());

        let func = vm.fn_table[method.func.unwrap()].clone();
        let res = func(vm, &[]);
        vm.current_regs_offset -= 2;

        res
    }
}

/// Defines a C method (native Rust function) on a Ruby class.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `klass` - The class to define the method on
/// * `name` - The name of the method
/// * `cmethod` - The native Rust function to bind as a method
pub fn mrb_define_cmethod(vm: &mut VM, klass: Rc<RClass>, name: &str, cmethod: RFn) {
    let index = vm.register_fn(cmethod);
    let method = RProc {
        is_rb_func: false,
        sym_id: Some(RSym::new(name.to_string())),
        next: None,
        irep: None,
        func: Some(index),
        environ: None,
        block_self: None,
    };
    let mut procs = klass.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}

/// Defines a Ruby method (RProc) on a Ruby class.
///
/// # Arguments
///
/// * `_vm` - The virtual machine instance (unused)
/// * `klass` - The class to define the method on
/// * `name` - The name of the method
/// * `method` - The Ruby proc to bind as a method
pub fn mrb_define_method(_vm: &mut VM, klass: Rc<RClass>, name: &str, method: RProc) {
    let mut procs = klass.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}

pub fn mrb_define_class_cmethod(vm: &mut VM, klass: Rc<RClass>, name: &str, cmethod: RFn) {
    let index = vm.register_fn(cmethod);
    let method = RProc {
        is_rb_func: false,
        sym_id: Some(RSym::new(name.to_string())),
        next: None,
        irep: None,
        func: Some(index),
        environ: None,
        block_self: None,
    };
    let klass_singleton = RObject::class_singleton(klass, vm);
    let mut procs = klass_singleton.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}

/// Defines a singleton C method (native Rust function) on a specific Ruby object.
///
/// Singleton methods are methods defined on individual objects rather than classes.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `dest` - The object to define the singleton method on
/// * `name` - The name of the method
/// * `cmethod` - The native Rust function to bind as a singleton method
pub fn mrb_define_singleton_cmethod(vm: &mut VM, dest: Rc<RObject>, name: &str, cmethod: RFn) {
    let index = vm.register_fn(cmethod);
    let method = RProc {
        is_rb_func: false,
        sym_id: Some(RSym::new(name.to_string())),
        next: None,
        irep: None,
        func: Some(index),
        environ: None,
        block_self: None,
    };
    let klass = dest.initialize_or_get_singleton_class(vm);
    let mut procs = klass.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}

/// Defines a singleton Ruby method (RProc) on a specific Ruby object.
///
/// Singleton methods are methods defined on individual objects rather than classes.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `dest` - The object to define the singleton method on
/// * `name` - The name of the method
/// * `method` - The Ruby proc to bind as a singleton method
pub fn mrb_define_singleton_method(vm: &mut VM, dest: Rc<RObject>, name: &str, method: RProc) {
    let klass = dest.initialize_or_get_singleton_class(vm);
    let mut procs = klass.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}

/// Defines a C method (native Rust function) on a Ruby module.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `module` - The module to define the method on
/// * `name` - The name of the method
/// * `cmethod` - The native Rust function to bind as a method
pub fn mrb_define_module_cmethod(vm: &mut VM, module: Rc<RModule>, name: &str, cmethod: RFn) {
    let index = vm.register_fn(cmethod);
    let method = RProc {
        is_rb_func: false,
        sym_id: Some(RSym::new(name.to_string())),
        next: None,
        irep: None,
        func: Some(index),
        environ: None,
        block_self: None,
    };
    let mut procs = module.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}

/// Defines a Ruby method (RProc) on a Ruby module.
///
/// # Arguments
///
/// * `_vm` - The virtual machine instance (unused)
/// * `module` - The module to define the method on
/// * `name` - The name of the method
/// * `method` - The Ruby proc to bind as a method
pub fn mrb_define_module_method(_vm: &mut VM, module: Rc<RModule>, name: &str, method: RProc) {
    let mut procs = module.procs.borrow_mut();
    procs.insert(name.to_string(), method);
}
