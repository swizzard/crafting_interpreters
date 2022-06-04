use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Block {
        stmts: Vec<Stmt>,
    },
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

impl From<Expr> for Stmt {
    fn from(value: Expr) -> Stmt {
        Stmt::Expr {
            expr: Box::new(value),
        }
    }
}

impl From<Stmt> for Expr {
    fn from(value: Stmt) -> Expr {
        match value {
            Stmt::Expr { expr } => *expr.clone(),
            _ => Expr::literal_nil(),
        }
    }
}
