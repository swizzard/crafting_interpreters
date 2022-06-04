use crate::errors::{InterpreterError, InterpreterResult};
use crate::token::Token;
use peekmore::{PeekMore, PeekMoreIterator};
use std::str::Chars;

type Cs<'a> = PeekMoreIterator<Chars<'a>>;

pub(crate) fn scan_tokens(s: String) -> InterpreterResult<Vec<Token>> {
    let mut tokens = Vec::with_capacity(s.capacity());
    let mut chars = s.chars().peekmore();
    let mut line = 1;
    let mut errored = false;
    while let Some(result) = scan_token(&mut chars, &mut line) {
        match result {
            Ok(t) => tokens.push(t),
            Err(e) => {
                errored = true;
                println!("{:?}", e)
            }
        };
    }
    tokens.push(Token::Eof { line });
    if errored {
        Err(InterpreterError::Parse { line })
    } else {
        Ok(tokens)
    }
}

fn scan_token(cs: &mut Cs<'_>, line: &mut usize) -> Option<InterpreterResult<Token>> {
    match cs.next() {
        Some('(') => Some(Ok(Token::LeftParen { line: *line })),
        Some(')') => Some(Ok(Token::RightParen { line: *line })),
        Some('{') => Some(Ok(Token::LeftBrace { line: *line })),
        Some('}') => Some(Ok(Token::RightBrace { line: *line })),
        Some(',') => Some(Ok(Token::Comma { line: *line })),
        Some('.') => Some(Ok(Token::Dot { line: *line })),
        Some('-') => Some(Ok(Token::Minus { line: *line })),
        Some('+') => Some(Ok(Token::Plus { line: *line })),
        Some(';') => Some(Ok(Token::Semicolon { line: *line })),
        Some('*') => Some(Ok(Token::Star { line: *line })),
        Some('!') => {
            if match_c(cs, '=') {
                Some(Ok(Token::BangEqual { line: *line }))
            } else {
                Some(Ok(Token::Bang { line: *line }))
            }
        }
        Some('=') => {
            if match_c(cs, '=') {
                Some(Ok(Token::EqualEqual { line: *line }))
            } else {
                Some(Ok(Token::Equal { line: *line }))
            }
        }
        Some('<') => {
            if match_c(cs, '=') {
                Some(Ok(Token::LessEqual { line: *line }))
            } else {
                Some(Ok(Token::Less { line: *line }))
            }
        }
        Some('>') => {
            if match_c(cs, '=') {
                Some(Ok(Token::GreaterEqual { line: *line }))
            } else {
                Some(Ok(Token::Greater { line: *line }))
            }
        }
        Some('/') => Some(match_slash(cs, *line)),
        Some('"') => Some(string(cs, line)),
        Some(c) if c.is_ascii_whitespace() => Some(whitespace(c, cs, line)),
        Some(c) if c.is_ascii_digit() => Some(number(c, cs, *line)),
        Some(c) if c.is_ascii_alphabetic() || c == '_' => Some(identifier(c, cs, *line)),
        Some(c) => Some(Err(InterpreterError::Interpreter {
            line: *line,
            message: format!("Unknown token {c}"),
        })),
        None => None,
    }
}

fn match_c(cs: &mut Cs<'_>, to_match: char) -> bool {
    if let Some(c) = cs.peek() {
        if *c == to_match {
            cs.next();
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn match_slash(cs: &mut Cs<'_>, line: usize) -> InterpreterResult<Token> {
    if match_c(cs, '/') {
        while let Some(c) = cs.peek() {
            if *c == '\n' {
                break;
            } else {
                cs.next();
            }
        }
        Ok(Token::Comment)
    } else {
        Ok(Token::Slash { line })
    }
}

fn whitespace(c: char, cs: &mut Cs<'_>, line: &mut usize) -> InterpreterResult<Token> {
    if c == '\n' {
        *line += 1;
    }
    while let Some(c) = cs.peek() {
        if *c == '\n' {
            *line += 1;
            cs.next();
        } else if c.is_ascii_whitespace() {
            cs.next();
        } else {
            break;
        }
    }
    Ok(Token::Whitespace)
}

fn string(cs: &mut Cs<'_>, line: &mut usize) -> InterpreterResult<Token> {
    let mut s = String::default();
    while let Some(c) = cs.peek() {
        match c {
            '"' => {
                cs.next();
                return Ok(Token::r#String {
                    lexeme: s.clone(),
                    literal: s,
                    line: *line,
                });
            }
            '\n' => {
                s.push(cs.next().unwrap());
                *line += 1;
            }
            _ => {
                s.push(cs.next().unwrap());
            }
        }
    }
    Err(InterpreterError::Interpreter {
        line: *line,
        message: String::from("Unterminated string"),
    })
}

fn number(c: char, cs: &mut Cs<'_>, line: usize) -> InterpreterResult<Token> {
    let mut s = String::from(c);
    while let Some(c) = cs.peek() {
        match c {
            '.' => {
                if let Some(nxt) = cs.peek_nth(2) {
                    if nxt.is_ascii_digit() {
                        s.push(cs.next().unwrap());
                    }
                } else {
                    break;
                }
            }
            ch if ch.is_ascii_digit() => {
                s.push(cs.next().unwrap());
            }
            _ => break,
        }
    }
    if let Ok(literal) = s.parse::<f32>() {
        Ok(Token::Number {
            lexeme: s,
            literal,
            line,
        })
    } else {
        Err(InterpreterError::Interpreter {
            line,
            message: format!("Invalid number: {s}"),
        })
    }
}

fn identifier(c: char, cs: &mut Cs<'_>, line: usize) -> InterpreterResult<Token> {
    let mut s = String::from(c);
    while let Some(c) = cs.peek() {
        if c.is_ascii_alphanumeric() {
            s.push(cs.next().unwrap());
        } else {
            break;
        }
    }
    ident_t(s, line)
}

fn ident_t(s: String, line: usize) -> InterpreterResult<Token> {
    let res = match s.as_str() {
        "and" => Token::And { line },
        "class" => Token::Class { line },
        "else" => Token::Else { line },
        "false" => Token::False { line },
        "for" => Token::For { line },
        "fun" => Token::Fun { line },
        "if" => Token::If { line },
        "nil" => Token::Nil { line },
        "or" => Token::Or { line },
        "print" => Token::Print { line },
        "return" => Token::Return { line },
        "super" => Token::Super { line },
        "this" => Token::This { line },
        "true" => Token::True { line },
        "var" => Token::Var { line },
        "while" => Token::While { line },
        _ => Token::Identifier {
            lexeme: s.clone(),
            literal: s,
            line,
        },
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    fn st(s: &str) -> InterpreterResult<Vec<Token>> {
        scan_tokens(s.into())
    }
    #[test]
    fn scanner_singletons() -> InterpreterResult<()> {
        assert_eq!(Token::LeftParen { line: 1 }, st("(")?[0]);
        assert_eq!(Token::RightParen { line: 1 }, st(")")?[0]);
        assert_eq!(Token::LeftBrace { line: 1 }, st("{")?[0]);
        assert_eq!(Token::RightBrace { line: 1 }, st("}")?[0]);
        assert_eq!(Token::Comma { line: 1 }, st(",")?[0]);
        assert_eq!(Token::Dot { line: 1 }, st(".")?[0]);
        assert_eq!(Token::Minus { line: 1 }, st("-")?[0]);
        assert_eq!(Token::Plus { line: 1 }, st("+")?[0]);
        assert_eq!(Token::Semicolon { line: 1 }, st(";")?[0]);
        assert_eq!(Token::Star { line: 1 }, st("*")?[0]);
        Ok(())
    }
    #[test]
    fn scanner_bang() -> InterpreterResult<()> {
        assert_eq!(Token::BangEqual { line: 1 }, st("!=")?[0]);
        assert_eq!(Token::Bang { line: 1 }, st("!")?[0]);
        let res = st("!,")?;
        assert_eq!(Token::Bang { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        let res = st("!=,")?;
        assert_eq!(Token::BangEqual { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        Ok(())
    }
    #[test]
    fn scanner_eq() -> InterpreterResult<()> {
        assert_eq!(Token::Equal { line: 1 }, st("=")?[0]);
        assert_eq!(Token::EqualEqual { line: 1 }, st("==")?[0]);
        let res = st("=,")?;
        assert_eq!(Token::Equal { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        let res = st("==,")?;
        assert_eq!(Token::EqualEqual { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        Ok(())
    }
    #[test]
    fn scanner_lt() -> InterpreterResult<()> {
        assert_eq!(Token::Less { line: 1 }, st("<")?[0]);
        assert_eq!(Token::LessEqual { line: 1 }, st("<=")?[0]);
        let res = st("<,")?;
        assert_eq!(Token::Less { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        let res = st("<=,")?;
        assert_eq!(Token::LessEqual { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        Ok(())
    }
    #[test]
    fn scanner_gt() -> InterpreterResult<()> {
        assert_eq!(Token::Greater { line: 1 }, st(">")?[0]);
        assert_eq!(Token::GreaterEqual { line: 1 }, st(">=")?[0]);
        let res = st(">,")?;
        assert_eq!(Token::Greater { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        let res = st(">=,")?;
        assert_eq!(Token::GreaterEqual { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        Ok(())
    }
    #[test]
    fn scanner_slash() -> InterpreterResult<()> {
        assert_eq!(Token::Comment, st("// comment\n")?[0]);
        assert_eq!(Token::Slash { line: 1 }, st("/")?[0]);
        let res = st("/,")?;
        assert_eq!(Token::Slash { line: 1 }, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        let res = st("// comment\n,")?;
        assert_eq!(Token::Comment, res[0]);
        assert_eq!(Token::Whitespace, res[1]);
        assert_eq!(Token::Comma { line: 2 }, res[2]);
        Ok(())
    }
    #[test]
    fn scanner_string() -> InterpreterResult<()> {
        let res = st("\"foo\"")?;
        assert_eq!(
            Token::r#String {
                lexeme: "foo".into(),
                literal: "foo".into(),
                line: 1
            },
            res[0]
        );
        let res = st("\"foo\nbar\"")?;
        assert_eq!(
            Token::r#String {
                lexeme: "foo\nbar".into(),
                literal: "foo\nbar".into(),
                line: 2
            },
            res[0]
        );
        let res = st("\"foo,\",")?;
        assert_eq!(
            Token::r#String {
                lexeme: "foo,".into(),
                literal: "foo,".into(),
                line: 1
            },
            res[0]
        );
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        Ok(())
    }
    #[test]
    fn scanner_whitespace_dont_inc_line() -> InterpreterResult<()> {
        assert_eq!(Token::Whitespace, st(" ")?[0]);
        assert_eq!(Token::Whitespace, st("\t")?[0]);
        assert_eq!(Token::Whitespace, st("     \t\t\r   ")?[0]);
        Ok(())
    }
    #[test]
    fn scanner_whitespace_inc_line() -> InterpreterResult<()> {
        let res = st("  ,\n,  ")?;
        assert_eq!(Token::Whitespace, res[0]);
        assert_eq!(Token::Comma { line: 1 }, res[1]);
        assert_eq!(Token::Whitespace, res[2]);
        assert_eq!(Token::Comma { line: 2 }, res[3]);
        assert_eq!(Token::Whitespace, res[4]);
        Ok(())
    }
    #[test]
    fn scanner_number() -> InterpreterResult<()> {
        assert_eq!(
            Token::Number {
                lexeme: "32".into(),
                literal: 32.0,
                line: 1
            },
            st("32")?[0]
        );
        assert_eq!(
            Token::Number {
                lexeme: "32.50".into(),
                literal: 32.5,
                line: 1
            },
            st("32.50")?[0]
        );
        let res = st("32.50.3")?;
        assert_eq!(
            Token::Number {
                lexeme: "32.50".into(),
                literal: 32.5,
                line: 1
            },
            res[0]
        );
        assert_eq!(Token::Dot { line: 1 }, res[1]);
        assert_eq!(
            Token::Number {
                lexeme: "3".into(),
                literal: 3.0,
                line: 1
            },
            res[2]
        );
        let res = st("32.,")?;
        assert_eq!(
            Token::Number {
                lexeme: "32".into(),
                literal: 32.0,
                line: 1
            },
            res[0]
        );
        assert_eq!(Token::Dot { line: 1 }, res[1]);
        assert_eq!(Token::Comma { line: 1 }, res[2]);
        Ok(())
    }
    #[test]
    fn scanner_non_reserved_identifier() -> InterpreterResult<()> {
        assert_eq!(
            Token::Identifier {
                lexeme: "_foo".into(),
                literal: "_foo".into(),
                line: 1
            },
            st("_foo")?[0]
        );
        let res = st("1foo")?;
        assert_eq!(
            Token::Number {
                lexeme: "1".into(),
                literal: 1.0,
                line: 1
            },
            res[0]
        );
        assert_eq!(
            Token::Identifier {
                lexeme: "foo".into(),
                literal: "foo".into(),
                line: 1
            },
            res[1]
        );
        assert_eq!(
            Token::Identifier {
                lexeme: "organ".into(),
                literal: "organ".into(),
                line: 1
            },
            st("organ")?[0]
        );
        Ok(())
    }
    #[test]
    fn scanner_reserved_identifier() -> InterpreterResult<()> {
        assert_eq!(Token::And { line: 1 }, st("and")?[0]);
        assert_eq!(Token::Class { line: 1 }, st("class")?[0]);
        assert_eq!(Token::Else { line: 1 }, st("else")?[0]);
        assert_eq!(Token::False { line: 1 }, st("false")?[0]);
        assert_eq!(Token::For { line: 1 }, st("for")?[0]);
        assert_eq!(Token::Fun { line: 1 }, st("fun")?[0]);
        assert_eq!(Token::If { line: 1 }, st("if")?[0]);
        assert_eq!(Token::Nil { line: 1 }, st("nil")?[0]);
        assert_eq!(Token::Or { line: 1 }, st("or")?[0]);
        assert_eq!(Token::Print { line: 1 }, st("print")?[0]);
        assert_eq!(Token::Return { line: 1 }, st("return")?[0]);
        assert_eq!(Token::Super { line: 1 }, st("super")?[0]);
        assert_eq!(Token::This { line: 1 }, st("this")?[0]);
        assert_eq!(Token::True { line: 1 }, st("true")?[0]);
        assert_eq!(Token::Var { line: 1 }, st("var")?[0]);
        assert_eq!(Token::While { line: 1 }, st("while")?[0]);
        Ok(())
    }
}
