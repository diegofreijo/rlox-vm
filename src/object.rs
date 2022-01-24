use crate::chunk::Chunk;

#[derive(Debug)]
pub struct ObjFunction {
    arity: u8,
    pub chunk: Chunk,
    name: String,
}

impl ObjFunction {
    pub fn new(name: &str) -> Self {
        ObjFunction {
            arity: 0,
            chunk: Chunk::new(),
            name: String::from(name),
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
