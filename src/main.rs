extern crate rlox_vm;

use rlox_vm::compiler::Compiler;
use rlox_vm::vm::VM;
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
    let mut vm = VM::new();
    let mut stdout = io::stdout();
    let source = String::from(raw_source);
    
    let mut compiler = Compiler::from_source(&source);
    let frame = compiler.compile();
    if !compiler.had_error {
        let result = vm.run(&frame, &mut stdout);
        if let Err(msg) = result {
            println!("[Runime Error] {}", msg);
        }
    } else {
        println!("Compiler error!");
    }
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

        let mut compiler = Compiler::from_source(&source);
        let frame = compiler.compile();
        if !compiler.had_error {
            let result = vm.run(&frame, &mut stdout);
            if let Err(msg) = result {
                println!("{}", msg);
                // println!("[Runime Error] {}", msg);
            }
        } else {
            println!("Compiler error!");
        }
    }
}
