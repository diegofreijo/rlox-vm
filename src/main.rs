extern crate rlox_vm;

use rlox_vm::compiler::Compiler;
use rlox_vm::vm::VM;
use std::io::{self, Write};

fn main() {
    repl()
}

fn repl() {
    let mut vm = VM::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("> ");
        io::stdout().flush().expect("Failed flushing to stdout");

        let mut source = String::new();

        loop {
            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Ok(_) => {
                    input = input.to_string();
                    if input.trim().is_empty() {
                        break;
                    } else {
                        source.push_str(&input.trim_end());
                    }
                }
                Err(error) => {
                    println!("error: {}", error);
                    break;
                }
            }
        }

        let mut compiler = Compiler::from(&source);
        compiler.compile();
        if !compiler.had_error {
            let result = vm.run(&compiler.chunk(), &mut stdout);
            if let Err(msg) = result {
                println!("[Runime Error] {}", msg);
            }
        } else {
            println!("Compiler error!");
        }
    }
}
