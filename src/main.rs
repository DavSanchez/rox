mod vm;

use std::process::ExitCode;

use vm::{chunk::Chunk, disassembler::Disassembler, opcode::OpCode};

fn main() -> ExitCode {
    match work() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn work() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();
    let constant_idx = chunk.write_constant(1.2)?;
    chunk.write_opcode(OpCode::Constant, 123);
    chunk.write_byte(constant_idx, 123);
    chunk.write_opcode(OpCode::Return, 123);

    let disassembler = Disassembler::new(&chunk, "test chunk");
    disassembler.write(&mut std::io::stdout())?;

    Ok(())
}
