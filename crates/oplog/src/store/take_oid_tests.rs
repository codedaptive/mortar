//! take_oid: an object id member, checked; a bad id is refused naming the key and the text.

use super::{take_oid, JsonValue};

fn text(s: &str) -> Option<JsonValue> {
    Some(JsonValue::String(s.to_string()))
}

#[test]
fn sha1_sha256_and_zero_ids_are_taken() {
    assert_eq!(take_oid(text("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), "new").map(|o| o.as_str().to_string()), Ok("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()));
    assert_eq!(take_oid(text("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"), "new").map(|o| o.as_str().to_string()), Ok("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string()));
    assert_eq!(take_oid(text("0000000000000000000000000000000000000000"), "new").map(|o| o.as_str().to_string()), Ok("0000000000000000000000000000000000000000".to_string()));
}

#[test]
fn bad_ids_are_refused_with_their_text() {
    assert_eq!(take_oid(text("abc"), "new"), Err("new \"abc\" is not a git object id".to_string()));
    let upper = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert_eq!(
        take_oid(text(upper), "old"),
        Err(format!("old \"{upper}\" is not a git object id"))
    );
}

#[test]
fn missing_or_non_string_is_refused() {
    assert_eq!(take_oid(None, "new"), Err("missing key \"new\"".to_string()));
    assert_eq!(take_oid(Some(JsonValue::Null), "new"), Err("new is not a string".to_string()));
}
