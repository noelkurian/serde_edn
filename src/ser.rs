use serde::ser;

use crate::error::Error;

pub struct Serializer {
    output: String,
}

impl Serializer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn into_output(self) -> String {
        self.output
    }

    fn write_escaped_string(&mut self, s: &str) {
        self.output.push('"');
        for c in s.chars() {
            match c {
                '"' => self.output.push_str("\\\""),
                '\\' => self.output.push_str("\\\\"),
                '\n' => self.output.push_str("\\n"),
                '\r' => self.output.push_str("\\r"),
                '\t' => self.output.push_str("\\t"),
                _ => self.output.push(c),
            }
        }
        self.output.push('"');
    }

    fn write_edn_char(&mut self, c: char) {
        self.output.push('\\');
        match c {
            '\n' => self.output.push_str("newline"),
            '\r' => self.output.push_str("return"),
            '\t' => self.output.push_str("tab"),
            ' ' => self.output.push_str("space"),
            _ if c as u32 <= 0x7F => self.output.push(c),
            _ => {
                use std::fmt::Write;
                write!(&mut self.output, "u{:04X}", c as u32)
                    .map_err(|_| Error::Custom("format error".to_string()))
                    .unwrap();
            }
        }
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = TupleVariantSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = StructVariantSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        use std::fmt::Write;
        write!(&mut self.output, "{}", v).map_err(|_| Error::Custom("format error".to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let s = if v.is_nan() {
            "##NaN".to_string()
        } else if v.is_infinite() {
            if v.is_sign_positive() {
                "##Inf".to_string()
            } else {
                "##-Inf".to_string()
            }
        } else {
            let buf = format!("{}", v);
            if !buf.contains('.') && !buf.contains('e') && !buf.contains('E') {
                format!("{}.0", buf)
            } else {
                buf
            }
        };
        self.output.push_str(&s);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.write_edn_char(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if v.starts_with('#') {
            self.output.push_str(v);
        } else if v.starts_with(':') && !v.starts_with("::") {
            self.output.push_str(v);
        } else {
            self.write_escaped_string(v);
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for &b in v {
            seq.serialize_element(&b)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output.push_str("nil");
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(name);
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.output.push_str(name);
        self.output.push('/');
        self.output.push_str(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.output.push_str(name);
        self.output.push(' ');
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.output.push_str(name);
        self.output.push('/');
        self.output.push_str(variant);
        self.output.push(' ');
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output.push('[');
        Ok(SeqSerializer::new(self, ']'))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.output.push_str(name);
        self.output.push(' ');
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.output.push('(');
        self.output.push_str(name);
        self.output.push('/');
        self.output.push_str(variant);
        self.output.push(' ');
        Ok(TupleVariantSerializer {
            serializer: self,
            first: true,
            end: ')',
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.output.push('{');
        Ok(MapSerializer::new(self, '}'))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.output.push_str(name);
        self.output.push(' ');
        self.output.push('{');
        Ok(StructSerializer {
            serializer: self,
            first: true,
            end: '}',
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.output.push_str(name);
        self.output.push('/');
        self.output.push_str(variant);
        self.output.push(' ');
        self.output.push('{');
        Ok(StructVariantSerializer {
            serializer: self,
            first: true,
            end: '}',
        })
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

pub struct SeqSerializer<'a> {
    serializer: &'a mut Serializer,
    first: bool,
    end: char,
}

impl<'a> SeqSerializer<'a> {
    fn new(serializer: &'a mut Serializer, end: char) -> Self {
        Self {
            serializer,
            first: true,
            end,
        }
    }
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}

pub struct TupleVariantSerializer<'a> {
    serializer: &'a mut Serializer,
    first: bool,
    end: char,
}

impl<'a> ser::SerializeTupleVariant for TupleVariantSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}

pub struct MapSerializer<'a> {
    serializer: &'a mut Serializer,
    first: bool,
    end: char,
}

impl<'a> MapSerializer<'a> {
    fn new(serializer: &'a mut Serializer, end: char) -> Self {
        Self {
            serializer,
            first: true,
            end,
        }
    }
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;
        key.serialize(&mut *self.serializer)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.serializer.output.push(' ');
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}

pub struct StructSerializer<'a> {
    serializer: &'a mut Serializer,
    first: bool,
    end: char,
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;

        self.serializer.output.push(':');
        self.serializer.output.push_str(key);
        self.serializer.output.push(' ');
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}

pub struct StructVariantSerializer<'a> {
    serializer: &'a mut Serializer,
    first: bool,
    end: char,
}

impl<'a> ser::SerializeStructVariant for StructVariantSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if !self.first {
            self.serializer.output.push(' ');
        }
        self.first = false;

        self.serializer.output.push(':');
        self.serializer.output.push_str(key);
        self.serializer.output.push(' ');
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<(), Self::Error> {
        self.serializer.output.push(self.end);
        Ok(())
    }
}
