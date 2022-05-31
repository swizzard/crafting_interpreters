use crate::expr::Expr;
use crate::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Variable {
        name: Token,
        initializer: Option<Box<Expr>>,
    },
    Print {
        expr: Box<Expr>,
    },
    Expr {
        expr: Box<Expr>,
    },
}
