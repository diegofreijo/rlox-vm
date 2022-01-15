use chunk::Chunk;

mod value;
mod chunk;

fn main() {
    let mut chunk = Chunk::new();
    
    let constant = chunk.add_constant(1.2);
    chunk.write(chunk::Operation::Constant(constant));
    
    let constant = chunk.add_constant(4.5);
    chunk.write(chunk::Operation::Constant(constant));
    
    chunk.write(chunk::Operation::Return);
    chunk.disassemble("test chunk");
}
