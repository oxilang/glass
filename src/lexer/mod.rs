use thiserror::Error;
pub use token::Token;

mod token;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum LexError {
    #[error("unexpected character: `{0}")]
    UnexpectedChar(char),
    #[error("unclosed string")]
    UnclosedString,
    #[error("invalid escape sequence")]
    InvalidEscapeSequence,
}

pub fn tokenize(file_content: String) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = file_content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Tokenize strings
        if c == '"' {
            let mut string_buf = String::new();
            i += 1;
            while i < chars.len() {
                if chars[i] == '"' {
                    break;
                }

                if chars[i] == '\\' {
                    i += 1;
                    if i >= chars.len() {
                        return Err(LexError::UnclosedString);
                    }
                    match chars[i] {
                        '"' => string_buf.push('"'),
                        '\\' => string_buf.push('\\'),
                        'n' => string_buf.push('\n'),
                        't' => string_buf.push('\t'),
                        'r' => string_buf.push('\r'),
                        _ => return Err(LexError::InvalidEscapeSequence),
                    }
                } else {
                    string_buf.push(chars[i]);
                }
                i += 1;
            }
            if i >= chars.len() {
                return Err(LexError::UnclosedString);
            }
            tokens.push(Token::String(string_buf));
            i += 1;
            continue;
        }

        // Tokenize numbers
        if c.is_ascii_digit() || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let mut num_buf = String::new();

            if c == '-' {
                num_buf.push(c);
                i += 1;
            }

            while i < chars.len() && chars[i].is_ascii_digit() {
                num_buf.push(chars[i]);
                i += 1;
            }

            if i < chars.len() && chars[i] == '.' {
                num_buf.push(chars[i]);
                i += 1;

                while i < chars.len() && chars[i].is_ascii_digit() {
                    num_buf.push(chars[i]);
                    i += 1;
                }
            }

            tokens.push(Token::Number(num_buf));
            continue;
        }

        // Tokenize keys
        if c.is_alphabetic() || c == '_' {
            let mut key_buf = String::new();
            key_buf.push(c);
            i += 1;

            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                key_buf.push(chars[i]);
                i += 1;
            }

            match key_buf.as_str() {
                "true" => tokens.push(Token::True),
                "false" => tokens.push(Token::False),
                _ => tokens.push(Token::Key(key_buf)),
            }
            continue;
        }

        // Tokenize simple token
        match c {
            '{' => {
                tokens.push(Token::OpenCurly);
                i += 1;
            }
            '}' => {
                tokens.push(Token::CloseCurly);
                i += 1;
            }
            '[' => {
                tokens.push(Token::OpenBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::CloseBracket);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            _ => return Err(LexError::UnexpectedChar(c)),
        }
    }

    Ok(tokens)
}
