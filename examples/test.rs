use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Person {
    name: String,
    age: i32,
}

fn main() {
    // Test deserialization
    let edn = r#"{:name "Fred", :age 30}"#;
    let person: Person = serde_edn::from_str(edn).unwrap();
    println!("Deserialized: {:?}", person);

    // Test serialization
    let person2 = Person {
        name: "Alice".to_string(),
        age: 25,
    };
    let edn2 = serde_edn::to_string(&person2).unwrap();
    println!("Serialized: {}", edn2);

    // Test Value
    let value: serde_edn::Value = serde_edn::from_str("[1 2 3]").unwrap();
    println!("Value: {:?}", value);

    // Test vector
    let vec: Vec<i32> = serde_edn::from_str("[1 2 3]").unwrap();
    println!("Vec: {:?}", vec);

    eprintln!("=== Testing UUID parsing ===");
    let input = r#"#uuid "f81d4fae-7dec-11d0-a765-00a0c91e6bf6""#;
    eprintln!("Input: '{}'", input);
    eprintln!(
        "Input chars: {:?}",
        input.chars().take(10).collect::<Vec<_>>()
    );

    let v: serde_edn::Value = serde_edn::from_str(input).unwrap();
    eprintln!("Parsed: {:?}", v);
}
