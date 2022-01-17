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
				op.disassemble(&chunk, _ip);
			}
            
			match op {
                crate::chunk::Operation::Constant(coffset) => {
                    let c = chunk.read_constant(*coffset);
					self.stack.push(*c);
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
                    let v = VM::pop_number(&mut self.stack);
                    self.stack.push(Value::Number(-v));
				},
                crate::chunk::Operation::Return => {
					println!("{:?}", &self.stack.pop().unwrap());
                    ret = InterpretResult::Ok;
                    break;
                }
            }
        }
        ret
    }

    fn binary<F>(stack: &mut Vec<Value>, implementation: F) where F: Fn(f64, f64) -> f64 {
        let b = VM::pop_number(stack);
		let a = VM::pop_number(stack);
        let result = implementation(a,b);
		stack.push(Value::Number(result));
    }

    fn pop_number(stack: &mut Vec<Value>) -> f64 {
        if let Value::Number(num) = stack.pop().unwrap(){
            num
        } else {
            panic!("Expected a Number but popped some other value");
        }
    }
}
