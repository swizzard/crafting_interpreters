#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // single-character tokens
    LeftParen {
        line: usize,
    },
    RightParen {
        line: usize,
    },
    LeftBrace {
        line: usize,
    },
    RightBrace {
        line: usize,
    },
    Comma {
        line: usize,
    },
    Dot {
        line: usize,
    },
    Minus {
        line: usize,
    },
    Plus {
        line: usize,
    },
    Semicolon {
        line: usize,
    },
    Slash {
        line: usize,
    },
    Star {
        line: usize,
    },
    // one or two character tokens
    Bang {
        line: usize,
    },
    BangEqual {
        line: usize,
    },
    Equal {
        line: usize,
    },
    EqualEqual {
        line: usize,
    },
    Greater {
        line: usize,
    },
    GreaterEqual {
        line: usize,
    },
    Less {
        line: usize,
    },
    LessEqual {
        line: usize,
    },
    // literals
    Identifier {
        lexeme: String,
        literal: String,
        line: usize,
    },
    r#String {
        lexeme: String,
        literal: String,
        line: usize,
    },
    Number {
        lexeme: String,
        literal: f32,
        line: usize,
    },
    // keywords
    And {
        line: usize,
    },
    Class {
        line: usize,
    },
    Else {
        line: usize,
    },
    False {
        line: usize,
    },
    Fun {
        line: usize,
    },
    For {
        line: usize,
    },
    If {
        line: usize,
    },
    Nil {
        line: usize,
    },
    Or {
        line: usize,
    },
    Print {
        line: usize,
    },
    Return {
        line: usize,
    },
    Super {
        line: usize,
    },
    This {
        line: usize,
    },
    True {
        line: usize,
    },
    Var {
        line: usize,
    },
    While {
        line: usize,
    },
    Eof {
        line: usize,
    },
    Comment,
    Whitespace,
}

impl Token {
    pub(crate) fn get_line(&self) -> Option<usize> {
        use Token::*;
        match self {
            Comment | Whitespace => None,
            LeftParen { line } => Some(*line),
            RightParen { line } => Some(*line),
            LeftBrace { line } => Some(*line),
            RightBrace { line } => Some(*line),
            Comma { line } => Some(*line),
            Dot { line } => Some(*line),
            Minus { line } => Some(*line),
            Plus { line } => Some(*line),
            Semicolon { line } => Some(*line),
            Slash { line } => Some(*line),
            Star { line } => Some(*line),
            Bang { line } => Some(*line),
            BangEqual { line } => Some(*line),
            Equal { line } => Some(*line),
            EqualEqual { line } => Some(*line),
            Greater { line } => Some(*line),
            GreaterEqual { line } => Some(*line),
            Less { line } => Some(*line),
            LessEqual { line } => Some(*line),
            Identifier { line, .. } => Some(*line),
            r#String { line, .. } => Some(*line),
            Number { line, .. } => Some(*line),
            And { line } => Some(*line),
            Class { line } => Some(*line),
            Else { line } => Some(*line),
            False { line } => Some(*line),
            Fun { line } => Some(*line),
            For { line } => Some(*line),
            If { line } => Some(*line),
            Nil { line } => Some(*line),
            Or { line } => Some(*line),
            Print { line } => Some(*line),
            Return { line } => Some(*line),
            Super { line } => Some(*line),
            This { line } => Some(*line),
            True { line } => Some(*line),
            Var { line } => Some(*line),
            While { line } => Some(*line),
            Eof { line } => Some(*line),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match *self {
            LeftParen { .. } => f.write_str("("),
            RightParen { .. } => f.write_str(")"),
            LeftBrace { .. } => f.write_str("{"),
            RightBrace { .. } => f.write_str("}"),
            Comma { .. } => f.write_str(","),
            Dot { .. } => f.write_str("."),
            Minus { .. } => f.write_str("-"),
            Plus { .. } => f.write_str("+"),
            Semicolon { .. } => f.write_str(";"),
            Slash { .. } => f.write_str("/"),
            Star { .. } => f.write_str("*"),
            Bang { .. } => f.write_str("!"),
            BangEqual { .. } => f.write_str("!="),
            Equal { .. } => f.write_str("="),
            EqualEqual { .. } => f.write_str("=="),
            Greater { .. } => f.write_str(">"),
            GreaterEqual { .. } => f.write_str(">="),
            Less { .. } => f.write_str("<"),
            LessEqual { .. } => f.write_str("<="),
            Identifier { ref literal, .. } | r#String { ref literal, .. } => {
                write!(f, "{}", literal)
            }
            Number { literal, .. } => write!(f, "{}", literal),
            And { .. } => f.write_str("and"),
            Class { .. } => f.write_str("class"),
            Else { .. } => f.write_str("else"),
            False { .. } => f.write_str("false"),
            Fun { .. } => f.write_str("fun"),
            For { .. } => f.write_str("for"),
            If { .. } => f.write_str("if"),
            Nil { .. } => f.write_str("nil"),
            Or { .. } => f.write_str("or"),
            Print { .. } => f.write_str("print"),
            Return { .. } => f.write_str("return"),
            Super { .. } => f.write_str("super"),
            This { .. } => f.write_str("this"),
            True { .. } => f.write_str("true"),
            Var { .. } => f.write_str("var"),
            While { .. } => f.write_str("while"),
            Eof { .. } | Comment { .. } | Whitespace { .. } => f.write_str(""),
        }
    }
}
