use std::io::{self, Write};
use vm::VM;

use crate::compiler::Compiler;

mod chunk;
mod compiler;
mod scanner;
mod token;
mod vm;

fn main() {
    repl()
}

fn repl() {
    let stdin = io::stdin();

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
                        source.push_str(&input);
                    }
                }
                Err(error) => {
                    println!("error: {}", error);
                    break;
                }
            }
        }

        let compiler = Compiler::from(&source);
        if !compiler.had_error {
            let mut vm = VM::new();
            match vm.run(&compiler.chunk) {
                vm::InterpretResult::Ok => println!("Ok"),
                // vm::InterpretResult::CompileError => println!("Compile error"),
                vm::InterpretResult::RuntimeError => println!("Runtime error"),
            }
        } else {
            println!("Compiler error!");
        }
    }

    // let constant = chunk.add_constant(1.2);
    // chunk.write(chunk::Operation::Constant(constant), 123);

    // let constant = chunk.add_constant(3.4);
    // chunk.write(chunk::Operation::Constant(constant), 123);

    // chunk.write(chunk::Operation::Add, 123);

    // let constant = chunk.add_constant(5.6);
    // chunk.write(chunk::Operation::Constant(constant), 123);

    // chunk.write(chunk::Operation::Divide, 123);

    // chunk.write(chunk::Operation::Negate, 123);

    // chunk.write(chunk::Operation::Return, 124);
    // chunk.disassemble("test chunk");
}
