use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::yamrb::helpers::mrb_funcall;
use crate::yamrb::value::{RClass, RHashMap};
use crate::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod},
        value::{RData, RObject, RType, RValue},
        vm::VM,
    },
};

use rand_core::SeedableRng;
use rand_xorshift;

const DEFAULT_RNG_KEY: &str = "@_default_rng";

pub struct Random {
    pub rng_state: rand_xorshift::XorShiftRng,
    pub seed: u64,
}

fn new_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        // ref: https://github.com/rust-lang/rust/blob/b2a322beb29110e22a1782e2ce5ed2a0719b81ed/library/std/src/sys/random/unsupported.rs
        let heap = Box::new(0u8);
        std::ptr::from_ref(&*heap).addr() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

fn create_seeded_rng(seed: u64) -> rand_xorshift::XorShiftRng {
    // Create a seed array for XorShiftRng (16 bytes)
    let seed_bytes = [
        (seed & 0xFF) as u8,
        ((seed >> 8) & 0xFF) as u8,
        ((seed >> 16) & 0xFF) as u8,
        ((seed >> 24) & 0xFF) as u8,
        ((seed >> 32) & 0xFF) as u8,
        ((seed >> 40) & 0xFF) as u8,
        ((seed >> 48) & 0xFF) as u8,
        ((seed >> 56) & 0xFF) as u8,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8, // Additional bytes to fill 16 bytes
    ];

    rand_xorshift::XorShiftRng::from_seed(seed_bytes)
}

pub(crate) fn initialize_rand(vm: &mut VM) {
    let random_class = vm.define_class("Random", None, None);

    mrb_define_class_cmethod(vm, random_class.clone(), "new", Box::new(mrb_random_new));
    mrb_define_cmethod(vm, random_class.clone(), "rand", Box::new(mrb_random_rand));
    mrb_define_cmethod(vm, random_class.clone(), "seed", Box::new(mrb_random_seed));
    mrb_define_class_cmethod(
        vm,
        random_class.clone(),
        "rand",
        Box::new(mrb_random_class_rand),
    );
    mrb_define_class_cmethod(
        vm,
        random_class.clone(),
        "srand",
        Box::new(mrb_random_class_srand),
    );

    let object_class = vm.object_class.clone();
    mrb_define_cmethod(vm, object_class, "rand", Box::new(mrb_kernel_rand));

    let default_rng = mrb_random_new(vm, &[]).expect("failed to generate default rng");
    let random_singleton_instance = RObject::class(random_class, vm);
    random_singleton_instance.set_ivar(DEFAULT_RNG_KEY, default_rng);
}

fn get_rng_singleton(vm: &mut VM) -> Rc<RObject> {
    vm.get_const_by_name("Random")
        .expect("Random class not found when accessing singleton instance")
}

fn get_rng_class(vm: &mut VM) -> Rc<RClass> {
    match &get_rng_singleton(vm).value {
        RValue::Class(class) => class.clone(),
        _ => panic!("Random is not a class"),
    }
}

fn get_default_rng(vm: &mut VM) -> Rc<RObject> {
    get_rng_singleton(vm).get_ivar(DEFAULT_RNG_KEY)
}

pub(crate) fn mrb_random_new(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let class = get_rng_class(vm);
    let args = if !args.is_empty() && args[args.len() - 1].is_nil() {
        &args[0..args.len() - 1]
    } else {
        args
    };
    let seed = if args.is_empty() {
        new_seed()
    } else {
        let seed_obj = &args[0];
        seed_obj.as_ref().try_into()?
    };
    let rand = Random {
        rng_state: create_seeded_rng(seed),
        seed,
    };
    // For simplicity, we return a new Random instance without state
    let random_data = RData {
        class,
        data: RefCell::new(Some(Rc::new(Box::new(rand)))),
        ref_count: 1,
    };
    let random_instance = Rc::new(RObject {
        tt: RType::Data,
        value: RValue::Data(Rc::new(random_data)),
        object_id: Cell::new(u64::MAX),
        singleton_class: RefCell::new(None),
        ivar: RefCell::new(RHashMap::default()),
    });

    Ok(random_instance)
}

// Random.srand
fn mrb_random_class_srand(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let args = if !args.is_empty() && args[args.len() - 1].is_nil() {
        &args[0..args.len() - 1]
    } else {
        args
    };
    let seed = if args.is_empty() {
        new_seed()
    } else {
        let seed_obj = &args[0];
        seed_obj.as_ref().try_into()?
    };

    let old_seed = {
        let default_rng = get_default_rng(vm);
        mrb_funcall(vm, Some(default_rng), "seed", &[])?
    };
    let new_rng = mrb_random_new(vm, &[Rc::new(RObject::integer(seed as i64))])?;
    let random_singleton = get_rng_singleton(vm);
    random_singleton.set_ivar(DEFAULT_RNG_KEY, new_rng);

    Ok(old_seed)
}

// Random#seed
fn mrb_random_seed(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;

    let seed = match &self_obj.value {
        RValue::Data(data) => {
            let borrow = data.data.borrow();
            let any_ref = borrow
                .as_ref()
                .ok_or_else(|| Error::RuntimeError("Invalid Random data".to_string()))?;
            let random = any_ref
                .downcast_ref::<Random>()
                .ok_or_else(|| Error::RuntimeError("Invalid Random data".to_string()))?;
            random.seed
        }
        _ => {
            return Err(Error::RuntimeError(
                "Random#seed must be called on a Random object".to_string(),
            ));
        }
    };

    Ok(Rc::new(RObject::integer(seed as i64)))
}

// Random#rand
fn mrb_random_rand(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    use rand_core::Rng;

    let args = if !args.is_empty() && args[args.len() - 1].is_nil() {
        &args[0..args.len() - 1]
    } else {
        args
    };

    let self_obj = vm.getself()?;

    let result = match &self_obj.value {
        RValue::Data(data) => {
            let mut borrow = data.data.borrow_mut();
            let any_ref = borrow
                .as_mut()
                .ok_or_else(|| Error::RuntimeError("Invalid Random data".to_string()))?;
            let random = Rc::get_mut(any_ref)
                .and_then(|boxed| boxed.downcast_mut::<Random>())
                .ok_or_else(|| Error::RuntimeError("Invalid Random data".to_string()))?;
            let rng: &mut dyn Rng = &mut random.rng_state;

            if args.is_empty() {
                // Return a float between 0.0 and 1.0
                let value = (rng.next_u32() as f64) / (u32::MAX as f64);
                Rc::new(RObject::float(value))
            } else {
                let max_obj = &args[0];
                match max_obj.value {
                    RValue::Integer(max) => {
                        if max <= 0 {
                            return Err(Error::ArgumentError("max must be positive".to_string()));
                        }
                        let value = (rng.next_u64() % (max as u64)) as i64;
                        Rc::new(RObject::integer(value))
                    }
                    RValue::Float(max) => {
                        if max <= 0.0 {
                            return Err(Error::ArgumentError("max must be positive".to_string()));
                        }
                        let value = (rng.next_u32() as f64) / (u32::MAX as f64) * max;
                        Rc::new(RObject::float(value))
                    }
                    _ => {
                        return Err(Error::ArgumentError(
                            "argument must be a number".to_string(),
                        ));
                    }
                }
            }
        }
        _ => {
            return Err(Error::RuntimeError(
                "Random#rand must be called on a Random object".to_string(),
            ));
        }
    };

    Ok(result)
}

// Random.rand (class method)
fn mrb_random_class_rand(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let default_rng = get_default_rng(vm);
    mrb_funcall(vm, Some(default_rng), "rand", args)
}

// Kernel#rand
fn mrb_kernel_rand(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let default_rng = get_default_rng(vm);
    mrb_funcall(vm, Some(default_rng), "rand", args)
}
