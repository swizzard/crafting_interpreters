use rustyline::error::ReadlineError;
use std::fmt;
use std::io;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("Format error: {source}")]
    Fmt {
        #[from]
        source: fmt::Error,
    },
    #[error("Readline error: {source}")]
    RL {
        #[from]
        source: ReadlineError,
    },
    #[error("[{line}] Error: {message}")]
    Interpreter { line: usize, message: String },
    #[error("Usage: rlox [script]")]
    Usage,
    #[error("Error parsing code")]
    ParseError,
}

pub type InterpreterResult<T> = Result<T, InterpreterError>;
