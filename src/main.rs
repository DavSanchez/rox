mod vm;

use std::process::ExitCode;

use vm::{chunk::Chunk, disassembler::Disassembler, opcode::OpCode};

use crate::vm::Vm;

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
    let mut vm = Vm::default();
    let mut chunk = Chunk::default();
    let constant_idx = chunk.write_constant(1.2.into())?;
    chunk.write_opcode(OpCode::Constant, 123);
    chunk.write_byte(constant_idx, 123);

    let constant_idx2 = chunk.write_constant(3.4.into())?;
    chunk.write_opcode(OpCode::Constant, 123);
    chunk.write_byte(constant_idx2, 123);

    chunk.write_opcode(OpCode::Add, 123);

    let constant_idx3 = chunk.write_constant(5.6.into())?;
    chunk.write_opcode(OpCode::Constant, 123);
    chunk.write_byte(constant_idx3, 123);

    chunk.write_opcode(OpCode::Divide, 123);

    chunk.write_opcode(OpCode::Negate, 123);
    chunk.write_opcode(OpCode::Return, 123);

    let disassembler = Disassembler::new(&chunk, "test chunk");
    disassembler.write(&mut std::io::stdout())?;

    vm.interpret(&chunk)?;

    Ok(())
}
