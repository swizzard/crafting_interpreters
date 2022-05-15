use crate::errors::{InterpreterError, InterpreterResult};
use crate::expr::Expr;
use crate::scanner::Token;

pub fn parse(tokens: Vec<Token>) -> (Option<Expr>, Vec<InterpreterError>) {
    let mut pos: usize = 0;
    let mut errors: Vec<InterpreterError> = Vec::default();
    let mut final_expr: Option<Expr> = None;
    let cleaned = clean_tokens(tokens);
    loop {
        match expression(&cleaned, &mut pos, 0) {
            Ok(expr) => {
                final_expr.replace(expr);
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
    (final_expr, errors)
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
        .filter(|t| matches!(t, Token::Comment | Token::Whitespace))
        .collect()
}

fn expression(tokens: &Vec<Token>, pos: &mut usize, line: usize) -> InterpreterResult<Expr> {
    equality(tokens, pos, line)
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
        t => Err(InterpreterError::Parse {
            line: t.get_line().unwrap_or(line),
        }),
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

fn previous<'a>(tokens: &'a [Token], pos: &usize, line: usize) -> InterpreterResult<&'a Token> {
    tokens.get(*pos - 1).ok_or(InterpreterError::Parse { line })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::scanner::Token;

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
    #[ignore]
    // TODO(SHR): implement this once we figure out what synchronize does/how it's used
    fn test_synchronize() {
        todo!()
    }
}
