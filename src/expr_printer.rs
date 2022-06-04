use crate::errors::InterpreterResult;
use crate::expr::Expr;
use crate::token::Token;
use crate::value::Value;
use std::fmt::Write;

#[derive(Default)]
pub struct ExprPrinter {
    s: String,
}

impl ExprPrinter {
    pub fn build(self, expr: &Expr) -> InterpreterResult<Self> {
        match expr {
            Expr::Literal { value } => self.build_literal(value),
            Expr::Grouping { expression } => self.build_grouping(expression.as_ref()),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.build_binary(operator, left.as_ref(), right.as_ref()),
            Expr::Unary { operator, right } => self.build_unary(operator, right.as_ref()),
            Expr::Variable { name } => self.build_variable(name),
            Expr::Assign { .. } => todo!(),
        }
    }
    pub fn print(self) -> InterpreterResult<String> {
        Ok(self.s)
    }
    fn build_literal(mut self, value: &Value) -> InterpreterResult<Self> {
        write!(&mut self.s, "{}", value)?;
        Ok(self)
    }
    fn build_variable(mut self, name: &Token) -> InterpreterResult<Self> {
        write!(&mut self.s, "{}", name)?;
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
