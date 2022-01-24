use core::panic;
use std::{
    collections::HashMap,
    io::Write,
    rc::Rc,
};

use crate::{
    chunk::{Chunk, IdentifierName},
    value::{ObjString, Value}, stack::Stack,
};

pub type InterpretResult<V> = Result<V, String>;

pub struct VM {
    stack: Stack,
    globals: HashMap<IdentifierName, Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Stack::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run<W: Write>(&mut self, chunk: &Chunk, output: &mut W) -> InterpretResult<()> {
        let code = chunk.code();
        let mut ip = 0;
        loop {
            let op = code
                .get(ip)
                .expect(&format!("Operation not found. ip: {}", ip));
            ip += 1;

            #[cfg(feature = "trace")]
            {
                writeln!(output, "          {:?}", self.stack);
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
                        self.globals
                            .insert(name.clone(), self.stack.peek()?.clone());
                    } else {
                        panic!("Undefined variable '{}'", name);
                    }
                }
                crate::chunk::Operation::GetLocal(i) => {
                    // let stack_index = self.stack.len() - 1 - i;
                    let val = self
                        .stack
                        .get(*i)?
                        // .expect("Local variable not found in the stack")
                        .clone();
                    self.stack.push(val);
                }
                crate::chunk::Operation::SetLocal(i) => {
                    // let stack_index = self.stack.len() - 1 - i;
                    let val = self
                        .stack
                        .peek()?
                        // .expect("Expression not found in the stack to assign to local")
                        .clone();
                    self.stack.set(*i, val);
                }
                crate::chunk::Operation::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                }
                crate::chunk::Operation::Greater => 
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a > b))?,
                crate::chunk::Operation::Less => 
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a < b))?,
                crate::chunk::Operation::Add => match self.stack.peek()? {
                    Value::Number(_) => VM::binary(&mut self.stack, |a, b| Value::Number(a + b))?,
                    Value::String(_) => {
                        let b = self.stack.pop_string()?;
                        let a = self.stack.pop_string()?;
                        let result = format!("{}{}", a.value(), b.value());
                        let value = Value::String(Rc::from(ObjString::from_owned(result)));
                        self.stack.push(value);
                    }
                    v => Err(format!("Can't add the operand {:?}", v))?,
                },
                crate::chunk::Operation::Substract => 
                    VM::binary(&mut self.stack, |a, b| Value::Number(a - b))?,

                crate::chunk::Operation::Multiply => 
                    VM::binary(&mut self.stack, |a, b| Value::Number(a * b))?,
                
                crate::chunk::Operation::Divide => 
                    VM::binary(&mut self.stack, |a, b| Value::Number(a / b))?,
                crate::chunk::Operation::Not => {
                    let old = self.stack.pop().unwrap();
                    let new = VM::is_falsey(&old);
                    self.stack.push(Value::Boolean(new));
                }
                crate::chunk::Operation::Negate => {
                    let n = self.stack.pop_number()?;
                    let res = -n;
                    self.stack.push(Value::Number(res));
                }
                crate::chunk::Operation::Print => {
                    writeln!(
                        output,
                        "{}",
                        self.stack
                            .pop()
                            .expect("Tried to print a non-existing value")
                    ).unwrap();
                }
                crate::chunk::Operation::Return => {
                    // match self.stack.pop() {
                    //     Some(val) => ret = InterpretResult::Ok(val),
                    //     None => ret = InterpretResult::Ok(Value::Nil),
                    // }
                    return Ok(());
                }
                crate::chunk::Operation::JumpIfFalse(offset) => {
                    let exp = self.stack.peek().expect("Missing the if expression");
                    if VM::is_falsey(exp) {
                        ip += offset;
                    }
                }
                crate::chunk::Operation::Jump(offset) => ip += offset,
                crate::chunk::Operation::Loop(offset) => ip -= offset,
            }
        }
    }

    fn binary<F>(stack: &mut Stack, implementation: F) -> InterpretResult<()>
    where
        F: Fn(f64,f64) -> Value,
    {
        let b = stack.pop_number()?;
        let a = stack.pop_number()?;
        let result = implementation(a, b);
        stack.push(result);
        Ok(())
    }

    fn is_falsey(val: &Value) -> bool {
        match val {
            Value::Boolean(b) => !b,
            Value::Nil => true,
            _ => false,
        }
    }
}
