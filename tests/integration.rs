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
            .prop_filter("non-nan", |n| !n.is_nan())
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
