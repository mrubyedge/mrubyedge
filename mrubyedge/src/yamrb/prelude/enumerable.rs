use std::{cell::Cell, rc::Rc};

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_call_block, mrb_define_module_cmethod, mrb_funcall},
        value::{RFn, RObject, RProc},
        vm::VM,
    },
};

pub(crate) fn initialize_enumerable(vm: &mut VM) {
    let enumerable_module = vm.define_module("Enumerable", None);

    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "map",
        Box::new(mrb_enumerable_map),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "find",
        Box::new(mrb_enumerable_find),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "select",
        Box::new(mrb_enumerable_select),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "all?",
        Box::new(mrb_enumerable_all),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "any?",
        Box::new(mrb_enumerable_any),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "delete_if",
        Box::new(mrb_enumerable_delete_if),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "each_with_index",
        Box::new(mrb_enumerable_each_with_index),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "sort",
        Box::new(mrb_enumerable_sort),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "sort_by",
        Box::new(mrb_enumerable_sort_by),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "max",
        Box::new(mrb_enumerable_max),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "min",
        Box::new(mrb_enumerable_min),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "minmax",
        Box::new(mrb_enumerable_minmax),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "compact",
        Box::new(mrb_enumerable_compact),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "count",
        Box::new(mrb_enumerable_count),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "to_a",
        Box::new(mrb_enumerable_to_a),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "uniq",
        Box::new(mrb_enumerable_uniq),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "reduce",
        Box::new(mrb_enumerable_reduce),
    );
    mrb_define_module_cmethod(
        vm,
        enumerable_module.clone(),
        "sum",
        Box::new(mrb_enumerable_sum),
    );
}

fn rproc_from_rust_block(vm: &mut VM, rfn: RFn) -> Result<Rc<RObject>, Error> {
    vm.push_fnblock(Rc::new(rfn))?;
    let block = RProc {
        is_rb_func: false,
        is_fnblock: true,
        sym_id: None,
        next: None,
        irep: None,
        func: None,
        environ: None,
        block_self: vm.getself().ok(),
    };
    Ok(RObject::proc(block).to_refcount_assigned())
}

// Enumerable#to_a: Returns an array containing all elements
fn mrb_enumerable_to_a(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let results: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let results_ref = results.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        mrb_funcall(
            vm,
            Some(results_ref.clone()),
            "push",
            std::slice::from_ref(&args[0]),
        )?;
        Ok(Rc::new(RObject::nil()))
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(results)
}

// Enumerable#map: Returns a new array with the results of running block once for every element
fn mrb_enumerable_map(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let results: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let results_ref = results.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        mrb_funcall(
            vm,
            Some(results_ref.clone()),
            "push",
            std::slice::from_ref(&result),
        )?;
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(results)
}

// Enumerable#find: Returns the first element for which the block returns true
fn mrb_enumerable_find(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let found = Cell::new(false);
    let result_box: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let result_box_ref = result_box.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        if found.get() {
            return Ok(Rc::new(RObject::nil()));
        }

        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        if result.is_truthy() {
            mrb_funcall(
                vm,
                Some(result_box_ref.clone()),
                "push",
                std::slice::from_ref(&args[0]),
            )?;
            found.set(true);
        }
        Ok(result)
    });
    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    let found = mrb_funcall(vm, result_box.into(), "pop", &[])?;
    Ok(found)
}

// Enumerable#select: Returns a new array containing all elements for which the block returns true
fn mrb_enumerable_select(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let results: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let results_ref = results.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        if result.is_truthy() {
            mrb_funcall(
                vm,
                Some(results_ref.clone()),
                "push",
                std::slice::from_ref(&args[0]),
            )?;
        }
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(results)
}

// Enumerable#all?: Returns true if all elements match the condition
fn mrb_enumerable_all(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let all_true = Rc::new(Cell::new(true));
    let all_true_ref = all_true.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        if !all_true_ref.get() {
            return Ok(Rc::new(RObject::nil()));
        }

        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        if !result.is_truthy() {
            all_true_ref.set(false);
        }
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(Rc::new(RObject::boolean(all_true.get())))
}

// Enumerable#any?: Returns true if any element matches the condition
fn mrb_enumerable_any(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let found_true = Rc::new(Cell::new(false));
    let found_true_ref = found_true.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        if found_true_ref.get() {
            return Ok(Rc::new(RObject::nil()));
        }

        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        if result.is_truthy() {
            found_true_ref.set(true);
        }
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(Rc::new(RObject::boolean(found_true.get())))
}

// Enumerable#delete_if: Deletes every element for which block evaluates to true
fn mrb_enumerable_delete_if(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let results: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let results_ref = results.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let block = original_block.clone();
        let result = mrb_call_block(vm, block, None, args, 0)?;
        if !result.is_truthy() {
            mrb_funcall(
                vm,
                Some(results_ref.clone()),
                "push",
                std::slice::from_ref(&args[0]),
            )?;
        }
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(results)
}

// Enumerable#each_with_index: Calls block with two arguments, the item and its index
fn mrb_enumerable_each_with_index(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;
    let index = Rc::new(Cell::new(0i64));
    let index_ref = index.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let block = original_block.clone();
        let idx = index_ref.get();
        let index_obj = Rc::new(RObject::integer(idx));
        let block_args = vec![args[0].clone(), index_obj];
        let result = mrb_call_block(vm, block, None, &block_args, 0)?;
        index_ref.set(idx + 1);
        Ok(result)
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(this)
}

// Enumerable#sort: Returns an array with sorted elements
fn mrb_enumerable_sort(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let array = mrb_funcall(vm, Some(this), "to_a", &[])?;
    let mut collected: Vec<Rc<RObject>> = array.as_ref().try_into()?;

    collected.sort_by(|a, b| {
        let args = vec![b.clone()];
        let cmp_result = mrb_funcall(vm, Some(a.clone()), "<=>", &args);
        match cmp_result {
            Ok(cmp_obj) => {
                let cmp_val: Result<i64, _> = cmp_obj.as_ref().try_into();
                match cmp_val {
                    Ok(v) => v.cmp(&0),
                    Err(_) => std::cmp::Ordering::Equal,
                }
            }
            Err(_) => std::cmp::Ordering::Equal,
        }
    });

    Ok(RObject::array(collected).to_refcount_assigned())
}

// Enumerable#sort_by: Returns an array with elements sorted by the block's return value
fn mrb_enumerable_sort_by(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let original_block = args
        .last()
        .cloned()
        .ok_or_else(|| Error::ArgumentError("block should be specified".to_string()))?;

    // Collect elements first using to_a
    let this = vm.getself()?;
    let array = mrb_funcall(vm, Some(this), "to_a", &[])?;
    let elements: Vec<Rc<RObject>> = array.as_ref().try_into()?;

    // Collect keys by calling the block on each element
    let mut sort_keys: Vec<Rc<RObject>> = Vec::new();
    for elem in &elements {
        let key = mrb_call_block(
            vm,
            original_block.clone(),
            None,
            std::slice::from_ref(elem),
            0,
        )?;
        sort_keys.push(key);
    }

    let mut elements = elements;
    let mut sort_keys = sort_keys;

    let mut pairs: Vec<(Rc<RObject>, Rc<RObject>)> =
        elements.drain(..).zip(sort_keys.drain(..)).collect();

    pairs.sort_by(|a, b| {
        let args = vec![b.1.clone()];
        let cmp_result = mrb_funcall(vm, Some(a.1.clone()), "<=>", &args);
        match cmp_result {
            Ok(cmp_obj) => {
                let cmp_val: Result<i64, _> = cmp_obj.as_ref().try_into();
                match cmp_val {
                    Ok(v) => v.cmp(&0),
                    Err(_) => std::cmp::Ordering::Equal,
                }
            }
            Err(_) => std::cmp::Ordering::Equal,
        }
    });

    let sorted: Vec<Rc<RObject>> = pairs.into_iter().map(|(elem, _)| elem).collect();
    Ok(RObject::array(sorted).to_refcount_assigned())
}

// Enumerable#max: Returns the maximum element
fn mrb_enumerable_max(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let array = mrb_funcall(vm, Some(this), "to_a", &[])?;
    let collected: Vec<Rc<RObject>> = array.as_ref().try_into()?;

    if collected.is_empty() {
        return Ok(Rc::new(RObject::nil()));
    }

    let mut max = collected[0].clone();
    for elem in collected.iter().skip(1) {
        let args = vec![elem.clone()];
        let cmp: i64 = mrb_funcall(vm, Some(max.clone()), "<=>", &args)?
            .as_ref()
            .try_into()?;
        if cmp < 0 {
            max = elem.clone();
        }
    }
    Ok(max)
}

// Enumerable#min: Returns the minimum element
fn mrb_enumerable_min(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let array = mrb_funcall(vm, Some(this), "to_a", &[])?;
    let collected: Vec<Rc<RObject>> = array.as_ref().try_into()?;

    if collected.is_empty() {
        return Ok(Rc::new(RObject::nil()));
    }

    let mut min = collected[0].clone();
    for elem in collected.iter().skip(1) {
        let args = vec![elem.clone()];
        let cmp: i64 = mrb_funcall(vm, Some(min.clone()), "<=>", &args)?
            .as_ref()
            .try_into()?;
        if cmp > 0 {
            min = elem.clone();
        }
    }
    Ok(min)
}

// Enumerable#minmax: Returns a two-element array containing the minimum and maximum
fn mrb_enumerable_minmax(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let array = mrb_funcall(vm, Some(this), "to_a", &[])?;
    let collected: Vec<Rc<RObject>> = array.as_ref().try_into()?;

    if collected.is_empty() {
        return Ok(
            RObject::array(vec![Rc::new(RObject::nil()), Rc::new(RObject::nil())])
                .to_refcount_assigned(),
        );
    }

    let mut min = collected[0].clone();
    let mut max = collected[0].clone();

    for elem in collected.iter().skip(1) {
        let args = vec![elem.clone()];
        let cmp_min: i64 = mrb_funcall(vm, Some(min.clone()), "<=>", &args)?
            .as_ref()
            .try_into()?;
        if cmp_min > 0 {
            min = elem.clone();
        }

        let args = vec![elem.clone()];
        let cmp_max: i64 = mrb_funcall(vm, Some(max.clone()), "<=>", &args)?
            .as_ref()
            .try_into()?;
        if cmp_max < 0 {
            max = elem.clone();
        }
    }

    Ok(RObject::array(vec![min, max]).to_refcount_assigned())
}

// Enumerable#compact: Returns a new array with nil values removed
fn mrb_enumerable_compact(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let results: Rc<RObject> = RObject::array(vec![]).to_refcount_assigned();
    let results_ref = results.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        if !args[0].is_nil() {
            mrb_funcall(
                vm,
                Some(results_ref.clone()),
                "push",
                std::slice::from_ref(&args[0]),
            )?;
        }
        Ok(Rc::new(RObject::nil()))
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    Ok(results)
}

// Enumerable#count: Returns the number of elements (with optional condition)
fn mrb_enumerable_count(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let count = Rc::new(Cell::new(0i64));

    if args.is_empty() {
        // Count all elements
        let count_ref = count.clone();
        let wrapping_block: RFn = Box::new(move |_vm: &mut VM, _args: &[Rc<RObject>]| {
            count_ref.set(count_ref.get() + 1);
            Ok(Rc::new(RObject::nil()))
        });

        let this = vm.getself()?;
        let block = rproc_from_rust_block(vm, wrapping_block)?;
        mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
        vm.pop_fnblock()?;
    } else {
        // Count elements matching the block condition
        let count_ref = count.clone();
        let original_block = args.last().cloned().unwrap();
        let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
            let block = original_block.clone();
            let result = mrb_call_block(vm, block, None, args, 0)?;
            if result.is_truthy() {
                count_ref.set(count_ref.get() + 1);
            }
            Ok(result)
        });

        let this = vm.getself()?;
        let block = rproc_from_rust_block(vm, wrapping_block)?;
        mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
        vm.pop_fnblock()?;
    }

    Ok(Rc::new(RObject::integer(count.get())))
}

// Enumerable#uniq: Returns a new array with duplicate values removed
fn mrb_enumerable_uniq(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let array = mrb_funcall(vm, Some(this), "to_a", &[])?;
    let collected: Vec<Rc<RObject>> = array.as_ref().try_into()?;

    let mut result = Vec::new();
    for elem in collected.iter() {
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

// Enumerable#reduce: Combines all elements by applying a binary operation
fn mrb_enumerable_reduce(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    // Check if we have an initial value or just a block
    let (initial_value, original_block) = if args.len() == 2 {
        // Initial value provided: reduce(initial) { |acc, elem| ... }
        (Some(args[0].clone()), args[1].clone())
    } else if args.len() == 1 {
        // No initial value: reduce { |acc, elem| ... }
        (None, args[0].clone())
    } else {
        return Err(Error::ArgumentError(
            "wrong number of arguments".to_string(),
        ));
    };

    let accumulator: Rc<RObject> = if let Some(init) = initial_value {
        RObject::array(vec![init]).to_refcount_assigned()
    } else {
        RObject::array(vec![]).to_refcount_assigned()
    };

    let acc_ref = accumulator.clone();
    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let current_elem = args[0].clone();
        let acc_array: Vec<Rc<RObject>> = acc_ref.as_ref().try_into()?;

        if acc_array.is_empty() {
            // First element becomes the initial accumulator
            mrb_funcall(
                vm,
                Some(acc_ref.clone()),
                "push",
                std::slice::from_ref(&current_elem),
            )?;
        } else {
            // Call block with (accumulator, element)
            let current_acc = acc_array[0].clone();
            let block = original_block.clone();
            let result = mrb_call_block(vm, block, None, &[current_acc, current_elem], 0)?;

            // Update accumulator
            mrb_funcall(vm, Some(acc_ref.clone()), "pop", &[])?;
            mrb_funcall(
                vm,
                Some(acc_ref.clone()),
                "push",
                std::slice::from_ref(&result),
            )?;
        }
        Ok(Rc::new(RObject::nil()))
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    // Return the final accumulator value
    let result = mrb_funcall(vm, Some(accumulator), "first", &[])?;
    Ok(result)
}

// Enumerable#sum: Returns the sum of all elements
fn mrb_enumerable_sum(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    // Check if we have an initial value
    let initial_value = if args.is_empty() || args[0].is_nil() {
        // Default initial value is 0
        Rc::new(RObject::integer(0))
    } else {
        // Initial value provided: sum(init)
        args[0].clone()
    };

    let accumulator: Rc<RObject> = RObject::array(vec![initial_value]).to_refcount_assigned();
    let acc_ref = accumulator.clone();

    let wrapping_block: RFn = Box::new(move |vm: &mut VM, args: &[Rc<RObject>]| {
        let current_elem = args[0].clone();
        let acc_array: Vec<Rc<RObject>> = acc_ref.as_ref().try_into()?;
        let current_acc = acc_array[0].clone();

        // Call + operator on accumulator with current element
        let result = mrb_funcall(vm, Some(current_acc), "+", &[current_elem])?;

        // Update accumulator
        mrb_funcall(vm, Some(acc_ref.clone()), "pop", &[])?;
        mrb_funcall(
            vm,
            Some(acc_ref.clone()),
            "push",
            std::slice::from_ref(&result),
        )?;
        Ok(Rc::new(RObject::nil()))
    });

    let this = vm.getself()?;
    let block = rproc_from_rust_block(vm, wrapping_block)?;
    mrb_funcall(vm, Some(this.clone()), "each", &[block])?;
    vm.pop_fnblock()?;

    // Return the final accumulator value
    let result = mrb_funcall(vm, Some(accumulator), "first", &[])?;
    Ok(result)
}
