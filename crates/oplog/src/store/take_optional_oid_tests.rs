//! take_optional_oid: null is None, an id is Some; the key must be present.

use super::{take_optional_oid, JsonValue};

#[test]
fn null_is_none() {
    assert_eq!(take_optional_oid(Some(JsonValue::Null), "old"), Ok(None));
}

#[test]
fn id_is_some() {
    let got = take_optional_oid(Some(JsonValue::String("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string())), "old");
    assert_eq!(got.map(|o| o.map(|o| o.as_str().to_string())), Ok(Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string())));
}

#[test]
fn absent_key_is_refused() {
    assert_eq!(take_optional_oid(None, "actual"), Err("missing key \"actual\"".to_string()));
}

#[test]
fn bad_id_and_number_are_refused() {
    assert_eq!(
        take_optional_oid(Some(JsonValue::String("null".to_string())), "old"),
        Err("old \"null\" is not a git object id".to_string())
    );
    assert_eq!(
        take_optional_oid(Some(JsonValue::Number(0)), "old"),
        Err("old is not a string".to_string())
    );
}
