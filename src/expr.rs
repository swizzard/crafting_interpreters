use crate::errors::InterpreterResult;
use crate::scanner::Token;
use std::fmt::Write;

#[derive(Debug, PartialEq)]
pub enum Value {
    r#String(String),
    Number(f32),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::r#String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Nil => f.write_str("nil"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
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
}

#[derive(Default)]
struct ExprPrinter {
    s: String,
}

impl ExprPrinter {
    fn build(self, expr: &Expr) -> InterpreterResult<Self> {
        match expr {
            Expr::Literal { value } => self.build_literal(value),
            Expr::Grouping { expression } => self.build_grouping(expression.as_ref()),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.build_binary(operator, left.as_ref(), right.as_ref()),
            Expr::Unary { operator, right } => self.build_unary(operator, right.as_ref()),
        }
    }
    fn print(self) -> InterpreterResult<String> {
        Ok(self.s)
    }
    fn build_literal(mut self, value: &Value) -> InterpreterResult<Self> {
        write!(&mut self.s, "{}", value)?;
        Ok(self)
    }
    fn build_grouping(self, expr: &Expr) -> InterpreterResult<Self> {
        self.l_paren("grouping")?.build(expr)?.r_paren()
    }
    fn build_binary(self, operator: &Token, left: &Expr, right: &Expr) -> InterpreterResult<Self> {
        self.l_paren(&format!("{}", operator))?
            .build(left)?
            .space()?
            .build(right)?
            .r_paren()
    }
    fn build_unary(self, operator: &Token, right: &Expr) -> InterpreterResult<Self> {
        self.l_paren(&format!("{}", operator))?
            .build(right)?
            .r_paren()
    }
    fn l_paren(mut self, name: &str) -> InterpreterResult<Self> {
        write!(&mut self.s, "({} ", name)?;
        Ok(self)
    }
    fn r_paren(mut self) -> InterpreterResult<Self> {
        self.s.write_str(")")?;
        Ok(self)
    }
    fn space(mut self) -> InterpreterResult<Self> {
        self.s.write_str(" ")?;
        Ok(self)
    }
}

impl Expr {
    pub fn print(&self) -> InterpreterResult<String> {
        ExprPrinter::default().build(self)?.print()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_print_literal() -> InterpreterResult<()> {
        let e = Expr::Literal {
            value: Value::r#String("hello".into()),
        };
        assert_eq!(e.print()?, String::from("hello"));
        let e = Expr::Literal {
            value: Value::Number(3.0),
        };
        assert_eq!(e.print()?, String::from("3"));
        let e = Expr::Literal { value: Value::Nil };
        assert_eq!(e.print()?, String::from("nil"));
        Ok(())
    }
    #[test]
    fn test_grouping() -> InterpreterResult<()> {
        let e = Expr::Grouping {
            expression: Box::new(Expr::Literal { value: Value::Nil }),
        };
        assert_eq!(e.print()?, String::from("(grouping nil)"));
        Ok(())
    }
    #[test]
    fn test_binary() -> InterpreterResult<()> {
        let e = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Value::Number(1.0),
            }),
            operator: Token::Plus { line: 0 },
            right: Box::new(Expr::Literal {
                value: Value::Number(2.0),
            }),
        };
        assert_eq!(e.print()?, String::from("(+ 1 2)"));
        Ok(())
    }
    #[test]
    fn test_unary() -> InterpreterResult<()> {
        let e = Expr::Unary {
            operator: Token::Minus { line: 0 },
            right: Box::new(Expr::Literal {
                value: Value::Number(1.0),
            }),
        };
        assert_eq!(e.print()?, String::from("(- 1)"));
        Ok(())
    }
}
