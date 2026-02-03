use lexer::LexError;
use parser::ParseError;
use thiserror::Error;

mod ast;
mod lexer;
mod parser;

#[derive(Debug, Error)]
pub enum GlassError {
    #[error("lexer error: {0}")]
    LexError(#[from] LexError),
    #[error("parser error: {0}")]
    ParseError(#[from] ParseError),
}

pub type GlassResult<T, E = GlassError> = Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Value;
    use thin_vec::thin_vec;

    #[test]
    fn parses_simple_example() {
        let input = r#"
            root {
                hello "world",
            },
        "#;

        let tokens = lexer::tokenize(input.to_string()).unwrap();
        let ast = parser::parse(tokens).unwrap();

        let expected = Value::Map(thin_vec![(
            "root".into(),
            Value::Map(thin_vec![(
                "hello".into(),
                Value::String("world".to_string()),
            ),]),
        )]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn parses_nested_example() {
        let input = r#"
            root {
                hello "world",
                nested {
                    hello "world",
                },
            },
        "#;

        let tokens = lexer::tokenize(input.to_string()).unwrap();
        let ast = parser::parse(tokens).unwrap();

        let expected = Value::Map(thin_vec![(
            "root".into(),
            Value::Map(thin_vec![
                ("hello".into(), Value::String("world".to_string()),),
                (
                    "nested".into(),
                    Value::Map(thin_vec![(
                        "hello".into(),
                        Value::String("world".to_string()),
                    ),]),
                ),
            ]),
        )]);

        assert_eq!(ast, expected);
    }

    #[test]
    fn parses_array_example() {
        let input = r#"
            root {
                hello ["world", "foo",],
                nested {
                    hello ["world", "foo",],
                },
            },
        "#;

        let tokens = lexer::tokenize(input.to_string()).unwrap();
        let ast = parser::parse(tokens).unwrap();

        let expected = Value::Map(thin_vec![(
            "root".into(),
            Value::Map(thin_vec![
                (
                    "hello".into(),
                    Value::Array(thin_vec![
                        Value::String("world".to_string()),
                        Value::String("foo".to_string()),
                    ]),
                ),
                (
                    "nested".into(),
                    Value::Map(thin_vec![(
                        "hello".into(),
                        Value::Array(thin_vec![
                            Value::String("world".to_string()),
                            Value::String("foo".to_string()),
                        ]),
                    ),]),
                ),
            ]),
        )]);

        assert_eq!(ast, expected);
    }
}
