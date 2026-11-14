//! parse_record_json: the record's object with string, null and string-array values; everything else refused.

use super::{parse_record_json, JsonError, RecordJsonValue};

fn s(text: &str) -> RecordJsonValue {
    RecordJsonValue::String(text.to_string())
}

fn invalid(text: &str) -> String {
    match parse_record_json(text) {
        Err(JsonError::Invalid(reason)) => reason,
        other => panic!("{text:?} gave {other:?}"),
    }
}

#[test]
fn members_in_text_order_with_each_value_kind() {
    assert_eq!(
        parse_record_json(r#"{"b":"x","a":null,"c":["p","q"],"d":[]}"#),
        Ok(vec![
            ("b".to_string(), s("x")),
            ("a".to_string(), RecordJsonValue::Null),
            ("c".to_string(), RecordJsonValue::Strings(vec!["p".to_string(), "q".to_string()])),
            ("d".to_string(), RecordJsonValue::Strings(Vec::new())),
        ])
    );
}

#[test]
fn whitespace_around_every_token() {
    assert_eq!(
        parse_record_json(" \n{ \"k\" :\t[ \"a\" , \"b\" ] ,\r\"n\" : null } \n"),
        Ok(vec![
            ("k".to_string(), RecordJsonValue::Strings(vec!["a".to_string(), "b".to_string()])),
            ("n".to_string(), RecordJsonValue::Null),
        ])
    );
    assert_eq!(parse_record_json(" { } "), Ok(Vec::new()));
}

#[test]
fn repeated_keys_are_kept() {
    assert_eq!(
        parse_record_json(r#"{"a":"1","a":"2"}"#),
        Ok(vec![("a".to_string(), s("1")), ("a".to_string(), s("2"))])
    );
}

#[test]
fn non_object_is_refused() {
    assert_eq!(
        invalid(r#"["a"]"#),
        "unexpected '[' at byte 0 where the record's '{' belongs: the record is not a JSON object"
    );
    assert_eq!(
        invalid(" null"),
        "unexpected 'n' at byte 1 where the record's '{' belongs: the record is not a JSON object"
    );
}

#[test]
fn numbers_booleans_and_nested_objects_are_refused_naming_the_key() {
    let tail = "which must be a string, null or an array of strings";
    assert_eq!(invalid(r#"{"zone":1}"#), format!("unexpected '1' at byte 8 as the value of \"zone\", {tail}"));
    assert_eq!(invalid(r#"{"zone":true}"#), format!("unexpected 't' at byte 8 as the value of \"zone\", {tail}"));
    assert_eq!(invalid(r#"{"zone":{}}"#), format!("unexpected '{{' at byte 8 as the value of \"zone\", {tail}"));
}

#[test]
fn array_items_other_than_strings_are_refused_naming_the_key() {
    assert_eq!(
        invalid(r#"{"p":["a",null]}"#),
        "unexpected 'n' at byte 10 in the array \"p\", where a string belongs"
    );
    assert_eq!(invalid(r#"{"p":[["a"]]}"#), "unexpected '[' at byte 6 in the array \"p\", where a string belongs");
}

#[test]
fn trailing_text_and_bad_punctuation_are_refused() {
    assert_eq!(invalid(r#"{"a":null} x"#), "unexpected 'x' at byte 11 after the record");
    assert_eq!(invalid(r#"{"a":null}{}"#), "unexpected '{' at byte 10 after the record");
    assert_eq!(invalid(r#"{"a" null}"#), "unexpected 'n' at byte 5 after a key");
    assert_eq!(invalid(r#"{"a":null "b":null}"#), "unexpected '\"' at byte 10 after a member (',' or '}')");
    assert_eq!(invalid(r#"{"a":null,}"#), "unexpected '}' at byte 10 where a key belongs");
    assert_eq!(invalid(r#"{"a":["x",]}"#), "unexpected ']' at byte 10 in the array \"a\", where a string belongs");
    assert_eq!(invalid(r#"{"a":nil}"#), "unexpected 'n' at byte 5 where null was begun");
}

#[test]
fn text_that_stops_early_is_end_of_input() {
    for text in ["", "  ", "{", "{\"a", "{\"a\"", "{\"a\":", "{\"a\":n", "{\"a\":nul", "{\"a\":\"x", "{\"a\":[", "{\"a\":[\"x\"", "{\"a\":[\"x\",", "{\"a\":null", "{\"a\":null,"] {
        assert_eq!(parse_record_json(text), Err(JsonError::EndOfInput), "{text:?}");
    }
}
