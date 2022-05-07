pub mod errors;
mod scanner;

pub use crate::errors::{InterpreterError, InterpreterResult};
use crate::scanner::scan_tokens;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs::File;
use std::io::Read;

pub fn main() -> InterpreterResult<()> {
    let mut args = env::args();

    if args.len() > 2 {
        Err(InterpreterError::Usage)
    } else if let Some(fname) = args.nth(1) {
        run_file(fname)
    } else {
        prompt()
    }
}

fn run_file(fname: String) -> InterpreterResult<()> {
    let mut f = File::open(fname)?;
    let mut s = String::default();
    f.read_to_string(&mut s)?;
    run(s)
}

fn prompt() -> InterpreterResult<()> {
    let mut rl = Editor::<()>::new();
    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(l) => match run(l) {
                Ok(_) => continue,
                Err(err @ InterpreterError::Interpreter { .. }) => {
                    println!("{:?}", err);
                }
                Err(e) => return Err(e),
            },
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye");
                return Ok(());
            }
            Err(err) => return Err(InterpreterError::from(err)),
        }
    }
}

fn run(s: String) -> InterpreterResult<()> {
    let tokens = scan_tokens(s)?;
    for t in tokens.iter() {
        println!("{:?}", t);
    }
    Ok(())
}
