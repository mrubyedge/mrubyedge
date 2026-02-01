use std::rc::Rc;

use crate::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod},
        prelude::object,
        value::{RObject, RSym, RValue},
        vm::VM,
    },
};

use super::array::mrb_array_push;

// Initializes String class and its methods.
pub(crate) fn initialize_string(vm: &mut VM) {
    let string_class = vm.define_standard_class("String");

    mrb_define_class_cmethod(vm, string_class.clone(), "new", Box::new(mrb_string_new));

    mrb_define_cmethod(vm, string_class.clone(), "+", Box::new(mrb_string_add));
    mrb_define_cmethod(vm, string_class.clone(), "*", Box::new(mrb_string_mul));
    mrb_define_cmethod(vm, string_class.clone(), "<<", Box::new(mrb_string_append));
    mrb_define_cmethod(vm, string_class.clone(), "[]", Box::new(mrb_string_slice));
    mrb_define_cmethod(vm, string_class.clone(), "b", Box::new(mrb_string_b));
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "clear",
        Box::new(mrb_string_clear),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "chomp",
        Box::new(mrb_string_chomp),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "chomp!",
        Box::new(mrb_string_chomp_self),
    );
    mrb_define_cmethod(vm, string_class.clone(), "dup", Box::new(mrb_string_dup));
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "empty?",
        Box::new(mrb_string_empty),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "getbyte",
        Box::new(mrb_string_getbyte),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "setbyte",
        Box::new(mrb_string_setbyte),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "index",
        Box::new(mrb_string_index),
    );
    mrb_define_cmethod(vm, string_class.clone(), "ord", Box::new(mrb_string_ord));
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "slice",
        Box::new(mrb_string_slice),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "slice!",
        Box::new(mrb_string_slice_self),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "split",
        Box::new(mrb_string_split),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "lstrip",
        Box::new(mrb_string_lstrip),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "lstrip!",
        Box::new(mrb_string_lstrip_self),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "rstrip",
        Box::new(mrb_string_rstrip),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "rstrip!",
        Box::new(mrb_string_rstrip_self),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "strip",
        Box::new(mrb_string_strip),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "strip!",
        Box::new(mrb_string_strip_self),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "to_sym",
        Box::new(mrb_string_to_sym),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "intern",
        Box::new(mrb_string_to_sym),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "start_with?",
        Box::new(mrb_string_start_with),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "end_with?",
        Box::new(mrb_string_end_with),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "include?",
        Box::new(mrb_string_include),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "bytes",
        Box::new(mrb_string_bytes),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "upcase",
        Box::new(mrb_string_upcase),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "upcase!",
        Box::new(mrb_string_upcase_self),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "downcase",
        Box::new(mrb_string_downcase),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "downcase!",
        Box::new(mrb_string_downcase_self),
    );
    mrb_define_cmethod(vm, string_class.clone(), "to_i", Box::new(mrb_string_to_i));
    mrb_define_cmethod(vm, string_class.clone(), "to_f", Box::new(mrb_string_to_f));
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "unpack",
        Box::new(mrb_string_unpack),
    );
    mrb_define_cmethod(vm, string_class.clone(), "size", Box::new(mrb_string_size));
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "bytesize",
        Box::new(mrb_string_size),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "length",
        Box::new(mrb_string_size),
    );
    mrb_define_cmethod(
        vm,
        string_class.clone(),
        "inspect",
        Box::new(mrb_string_inspect),
    );
    mrb_define_cmethod(vm, string_class.clone(), "to_s", Box::new(object::mrb_self));
}

pub fn mrb_string_inspect(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(format!("\"{}\"", this))))
}

pub fn mrb_string_new(_vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let mut args = args;
    if !args.is_empty() && args.last().unwrap().is_nil() {
        args = &args[..args.len() - 1];
    }
    if args.is_empty() {
        return Ok(Rc::new(RObject::string("".to_string())));
    }
    let s: String = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::string(s)))
}

fn bytes_of<const N: usize>(value: &[u8], cursor: usize) -> Result<[u8; N], Error> {
    if value.len() < cursor + N {
        return Err(Error::RuntimeError("Not enough bytes".to_string()));
    }
    value[cursor..cursor + N]
        .try_into()
        .map_err(|_| Error::RuntimeError(format!("Bit size mismatch: {}", N)))
}

// Represents Ruby's String#unpack method.
// We just support Ruby#pack's format of:
//   - Q: 64-bit unsigned (unsigned long long)
//   - q: 64-bit signed (signed long long)
//   - L: 32-bit unsigned (unsigned long)
//   - l: 32-bit signed (signed long)
//   - I: 32-bit unsigned (unsigned int)
//   - i: 32-bit signed (signed int)
//   - S: 16-bit unsigned (unsigned short)
//   - s: 16-bit signed (signed short)
//   - C: 8-bit unsigned (unsigned char)
//   - c: 8-bit signed (signed char)
// for now.
fn mrb_string_unpack(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let value: Vec<u8> = this.as_ref().try_into()?;
    let format: Vec<u8> = args[0].as_ref().try_into()?;
    let mut cursor: usize = 0;
    let result = Rc::new(RObject::array(Vec::new()));

    for c in format.iter() {
        let value = match c {
            b'Q' => {
                let value = u64::from_le_bytes(bytes_of::<8>(&value, cursor)?);
                cursor += 8;
                value as i64
            }
            b'q' => {
                let value = i64::from_le_bytes(bytes_of::<8>(&value, cursor)?);
                cursor += 8;
                value as i64
            }
            b'L' | b'I' => {
                let value = u32::from_le_bytes(bytes_of::<4>(&value, cursor)?);
                cursor += 4;
                value as i64
            }
            b'l' | b'i' => {
                let value = i32::from_le_bytes(bytes_of::<4>(&value, cursor)?);
                cursor += 4;
                value as i64
            }
            b'S' => {
                let value = u16::from_le_bytes(bytes_of::<2>(&value, cursor)?);
                cursor += 2;
                value as i64
            }
            b's' => {
                let value = i16::from_le_bytes(bytes_of::<2>(&value, cursor)?);
                cursor += 2;
                value as i64
            }
            b'C' => {
                let value = u8::from_le_bytes(bytes_of::<1>(&value, cursor)?);
                cursor += 1;
                value as i64
            }
            b'c' => {
                let value = i8::from_le_bytes(bytes_of::<1>(&value, cursor)?);
                cursor += 1;
                value as i64
            }
            b' ' => {
                // ignore space
                continue;
            }
            _ => {
                return Err(Error::RuntimeError("Unsupported format".to_string()));
            }
        };
        mrb_array_push(result.clone(), &[Rc::new(RObject::integer(value as i64))])?;
    }

    Ok(result)
}

#[test]
fn test_mrb_string_unpack() {
    use crate::yamrb::*;

    let mut vm = VM::empty();
    prelude::prelude(&mut vm);

    let data = Rc::new(RObject::string_from_vec(vec![
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x04, 0x04, 0x03, 0x03, 0x02, 0x02, 0x00, 0x00,
    ]));
    let format = Rc::new(RObject::string("c s l q".to_string()));
    let arg = vec![format];

    let ret = helpers::mrb_funcall(&mut vm, Some(data), "unpack", &arg).expect("unpack failed");

    let answers = [
        0x01,
        0x02 | 0x03 << 8,
        0x04 | 0x05 << 8 | 0x06 << 16 | 0x07 << 24,
        (0x04 | 0x04 << 8 | 0x03 << 16 | 0x03 << 24 | 0x02 << 32 | 0x02 << 40),
    ];

    for (i, expected) in answers.iter().enumerate() {
        let args = vec![Rc::new(RObject::integer(i as i64))];
        let value =
            prelude::array::mrb_array_get_index(ret.clone(), &args).expect("getting index failed");
        let value: i64 = value.as_ref().try_into().expect("value is not integer");
        assert_eq!(value, *expected);
    }
}

fn mrb_string_size(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let value: Vec<u8> = this.as_ref().try_into()?;
    Ok(Rc::new(RObject::integer(value.len() as i64)))
}

fn mrb_string_add(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let other: String = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this + &other)))
}

fn mrb_string_mul(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let times: i64 = args[0].as_ref().try_into()?;
    if times < 0 {
        return Err(Error::ArgumentError("negative argument".to_string()));
    }
    Ok(Rc::new(RObject::string(this.repeat(times as usize))))
}

fn mrb_string_append(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let other: String = args[0].as_ref().try_into()?;
    this.string_borrow_mut()?
        .extend_from_slice(other.as_bytes());
    Ok(this)
}

fn mrb_string_slice(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let chars: Vec<char> = this.chars().collect();

    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments".to_string(),
        ));
    }

    let index: i64 = args[0].as_ref().try_into()?;
    let len = chars.len() as i64;
    let idx = if index < 0 { len + index } else { index };

    if idx < 0 || idx >= len {
        return Ok(Rc::new(RObject::nil()));
    }

    if args.len() == 1 {
        Ok(Rc::new(RObject::string(chars[idx as usize].to_string())))
    } else {
        let length: i64 = args[1].as_ref().try_into()?;
        if length < 0 {
            return Ok(Rc::new(RObject::nil()));
        }
        let end = (idx + length).min(len);
        let result: String = chars[idx as usize..end as usize].iter().collect();
        Ok(Rc::new(RObject::string(result)))
    }
}

fn mrb_string_slice_self(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let chars: Vec<char> = s.chars().collect();

    if args.is_empty() {
        return Err(Error::ArgumentError(
            "wrong number of arguments".to_string(),
        ));
    }

    let args = if args[args.len() - 1].is_nil() {
        &args[..args.len() - 1]
    } else {
        args
    };

    let index: i64 = args[0].as_ref().try_into()?;
    let len = chars.len() as i64;
    let idx = if index < 0 { len + index } else { index };

    if idx < 0 || idx >= len {
        return Ok(Rc::new(RObject::nil()));
    }

    let (removed, remaining) = if args.len() == 1 {
        let removed = chars[idx as usize].to_string();
        let mut remaining_chars = chars.clone();
        remaining_chars.remove(idx as usize);
        let remaining: String = remaining_chars.iter().collect();
        (removed, remaining)
    } else {
        let length: i64 = args[1].as_ref().try_into()?;
        if length < 0 {
            return Ok(Rc::new(RObject::nil()));
        }
        let end = (idx + length).min(len);
        let removed: String = chars[idx as usize..end as usize].iter().collect();
        let mut remaining_chars = chars.clone();
        remaining_chars.drain(idx as usize..end as usize);
        let remaining: String = remaining_chars.iter().collect();
        (removed, remaining)
    };

    *this.string_borrow_mut()? = remaining.as_bytes().to_vec();
    Ok(Rc::new(RObject::string(removed)))
}

// Returns self with UTF-8 flag set to false (binary encoding).
fn mrb_string_b(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;

    if let RValue::String(value, _) = &this.value {
        Ok(RObject::string_from_vec(value.borrow().to_owned()).to_refcount_assigned())
    } else {
        Err(Error::TypeMismatch)
    }
}

fn mrb_string_clear(_vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    Ok(Rc::new(RObject::string(String::new())))
}

fn mrb_string_chomp(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let result = this
        .strip_suffix("\r\n")
        .or_else(|| this.strip_suffix('\n'))
        .or_else(|| this.strip_suffix('\r'))
        .unwrap_or(&this);
    Ok(Rc::new(RObject::string(result.to_string())))
}

fn mrb_string_chomp_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let result = s
        .strip_suffix("\r\n")
        .or_else(|| s.strip_suffix('\n'))
        .or_else(|| s.strip_suffix('\r'))
        .unwrap_or(&s);

    if result.len() != s.len() {
        *this.string_borrow_mut()? = result.as_bytes().to_vec();
        Ok(this)
    } else {
        Ok(Rc::new(RObject::nil()))
    }
}

fn mrb_string_dup(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this)))
}

fn mrb_string_empty(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::boolean(this.is_empty())))
}

fn mrb_string_getbyte(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<u8> = vm.getself()?.as_ref().try_into()?;
    let index: i64 = args[0].as_ref().try_into()?;
    let len = this.len() as i64;
    let idx = if index < 0 { len + index } else { index };

    if idx < 0 || idx >= len {
        return Ok(Rc::new(RObject::nil()));
    }
    Ok(Rc::new(RObject::integer(this[idx as usize] as i64)))
}

fn mrb_string_setbyte(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let mut bytes: Vec<u8> = this.as_ref().try_into()?;
    let index: i64 = args[0].as_ref().try_into()?;
    let value: i64 = args[1].as_ref().try_into()?;

    let len = bytes.len() as i64;
    let idx = if index < 0 { len + index } else { index };

    if idx < 0 || idx >= len {
        return Err(Error::ArgumentError(format!(
            "index {} out of string",
            index
        )));
    }

    if !(0..=255).contains(&value) {
        return Err(Error::ArgumentError(format!("{} out of char range", value)));
    }

    bytes[idx as usize] = value as u8;
    *this.string_borrow_mut()? = bytes;

    Ok(Rc::new(RObject::integer(value)))
}

fn mrb_string_index(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let search: String = args[0].as_ref().try_into()?;

    match this.find(&search) {
        Some(pos) => Ok(Rc::new(RObject::integer(pos as i64))),
        None => Ok(Rc::new(RObject::nil())),
    }
}

fn mrb_string_ord(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    if let Some(ch) = this.chars().next() {
        Ok(Rc::new(RObject::integer(ch as i64)))
    } else {
        Err(Error::ArgumentError("empty string".to_string()))
    }
}

fn mrb_string_split(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;

    let result = if args.is_empty() {
        // Split by whitespace
        this.split_whitespace()
            .map(|s| Rc::new(RObject::string(s.to_string())))
            .collect()
    } else {
        let separator: String = args[0].as_ref().try_into()?;
        this.split(&separator)
            .map(|s| Rc::new(RObject::string(s.to_string())))
            .collect()
    };

    Ok(Rc::new(RObject::array(result)))
}

fn mrb_string_lstrip(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this.trim_start().to_string())))
}

fn mrb_string_lstrip_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let result = s.trim_start();

    if result.len() != s.len() {
        *this.string_borrow_mut()? = result.as_bytes().to_vec();
        Ok(this)
    } else {
        Ok(Rc::new(RObject::nil()))
    }
}

fn mrb_string_rstrip(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this.trim_end().to_string())))
}

fn mrb_string_rstrip_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let result = s.trim_end();

    if result.len() != s.len() {
        *this.string_borrow_mut()? = result.as_bytes().to_vec();
        Ok(this)
    } else {
        Ok(Rc::new(RObject::nil()))
    }
}

fn mrb_string_strip(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this.trim().to_string())))
}

fn mrb_string_strip_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let result = s.trim();

    if result.len() != s.len() {
        *this.string_borrow_mut()? = result.as_bytes().to_vec();
        Ok(this)
    } else {
        Ok(Rc::new(RObject::nil()))
    }
}

fn mrb_string_to_sym(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::symbol(RSym::new(this))))
}

fn mrb_string_start_with(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let prefix: String = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::boolean(this.starts_with(&prefix))))
}

fn mrb_string_end_with(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let suffix: String = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::boolean(this.ends_with(&suffix))))
}

fn mrb_string_include(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let search: String = args[0].as_ref().try_into()?;
    Ok(Rc::new(RObject::boolean(this.contains(&search))))
}

fn mrb_string_bytes(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: Vec<u8> = vm.getself()?.as_ref().try_into()?;
    let result: Vec<Rc<RObject>> = this
        .into_iter()
        .map(|b| Rc::new(RObject::integer(b as i64)))
        .collect();
    Ok(Rc::new(RObject::array(result)))
}

fn mrb_string_upcase(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this.to_uppercase())))
}

fn mrb_string_upcase_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let result = s.to_uppercase();

    if result != s {
        *this.string_borrow_mut()? = result.as_bytes().to_vec();
        Ok(this)
    } else {
        Ok(Rc::new(RObject::nil()))
    }
}

fn mrb_string_downcase(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    Ok(Rc::new(RObject::string(this.to_lowercase())))
}

fn mrb_string_downcase_self(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this = vm.getself()?;
    let s: String = this.as_ref().try_into()?;
    let result = s.to_lowercase();

    if result != s {
        *this.string_borrow_mut()? = result.as_bytes().to_vec();
        Ok(this)
    } else {
        Ok(Rc::new(RObject::nil()))
    }
}

fn mrb_string_to_i(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let trimmed = this.trim();
    let result = trimmed.parse::<i64>().unwrap_or(0);
    Ok(Rc::new(RObject::integer(result)))
}

fn mrb_string_to_f(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let this: String = vm.getself()?.as_ref().try_into()?;
    let trimmed = this.trim();
    let result = trimmed.parse::<f64>().unwrap_or(0.0);
    Ok(Rc::new(RObject::float(result)))
}

#[test]
fn test_mrb_string_size() {
    use crate::yamrb::*;

    let mut vm = VM::empty();

    let data = Rc::new(RObject::string("".into()));
    let ret = helpers::mrb_funcall(&mut vm, Some(data), "size", &[]).expect("size failed");
    let ret: i64 = ret.as_ref().try_into().expect("size is not integer");
    assert_eq!(ret, 0);

    let data = Rc::new(RObject::string("Hello, World".into()));
    let ret = helpers::mrb_funcall(&mut vm, Some(data), "length", &[]).expect("size failed");
    let ret: i64 = ret.as_ref().try_into().expect("size is not integer");
    assert_eq!(ret, 12);
}
