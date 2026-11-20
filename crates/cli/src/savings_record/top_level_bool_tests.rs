//! top_level_bool: the last top-level boolean member of the key, or None; Err for anything but one JSON object.

use super::top_level_bool;

const KEY: &str = "savings_line";

#[test]
fn an_empty_object_has_none() {
    assert_eq!(top_level_bool("{}", KEY), Ok(None));
}

#[test]
fn false_and_true() {
    assert_eq!(top_level_bool(r#"{"savings_line": false}"#, KEY), Ok(Some(false)));
    assert_eq!(top_level_bool(r#"{"savings_line": true}"#, KEY), Ok(Some(true)));
}

#[test]
fn whitespace_around_and_inside() {
    assert_eq!(top_level_bool(" \n\t{ \r\n\"savings_line\" :\tfalse\n}\r\n ", KEY), Ok(Some(false)));
}

#[test]
fn the_last_duplicate_wins() {
    assert_eq!(top_level_bool(r#"{"savings_line": false, "savings_line": true}"#, KEY), Ok(Some(true)));
    assert_eq!(top_level_bool(r#"{"savings_line": true, "savings_line": false}"#, KEY), Ok(Some(false)));
}

#[test]
fn the_last_boolean_of_the_key_wins_over_a_later_non_boolean() {
    assert_eq!(top_level_bool(r#"{"savings_line": false, "savings_line": 0}"#, KEY), Ok(Some(false)));
}

#[test]
fn nested_members_do_not_count() {
    assert_eq!(top_level_bool(r#"{"a":{"savings_line":false}}"#, KEY), Ok(None));
    assert_eq!(top_level_bool(r#"{"a":[{"savings_line":false}]}"#, KEY), Ok(None));
}

#[test]
fn a_non_boolean_value_is_none() {
    for value in ["0", "null", "\"false\"", "[false]", "{}", "-1.5e3"] {
        assert_eq!(top_level_bool(&format!("{{\"savings_line\": {value}}}"), KEY), Ok(None), "{value}");
    }
}

#[test]
fn other_keys_of_every_type_around_it() {
    let text = r#"{"s": "a \"q\" \\ \/ \b\f\n\r\t \u00e9 \ud83c\udf31 é", "n": -0.5e+10, "z": 0, "i": 12, "e": 1E-2,
        "t": true, "f": false, "u": null, "a": [1, "x", [], {}, [true, null]], "o": {"k": {"savings_line": true}},
        "savings_line": false, "after": [{"x": -3}], "empty": ""}"#;
    assert_eq!(top_level_bool(text, KEY), Ok(Some(false)));
}

#[test]
fn the_key_is_matched_after_its_escapes_are_decoded() {
    assert_eq!(top_level_bool(r#"{"savings\u005fline": false}"#, KEY), Ok(Some(false)));
    assert_eq!(top_level_bool(r#"{"savings_lin": false, "Savings_line": false}"#, KEY), Ok(None));
}

#[test]
fn text_that_is_not_one_object_is_an_error() {
    let cases = [
        r#"{"savings_line": fals"#,
        "[]",
        "",
        "   ",
        "false",
        r#"{"savings_line": false} x"#,
        r#"{"savings_line": false}{}"#,
        r#"{"savings_line": "\x"}"#,
        r#"{"savings_line": "\u12g4"}"#,
        r#"{"savings_line": false,}"#,
        r#"{"savings_line" false}"#,
        r#"{savings_line: false}"#,
        r#"{"a": [1,]}"#,
        r#"{"a": 01}"#,
        r#"{"a": 1.}"#,
        r#"{"a": 1e}"#,
        r#"{"a": +1}"#,
        r#"{"a": -}"#,
        "{\"a\": \"raw\ttab\"}",
        r#"{"a": "unclosed}"#,
        r#"{"a": nul}"#,
        r#"{"a": True}"#,
        "{",
    ];
    for text in cases {
        assert!(top_level_bool(text, KEY).is_err(), "{text}");
    }
}
