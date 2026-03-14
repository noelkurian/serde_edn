use std::collections::HashMap;

use serde::de;
use serde::de::Visitor;

use crate::error::{Error, ParseError};
use crate::tags::handle_tagged_value;
use crate::types::{Keyword, Symbol};
use crate::Value;

pub struct EdnDeserializer<'de> {
    #[allow(dead_code)]
    input: &'de str,
    chars: std::iter::Peekable<std::str::Chars<'de>>,
    line: usize,
    column: usize,
}

impl<'de> EdnDeserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn error(&self, message: impl Into<String>) -> Error {
        Error::Parse(ParseError::new(message, self.line, self.column))
    }

    fn next(&mut self) -> Option<char> {
        match self.chars.next() {
            Some('\n') => {
                self.line += 1;
                self.column = 1;
                Some('\n')
            }
            Some(c) => {
                self.column += 1;
                Some(c)
            }
            None => None,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() || c == ',' {
                self.next();
            } else if c == ';' {
                while let Some(&c) = self.chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.next();
                }
            } else {
                break;
            }
        }
    }

    pub fn parse_value(&mut self) -> Result<Value, Error> {
        loop {
            self.skip_whitespace();

            let c = match self.chars.peek() {
                Some(c) => *c,
                None => return Err(Error::Eof),
            };

            let result = match c {
                '(' => self.parse_list(),
                '[' => self.parse_vector(),
                '{' => self.parse_map_or_set(),
                '#' => {
                    self.next();
                    match self.parse_dispatch_value()? {
                        Some(v) => Ok(v),
                        None => continue,
                    }
                }
                ':' => self.parse_keyword(),
                '"' => self.parse_string(),
                '\\' => self.parse_char(),
                '-' | '+' | '0'..='9' => self.parse_number(),
                _ => {
                    if c.is_alphabetic() || "+-.*!_?$%&=<>".contains(c) {
                        self.parse_symbol_or_keyword()
                    } else {
                        Err(self.error(format!("unexpected character: {}", c)))
                    }
                }
            };
            return result;
        }
    }

    fn parse_list(&mut self) -> Result<Value, Error> {
        self.next();
        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            if let Some(&')') = self.chars.peek() {
                self.next();
                break;
            }
            items.push(self.parse_value()?);
        }
        Ok(Value::List(items))
    }

    fn parse_vector(&mut self) -> Result<Value, Error> {
        self.next();
        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            if let Some(&']') = self.chars.peek() {
                self.next();
                break;
            }
            items.push(self.parse_value()?);
        }
        Ok(Value::Vector(items))
    }

    fn parse_map_or_set(&mut self) -> Result<Value, Error> {
        self.next();

        self.skip_whitespace();

        if let Some(&'#') = self.chars.peek() {
            self.next();
            if let Some(&'{') = self.chars.peek() {
                self.next();
                let mut items = Vec::new();
                loop {
                    self.skip_whitespace();
                    if let Some(&'}') = self.chars.peek() {
                        self.next();
                        break;
                    }
                    items.push(self.parse_value()?);
                }
                return Ok(Value::Set(items));
            }
        }

        let mut map = HashMap::new();
        loop {
            self.skip_whitespace();
            if let Some(&'}') = self.chars.peek() {
                self.next();
                break;
            }
            let key = self.parse_value()?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
        }
        Ok(Value::Map(map))
    }

    fn parse_dispatch_value(&mut self) -> Result<Option<Value>, Error> {
        let c = match self.chars.peek() {
            Some(c) => *c,
            None => return Err(self.error("unexpected end after '#'")),
        };

        match c {
            '_' => {
                self.next();
                self.skip_whitespace();
                self.parse_value()?;
                Ok(None)
            }
            '{' => self.parse_set().map(Some),
            c if c.is_alphabetic() => {
                let tag = self.parse_symbol_core()?;
                self.skip_whitespace();
                let value = self.parse_value()?;

                let result = handle_tagged_value(&tag, &value)?;
                Ok(Some(result))
            }
            _ => Err(self.error("unsupported dispatch")),
        }
    }

    fn parse_set(&mut self) -> Result<Value, Error> {
        self.next();
        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            if let Some(&'}') = self.chars.peek() {
                self.next();
                break;
            }
            items.push(self.parse_value()?);
        }
        Ok(Value::Set(items))
    }

    fn parse_keyword(&mut self) -> Result<Value, Error> {
        self.next();
        let name = self.parse_symbol_core()?;
        Ok(Value::Keyword(Keyword(format!(":{}", name))))
    }

    fn parse_string(&mut self) -> Result<Value, Error> {
        self.next();
        let mut s = String::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '"' => {
                    self.next();
                    return Ok(Value::String(s));
                }
                '\\' => {
                    self.next();
                    let c = match self.chars.peek() {
                        Some(&c) => c,
                        None => return Err(self.error("unexpected end in string escape")),
                    };
                    let escaped = match c {
                        't' => '\t',
                        'r' => '\r',
                        'n' => '\n',
                        '\\' => '\\',
                        '"' => '"',
                        _ => return Err(self.error(format!("invalid escape: \\{}", c))),
                    };
                    s.push(escaped);
                    self.next();
                }
                _ => {
                    s.push(c);
                    self.next();
                }
            }
        }

        Err(self.error("unterminated string"))
    }

    fn parse_char(&mut self) -> Result<Value, Error> {
        self.next();
        let c = match self.chars.peek() {
            Some(c) => *c,
            None => return Err(self.error("unexpected end after '\\'")),
        };

        let ch = if c.is_alphabetic() {
            let mut name = String::new();
            while let Some(&c) = self.chars.peek() {
                if c.is_alphabetic() {
                    name.push(c);
                    self.next();
                } else {
                    break;
                }
            }

            match name.as_str() {
                "newline" => '\n',
                "return" => '\r',
                "space" => ' ',
                "tab" => '\t',
                "u" => {
                    let mut hex = String::new();
                    for _ in 0..4 {
                        let c = self
                            .chars
                            .peek()
                            .copied()
                            .ok_or_else(|| self.error("unexpected end in unicode escape"))?;
                        if c.is_ascii_hexdigit() {
                            hex.push(c);
                            self.next();
                        } else {
                            return Err(self.error("expected 4 hex digits after \\u"));
                        }
                    }
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|_| self.error("invalid unicode escape"))?;
                    char::from_u32(code).ok_or_else(|| self.error("invalid unicode codepoint"))?
                }
                _ if name.len() == 1 => name.chars().next().unwrap(),
                _ => return Err(self.error(format!("invalid character literal: \\{}", name))),
            }
        } else {
            self.next();
            c
        };

        Ok(Value::Char(ch))
    }

    fn parse_number(&mut self) -> Result<Value, Error> {
        let mut s = String::new();

        if let Some(&c) = self.chars.peek() {
            if c == '+' || c == '-' {
                s.push(c);
                self.next();
            }
        }

        let mut has_digits = false;
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.next();
                has_digits = true;
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(self.error("expected digit"));
        }

        let mut has_frac = false;
        if let Some(&c) = self.chars.peek() {
            if c == '.' {
                s.push(c);
                self.next();

                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        s.push(c);
                        self.next();
                        has_frac = true;
                    } else {
                        break;
                    }
                }
            }
        }

        let has_exponent = if let Some(c) = self.chars.peek() {
            if *c == 'e' || *c == 'E' {
                s.push(*c);
                self.next();

                if let Some(c) = self.chars.peek() {
                    if *c == '+' || *c == '-' {
                        s.push(*c);
                        self.next();
                    }
                }

                let mut has_exp_digits = false;
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        s.push(c);
                        self.next();
                        has_exp_digits = true;
                    } else {
                        break;
                    }
                }
                if !has_exp_digits {
                    return Err(self.error("expected digits after exponent"));
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        let is_float = has_frac || has_exponent;

        if is_float {
            let f: f64 = s
                .parse()
                .map_err(|_| self.error(format!("invalid float: {}", s)))?;
            Ok(Value::Float(f))
        } else {
            let i: i64 = s
                .parse()
                .map_err(|_| self.error(format!("invalid integer: {}", s)))?;

            if let Some(c) = self.chars.peek() {
                if *c == 'N' || *c == 'M' {
                    self.next();
                }
            }

            Ok(Value::Integer(i))
        }
    }

    fn parse_symbol_or_keyword(&mut self) -> Result<Value, Error> {
        let symbol = self.parse_symbol_core()?;

        if symbol.starts_with(':') {
            Ok(Value::Keyword(Keyword(symbol)))
        } else if symbol == "nil" {
            Ok(Value::Nil)
        } else if symbol == "true" {
            Ok(Value::Bool(true))
        } else if symbol == "false" {
            Ok(Value::Bool(false))
        } else if symbol == "/" {
            Ok(Value::Symbol(Symbol("/".to_string())))
        } else {
            Ok(Value::Symbol(Symbol(symbol)))
        }
    }

    fn parse_symbol_core(&mut self) -> Result<String, Error> {
        let mut s = String::new();

        let c = match self.chars.peek() {
            Some(c) => *c,
            None => return Err(self.error("unexpected end in symbol")),
        };

        if c.is_alphabetic() || "+-.*!_?$%&=<>".contains(c) {
            s.push(c);
            self.next();
        } else {
            return Err(self.error(format!("invalid symbol start: {}", c)));
        }

        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || "+-.*!_?$%&=<>:#".contains(c) {
                s.push(c);
                self.next();
            } else {
                break;
            }
        }

        if let Some(&'/') = self.chars.peek() {
            s.push('/');
            self.next();

            while let Some(&c) = self.chars.peek() {
                if c.is_alphanumeric() || "+-.*!_?$%&=<>:#".contains(c) {
                    s.push(c);
                    self.next();
                } else {
                    break;
                }
            }
        }

        Ok(s)
    }
}

impl<'de> serde::Deserializer<'de> for EdnDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.skip_whitespace();
        let value = self.parse_value()?;

        match value {
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
                let elements: Vec<Value> = v;
                visitor.visit_seq(SeqAccess {
                    elements: elements.into_iter(),
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
                let elements: Vec<Value> = v;
                visitor.visit_seq(SeqAccess {
                    elements: elements.into_iter(),
                    len,
                })
            }
            Value::Tagged { tag, value } => {
                let tag_name = tag.as_str().strip_prefix(':').unwrap_or(tag.as_str());
                let s = match value.as_ref() {
                    Value::String(s) => format!("#{} \"{}\"", tag_name, s),
                    Value::Integer(i) => format!("#{} Integer({})", tag_name, i),
                    Value::Float(f) => format!("#{} Float({})", tag_name, f),
                    Value::Bool(b) => format!("#{} Bool({})", tag_name, b),
                    Value::Nil => format!("#{} Nil", tag_name),
                    Value::Char(c) => format!("#{} Char({:?})", tag_name, c),
                    Value::Keyword(sym) => format!("#{} Keyword({})", tag_name, sym),
                    Value::Symbol(sym) => format!("#{} Symbol({})", tag_name, sym),
                    _ => {
                        let value_edn = crate::to_string(value.as_ref())
                            .map_err(|e| Error::Custom(format!("serialization error: {}", e)))?;
                        format!("#{} {}", tag_name, value_edn)
                    }
                };
                visitor.visit_str(&s)
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct SeqAccess {
    elements: std::vec::IntoIter<Value>,
    len: usize,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        let element = self.elements.next().unwrap();
        // Use the Value itself as the deserializer - it implements Deserialize
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
    type Error = Error;

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
