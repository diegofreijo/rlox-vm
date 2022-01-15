use chunk::Chunk;
use vm::VM;

mod vm;
mod chunk;

fn main() {
    let mut chunk = Chunk::new();
    
    let constant = chunk.add_constant(1.2);
    chunk.write(chunk::Operation::Constant(constant), 123);

    let constant = chunk.add_constant(3.4);
    chunk.write(chunk::Operation::Constant(constant), 123);
    
    chunk.write(chunk::Operation::Add, 123);
    
    let constant = chunk.add_constant(5.6);
    chunk.write(chunk::Operation::Constant(constant), 123);

    chunk.write(chunk::Operation::Divide, 123);

    chunk.write(chunk::Operation::Negate, 123);

    chunk.write(chunk::Operation::Return, 124);
    chunk.disassemble("test chunk");

    println!("----------------");
    println!("Execution starting");

    let mut vm = VM::new(chunk);
    vm.run();
}
