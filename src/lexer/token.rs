#[derive(Debug)]
pub enum Token {
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Comma,

    Key(String),
    Number(String),
    String(String),
}
