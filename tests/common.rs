use std::io::Write;

use rlox_vm::{compiler::Compiler, vm::VM};

#[derive(Debug)]
pub struct Output {
    pub contents: String,
}

impl Output {
    pub fn new() -> Self {
        Output {
            contents: String::new(),
        }
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.contents.push_str(&std::str::from_utf8(buf).unwrap());

        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}


pub fn assert_expression(exp_source: &str, expected: &str) {
	let source= format!("print {};", exp_source);
    let mut compiler = Compiler::from(&source);
	compiler.compile();

	assert!(!compiler.had_error);

	let mut vm = VM::new();
	let mut stdout = Output::new();
	
	vm.run(&compiler.chunk, &mut stdout).unwrap();

	assert_eq!(stdout.contents.trim_end_matches("\n"), expected);
}

pub fn assert_script_output(script_source: &str, expected: &str) {
	let source= format!("{}", script_source);
    let mut compiler = Compiler::from(&source);
	compiler.compile();

	assert!(!compiler.had_error);

	let mut vm = VM::new();
	let mut stdout = Output::new();
	
	vm.run(&compiler.chunk, &mut stdout).unwrap();

	assert_eq!(stdout.contents.trim_end_matches("\n"), expected);
}
