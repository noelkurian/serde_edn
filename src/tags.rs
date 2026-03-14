use crate::types::Symbol;
use crate::{Error, Value};
use std::collections::HashMap;

/// Handler for custom tagged literals
pub type TagHandler = fn(&Value) -> Result<Value, Error>;

/// Registry for custom tagged literal handlers
pub struct TagRegistry {
    handlers: HashMap<String, TagHandler>,
}

impl TagRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };
        registry.register_builtin_handlers();
        registry
    }

    pub fn register(&mut self, tag: &str, handler: TagHandler) {
        self.handlers.insert(tag.to_string(), handler);
    }

    pub fn get(&self, tag: &str) -> Option<TagHandler> {
        self.handlers.get(tag).copied()
    }

    fn register_builtin_handlers(&mut self) {
        self.register("inst", handle_inst);
        self.register("uuid", Self::handle_uuid);
    }

    /// Format i64 milliseconds to RFC-3339 UTC string
    pub fn format_inst_ms(ms: i64) -> String {
        crate::tags::format_inst_ms(ms)
    }

    /// UUID: validate format and return string value
    fn handle_uuid(value: &Value) -> Result<Value, Error> {
        match value {
            Value::String(s) => {
                uuid::Uuid::parse_str(s)
                    .map_err(|e| Error::Custom(format!("Invalid #uuid: {}", e)))?;
                Ok(Value::String(s.clone()))
            }
            _ => Err(Error::Custom("#uuid requires string".into())),
        }
    }
}

/// Format i64 milliseconds to RFC-3339 UTC string
pub fn format_inst_ms(ms: i64) -> String {
    use chrono::{DateTime, SecondsFormat, Utc};

    let default_dt = DateTime::from_timestamp(0, 0).expect("epoch timestamp is always valid");
    let dt = if let Some(dt) = DateTime::from_timestamp_millis(ms) {
        dt
    } else {
        default_dt
    }
    .with_timezone(&Utc);
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub(crate) fn handle_inst(value: &Value) -> Result<Value, Error> {
    use chrono::{DateTime, Utc};

    match value {
        Value::String(s) => {
            let dt: DateTime<Utc> = DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| Error::Custom(format!("Invalid #inst: {}", e)))?;
            let ms = dt.timestamp_millis();
            Ok(Value::Tagged {
                tag: Symbol(":inst".to_string()),
                value: Box::new(Value::Integer(ms)),
            })
        }
        _ => Err(Error::Custom("#inst requires RFC-3339 string".into())),
    }
}

impl Default for TagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a tagged value, returning the deserialized result
pub fn handle_tagged_value(tag: &str, value: &Value) -> Result<Value, Error> {
    let registry = TagRegistry::default();
    if let Some(handler) = registry.get(tag) {
        handler(value)
    } else {
        // No handler found, keep as tagged value
        Ok(Value::Tagged {
            tag: Symbol(format!(":{}", tag)),
            value: Box::new(value.clone()),
        })
    }
}
