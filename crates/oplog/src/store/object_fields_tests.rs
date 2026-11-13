//! object_fields: values sorted into the key list's slots; unknown, repeated and non-object refused.

use super::{object_fields, JsonValue};

fn n(v: u64) -> JsonValue {
    JsonValue::Number(v)
}

fn object(pairs: &[(&str, u64)]) -> JsonValue {
    JsonValue::Object(pairs.iter().map(|(k, v)| (k.to_string(), n(*v))).collect())
}

#[test]
fn slots_follow_the_key_list_not_the_text_order() {
    assert_eq!(
        object_fields(object(&[("c", 3), ("a", 1), ("b", 2)]), ["a", "b", "c"], "x"),
        Ok([Some(n(1)), Some(n(2)), Some(n(3))])
    );
}

#[test]
fn absent_keys_are_none() {
    assert_eq!(
        object_fields(object(&[("b", 2)]), ["a", "b", "c"], "x"),
        Ok([None, Some(n(2)), None])
    );
    assert_eq!(object_fields(object(&[]), ["a"], "x"), Ok([None]));
}

#[test]
fn unknown_key_is_refused_by_name() {
    assert_eq!(
        object_fields(object(&[("a", 1), ("zz", 2)]), ["a"], "the move"),
        Err("the move has unknown key \"zz\"".to_string())
    );
}

#[test]
fn repeated_key_is_refused_by_name() {
    assert_eq!(
        object_fields(object(&[("a", 1), ("a", 1)]), ["a", "b"], "the record"),
        Err("the record has key \"a\" twice".to_string())
    );
}

#[test]
fn non_object_is_refused() {
    for value in [JsonValue::Null, n(1), JsonValue::String("{}".to_string()), JsonValue::Array(Vec::new())] {
        assert_eq!(
            object_fields(value, ["a"], "the result"),
            Err("the result is not a JSON object".to_string())
        );
    }
}
