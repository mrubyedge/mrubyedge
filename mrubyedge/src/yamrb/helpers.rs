use std::rc::Rc;

use crate::{Error, yamrb::vm::Breadcrumb};

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
    passed_block: Option<Rc<RObject>>,
    method_info: Option<(RSym, Rc<RModule>)>,
    return_register: usize,
) -> Result<Rc<RObject>, Error> {
    let irep = block
        .irep
        .clone()
        .ok_or_else(|| Error::RuntimeError("No IREP".to_string()))?;
    let is_method = method_info.is_some();
    let frame_depth = vm.frames.len();
    let prev_upper = vm.upper.clone();
    let (method_id, method_owner) = match method_info {
        Some((id, owner)) => (id, Some(owner)),
        None => (RSym::new("<block>".to_string()), None),
    };
    let prev_n_args = vm.current_n_args.get();
    push_callinfo(vm, method_id, args.len(), method_owner, return_register);

    if !is_method {
        let frame = vm.frames.last_mut().unwrap();
        frame.is_lambda = block.is_lambda;
        frame.return_target = block.return_target.unwrap_or(frame.id);
    }

    if let Some(ci) = &vm.current_callinfo {
        ci.has_block.set(
            passed_block
                .as_ref()
                .is_some_and(|b| !matches!(b.value, RValue::Nil)),
        );
    }
    let old_callinfo = vm.current_callinfo.take();
    if let Some(ci) = &old_callinfo {
        vm.suspended_callinfos.push(ci.clone());
    }

    // Since call_block does not move the registers offset,
    // keep the state before the call.
    let prev_self = vm.current_regs()[0].replace(recv);

    let mut prev_args = vec![];
    for (i, arg) in args.iter().enumerate() {
        let old = vm.current_regs()[i + 1].replace(arg.clone());
        prev_args.push(old);
    }

    // Keep the block separate from positional arguments, including when it
    // is absent, and restore the caller's register after the call.
    let prev_block = vm.current_regs()[args.len() + 1]
        .replace(passed_block.unwrap_or_else(|| Rc::new(RObject::nil())));

    vm.pc.set(0);
    vm.current_irep = irep;
    // The callee's environ becomes the active upper env, but the caller's must
    // be restored on return. Deriving the restore from the callee's environ
    // chain used to clobber `vm.upper` to `None` after native->Ruby calls,
    // corrupting the upvar chain of enclosing blocks.
    vm.upper = block.environ;

    let res = vm.run_internal();
    while vm.frames.len() > frame_depth {
        vm.leave_frame();
    }
    if res.is_err() {
        vm.pending_exception = vm
            .exception
            .clone()
            .filter(|exception| !exception.error_type.borrow().is_control_flow());
    }
    if old_callinfo.is_some() {
        vm.suspended_callinfos.pop();
    }

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

    vm.current_regs()[args.len() + 1] = prev_block;
    vm.current_n_args.set(prev_n_args);

    if let Some(ci) = old_callinfo {
        if let Some(prev) = &ci.prev {
            vm.current_callinfo.replace(prev.clone());
        }
        vm.current_irep = ci.pc_irep.clone();
        vm.pc.set(ci.pc);
        vm.current_regs_offset = ci.current_regs_offset;
        vm.target_class = ci.target_class.clone();
    }
    vm.upper = prev_upper;

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
    return_register: usize,
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
    let upper = vm.current_breadcrumb.take();
    let new_breadcrumb = Rc::new(Breadcrumb {
        upper,
        event: "block_call",
        caller: None,
        return_reg: None,
    });
    vm.current_breadcrumb.replace(new_breadcrumb);
    let res = if block.is_rb_func {
        call_block(vm, block, recv, args, None, None, return_register)
    } else if block.is_fnblock {
        let func = vm.pop_fnblock()?;
        let res = func(vm, args);
        vm.push_fnblock(func)?;
        res
    } else {
        Err(Error::RuntimeError(
            "Cannot call non-block RProc".to_string(),
        ))
    };
    let cur = vm.current_breadcrumb.take().expect("not found breadcrumb");
    if let Some(upper) = &cur.as_ref().upper {
        vm.current_breadcrumb.replace(upper.clone());
    }
    res
}

/// Runs one iteration of a block for a native iterator. A `break` inside the
/// block stops the iteration and its value becomes the method's result (Ruby
/// semantics); the pending exception is consumed so it does not re-fire as a
/// phantom error in the enclosing loop. Returns `Ok(Some(value))` when the
/// block broke.
pub fn mrb_call_block_breakable(
    vm: &mut VM,
    block: Rc<RObject>,
    recv: Option<Rc<RObject>>,
    args: &[Rc<RObject>],
) -> Result<Option<Rc<RObject>>, Error> {
    match mrb_call_block(vm, block, recv, args, 0) {
        Ok(_) => Ok(None),
        Err(Error::Break(v)) => {
            vm.exception.take();
            vm.break_landing.take();
            Ok(Some(v))
        }
        Err(e) => Err(e),
    }
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
    mrb_funcall_with_block(vm, top_self, name, args, None)
}

/// Calls a method with positional arguments and a separate optional block.
/// A Proc in `args` is an ordinary positional argument, not an implicit block.
/// Native methods receive the block appended to their argument slice.
pub fn mrb_funcall_with_block(
    vm: &mut VM,
    top_self: Option<Rc<RObject>>,
    name: &str,
    args: &[Rc<RObject>],
    block: Option<Rc<RObject>>,
) -> Result<Rc<RObject>, Error> {
    let recv: Rc<RObject> = match &top_self {
        Some(obj) => obj.clone(),
        None => vm.getself()?,
    };
    let binding = recv.singleton_or_this_class(vm);
    let (owner_module, method) = match vm.resolve_method_cached(&binding, name) {
        Some((owner, method)) => (owner, method),
        None => {
            if name == "method_missing" {
                return Err(Error::Internal(
                    "[BUG] method_missing not defined".to_string(),
                ));
            }

            let mut mm_args = vec![Rc::new(RObject::symbol(RSym::new(name.to_string())))];
            mm_args.extend_from_slice(args);
            return mrb_funcall_with_block(vm, top_self, "method_missing", &mm_args, block);
        }
    };

    let upper = vm.current_breadcrumb.take();
    let new_breadcrumb = Rc::new(Breadcrumb {
        upper,
        event: "funcall",
        caller: Some(name.to_string()),
        return_reg: None,
    });
    vm.current_breadcrumb.replace(new_breadcrumb);

    let res = if method.is_rb_func {
        let method_id = method
            .sym_id
            .clone()
            .unwrap_or_else(|| RSym::new(name.to_string()));
        call_block(
            vm,
            method,
            recv.clone(),
            args,
            block,
            Some((method_id, owner_module)),
            0, // unused
        )
    } else {
        let prev = vm.current_regs()[0].replace(recv.clone());
        let func = vm.fn_table.get(method.func.unwrap()).unwrap();
        let mut native_args = args.to_vec();
        if let Some(block) = block {
            native_args.push(block);
        }
        let res = func(vm, &native_args);
        if let Err(error) = &res {
            vm.prepare_exception(error, Some(name));
        }
        if let Some(prev) = prev {
            vm.current_regs()[0].replace(prev);
        } else {
            vm.current_regs()[0].take();
        }

        res
    };
    let cur = vm.current_breadcrumb.take().expect("not found breadcrumb");
    if let Some(upper) = &cur.as_ref().upper {
        vm.current_breadcrumb.replace(upper.clone());
    }

    res
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
            None,
            Some((method_id, owner_module)),
            0, // unused
        )
    } else {
        let old = vm.current_regs()[0].replace(recv.clone());
        let func = vm.fn_table.get(method.func.unwrap()).unwrap();
        let res = func(vm, &[]);
        if let Some(old) = old {
            vm.current_regs()[0].replace(old);
        } else {
            vm.current_regs()[0].take();
        }
        res
    }
}

pub fn mrb_call_p(vm: &mut VM, recv: Rc<RObject>) {
    let inspect = mrb_call_inspect(vm, recv).expect("failed to call inspect");
    let inspect: String = inspect
        .as_ref()
        .try_into()
        .expect("failed to convert to string");
    eprintln!("{}", inspect);
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
        is_lambda: false,
        return_target: None,
        is_rb_func: false,
        is_fnblock: false,
        sym_id: Some(RSym::new(name.to_string())),
        next: None,
        irep: None,
        func: Some(index),
        environ: None,
        block_self: None,
    };
    let mut procs = klass.procs.borrow_mut();
    procs.insert(name.to_string(), method);
    vm.bump_method_version();
}

/// Defines a Ruby method (RProc) on a Ruby class.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `klass` - The class to define the method on
/// * `name` - The name of the method
/// * `method` - The Ruby proc to bind as a method
pub fn mrb_define_method(vm: &mut VM, klass: Rc<RClass>, name: &str, method: RProc) {
    let mut procs = klass.procs.borrow_mut();
    procs.insert(name.to_string(), method);
    vm.bump_method_version();
}

pub fn mrb_define_class_cmethod(vm: &mut VM, klass: Rc<RClass>, name: &str, cmethod: RFn) {
    let index = vm.register_fn(cmethod);
    let method = RProc {
        is_lambda: false,
        return_target: None,
        is_rb_func: false,
        is_fnblock: false,
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
    vm.bump_method_version();
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
        is_lambda: false,
        return_target: None,
        is_rb_func: false,
        is_fnblock: false,
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
    vm.bump_method_version();
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
    vm.bump_method_version();
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
        is_lambda: false,
        return_target: None,
        is_rb_func: false,
        is_fnblock: false,
        sym_id: Some(RSym::new(name.to_string())),
        next: None,
        irep: None,
        func: Some(index),
        environ: None,
        block_self: None,
    };
    let mut procs = module.procs.borrow_mut();
    procs.insert(name.to_string(), method);
    vm.bump_method_version();
}

/// Defines a Ruby method (RProc) on a Ruby module.
///
/// # Arguments
///
/// * `vm` - The virtual machine instance
/// * `module` - The module to define the method on
/// * `name` - The name of the method
/// * `method` - The Ruby proc to bind as a method
pub fn mrb_define_module_method(vm: &mut VM, module: Rc<RModule>, name: &str, method: RProc) {
    let mut procs = module.procs.borrow_mut();
    procs.insert(name.to_string(), method);
    vm.bump_method_version();
}

#[test]
fn test_mrb_inspect() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = VM::empty();
    let old_top_self = RObject::integer(1).to_refcount_assigned();
    vm.current_regs()[0].replace(old_top_self.clone());

    let class_a = vm.define_class("A", None, None);
    let class_a = RObject::class(class_a, &mut vm);
    let obj_a = mrb_funcall(&mut vm, Some(class_a), "new", &[])?;
    let res = mrb_call_inspect(&mut vm, obj_a.clone()).unwrap();
    let res_str: String = res.as_ref().try_into().unwrap();
    assert_eq!(
        res_str,
        "#<A:0x".to_string() + &format!("{:016x}", obj_a.object_id.get()) + ">"
    );

    // assert not to brake registers
    let updated = vm.get_current_regs_cloned(0)?;
    assert_eq!(updated.object_id.get(), old_top_self.object_id.get());

    Ok(())
}
