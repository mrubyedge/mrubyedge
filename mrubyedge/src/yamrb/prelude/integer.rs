use std::rc::Rc;

use crate::Error;
use crate::yamrb::helpers::mrb_define_cmethod;

use crate::yamrb::value::RValue;
use crate::yamrb::{helpers::mrb_call_block, value::RObject, vm::VM};

pub(crate) fn initialize_integer(vm: &mut VM) {
    let integer_class = vm.define_standard_class("Integer");

    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "[]",
        Box::new(mrb_integer_bitref),
    );
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "-@",
        Box::new(mrb_integer_negative),
    );
    mrb_define_cmethod(vm, integer_class.clone(), "+", Box::new(mrb_integer_add));
    mrb_define_cmethod(vm, integer_class.clone(), "-", Box::new(mrb_integer_sub));
    mrb_define_cmethod(vm, integer_class.clone(), "**", Box::new(mrb_integer_power));
    mrb_define_cmethod(vm, integer_class.clone(), "%", Box::new(mrb_integer_mod));
    mrb_define_cmethod(vm, integer_class.clone(), "&", Box::new(mrb_integer_and));
    mrb_define_cmethod(vm, integer_class.clone(), "|", Box::new(mrb_integer_or));
    mrb_define_cmethod(vm, integer_class.clone(), "^", Box::new(mrb_integer_xor));
    mrb_define_cmethod(vm, integer_class.clone(), "~", Box::new(mrb_integer_not));
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "<<",
        Box::new(mrb_integer_lshift),
    );
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        ">>",
        Box::new(mrb_integer_rshift),
    );
    mrb_define_cmethod(vm, integer_class.clone(), "abs", Box::new(mrb_integer_abs));
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "to_i",
        Box::new(mrb_integer_to_i),
    );
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "to_f",
        Box::new(mrb_integer_to_f),
    );
    mrb_define_cmethod(vm, integer_class.clone(), "chr", Box::new(mrb_integer_chr));
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "times",
        Box::new(mrb_integer_times),
    );
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "inspect",
        Box::new(mrb_integer_inspect),
    );
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "to_s",
        Box::new(mrb_integer_inspect),
    );
    mrb_define_cmethod(
        vm,
        integer_class.clone(),
        "clamp",
        Box::new(mrb_integer_clamp),
    );
}

fn mrb_integer_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this.to_string())))
}

fn mrb_integer_times(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    for i in 0..this {
        let block = args[0].clone();
        let args = vec![Rc::new(RObject::integer(i))];
        mrb_call_block(vm, block, None, &args, 0)?;
    }
    vm.getself()
}

fn mrb_integer_mod(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs: i64 = args[0].as_ref().try_into()?;

    Ok(Rc::new(RObject::integer(lhs % rhs)))
}

fn mrb_integer_bitref(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    let index: i64 = args[0].as_ref().try_into()?;

    if index < 0 {
        return Ok(Rc::new(RObject::integer(0)));
    }

    let bit = (this >> index) & 1;
    Ok(Rc::new(RObject::integer(bit)))
}

fn mrb_integer_negative(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(-this)))
}

fn mrb_integer_add(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs_obj = &args[0];

    match &rhs_obj.as_ref().value {
        RValue::Integer(rhs) => Ok(Rc::new(RObject::integer(lhs + rhs))),
        RValue::Float(rhs) => Ok(Rc::new(RObject::float(lhs as f64 + rhs))),
        _ => Err(Error::TypeMismatch),
    }
}

fn mrb_integer_sub(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs_obj = &args[0];

    match &rhs_obj.as_ref().value {
        RValue::Integer(rhs) => Ok(Rc::new(RObject::integer(lhs - rhs))),
        RValue::Float(rhs) => Ok(Rc::new(RObject::float(lhs as f64 - rhs))),
        _ => Err(Error::TypeMismatch),
    }
}

fn mrb_integer_power(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let base: i64 = vm.getself()?.as_ref().try_into()?;
    let exponent_obj = &args[0];

    match &exponent_obj.as_ref().value {
        RValue::Integer(exp) => {
            if *exp >= 0 {
                // Positive integer exponent
                let result = base.pow(*exp as u32);
                Ok(Rc::new(RObject::integer(result)))
            } else {
                // Negative integer exponent - return float
                let result = (base as f64).powf(*exp as f64);
                Ok(Rc::new(RObject::float(result)))
            }
        }
        RValue::Float(exp) => {
            // Float exponent - return float
            let result = (base as f64).powf(*exp);
            Ok(Rc::new(RObject::float(result)))
        }
        _ => Err(Error::TypeMismatch),
    }
}

fn mrb_integer_and(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs: i64 = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(lhs & rhs)))
}

fn mrb_integer_or(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs: i64 = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(lhs | rhs)))
}

fn mrb_integer_xor(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs: i64 = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(lhs ^ rhs)))
}

fn mrb_integer_not(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(!this)))
}

fn mrb_integer_lshift(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs: i64 = args[0].as_ref().try_into()?;

    if rhs < 0 {
        return Err(Error::ArgumentError("negative shift count".to_string()));
    }

    Ok(Rc::new(RObject::integer(lhs << rhs)))
}

fn mrb_integer_rshift(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let lhs: i64 = vm.getself()?.as_ref().try_into()?;
    let rhs: i64 = args[0].as_ref().try_into()?;

    if rhs < 0 {
        return Err(Error::ArgumentError("negative shift count".to_string()));
    }

    Ok(Rc::new(RObject::integer(lhs >> rhs)))
}

fn mrb_integer_abs(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(this.abs())))
}

fn mrb_integer_to_i(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    vm.getself()
}

fn mrb_integer_to_f(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::float(this as f64)))
}

fn mrb_integer_chr(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: i64 = vm.getself()?.as_ref().try_into()?;

    if !(0..=0x10FFFF).contains(&this) {
        return Err(Error::RangeError(format!("{} out of char range", this)));
    }

    let ch = char::from_u32(this as u32)
        .ok_or_else(|| Error::RangeError(format!("invalid codepoint: {}", this)))?;

    Ok(Rc::new(RObject::string(ch.to_string())))
}

fn mrb_integer_clamp(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.len() < 2 {
        return Err(Error::ArgumentError(format!(
            "wrong number of arguments (given {}, expected 2)",
            args.len()
        )));
    }

    let this: i64 = vm.getself()?.as_ref().try_into()?;
    let min: i64 = args[0].as_ref().try_into()?;
    let max: i64 = args[1].as_ref().try_into()?;

    if min > max {
        return Err(Error::ArgumentError(
            "min argument must be smaller than max argument".to_string(),
        ));
    }

    let result = if this < min {
        min
    } else if this > max {
        max
    } else {
        this
    };

    Ok(Rc::new(RObject::integer(result)))
}
