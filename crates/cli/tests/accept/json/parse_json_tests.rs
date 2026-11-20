use super::{parse_json, JsonValue};

fn s(text: &str) -> JsonValue {
    JsonValue::Str(text.to_string())
}

fn n(text: &str) -> JsonValue {
    JsonValue::Number(text.to_string())
}

#[test]
fn init_json_line_parses_member_by_member_in_order() {
    let line = r#"{"verb":"init","ok":true,"outcome":"written","exit":0,"next":"mr folder zone <zone> <path>","lines":["wrote the Mortar config /x/.git/mortar/config.json"],"owner":null,"config_path":"/x/.git/mortar/config.json"}"#;
    let expected = JsonValue::Object(vec![
        ("verb".to_string(), s("init")),
        ("ok".to_string(), JsonValue::Bool(true)),
        ("outcome".to_string(), s("written")),
        ("exit".to_string(), n("0")),
        ("next".to_string(), s("mr folder zone <zone> <path>")),
        ("lines".to_string(), JsonValue::Array(vec![s("wrote the Mortar config /x/.git/mortar/config.json")])),
        ("owner".to_string(), JsonValue::Null),
        ("config_path".to_string(), s("/x/.git/mortar/config.json")),
    ]);
    assert_eq!(parse_json(&format!("{line}\n")), Ok(expected));
}

#[test]
fn string_escapes_and_a_surrogate_pair() {
    let text = r#""a\"b\\c\/d\b\f\n\r\t\u00e9\ud83d\ude00""#;
    assert_eq!(parse_json(text), Ok(s("a\"b\\c/d\u{8}\u{c}\n\r\t\u{e9}\u{1f600}")));
}

#[test]
fn a_surrogate_pair_escape_is_one_character() {
    assert_eq!(parse_json(r#""A\ud83d\ude00""#), Ok(s("A\u{1f600}")));
}

#[test]
fn nested_arrays_numbers_and_whitespace() {
    let expected = JsonValue::Array(vec![
        JsonValue::Array(vec![]),
        JsonValue::Array(vec![n("1"), n("-2.5e+3"), n("0")]),
        JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Null, JsonValue::Bool(false)])]),
        JsonValue::Object(vec![]),
    ]);
    assert_eq!(parse_json(" [ [], [1, -2.5e+3, 0], [[null, false]], {} ]\n"), Ok(expected));
}

#[test]
fn truncated_text_is_an_error() {
    assert_eq!(parse_json(r#"{"a":1"#), Err("expected ',' or '}' at the end of the text".to_string()));
    assert_eq!(parse_json(r#""abc"#), Err("unterminated string at the end of the text".to_string()));
    assert_eq!(parse_json(""), Err("expected a value at the end of the text".to_string()));
}

#[test]
fn trailing_text_is_an_error() {
    assert_eq!(parse_json("{} x"), Err("trailing text at byte 3".to_string()));
    assert_eq!(parse_json("01"), Err("trailing text at byte 1".to_string()));
}

#[test]
fn malformed_values_are_errors() {
    assert_eq!(parse_json("[1,]"), Err("expected a value at byte 3".to_string()));
    assert_eq!(parse_json(r#""\ud83d""#), Err("lone high surrogate at byte 7".to_string()));
    assert_eq!(parse_json(r#"{"a" 1}"#), Err("expected ':' at byte 5".to_string()));
    assert_eq!(parse_json("1."), Err("expected a digit at the end of the text".to_string()));
}
