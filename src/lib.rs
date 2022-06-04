mod environment;
pub mod errors;
mod expr;
mod expr_printer;
mod interpreter;
mod parser;
mod prompt;
mod scanner;
mod stmt;
mod token;
mod value;

pub use crate::errors::{InterpreterError, InterpreterResult};
use crate::interpreter::Interpreter;
use crate::scanner::scan_tokens;
use rustyline::error::ReadlineError;
use std::env;
use std::fs::File;
use std::io::Read;

pub fn main() -> InterpreterResult<()> {
    let mut args = env::args();
    let mut runner = Runner::default();
    if args.len() > 2 {
        Err(InterpreterError::Usage)
    } else if let Some(fname) = args.nth(1) {
        runner.run_file(fname)
    } else {
        runner.prompt()
    }
}

#[derive(Default)]
pub struct Runner {
    interpreter: Interpreter,
}

impl Runner {
    fn run(&self, s: String) -> InterpreterResult<()> {
        let tokens = scan_tokens(s)?;
        let (expr, errs) = parser::parse(tokens);
        if let Some(ref res) = expr {
            println!("{}", self.interpreter.interpret(res)?);
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
    fn run_file(&mut self, fname: String) -> InterpreterResult<()> {
        let mut f = File::open(fname)?;
        let mut s = String::default();
        f.read_to_string(&mut s)?;
        self.run(s)
    }
    fn prompt(&mut self) -> InterpreterResult<()> {
        let prompt = prompt::Prompt::new(">> ");
        for line in prompt {
            match line {
                Ok(l) => match self.run(l) {
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
        Ok(())
    }
}
