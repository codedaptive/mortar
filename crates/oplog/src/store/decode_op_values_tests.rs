//! decode_op_values: pairs in order, an empty array, and each refusal naming its 1-based item.
//! The values are built as JsonValue literals, so no parser is involved.

use super::{decode_op_values, JsonValue};

fn s(text: &str) -> JsonValue {
    JsonValue::String(text.to_string())
}

fn obj(members: &[(&str, JsonValue)]) -> JsonValue {
    JsonValue::Object(members.iter().map(|(k, v)| (k.to_string(), v.clone())).collect())
}

fn pair(name: &str, value: &str) -> JsonValue {
    obj(&[("name", s(name)), ("value", s(value))])
}

fn refusal(items: Vec<JsonValue>) -> String {
    decode_op_values(JsonValue::Array(items)).unwrap_err()
}

#[test]
fn two_pairs_in_array_order() {
    let items = vec![pair("b", "2"), obj(&[("value", s("")), ("name", s("a"))])];
    assert_eq!(
        decode_op_values(JsonValue::Array(items)),
        Ok(vec![
            ("b".to_string(), "2".to_string()),
            ("a".to_string(), String::new()),
        ])
    );
}

#[test]
fn an_empty_array_is_no_values() {
    assert_eq!(decode_op_values(JsonValue::Array(Vec::new())), Ok(Vec::new()));
}

#[test]
fn not_an_array_is_refused() {
    for value in [pair("a", "b"), JsonValue::Null, s("x")] {
        assert_eq!(decode_op_values(value), Err("values is not an array".to_string()));
    }
}

#[test]
fn an_item_that_is_not_an_object_is_refused() {
    assert_eq!(
        refusal(vec![pair("a", "b"), s("x")]),
        "value 2: the item is not a JSON object"
    );
}

#[test]
fn a_missing_name_is_refused() {
    assert_eq!(refusal(vec![obj(&[("value", s("b"))])]), "value 1: missing key \"name\"");
}

#[test]
fn a_value_that_is_not_a_string_is_refused() {
    assert_eq!(
        refusal(vec![obj(&[("name", s("a")), ("value", JsonValue::Number(3))])]),
        "value 1: value is not a string"
    );
}

#[test]
fn an_unknown_item_key_is_refused() {
    let extra = obj(&[("name", s("a")), ("value", s("b")), ("x", s("c"))]);
    assert_eq!(
        refusal(vec![pair("a", "b"), extra]),
        "value 2: the item has unknown key \"x\""
    );
}

#[test]
fn a_repeated_item_key_is_refused() {
    let twice = obj(&[("name", s("a")), ("name", s("b")), ("value", s("c"))]);
    assert_eq!(refusal(vec![twice]), "value 1: the item has key \"name\" twice");
}
