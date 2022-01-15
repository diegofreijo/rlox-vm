use std::vec;

use crate::chunk::{Chunk, Value};

pub enum InterpretResult {
    Ok,
    CompileError,
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

    pub fn run(&self) -> InterpretResult {
        let mut ret = InterpretResult::RuntimeError;
        for (ip, op) in self.chunk.code().iter().enumerate() {

            #[cfg(feature="trace")]
			op.disassemble(&self.chunk, ip);
            
			match op {
                crate::chunk::Operation::Constant(coffset) => {
                    println!("{}", self.chunk.read_constant(*coffset));
                }
                crate::chunk::Operation::Return => {
                    ret = InterpretResult::Ok;
                    break;
                }
            }
        }
        ret
    }
}
