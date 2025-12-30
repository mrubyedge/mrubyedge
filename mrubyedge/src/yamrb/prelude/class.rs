use std::rc::Rc;

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_define_cmethod, mrb_funcall},
        value::*,
        vm::VM,
    },
};

pub(crate) fn initialize_class(vm: &mut VM) {
    let module_class = vm.get_class_by_name("Module");
    mrb_define_cmethod(
        vm,
        module_class.clone(),
        "inspect",
        Box::new(mrb_module_inspect),
    );

    let class_class = vm.define_standard_class_with_superclass("Class", module_class);

    // Create singleton class for Object class
    RObject::class(vm.object_class.clone(), vm).initialize_or_get_singleton_class_for_class(vm);

    mrb_define_cmethod(vm, class_class.clone(), "new", Box::new(mrb_class_new));
    mrb_define_cmethod(
        vm,
        class_class.clone(),
        "attr_reader",
        Box::new(mrb_class_attr_reader),
    );
    mrb_define_cmethod(
        vm,
        class_class.clone(),
        "attr_writer",
        Box::new(mrb_class_attr_writer),
    );
    mrb_define_cmethod(
        vm,
        class_class.clone(),
        "attr_accessor",
        Box::new(mrb_class_attr_acceccor),
    );
    mrb_define_cmethod(
        vm,
        class_class.clone(),
        "attr",
        Box::new(mrb_class_attr_acceccor),
    );
    mrb_define_cmethod(vm, class_class, "ancestors", Box::new(mrb_class_ancestors));
}

fn mrb_class_new(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let class = vm.getself()?;
    let class = match &class.value {
        RValue::Class(c) => c.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Class#new must be called from class".to_string(),
            ));
        }
    };

    let obj = RObject::instance(class).to_refcount_assigned();

    mrb_funcall(vm, Some(obj.clone()), "initialize", args)?;

    Ok(obj)
}

fn mrb_class_attr_reader(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let class_ = vm.getself()?;
    let class = match &class_.value {
        RValue::Class(c) => c.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Class#attr_reader must be called from class".to_string(),
            ));
        }
    };
    for arg in args.iter() {
        match arg.value {
            RValue::Symbol(ref sym) => {
                let sym_id: &'static str = sym.name.clone().leak();
                let method = move |vm: &mut VM, _args: &[Rc<RObject>]| {
                    let this = vm.getself()?;
                    let key = format!("@{}", sym_id);
                    Ok(this.get_ivar(&key))
                };
                mrb_define_cmethod(vm, class.clone(), sym_id, Box::new(method));
            }
            RValue::Nil => {
                // skip
            }
            _ => {
                return Err(Error::RuntimeError(
                    "Class#attr_reader must be called with symbols".to_string(),
                ));
            }
        }
    }
    Ok(Rc::new(RObject::nil()))
}

fn mrb_class_attr_writer(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let class_ = vm.getself()?;
    let class = match &class_.value {
        RValue::Class(c) => c.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Class#attr_reader must be called from class".to_string(),
            ));
        }
    };
    for arg in args.iter() {
        match arg.value {
            RValue::Symbol(ref sym) => {
                let sym_id: &'static str = sym.name.clone().leak();
                let method = move |vm: &mut VM, args: &[Rc<RObject>]| {
                    let this = vm.getself()?;
                    let key = format!("@{}", sym_id);
                    let value = args[0].clone();
                    this.set_ivar(&key, value.clone());
                    Ok(value)
                };
                let sym_id = format!("{}=", sym_id);
                mrb_define_cmethod(vm, class.clone(), &sym_id, Box::new(method));
            }
            RValue::Nil => {
                // skip
            }
            _ => {
                return Err(Error::RuntimeError(
                    "Class#attr_reader must be called with symbols".to_string(),
                ));
            }
        }
    }
    Ok(Rc::new(RObject::nil()))
}

fn mrb_class_attr_acceccor(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    mrb_class_attr_reader(vm, args)?;
    mrb_class_attr_writer(vm, args)
}

fn mrb_class_ancestors(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_module = vm.getself()?;
    let target_class = match &self_module.value {
        RValue::Class(class) => class.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Module#ancestors must be called on class or module".to_string(),
            ));
        }
    };
    let ancestors: Vec<Rc<RObject>> = build_lookup_chain(&target_class)
        .iter()
        .map(|m| RObject::class_or_module(m.clone(), vm))
        .collect();
    Ok(RObject::array(ancestors).to_refcount_assigned())
}

fn mrb_module_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let class = vm.getself()?;
    let class_name = match &class.value {
        RValue::Class(c) => c.full_name(),
        RValue::Module(m) => m.full_name(),
        _ => {
            return Err(Error::RuntimeError(
                "Module#inspect must be called from module or class".to_string(),
            ));
        }
    };
    Ok(Rc::new(RObject::string(class_name)))
}

#[test]
fn test_class_attr_accessor() {
    use crate::yamrb::helpers::*;

    let mut vm = VM::empty();
    let class = vm.define_class("Test", None, None);
    let args = vec![RObject::symbol("foo".into()).to_refcount_assigned()];
    let classobj = RObject::class(class.clone(), &mut vm);
    vm.current_regs()[0].replace(classobj.clone());
    mrb_class_attr_acceccor(&mut vm, &args).expect("mrb_class_attr_acceccor failed");

    let instance = RObject::instance(class).to_refcount_assigned();

    let args = vec![RObject::integer(557188).to_refcount_assigned()];
    mrb_funcall(&mut vm, Some(instance.clone()), "foo=", &args).expect("call obj.foo = failed");

    let ret =
        mrb_funcall(&mut vm, Some(instance.clone()), "foo", &[]).expect("call obj.foo failed");
    let ret: i64 = ret.as_ref().try_into().expect("obj.foo must be integer");
    assert_eq!(ret, 557188);
}
