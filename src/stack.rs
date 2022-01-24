use std::rc::Rc;

use crate::{value::{Value},object::ObjString, vm::InterpretResult};

pub struct Stack {
    values: Vec<Value>,
}

impl Stack {
    pub fn new() -> Self {
        Stack { values: vec![] }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value)
    }

    pub fn pop(&mut self) -> InterpretResult<Value> {
        self.values.pop().ok_or(
            "Tried to pop an empty stack.".to_string(),
        )
    }

    pub fn pop_number(&mut self) -> InterpretResult<f64> {
        match self.pop()? {
            Value::Number(n) => Ok(n),
            v => Err(
                format!("Expected to pop a number but found '{}'.", v).to_string(),
            ),
        }
    }

	pub fn pop_string(&mut self) -> InterpretResult<Rc<ObjString>> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            v => Err(
                format!("Expected to pop a string but found '{}'.", v).to_string(),
            ),
        }
    }

    pub fn get(&self, index: usize) -> InterpretResult<&Value> {
        self.values.get(index).ok_or(
            format!("No value found at index {}.", index).to_string(),
        )
    }

    pub fn set(&mut self, index: usize, value: Value) {
        self.values[index] = value;
    }

    pub fn peek(&self) -> InterpretResult<&Value> {
        self.values.last().ok_or(
            "Tried to peek an empty stack".to_string(),
        )
    }
}
