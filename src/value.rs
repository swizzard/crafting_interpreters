use crate::errors::InterpreterError;
use float_eq::float_eq;

#[derive(Clone, Debug)]
pub enum Value {
    r#String(String),
    Number(f32),
    Bool(bool),
    Nil,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::r#String(s) => match other {
                Self::r#String(o) => s == o,
                _ => false,
            },
            Self::Number(n) => match other {
                Self::Number(o) => float_eq!(n, o, abs <= 0.000_1),
                _ => false,
            },
            Self::Bool(b) => match other {
                Self::Bool(o) => b == o,
                _ => false,
            },
            Self::Nil => matches!(other, Self::Nil),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::r#String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => f.write_str("nil"),
        }
    }
}

impl TryFrom<f32> for Value {
    type Error = InterpreterError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Ok(Value::Number(value))
    }
}

impl TryFrom<String> for Value {
    type Error = InterpreterError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Value::r#String(value))
    }
}

impl TryFrom<bool> for Value {
    type Error = InterpreterError;

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        Ok(Value::Bool(value))
    }
}

impl TryFrom<&Value> for f32 {
    type Error = InterpreterError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(*n),
            Value::Bool(_) => Err(InterpreterError::type_error(
                String::from("number"),
                String::from("boolean"),
            )),
            Value::r#String(_) => Err(InterpreterError::type_error(
                String::from("number"),
                String::from("string"),
            )),
            Value::Nil => Err(InterpreterError::type_error(
                String::from("number"),
                String::from("nil"),
            )),
        }
    }
}

impl TryFrom<&Value> for String {
    type Error = InterpreterError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(_) => Err(InterpreterError::type_error(
                String::from("string"),
                String::from("string"),
            )),
            Value::Bool(_) => Err(InterpreterError::type_error(
                String::from("string"),
                String::from("boolean"),
            )),
            Value::r#String(s) => Ok(s.clone()),
            Value::Nil => Err(InterpreterError::type_error(
                String::from("string"),
                String::from("nil"),
            )),
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = InterpreterError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(_) => Err(InterpreterError::type_error(
                String::from("boolean"),
                String::from("number"),
            )),
            Value::Bool(b) => Ok(*b),
            Value::r#String(_) => Err(InterpreterError::type_error(
                String::from("boolean"),
                String::from("string"),
            )),
            Value::Nil => Ok(false),
        }
    }
}
