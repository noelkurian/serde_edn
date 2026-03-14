mod common;

use chrono::{DateTime, Timelike, Utc};
use common::assert_round_trip;
use serde_edn::{from_str, to_string, Value};

#[test]
fn test_inst_parsing() {
    let v: Value = from_str(r#"#inst "1985-04-12T23:20:50.52Z""#).unwrap();
    assert!(matches!(
        v,
        Value::Tagged { ref tag, .. } if tag.as_str() == ":inst"
    ));

    let ms = match v {
        Value::Tagged { value: box_val, .. } => match *box_val {
            Value::Integer(m) => m,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    let dt = DateTime::from_timestamp_millis(ms)
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(
        dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "1985-04-12T23:20:50.520Z"
    );
}

#[test]
fn test_inst_roundtrip() {
    let input = r#"#inst "1985-04-12T23:20:50Z""#;
    let v: Value = from_str(input).unwrap();
    let output = to_string(&v).unwrap();

    println!("Output: {}", output);

    // Should round-trip (milliseconds added by conversion)
    assert!(output.starts_with("#inst"));
    assert!(output.contains("1985-04-12"));
}

#[test]
fn test_uuid_valid() {
    let v: Value = from_str(r#"#uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6""#).unwrap();
    assert_eq!(
        v,
        Value::String("f81d4fae-7dec-11d0-a765-00a0c91e6bf6".to_string())
    );
}

#[test]
fn test_uuid_invalid() {
    // Wrong length
    assert!(from_str::<Value>(r#"#uuid "not-a-uuid""#).is_err());
    // Wrong format
    assert!(from_str::<Value>(r#"#uuid "ffffffff-ffff-ffff-ffff-ffffffffffffg""#).is_err());
}

#[test]
fn test_custom_tag() {
    let v: Value = from_str(r#"#myapp/foo "bar""#).unwrap();
    match v {
        Value::Tagged { tag, value } => {
            assert_eq!(tag.as_str(), ":myapp/foo");
            assert_eq!(*value, Value::String("bar".to_string()));
        }
        _ => panic!("Expected Tagged value, got {:?}", v),
    }
}

#[test]
fn test_inst_timezone_conversion() {
    let v: Value = from_str(r#"#inst "2024-01-01T12:00:00+05:00""#).unwrap();
    assert!(matches!(
        v,
        Value::Tagged {
            ref tag,
            ..
        } if tag.as_str() == ":inst"
    ));

    let ms = match v {
        Value::Tagged { value: box_val, .. } => match *box_val {
            Value::Integer(m) => m,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    // +05:00 offset means 12:00 local = 07:00 UTC
    let dt = DateTime::from_timestamp_millis(ms)
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(dt.hour(), 7);
    assert_eq!(dt.minute(), 0);
}

#[test]
fn test_multiple_tags_in_vector() {
    let v: Value = from_str(
        r#"[#inst "2024-01-01T00:00:00Z" #uuid "12345678-1234-1234-1234-123456789abc" 42]"#,
    )
    .unwrap();

    if let Value::Vector(vec) = &v {
        assert!(matches!(
            vec[0],
            Value::Tagged {
                ref tag,
                ..
            } if tag.as_str() == ":inst"
        ));
        // Check inner value is Integer
        if let Value::Tagged { value: box_val, .. } = &vec[0] {
            assert!(matches!(&**box_val, Value::Integer(_)));
        }
        assert!(matches!(vec[1], Value::String(_)));
        assert_eq!(vec[2], Value::Integer(42));
    } else {
        panic!("Expected Vector");
    }
}

#[test]
fn test_custom_tag_roundtrip() {
    assert_round_trip!(r#"#myapp/foo "bar""#);
}
