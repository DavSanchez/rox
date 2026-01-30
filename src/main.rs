mod chunk;

use chunk::Chunk;
use chunk::opcode::OpCode;

fn main() {
    let mut chunk = Chunk::new();
    chunk.code_array.push(OpCode::Return);

    chunk.disassemble("test chunk");
}
