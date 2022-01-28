use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct ObjFunction {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: String,
}

impl ObjFunction {
    pub fn new(name: &str) -> Self {
        Self {
            arity: 0,
            chunk: Chunk::new(),
            name: String::from(name),
        }
    }
}


#[derive(Debug, Clone)]
pub struct ObjNative {
    pub name: String,
    pub function: fn() -> f64,
}

impl ObjNative {
    pub fn new(name: &str, function: fn() -> f64) -> Self {
        Self {
            name: String::from(name),
            function,
        }
    }
}

#[derive(Debug)]
pub struct ObjString {
    pub value: String,
}

impl ObjString {
    pub fn from(value: &str) -> Self {
        ObjString {
            value: String::from(value),
        }
    }

    pub fn from_owned(value: String) -> Self {
        ObjString { value: value }
    }

    pub fn value(&self) -> &String {
        &self.value
    }
}
