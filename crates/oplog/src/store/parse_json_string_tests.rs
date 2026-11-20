//! parse_json_string: every JSON escape, surrogate pairs, and EndOfInput versus Invalid.
//! Each input starts with the opening quote; parsing starts at byte 1.

use super::{parse_json_string, JsonError};

fn parse(text: &str) -> Result<(String, usize), JsonError> {
    parse_json_string(text, 1)
}

fn invalid(text: &str) -> bool {
    matches!(parse(text), Err(JsonError::Invalid(_)))
}

#[test]
fn plain_string_and_end_position() {
    assert_eq!(parse(r#""ab""#), Ok(("ab".to_string(), 4)));
    assert_eq!(parse(r#""""#), Ok((String::new(), 2)));
    assert_eq!(parse(r#""a",1"#), Ok(("a".to_string(), 3)));
}

#[test]
fn every_short_escape() {
    assert_eq!(
        parse(r#""\"\\\/\b\f\n\r\t""#),
        Ok(("\"\\/\u{8}\u{c}\n\r\t".to_string(), 18))
    );
}

#[test]
fn u_escapes_upper_and_lower_hex() {
    assert_eq!(parse(r#""\u0041\u00e9\u00E9""#), Ok(("Aéé".to_string(), 20)));
    assert_eq!(parse(r#""\u0000""#), Ok(("\u{0}".to_string(), 8)));
}

#[test]
fn surrogate_pair_is_one_char() {
    assert_eq!(parse(r#""\uD83D\uDE00""#), Ok(("😀".to_string(), 14)));
    assert_eq!(parse(r#""\ud83d\ude00x""#), Ok(("😀x".to_string(), 15)));
}

#[test]
fn raw_non_ascii_is_kept() {
    assert_eq!(parse("\"é主\u{2028}\""), Ok(("é主\u{2028}".to_string(), 10)));
}

#[test]
fn lone_surrogates_are_invalid() {
    assert!(invalid(r#""\uDE00""#));
    assert!(invalid(r#""\uD83Dx""#));
    assert!(invalid(r#""\uD83D\n""#));
    assert!(invalid(r#""\uD83D\u0041""#));
    assert!(invalid(r#""\uD83D\uD83D""#));
}

#[test]
fn raw_control_chars_are_invalid() {
    assert!(invalid("\"a\nb\""));
    assert!(invalid("\"a\tb\""));
    assert!(invalid("\"\u{1f}\""));
}

#[test]
fn unknown_escape_and_bad_hex_are_invalid() {
    assert!(invalid(r#""\x""#));
    assert!(invalid(r#""\é""#));
    assert!(invalid(r#""\u00g0""#));
    assert!(invalid(r#""\u12""#));
}

#[test]
fn text_ending_inside_the_string_is_end_of_input() {
    for text in [
        "\"",
        "\"abc",
        "\"ab\\",
        "\"\\u",
        "\"\\u00",
        "\"\\uD83D",
        "\"\\uD83D\\",
        "\"\\uD83D\\u",
        "\"\\uD83D\\uDE",
        "\"\\uD83D\\uDE00",
    ] {
        assert_eq!(parse(text), Err(JsonError::EndOfInput), "{text:?}");
    }
}
