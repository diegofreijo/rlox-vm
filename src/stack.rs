use std::rc::Rc;

use crate::{value::{Value},object::ObjString, vm::{InterpretResult, RuntimeError}};

#[derive(Debug)]
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
            RuntimeError::new("Tried to pop an empty stack.")
        )
    }

    pub fn pop_number(&mut self) -> InterpretResult<f64> {
        match self.pop()? {
            Value::Number(n) => Ok(n),
            v => Err(
                RuntimeError::new(&format!("Expected to pop a number but found '{}'.\n{:#?}", v, self))
            ),
        }
    }

	pub fn pop_string(&mut self) -> InterpretResult<Rc<ObjString>> {
        match self.pop()? {
            Value::String(s) => Ok(s),
            v => Err(
                RuntimeError::new(&format!("Expected to pop a string but found '{}'.", v))
            ),
        }
    }

    pub fn get(&self, index: usize) -> InterpretResult<&Value> {
        self.values.get(index).ok_or(
            RuntimeError::new(&format!("No value found at index {}.", index))
        )
    }

    pub fn set(&mut self, index: usize, value: Value) {
        self.values[index] = value;
    }

    pub fn peek(&self) -> InterpretResult<&Value> {
        self.values.last().ok_or(
            RuntimeError::new("Tried to peek an empty stack"),
        )
    }

    pub fn peek_many(&self, count: usize) -> InterpretResult<&Value> {
        self.get( self.values.len() - 1 - count)
    }

    pub fn contents(&self) -> &Vec<Value> {
        &self.values
    }
}
