extern crate rlox_vm;

use std::io::{self, Write};
use rlox_vm::vm::{VM};
use rlox_vm::compiler::Compiler;


fn main() {
    repl()
}

fn repl() {
    let mut vm = VM::new();
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
            let _result = vm.run(&compiler.chunk);
            // println!("{:?}", result);
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
