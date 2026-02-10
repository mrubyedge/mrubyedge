use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::mrb_define_singleton_cmethod,
        value::{RObject, RValue},
        vm::VM,
    },
};

pub fn init_math(vm: &mut VM) {
    let math_module = vm.define_module("Math", None);

    // Define constants
    math_module.consts.borrow_mut().insert(
        "PI".to_string(),
        RObject::float(std::f64::consts::PI).to_refcount_assigned(),
    );
    math_module.consts.borrow_mut().insert(
        "E".to_string(),
        RObject::float(std::f64::consts::E).to_refcount_assigned(),
    );

    // Get the module object to define singleton methods (module methods)
    let math_module_obj = vm.get_const_by_name("Math").expect("Math module not found");

    // Trigonometric functions
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "sin", Box::new(mrb_math_sin));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "cos", Box::new(mrb_math_cos));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "tan", Box::new(mrb_math_tan));

    // Inverse trigonometric functions
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "asin", Box::new(mrb_math_asin));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "acos", Box::new(mrb_math_acos));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "atan", Box::new(mrb_math_atan));
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "atan2",
        Box::new(mrb_math_atan2),
    );

    // Hyperbolic functions
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "sinh", Box::new(mrb_math_sinh));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "cosh", Box::new(mrb_math_cosh));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "tanh", Box::new(mrb_math_tanh));

    // Inverse hyperbolic functions
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "asinh",
        Box::new(mrb_math_asinh),
    );
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "acosh",
        Box::new(mrb_math_acosh),
    );
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "atanh",
        Box::new(mrb_math_atanh),
    );

    // Exponential and logarithmic functions
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "exp", Box::new(mrb_math_exp));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "log", Box::new(mrb_math_log));
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "log10",
        Box::new(mrb_math_log10),
    );
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "log2", Box::new(mrb_math_log2));

    // Root functions
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "sqrt", Box::new(mrb_math_sqrt));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "cbrt", Box::new(mrb_math_cbrt));

    // Other mathematical functions
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "hypot",
        Box::new(mrb_math_hypot),
    );
    mrb_define_singleton_cmethod(
        vm,
        math_module_obj.clone(),
        "ldexp",
        Box::new(mrb_math_ldexp),
    );
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "erf", Box::new(mrb_math_erf));
    mrb_define_singleton_cmethod(vm, math_module_obj.clone(), "erfc", Box::new(mrb_math_erfc));
}

// Helper function to get a float from RObject
fn get_float_arg(obj: &RObject) -> Result<f64, Error> {
    match &obj.value {
        RValue::Integer(i) => Ok(*i as f64),
        RValue::Float(f) => Ok(*f),
        _ => Err(Error::internal("expected Numeric for Math function")),
    }
}

// Helper function to check argument count (excluding trailing nil)
fn check_args_count(args: &[Rc<RObject>], expected: usize) -> Result<Vec<Rc<RObject>>, Error> {
    let args = if args.len() > 0 && args[args.len() - 1].is_nil() {
        &args[0..args.len() - 1]
    } else {
        args
    };

    if args.len() != expected {
        return Err(Error::ArgumentError(format!(
            "wrong number of arguments (given {}, expected {})",
            args.len(),
            expected
        )));
    }

    Ok(args.to_vec())
}

// Trigonometric functions
pub fn mrb_math_sin(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.sin()).to_refcount_assigned())
}

pub fn mrb_math_cos(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.cos()).to_refcount_assigned())
}

pub fn mrb_math_tan(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.tan()).to_refcount_assigned())
}

// Inverse trigonometric functions
pub fn mrb_math_asin(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.asin()).to_refcount_assigned())
}

pub fn mrb_math_acos(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.acos()).to_refcount_assigned())
}

pub fn mrb_math_atan(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.atan()).to_refcount_assigned())
}

pub fn mrb_math_atan2(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 2)?;
    let y = get_float_arg(&args[0])?;
    let x = get_float_arg(&args[1])?;
    Ok(RObject::float(y.atan2(x)).to_refcount_assigned())
}

// Hyperbolic functions
pub fn mrb_math_sinh(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.sinh()).to_refcount_assigned())
}

pub fn mrb_math_cosh(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.cosh()).to_refcount_assigned())
}

pub fn mrb_math_tanh(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.tanh()).to_refcount_assigned())
}

// Inverse hyperbolic functions
pub fn mrb_math_asinh(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.asinh()).to_refcount_assigned())
}

pub fn mrb_math_acosh(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.acosh()).to_refcount_assigned())
}

pub fn mrb_math_atanh(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.atanh()).to_refcount_assigned())
}

// Exponential and logarithmic functions
pub fn mrb_math_exp(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.exp()).to_refcount_assigned())
}

pub fn mrb_math_log(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args_vec = if args.len() > 0 && args[args.len() - 1].is_nil() {
        args[0..args.len() - 1].to_vec()
    } else {
        args.to_vec()
    };

    if args_vec.len() == 1 {
        let x = get_float_arg(&args_vec[0])?;
        Ok(RObject::float(x.ln()).to_refcount_assigned())
    } else if args_vec.len() == 2 {
        let x = get_float_arg(&args_vec[0])?;
        let base = get_float_arg(&args_vec[1])?;
        Ok(RObject::float(x.log(base)).to_refcount_assigned())
    } else {
        Err(Error::ArgumentError(format!(
            "wrong number of arguments (given {}, expected 1..2)",
            args_vec.len()
        )))
    }
}

pub fn mrb_math_log10(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.log10()).to_refcount_assigned())
}

pub fn mrb_math_log2(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.log2()).to_refcount_assigned())
}

// Root functions
pub fn mrb_math_sqrt(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.sqrt()).to_refcount_assigned())
}

pub fn mrb_math_cbrt(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    Ok(RObject::float(x.cbrt()).to_refcount_assigned())
}

// Other mathematical functions
pub fn mrb_math_hypot(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 2)?;
    let x = get_float_arg(&args[0])?;
    let y = get_float_arg(&args[1])?;
    Ok(RObject::float(x.hypot(y)).to_refcount_assigned())
}

pub fn mrb_math_ldexp(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 2)?;
    let fraction = get_float_arg(&args[0])?;
    let exponent: i32 = args[1].as_ref().try_into()?;
    Ok(RObject::float(fraction * 2f64.powi(exponent)).to_refcount_assigned())
}

pub fn mrb_math_erf(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    let result = erf_approximation(x);
    Ok(RObject::float(result).to_refcount_assigned())
}

pub fn mrb_math_erfc(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = check_args_count(args, 1)?;
    let x = get_float_arg(&args[0])?;
    let result = 1.0 - erf_approximation(x);
    Ok(RObject::float(result).to_refcount_assigned())
}

// Abramowitz and Stegun approximation of error function
fn erf_approximation(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke_sin() {
        let x = std::f64::consts::PI / 2.0;
        assert!((x.sin() - 1.0).abs() < 1e-10);
    }
}
