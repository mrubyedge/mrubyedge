use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::mrb_define_cmethod;

use crate::yamrb::{value::RObject, vm::VM};

pub(crate) fn initialize_float(vm: &mut VM) {
    let float_class = vm.define_standard_class("Float");
    mrb_define_cmethod(vm, float_class.clone(), "to_i", Box::new(mrb_float_to_i));
    mrb_define_cmethod(vm, float_class.clone(), "to_f", Box::new(mrb_float_to_f));
    mrb_define_cmethod(vm, float_class.clone(), "+", Box::new(mrb_float_add));
    mrb_define_cmethod(vm, float_class.clone(), "-", Box::new(mrb_float_sub));
    mrb_define_cmethod(vm, float_class.clone(), "*", Box::new(mrb_float_mul));
    mrb_define_cmethod(vm, float_class.clone(), "/", Box::new(mrb_float_div));
    mrb_define_cmethod(vm, float_class.clone(), "+@", Box::new(mrb_float_positive));
    mrb_define_cmethod(vm, float_class.clone(), "-@", Box::new(mrb_float_negative));
    mrb_define_cmethod(vm, float_class.clone(), "**", Box::new(mrb_float_power));
    mrb_define_cmethod(vm, float_class.clone(), "abs", Box::new(mrb_float_abs));
    mrb_define_cmethod(
        vm,
        float_class.clone(),
        "inspect",
        Box::new(mrb_float_inspect),
    );
    mrb_define_cmethod(vm, float_class.clone(), "to_s", Box::new(mrb_float_inspect));
    mrb_define_cmethod(vm, float_class.clone(), "clamp", Box::new(mrb_float_clamp));
}

pub fn mrb_float_to_i(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => {
            let int_value = *f as i64;
            Ok(RObject::integer(int_value).to_refcount_assigned())
        }
        _ => Err(Error::RuntimeError(
            "Float#to_i must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_to_f(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => Ok(RObject::float(*f).to_refcount_assigned()),
        _ => Err(Error::RuntimeError(
            "Float#to_f must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => {
            let s = format!("{}", f);
            Ok(RObject::string(s).to_refcount_assigned())
        }
        _ => Err(Error::RuntimeError(
            "Float#inspect must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_clamp(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.len() < 2 {
        return Err(Error::ArgumentError(format!(
            "wrong number of arguments (given {}, expected 2)",
            args.len()
        )));
    }

    let this = vm.getself()?;
    let this_float = match &this.value {
        crate::yamrb::value::RValue::Float(f) => *f,
        _ => {
            return Err(Error::RuntimeError(
                "Float#clamp must be called on a Float".to_string(),
            ));
        }
    };

    // Convert min and max to f64
    let min = match &args[0].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    let max = match &args[1].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    if min > max {
        return Err(Error::ArgumentError(
            "min argument must be smaller than max argument".to_string(),
        ));
    }

    let result = if this_float < min {
        min
    } else if this_float > max {
        max
    } else {
        this_float
    };

    Ok(RObject::float(result).to_refcount_assigned())
}

pub fn mrb_float_add(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }

    let this = vm.getself()?;
    let this_float = match &this.value {
        crate::yamrb::value::RValue::Float(f) => *f,
        _ => {
            return Err(Error::RuntimeError(
                "Float#+ must be called on a Float".to_string(),
            ));
        }
    };

    let other = match &args[0].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    Ok(RObject::float(this_float + other).to_refcount_assigned())
}

pub fn mrb_float_sub(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }

    let this = vm.getself()?;
    let this_float = match &this.value {
        crate::yamrb::value::RValue::Float(f) => *f,
        _ => {
            return Err(Error::RuntimeError(
                "Float#- must be called on a Float".to_string(),
            ));
        }
    };

    let other = match &args[0].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    Ok(RObject::float(this_float - other).to_refcount_assigned())
}

pub fn mrb_float_mul(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }

    let this = vm.getself()?;
    let this_float = match &this.value {
        crate::yamrb::value::RValue::Float(f) => *f,
        _ => {
            return Err(Error::RuntimeError(
                "Float#* must be called on a Float".to_string(),
            ));
        }
    };

    let other = match &args[0].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    Ok(RObject::float(this_float * other).to_refcount_assigned())
}

pub fn mrb_float_div(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }

    let this = vm.getself()?;
    let this_float = match &this.value {
        crate::yamrb::value::RValue::Float(f) => *f,
        _ => {
            return Err(Error::RuntimeError(
                "Float#/ must be called on a Float".to_string(),
            ));
        }
    };

    let other = match &args[0].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    if other == 0.0 {
        return Err(Error::ZeroDivisionError);
    }

    Ok(RObject::float(this_float / other).to_refcount_assigned())
}

pub fn mrb_float_positive(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => Ok(RObject::float(*f).to_refcount_assigned()),
        _ => Err(Error::RuntimeError(
            "Float#+@ must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_negative(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => Ok(RObject::float(-*f).to_refcount_assigned()),
        _ => Err(Error::RuntimeError(
            "Float#-@ must be called on a Float".to_string(),
        )),
    }
}

pub fn mrb_float_power(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments (given 0, expected 1)".to_string(),
        ));
    }

    let this = vm.getself()?;
    let this_float = match &this.value {
        crate::yamrb::value::RValue::Float(f) => *f,
        _ => {
            return Err(Error::RuntimeError(
                "Float#** must be called on a Float".to_string(),
            ));
        }
    };

    let other = match &args[0].value {
        crate::yamrb::value::RValue::Float(f) => *f,
        crate::yamrb::value::RValue::Integer(i) => *i as f64,
        _ => return Err(Error::TypeMismatch),
    };

    Ok(RObject::float(this_float.powf(other)).to_refcount_assigned())
}

pub fn mrb_float_abs(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    match &this.value {
        crate::yamrb::value::RValue::Float(f) => Ok(RObject::float(f.abs()).to_refcount_assigned()),
        _ => Err(Error::RuntimeError(
            "Float#abs must be called on a Float".to_string(),
        )),
    }
}
