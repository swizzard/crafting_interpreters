use crate::errors::InterpreterResult;
use crate::scanner::Token;
use std::fmt::Write;

#[derive(Debug, PartialEq)]
pub enum Value {
    r#String(String),
    Number(f32),
    Bool(bool),
    Nil,
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
