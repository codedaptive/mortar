//! savings_line_on: off only when the last top-level boolean "savings_line" is false; everything else is on.

use super::savings_line_on;

#[test]
fn on_for_a_missing_file_or_an_empty_object() {
    assert!(savings_line_on(None));
    assert!(savings_line_on(Some("{}")));
}

#[test]
fn off_for_false_and_on_for_true() {
    assert!(!savings_line_on(Some(r#"{"savings_line": false}"#)));
    assert!(!savings_line_on(Some(r#"{"other": 1, "savings_line": false}"#)));
    assert!(savings_line_on(Some(r#"{"savings_line": true}"#)));
}

#[test]
fn whitespace_around_and_inside() {
    assert!(!savings_line_on(Some(" \n\t{ \r\n\"savings_line\" :\tfalse\n}\r\n ")));
}

#[test]
fn the_last_duplicate_wins() {
    assert!(savings_line_on(Some(r#"{"savings_line": false, "savings_line": true}"#)));
    assert!(!savings_line_on(Some(r#"{"savings_line": true, "savings_line": false}"#)));
}

#[test]
fn the_last_boolean_wins_over_a_later_non_boolean() {
    assert!(!savings_line_on(Some(r#"{"savings_line": false, "savings_line": 0}"#)));
    assert!(!savings_line_on(Some(r#"{"savings_line": false, "savings_line": null}"#)));
}

#[test]
fn on_for_a_non_boolean() {
    for value in ["0", "null", "\"false\"", "[false]", "{}"] {
        assert!(savings_line_on(Some(&format!("{{\"savings_line\": {value}}}"))), "{value}");
    }
}

#[test]
fn nested_members_do_not_count() {
    assert!(savings_line_on(Some(r#"{"a":{"savings_line":false}}"#)));
    assert!(savings_line_on(Some(r#"{"a":[{"savings_line":false}]}"#)));
}

#[test]
fn the_key_is_matched_after_its_escapes_are_decoded() {
    assert!(!savings_line_on(Some(r#"{"savings\u005fline": false}"#)));
    assert!(savings_line_on(Some(r#"{"savings_lin": false, "Savings_line": false}"#)));
}

#[test]
fn other_keys_of_every_type_around_it() {
    let text = r#"{"s": "a \"q\" \\ \/ \b\f\n\r\t é 🌱 é", "z": 0, "i": 12, "big": 18446744073709551615,
        "t": true, "f": false, "u": null, "a": [1, "x", [], {}, [true, null]], "o": {"k": {"savings_line": true}},
        "savings_line": false, "after": [{"x": 3}], "empty": ""}"#;
    assert!(!savings_line_on(Some(text)));
}

#[test]
fn on_for_a_top_level_that_is_not_an_object() {
    for text in ["[]", "false", "[{\"savings_line\": false}]", "\"savings_line\"", "0", "null"] {
        assert!(savings_line_on(Some(text)), "{text}");
    }
}

#[test]
fn on_for_text_that_does_not_parse() {
    let cases = [
        r#"{"savings_line": fals"#,
        "",
        "   ",
        r#"{"savings_line": false} x"#,
        r#"{"savings_line": false} trailing"#,
        r#"{"savings_line": false}{}"#,
        r#"{"savings_line": false, "a": "\x"}"#,
        r#"{"savings_line": false, "a": "\u12g4"}"#,
        r#"{"savings_line": false,}"#,
        r#"{"savings_line" false}"#,
        r#"{savings_line: false}"#,
        r#"{"savings_line": false, "a": [1,]}"#,
        r#"{"savings_line": false, "a": 01}"#,
        r#"{"savings_line": false, "a": 1.}"#,
        r#"{"savings_line": false, "a": 1e}"#,
        r#"{"savings_line": false, "a": +1}"#,
        r#"{"savings_line": false, "a": -}"#,
        "{\"savings_line\": false, \"a\": \"raw\ttab\"}",
        r#"{"savings_line": false, "a": "unclosed}"#,
        r#"{"savings_line": false, "a": nul}"#,
        r#"{"savings_line": false, "a": True}"#,
        "{",
    ];
    for text in cases {
        assert!(savings_line_on(Some(text)), "{text}");
    }
}

#[test]
fn numbers_and_surrogates_parse_json_refuses_read_as_unparsable_and_keep_the_line_on() {
    let cases = [
        r#"{"savings_line": false, "x": -3}"#,
        r#"{"savings_line": false, "x": 1.5}"#,
        r#"{"savings_line": false, "x": 2e3}"#,
        r#"{"savings_line": false, "x": 18446744073709551616}"#,
        r#"{"savings_line": false, "x": "\uD800"}"#,
    ];
    for text in cases {
        assert!(savings_line_on(Some(text)), "{text}");
    }
}
