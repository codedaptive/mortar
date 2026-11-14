//! The JSON parser through the crate's public path: `oplog::parse_json`, `oplog::JsonValue` and
//! `oplog::JsonError` as a caller outside the crate names them. No repository is involved.

use oplog::{parse_json, JsonError, JsonValue};

#[test]
fn public_parse_json_keeps_key_order_and_duplicates() {
    let expected = JsonValue::Object(vec![
        (
            "b".to_string(),
            JsonValue::Array(vec![JsonValue::Null, JsonValue::String("x".to_string()), JsonValue::Number(7)]),
        ),
        ("a".to_string(), JsonValue::Object(vec![])),
        ("b".to_string(), JsonValue::Number(0)),
    ]);
    assert_eq!(parse_json(r#"{"b":[null,"x",7],"a":{},"b":0}"#), Ok(expected));
}

#[test]
fn public_parse_json_refuses_through_the_public_error_type() {
    assert!(matches!(parse_json("true"), Err(JsonError::Invalid(_))), "true: {:?}", parse_json("true"));
    assert!(matches!(parse_json(r#"{"a":"#), Err(JsonError::EndOfInput)), "torn: {:?}", parse_json(r#"{"a":"#));
}
