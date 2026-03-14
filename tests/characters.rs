mod common;

use common::{assert_round_trip, parse};
use serde_edn::Value;

#[test]
fn test_unicode_escapes() {
    assert_eq!(parse::<Value>(r#"\u0041"#), Value::Char('A'));
    assert_eq!(parse::<Value>(r#"\u03B1"#), Value::Char('α'));
    assert_eq!(parse::<Value>(r#"\u2764"#), Value::Char('❤'));
}

#[test]
fn test_named_chars() {
    assert_eq!(parse::<Value>(r#"\newline"#), Value::Char('\n'));
    assert_eq!(parse::<Value>(r#"\return"#), Value::Char('\r'));
    assert_eq!(parse::<Value>(r#"\tab"#), Value::Char('\t'));
    assert_eq!(parse::<Value>(r#"\space"#), Value::Char(' '));
}

#[test]
fn test_literal_chars() {
    assert_eq!(parse::<Value>(r#"\A"#), Value::Char('A'));
    assert_eq!(parse::<Value>(r#"\b"#), Value::Char('b'));
    assert_eq!(parse::<Value>(r#"\!"#), Value::Char('!'));
}

#[test]
fn test_char_round_trip() {
    assert_round_trip!(r#"\A"#);
    assert_round_trip!(r#"\newline"#);
    assert_round_trip!(r#"\u0041"#);
    assert_round_trip!(r#"\u03B1"#);
}

#[test]
fn test_invalid_unicode_escape() {
    let result = serde_edn::from_str::<Value>(r#"\u00G1"#);
    assert!(result.is_err()); // G is not hex

    let result = serde_edn::from_str::<Value>(r#"\u004"#); // Only 3 digits
    assert!(result.is_err());
}
