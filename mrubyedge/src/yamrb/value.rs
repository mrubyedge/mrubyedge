use std::any::Any;
use std::cell::Cell;
use std::rc::Weak;
use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::Error;
use crate::yamrb::helpers::mrb_call_inspect;

use super::shared_memory::SharedMemory;
use super::vm::{ENV, IREP, VM};

/// Tag that identifies each runtime object variant handled by the VM.
#[derive(Debug, Clone, Copy)]
pub enum RType {
    Bool,
    Symbol,
    Integer,
    Float,
    Class,
    Module,
    Instance,
    Proc,
    Array,
    Hash,
    String,
    Range,
    SharedMemory,
    Data,
    Exception,
    Nil,
}

#[cfg(feature = "mruby-hash-fnv")]
pub type RHashMap<K, V> = fnv::FnvHashMap<K, V>;
#[cfg(feature = "mruby-hash-fnv")]
pub type RHashSet<K> = fnv::FnvHashSet<K>;
#[cfg(feature = "mruby-hash-fnv")]
pub type RHash = fnv::FnvHashMap<ValueHasher, (Rc<RObject>, Rc<RObject>)>;
#[cfg(not(feature = "mruby-hash-fnv"))]
pub type RHashMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(feature = "mruby-hash-fnv"))]
pub type RHashSet<K> = std::collections::HashSet<K>;
#[cfg(not(feature = "mruby-hash-fnv"))]
pub type RHash = std::collections::HashMap<ValueHasher, (Rc<RObject>, Rc<RObject>)>;

/// Actual storage for Ruby values, including boxed objects and immediates.
#[derive(Debug, Clone)]
pub enum RValue {
    Bool(bool),
    Symbol(RSym),
    Integer(i64),
    Float(f64),
    Class(Rc<RClass>),
    Module(Rc<RModule>),
    Instance(RInstance),
    Proc(RProc),
    Array(RefCell<Vec<Rc<RObject>>>),
    Hash(RefCell<RHash>),
    /// (bytes, is_utf8)
    /// FIXME: currently, we compare strings by bytes only, so is_utf8 is unused.
    String(RefCell<Vec<u8>>, Cell<bool>),
    Range(Rc<RObject>, Rc<RObject>, bool),
    SharedMemory(Rc<RefCell<SharedMemory>>),
    Data(Rc<RData>),
    Exception(Rc<RException>),
    Nil,
}

/// Canonical representation used when Ruby objects serve as Hash keys.
/// TODO: This will be used to implement Hash#hash and Hash#eql?.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueHasher {
    Bool(bool),
    Integer(i64),
    Float(Vec<u8>),
    Symbol(String),
    String(Vec<u8>),
    Class(String),
}

/// Normalized form used to compare Ruby values for equality in tests and Hashes.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueEquality {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Symbol(String),
    String(Vec<u8>),
    Class(String),
    Range(Box<ValueEquality>, Box<ValueEquality>, bool),
    Array(Vec<ValueEquality>),
    KeyValue(ValueEqualityForKeyValue),
    ObjectID(u64),
    Nil,
}

/// Key-value specific equality helper storing both keys and resolved values.
#[derive(Debug, Clone)]
pub struct ValueEqualityForKeyValue(RHashSet<ValueHasher>, RHashMap<ValueHasher, ValueEquality>);

impl PartialEq for ValueEqualityForKeyValue {
    fn eq(&self, other: &Self) -> bool {
        if self.0 != other.0 {
            return false;
        }
        for key in self.0.iter() {
            if self.1.get(key) != other.1.get(key) {
                return false;
            }
        }
        true
    }
}

/// Heap-allocated Ruby object wrapper containing type tag, value, and object id.
#[derive(Debug, Clone)]
pub struct RObject {
    pub tt: RType,
    pub value: RValue,
    pub object_id: Cell<u64>,

    pub singleton_class: RefCell<Option<Rc<RClass>>>,

    pub ivar: RefCell<RHashMap<String, Rc<RObject>>>,
}

const UNSET_OBJECT_ID: u64 = u64::MAX;

impl RObject {
    pub fn nil() -> Self {
        RObject {
            tt: RType::Nil,
            value: RValue::Nil,
            object_id: 4.into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn boolean(b: bool) -> Self {
        RObject {
            tt: RType::Bool,
            value: RValue::Bool(b),
            object_id: (if b { 20 } else { 0 }).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn symbol(sym: RSym) -> Self {
        RObject {
            tt: RType::Symbol,
            value: RValue::Symbol(sym),
            object_id: 2.into(), // TODO: calc the same id for the same symbol
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn integer(n: i64) -> Self {
        let object_id = if (i32::MAX as i64) <= n || n <= (i32::MIN as i64) {
            UNSET_OBJECT_ID
        } else {
            (n * 2) as u64 + 1
        };

        RObject {
            tt: RType::Integer,
            value: RValue::Integer(n),
            object_id: object_id.into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn float(f: f64) -> Self {
        RObject {
            tt: RType::Float,
            value: RValue::Float(f),
            object_id: f.to_bits().into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn string(s: String) -> Self {
        RObject {
            tt: RType::String,
            value: RValue::String(RefCell::new(s.into_bytes()), Cell::new(true)),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn string_from_vec(v: Vec<u8>) -> Self {
        RObject {
            tt: RType::String,
            value: RValue::String(RefCell::new(v), Cell::new(false)),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn array(v: Vec<Rc<RObject>>) -> Self {
        RObject {
            tt: RType::Array,
            value: RValue::Array(RefCell::new(v)),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn hash(h: RHash) -> Self {
        RObject {
            tt: RType::Hash,
            value: RValue::Hash(RefCell::new(h)),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn range(start: Rc<RObject>, end: Rc<RObject>, exclusive: bool) -> Self {
        RObject {
            tt: RType::Range,
            value: RValue::Range(start, end, exclusive),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn class(c: Rc<RClass>, vm: &mut VM) -> Rc<Self> {
        match vm.class_object_table.get(&c.full_name()) {
            Some(robj) => robj.clone(),
            None => {
                let robj = Self::newclass(c.clone());
                vm.class_object_table.insert(c.full_name(), robj.clone());
                robj
            }
        }
    }

    fn newclass(c: Rc<RClass>) -> Rc<Self> {
        RObject {
            tt: RType::Class,
            value: RValue::Class(c),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
        .to_refcount_assigned()
    }

    pub fn class_singleton(c: Rc<RClass>, vm: &mut VM) -> Rc<RClass> {
        let class_obj = Self::class(c, vm);
        class_obj.initialize_or_get_singleton_class_for_class(vm)
    }

    pub fn module(m: Rc<RModule>) -> Self {
        RObject {
            tt: RType::Module,
            value: RValue::Module(m),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn class_or_module(c: Rc<RModule>, vm: &mut VM) -> Rc<Self> {
        match c.as_ref().underlying.borrow().as_ref() {
            Some(weak_class) => {
                if let Some(class) = weak_class.upgrade() {
                    RObject::class(class, vm)
                } else {
                    panic!("[BUG] Class weak reference is dead");
                }
            }
            None => Rc::new(RObject::module(c.clone())),
        }
    }

    pub fn instance(c: Rc<RClass>) -> Self {
        RObject {
            tt: RType::Instance,
            value: RValue::Instance(RInstance {
                class: c,
                ref_count: 1,
            }),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn proc(p: RProc) -> Self {
        RObject {
            tt: RType::Proc,
            value: RValue::Proc(p),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn exception(e: Rc<RException>) -> Self {
        RObject {
            tt: RType::Exception,
            value: RValue::Exception(e),
            object_id: (UNSET_OBJECT_ID).into(),
            singleton_class: RefCell::new(None),
            ivar: RefCell::new(RHashMap::default()),
        }
    }

    pub fn to_refcount_assigned(self) -> Rc<Self> {
        let rc = Rc::new(self);
        let id = Rc::as_ptr(&rc) as u64;
        if rc.object_id.get() == UNSET_OBJECT_ID {
            rc.object_id.set(id);
        }
        rc
    }

    pub fn is_falsy(&self) -> bool {
        match self.tt {
            RType::Nil => true,
            RType::Bool => match self.value {
                RValue::Bool(b) => !b,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    pub fn is_nil(&self) -> bool {
        matches!(self.tt, RType::Nil)
    }

    pub fn is_main(&self) -> bool {
        self.object_id.get() == 0
    }

    pub fn set_ivar(&self, key: &str, value: Rc<RObject>) {
        self.ivar.borrow_mut().insert(key.to_string(), value);
    }

    pub fn get_ivar(&self, key: &str) -> Rc<RObject> {
        self.ivar
            .borrow()
            .get(key)
            .cloned()
            .or_else(|| Some(RObject::nil().to_refcount_assigned()))
            .unwrap()
    }

    // TODO: implment Object#hash
    pub fn as_hash_key(&self) -> Result<ValueHasher, Error> {
        match &self.value {
            RValue::Bool(b) => Ok(ValueHasher::Bool(*b)),
            RValue::Integer(i) => Ok(ValueHasher::Integer(*i)),
            RValue::Float(f) => Ok(ValueHasher::Float(f.to_be_bytes().to_vec())),
            RValue::Symbol(s) => Ok(ValueHasher::Symbol(s.name.clone())),
            RValue::String(s, _) => Ok(ValueHasher::String(s.borrow().clone())),
            RValue::Class(c) => Ok(ValueHasher::Class(c.sym_id.name.clone())),
            _ => Err(Error::TypeMismatch),
        }
    }

    pub fn as_eq_value(&self) -> ValueEquality {
        match &self.value {
            RValue::Bool(b) => ValueEquality::Bool(*b),
            RValue::Integer(i) => ValueEquality::Integer(*i),
            RValue::Float(f) => ValueEquality::Float(*f),
            RValue::Symbol(s) => ValueEquality::Symbol(s.name.clone()),
            RValue::String(s, _) => ValueEquality::String(s.borrow().clone()),
            RValue::Class(c) => ValueEquality::Class(c.sym_id.name.clone()),
            RValue::Range(s, e, ex) => {
                ValueEquality::Range(Box::new(s.as_eq_value()), Box::new(e.as_eq_value()), *ex)
            }
            RValue::Array(a) => {
                let arr = a.borrow().iter().map(|v| v.as_eq_value()).collect();
                ValueEquality::Array(arr)
            }
            RValue::Hash(ha) => {
                let keys: RHashSet<_> = ha.borrow().keys().cloned().collect();
                ValueEquality::KeyValue(ValueEqualityForKeyValue(
                    keys,
                    ha.borrow()
                        .iter()
                        .map(|(k, (_, v))| (k.clone(), v.as_ref().as_eq_value()))
                        .collect(),
                ))
            }
            RValue::Nil => ValueEquality::Nil,
            _ => ValueEquality::ObjectID(self.object_id.get()),
        }
    }

    pub fn get_class(&self, vm: &VM) -> Rc<RClass> {
        match &self.value {
            RValue::Class(_) => vm.get_class_by_name("Class"),
            RValue::Module(_) => vm.get_class_by_name("Module"),
            RValue::Instance(i) => i.class.clone(),
            RValue::Bool(b) => {
                if *b {
                    vm.get_class_by_name("TrueClass")
                } else {
                    vm.get_class_by_name("FalseClass")
                }
            }
            RValue::Symbol(_) => vm.get_class_by_name("Symbol"),
            RValue::Integer(_) => vm.get_class_by_name("Integer"),
            RValue::Float(_) => vm.get_class_by_name("Float"),
            RValue::Proc(_) => vm.get_class_by_name("Proc"),
            RValue::Array(_) => vm.get_class_by_name("Array"),
            RValue::Hash(_) => vm.get_class_by_name("Hash"),
            RValue::String(_, _) => vm.get_class_by_name("String"),
            RValue::Range(_, _, _) => vm.get_class_by_name("Range"),
            RValue::SharedMemory(_) => vm.get_class_by_name("SharedMemory"),
            RValue::Data(d) => d.class.clone(),
            RValue::Exception(e) => e.class.clone(),
            RValue::Nil => vm.get_class_by_name("NilClass"),
        }
    }

    pub(crate) fn initialize_or_get_singleton_class(self: &Rc<Self>, vm: &mut VM) -> Rc<RClass> {
        if let Some(sclass) = self.singleton_class.borrow().as_ref() {
            return sclass.clone();
        }

        let class_name = {
            let inspect = mrb_call_inspect(vm, self.clone());
            match inspect {
                Ok(inspect) => inspect
                    .as_ref()
                    .try_into()
                    .unwrap_or_else(|_| "<Singleton Class - unknown inspect type>".to_string()),
                Err(e) => format!("<Singleton Class - inspect error: {:?}>", e),
            }
        };

        let parent_module = self.get_class(vm).parent.borrow().clone();
        let sclass = Rc::new(RClass::new_singleton(
            &class_name,
            Some(self.get_class(vm).clone()),
            parent_module.clone(),
        ));
        sclass.update_module_weakref();

        self.singleton_class.replace(Some(sclass.clone()));
        sclass
    }

    pub(crate) fn initialize_or_get_singleton_class_for_class(
        self: &Rc<Self>,
        vm: &mut VM,
    ) -> Rc<RClass> {
        if self.singleton_class.borrow().is_some() {
            return self.singleton_class.borrow().as_ref().unwrap().clone();
        }

        let class = match &self.value {
            RValue::Class(c) => c.clone(),
            _ => panic!("Not called on a class"),
        };
        let class_name = format!("#<Class:{}>", class.full_name());
        let super_class = match &class.super_class {
            Some(parent) => {
                let parent_obj = RObject::class(parent.clone(), vm);
                parent_obj.initialize_or_get_singleton_class_for_class(vm)
            }
            None => vm.get_class_by_name("Class"),
        };

        let parent_module = self.get_class(vm).parent.borrow().clone();
        let sclass = Rc::new(RClass::new_singleton(
            &class_name,
            Some(super_class),
            parent_module.clone(),
        ));
        sclass.update_module_weakref();

        self.singleton_class.replace(Some(sclass.clone()));
        class
            .singleton_class_ref
            .borrow_mut()
            .replace(sclass.clone());
        sclass
    }

    pub fn singleton_or_this_class(self: &Rc<Self>, vm: &mut VM) -> Rc<RClass> {
        if let Some(sclass) = self.singleton_class.borrow().as_ref() {
            return sclass.clone();
        }
        self.get_class(vm)
    }

    pub fn intern(&self) -> Result<RSym, Error> {
        match &self.value {
            RValue::String(s, _) => Ok(RSym::new(String::from_utf8_lossy(&s.borrow()).to_string())),
            RValue::Symbol(s) => Ok(s.clone()),
            _ => Err(Error::TypeMismatch),
        }
    }

    pub(crate) fn string_borrow_mut(&self) -> Result<std::cell::RefMut<'_, Vec<u8>>, Error> {
        match &self.value {
            RValue::String(s, _) => Ok(s.borrow_mut()),
            _ => Err(Error::TypeMismatch),
        }
    }

    pub(crate) fn array_borrow_mut(
        &self,
    ) -> Result<std::cell::RefMut<'_, Vec<Rc<RObject>>>, Error> {
        match &self.value {
            RValue::Array(arr) => Ok(arr.borrow_mut()),
            _ => Err(Error::TypeMismatch),
        }
    }

    pub(crate) fn hash_borrow_mut(&self) -> Result<std::cell::RefMut<'_, RHash>, Error> {
        match &self.value {
            RValue::Hash(h) => Ok(h.borrow_mut()),
            _ => Err(Error::TypeMismatch),
        }
    }

    pub(crate) fn string_is_utf8(&self) -> Result<bool, Error> {
        match &self.value {
            RValue::String(_, is_utf8) => Ok(is_utf8.get()),
            _ => Err(Error::TypeMismatch),
        }
    }

    pub fn as_vec_owned(&self) -> Result<Vec<Rc<RObject>>, Error> {
        match &self.value {
            RValue::Array(arr) => Ok(arr.borrow().to_owned()),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for i32 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i as i32),
            RValue::Bool(b) => {
                if b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            RValue::Float(f) => Ok(f as i32),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for (i32, i32) {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::Array(ar) => {
                let vec = ar.borrow();
                if vec.len() != 2 {
                    return Err(Error::ArgumentError(
                        "expected array of length 2".to_string(),
                    ));
                }
                let first: i32 = vec[0].as_ref().try_into()?;
                let second: i32 = vec[1].as_ref().try_into()?;
                Ok((first, second))
            }
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for (i32, i32, i32) {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::Array(ar) => {
                let vec = ar.borrow();
                if vec.len() != 3 {
                    return Err(Error::ArgumentError(
                        "expected array of length 3".to_string(),
                    ));
                }
                let first: i32 = vec[0].as_ref().try_into()?;
                let second: i32 = vec[1].as_ref().try_into()?;
                let third: i32 = vec[2].as_ref().try_into()?;
                Ok((first, second, third))
            }
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for (i32, i32, i32, i32) {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::Array(ar) => {
                let vec = ar.borrow();
                if vec.len() != 4 {
                    return Err(Error::ArgumentError(
                        "expected array of length 4".to_string(),
                    ));
                }
                let first: i32 = vec[0].as_ref().try_into()?;
                let second: i32 = vec[1].as_ref().try_into()?;
                let third: i32 = vec[2].as_ref().try_into()?;
                let fourth: i32 = vec[3].as_ref().try_into()?;
                Ok((first, second, third, fourth))
            }
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for u32 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i as u32),
            RValue::Bool(b) => {
                if b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            RValue::Float(f) => Ok(f as u32),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for i64 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i),
            RValue::Bool(b) => {
                if b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            RValue::Float(f) => Ok(f as i64),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for u64 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i as u64),
            RValue::Bool(b) => {
                if b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            RValue::Float(f) => Ok(f as u64),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for usize {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i as usize),
            RValue::Bool(b) => {
                if b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            RValue::Float(f) => Ok(f as usize),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for f32 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i as f32),
            RValue::Bool(b) => {
                if b {
                    Ok(1.0)
                } else {
                    Ok(0.0)
                }
            }
            RValue::Float(f) => Ok(f as f32),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for f64 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match value.value {
            RValue::Integer(i) => Ok(i as f64),
            RValue::Bool(b) => {
                if b {
                    Ok(1.0)
                } else {
                    Ok(0.0)
                }
            }
            RValue::Float(f) => Ok(f),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for bool {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::Bool(b) => Ok(*b),
            RValue::Integer(i) => Ok(*i != 0),
            RValue::Nil => Ok(false),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for String {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::String(s, _) => Ok(String::from_utf8_lossy(&s.borrow()).to_string()),
            RValue::Symbol(sym) => Ok(sym.name.clone()),
            v => Ok(format!("{:?}", v)),
        }
    }
}

impl TryFrom<&RObject> for Vec<u8> {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::String(s, _) => Ok(s.borrow().clone()),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for Vec<Rc<RObject>> {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::Array(a) => Ok(a.borrow().clone()),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for Vec<(Rc<RObject>, Rc<RObject>)> {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::Hash(h) => Ok(h
                .borrow()
                .values()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl TryFrom<&RObject> for () {
    type Error = Error;

    fn try_from(_: &RObject) -> Result<Self, Self::Error> {
        Ok(())
    }
}

impl TryFrom<&RObject> for *mut u8 {
    type Error = Error;

    fn try_from(value: &RObject) -> Result<Self, Self::Error> {
        match &value.value {
            RValue::SharedMemory(sm) => Ok(sm.borrow_mut().as_mut_ptr()),
            _ => Err(Error::TypeMismatch),
        }
    }
}

impl PartialEq for RObject {
    fn eq(&self, other: &Self) -> bool {
        self.object_id.get() == other.object_id.get()
    }
}

/// Ruby module with methods, constants, and mixin relationships.
#[derive(Debug, Clone)]
pub struct RModule {
    pub sym_id: RSym,
    pub procs: RefCell<RHashMap<String, RProc>>,
    pub consts: RefCell<RHashMap<String, Rc<RObject>>>,
    pub mixed_in_modules: RefCell<Vec<Rc<RModule>>>,
    pub parent: RefCell<Option<Rc<RModule>>>,

    pub underlying: RefCell<Option<Weak<RClass>>>,
}

impl RModule {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        RModule {
            sym_id: RSym::new(name),
            procs: RefCell::new(RHashMap::default()),
            consts: RefCell::new(RHashMap::default()),
            mixed_in_modules: RefCell::new(Vec::new()),
            parent: RefCell::new(None),
            underlying: RefCell::new(None),
        }
    }

    pub fn getmcnst(&self, name: &str) -> Option<Rc<RObject>> {
        let consts = self.consts.borrow();
        consts.get(name).cloned()
    }

    // Alias
    pub fn get_const_by_name(&self, name: &str) -> Option<Rc<RObject>> {
        self.getmcnst(name)
    }

    pub fn find_method(&self, name: &str) -> Option<RProc> {
        // First check this module's methods
        let procs = self.procs.borrow();
        if let Some(p) = procs.get(name) {
            return Some(p.clone());
        }
        drop(procs);

        // Then check mixed-in modules
        let mixed_in = self.mixed_in_modules.borrow();
        for module in mixed_in.iter() {
            if let Some(p) = module.find_method(name) {
                return Some(p);
            }
        }

        None
    }

    pub fn full_name(self: &Rc<Self>) -> String {
        let mut names = Vec::new();
        let mut current: Option<Rc<RModule>> = Some(self.clone());
        while let Some(module) = current {
            names.push(module.sym_id.name.clone());
            current = module.parent.borrow().clone();
        }
        names.reverse();
        names.join("::")
    }
}

pub trait AsModule {
    fn as_module(&self) -> Rc<RModule>;
}

impl AsModule for Rc<RModule> {
    fn as_module(&self) -> Rc<RModule> {
        self.clone()
    }
}

impl AsModule for Rc<RClass> {
    fn as_module(&self) -> Rc<RModule> {
        self.module.clone()
    }
}

fn collect_module_chain(
    module: &Rc<RModule>,
    chain: &mut Vec<Rc<RModule>>,
    visited: &mut RHashSet<usize>,
) {
    let key = Rc::as_ptr(module) as usize;
    if !visited.insert(key) {
        return;
    }

    chain.push(module.clone());
    let mixed_in = module.mixed_in_modules.borrow();
    for mixin in mixed_in.iter() {
        collect_module_chain(mixin, chain, visited);
    }
}

impl From<Rc<RModule>> for RObject {
    fn from(value: Rc<RModule>) -> Self {
        RObject::module(value)
    }
}

/// Ruby class metadata, including its module namespace and optional superclass.
/// Attributes and methods required for Class implementation are stored in the associated RModule.
#[derive(Debug, Clone)]
pub struct RClass {
    pub module: Rc<RModule>,
    pub super_class: Option<Rc<RClass>>,
    pub singleton_class_ref: RefCell<Option<Rc<RClass>>>,
    pub is_singleton: bool,
    pub extended_modules: RefCell<Vec<Rc<RModule>>>,
}

impl RClass {
    pub fn new(
        name: &str,
        super_class: Option<Rc<RClass>>,
        parent_module: Option<Rc<RModule>>,
    ) -> Self {
        let module = Rc::new(RModule::new(name));
        let singleton_class_ref = RefCell::new(None);
        if let Some(parent) = parent_module {
            module.parent.replace(Some(parent));
        }
        RClass {
            module,
            super_class,
            singleton_class_ref,
            is_singleton: false,
            extended_modules: RefCell::new(Vec::new()),
        }
    }

    pub fn new_singleton(
        name: &str,
        super_class: Option<Rc<RClass>>,
        parent_module: Option<Rc<RModule>>,
    ) -> Self {
        let module = Rc::new(RModule::new(name));
        let singleton_class_ref = RefCell::new(None);
        if let Some(parent) = parent_module {
            module.parent.replace(Some(parent));
        }
        RClass {
            module,
            super_class,
            singleton_class_ref,
            is_singleton: true,
            extended_modules: RefCell::new(Vec::new()),
        }
    }

    pub fn getmcnst(&self, name: &str) -> Option<Rc<RObject>> {
        self.module.getmcnst(name)
    }

    // find_method will search method from self to superclass
    pub fn find_method(&self, name: &str) -> Option<RProc> {
        // First check this class's module
        if let Some(p) = self.module.find_method(name) {
            return Some(p);
        }

        // For singleton classes, check extended modules
        if self.is_singleton {
            let extended = self.extended_modules.borrow();
            for module in extended.iter() {
                if let Some(p) = module.find_method(name) {
                    return Some(p);
                }
            }
        }

        // Finally check superclass
        match &self.super_class {
            Some(sc) => sc.find_method(name),
            None => None,
        }
    }

    pub fn full_name(&self) -> String {
        self.module.full_name()
    }

    pub(crate) fn update_module_weakref(self: &Rc<Self>) {
        self.module
            .underlying
            .borrow_mut()
            .replace(Rc::downgrade(self));
    }
}

fn collect_class_chain(
    class: &Rc<RClass>,
    chain: &mut Vec<Rc<RModule>>,
    visited: &mut RHashSet<usize>,
) {
    collect_module_chain(&class.module, chain, visited);

    // For singleton classes, include extended modules
    if class.is_singleton {
        let extended = class.extended_modules.borrow();
        for module in extended.iter() {
            collect_module_chain(module, chain, visited);
        }
    }

    if let Some(super_class) = &class.super_class {
        collect_class_chain(super_class, chain, visited);
    }
}

pub(crate) fn build_lookup_chain(class: &Rc<RClass>) -> Vec<Rc<RModule>> {
    let mut chain = Vec::new();
    let mut visited = RHashSet::default();
    collect_class_chain(class, &mut chain, &mut visited);
    chain
}

pub(crate) fn build_module_lookup_chain(module: &Rc<RModule>) -> Vec<Rc<RModule>> {
    let mut chain = Vec::new();
    let mut visited = RHashSet::default();
    collect_module_chain(module, &mut chain, &mut visited);
    chain
}

pub(crate) fn resolve_method(self_class: &Rc<RClass>, name: &str) -> Option<(Rc<RModule>, RProc)> {
    for module in build_lookup_chain(self_class) {
        if let Some(proc) = module.procs.borrow().get(name) {
            return Some((module.clone(), proc.clone()));
        }
    }
    None
}

pub(crate) fn resolve_next_method(
    self_class: &Rc<RClass>,
    name: &str,
    current_owner: &Rc<RModule>,
) -> Option<(Rc<RModule>, RProc)> {
    let mut passed = false;
    for module in build_lookup_chain(self_class) {
        if !passed {
            if Rc::ptr_eq(&module, current_owner) {
                passed = true;
            }
            continue;
        }
        if let Some(proc) = module.procs.borrow().get(name) {
            return Some((module.clone(), proc.clone()));
        }
    }
    None
}

// Provide convenient accessors to module fields
impl std::ops::Deref for RClass {
    type Target = RModule;
    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

/// Backing storage for Ruby object instances (instance variables).
#[derive(Debug, Clone)]
pub struct RInstance {
    pub class: Rc<RClass>,
    pub ref_count: usize,
}

type RDataContainer = Box<dyn Any>;

/// Backing storage for Ruby object instances (instance variables w/ data).
#[derive(Debug, Clone)]
pub struct RData {
    pub class: Rc<RClass>,
    pub data: RefCell<Option<Rc<RDataContainer>>>,
    pub ref_count: usize,
}

/// Ruby procedure (so-called `Proc`) representation (Ruby-defined or native function pointer).
#[derive(Debug, Clone)]
pub struct RProc {
    pub is_rb_func: bool,
    pub is_fnblock: bool,
    pub sym_id: Option<RSym>,
    pub next: Option<Rc<RProc>>,
    pub irep: Option<Rc<IREP>>,
    pub func: Option<usize>,
    pub environ: Option<Rc<ENV>>,
    pub block_self: Option<Rc<RObject>>,
}

/// Native Rust callable used to implement Ruby methods in the VM.
pub type RFn = Box<dyn Fn(&mut VM, &[Rc<RObject>]) -> Result<Rc<RObject>, Error>>;
/// Interned symbol name used across the VM to identify methods and constants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RSym {
    pub name: String,
}

impl RSym {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<&'static str> for RSym {
    fn from(value: &'static str) -> Self {
        Self {
            name: value.to_string(),
        }
    }
}

/// Constant pool entry emitted by mruby bytecode.
/// Strings or binary blobs is supported for now.
#[derive(Debug, Clone)]
pub enum RPool {
    Str(String),
    Data(Vec<u8>),
    Int(i64),
    Float(f64),
}

impl RPool {
    pub fn as_str(&self) -> &str {
        match self {
            RPool::Str(s) => s,
            _ => unreachable!("RPool is not a string...?"),
        }
    }
}

/// Runtime exception object storing class, error payload, and backtrace info.
#[derive(Debug)]
pub struct RException {
    pub class: Rc<RClass>,
    pub error_type: RefCell<Error>,
    pub message: String,
    pub backtrace: Vec<String>, // TODO
}

impl RClass {
    pub fn from_error(vm: &mut VM, e: &Error) -> Rc<Self> {
        match e {
            Error::General => vm.get_class_by_name("Exception"),
            Error::Internal(_) => vm.get_class_by_name("InternalError"),
            Error::InvalidOpCode => vm.get_class_by_name("LoadError"),
            Error::RuntimeError(_) => vm.get_class_by_name("RuntimeError"),
            Error::ArgumentError(_) => vm.get_class_by_name("ArgumentError"),
            Error::RangeError(_) => vm.get_class_by_name("RangeError"),
            Error::TypeMismatch => vm.get_class_by_name("LoadError"),
            Error::NoMethodError(_) => vm.get_class_by_name("NoMethodError"),
            Error::NameError(_) => vm.get_class_by_name("NameError"),
            Error::ZeroDivisionError => vm.get_class_by_name("ZeroDivisionError"),

            Error::Break(_) => vm.get_class_by_name("_Break"),
            Error::BlockReturn(_, _) => vm.get_class_by_name("_BlockReturn"),
        }
    }
}

impl RException {
    pub fn from_error(vm: &mut VM, e: &Error) -> Self {
        RException {
            class: RClass::from_error(vm, e),
            error_type: RefCell::new(e.clone()),
            message: e.message(),
            backtrace: Vec::new(),
        }
    }
}
