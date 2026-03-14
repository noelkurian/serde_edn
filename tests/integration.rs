// Integration test with large nested document covering all data types

mod common;

use common::assert_round_trip;

#[test]
fn test_large_nested_document() {
    // Test all EDN data types in a single nested structure
    let edn = r#"{
  :name "Test Document",
  :version 1,
  :float-value 3.14,
  :scientific 1.5e-3,
  :bignum 42N,
  :boolean true,
  :nil-value nil,
  :characters [\A \newline \return \tab \space \u0041 \u03B1 \u2764],
  :symbols (foo bar/baz name space.ns/name),
  :keywords [:a :b :user/name :myapp/ns-tag],
  :list (1 2 3 "string" :keyword nil),
  :vector [true false "vector" [nested vector] {nested map}],
  :nested-map {:inner "value", :number 42},
  :set #{1 2 3 4 5 :a :b},
  :inst   #inst "1985-04-12T23:20:50.52Z",
  :uuid #uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6",
  :custom-tag #myapp/custom "custom value",
  :deep-nesting {:level1 {:level2 {:level3 {:level4 "deepest"}}}},
  :complex-vector [
    {:a 1 :b 2}
    {:c 3 :d 4}
    [nested inside vector]
    {:nested {:inner {:deep [1 2 3]}}}
  ]
}"#;

    // Ensure the document parses without errors
    let v1: serde_edn::Value = serde_edn::from_str(edn).unwrap();

    // Ensure the value serializes back to valid EDN
    let serialized = serde_edn::to_string(&v1).unwrap();

    // Round-trip test
    let v2: serde_edn::Value = serde_edn::from_str(&serialized).unwrap();

    // Values should be equivalent (structural equality)
    assert_eq!(v1, v2, "Round-trip failed for integration test");
}

#[test]
fn test_unicode_and_exponents() {
    let edn = r#"{
  :unicode-chars [\u0041 \u03B1 \u01C0 \u2665 \u00A9],
  :exponent-values [1e10 1.5e-3 2.5e2 9.9E-2],
  :bignum-values [42N +100N -50N],
  :mixed [1e10 42N 3.14 -1.5e-3]
}"#;

    assert_round_trip!(edn);
}

#[test]
fn test_all_tagged_literals() {
    let edn = r#"[
  #inst "1985-04-12T23:20:50Z"
  #inst "2024-01-01T00:00:00+05:00"
  #uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6"
  #uuid "00000000-0000-0000-0000-000000000000"
  #myapp/tag1 "value1"
  #myapp/ns.tag2 "value2"
  #myapp/tag3 {:nested "value"}
]"#;

    assert_round_trip!(edn);
}

#[test]
fn test_nested_collections() {
    let edn = r#"{
  :map-of-maps {
    :outer1 {:inner1a 1 :inner1b 2}
    :outer2 {:inner2a 3 :inner2b {:deep 4}}
  }
  :vector-of-vectors [
    [1 2 3]
    [4 5 6]
    [[nested]]
  ]
  :list-of-lists (
    (1 2 3)
    (4 5 6)
    ((nested in list))
  )
  :set-of-sets #{
    #{1 2 3}
    #{4 5 6}
  }
}"#;

    assert_round_trip!(edn);
}
