use std::fmt::Display;

use crate::lexer::LexError;
use crate::parser::ParseError;
use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("serde error: {0}")]
    Serde(String),
    #[error("lexer error: {0}")]
    LexError(#[from] LexError),
    #[error("parser error: {0}")]
    ParseError(#[from] ParseError),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}
