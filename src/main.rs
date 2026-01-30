mod chunk;

use chunk::{ByteCodeChunk, OpCode};

fn main() {
    let mut chunk = ByteCodeChunk::new();
    chunk.push(OpCode::Return);

    chunk.disassemble("test chunk");
}
