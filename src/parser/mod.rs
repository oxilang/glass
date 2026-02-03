use thin_vec::{ThinVec, thin_vec};
use thiserror::Error;

use crate::{ast::Value, lexer::Token};

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParseError {
    #[error("unexpected token: `{0}`")]
    UnexpectedToken(Token),
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("invalid root")]
    InvalidRoot,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Result<Value, ParseError> {
        let mut parser = Parser { tokens, pos: 0 };
        parser.parse_root()
    }

    fn parse_root(&mut self) -> Result<Value, ParseError> {
        let key = self.parse_key()?;

        if key.as_ref() != "root" {
            return Err(ParseError::InvalidRoot);
        }

        self.expect(Token::OpenCurly)?;

        let value = self.parse_map()?;

        let map = (key, value);

        self.expect(Token::Comma)?;

        Ok(Value::Map(thin_vec![map]))
    }

    fn parse_map(&mut self) -> Result<Value, ParseError> {
        let mut map = ThinVec::new();

        while *self.peek() != Token::CloseCurly {
            if *self.peek() == Token::Eof {
                return Err(ParseError::UnexpectedEof);
            }
            let key = self.parse_key()?;
            let value = self.parse_value()?;
            map.push((key, value));

            self.expect(Token::Comma)?;
            if *self.peek() == Token::CloseCurly {
                break;
            }
        }

        self.expect(Token::CloseCurly)?;
        Ok(Value::Map(map))
    }

    fn parse_key(&mut self) -> Result<Box<str>, ParseError> {
        let tok = self.peek().clone();
        match tok {
            Token::Key(key) => {
                self.advance();
                Ok(key.into_boxed_str())
            }
            _ => Err(ParseError::UnexpectedToken(self.peek().clone())),
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        let tok = self.peek().clone();
        match tok {
            Token::OpenCurly => {
                self.advance();
                self.parse_map()
            }
            Token::OpenBracket => {
                self.advance();
                self.parse_array()
            }
            Token::String(str) => {
                self.advance();
                Ok(Value::String(str))
            }
            Token::Number(num) => {
                self.advance();
                Ok(Value::Number(num.parse().unwrap()))
            }
            Token::True => {
                self.advance();
                Ok(Value::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Value::Bool(false))
            }
            _ => Err(ParseError::UnexpectedToken(self.peek().clone())),
        }
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        let mut array = ThinVec::new();

        while *self.peek() != Token::CloseBracket {
            if *self.peek() == Token::Eof {
                return Err(ParseError::UnexpectedEof);
            }
            let value = self.parse_value()?;
            array.push(value);

            self.expect(Token::Comma)?;
            if *self.peek() == Token::CloseBracket {
                break;
            }
        }

        self.expect(Token::CloseBracket)?;
        Ok(Value::Array(array))
    }

    fn peek(&mut self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &Token::Eof
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(self.peek().clone()))
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Value, ParseError> {
    Parser::parse(tokens)
}
