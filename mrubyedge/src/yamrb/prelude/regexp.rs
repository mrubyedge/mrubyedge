#[cfg(feature = "mruby-regexp")]
use std::rc::Rc;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use regex::Regex;

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall},
        value::{RData, RObject, RValue},
        vm::VM,
    },
};

pub(crate) fn initialize_regexp(vm: &mut VM) {
    let regexp_class = vm.define_standard_class("Regexp");

    mrb_define_class_cmethod(vm, regexp_class.clone(), "new", Box::new(mrb_regexp_new));
    mrb_define_class_cmethod(
        vm,
        regexp_class.clone(),
        "compile",
        Box::new(mrb_regexp_new),
    );

    mrb_define_cmethod(
        vm,
        regexp_class.clone(),
        "=~",
        Box::new(mrb_regexp_match_tilda),
    );
    mrb_define_cmethod(
        vm,
        regexp_class.clone(),
        "!~",
        Box::new(mrb_regexp_not_match_tilda),
    );
    mrb_define_cmethod(
        vm,
        regexp_class.clone(),
        "match",
        Box::new(mrb_regexp_match),
    );

    let matchdata_class = vm.define_standard_class("MatchData");
    mrb_define_cmethod(
        vm,
        matchdata_class.clone(),
        "[]",
        Box::new(mrb_matchdata_index),
    );

    // Additional counterpart Regexp methods to String
    let string_class = vm.get_class_by_name("String");
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "=~",
        Box::new(mrb_string_regexp_match_tilda),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "!~",
        Box::new(mrb_string_regexp_not_match_tilda),
    );
}

pub struct RRegexp {
    pattern: String,
}

pub struct RMatchData {
    captures: Vec<(usize, usize)>,
    haystack: String,
}

fn get_regexp_from_object(obj: &Rc<RObject>) -> Result<Regex, Error> {
    let pattern_str: String = match &obj.value {
        RValue::Data(data) => {
            let borrow = data.data.borrow();
            let any_ref = borrow
                .as_ref()
                .ok_or_else(|| Error::RuntimeError("Invalid Regexp data".to_string()))?;
            let regexp = any_ref
                .downcast_ref::<RRegexp>()
                .ok_or_else(|| Error::RuntimeError("Invalid Regexp data".to_string()))?;
            regexp.pattern.clone()
        }
        _ => {
            return Err(Error::RuntimeError(
                "Regexp#=~ must be called on a Regexp".to_string(),
            ));
        }
    };

    let pattern = Regex::new(&pattern_str).map_err(|e| {
        Error::RuntimeError(format!("Invalid regexp pattern in Regexp#=~: {:?}", e))
    })?;
    Ok(pattern)
}

pub fn mrb_regexp_new(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let pattern_obj = args[0].clone();
    match &pattern_obj.value {
        RValue::String(pattern) => {
            // For simplicity, we only support literal patterns without options.
            let pattern_str = pattern.clone().borrow().to_owned();
            let regexp = RRegexp {
                pattern: String::from_utf8(pattern_str).map_err(|e| {
                    Error::RuntimeError(format!("Invalid regexp expression: {:?}", e))
                })?,
            };
            let regexp_data = Rc::new(RData {
                class: vm.get_class_by_name("Regexp"),
                ivar: RefCell::new(HashMap::new()),
                data: RefCell::new(Some(Rc::new(Box::new(regexp) as Box<dyn std::any::Any>))),
                ref_count: 1,
            });
            Ok(RObject {
                tt: crate::yamrb::value::RType::Data,
                value: RValue::Data(regexp_data),
                object_id: Cell::new(0),
                singleton_class: RefCell::new(None),
            }
            .to_refcount_assigned())
        }
        _ => Err(Error::RuntimeError(
            "Regexp.new requires a string pattern".to_string(),
        )),
    }
}

fn mrb_regexp_match_tilda(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let regexp_obj = vm.getself()?;
    let target_obj = args[0].clone();

    let regexp = get_regexp_from_object(&regexp_obj)?;

    let target_str = match &target_obj.value {
        RValue::String(s) => s.clone().borrow().to_owned(),
        _ => {
            return Err(Error::RuntimeError(
                "Regexp#=~ requires a string argument".to_string(),
            ));
        }
    };

    let haystack = String::from_utf8(target_str).map_err(|e| {
        Error::RuntimeError(format!("Invalid string argument for Regexp#=~: {:?}", e))
    })?;

    match regexp.find(&haystack) {
        Some(matched) => Ok(RObject::integer(matched.start() as i64).to_refcount_assigned()),
        None => Ok(RObject::nil().to_refcount_assigned()),
    }
}

fn mrb_regexp_not_match_tilda(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    match mrb_regexp_match_tilda(vm, args)? {
        res if res.is_nil() => Ok(RObject::boolean(true).to_refcount_assigned()),
        _ => Ok(RObject::boolean(false).to_refcount_assigned()),
    }
}

fn mrb_regexp_match(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let regexp_obj = vm.getself()?;
    let target_obj = args[0].clone();

    let regexp = get_regexp_from_object(&regexp_obj)?;

    let target_str = match &target_obj.value {
        RValue::String(s) => s.clone().borrow().to_owned(),
        _ => {
            return Err(Error::RuntimeError(
                "Regexp#match requires a string argument".to_string(),
            ));
        }
    };

    let haystack = String::from_utf8(target_str).map_err(|e| {
        Error::RuntimeError(format!("Invalid string argument for Regexp#match: {:?}", e))
    })?;

    match regexp.captures(&haystack) {
        Some(captures) => {
            let mut caps_vec = Vec::new();
            for cap in captures.iter() {
                if let Some(m) = cap {
                    caps_vec.push((m.start(), m.end()));
                } else {
                    caps_vec.push((usize::MAX, usize::MAX)); // Indicate no match
                }
            }
            let matchdata = RMatchData {
                captures: caps_vec,
                haystack,
            };
            let matchdata_data = Rc::new(RData {
                class: vm.get_class_by_name("MatchData"),
                ivar: RefCell::new(HashMap::new()),
                data: RefCell::new(Some(Rc::new(Box::new(matchdata) as Box<dyn std::any::Any>))),
                ref_count: 1,
            });
            Ok(RObject {
                tt: crate::yamrb::value::RType::Data,
                value: RValue::Data(matchdata_data),
                object_id: Cell::new(0),
                singleton_class: RefCell::new(None),
            }
            .to_refcount_assigned())
        }
        None => Ok(RObject::nil().to_refcount_assigned()),
    }
}

fn mrb_matchdata_index(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let matchdata_obj = vm.getself()?;
    let index_obj = args[0].clone();

    let index = match &index_obj.value {
        RValue::Integer(i) => *i as isize,
        _ => {
            return Err(Error::ArgumentError(
                "MatchData#[] requires an integer index".to_string(),
            ));
        }
    };

    let matchdata = match &matchdata_obj.value {
        RValue::Data(data) => {
            let borrow = data.data.borrow();
            let any_ref = borrow
                .as_ref()
                .ok_or_else(|| Error::RuntimeError("Invalid MatchData data".to_string()))?;
            any_ref.clone()
        }
        _ => {
            return Err(Error::RuntimeError(
                "MatchData#[] must be called on a MatchData".to_string(),
            ));
        }
    };
    let matchdata = matchdata
        .downcast_ref::<RMatchData>()
        .ok_or_else(|| Error::RuntimeError("Invalid MatchData data".to_string()))?;

    let cap_index = if index < 0 {
        (matchdata.captures.len() as isize + index) as usize
    } else {
        index as usize
    };

    if cap_index >= matchdata.captures.len() {
        return Ok(RObject::nil().to_refcount_assigned());
    }
    let (start, end) = matchdata.captures[cap_index];
    if start == usize::MAX && end == usize::MAX {
        return Ok(RObject::nil().to_refcount_assigned());
    }
    let matched_str = &matchdata.haystack[start..end];
    Ok(RObject::string(matched_str.to_string()).to_refcount_assigned())
}

fn mrb_string_regexp_match_tilda(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let string_obj = vm.getself()?;
    let regexp_obj = args[0].clone();
    mrb_funcall(vm, Some(regexp_obj), "=~", &[string_obj])
}

fn mrb_string_regexp_not_match_tilda(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    let string_obj = vm.getself()?;
    let regexp_obj = args[0].clone();
    mrb_funcall(vm, Some(regexp_obj), "!~", &[string_obj])
}
