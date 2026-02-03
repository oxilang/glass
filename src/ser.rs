use crate::error::{Error, Result};
use serde::ser::{self, Serialize};

pub struct Serializer {
    output: String,
    current_indent: usize,
    indent_size: usize,
    is_top_level: bool,
    has_root_wrapper: bool,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
        current_indent: 0,
        indent_size: 4,
        is_top_level: true,
        has_root_wrapper: false,
    };

    value.serialize(&mut serializer)?;

    Ok(serializer.output)
}

impl Serializer {
    fn write_indent(&mut self) {
        for _ in 0..self.current_indent * self.indent_size {
            self.output.push(' ');
        }
    }
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output.push('"');
        for c in v.chars() {
            match c {
                '"' => self.output.push_str("\\\""),
                '\\' => self.output.push_str("\\\\"),
                '\n' => self.output.push_str("\\n"),
                '\t' => self.output.push_str("\\t"),
                '\r' => self.output.push_str("\\r"),
                _ => self.output.push(c),
            }
        }
        self.output.push('"');
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::Serde("byte arrays not supported".to_owned()))
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::Serde("null values not supported".to_owned()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::Serde("unit values not supported".to_owned()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::Serde("unit structs not supported".to_owned()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.output.push_str(variant);
        self.output.push(' ');
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output.push_str("[\n");
        self.current_indent += 1;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Serde("tuple variants not supported".to_owned()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output.push_str("{\n");
        self.current_indent += 1;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        if self.is_top_level && !self.has_root_wrapper {
            self.output.push_str("root {\n");
            self.current_indent = 1;
            self.has_root_wrapper = true;
        } else {
            self.output.push_str("{\n");
            self.current_indent += 1;
        }
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Serde("struct variants not supported".to_owned()))
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_indent();
        let was_top_level = self.is_top_level;
        self.is_top_level = false;
        value.serialize(&mut **self)?;
        self.is_top_level = was_top_level;
        self.output.push_str(",\n");
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.current_indent -= 1;
        self.write_indent();
        self.output.push(']');
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Serde("tuple variants not supported".to_owned()))
    }

    fn end(self) -> Result<()> {
        Err(Error::Serde("tuple variants not supported".to_owned()))
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_indent();
        match key.serialize(MapKeySerializer)? {
            MapKey::String(s) => self.output.push_str(&s),
        }
        self.output.push(' ');
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let was_top_level = self.is_top_level;
        self.is_top_level = false;
        value.serialize(&mut **self)?;
        self.is_top_level = was_top_level;
        self.output.push_str(",\n");
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.current_indent -= 1;
        self.write_indent();
        self.output.push('}');
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_indent();
        self.output.push_str(key);
        self.output.push(' ');
        let was_top_level = self.is_top_level;
        self.is_top_level = false;
        value.serialize(&mut **self)?;
        self.is_top_level = was_top_level;
        self.output.push_str(",\n");
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.current_indent -= 1;
        self.write_indent();
        self.output.push('}');
        if self.is_top_level {
            self.output.push_str(",\n");
        }
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Serde("struct variants not supported".to_owned()))
    }

    fn end(self) -> Result<()> {
        Err(Error::Serde("struct variants not supported".to_owned()))
    }
}

enum MapKey {
    String(String),
}

struct MapKeySerializer;

impl ser::Serializer for MapKeySerializer {
    type Ok = MapKey;
    type Error = Error;

    type SerializeSeq = ser::Impossible<MapKey, Error>;
    type SerializeTuple = ser::Impossible<MapKey, Error>;
    type SerializeTupleStruct = ser::Impossible<MapKey, Error>;
    type SerializeTupleVariant = ser::Impossible<MapKey, Error>;
    type SerializeMap = ser::Impossible<MapKey, Error>;
    type SerializeStruct = ser::Impossible<MapKey, Error>;
    type SerializeStructVariant = ser::Impossible<MapKey, Error>;

    fn serialize_bool(self, _v: bool) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_i8(self, _v: i8) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_i16(self, _v: i16) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_i32(self, _v: i32) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_i64(self, _v: i64) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_u8(self, _v: u8) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_u16(self, _v: u16) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_u32(self, _v: u32) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_u64(self, _v: u64) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_f32(self, _v: f32) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_f64(self, _v: f64) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_char(self, _v: char) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_str(self, v: &str) -> Result<MapKey> {
        Ok(MapKey::String(v.to_owned()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_none(self) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<MapKey>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_unit(self) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<MapKey> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<MapKey>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<MapKey>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::Serde("map keys must be strings".to_owned()))
    }
}
