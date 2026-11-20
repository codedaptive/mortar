//! take_number: a number, or a refusal naming the key.

use oplog::JsonValue as V;

use super::take_number;

#[test]
fn a_number_is_taken() {
    assert_eq!(take_number(Some(V::Number(42)), "k"), Ok(42));
}

#[test]
fn missing_and_other_values_are_refused() {
    assert_eq!(take_number(None, "cap_bytes"), Err("missing key \"cap_bytes\"".to_string()));
    for value in [V::Null, V::Bool(false), V::String("1".to_string()), V::Array(Vec::new())] {
        assert_eq!(take_number(Some(value), "cap_bytes"), Err("cap_bytes is not a number".to_string()));
    }
}
