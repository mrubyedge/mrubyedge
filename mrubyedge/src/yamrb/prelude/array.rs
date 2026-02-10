use std::rc::Rc;

use crate::{
    Error,
    yamrb::{
        helpers::{
            self, mrb_call_block, mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall,
        },
        prelude::module::mrb_include_module,
        value::{RObject, RValue},
        vm::VM,
    },
};

pub(crate) fn initialize_array(vm: &mut VM) {
    let array_class = vm.define_standard_class("Array");

    mrb_define_class_cmethod(vm, array_class.clone(), "new", Box::new(mrb_array_new));

    mrb_define_cmethod(vm, array_class.clone(), "+", Box::new(mrb_array_add));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "push",
        Box::new(mrb_array_push_self),
    );
    mrb_define_cmethod(vm, array_class.clone(), "<<", Box::new(mrb_array_push_self));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "[]",
        Box::new(mrb_array_get_index_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "at",
        Box::new(mrb_array_get_index_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "[]=",
        Box::new(mrb_array_set_index_self),
    );
    mrb_define_cmethod(vm, array_class.clone(), "clear", Box::new(mrb_array_clear));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "delete_at",
        Box::new(mrb_array_delete_at),
    );
    mrb_define_cmethod(vm, array_class.clone(), "each", Box::new(mrb_array_each));
    mrb_define_cmethod(vm, array_class.clone(), "empty?", Box::new(mrb_array_empty));
    mrb_define_cmethod(vm, array_class.clone(), "size", Box::new(mrb_array_size));
    mrb_define_cmethod(vm, array_class.clone(), "length", Box::new(mrb_array_size));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "include?",
        Box::new(mrb_array_include),
    );
    mrb_define_cmethod(vm, array_class.clone(), "&", Box::new(mrb_array_and));
    mrb_define_cmethod(vm, array_class.clone(), "|", Box::new(mrb_array_or));
    mrb_define_cmethod(vm, array_class.clone(), "first", Box::new(mrb_array_first));
    mrb_define_cmethod(vm, array_class.clone(), "last", Box::new(mrb_array_last));
    mrb_define_cmethod(vm, array_class.clone(), "pop", Box::new(mrb_array_pop));
    mrb_define_cmethod(vm, array_class.clone(), "shift", Box::new(mrb_array_shift));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "unshift",
        Box::new(mrb_array_unshift),
    );
    mrb_define_cmethod(vm, array_class.clone(), "dup", Box::new(mrb_array_dup));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "uniq!",
        Box::new(mrb_array_uniq_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "map!",
        Box::new(mrb_array_map_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "select!",
        Box::new(mrb_array_select_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "reject!",
        Box::new(mrb_array_reject_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "sort!",
        Box::new(mrb_array_sort_self),
    );
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "sort_by!",
        Box::new(mrb_array_sort_by_self),
    );
    mrb_define_cmethod(vm, array_class.clone(), "pack", Box::new(mrb_array_pack));
    mrb_define_cmethod(
        vm,
        array_class.clone(),
        "inspect",
        Box::new(mrb_array_inspect),
    );
    mrb_define_cmethod(vm, array_class.clone(), "to_s", Box::new(mrb_array_inspect));
    mrb_define_cmethod(vm, array_class.clone(), "join", Box::new(mrb_array_join));

    let enumerable_module = vm.get_module_by_name("Enumerable");
    mrb_include_module(&array_class, enumerable_module).expect("failed to include Enumerable");
}

pub fn mrb_array_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let array: Vec<Rc<RObject>> = this.as_ref().try_into()?;
    let mut s = String::new();
    s.push('[');
    for (i, elem) in array.iter().enumerate() {
        let elem_str: String = helpers::mrb_funcall(vm, Some(elem.clone()), "inspect", &[])?
            .as_ref()
            .try_into()?;
        s.push_str(&elem_str);
        if i + 1 < array.len() {
            s.push_str(", ");
        }
    }
    s.push(']');
    Ok(Rc::new(RObject::string(s)))
}

pub fn mrb_array_new(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let array = if args.is_empty() {
        vec![]
    } else {
        let size: usize = args[0].as_ref().try_into()?;
        {
            let mut v = Vec::with_capacity(size);
            for _ in 0..size {
                v.push(Rc::new(RObject::nil()));
            }
            v
        }
    };
    Ok(Rc::new(RObject::array(array)))
}

fn mrb_array_push_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let args = if args[args.len() - 1].as_ref().is_nil() {
        &args[..args.len() - 1]
    } else {
        args
    };
    mrb_array_push(this, args)
}

pub fn mrb_array_push(this: Rc<RObject>, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let mut array = this.array_borrow_mut()?;
    for arg in args {
        array.push(arg.clone());
    }
    drop(array);
    Ok(this)
}

fn mrb_array_get_index_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    mrb_array_get_index(this, args)
}

pub fn mrb_array_get_index(this: Rc<RObject>, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let array = match &this.value {
        RValue::Array(a) => a.clone(),
        _ => {
            return Err(Error::RuntimeError(
                "Array#push must be called on an Array".to_string(),
            ));
        }
    };
    let index: u32 = args[0].as_ref().try_into()?;
    let value = array.borrow()[index as usize].clone();
    Ok(value)
}

fn mrb_array_set_index_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    mrb_array_set_index(this, args)
}

pub fn mrb_array_set_index(this: Rc<RObject>, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let index: usize = args[0].as_ref().try_into()?;
    let value = &args[1];
    match &this.value {
        RValue::Array(a) => {
            let mut a = a.borrow_mut();
            if a.len() <= index {
                a.insert(index, value.clone());
            } else {
                a[index] = value.clone();
            }
        }
        _ => {
            return Err(Error::RuntimeError(
                "Array#push must be called on an Array".to_string(),
            ));
        }
    };
    Ok(value.clone())
}

fn mrb_array_each(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let block = &args[0];
    match &this.value {
        RValue::Array(a) => {
            let a = a.borrow();
            for elem in a.iter() {
                let args = vec![elem.clone()];
                mrb_call_block(vm, block.clone(), None, &args, 0)?;
            }
        }
        _ => {
            return Err(Error::RuntimeError(
                "Array#each must be called on an Array".to_string(),
            ));
        }
    };
    Ok(this.clone())
}

fn mrb_array_pack(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let format: Vec<u8> = args[0].as_ref().try_into()?;
    let mut buf = vec![];
    match &this.value {
        RValue::Array(a) => {
            let a = a.borrow();
            let mut index: usize = 0;
            for c in format.iter() {
                match c {
                    b'Q' => {
                        let value: u64 = a[index].as_ref().try_into()?;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b'q' => {
                        let value: i64 = a[index].as_ref().try_into()?;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b'L' | b'I' => {
                        let value: u32 = a[index].as_ref().try_into()?;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b'l' | b'i' => {
                        let value: i32 = a[index].as_ref().try_into()?;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b'S' => {
                        let value: u32 = a[index].as_ref().try_into()?;
                        let value = value as u16;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b's' => {
                        let value: i32 = a[index].as_ref().try_into()?;
                        let value = value as i16;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b'C' => {
                        let value: u32 = a[index].as_ref().try_into()?;
                        let value = value as u8;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b'c' => {
                        let value: i32 = a[index].as_ref().try_into()?;
                        let value = value as i8;
                        buf.extend_from_slice(&value.to_le_bytes());
                        index += 1;
                    }
                    b' ' => {
                        // skip
                        continue;
                    }
                    _ => {
                        return Err(Error::RuntimeError("Unsupported format".to_string()));
                    }
                }
            }
        }
        _ => {
            return Err(Error::RuntimeError(
                "Array#pack must be called on an Array".to_string(),
            ));
        }
    };
    let value = Rc::new(RObject::string_from_vec(buf));
    Ok(value)
}

#[test]
fn test_mrb_array_push_and_index() {
    use crate::yamrb::*;
    let mut vm = VM::empty();
    prelude::prelude(&mut vm);

    let array = Rc::new(RObject::array(vec![]));
    let args = vec![
        Rc::new(RObject::integer(1)),
        Rc::new(RObject::integer(2)),
        Rc::new(RObject::integer(3)),
    ];
    mrb_array_push(array.clone(), &args).expect("push failed");

    let answers = [1, 2, 3];

    for (i, expected) in answers.iter().enumerate() {
        let args = vec![Rc::new(RObject::integer(i as i64))];
        let value = mrb_array_get_index(array.clone(), &args).expect("getting index failed");
        let value: i64 = value.as_ref().try_into().expect("value is not integer");
        assert_eq!(value, *expected);
    }
}

#[test]
fn test_mrb_array_set_and_index() {
    use crate::yamrb::*;
    let mut vm = VM::empty();
    prelude::prelude(&mut vm);

    let array = Rc::new(RObject::array(vec![]));
    let args = vec![
        Rc::new(RObject::nil()),
        Rc::new(RObject::nil()),
        Rc::new(RObject::integer(0)),
    ];
    mrb_array_push(array.clone(), &args).expect("push failed");

    let upd_index = Rc::new(RObject::integer(2));
    let newval = Rc::new(RObject::integer(42));
    let args = vec![upd_index, newval];

    mrb_array_set_index(array.clone(), &args).expect("set index failed");

    let value = mrb_array_get_index(array.clone(), &args).expect("getting index failed");
    let value: i64 = value.as_ref().try_into().expect("value is not integer");
    assert_eq!(value, 42);
}

#[test]
fn test_mrb_array_pack() {
    use crate::yamrb::*;
    let mut vm = VM::empty();
    prelude::prelude(&mut vm);

    let array = Rc::new(RObject::array(vec![
        Rc::new(RObject::integer(1)),
        Rc::new(RObject::integer(2)),
        Rc::new(RObject::integer(3)),
        Rc::new(RObject::integer(4)),
    ]));
    vm.current_regs()[0].replace(array);
    let format = Rc::new(RObject::string("c s l q".to_string()));
    let args = vec![format];
    let value = mrb_array_pack(&mut vm, &args).expect("pack failed");

    let expected: Vec<u8> = vec![
        0x01, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let value: Vec<u8> = value.as_ref().try_into().expect("value is not string");
    for (i, v) in value.iter().enumerate() {
        assert_eq!(*v, expected[i]);
    }
}

fn mrb_array_size(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let value: Vec<Rc<RObject>> = this.as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(value.len() as i64)))
}

// Array#+: Returns a new array containing elements from both arrays
fn mrb_array_add(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    let other: Vec<Rc<RObject>> = args[0].as_ref().try_into()?;
    let mut result = this;
    result.extend(other);
    Ok(Rc::new(RObject::array(result)))
}

// Array#clear: Removes all elements from the array (destructive)
fn mrb_array_clear(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    this.array_borrow_mut()?.clear();
    Ok(this)
}

// Array#delete_at: Deletes the element at the specified index (destructive)
fn mrb_array_delete_at(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let index: i64 = args[0].as_ref().try_into()?;
    let mut arr = this.array_borrow_mut()?;
    let len = arr.len() as i64;
    let idx = if index < 0 { len + index } else { index };

    if idx < 0 || idx >= len {
        return Ok(Rc::new(RObject::nil()));
    }

    let removed = arr.remove(idx as usize);
    Ok(removed)
}

// Array#empty?: Returns true if the array contains no elements
fn mrb_array_empty(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::boolean(this.is_empty())))
}

// Array#include?: Returns true if the array contains the given object
fn mrb_array_include(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    let search = &args[0];

    for elem in this.iter() {
        if elem.as_eq_value() == search.as_eq_value() {
            return Ok(Rc::new(RObject::boolean(true)));
        }
    }
    Ok(Rc::new(RObject::boolean(false)))
}

// Array#&: Set intersection - returns a new array containing elements common to both arrays
fn mrb_array_and(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    let other: Vec<Rc<RObject>> = args[0].as_ref().try_into()?;

    let mut result = Vec::new();
    for elem in this.iter() {
        let elem_eq = elem.as_eq_value();
        if other.iter().any(|e| e.as_eq_value() == elem_eq)
            && !result
                .iter()
                .any(|e: &Rc<RObject>| e.as_eq_value() == elem_eq)
        {
            result.push(elem.clone());
        }
    }
    Ok(Rc::new(RObject::array(result)))
}

// Array#|: Set union - returns a new array by joining arrays, excluding duplicates
fn mrb_array_or(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    let other: Vec<Rc<RObject>> = args[0].as_ref().try_into()?;

    let mut result = Vec::new();
    for elem in this.iter().chain(other.iter()) {
        let elem_eq = elem.as_eq_value();
        if !result
            .iter()
            .any(|e: &Rc<RObject>| e.as_eq_value() == elem_eq)
        {
            result.push(elem.clone());
        }
    }
    Ok(Rc::new(RObject::array(result)))
}

// Array#first: Returns the first element, or the first n elements
fn mrb_array_first(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    let args = if !args.is_empty() && args[args.len() - 1].as_ref().is_nil() {
        &args[..args.len() - 1]
    } else {
        args
    };

    if args.is_empty() {
        Ok(this
            .first()
            .cloned()
            .unwrap_or_else(|| Rc::new(RObject::nil())))
    } else {
        let n: i64 = args[0].as_ref().try_into()?;
        if n < 0 {
            return Err(Error::ArgumentError("negative array size".to_string()));
        }
        let n = (n as usize).min(this.len());
        Ok(Rc::new(RObject::array(this[..n].to_vec())))
    }
}

// Array#last: Returns the last element, or the last n elements
fn mrb_array_last(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    let args = if !args.is_empty() && args[args.len() - 1].as_ref().is_nil() {
        &args[..args.len() - 1]
    } else {
        args
    };

    if args.is_empty() {
        Ok(this
            .last()
            .cloned()
            .unwrap_or_else(|| Rc::new(RObject::nil())))
    } else {
        let n: i64 = args[0].as_ref().try_into()?;
        if n < 0 {
            return Err(Error::ArgumentError("negative array size".to_string()));
        }
        let n = (n as usize).min(this.len());
        let start = this.len().saturating_sub(n);
        Ok(Rc::new(RObject::array(this[start..].to_vec())))
    }
}

// Array#pop: Removes and returns the last element (destructive)
fn mrb_array_pop(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let removed = this.array_borrow_mut()?.pop();
    Ok(removed.unwrap_or_else(|| Rc::new(RObject::nil())))
}

// Array#shift: Removes and returns the first element (destructive)
fn mrb_array_shift(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let mut arr = this.array_borrow_mut()?;
    if arr.is_empty() {
        Ok(Rc::new(RObject::nil()))
    } else {
        Ok(arr.remove(0))
    }
}

// Array#unshift: Prepends objects to the front of the array (destructive)
fn mrb_array_unshift(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let mut arr = this.array_borrow_mut()?;
    let args = if args[args.len() - 1].as_ref().is_nil() {
        &args[..args.len() - 1]
    } else {
        args
    };
    for (i, arg) in args.iter().enumerate() {
        arr.insert(i, arg.clone());
    }
    drop(arr);
    Ok(this)
}

// Array#dup: Returns a shallow copy of the array
fn mrb_array_dup(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::array(this)))
}

// Array#uniq!: Removes duplicate elements from self (destructive)
fn mrb_array_uniq_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let arr: Vec<Rc<RObject>> = this.as_ref().try_into()?;

    let unique: Vec<Rc<RObject>> = mrb_funcall(vm, Some(this.clone()), "uniq", &[])?
        .as_ref()
        .try_into()?;

    if unique.len() == arr.len() {
        return Ok(Rc::new(RObject::nil()));
    }

    *this.array_borrow_mut()? = unique;
    Ok(this)
}

// Array#map!: Invokes the given block once for each element, replacing the element with the value returned by the block (destructive)
fn mrb_array_map_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let mapped: Vec<Rc<RObject>> = mrb_funcall(vm, Some(this.clone()), "map", args)?
        .as_ref()
        .try_into()?;

    *this.array_borrow_mut()? = mapped;
    Ok(this)
}

// Array#select!: Invokes the given block for each element, keeping only elements for which the block returns true (destructive)
fn mrb_array_select_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let arr: Vec<Rc<RObject>> = this.as_ref().try_into()?;

    let selected: Vec<Rc<RObject>> = mrb_funcall(vm, Some(this.clone()), "select", args)?
        .as_ref()
        .try_into()?;

    if selected.len() == arr.len() {
        return Ok(Rc::new(RObject::nil()));
    }

    *this.array_borrow_mut()? = selected;
    Ok(this)
}

// Array#reject!: Invokes the given block for each element, removing elements for which the block returns true (destructive)
fn mrb_array_reject_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let arr: Vec<Rc<RObject>> = this.as_ref().try_into()?;

    let rejected: Vec<Rc<RObject>> = mrb_funcall(vm, Some(this.clone()), "delete_if", args)?
        .as_ref()
        .try_into()?;

    if rejected.len() == arr.len() {
        return Ok(Rc::new(RObject::nil()));
    }

    *this.array_borrow_mut()? = rejected;
    Ok(this)
}

// Array#sort!: Sorts the array in place (destructive)
fn mrb_array_sort_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let sorted: Vec<Rc<RObject>> = mrb_funcall(vm, Some(this.clone()), "sort", &[])?
        .as_ref()
        .try_into()?;

    *this.array_borrow_mut()? = sorted;
    Ok(this)
}

// Array#sort_by!: Sorts the array in place by the result of the block (destructive)
fn mrb_array_sort_by_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let sorted: Vec<Rc<RObject>> = mrb_funcall(vm, Some(this.clone()), "sort_by", args)?
        .as_ref()
        .try_into()?;

    *this.array_borrow_mut()? = sorted;
    Ok(this)
}

// Array#join: Returns a string created by converting each element to a string, separated by the given separator
fn mrb_array_join(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<Rc<RObject>> = vm.getself()?.as_ref().try_into()?;

    let separator = if args.is_empty() {
        "".to_string()
    } else {
        args[0].as_ref().try_into()?
    };

    let mut result = String::new();
    for (i, elem) in this.iter().enumerate() {
        let elem_str: String = helpers::mrb_funcall(vm, Some(elem.clone()), "to_s", &[])?
            .as_ref()
            .try_into()?;
        result.push_str(&elem_str);
        if i + 1 < this.len() {
            result.push_str(&separator);
        }
    }

    Ok(Rc::new(RObject::string(result)))
}

#[test]
fn test_mrb_array_size() {
    use crate::yamrb::*;

    let mut vm = VM::empty();

    let data = Rc::new(RObject::array(vec![]));
    let ret = helpers::mrb_funcall(&mut vm, Some(data.clone()), "size", &[]).expect("size failed");
    let ret: i64 = ret.as_ref().try_into().expect("size is not integer");
    assert_eq!(ret, 0);

    mrb_array_push(data.clone(), &[Rc::new(RObject::integer(1))]).expect("push failed");
    mrb_array_push(data.clone(), &[Rc::new(RObject::integer(2))]).expect("push failed");
    mrb_array_push(data.clone(), &[Rc::new(RObject::integer(3))]).expect("push failed");

    let ret = helpers::mrb_funcall(&mut vm, Some(data), "size", &[]).expect("size failed");
    let ret: i64 = ret.as_ref().try_into().expect("size is not integer");
    assert_eq!(ret, 3);
}
