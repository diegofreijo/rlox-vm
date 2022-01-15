use std::vec;

use crate::chunk::{Chunk, Value};

pub enum InterpretResult {
    Ok,
    // CompileError,
    RuntimeError,
}

pub struct VM {
    chunk: Chunk,
    // ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        VM {
            chunk,
            stack: vec![],
        }
    }

    pub fn run(&mut self) -> InterpretResult {
        let mut ret = InterpretResult::RuntimeError;
        for (ip, op) in self.chunk.code().iter().enumerate() {

            #[cfg(feature="trace")]
			{
				println!("          {:?}", self.stack);
				op.disassemble(&self.chunk, ip);
			}
            
			match op {
                crate::chunk::Operation::Constant(coffset) => {
                    let c = self.chunk.read_constant(*coffset);
					self.stack.push(c);
                }
				crate::chunk::Operation::Add => {
					VM::binary(&mut self.stack, |a,b| a+b);
				},
				crate::chunk::Operation::Substract => {
					VM::binary(&mut self.stack, |a,b| a-b);
				},
				crate::chunk::Operation::Multiply => {
					VM::binary(&mut self.stack, |a,b| a*b);
				},
				crate::chunk::Operation::Divide => {
					VM::binary(&mut self.stack, |a,b| a/b);
				},
    			crate::chunk::Operation::Negate => {
					let v = self.stack.pop().unwrap();
					self.stack.push(-v);
				},
                crate::chunk::Operation::Return => {
					println!("{}", &self.stack.pop().unwrap());
                    ret = InterpretResult::Ok;
                    break;
                }
            }
        }
        ret
    }

    fn binary<F>(stack: &mut Vec<f64>, implementation: F) where F: Fn(Value, Value) -> Value {
        let b = stack.pop().unwrap();
		let a = stack.pop().unwrap();
		stack.push(implementation(a,b));
    }
}
