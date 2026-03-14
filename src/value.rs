use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::de;
use serde::ser::{Serialize, SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serializer};

use crate::types::{Keyword, Symbol};

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    String(String),
    Char(char),
    Symbol(Symbol),
    Keyword(Keyword),
    Integer(i64),
    Float(f64),
    List(Vec<Value>),
    Vector(Vec<Value>),
    Map(HashMap<Value, Value>),
    Set(Vec<Value>),
    Tagged { tag: Symbol, value: Box<Value> },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Keyword(a), Value::Keyword(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Vector(a), Value::Vector(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::Tagged { tag: ta, value: va }, Value::Tagged { tag: tb, value: vb }) => {
                ta == tb && va == vb
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match crate::to_string(self) {
            Ok(s) => f.write_str(&s),
            Err(_) => write!(f, "Value(/* serialization error */)",),
        }
    }
}

impl Hash for Value {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        match self {
            Value::Nil => 0u8.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::String(s) => s.hash(state),
            Value::Char(c) => c.hash(state),
            Value::Symbol(s) => s.hash(state),
            Value::Keyword(k) => k.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::List(v) => {
                1u8.hash(state);
                for x in v {
                    x.hash(state);
                }
            }
            Value::Vector(v) => {
                2u8.hash(state);
                for x in v {
                    x.hash(state);
                }
            }
            Value::Map(m) => {
                3u8.hash(state);
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Set(v) => {
                4u8.hash(state);
                for x in v {
                    x.hash(state);
                }
            }
            Value::Tagged { tag, value } => {
                5u8.hash(state);
                tag.hash(state);
                value.hash(state);
            }
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Nil => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::String(s) => serializer.serialize_str(s),
            Value::Char(c) => serializer.serialize_char(*c),
            Value::Symbol(s) => s.serialize(serializer),
            Value::Keyword(k) => k.serialize(serializer),
            Value::Integer(i) => serializer.serialize_i64(*i),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::List(v) | Value::Vector(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::Set(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Tagged { tag, value } => {
                let tag_name = if let Some(name) = tag.as_str().strip_prefix(':') {
                    name
                } else {
                    tag.as_str()
                };
                if tag_name == "inst" {
                    if let Value::Integer(ms) = &**value {
                        let rfc3339 = crate::tags::format_inst_ms(*ms);
                        serializer.serialize_str(&format!("#inst \"{}\"", rfc3339))
                    } else {
                        Err(serde::ser::Error::custom("expected Integer for #inst"))
                    }
                } else if tag_name == "uuid" {
                    let s = match &**value {
                        Value::String(s) => s.clone(),
                        _ => return Err(serde::ser::Error::custom("expected string for #uuid")),
                    };
                    serializer.serialize_str(&format!("#uuid \"{}\"", s))
                } else {
                    let s = match &**value {
                        Value::String(v) => format!("#{} \"{}\"", tag_name, v),
                        Value::Integer(v) => format!("#{} {}", tag_name, v),
                        Value::Float(v) => format!("#{} {}", tag_name, v),
                        Value::Bool(v) => format!("#{} {}", tag_name, v),
                        Value::Nil => format!("#{} nil", tag_name),
                        Value::Char(v) => format!("#{} \\{}", tag_name, v),
                        Value::Keyword(v) => format!("#{} {}", tag_name, v),
                        Value::Symbol(v) => format!("#{} {}", tag_name, v),
                        Value::List(_)
                        | Value::Vector(_)
                        | Value::Map(_)
                        | Value::Set(_)
                        | Value::Tagged { .. } => {
                            let value_edn = crate::to_string(&value).map_err(|e| {
                                serde::ser::Error::custom(format!("serialization error: {}", e))
                            })?;
                            format!("#{} {}", tag_name, value_edn)
                        }
                    };
                    serializer.serialize_str(&s)
                }
            }
        }
    }
}

struct SeqAccess {
    elements: std::vec::IntoIter<Value>,
    len: usize,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = crate::Error;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        let element = self
            .elements
            .next()
            .expect("element must exist when len > 0");
        seed.deserialize(element).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct MapAccess {
    entries: Vec<(Value, Value)>,
    current_key: Option<Value>,
}

impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = crate::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.entries.is_empty() {
            return Ok(None);
        }
        let (key, value) = self.entries.remove(0);
        self.current_key = Some(value);
        seed.deserialize(key).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self
            .current_key
            .take()
            .expect("next_value called before next_key");
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.entries.len() + if self.current_key.is_some() { 1 } else { 0 })
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an EDN value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Integer(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Integer(v as i64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Char(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v.starts_with(':') {
                    Ok(Value::Keyword(Keyword(v.to_string())))
                } else if v.starts_with('#') {
                    // Manually parse tagged value format: #tag "value" or #tag Debug(...)
                    let rest = &v[1..];
                    if let Some(space_idx) = rest.find(' ') {
                        let tag_name = &rest[..space_idx];
                        let value_part = rest[space_idx + 1..].trim();
                        if value_part.starts_with('"') && value_part.ends_with('"') {
                            let inner_value = &value_part[1..value_part.len() - 1];
                            return Ok(Value::Tagged {
                                tag: Symbol(format!(":{}", tag_name)),
                                value: Box::new(Value::String(inner_value.to_string())),
                            });
                        } else if value_part.starts_with("Integer(") && value_part.ends_with(')') {
                            let inner = &value_part[8..value_part.len() - 1];
                            if let Ok(i) = inner.parse::<i64>() {
                                return Ok(Value::Tagged {
                                    tag: Symbol(format!(":{}", tag_name)),
                                    value: Box::new(Value::Integer(i)),
                                });
                            }
                        } else if value_part.starts_with("Float(") && value_part.ends_with(')') {
                            let inner = &value_part[6..value_part.len() - 1];
                            if let Ok(f) = inner.parse::<f64>() {
                                return Ok(Value::Tagged {
                                    tag: Symbol(format!(":{}", tag_name)),
                                    value: Box::new(Value::Float(f)),
                                });
                            }
                        } else if value_part.starts_with("Bool(") && value_part.ends_with(')') {
                            let inner = &value_part[5..value_part.len() - 1];
                            if inner == "true" {
                                return Ok(Value::Tagged {
                                    tag: Symbol(format!(":{}", tag_name)),
                                    value: Box::new(Value::Bool(true)),
                                });
                            } else if inner == "false" {
                                return Ok(Value::Tagged {
                                    tag: Symbol(format!(":{}", tag_name)),
                                    value: Box::new(Value::Bool(false)),
                                });
                            }
                        } else if value_part == "Nil" {
                            return Ok(Value::Tagged {
                                tag: Symbol(format!(":{}", tag_name)),
                                value: Box::new(Value::Nil),
                            });
                        } else if value_part.starts_with("Char(") && value_part.ends_with(')') {
                            let inner = &value_part[5..value_part.len() - 1];
                            if let Ok(c) = inner.parse::<char>() {
                                return Ok(Value::Tagged {
                                    tag: Symbol(format!(":{}", tag_name)),
                                    value: Box::new(Value::Char(c)),
                                });
                            }
                        } else if value_part.starts_with("Keyword(") && value_part.ends_with(')') {
                            let inner = &value_part[8..value_part.len() - 1];
                            return Ok(Value::Tagged {
                                tag: Symbol(format!(":{}", tag_name)),
                                value: Box::new(Value::Keyword(Keyword(inner.to_string()))),
                            });
                        } else if value_part.starts_with("Symbol(") && value_part.ends_with(')') {
                            let inner = &value_part[7..value_part.len() - 1];
                            return Ok(Value::Tagged {
                                tag: Symbol(format!(":{}", tag_name)),
                                value: Box::new(Value::Symbol(Symbol(inner.to_string()))),
                            });
                        } else {
                            // Try to parse as EDN literal (maps, lists, etc.)
                            match crate::from_str::<Value>(value_part) {
                                Ok(parsed_value) => {
                                    return Ok(Value::Tagged {
                                        tag: Symbol(format!(":{}", tag_name)),
                                        value: Box::new(parsed_value),
                                    })
                                }
                                Err(_) => {
                                    // If parsing fails, treat as error
                                    return Ok(Value::String(v.to_string()));
                                }
                            }
                        }
                    }
                    Ok(Value::String(v.to_string()))
                } else {
                    Ok(Value::String(v.to_string()))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v.starts_with(':') {
                    Ok(Value::Keyword(Keyword(v)))
                } else if v.starts_with('#') {
                    // Use visit_str logic for consistency
                    self.visit_str(&v)
                } else {
                    Ok(Value::String(v))
                }
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Value::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }
                Ok(Value::Vector(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut m = HashMap::new();
                while let Some((k, v)) = map.next_entry()? {
                    m.insert(k, v);
                }
                Ok(Value::Map(m))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = crate::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Integer(i) => visitor.visit_i64(i),
            Value::Float(f) => visitor.visit_f64(f),
            Value::Char(c) => visitor.visit_char(c),
            Value::String(s) => visitor.visit_string(s),
            Value::Symbol(s) => visitor.visit_str(s.as_str()),
            Value::Keyword(k) => visitor.visit_str(k.as_str()),
            Value::List(v) | Value::Vector(v) => {
                let len = v.len();
                visitor.visit_seq(SeqAccess {
                    elements: v.into_iter(),
                    len,
                })
            }
            Value::Map(m) => {
                let entries: Vec<_> = m.into_iter().collect();
                visitor.visit_map(MapAccess {
                    entries,
                    current_key: None,
                })
            }
            Value::Set(v) => {
                let len = v.len();
                visitor.visit_seq(SeqAccess {
                    elements: v.into_iter(),
                    len,
                })
            }
            Value::Tagged { tag, value } => {
                let tag_name = if let Some(name) = tag.as_str().strip_prefix(':') {
                    name
                } else {
                    tag.as_str()
                };
                let inner = &*value;
                let s = match inner {
                    Value::String(s) => format!("#{} \"{}\"", tag_name, s),
                    Value::Integer(v) => format!("#{} {}", tag_name, v),
                    Value::Float(v) => format!("#{} {}", tag_name, v),
                    Value::Bool(v) => format!("#{} {}", tag_name, v),
                    Value::Nil => format!("#{} nil", tag_name),
                    Value::Char(v) => format!("#{} \\{}", tag_name, v),
                    Value::Keyword(v) => format!("#{} {}", tag_name, v),
                    Value::Symbol(v) => format!("#{} {}", tag_name, v.as_str()),
                    Value::List(_)
                    | Value::Vector(_)
                    | Value::Map(_)
                    | Value::Set(_)
                    | Value::Tagged { .. } => {
                        let value_edn = crate::to_string(inner).map_err(|e| {
                            crate::Error::Custom(format!("serialization error: {}", e))
                        })?;
                        format!("#{} {}", tag_name, value_edn)
                    }
                };
                visitor.visit_str(&s)
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Keyword(k) => {
                let s = k.as_str();
                let s = if let Some(stripped) = s.strip_prefix(':') {
                    stripped
                } else {
                    s
                };
                visitor.visit_str(s)
            }
            Value::String(s) => visitor.visit_str(&s),
            Value::Symbol(s) => visitor.visit_str(s.as_str()),
            _ => self.deserialize_any(visitor),
        }
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
