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
    RangeError(String),
    TypeMismatch,
    NoMethodError(String),
    NameError(String),

    Break(Rc<RObject>),
    BlockReturn(usize, Rc<RObject>),
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
            Error::RangeError(msg) => format!("Out of range: {}", msg),
            Error::TypeMismatch => "Type mismatch".to_string(),
            Error::NoMethodError(msg) => format!("Method not found: {}", msg),
            Error::NameError(msg) => format!("Cannot found name: {}", msg),

            Error::Break(_) => "[Break]".to_string(),
            Error::BlockReturn(_, _) => "[BlockReturn]".to_string(),
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
                | (Error::RangeError(_), "RangeError")
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

/// Used for asynchronous or multithreaded context
#[derive(Debug, Clone, PartialEq)]
pub enum StaticError {
    General(String),
}

impl fmt::Display for StaticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error nr {:?}", self)
    }
}

impl error::Error for StaticError {}

unsafe impl Sync for StaticError {}
unsafe impl Send for StaticError {}

impl From<Error> for StaticError {
    fn from(err: Error) -> StaticError {
        match err {
            Error::General => StaticError::General("General error".to_string()),
            Error::Internal(msg) => StaticError::General(format!("[Internal Error] {}", msg)),
            Error::InvalidOpCode => StaticError::General("Invalid opcode".to_string()),
            Error::RuntimeError(msg) => StaticError::General(msg),
            Error::ArgumentError(msg) => StaticError::General(format!("Invalid argument: {}", msg)),
            Error::RangeError(msg) => StaticError::General(format!("Out of range: {}", msg)),
            Error::TypeMismatch => StaticError::General("Type mismatch".to_string()),
            Error::NoMethodError(msg) => StaticError::General(format!("Method not found: {}", msg)),
            Error::NameError(msg) => StaticError::General(format!("Cannot found name: {}", msg)),

            Error::Break(_) => StaticError::General("[Break]".to_string()),
            Error::BlockReturn(_, _) => StaticError::General("[BlockReturn]".to_string()),
        }
    }
}
