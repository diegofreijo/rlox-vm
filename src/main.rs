extern crate rlox_vm;

use rlox_vm::{ interpreter::Interpreter};
use std::{io::{self, Write}, env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(args.get(1).unwrap()),
        _ => panic!("Too many arguments passed: {:?}", args),
    }
}

fn run_file(path: &str) {
    println!("Running script at {}", path);
    let contents = fs::read_to_string(path).expect("Something went wrong reading the file");
    run_source(&contents);
}

fn run_source(raw_source: &str) {
    let mut interpreter = Interpreter::new(io::stdout());
    interpreter.interpret(raw_source);
}

fn repl() {
    let stdin = io::stdin();
    let mut interpreter = Interpreter::new(io::stdout());

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

        interpreter.interpret(&source);
    }
}
