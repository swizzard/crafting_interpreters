use crate::errors::{InterpreterError, InterpreterResult};
use crate::expr_printer::ExprPrinter;
use crate::token::Token;
pub use crate::value::Value;
use std::cmp::PartialEq;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Value,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

impl Expr {
    pub fn literal_num(n: f32) -> Self {
        Self::Literal {
            value: Value::Number(n),
        }
    }
    pub fn literal_string<T>(s: T) -> Self
    where
        T: Into<String>,
    {
        Self::Literal {
            value: Value::r#String(s.into()),
        }
    }
    pub fn literal_bool(b: bool) -> Self {
        Self::Literal {
            value: Value::Bool(b),
        }
    }
    pub fn literal_nil() -> Self {
        Self::Literal { value: Value::Nil }
    }
    pub fn print(&self) -> InterpreterResult<String> {
        ExprPrinter::default().build(self)?.print()
    }
}

impl TryFrom<String> for Expr {
    type Error = InterpreterError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Expr::literal_string(value))
    }
}

impl TryFrom<f32> for Expr {
    type Error = InterpreterError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Ok(Expr::literal_num(value))
    }
}

impl TryFrom<&Expr> for String {
    type Error = InterpreterError;
    fn try_from(value: &Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Literal {
                value: Value::r#String(s),
            } => Ok(s.clone()),
            Expr::Literal {
                value: Value::Number(_),
            } => type_error("string", "number"),
            Expr::Literal { value: Value::Nil } => type_error("string", "nil"),
            Expr::Literal {
                value: Value::Bool(_),
            } => type_error("string", "boolean"),
            Expr::Assign { .. } => type_error("string", "assignment expression"),
            Expr::Binary { .. } => type_error("string", "binary expression"),
            Expr::Grouping { .. } => type_error("string", "grouping expression"),
            Expr::Unary { .. } => type_error("string", "unary expression"),
            Expr::Variable { .. } => type_error("string", "variable"),
        }
    }
}

impl TryFrom<&Expr> for f32 {
    type Error = InterpreterError;
    fn try_from(value: &Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Literal {
                value: Value::Number(n),
            } => Ok(*n),
            Expr::Literal {
                value: Value::r#String(_),
            } => type_error("number", "string"),
            Expr::Literal { value: Value::Nil } => type_error("number", "nil"),
            Expr::Literal {
                value: Value::Bool(_),
            } => type_error("number", "boolean"),
            Expr::Assign { .. } => type_error("number", "assignment expression"),
            Expr::Binary { .. } => type_error("number", "binary expression"),
            Expr::Grouping { .. } => type_error("number", "grouping expression"),
            Expr::Unary { .. } => type_error("nubmer", "unary expression"),
            Expr::Variable { .. } => type_error("number", "variable"),
        }
    }
}
impl TryFrom<&Expr> for bool {
    type Error = InterpreterError;
    fn try_from(value: &Expr) -> Result<Self, Self::Error> {
        match value {
            Expr::Literal {
                value: Value::Bool(b),
            } => Ok(*b),
            Expr::Literal {
                value: Value::r#String(_),
            } => type_error("boolean", "string"),
            Expr::Literal { value: Value::Nil } => type_error("boolean", "nil"),
            Expr::Literal {
                value: Value::Number(_),
            } => type_error("boolean", "number"),
            Expr::Assign { .. } => type_error("boolean", "assignment expression"),
            Expr::Binary { .. } => type_error("boolean", "binary expression"),
            Expr::Grouping { .. } => type_error("boolean", "grouping expression"),
            Expr::Unary { .. } => type_error("boolean", "unary expression"),
            Expr::Variable { .. } => type_error("boolean", "variable"),
        }
    }
}

fn type_error<T, U>(expected: T, actual: T) -> InterpreterResult<U>
where
    T: Into<String>,
{
    Err(InterpreterError::type_error(expected.into(), actual.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expr_print_literal() -> InterpreterResult<()> {
        let e = Expr::literal_string("hello");
        assert_eq!(e.print()?, String::from("hello"));
        let e = Expr::literal_num(3.0);
        assert_eq!(e.print()?, String::from("3"));
        let e = Expr::literal_nil();
        assert_eq!(e.print()?, String::from("nil"));
        Ok(())
    }
    #[test]
    fn expr_grouping() -> InterpreterResult<()> {
        let e = Expr::Grouping {
            expression: Box::new(Expr::literal_nil()),
        };
        assert_eq!(e.print()?, String::from("(grouping nil)"));
        Ok(())
    }
    #[test]
    fn expr_binary() -> InterpreterResult<()> {
        let e = Expr::Binary {
            left: Box::new(Expr::literal_num(1.0)),
            operator: Token::Plus { line: 0 },
            right: Box::new(Expr::literal_num(2.0)),
        };
        assert_eq!(e.print()?, String::from("(+ 1 2)"));
        Ok(())
    }
    #[test]
    fn expr_unary() -> InterpreterResult<()> {
        let e = Expr::Unary {
            operator: Token::Minus { line: 0 },
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(e.print()?, String::from("(- 1)"));
        Ok(())
    }
}
