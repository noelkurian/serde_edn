mod de;
mod error;
mod ser;
mod tags;
mod types;
mod value;

pub use error::Error;
pub use tags::{format_inst_ms, handle_tagged_value, TagHandler, TagRegistry};
pub use types::{Keyword, Symbol};
pub use value::Value;

pub fn from_str<'de, T>(s: &'de str) -> Result<T, Error>
where
    T: serde::Deserialize<'de>,
{
    let deserializer = de::EdnDeserializer::from_str(s);
    T::deserialize(deserializer)
}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: serde::Serialize,
{
    let mut serializer = ser::Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.into_output())
}
