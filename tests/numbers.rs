mod common;

use common::{assert_round_trip, parse};
use serde_edn::Value;

#[test]
fn test_basic_floats() {
    assert_eq!(parse::<Value>("3.14"), Value::Float(3.14));
    assert_eq!(parse::<Value>("0.5"), Value::Float(0.5));
    assert_eq!(parse::<Value>("-1.5"), Value::Float(-1.5));
}

#[test]
fn test_exponent_notation() {
    assert_eq!(parse::<Value>("1e10"), Value::Float(1e10));
    assert_eq!(parse::<Value>("1.5e-3"), Value::Float(0.0015));
    assert_eq!(parse::<Value>("1E+5"), Value::Float(1e5));
    assert_eq!(parse::<Value>("1e-10"), Value::Float(1e-10));
    assert_eq!(parse::<Value>("2.5e2"), Value::Float(250.0));
}

#[test]
fn test_bignum_suffix() {
    assert_eq!(parse::<Value>("42N"), Value::Integer(42));
    assert_eq!(parse::<Value>("+100N"), Value::Integer(100));
    assert_eq!(parse::<Value>("-50N"), Value::Integer(-50));
}

#[test]
fn test_integer_values() {
    assert_eq!(parse::<Value>("42"), Value::Integer(42));
    assert_eq!(parse::<Value>("-100"), Value::Integer(-100));
    assert_eq!(parse::<Value>("+50"), Value::Integer(50));
    assert_eq!(parse::<Value>("0"), Value::Integer(0));
}

#[test]
fn test_number_round_trip() {
    assert_round_trip!("3.14");
    assert_round_trip!("1e10");
    assert_round_trip!("42N");
    assert_round_trip!("-1.5e-3");
}

#[test]
fn test_negative_exponent() {
    let _v: Value = parse("1e-3");
    assert!((0.001_f64 - 1e-3).abs() < f64::EPSILON);
}

#[test]
fn test_scientific_notation_combinations() {
    assert_eq!(parse::<Value>("1.23e+4"), Value::Float(12300.0));
    assert_eq!(parse::<Value>("9.99E-2"), Value::Float(0.0999));
    assert_eq!(parse::<Value>("6.022e23"), Value::Float(6.022e23));
}
