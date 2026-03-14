# serde_edn

EDN (Extensible Data Notation) serialization and deserialization for Rust, built on [serde](https://serde.rs/).

## Overview

This library provides a complete implementation of [EDN](https://github.com/edn-format/edn), the data format used by Clojure and ClojureScript. EDN is similar to JSON but offers richer data types including symbols, keywords, sets, tagged literals, and built-in support for dates and UUIDs.

### Key Features

- **Full EDN Parsing**: Complete support for all EDN data types and syntax
- **Serde Integration**: Works seamlessly with existing Rust serialization ecosystem
- **Bidirectional Serialization**: Serialize and deserialize between Rust types and EDN
- **Dynamic Values**: Use `Value` enum for flexible, type-safe document manipulation
- **Tagged Literals**: Built-in support for `#inst` (datetime) and `#uuid` tags
- **Custom Tags**: Extensible tag handler registry for custom tagged literals
- **Round-Trip Safe**: Documents serialize and deserialize without data loss
- **Character Escaping**: Full support for EDN character literals including Unicode
- **Namespace Support**: Handles namespaced symbols and keywords

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
serde_edn = { git = "https://github.com/noelkurian/serde_edn", branch = "main" }
```

## Quick Start

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Person {
    name: String,
    age: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Deserialize EDN to Rust struct
    let edn = r#"{:name "Fred", :age 30}"#;
    let person: Person = serde_edn::from_str(edn)?;
    println!("Deserialized: {:?}", person);

    // Serialize Rust struct to EDN
    let person2 = Person {
        name: "Alice".to_string(),
        age: 25,
    };
    let edn2 = serde_edn::to_string(&person2)?;
    println!("Serialized: {}", edn2);

    Ok(())
}
```

## Supported Data Types

| EDN Type | Rust Type | Example |
|----------|-----------|---------|
| nil | `()`, `Option::None` | `nil` |
| boolean | `bool` | `true`, `false` |
| string | `String`, `&str` | `"hello"` |
| character | `char` | `\a`, `\newline`, `\u0041` |
| symbol | `Symbol` | `foo`, `name/space` |
| keyword | `Keyword` | `:foo`, `:name/space` |
| integer | `i64`, `i32`, `i16`, `i8` | `42`, `-10`, `+100N` |
| float | `f64`, `f32` | `3.14`, `1.5e-3`, `##NaN` |
| list | `Vec<T>` | `(1 2 3)` |
| vector | `Vec<T>` | `[1 2 3]` |
| map | `HashMap<K, V>` | `{:key "value"}` |
| set | `HashSet<T>` | `#{1 2 3}` |
| tagged literal | varies | `#inst "..."`, `#uuid "..."` |

## Detailed Usage

### Basic Types

```rust
use serde_edn::{from_str, to_string};

// Strings
let s: String = from_str(r#""hello world""#).expect("failed to parse string");
assert_eq!(s, "hello world");

// Numbers
let n: i64 = from_str("42").expect("failed to parse integer");
let f: f64 = from_str("3.14").expect("failed to parse float");
let exp: f64 = from_str("1.5e-3").expect("failed to parse scientific notation");

// Booleans
let b: bool = from_str("true").expect("failed to parse boolean");

// Characters (supports named chars and unicode)
let c: char = from_str(r#"\a"#).expect("failed to parse character");
let newline: char = from_str(r#"\newline"#).expect("failed to parse newline");
let unicode: char = from_str(r#"\u03B1"#).expect("failed to parse unicode"); // Greek alpha
```

### Collections

```rust
use std::collections::HashMap;

// Vectors
let vec: Vec<i32> = from_str("[1 2 3]").expect("failed to parse vector");
assert_eq!(vec, vec![1, 2, 3]);

// Lists (deserialized as vectors)
let list: Vec<String> = from_str(r#"("a" "b" "c")"#).expect("failed to parse list");

// Maps
let map: HashMap<String, i32> = from_str(r#"{"a" 1 "b" 2}"#).expect("failed to parse map");
assert_eq!(map.get("a"), Some(&1));

// Keyword maps (idiomatic EDN)
let map: HashMap<String, i32> = from_str(r#"{:a 1 :b 2}"#).expect("failed to parse keyword map");
```

### Symbols and Keywords

```rust
use serde_edn::{Keyword, Symbol};

// Keywords (must use Value type for keywords)
use serde_edn::Value;

let value: Value = from_str(":foo").expect("failed to parse keyword");
assert!(matches!(value, Value::Keyword(_)));

// Symbols
let value: Value = from_str("foo").expect("failed to parse symbol");
assert!(matches!(value, Value::Symbol(_)));

// Namespaced
let ns_keyword: Value = from_str(":namespace/name").expect("failed to parse namespaced keyword");
let ns_symbol: Value = from_str("namespace/name").expect("failed to parse namespaced symbol");
```

### Tagged Literals

The library includes built-in support for the most common EDN tagged literals.

#### Instant (Datetime)

```rust
use serde_edn::Value;

// Parse an RFC-3339 datetime
let value: Value = from_str(r#"#inst "1985-04-12T23:20:50.52Z""#).expect("failed to parse datetime");

// The #inst tag is converted to milliseconds since epoch
if let Value::Tagged { tag, value } = value {
    assert_eq!(tag.as_str(), ":inst");
    // The inner value is an Integer containing milliseconds
}
```

#### UUID

```rust
use serde_edn::Value;

// Parse a UUID (validates format automatically)
let value: Value = from_str(r#"#uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6""#).expect("failed to parse UUID");

if let Value::String(uuid) = value {
    assert_eq!(uuid, "f81d4fae-7dec-11d0-a765-00a0c91e6bf6");
}

// Invalid UUIDs will return an error
let result: Result<Value, _> = from_str(r#"#uuid "not-a-uuid""#);
assert!(result.is_err());
```

#### Custom Tags

Tags without built-in handlers are preserved as tagged values:

```rust
use serde_edn::Value;

// Custom application-specific tag
let value: Value = from_str(r#"#myapp/custom "custom value""#).expect("failed to parse custom tag");

if let Value::Tagged { tag, value } = value {
    assert_eq!(tag.as_str(), ":myapp/custom");
    // value contains whatever was tagged
}
```

### Working with Dynamic Values

The `Value` enum provides a type-safe way to work with arbitrary EDN documents:

```rust
use serde_edn::{from_str, to_string, Value};
use std::collections::HashMap;

// Parse any EDN document
let value: Value = from_str(r#"{
  :name "test",
  :items [1 2 3],
  :nested {:key "value"}
}"#).expect("failed to parse document");

// Match on value types
match &value {
    Value::Map(map) => {
        if let Some(Value::String(name)) = map.get(&Value::Keyword(Keyword::new(":name"))) {
            println!("Name: {}", name);
        }
    }
    _ => {}
}

// Serialize back to EDN
let output = to_string(&value).expect("failed to serialize document");
```

### Custom Structs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    database_url: String,
    max_connections: u32,
    cache_ttl: i64,
    feature_flags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

// Deserialize from EDN
let edn = r#"{
  :database-url "postgres://localhost/db",
  :max-connections 100,
  :cache-ttl 3600,
  :feature-flags ["alpha" "beta" "gamma"],
  :location {:x 10.5 :y 20.3}
}"#;

let config: Config = from_str(edn).expect("failed to parse config");

// Serialize to EDN
let output = to_string(&config).expect("failed to serialize config");
```

### Enum Handling

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending(String),
}

let status: Status = from_str(r#"Status/Active"#).expect("failed to parse status");
assert_eq!(status, Status::Active);

let pending: Status = from_str(r#"Status/Pending "waiting for resource""#).expect("failed to parse pending status");
```

## Character Literals

EDN supports various character literal formats:

```rust
use serde_edn::from_str;

// Regular characters
assert_eq!(from_str::<char>(r#"\a"#).expect("failed to parse character"), 'a');

// Named characters
assert_eq!(from_str::<char>(r#"\newline"#).expect("failed to parse newline"), '\n');
assert_eq!(from_str::<char>(r#"\return"#).expect("failed to parse return"), '\r');
assert_eq!(from_str::<char>(r#"\tab"#).expect("failed to parse tab"), '\t');
assert_eq!(from_str::<char>(r#"\space"#).expect("failed to parse space"), ' ');

// Unicode escape sequences
assert_eq!(from_str::<char>(r#"\u0041"#).expect("failed to parse unicode"), 'A');
assert_eq!(from_str::<char>(r#"\u03B1"#).expect("failed to parse unicode"), 'α'); // Greek alpha
assert_eq!(from_str::<char>(r#"\u2665"#).expect("failed to parse unicode"), '♥'); // Heart symbol
```

## Number Formats

```rust
use serde_edn::from_str;

// Integers
assert_eq!(from_str::<i64>("42").expect("failed to parse integer"), 42);
assert_eq!(from_str::<i64>("-100").expect("failed to parse integer"), -100);
assert_eq!(from_str::<i64>("+50").expect("failed to parse integer"), 50);

// Big numbers (N suffix)
assert_eq!(from_str::<i64>("42N").expect("failed to parse integer"), 42);

// Floats
assert_eq!(from_str::<f64>("3.14").expect("failed to parse float"), 3.14);
assert_eq!(from_str::<f64>("-1.5").expect("failed to parse float"), -1.5);

// Scientific notation
assert_eq!(from_str::<f64>("1e10").expect("failed to parse float"), 1e10);
assert_eq!(from_str::<f64>("1.5e-3").expect("failed to parse float"), 0.0015);
assert_eq!(from_str::<f64>("1E+5").expect("failed to parse float"), 1e5);
assert_eq!(from_str::<f64>("2.5e2").expect("failed to parse float"), 250.0);

// Special float values
let nan: f64 = from_str("##NaN").expect("failed to parse NaN");
assert!(nan.is_nan());
```

## Comments and Whitespace

EDN supports comments and relaxed whitespace:

```rust
use serde_edn::from_str;

// Line comments start with semicolon
let edn = r#"
  {
    ; This is a comment
    :key1 "value1",  ; trailing comma is ignored
    :key2 "value2"
  }
"#;

let map: HashMap<String, String> = from_str(edn).expect("failed to parse EDN with comments");
```

## Advanced Features

### Custom Tag Handlers

You can register custom handlers for tagged literals:

```rust
use serde_edn::{TagRegistry, Value, Error, Symbol};

// Define a custom tag handler
fn handle_custom_tag(value: &Value) -> Result<Value, Error> {
    match value {
        Value::String(s) => {
            // Process the string value
            Ok(Value::String(format!("processed: {}", s)))
        }
        _ => Err(Error::Custom("expected string value".to_string())),
    }
}

// Create a registry and register the handler
let mut registry = TagRegistry::new();
registry.register("myapp/custom", handle_custom_tag);
```

### Round-Trip Serialization

The library ensures data integrity through round-trip serialization:

```rust
use serde_edn::{from_str, to_string, Value};

// Original EDN
let original = r#"{
  :name "test",
  :numbers [1 2 3],
  :nested {:key "value"}
}"#;

// Parse to Value
let v1: Value = from_str(original).expect("failed to parse original EDN");

// Serialize back to EDN
let serialized = to_string(&v1).expect("failed to serialize EDN");

// Parse again
let v2: Value = from_str(&serialized).expect("failed to parse serialized EDN");

// Values are equivalent
assert_eq!(v1, v2);
```

## API Reference

### Main Functions

- `from_str<'de, T>(s: &'de str) -> Result<T, Error>` - Deserialize EDN string to type T
- `to_string<T>(value: &T) -> Result<String, Error>` - Serialize value T to EDN string

### Types

- `Value` - Enum representing all possible EDN values
- `Keyword` - EDN keyword type (e.g., `:foo`)
- `Symbol` - EDN symbol type (e.g., `foo`)
- `Error` - Error type for parsing and serialization operations
- `ParseError` - Detailed parse error with line/column information
- `TagRegistry` - Registry for custom tagged literal handlers

### Type Conversions

All types implement `serde::Serialize` and `serde::Deserialize`, allowing seamless integration with the serde ecosystem.

## Error Handling

```rust
use serde_edn::from_str;

// Parse errors include location information
let result: Result<serde_edn::Value, _> = from_str(r#"{:key invalid}"#);

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(e) => {
        if let serde_edn::Error::Parse(pe) = e {
            eprintln!("Parse error at {}:{}: {}", pe.line, pe.column, pe.message);
        }
    }
}
```

## Migration from Other EDN Libraries

If you're coming from other EDN libraries in other languages:

- **Clojure**: The data model matches Clojure's EDN reader/writer
- **Python (edn_format)**: Similar API, with stronger type safety
- **JavaScript (cljs-bean)**: More ergonomic with Rust's type system

See the `examples/` directory for more complete usage examples.

## Testing

Run the test suite:

```bash
cargo test
```

Run examples:

```bash
cargo run --example test
```

## License

MIT

## EDN Specification

This library follows the [official EDN specification](https://github.com/edn-format/edn) with support for:

- All scalar types (nil, boolean, string, character, symbol, keyword, integer, float)
- All collection types (list, vector, map, set)
- Tagged literals (with extensible tag handler system)
- Comments (line comments starting with `;`)
- Whitespace flexibility (commas treated as whitespace)
