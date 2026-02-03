use thin_vec::ThinVec;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Map(ThinVec<(Box<str>, Value)>),
    Array(ThinVec<Value>),
    String(String),
    Number(f64),
    Bool(bool),
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Map(map) => {
                // Special case: if the map has a single "root" key, serialize as struct
                if map.len() == 1 && map[0].0.as_ref() == "root" {
                    if let Value::Map(inner_map) = &map[0].1 {
                        let keys: Vec<String> =
                            inner_map.iter().map(|(k, _)| k.to_string()).collect();
                        let mut struct_ser =
                            serializer.serialize_struct("root", inner_map.len())?;
                        for (i, (_k, v)) in inner_map.iter().enumerate() {
                            let key: &'static str = Box::leak(keys[i].clone().into_boxed_str());
                            struct_ser.serialize_field(key, v)?;
                        }
                        struct_ser.end()
                    } else {
                        map[0].1.serialize(serializer)
                    }
                } else {
                    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
                    for (key, value) in map.iter() {
                        map_ser.serialize_entry(key.as_ref(), value)?;
                    }
                    map_ser.end()
                }
            }
            Value::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for value in arr.iter() {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Value::String(s) => serializer.serialize_str(s),
            Value::Number(n) => serializer.serialize_f64(*n),
            Value::Bool(b) => serializer.serialize_bool(*b),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a glass value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bool(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Number(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = ThinVec::new();
                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }
                Ok(Value::Array(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut vec = ThinVec::new();
                while let Some((key, value)) = map.next_entry()? {
                    vec.push((key, value));
                }
                Ok(Value::Map(vec))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
