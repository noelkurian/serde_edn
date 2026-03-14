// Common fixtures and helpers for all test modules

use serde_edn::{from_str, to_string};

pub fn parse<T>(s: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    from_str(s).unwrap()
}

pub fn serialize<T>(v: &T) -> String
where
    T: serde::Serialize,
{
    to_string(v).unwrap()
}

pub fn round_trip<T>(edn: &str) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let parsed: T = parse(edn);
    let serialized = serialize(&parsed);
    let reparsed: T = parse(&serialized);
    assert_eq!(parsed, reparsed);
    parsed
}

macro_rules! assert_round_trip {
    ($edn:expr) => {
        let parsed: serde_edn::Value = serde_edn::from_str($edn).unwrap();
        let serialized = serde_edn::to_string(&parsed).unwrap();
        let reparsed: serde_edn::Value = serde_edn::from_str(&serialized).unwrap();
        assert_eq!(
            parsed, reparsed,
            "Round trip failed: {} -> {} -> {:?}",
            $edn, serialized, reparsed
        );
    };
}

pub(crate) use assert_round_trip;
