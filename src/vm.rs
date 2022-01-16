use std::vec;

use crate::chunk::{Chunk, Value};

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: vec![],
        }
    }

    // pub fn interpret(&mut self, source: &String) {

    // }

    pub fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        let mut ret = InterpretResult::RuntimeError;
        for (_ip, op) in chunk.code().iter().enumerate() {

            #[cfg(feature="trace")]
			{
				println!("          {:?}", self.stack);
				op.disassemble(&chunk, ip);
			}
            
			match op {
                crate::chunk::Operation::Constant(coffset) => {
                    let c = chunk.read_constant(*coffset);
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
