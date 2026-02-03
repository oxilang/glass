use glass::{Value, from_str, to_string};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use thin_vec::{ThinVec, thin_vec};

#[test]
fn parses_simple_example() {
    let input = r#"
        root {
            hello "world",
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![(
            "hello".into(),
            Value::String("world".to_string()),
        ),])
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

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![
            ("hello".into(), Value::String("world".to_string()),),
            (
                "nested".into(),
                Value::Map(thin_vec![(
                    "hello".into(),
                    Value::String("world".to_string()),
                ),])
            ),
        ])
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

    let ast: Value = from_str(input).unwrap();

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
                ),])
            ),
        ])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn parses_booleans() {
    let input = r#"
        root {
            is_true true,
            is_false false,
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![
            ("is_true".into(), Value::Bool(true)),
            ("is_false".into(), Value::Bool(false)),
        ])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn parses_numbers() {
    let input = r#"
        root {
            integer 123,
            negative -456,
            float 12.34,
            negative_float -56.78,
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![
            ("integer".into(), Value::Number(123.0)),
            ("negative".into(), Value::Number(-456.0)),
            ("float".into(), Value::Number(12.34)),
            ("negative_float".into(), Value::Number(-56.78)),
        ])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn parses_escaped_strings() {
    let input = r#"
        root {
            escaped "line1\nline2",
            quoted "\"quoted\"",
            backslash "\\",
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![
            ("escaped".into(), Value::String("line1\nline2".to_string())),
            ("quoted".into(), Value::String("\"quoted\"".to_string())),
            ("backslash".into(), Value::String("\\".to_string())),
        ])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn parses_hex_escape_sequences() {
    let input = r#"
        root {
            hex_lower "\x41\x42\x43",
            hex_upper "\x7a\x78\x79",
            hex_mixed "A\x30\x31\x32Z",
            hex_nul "\x00",
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![
            ("hex_lower".into(), Value::String("ABC".to_string())),
            ("hex_upper".into(), Value::String("zxy".to_string())),
            ("hex_mixed".into(), Value::String("A012Z".to_string())),
            ("hex_nul".into(), Value::String("\0".to_string())),
        ])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn hex_escape_mixed_with_other_escapes() {
    let input = r#"
        root {
            mixed "hello\x41world\n",
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![(
            "mixed".into(),
            Value::String("helloAworld\n".to_string())
        ),])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn invalid_hex_escape_missing_chars() {
    let input = r#"
        root {
            bad "\x",
        },
    "#;

    let result: Result<Value, _> = from_str(input);
    assert!(result.is_err());
}

#[test]
fn invalid_hex_escape_single_char() {
    let input = r#"
        root {
            bad "\x4",
        },
    "#;

    let result: Result<Value, _> = from_str(input);
    assert!(result.is_err());
}

#[test]
fn invalid_hex_escape_non_hex_chars() {
    let input = r#"
        root {
            bad "\xGG",
        },
    "#;

    let result: Result<Value, _> = from_str(input);
    assert!(result.is_err());
}

#[test]
fn invalid_hex_escape_partial_hex() {
    let input = r#"
        root {
            bad "\x1G",
        },
    "#;

    let result: Result<Value, _> = from_str(input);
    assert!(result.is_err());
}

#[test]
fn parses_empty_collections() {
    let input = r#"
        root {
            empty_map {},
            empty_array [],
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![
            ("empty_map".into(), Value::Map(thin_vec![])),
            ("empty_array".into(), Value::Array(thin_vec![])),
        ])
    )]);

    assert_eq!(ast, expected);
}

#[test]
fn parses_mixed_array() {
    let input = r#"
        root {
            mixed [1, "two", true, [], {},],
        },
    "#;

    let ast: Value = from_str(input).unwrap();

    let expected = Value::Map(thin_vec![(
        "root".into(),
        Value::Map(thin_vec![(
            "mixed".into(),
            Value::Array(thin_vec![
                Value::Number(1.0),
                Value::String("two".to_string()),
                Value::Bool(true),
                Value::Array(thin_vec![]),
                Value::Map(thin_vec![]),
            ])
        ),])
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
        ])
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

// Property Testing
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]*"
}

fn value_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        any::<String>().prop_map(Value::String),
        any::<f64>()
            .prop_filter("non-nan-non-infinite", |n| !n.is_nan() && n.is_finite())
            .prop_map(Value::Number),
        any::<bool>().prop_map(Value::Bool),
    ];

    leaf.prop_recursive(
        4,  // deep
        64, // max size
        5,  // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..5)
                    .prop_map(|v| Value::Array(ThinVec::from(v))),
                prop::collection::vec((identifier_strategy().prop_map(Into::into), inner), 0..5)
                    .prop_map(|v| Value::Map(ThinVec::from(v))),
            ]
        },
    )
}

fn map_value_strategy() -> impl Strategy<Value = Value> {
    prop::collection::vec(
        (identifier_strategy().prop_map(Into::into), value_strategy()),
        0..5,
    )
    .prop_map(|v| Value::Map(ThinVec::from(v)))
}

proptest! {
    #[test]
    fn test_value_roundtrip_prop(val in map_value_strategy()) {
        let wrapped = Value::Map(thin_vec![("root".into(), val.clone())]);
        let serialized = to_string(&wrapped).unwrap();
        let deserialized: Value = from_str(&serialized).unwrap();
        assert_eq!(wrapped, deserialized);
    }
}
