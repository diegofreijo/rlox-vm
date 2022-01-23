use std::rc::Rc;

use crate::{value::{Value, ObjString}, vm::InterpretResult};

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

    pub fn pop(&mut self) -> Result<Value, InterpretResult> {
        self.values.pop().ok_or(InterpretResult::RuntimeError(
            "Tried to pop an empty stack.".to_string(),
        ))
    }

    pub fn pop_number(&mut self) -> Result<f64, InterpretResult> {
        match self.pop()? {
            Value::Number(n) => Ok(n),
            v => Err(InterpretResult::RuntimeError(
                format!("Expected to pop a number but found '{}'.", v).to_string(),
            )),
        }
    }

	pub fn pop_string(&mut self) -> Result<Rc<ObjString>, InterpretResult> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            v => Err(InterpretResult::RuntimeError(
                format!("Expected to pop a string but found '{}'.", v).to_string(),
            )),
        }
    }

    pub fn get(&self, index: usize) -> Result<&Value, InterpretResult> {
        self.values.get(index).ok_or(InterpretResult::RuntimeError(
            format!("No value found at index {}.", index).to_string(),
        ))
    }

    pub fn set(&mut self, index: usize, value: Value) {
        self.values[index] = value;
    }

    pub fn peek(&self) -> Result<&Value, InterpretResult> {
        self.values.last().ok_or(InterpretResult::RuntimeError(
            "Tried to peek an empty stack".to_string(),
        ))
    }
}
