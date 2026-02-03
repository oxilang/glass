mod ast;
mod de;
mod error;
mod lexer;
mod parser;
mod ser;

pub use de::from_str;
pub use error::{Error, Result};
pub use ser::to_string;

#[cfg(test)]
mod tests {
    use crate::{ast::Value, from_str, lexer, parser, to_string};
    use serde::{Deserialize, Serialize};
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        age: f64,
        hobbies: Vec<String>,
    }

    #[test]
    fn test_serde_roundtrip() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30.0,
            hobbies: vec!["reading".to_string(), "coding".to_string()],
        };

        let serialized = to_string(&person).unwrap();

        assert!(serialized.contains("root {"));
        assert!(serialized.contains("name \"Alice\""));
        assert!(serialized.contains("age 30"));
        assert!(serialized.contains("hobbies ["));

        let deserialized: Person = from_str(&serialized).unwrap();
        assert_eq!(person, deserialized);
    }

    #[test]
    fn test_value_roundtrip() {
        let value = Value::Map(thin_vec![(
            "test".into(),
            Value::Array(thin_vec![
                Value::String("hello".to_string()),
                Value::Number(42.0),
            ]),
        )]);

        let wrapped = Value::Map(thin_vec![("root".into(), value.clone())]);

        let serialized = to_string(&wrapped).unwrap();

        let deserialized: Value = from_str(&serialized).unwrap();
        assert_eq!(wrapped, deserialized);
    }

    #[test]
    fn test_simple_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Config {
            name: String,
            version: String,
        }

        let config = Config {
            name: "myapp".to_string(),
            version: "1.0.0".to_string(),
        };

        let serialized = to_string(&config).unwrap();
        let deserialized: Config = from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_nested_struct() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Inner {
            value: f64,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Outer {
            inner: Inner,
            name: String,
        }

        let outer = Outer {
            inner: Inner { value: 42.0 },
            name: "test".to_string(),
        };

        let serialized = to_string(&outer).unwrap();
        let deserialized: Outer = from_str(&serialized).unwrap();
        assert_eq!(outer, deserialized);
    }
}
