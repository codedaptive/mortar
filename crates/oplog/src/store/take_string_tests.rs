//! take_string: a string member, or a refusal naming the key.

use super::{take_string, JsonValue};

#[test]
fn string_is_taken() {
    assert_eq!(
        take_string(Some(JsonValue::String("land".to_string())), "kind"),
        Ok("land".to_string())
    );
}

#[test]
fn absent_key_is_missing() {
    assert_eq!(take_string(None, "actor"), Err("missing key \"actor\"".to_string()));
}

#[test]
fn other_values_are_not_strings() {
    for value in [JsonValue::Null, JsonValue::Number(3), JsonValue::Array(Vec::new())] {
        assert_eq!(take_string(Some(value), "kind"), Err("kind is not a string".to_string()));
    }
}
