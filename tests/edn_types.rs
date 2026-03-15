use serde_edn::{from_str, to_string, EdnList, EdnSet, Value};

#[test]
fn test_edn_list_serialization() {
    let list = EdnList::from_vec(vec![1, 2, 3]);
    let edn = to_string(&list).unwrap();
    assert_eq!(edn, "(1 2 3)");
}

#[test]
fn test_edn_list_deserialization() {
    let list: EdnList<i32> = from_str("(1 2 3)").unwrap();
    assert_eq!(list.items, vec![1, 2, 3]);
}

#[test]
fn test_edn_set_serialization() {
    let mut set = EdnSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);
    let edn = to_string(&set).unwrap();
    assert!(edn.starts_with("#{"));
    assert!(edn.ends_with("}"));
}

#[test]
fn test_edn_set_deserialization() {
    let set: EdnSet<i32> = from_str("#{1 2 3}").unwrap();
    assert!(set.contains(&1));
    assert!(set.contains(&2));
    assert!(set.contains(&3));
}

#[test]
fn test_value_list_serialization() {
    let val = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
    let edn = to_string(&val).unwrap();
    assert_eq!(edn, "(1 2)");
}

#[test]
fn test_value_set_serialization() {
    let val = Value::Set(vec![Value::Integer(1), Value::Integer(2)]);
    let edn = to_string(&val).unwrap();
    assert_eq!(edn, "#{1 2}");
}

#[test]
fn test_value_vector_uses_brackets() {
    let val = Value::Vector(vec![Value::Integer(1), Value::Integer(2)]);
    let edn = to_string(&val).unwrap();
    assert_eq!(edn, "[1 2]");
}

#[test]
fn test_round_trip_list() {
    let original = "(1 2 3)";
    let val: Value = from_str(original).unwrap();
    let edn = to_string(&val).unwrap();
    assert_eq!(edn, original);
}

#[test]
fn test_round_trip_set() {
    let original = "#{1 2 3}";
    let val: Value = from_str(original).unwrap();
    let edn = to_string(&val).unwrap();
    assert_eq!(edn, original);
}

#[test]
fn test_empty_list() {
    let list = EdnList::<i32>::new();
    let edn = to_string(&list).unwrap();
    assert_eq!(edn, "()");
    let parsed: EdnList<i32> = from_str("()").unwrap();
    assert!(parsed.is_empty());
}

#[test]
fn test_empty_set() {
    let set = EdnSet::<i32>::new();
    let edn = to_string(&set).unwrap();
    assert_eq!(edn, "#{}");
    let parsed: EdnSet<i32> = from_str("#{}").unwrap();
    assert!(parsed.is_empty());
}

#[test]
fn test_nested_list() {
    let list = EdnList::from_vec(vec![
        EdnList::from_vec(vec![1, 2]),
        EdnList::from_vec(vec![3, 4]),
    ]);
    let edn = to_string(&list).unwrap();
    assert_eq!(edn, "((1 2) (3 4))");
    let parsed: EdnList<EdnList<i32>> = from_str("((1 2) (3 4))").unwrap();
    assert_eq!(parsed.items.len(), 2);
}

#[test]
fn test_list_with_strings() {
    let list = EdnList::from_vec(vec!["hello".to_string(), "world".to_string()]);
    let edn = to_string(&list).unwrap();
    assert_eq!(edn, r#"("hello" "world")"#);
    let parsed: EdnList<String> = from_str(r#"("hello" "world")"#).unwrap();
    assert_eq!(parsed.items, vec!["hello", "world"]);
}

#[test]
fn test_vec_still_uses_brackets() {
    let v = vec![1, 2, 3];
    let edn = to_string(&v).unwrap();
    assert_eq!(edn, "[1 2 3]");
}

#[test]
fn test_value_vector_round_trip() {
    let original = "[1 2 3]";
    let val: Value = from_str(original).unwrap();
    match &val {
        Value::Vector(items) => assert_eq!(items.len(), 3),
        _ => panic!("expected Vector"),
    }
    let edn = to_string(&val).unwrap();
    assert_eq!(edn, original);
}
