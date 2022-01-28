use std::{rc::Rc, fmt::{Display}};
use crate::object::{ObjString, ObjFunction, ObjNative};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<ObjString>),
    Function(Rc<ObjFunction>),
    Native(ObjNative),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0.value == r0.value,
            (Self::Function(f1), Self::Function(f2)) => 
                f1.chunk == f2.chunk && f1.arity == f2.arity && f1.name == f2.name,
            _ => false,
        }
    }
}

impl Value {
	pub fn new_string(value: &str) -> Self {
		Value::String(
			Rc::from(ObjString::from(value))
		)
	}

    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(b) => !b,
            Value::Nil => true,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => f.write_str("nil"),
            Value::Boolean(b) => f.write_str(&b.to_string()),
            Value::Number(n) => f.write_str(&n.to_string()),
            Value::String(obj) => f.write_str(&obj.value),
            Value::Function(of) => f.write_str(&format!("<fn '{}'>", of.name)),
            Value::Native(native) => f.write_str(&format!("<native '{}'>", native.name)),
        }
    }
}
