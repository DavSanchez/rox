mod array;
mod compiler;
mod vm;

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use vm::Vm;
use vm::error::RoxError;

fn main() -> ExitCode {
    let mut vm = Vm::default();

    let args: Vec<String> = std::env::args().skip(1).collect();
    vm.set_trace(args.iter().any(|arg| arg == "--trace"));

    let path = args.into_iter().find(|arg| !arg.starts_with("--"));
    let result = match path.as_deref() {
        None => repl(&mut vm),
        Some(path) => run_file(&mut vm, path),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(RoxError::Io(e)) => {
            eprintln!("{e}");
            ExitCode::from(74)
        }
        Err(RoxError::Compile(e)) => {
            eprintln!("{e}");
            ExitCode::from(65)
        }
        Err(RoxError::Runtime(e)) => {
            eprintln!("{e}");
            ExitCode::from(70)
        }
    }
}

fn repl(vm: &mut Vm) -> Result<(), RoxError> {
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            println!();
            break;
        }

        if let Err(err) = vm.interpret(&line) {
            eprintln!("{err}");
        }
    }
    Ok(())
}

fn run_file(vm: &mut Vm, path: &str) -> Result<(), RoxError> {
    let source = std::fs::read_to_string(path)?;
    vm.interpret(&source)?;
    Ok(())
}
