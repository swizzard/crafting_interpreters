use crate::environment::Environment;
use crate::errors::{InterpreterError, InterpreterResult};
use crate::expr::{Expr, Value};
use crate::scanner::Token;
use crate::stmt::Stmt;

#[derive(Debug, Default)]
pub(crate) struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub(crate) fn interpret(&mut self, stmt: &Stmt) -> InterpreterResult<Value> {
        match stmt {
            Stmt::Expr { expr } => self.interpret_expr(expr),
            Stmt::Print { expr } => {
                let val = self.interpret_expr(expr)?;
                Self::print(val)
            }
            Stmt::Variable {
                name: Token::Identifier { literal, .. },
                initializer,
            } => {
                let val = match initializer {
                    Some(initializer) => self.interpret_expr(initializer)?,
                    None => Value::Nil,
                };
                self.env.define(String::from(literal), val);
                Ok(Value::Nil)
            }
            _ => Err(InterpreterError::SyntaxError {
                line: 0,
                message: "Invalid variable".into(),
            }),
        }
    }

    fn print(val: Value) -> InterpreterResult<Value> {
        println!("{}", val);
        Ok(Value::Nil)
    }

    fn interpret_expr(&self, expr: &Expr) -> InterpreterResult<Value> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expression } => self.interpret_grouping(expression.as_ref()),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.interpret_binary(operator, left.as_ref(), right.as_ref()),
            Expr::Unary { operator, right } => self.interpret_unary(operator, right.as_ref()),
            Expr::Variable {
                name: Token::Identifier { literal, line, .. },
            } => self.get_variable(literal, line),
            _ => Err(InterpreterError::SyntaxError {
                line: 0,
                message: "Invalid variable".into(),
            }),
        }
    }
    fn get_variable(&self, literal: &str, line: &usize) -> InterpreterResult<Value> {
        match self.env.get(literal) {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(e.add_line_to_undefined_error(*line)),
        }
    }

    fn interpret_grouping(&self, expr: &Expr) -> InterpreterResult<Value> {
        self.interpret_expr(expr)
    }

    fn interpret_binary(
        &self,
        operator: &Token,
        left: &Expr,
        right: &Expr,
    ) -> InterpreterResult<Value> {
        match operator {
            Token::Minus { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Number(left - right))
            }
            Token::Slash { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Number(left / right))
            }
            Token::Star { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Number(left * right))
            }
            Token::Plus { line } => {
                let left_num = cast_f32(left, line);
                if left_num.is_ok() {
                    let right_num = cast_f32(right, line)?;
                    left_num.map(|n| Value::Number(n + right_num))
                } else {
                    let left_str = cast_string(left, line)?;
                    let right_str = cast_string(right, line)?;
                    Ok(Value::r#String(format!("{}{}", left_str, right_str)))
                }
            }
            Token::Greater { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Bool(left > right))
            }
            Token::Less { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Bool(left < right))
            }
            Token::GreaterEqual { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Bool(left >= right))
            }
            Token::LessEqual { line } => {
                let left = cast_f32(left, line)?;
                let right = cast_f32(right, line)?;
                Ok(Value::Bool(left <= right))
            }
            Token::EqualEqual { .. } => Ok(Value::Bool(left == right)),
            Token::BangEqual { .. } => Ok(Value::Bool(left != right)),
            t => Err(InterpreterError::SyntaxError {
                line: t.get_line().unwrap_or(0),
                message: "Invalid binary expression".into(),
            }),
        }
    }

    fn interpret_unary(&self, operator: &Token, right: &Expr) -> InterpreterResult<Value> {
        match operator {
            Token::Minus { line } => {
                let num = cast_f32(right, line)?;
                Ok(Value::Number(-num))
            }
            Token::Bang { line } => {
                let b = cast_bool(right, line)?;
                Ok(Value::Bool(!b))
            }
            t => Err(InterpreterError::SyntaxError {
                line: t.get_line().unwrap_or(0),
                message: "Invalid unary expression".into(),
            }),
        }
    }
}

fn cast_f32(expr: &Expr, line: &usize) -> InterpreterResult<f32> {
    f32::try_from(expr).map_err(|e| e.add_line_to_type_error(*line))
}

fn cast_string(expr: &Expr, line: &usize) -> InterpreterResult<String> {
    String::try_from(expr).map_err(|e| e.add_line_to_type_error(*line))
}

fn cast_bool(expr: &Expr, line: &usize) -> InterpreterResult<bool> {
    bool::try_from(expr).map_err(|e| e.add_line_to_type_error(*line))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interpreter_literal() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::literal_string("hello");
        assert_eq!(
            interpreter.interpret_expr(&e)?,
            Value::r#String(String::from("hello"))
        );
        let e = Expr::literal_num(3.0);
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Number(3.0));
        let e = Expr::literal_bool(true);
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        let e = Expr::literal_nil();
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Nil);
        Ok(())
    }
    #[test]
    fn interpreter_grouping() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::Grouping {
            expression: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Number(3.0));
        Ok(())
    }
    #[test]
    fn interpreter_unary_ok() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::Unary {
            operator: Token::Minus { line: 1 },
            right: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Number(-3.0));
        let e = Expr::Unary {
            operator: Token::Bang { line: 1 },
            right: Box::new(Expr::literal_bool(true)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        Ok(())
    }
    #[test]
    fn interpreter_unary_not_ok() {
        let interpreter = Interpreter::default();
        let e = Expr::Unary {
            operator: Token::Minus { line: 1 },
            right: Box::new(Expr::literal_string("foo")),
        };
        if let Err(InterpreterError::Type {
            line,
            expected_type,
            actual_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error negating string")
        }
        let e = Expr::Unary {
            operator: Token::Bang { line: 1 },
            right: Box::new(Expr::literal_string("foo")),
        };
        if let Err(InterpreterError::Type {
            line,
            expected_type,
            actual_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("boolean"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error notting string")
        }
    }
    #[test]
    fn interpreter_binary_ok() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::Binary {
            operator: Token::Minus { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_num(2.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Number(1.0));
        let e = Expr::Binary {
            operator: Token::Slash { line: 1 },
            left: Box::new(Expr::literal_num(4.0)),
            right: Box::new(Expr::literal_num(2.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Number(2.0));
        let e = Expr::Binary {
            operator: Token::Greater { line: 1 },
            left: Box::new(Expr::literal_num(2.0)),
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        let e = Expr::Binary {
            operator: Token::Less { line: 1 },
            left: Box::new(Expr::literal_num(2.0)),
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        let e = Expr::Binary {
            operator: Token::GreaterEqual { line: 1 },
            left: Box::new(Expr::literal_num(2.0)),
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        let e = Expr::Binary {
            operator: Token::LessEqual { line: 1 },
            left: Box::new(Expr::literal_num(2.0)),
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        Ok(())
    }
    #[test]
    fn interpreter_binary_plus_ok() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::Binary {
            operator: Token::Plus { line: 1 },
            left: Box::new(Expr::literal_num(1.0)),
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Number(2.0));
        let e = Expr::Binary {
            operator: Token::Plus { line: 1 },
            left: Box::new(Expr::literal_string("hello")),
            right: Box::new(Expr::literal_string(" there")),
        };
        assert_eq!(
            interpreter.interpret_expr(&e)?,
            Value::r#String(String::from("hello there"))
        );
        Ok(())
    }
    #[test]
    fn interpreter_binary_not_ok() {
        let interpreter = Interpreter::default();
        let e = Expr::Binary {
            operator: Token::Minus { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error subtracting string from number");
        }
        let e = Expr::Binary {
            operator: Token::Slash { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error dividing number by string");
        }
        let e = Expr::Binary {
            operator: Token::Star { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error multiplying number by string");
        }
        let e = Expr::Binary {
            operator: Token::Greater { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error comparing number gt string");
        }
        let e = Expr::Binary {
            operator: Token::Less { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error comparing number lt string");
        }
        let e = Expr::Binary {
            operator: Token::GreaterEqual { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error comparing number gte string");
        }
        let e = Expr::Binary {
            operator: Token::LessEqual { line: 1 },
            left: Box::new(Expr::literal_num(3.0)),
            right: Box::new(Expr::literal_string("hello")),
        };
        if let Err(InterpreterError::Type {
            line,
            actual_type,
            expected_type,
        }) = interpreter.interpret_expr(&e)
        {
            assert_eq!(Some(1), line);
            assert_eq!(String::from("number"), expected_type);
            assert_eq!(String::from("string"), actual_type);
        } else {
            panic!("no error comparing number lte string");
        }
    }

    #[test]
    fn interpreter_binary_eq_same_type() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_num(1.0)),
            right: Box::new(Expr::literal_num(1.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_num(1.0)),
            right: Box::new(Expr::literal_num(2.0)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_string("hi")),
            right: Box::new(Expr::literal_string("hi")),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_string("hi")),
            right: Box::new(Expr::literal_string("bye")),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_bool(true)),
            right: Box::new(Expr::literal_bool(true)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_bool(true)),
            right: Box::new(Expr::literal_bool(false)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_nil()),
            right: Box::new(Expr::literal_nil()),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(true));
        Ok(())
    }
    #[test]
    fn interpreter_binary_eq_different_types() -> InterpreterResult<()> {
        let interpreter = Interpreter::default();
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_num(1.0)),
            right: Box::new(Expr::literal_string("1.0")),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_string("true")),
            right: Box::new(Expr::literal_bool(true)),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        let e = Expr::Binary {
            operator: Token::EqualEqual { line: 1 },
            left: Box::new(Expr::literal_bool(false)),
            right: Box::new(Expr::literal_nil()),
        };
        assert_eq!(interpreter.interpret_expr(&e)?, Value::Bool(false));
        Ok(())
    }
    #[test]
    fn interpreter_define_variable_initializer() -> InterpreterResult<()> {
        let mut interpreter = Interpreter::default();
        let s = Stmt::Variable {
            name: Token::Identifier {
                literal: String::from("foo"),
                lexeme: String::from("foo"),
                line: 0,
            },
            initializer: Some(Box::new(Expr::literal_num(3.0))),
        };
        interpreter.interpret(&s)?;
        assert_eq!(interpreter.get_variable("foo", &0)?, Value::Number(3.0));
        Ok(())
    }
    #[test]
    fn interpreter_define_variable_no_initializer() -> InterpreterResult<()> {
        let mut interpreter = Interpreter::default();
        let s = Stmt::Variable {
            name: Token::Identifier {
                literal: String::from("foo"),
                lexeme: String::from("foo"),
                line: 0,
            },
            initializer: None,
        };
        interpreter.interpret(&s)?;
        assert_eq!(interpreter.get_variable("foo", &0)?, Value::Nil);
        Ok(())
    }
}
