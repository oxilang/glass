use crate::ast::Value;
use crate::error::{Error, Result};
use crate::lexer::tokenize;
use crate::parser::parse;
use serde::Deserializer;
use serde::de::{self, DeserializeOwned, IntoDeserializer, MapAccess, SeqAccess, Visitor};

pub fn from_str<T>(s: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let tokens = tokenize(s.to_owned())?;
    let value = parse(tokens)?;

    T::deserialize(value)
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_string(v),
            Value::Array(v) => visitor.visit_seq(ValueSeq::new(v)),
            Value::Map(v) => visitor.visit_map(ValueMap::new(v)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(Error::Serde(format!("expected bool, got {:?}", self))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(v) => visitor.visit_i64(v as i64),
            _ => Err(Error::Serde(format!("expected number, got {:?}", self))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(v) => visitor.visit_u64(v as u64),
            _ => Err(Error::Serde(format!("expected number, got {:?}", self))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Number(v) => visitor.visit_f64(v),
            _ => Err(Error::Serde(format!("expected number, got {:?}", self))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(s) => {
                let mut chars = s.chars();
                if let Some(c) = chars.next()
                    && chars.next().is_none()
                {
                    return visitor.visit_char(c);
                }

                Err(Error::Serde(format!("expected single char, got {}", s)))
            }
            _ => Err(Error::Serde(format!("expected string, got {:?}", self))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::String(v) => visitor.visit_string(v),
            _ => Err(Error::Serde(format!("expected string, got {:?}", self))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Serde("byte arrays not supported".to_owned()))
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Serde("byte buffers not supported".to_owned()))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Glass doesn't have null, so always visit Some
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Serde("unit values not supported".to_owned()))
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Serde("unit structs not supported".to_owned()))
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Array(v) => visitor.visit_seq(ValueSeq::new(v)),
            _ => Err(Error::Serde(format!("expected array, got {:?}", self))),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Map(map) if map.len() == 1 && map[0].0.as_ref() == "root" => {
                map[0].1.clone().deserialize_map(visitor)
            }
            Value::Map(v) => visitor.visit_map(ValueMap::new(v)),
            _ => Err(Error::Serde(format!("expected map, got {:?}", self))),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self {
            Value::Map(map) if map.len() == 1 && map[0].0.as_ref() == "root" => {
                map[0].1.clone().deserialize_map(visitor)
            }
            _ => self.deserialize_map(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Map(entries) => {
                if entries.len() == 1 {
                    let (key, value) = entries.into_iter().next().unwrap();
                    visitor.visit_enum(EnumAccessor { name: key, value })
                } else {
                    Err(Error::Serde("expected single key map for enum".to_owned()))
                }
            }
            Value::String(s) => visitor.visit_enum(s.as_str().into_deserializer()),
            _ => Err(Error::Serde(format!(
                "expected map or string for enum, got {:?}",
                self
            ))),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ValueSeq {
    values: thin_vec::ThinVec<Value>,
    index: usize,
}

impl ValueSeq {
    fn new(values: thin_vec::ThinVec<Value>) -> Self {
        Self { values, index: 0 }
    }
}

impl<'de> SeqAccess<'de> for ValueSeq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.index < self.values.len() {
            let value = self.values[self.index].clone();
            self.index += 1;
            seed.deserialize(value).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct ValueMap {
    entries: thin_vec::ThinVec<(Box<str>, Value)>,
    index: usize,
    current_value: Option<Value>,
}

impl ValueMap {
    fn new(entries: thin_vec::ThinVec<(Box<str>, Value)>) -> Self {
        Self {
            entries,
            index: 0,
            current_value: None,
        }
    }
}

impl<'de> MapAccess<'de> for ValueMap {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.index < self.entries.len() {
            let (key, value) = &self.entries[self.index];
            self.current_value = Some(value.clone());
            seed.deserialize(key.as_ref().into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(value) = self.current_value.take() {
            self.index += 1;
            seed.deserialize(value)
        } else {
            Err(Error::Serde("no more values in map".to_owned()))
        }
    }
}

struct EnumAccessor {
    name: Box<str>,
    value: Value,
}

impl<'de> de::EnumAccess<'de> for EnumAccessor {
    type Error = Error;
    type Variant = ValueVariant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant_value = Value::String(self.name.to_string());
        let variant = seed.deserialize(variant_value)?;
        Ok((variant, ValueVariant { value: self.value }))
    }
}

struct ValueVariant {
    value: Value,
}

impl<'de> de::VariantAccess<'de> for ValueVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::Serde("unit variants not supported".to_owned()))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.value.deserialize_seq(visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.value.deserialize_map(visitor)
    }
}
