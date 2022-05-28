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
    #[error("Error parsing code on line {line}")]
    Parse { line: usize },
    #[error("Type error{}: expected {expected_type}, got {actual_type}", show_line(.line))]
    Type {
        expected_type: String,
        actual_type: String,
        line: Option<usize>,
    },
    #[error("Syntax error on line {line}")]
    SyntaxError { line: usize },
    #[error("An unknown error has occurred")]
    Unknown,
}

impl InterpreterError {
    pub(crate) fn add_line_to_type_error(self, new_line: usize) -> Self {
        match self {
            Self::Type {
                line: _,
                expected_type,
                actual_type,
            } => Self::Type {
                line: Some(new_line),
                expected_type,
                actual_type,
            },
            _ => panic!("don't do this"),
        }
    }
    pub(crate) fn type_error(expected_type: String, actual_type: String) -> Self {
        Self::Type {
            expected_type,
            actual_type,
            line: None,
        }
    }
}

fn show_line(line: &Option<usize>) -> String {
    line.map_or(String::default(), |l| format!(" on line {}", l))
}

pub type InterpreterResult<T> = Result<T, InterpreterError>;
