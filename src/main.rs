use std::io;

use chunk::Chunk;
use vm::VM;

mod chunk;
mod vm;
mod compiler;
mod scanner;
mod token;

fn main() {
    repl()
}

fn repl() {
    let mut vm = VM::new();

    loop {
        print!("> ");

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let chunk = Chunk::new();
                vm.run(chunk);
            }
            Err(error) => println!("error: {}", error),
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
