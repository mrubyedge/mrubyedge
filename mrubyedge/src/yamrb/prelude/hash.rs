use std::rc::Rc;

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_call_block, mrb_call_inspect, mrb_define_class_cmethod, mrb_define_cmethod},
        value::{RHashMap, RObject, RValue},
        vm::VM,
    },
};

pub(crate) fn initialize_hash(vm: &mut VM) {
    let hash_class = vm.define_standard_class("Hash");

    mrb_define_class_cmethod(vm, hash_class.clone(), "new", Box::new(mrb_hash_new));

    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "[]",
        Box::new(mrb_hash_get_index_self),
    );
    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "[]=",
        Box::new(mrb_hash_set_index_self),
    );
    mrb_define_cmethod(vm, hash_class.clone(), "clear", Box::new(mrb_hash_clear));
    mrb_define_cmethod(vm, hash_class.clone(), "dup", Box::new(mrb_hash_dup));
    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "delete",
        Box::new(mrb_hash_delete_self),
    );
    mrb_define_cmethod(vm, hash_class.clone(), "empty?", Box::new(mrb_hash_empty));
    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "has_key?",
        Box::new(mrb_hash_has_key),
    );
    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "has_value?",
        Box::new(mrb_hash_has_value),
    );
    mrb_define_cmethod(vm, hash_class.clone(), "key", Box::new(mrb_hash_key));
    mrb_define_cmethod(vm, hash_class.clone(), "keys", Box::new(mrb_hash_keys));
    mrb_define_cmethod(vm, hash_class.clone(), "each", Box::new(mrb_hash_each));
    mrb_define_cmethod(vm, hash_class.clone(), "size", Box::new(mrb_hash_size));
    mrb_define_cmethod(vm, hash_class.clone(), "length", Box::new(mrb_hash_size));
    mrb_define_cmethod(vm, hash_class.clone(), "count", Box::new(mrb_hash_size));
    mrb_define_cmethod(vm, hash_class.clone(), "merge", Box::new(mrb_hash_merge));
    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "merge!",
        Box::new(mrb_hash_merge_self),
    );
    mrb_define_cmethod(vm, hash_class.clone(), "to_h", Box::new(mrb_hash_to_h));
    mrb_define_cmethod(vm, hash_class.clone(), "values", Box::new(mrb_hash_values));
    mrb_define_cmethod(
        vm,
        hash_class.clone(),
        "inspect",
        Box::new(mrb_hash_inspect),
    );
    mrb_define_cmethod(vm, hash_class.clone(), "to_s", Box::new(mrb_hash_inspect));
}

pub fn mrb_hash_new(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::hash(RHashMap::default())))
}

fn mrb_hash_get_index_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    mrb_hash_get_index(this, args[0].clone())
}

pub fn mrb_hash_get_index(this: Rc<RObject>, key: Rc<RObject>) -> Result<Rc<RObject>, Error> {
    let hash = match &this.value {
        RValue::Hash(a) => a.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Hash#[] must called on a hash".to_string(),
            ));
        }
    };
    let hash = hash.borrow();
    let key = key.as_ref().as_hash_key()?;
    match hash.get(&key) {
        Some((_, value)) => Ok(value.clone()),
        None => Ok(Rc::new(RObject::nil())),
    }
}

fn mrb_hash_set_index_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let key = args[0].clone();
    let value = args[1].clone();
    mrb_hash_set_index(this, key, value)
}

pub fn mrb_hash_set_index(
    this: Rc<RObject>,
    key: Rc<RObject>,
    value: Rc<RObject>,
) -> Result<Rc<RObject>, Error> {
    let hash = match &this.value {
        RValue::Hash(a) => a,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#[] must called on a hash".to_string(),
            ));
        }
    };
    let mut hash = hash.borrow_mut();
    let hashed = key.as_hash_key()?;
    hash.insert(hashed, (key.clone(), value.clone()));
    Ok(value.clone())
}

pub fn mrb_hash_delete(this: Rc<RObject>, key: Rc<RObject>) -> Result<Rc<RObject>, Error> {
    let hash = match &this.value {
        RValue::Hash(a) => a,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#delete must called on a hash".to_string(),
            ));
        }
    };
    let mut hash = hash.borrow_mut();
    let hashed = key.as_hash_key()?;
    match hash.remove(&hashed) {
        Some((_, value)) => Ok(value.clone()),
        None => Ok(Rc::new(RObject::nil())),
    }
}

fn mrb_hash_delete_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let key = args[0].clone();
    mrb_hash_delete(this, key)
}

fn mrb_hash_each(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let block = &args[0];
    match &this.value {
        RValue::Hash(hash) => {
            let hash = hash.borrow();
            for (_, (key, value)) in hash.iter() {
                let args = vec![key.clone(), value.clone()];
                mrb_call_block(vm, block.clone(), None, &args, 0)?;
            }
        }
        _ => {
            return Err(Error::RuntimeError(
                "Hash#each must be called on a hash".to_string(),
            ));
        }
    };
    Ok(this.clone())
}

fn mrb_hash_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#inspect must be called on a hash".to_string(),
            ));
        }
    };
    let hash = hash.borrow();
    let mut parts: Vec<String> = Vec::new();
    for (_, (key, value)) in hash.iter() {
        let key_inspect: String = mrb_call_inspect(vm, key.clone())?.as_ref().try_into()?;
        let value_inspect: String = mrb_call_inspect(vm, value.clone())?.as_ref().try_into()?;
        parts.push(format!("{}=>{}", key_inspect, value_inspect));
    }
    let inspect = format!("{{{}}}", parts.join(", "));
    Ok(Rc::new(RObject::string(inspect)))
}

#[test]
fn test_hashing() {
    let vec1 = RObject::string("key".to_string());
    let vec2 = RObject::string("key".to_string()).clone();
    assert_eq!(vec1.as_hash_key(), vec2.as_hash_key());
}

#[test]
fn test_mrb_hash_set_and_index() {
    use crate::yamrb::*;
    let mut vm = VM::empty();
    prelude::prelude(&mut vm);

    let hash = Rc::new(RObject::hash(RHashMap::default()));
    let keys = [
        Rc::new(RObject::string("key".to_string())),
        Rc::new(RObject::integer(1234)),
        Rc::new(RObject::symbol("key2".into())),
    ];
    let values = [
        Rc::new(RObject::integer(1)),
        Rc::new(RObject::integer(2)),
        Rc::new(RObject::integer(42)),
    ];

    for (i, key) in keys.iter().enumerate() {
        let value = &values[i];
        mrb_hash_set_index(hash.clone(), key.clone(), value.clone()).expect("set index failed");
    }

    for (i, key) in keys.iter().enumerate() {
        let value = mrb_hash_get_index(hash.clone(), key.clone()).expect("getting index failed");
        let value: i64 = value.as_ref().try_into().expect("value is not integer");
        let expected: i64 = values[i]
            .as_ref()
            .try_into()
            .expect("expected is not integer");
        assert_eq!(value, expected);
    }
}

#[test]
fn test_mrb_hash_set_and_index_not_found() {
    use crate::yamrb::*;
    let mut vm = VM::empty();
    prelude::prelude(&mut vm);

    let hash = Rc::new(RObject::hash(RHashMap::default()));
    let key = Rc::new(RObject::string("key".to_string()));
    let value = Rc::new(RObject::integer(42));

    mrb_hash_set_index(hash.clone(), key.clone(), value.clone()).expect("set index failed");

    let key = Rc::new(RObject::string("key2".to_string()));
    let value = mrb_hash_get_index(hash.clone(), key.clone()).expect("getting index failed");
    let value = value.as_ref();
    assert!(value.is_nil());
}

fn mrb_hash_size(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let hash = match &this.value {
        RValue::Hash(a) => a,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#size must be called on a hash".to_string(),
            ));
        }
    };
    let hash = hash.borrow();
    Ok(Rc::new(RObject::integer(hash.len() as i64)))
}

// Hash#clear: Removes all key-value pairs from the hash (destructive)
fn mrb_hash_clear(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    this.hash_borrow_mut()?.clear();
    Ok(this)
}

// Hash#dup: Returns a shallow copy of the hash
fn mrb_hash_dup(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let hash = match &this.value {
        RValue::Hash(h) => h.borrow().clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Hash#dup must be called on a hash".to_string(),
            ));
        }
    };
    Ok(Rc::new(RObject::hash(hash)))
}

// Hash#empty?: Returns true if the hash contains no key-value pairs
fn mrb_hash_empty(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#empty? must be called on a hash".to_string(),
            ));
        }
    };
    Ok(Rc::new(RObject::boolean(hash.borrow().is_empty())))
}

// Hash#has_key?: Returns true if the given key is present in the hash
fn mrb_hash_has_key(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let key = args[0].as_hash_key()?;
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#has_key? must be called on a hash".to_string(),
            ));
        }
    };
    Ok(Rc::new(RObject::boolean(hash.borrow().contains_key(&key))))
}

// Hash#has_value?: Returns true if the given value is present for some key in the hash
fn mrb_hash_has_value(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let search_value = &args[0];
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#has_value? must be called on a hash".to_string(),
            ));
        }
    };

    let search_eq = search_value.as_eq_value();
    for (_, (_, value)) in hash.borrow().iter() {
        if value.as_eq_value() == search_eq {
            return Ok(Rc::new(RObject::boolean(true)));
        }
    }
    Ok(Rc::new(RObject::boolean(false)))
}

// Hash#key: Returns the key of an occurrence of a given value
fn mrb_hash_key(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let search_value = &args[0];
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#key must be called on a hash".to_string(),
            ));
        }
    };

    let search_eq = search_value.as_eq_value();
    for (_, (key, value)) in hash.borrow().iter() {
        if value.as_eq_value() == search_eq {
            return Ok(key.clone());
        }
    }
    Ok(Rc::new(RObject::nil()))
}

// Hash#keys: Returns a new array populated with the keys from this hash
fn mrb_hash_keys(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#keys must be called on a hash".to_string(),
            ));
        }
    };

    let keys: Vec<Rc<RObject>> = hash.borrow().values().map(|(k, _)| k.clone()).collect();
    Ok(RObject::array(keys).to_refcount_assigned())
}

// Hash#values: Returns a new array populated with the values from this hash
fn mrb_hash_values(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let hash = match &this.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::RuntimeError(
                "Hash#values must be called on a hash".to_string(),
            ));
        }
    };

    let values: Vec<Rc<RObject>> = hash.borrow().values().map(|(_, v)| v.clone()).collect();
    Ok(RObject::array(values).to_refcount_assigned())
}

// Hash#merge: Returns a new hash containing the contents of other_hash and the contents of self
fn mrb_hash_merge(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let other = &args[0];

    let this_hash = match &this.value {
        RValue::Hash(h) => h.borrow().clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Hash#merge must be called on a hash".to_string(),
            ));
        }
    };

    let other_hash = match &other.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::ArgumentError("argument must be a hash".to_string()));
        }
    };

    let mut result = this_hash;
    for (key_hash, (key, value)) in other_hash.borrow().iter() {
        result.insert(key_hash.clone(), (key.clone(), value.clone()));
    }

    Ok(RObject::hash(result).to_refcount_assigned())
}

// Hash#merge!: Adds the contents of other_hash to self (destructive)
fn mrb_hash_merge_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let other = &args[0];

    let other_hash = match &other.value {
        RValue::Hash(h) => h,
        _ => {
            return Err(Error::ArgumentError("argument must be a hash".to_string()));
        }
    };

    let mut this_hash = this.hash_borrow_mut()?;
    for (key_hash, (key, value)) in other_hash.borrow().iter() {
        this_hash.insert(key_hash.clone(), (key.clone(), value.clone()));
    }
    drop(this_hash);

    Ok(this)
}

// Hash#to_h: Returns self
fn mrb_hash_to_h(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    vm.getself()
}

#[test]
fn test_mrb_hash_size() {
    let mut vm = VM::empty();

    let hash = Rc::new(RObject::hash(RHashMap::default()));
    let key = Rc::new(RObject::string("key".to_string()));
    let value = Rc::new(RObject::integer(42));
    vm.current_regs()[0].replace(hash.clone());

    let size = mrb_hash_size(&mut vm, &[]).expect("getting size failed");
    let size: i64 = size.as_ref().try_into().expect("size is not integer");
    assert_eq!(size, 0);

    mrb_hash_set_index(hash.clone(), key.clone(), value.clone()).expect("set index failed");

    let size = mrb_hash_size(&mut vm, &[]).expect("getting size failed");
    let size: i64 = size.as_ref().try_into().expect("size is not integer");
    assert_eq!(size, 1);
}
