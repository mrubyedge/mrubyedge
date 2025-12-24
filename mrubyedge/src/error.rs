use std::error;
use std::fmt;
use std::rc::Rc;

use crate::yamrb::value::RClass;
use crate::yamrb::value::RObject;
use crate::yamrb::vm::VM;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    General,
    Internal(String),
    InvalidOpCode,
    RuntimeError(String),
    ArgumentError(String),
    TypeMismatch,
    NoMethodError(String),
    NameError(String),

    Break(Rc<RObject>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error nr {:?}", self)
    }
}

impl error::Error for Error {}

impl Error {
    pub fn internal(msg: impl Into<String>) -> Error {
        Error::Internal(msg.into())
    }

    pub fn message(&self) -> String {
        match self {
            Error::General => "General error".to_string(),
            Error::Internal(msg) => format!("[Internal Error] {}", msg.clone()),
            Error::InvalidOpCode => "Invalid opcode".to_string(),
            Error::RuntimeError(msg) => msg.clone(),
            Error::ArgumentError(msg) => format!("Invalid argument: {}", msg),
            Error::TypeMismatch => "Type mismatch".to_string(),
            Error::NoMethodError(msg) => format!("Method not found: {}", msg),
            Error::NameError(msg) => format!("Cannot found name: {}", msg),

            Error::Break(_) => "[Break]".to_string(),
        }
    }

    pub fn is_instance_of(&self, other: Rc<RClass>) -> bool {
        matches!(
            (self, other.sym_id.name.as_str()),
            (Error::General, "StandardError")
                | (Error::Internal(_), "InternalError")
                | (Error::InvalidOpCode, "StandardError")
                | (Error::RuntimeError(_), "RuntimeError")
                | (Error::ArgumentError(_), "ArgumentError")
                | (Error::TypeMismatch, "StandardError")
                | (Error::NoMethodError(_), "NoMethodError")
                | (Error::NameError(_), "NameError")
        )
    }

    pub fn is_a(&self, vm: &mut VM, other: Rc<RClass>) -> bool {
        let mut klass: Option<Rc<RClass>> = RClass::from_error(vm, self).into();
        let target_klass_name = other.sym_id.name.as_str();

        while let Some(k) = klass {
            if k.sym_id.name.as_str() == target_klass_name {
                return true;
            }
            klass = k.super_class.clone();
        }
        false
    }
}
