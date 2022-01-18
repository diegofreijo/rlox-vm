use core::panic;
use std::{collections::HashMap, rc::Rc, vec};

use crate::{
    chunk::{Chunk, IdentifierName},
    value::{ObjString, Value},
};

#[derive(Debug, PartialEq)]
pub enum InterpretResult {
    Ok(Value),
    // CompileError,
    RuntimeError,
}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<IdentifierName, Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: vec![],
            globals: HashMap::new(),
        }
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
                crate::chunk::Operation::Constant(iid) => {
                    let c = chunk.read_constant(*iid);
                    self.stack.push(c.clone());
                }
                crate::chunk::Operation::Nil => self.stack.push(Value::Nil),
                crate::chunk::Operation::True => self.stack.push(Value::Boolean(true)),
                crate::chunk::Operation::False => self.stack.push(Value::Boolean(false)),
                crate::chunk::Operation::Pop => {
                    self.stack.pop().expect("There was nothing to pop");
                }
                crate::chunk::Operation::GetGlobal(name) => {
                    let val = self
                        .globals
                        .get(name)
                        .expect(&format!("Undefined variable '{}'", name));
                    self.stack.push(val.clone());
                }
                crate::chunk::Operation::DefineGlobal(name) => {
                    self.globals.insert(name.clone(), self.stack.pop().unwrap());
                    // let name = VM::pop_string(&mut self.stack);
                    // self.globals
                    //     .insert(String::from(name.value()), self.stack.pop().unwrap());
                }
                crate::chunk::Operation::SetGlobal(name) => {
                    if self.globals.contains_key(name) {
                        self.globals.insert(name.clone(), self.peek_stack().unwrap().clone());
                    } else {
                        panic!("Undefined variable '{}'", name);
                    }
                }
                crate::chunk::Operation::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                }
                crate::chunk::Operation::Greater => {
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a > b));
                }
                crate::chunk::Operation::Less => {
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a < b));
                }
                crate::chunk::Operation::Add => match self.peek_stack().unwrap() {
                    Value::Number(_) => VM::binary(&mut self.stack, |a, b| Value::Number(a + b)),
                    Value::String(_) => {
                        let b = VM::pop_string(&mut self.stack);
                        let a = VM::pop_string(&mut self.stack);
                        let result = format!("{}{}", a.value(), b.value());
                        let value = Value::String(Rc::from(ObjString::from_owned(result)));
                        self.stack.push(value);
                    }
                    v => println!("Can't add the operand {:?}", v),
                },
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
                }
                crate::chunk::Operation::Negate => {
                    let v = VM::pop_number(&mut self.stack);
                    self.stack.push(Value::Number(-v));
                }
                crate::chunk::Operation::Print => {
                    println!(
                        "{}",
                        self.stack
                            .pop()
                            .expect("Tried to print a non-existing value")
                    );
                }
                crate::chunk::Operation::Return => {
                    match self.stack.pop() {
                        Some(val) => ret = InterpretResult::Ok(val),
                        None => ret = InterpretResult::Ok(Value::Nil),
                    }
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
            other => panic!("Expected a Number but found the value {:?}", other),
        }
    }

    fn pop_string(stack: &mut Vec<Value>) -> Rc<ObjString> {
        match stack.pop().unwrap() {
            Value::String(rc) => rc,
            other => panic!("Expected a String but found the value {:?}", other),
        }
    }

    fn is_falsey(val: Value) -> bool {
        match val {
            Value::Boolean(b) => !b,
            Value::Nil => true,
            _ => false,
        }
    }

    fn peek_stack(&self) -> Option<&Value> {
        self.stack.last()
    }
}
