//! parse_json: the values a record holds, the forms it refuses, and EndOfInput for text cut short.

use super::{parse_json, JsonError, JsonValue};

fn s(text: &str) -> JsonValue {
    JsonValue::String(text.to_string())
}

fn invalid(text: &str) -> bool {
    matches!(parse_json(text), Err(JsonError::Invalid(_)))
}

#[test]
fn scalars() {
    assert_eq!(parse_json("null"), Ok(JsonValue::Null));
    assert_eq!(parse_json("0"), Ok(JsonValue::Number(0)));
    assert_eq!(parse_json("1763030400"), Ok(JsonValue::Number(1763030400)));
    assert_eq!(
        parse_json("18446744073709551615"),
        Ok(JsonValue::Number(18446744073709551615))
    );
    assert_eq!(parse_json(r#""a\nb""#), Ok(s("a\nb")));
}

#[test]
fn empty_containers() {
    assert_eq!(parse_json("{}"), Ok(JsonValue::Object(Vec::new())));
    assert_eq!(parse_json("[]"), Ok(JsonValue::Array(Vec::new())));
    assert_eq!(parse_json("[ ]"), Ok(JsonValue::Array(Vec::new())));
    assert_eq!(parse_json("{ }"), Ok(JsonValue::Object(Vec::new())));
}

#[test]
fn nested_with_whitespace_everywhere() {
    assert_eq!(
        parse_json(" \t\r\n{ \"a\" : [ 1 , null , { } ] ,\n\"b\":\"x\" } \n"),
        Ok(JsonValue::Object(vec![
            (
                "a".to_string(),
                JsonValue::Array(vec![
                    JsonValue::Number(1),
                    JsonValue::Null,
                    JsonValue::Object(Vec::new()),
                ])
            ),
            ("b".to_string(), s("x")),
        ]))
    );
}

#[test]
fn object_keeps_text_order_and_repeated_keys() {
    assert_eq!(
        parse_json(r#"{"z":1,"a":2,"z":3}"#),
        Ok(JsonValue::Object(vec![
            ("z".to_string(), JsonValue::Number(1)),
            ("a".to_string(), JsonValue::Number(2)),
            ("z".to_string(), JsonValue::Number(3)),
        ]))
    );
}

#[test]
fn numbers_outside_the_subset_are_invalid() {
    for text in ["-1", "1.5", "1e3", "1E3", "01", "00", "18446744073709551616", "+1", "[01]"] {
        assert!(invalid(text), "{text:?}");
    }
}

#[test]
fn other_json_and_junk_are_invalid() {
    for text in [
        "true",
        "false",
        "nux",
        "nulL",
        "{} x",
        "null null",
        "{\"a\":1,}",
        "[1,]",
        "[,1]",
        "{1:2}",
        "{\"a\" 1}",
        "{\"a\":1 \"b\":2}",
        "[1 2]",
        "'a'",
        "é",
    ] {
        assert!(invalid(text), "{text:?}");
    }
}

#[test]
fn text_cut_short_is_end_of_input() {
    for text in [
        "",
        "   ",
        "{",
        "{\"a\"",
        "{\"a\" ",
        "{\"a\":",
        "{\"a\":1",
        "{\"a\":1,",
        "{\"a\":1, ",
        "{\"a\":\"x",
        "[",
        "[1",
        "[1,",
        "[[",
        "n",
        "nu",
        "nul",
        "{\"a\":nu",
    ] {
        assert_eq!(parse_json(text), Err(JsonError::EndOfInput), "{text:?}");
    }
}
