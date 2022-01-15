use chunk::Chunk;

mod chunk;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(chunk::Operation::Return);
    chunk.write(chunk::Operation::Return);
    chunk.write(chunk::Operation::Return);
    chunk.disassemble("test chunk");
}
