use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Comma,

    True,
    False,

    Key(String),
    Number(String),
    String(String),

    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::OpenCurly => write!(f, "{{"),
            Token::CloseCurly => write!(f, "}}"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),

            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),

            Token::Key(key) => write!(f, "{key}"),
            Token::Number(num) => write!(f, "{num}"),
            Token::String(str) => write!(f, "\"{str}\""),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}
