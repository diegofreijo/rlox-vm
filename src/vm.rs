use std::{collections::HashMap, io::Write, rc::Rc};

use crate::{
    chunk::{Chunk, IdentifierName},
    stack::Stack,
    value::{ Value},object::{ObjString, ObjFunction}
};

pub type InterpretResult<V> = Result<V, String>;

struct CallFrame {
    function: ObjFunction,
    ip: usize,
    first_slot: usize,
}

impl CallFrame {
    pub fn new(function: ObjFunction) -> Self {
        CallFrame {
            function,
            ip: 0,
            first_slot: 0,
        }
    }
}

pub struct VM {
    stack: Stack,
    globals: HashMap<IdentifierName, Value>,
    frames: Vec<CallFrame>,
}

impl  VM {
    pub fn new() -> Self {
        VM {
            stack: Stack::new(),
            globals: HashMap::new(),
            frames: vec![],
        }
    }

    pub fn run<W: Write>(&mut self, function: ObjFunction, output: &mut W) -> InterpretResult<()> {
        self.frames = vec![CallFrame::new(function)];
        let mut frame = self.frames.first_mut().ok_or("")?;
        let code = frame.function.chunk.code();
        let chunk = &frame.function.chunk;
        
        loop {
            let op = code
                .get(frame.ip)
                .ok_or(&format!("Operation not found. ip: {}", frame.ip))?;
            frame.ip += 1;

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
                    self.stack.pop()?;//.expect("There was nothing to pop");
                }
                crate::chunk::Operation::GetGlobal(name) => {
                    let val = self
                        .globals
                        .get(name)
                        .ok_or(&format!("Undefined variable '{}'", name))?;
                    self.stack.push(val.clone());
                }
                crate::chunk::Operation::DefineGlobal(name) => {
                    self.globals.insert(name.clone(), self.stack.pop()?);
                }
                crate::chunk::Operation::SetGlobal(name) => {
                    if !self.globals.contains_key(name) {
                        return Err(format!("Undefined variable '{}'", name));
                    }

                    self.globals
                        .insert(name.clone(), self.stack.peek()?.clone());
                }
                crate::chunk::Operation::GetLocal(i) => {
                    let absolute_index = i  + frame.first_slot;
                    let val = self
                        .stack
                        .get(absolute_index)?
                        // .expect("Local variable not found in the stack")
                        .clone();
                    self.stack.push(val);
                }
                crate::chunk::Operation::SetLocal(i) => {
                    let absolute_index =  frame.first_slot + i;
                    let val = self
                        .stack
                        .peek()?
                        // .expect("Expression not found in the stack to assign to local")
                        .clone();
                    self.stack.set(absolute_index, val);
                }
                crate::chunk::Operation::Equal => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(Value::Boolean(a == b));
                }
                crate::chunk::Operation::Greater => {
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a > b))?
                }
                crate::chunk::Operation::Less => {
                    VM::binary(&mut self.stack, |a, b| Value::Boolean(a < b))?
                }
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
                crate::chunk::Operation::Substract => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a - b))?
                }

                crate::chunk::Operation::Multiply => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a * b))?
                }

                crate::chunk::Operation::Divide => {
                    VM::binary(&mut self.stack, |a, b| Value::Number(a / b))?
                }
                crate::chunk::Operation::Not => {
                    let old = self.stack.pop()?;
                    let new = old.is_falsey();
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
                        self.stack.pop()? // .expect("Tried to print a non-existing value")
                    ).map_err(|x| format!("Unexpected error while printing to output: {}", x))?;
                }
                crate::chunk::Operation::Return => {
                    // match self.stack.pop() {
                    //     Some(val) => ret = InterpretResult::Ok(val),
                    //     None => ret = InterpretResult::Ok(Value::Nil),
                    // }
                    return Ok(());
                }
                crate::chunk::Operation::JumpIfFalse(offset) => {
                    let exp = self.stack.peek()?;//.expect("Missing the if expression");
                    if exp.is_falsey() {
                        frame.ip += offset;
                    }
                }
                crate::chunk::Operation::Jump(offset) => frame.ip += offset,
                crate::chunk::Operation::Loop(offset) => frame.ip -= offset,
            }
        }
    }

    fn binary<F>(stack: &mut Stack, implementation: F) -> InterpretResult<()>
    where
        F: Fn(f64, f64) -> Value,
    {
        let b = stack.pop_number()?;
        let a = stack.pop_number()?;
        let result = implementation(a, b);
        stack.push(result);
        Ok(())
    }
}
