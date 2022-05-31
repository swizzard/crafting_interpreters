mod environment;
pub mod errors;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;

pub use crate::errors::{InterpreterError, InterpreterResult};
use crate::scanner::scan_tokens;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs::File;
use std::io::Read;

pub fn main() -> InterpreterResult<()> {
    let mut args = env::args();
    let interpreter = interpreter::Interpreter::default();

    if args.len() > 2 {
        Err(InterpreterError::Usage)
    } else if let Some(fname) = args.nth(1) {
        run_file(interpreter, fname)
    } else {
        prompt(interpreter)
    }
}

fn run_file(mut interpreter: interpreter::Interpreter, fname: String) -> InterpreterResult<()> {
    let mut f = File::open(fname)?;
    let mut s = String::default();
    f.read_to_string(&mut s)?;
    run(&mut interpreter, s)
}

fn prompt(mut interpreter: interpreter::Interpreter) -> InterpreterResult<()> {
    let mut rl = Editor::<()>::new();
    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(l) => match run(&mut interpreter, l) {
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

fn run(interpreter: &mut interpreter::Interpreter, s: String) -> InterpreterResult<()> {
    let tokens = scan_tokens(s)?;
    let (expr, errs) = parser::parse(tokens);
    if let Some(ref res) = expr {
        println!("{}", interpreter.interpret(res)?);
        Ok(())
    } else {
        let mut e = InterpreterError::Unknown;
        for err in errs.into_iter() {
            println!("{}", &err);
            e = err;
        }
        Err(e)
    }
}
