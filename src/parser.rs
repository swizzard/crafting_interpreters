use crate::errors::{InterpreterError, InterpreterResult};
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::Token;

pub fn parse(tokens: Vec<Token>) -> (Option<Stmt>, Vec<InterpreterError>) {
    let mut pos: usize = 0;
    let mut errors: Vec<InterpreterError> = Vec::default();
    let mut final_stmt: Option<Stmt> = None;
    let cleaned = clean_tokens(tokens);
    loop {
        match declaration(&cleaned, &mut pos, 0) {
            Ok(stmt) => {
                final_stmt.replace(stmt);
                break;
            }
            Err(err) => {
                errors.push(err);
                if synchronize(&cleaned, &mut pos) {
                    continue;
                } else {
                    break;
                }
            }
        }
    }
    (final_stmt, errors)
}

fn declaration(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Stmt> {
    if match_var(tokens, pos) {
        variable(tokens, pos, line)
    } else {
        statement(tokens, pos, line)
    }
}

fn variable(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Stmt> {
    let name = identifier(tokens, pos, line)?;
    let initializer = if match_assign(tokens, pos) {
        Some(Box::new(expression(tokens, pos, line)?))
    } else {
        None
    };
    expect_semicolon(tokens, pos, line)?;
    Ok(Stmt::Variable { name, initializer })
}

fn statement(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Stmt> {
    if match_print(tokens, pos) {
        let expr = expression(tokens, pos, line)?;
        expect_semicolon(tokens, pos, line)?;
        Ok(Stmt::Print {
            expr: Box::new(expr),
        })
    } else if match_block(tokens, pos) {
        let stmts = block(tokens, pos, line)?;
        expect_semicolon(tokens, pos, line)?;
        Ok(Stmt::Block { stmts })
    } else {
        let expr = expression(tokens, pos, line)?;
        expect_semicolon(tokens, pos, line)?;
        Ok(Stmt::Expr {
            expr: Box::new(expr),
        })
    }
}

fn block(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Vec<Stmt>> {
    let mut statements = Vec::default();
    while !check_right_brace(tokens, pos) {
        statements.push(declaration(tokens, pos, line)?);
    }
    expect_right_brace(tokens, pos, line)?;
    Ok(statements)
}

fn expression(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    assign(tokens, pos, line)
}

fn assign(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    let expr = equality(tokens, pos, line)?;
    if match_assign(tokens, pos) {
        let equals = previous(tokens, pos, line)?;
        let value = assign(tokens, pos, line)?;
        match expr {
            Expr::Variable { name, .. } => Ok(Expr::Assign {
                name,
                value: Box::new(value),
            }),
            _ => Err(InterpreterError::SyntaxError {
                line,
                message: format!("Invalid assignment target {:?}", equals),
            }),
        }
    } else {
        Ok(expr)
    }
}

fn equality(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    let mut expr = comparison(tokens, pos, line)?;
    while match_eq(tokens, pos) {
        let operator = previous(tokens, pos, line)?;
        let right = comparison(tokens, pos, line)?;
        let new_expr = Expr::Binary {
            left: Box::new(expr),
            operator: operator.clone(),
            right: Box::new(right),
        };
        expr = new_expr;
    }
    Ok(expr)
}

fn comparison(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    let mut expr = term(tokens, pos, line)?;
    while match_comp(tokens, pos) {
        let operator = previous(tokens, pos, line)?;
        let right = term(tokens, pos, line)?;
        let new_expr = Expr::Binary {
            left: Box::new(expr),
            operator: operator.clone(),
            right: Box::new(right),
        };
        expr = new_expr;
    }
    Ok(expr)
}

fn term(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    let mut expr = factor(tokens, pos, line)?;
    while match_term(tokens, pos) {
        let operator = previous(tokens, pos, line)?;
        let right = factor(tokens, pos, line)?;
        let new_expr = Expr::Binary {
            left: Box::new(expr),
            operator: operator.clone(),
            right: Box::new(right),
        };
        expr = new_expr;
    }
    Ok(expr)
}

fn factor(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    let mut expr = unary(tokens, pos, line)?;
    while match_factor(tokens, pos) {
        let operator = previous(tokens, pos, line)?;
        let right = unary(tokens, pos, line)?;
        let new_expr = Expr::Binary {
            left: Box::new(expr),
            operator: operator.clone(),
            right: Box::new(right),
        };
        expr = new_expr;
    }
    Ok(expr)
}

fn unary(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    if match_unary(tokens, pos) {
        let operator = previous(tokens, pos, line)?;
        let right = unary(tokens, pos, line)?;
        Ok(Expr::Unary {
            operator: operator.clone(),
            right: Box::new(right),
        })
    } else {
        primary(tokens, pos, line)
    }
}

fn primary(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    let t = tokens.get(*pos).ok_or(InterpreterError::Parse { line })?;
    match t {
        Token::True { .. } => {
            *pos += 1;
            Ok(Expr::literal_bool(true))
        }
        Token::False { .. } => {
            *pos += 1;
            Ok(Expr::literal_bool(false))
        }
        Token::Nil { .. } => {
            *pos += 1;
            Ok(Expr::literal_nil())
        }
        Token::Number { literal, .. } => {
            *pos += 1;
            Ok(Expr::literal_num(*literal))
        }
        Token::r#String { literal, .. } => {
            *pos += 1;
            Ok(Expr::literal_string(literal))
        }
        Token::LeftParen { line } => {
            *pos += 1;
            let expr = expression(tokens, pos, *line)?;
            let next = tokens
                .get(*pos)
                .ok_or(InterpreterError::Parse { line: *line })?;
            if let Token::RightParen { .. } = *next {
                Ok(Expr::Grouping {
                    expression: Box::new(expr),
                })
            } else {
                Err(InterpreterError::Parse { line: *line })
            }
        }
        ident @ Token::Identifier { .. } => {
            *pos += 1;
            Ok(Expr::Variable {
                name: ident.clone(),
            })
        }
        t => Err(InterpreterError::Parse {
            line: t.get_line().unwrap_or(line),
        }),
    }
}

fn identifier(tokens: &[Token], pos: &mut usize, line: usize) -> InterpreterResult<Token> {
    if let Some(ident @ Token::Identifier { .. }) = tokens.get(*pos) {
        *pos += 1;
        Ok(ident.clone())
    } else {
        Err(InterpreterError::SyntaxError {
            line,
            message: "Expected variable name".into(),
        })
    }
}

fn match_eq(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::BangEqual { .. } | Token::EqualEqual { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_comp(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Greater { .. }
        | Token::GreaterEqual { .. }
        | Token::Less { .. }
        | Token::LessEqual { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_term(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Minus { .. } | Token::Plus { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_factor(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Slash { .. } | Token::Star { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_unary(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Bang { .. } | Token::Minus { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_print(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Print { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_var(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Var { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_assign(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::Equal { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn match_block(tokens: &[Token], pos: &mut usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::LeftBrace { .. } => {
            *pos += 1;
            true
        }
        _ => false,
    })
}

fn check_right_brace(tokens: &[Token], pos: &usize) -> bool {
    tokens.get(*pos).map_or(false, |t| match t {
        Token::RightBrace { .. } => true,
        _ => false,
    })
}

fn previous<'a>(tokens: &'a [Token], pos: &usize, line: usize) -> InterpreterResult<&'a Token> {
    tokens.get(*pos - 1).ok_or(InterpreterError::Parse { line })
}

fn expect_semicolon(tokens: &[Token], pos: &mut usize, line: usize) -> InterpreterResult<()> {
    if let Some(Token::Semicolon { .. }) = tokens.get(*pos) {
        *pos += 1;
        Ok(())
    } else {
        Err(InterpreterError::SyntaxError {
            line,
            message: "Expected semicolon".into(),
        })
    }
}

fn expect_right_brace(tokens: &[Token], pos: &mut usize, line: usize) -> InterpreterResult<()> {
    if let Some(Token::RightBrace { .. }) = tokens.get(*pos) {
        *pos += 1;
        Ok(())
    } else {
        Err(InterpreterError::SyntaxError {
            line,
            message: "Expected right brace".into(),
        })
    }
}

fn synchronize(tokens: &[Token], pos: &mut usize) -> bool {
    *pos += 1;
    while let Some(t) = tokens.get(*pos) {
        if let Some(Token::Semicolon { .. }) = previous(tokens, pos, 0).ok().as_ref() {
            return true;
        } else {
            match t {
                Token::Class { .. }
                | Token::Fun { .. }
                | Token::Var { .. }
                | Token::For { .. }
                | Token::If { .. }
                | Token::While { .. }
                | Token::Print { .. }
                | Token::Return { .. } => return true,
                _ => *pos += 1,
            }
        }
    }
    false
}

fn clean_tokens(tokens: Vec<Token>) -> Vec<Token> {
    tokens
        .into_iter()
        .filter(|t| !matches!(t, Token::Comment | Token::Whitespace))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::token::Token;

    #[test]
    fn parser_primary() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![Token::True { line: 0 }];
        assert_eq!(primary(&ts, &mut pos, 0)?, Expr::literal_bool(true));
        let mut pos: usize = 0;
        let ts = vec![Token::False { line: 0 }];
        assert_eq!(primary(&ts, &mut pos, 0)?, Expr::literal_bool(false));
        let mut pos: usize = 0;
        let ts = vec![Token::Nil { line: 0 }];
        assert_eq!(primary(&ts, &mut pos, 0)?, Expr::literal_nil());
        let mut pos: usize = 0;
        let ts = vec![Token::Number {
            lexeme: String::from("3.0"),
            literal: 3.0,
            line: 0,
        }];
        assert_eq!(primary(&ts, &mut pos, 0)?, Expr::literal_num(3.0));
        let mut pos = 0;
        let ts = vec![Token::r#String {
            lexeme: String::from("hello"),
            literal: String::from("hello"),
            line: 0,
        }];
        assert_eq!(primary(&ts, &mut pos, 0)?, Expr::literal_string("hello"));
        Ok(())
    }
    #[test]
    fn parser_primary_grouping() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::LeftParen { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
            Token::RightParen { line: 0 },
        ];
        let expected = Expr::Grouping {
            expression: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(primary(&ts, &mut pos, 0)?, expected);
        let mut pos: usize = 0;
        let ts = vec![
            Token::LeftParen { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
            Token::Semicolon { line: 0 },
        ];
        let err = primary(&ts, &mut pos, 0).unwrap_err();
        assert!(matches!(err, InterpreterError::Parse { line: 0 }));
        Ok(())
    }
    #[test]
    fn parser_unary() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![Token::Bang { line: 0 }, Token::False { line: 0 }];
        let expected = Expr::Unary {
            operator: Token::Bang { line: 0 },
            right: Box::new(Expr::literal_bool(false)),
        };
        assert_eq!(unary(&ts, &mut pos, 0)?, expected);
        let mut pos: usize = 0;
        let ts = vec![
            Token::Minus { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
        ];
        let expected = Expr::Unary {
            operator: Token::Minus { line: 0 },
            right: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(unary(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    fn parser_factor() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::Number {
                lexeme: String::from("2.0"),
                literal: 2.0,
                line: 0,
            },
            Token::Slash { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
        ];
        let expected = Expr::Binary {
            left: Box::new(Expr::literal_num(2.0)),
            operator: Token::Slash { line: 0 },
            right: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(factor(&ts, &mut pos, 0)?, expected);
        let mut pos: usize = 0;
        let ts = vec![
            Token::Number {
                lexeme: String::from("2.0"),
                literal: 2.0,
                line: 0,
            },
            Token::Star { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
        ];
        let expected = Expr::Binary {
            left: Box::new(Expr::literal_num(2.0)),
            operator: Token::Star { line: 0 },
            right: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(factor(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    fn parser_term() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
            Token::Plus { line: 0 },
            Token::Number {
                lexeme: String::from("2.0"),
                literal: 2.0,
                line: 0,
            },
        ];
        let expected = Expr::Binary {
            left: Box::new(Expr::literal_num(3.0)),
            operator: Token::Plus { line: 0 },
            right: Box::new(Expr::literal_num(2.0)),
        };
        assert_eq!(term(&ts, &mut pos, 0)?, expected);
        let mut pos: usize = 0;
        let ts = vec![
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
            Token::Minus { line: 0 },
            Token::Number {
                lexeme: String::from("2.0"),
                literal: 2.0,
                line: 0,
            },
        ];
        let expected = Expr::Binary {
            left: Box::new(Expr::literal_num(3.0)),
            operator: Token::Minus { line: 0 },
            right: Box::new(Expr::literal_num(2.0)),
        };
        assert_eq!(term(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    fn parser_comparison() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
            Token::Minus { line: 0 },
            Token::Number {
                lexeme: String::from("2.0"),
                literal: 2.0,
                line: 0,
            },
            Token::LessEqual { line: 0 },
            Token::Number {
                lexeme: String::from("1.0"),
                literal: 1.0,
                line: 0,
            },
            Token::Plus { line: 0 },
            Token::Number {
                lexeme: String::from("4.0"),
                literal: 4.0,
                line: 0,
            },
        ];
        let expected = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::literal_num(3.0)),
                operator: Token::Minus { line: 0 },
                right: Box::new(Expr::literal_num(2.0)),
            }),
            operator: Token::LessEqual { line: 0 },
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::literal_num(1.0)),
                operator: Token::Plus { line: 0 },
                right: Box::new(Expr::literal_num(4.0)),
            }),
        };
        assert_eq!(comparison(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    fn parser_equality() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::r#String {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            Token::EqualEqual { line: 0 },
            Token::r#String {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
        ];
        let expected = Expr::Binary {
            left: Box::new(Expr::literal_string("foo")),
            operator: Token::EqualEqual { line: 0 },
            right: Box::new(Expr::literal_string("foo")),
        };
        assert_eq!(equality(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    fn parser_variable_initializer() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::Var { line: 0 },
            Token::Identifier {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            Token::Equal { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
            Token::Semicolon { line: 0 },
        ];
        let expected = Stmt::Variable {
            name: Token::Identifier {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            initializer: Some(Box::new(Expr::literal_num(3.0))),
        };
        let actual = declaration(&ts, &mut pos, 0)?;
        println!("{:?}", actual);
        assert_eq!(actual, expected);
        Ok(())
    }
    #[test]
    fn parser_variable_no_initializer() -> InterpreterResult<()> {
        let mut pos: usize = 0;
        let ts = vec![
            Token::Var { line: 0 },
            Token::Identifier {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            Token::Semicolon { line: 0 },
        ];
        let expected = Stmt::Variable {
            name: Token::Identifier {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            initializer: None,
        };
        assert_eq!(declaration(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    fn parser_assign() -> InterpreterResult<()> {
        let mut pos = 0_usize;
        let ts = vec![
            Token::Identifier {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            Token::Equal { line: 0 },
            Token::Number {
                lexeme: String::from("3.0"),
                literal: 3.0,
                line: 0,
            },
        ];
        let expected = Expr::Assign {
            name: Token::Identifier {
                lexeme: String::from("foo"),
                literal: String::from("foo"),
                line: 0,
            },
            value: Box::new(Expr::literal_num(3.0)),
        };
        assert_eq!(assign(&ts, &mut pos, 0)?, expected);
        Ok(())
    }
    #[test]
    #[ignore]
    // TODO(SHR): implement this once we figure out what synchronize does/how it's used
    fn test_synchronize() {
        todo!()
    }
}
