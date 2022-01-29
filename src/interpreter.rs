use std::io::Write;

use crate::{compiler::Compiler, vm::VM};

pub struct Interpreter<W: Write> {
    vm: VM,
    output: W,
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        Self {
            vm: VM::new(),
            output,
        }
    }

    pub fn interpret(&mut self, raw_source: &str) {
        let source = String::from(raw_source);
        let mut compiler = Compiler::from_source(&source);
        let function = compiler.compile();

        if !compiler.had_error {
            match self.vm.run_main(&function, &mut self.output) {
                Ok(_ret) => (), // I do nothing for now
                Err(msg) => {
                    writeln!(self.output, "[Runime Error] {}", msg).unwrap();
                }
            };
        } else {
            writeln!(self.output, "Compiler error!").unwrap();
        }
    }
}
