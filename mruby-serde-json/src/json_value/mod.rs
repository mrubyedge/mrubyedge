use std::cell::RefCell;
use std::rc::Rc;

use mrubyedge::Error;
use mrubyedge::yamrb::helpers::mrb_funcall;
use mrubyedge::yamrb::value::RObject;
use mrubyedge::yamrb::value::RValue;
use mrubyedge::yamrb::vm::VM;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

pub struct Json<'a> {
    mrb: Rc<RefCell<&'a mut VM>>,
    inner: Rc<RObject>,
}

impl<'a> Json<'a> {
    pub fn get_inner(&self) -> Rc<RObject> {
        self.inner.clone()
    }

    pub fn from_robject(mrb: Rc<RefCell<&'a mut VM>>, inner: Rc<RObject>) -> Self {
        Self { mrb, inner }
    }
}

impl<'a> From<Json<'a>> for Rc<RObject> {
    fn from(value: Json<'a>) -> Self {
        value.get_inner()
    }
}

impl<'a> Serialize for Json<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.get_inner().value {
            RValue::Nil => serializer.serialize_none(),
            RValue::Bool(b) => serializer.serialize_bool(b),
            RValue::Integer(i) => serializer.serialize_i64(i),
            RValue::Float(f) => serializer.serialize_f64(f),
            RValue::String(ref s, _) => {
                serializer.serialize_str(&String::from_utf8_lossy(&s.borrow()))
            }
            RValue::Symbol(ref s) => serializer.serialize_str(&s.name),
            RValue::Array(ref arr) => {
                let arr = arr.borrow();
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr.iter() {
                    let json_item = Json::from_robject(self.mrb.clone(), item.clone());
                    seq.serialize_element(&json_item)?;
                }
                seq.end()
            }
            RValue::Hash(ref hash) => {
                let hash = hash.borrow();
                let mut map = serializer.serialize_map(Some(hash.len()))?;
                for (_, (key, value)) in hash.iter() {
                    let key_str = match key.value {
                        RValue::String(ref s, _) => {
                            String::from_utf8_lossy(&s.borrow()).to_string()
                        }
                        RValue::Symbol(ref s) => s.name.to_string(),
                        _ => {
                            return Err(serde::ser::Error::custom("Non-string key in JSON object"));
                        }
                    };
                    let json_value = Json::from_robject(self.mrb.clone(), value.clone());
                    map.serialize_entry(&key_str, &json_value)?;
                }
                map.end()
            }
            _ => {
                let obj = self.get_inner();
                let vm = &mut self.mrb.borrow_mut();
                let serializable = mrb_funcall(vm, Some(obj), "to_json", &[]);
                let json_obj = Json::from_robject(
                    self.mrb.clone(),
                    serializable.expect("to_json not defined for instance"),
                );
                json_obj.serialize(serializer)
            }
        }
    }
}

pub(crate) fn mrb_json_dump(vm: &mut VM, obj: Rc<RObject>) -> Result<Rc<RObject>, Error> {
    let vm = Rc::new(RefCell::new(vm));
    let json_value = Json::from_robject(vm, obj);
    let serialized =
        serde_json::to_string(&json_value).expect("Failed to serialize JSON value to string");
    Ok(RObject::string(serialized).to_refcount_assigned())
}

pub struct JsonValue {
    inner: Rc<RObject>,
}

impl JsonValue {
    pub fn new(inner: Rc<RObject>) -> Self {
        Self { inner }
    }

    pub fn get_inner(&self) -> Rc<RObject> {
        self.inner.clone()
    }
}

impl From<JsonValue> for Rc<RObject> {
    fn from(value: JsonValue) -> Self {
        value.get_inner()
    }
}

impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
        let obj = match value {
            Value::Null => RObject::nil().to_refcount_assigned(),
            Value::Bool(b) => RObject::boolean(b).to_refcount_assigned(),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    RObject::integer(i).to_refcount_assigned()
                } else if let Some(u) = n.as_u64() {
                    RObject::integer(u as i64).to_refcount_assigned()
                } else if let Some(f) = n.as_f64() {
                    RObject::float(f).to_refcount_assigned()
                } else {
                    panic!("Invalid range of numeric value");
                }
            }
            Value::String(s) => RObject::string(s).to_refcount_assigned(),
            Value::Array(arr) => {
                let vec = arr.into_iter().map(JsonValue::from).collect::<Vec<_>>();
                let arr = RObject::array(vec.into_iter().map(|j| j.get_inner()).collect());
                arr.to_refcount_assigned()
            }
            Value::Object(obj) => {
                let map = obj
                    .into_iter()
                    .map(|(k, v)| {
                        let key = RObject::string(k).to_refcount_assigned();
                        (
                            key.as_hash_key().expect("object cannot use for hashed key"),
                            (key.clone(), JsonValue::from(v).get_inner()),
                        )
                    })
                    .collect();
                let hash = RObject::hash(map);
                hash.to_refcount_assigned()
            }
        };
        JsonValue::new(obj)
    }
}

pub(crate) fn mrb_json_load(_vm: &mut VM, json_str: impl Into<String>) -> Result<JsonValue, Error> {
    let value = serde_json::from_str::<serde_json::Value>(&json_str.into())
        .map_err(|e| Error::RuntimeError(format!("Failed to parse JSON string: {}", e)))?;
    Ok(value.into())
}
