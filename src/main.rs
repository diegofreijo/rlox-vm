use chunk::Chunk;
use vm::VM;

mod vm;
mod chunk;

fn main() {
    let mut chunk = Chunk::new();
    
    let constant = chunk.add_constant(1.2);
    chunk.write(chunk::Operation::Constant(constant), 123);
    
    let constant = chunk.add_constant(4.5);
    chunk.write(chunk::Operation::Constant(constant), 123);
    
    chunk.write(chunk::Operation::Return, 124);
    chunk.disassemble("test chunk");

    println!("----------------");
    println!("Execution starting");

    let vm = VM::new(chunk);
    vm.run();
}
