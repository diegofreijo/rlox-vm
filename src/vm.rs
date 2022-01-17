use core::panic;
use std::vec;

use crate::chunk::{Chunk, Value};

pub enum InterpretResult {
    Ok,
    // CompileError,
    RuntimeError,
}

pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM { stack: vec![] }
    }

    pub fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut ret = InterpretResult::RuntimeError;
        for (_ip, op) in chunk.code().iter().enumerate() {
            #[cfg(feature = "trace")]
            {
                println!("          {:?}", self.stack);
                op.disassemble(&chunk, _ip);
            }

            match op {
                crate::chunk::Operation::Constant(coffset) => {
                    let c = chunk.read_constant(*coffset);
                    self.stack.push(*c);
                }
                crate::chunk::Operation::Nil => self.stack.push(Value::Nil),
                crate::chunk::Operation::True => self.stack.push(Value::Boolean(true)),
                crate::chunk::Operation::False => self.stack.push(Value::Boolean(false)),
                crate::chunk::Operation::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                },
                crate::chunk::Operation::Greater => {
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a > b));
                }
                crate::chunk::Operation::Less => {
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a < b));
                }
                crate::chunk::Operation::Add => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a + b));
                }
                crate::chunk::Operation::Substract => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a - b));
                }
                crate::chunk::Operation::Multiply => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a * b));
                }
                crate::chunk::Operation::Divide => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a / b));
                }
                crate::chunk::Operation::Not => {
                    let old = self.stack.pop().unwrap();
                    let new = VM::is_falsey(old);
                    self.stack.push(Value::Boolean(new));
                },
                crate::chunk::Operation::Negate => {
                    let v = VM::pop_number(&mut self.stack);
                    self.stack.push(Value::Number(-v));
                }
                crate::chunk::Operation::Return => {
                    println!("{:?}", &self.stack.pop().unwrap());
                    ret = InterpretResult::Ok;
                    break;
                }
            }
        }
        ret
    }

    fn binary<F>(stack: &mut Vec<Value>, implementation: F)
    where
        F: Fn(f64, f64) -> Value,
    {
        let b = VM::pop_number(stack);
        let a = VM::pop_number(stack);
        let result = implementation(a, b);
        stack.push(result);
    }

    fn pop_number(stack: &mut Vec<Value>) -> f64 {
        match stack.pop().unwrap() {
            Value::Number(num) => num,
            other => panic!("Expected a Number but popped the value {:?}", other),
        }
    }

    fn is_falsey(val: Value) -> bool {
        match val {
            Value::Boolean(b) => !b,
            Value::Nil => true,
            Value::Number(_) => false,
        }
    }
}
