//! parse_config_json: objects, arrays, strings, null and u64 integers; booleans, signs, fractions and
//! trailing text refused; text ending early is EndOfInput.

use oplog::JsonError;

use super::{parse_config_json, ConfigJsonValue as V};

fn s(text: &str) -> V {
    V::String(text.to_string())
}

fn invalid(text: &str) -> String {
    match parse_config_json(text) {
        Err(JsonError::Invalid(reason)) => reason,
        other => panic!("{text:?} gave {other:?}"),
    }
}

#[test]
fn every_value_kind_nested_with_members_in_text_order_and_repeats_kept() {
    assert_eq!(
        parse_config_json(r#"{"b":[1,"x",null,[],{}],"a":{"k":"v"},"b":0}"#),
        Ok(V::Object(vec![
            ("b".to_string(), V::Array(vec![V::Number(1), s("x"), V::Null, V::Array(Vec::new()), V::Object(Vec::new())])),
            ("a".to_string(), V::Object(vec![("k".to_string(), s("v"))])),
            ("b".to_string(), V::Number(0)),
        ]))
    );
}

#[test]
fn whitespace_around_every_token() {
    assert_eq!(
        parse_config_json(" \n{ \"a\" :\t[ 7 , \"\\u00e9\" ] }\r\n"),
        Ok(V::Object(vec![("a".to_string(), V::Array(vec![V::Number(7), s("é")]))]))
    );
}

#[test]
fn a_top_level_scalar_is_a_value() {
    assert_eq!(parse_config_json("18446744073709551615"), Ok(V::Number(u64::MAX)));
    assert_eq!(parse_config_json("null"), Ok(V::Null));
    assert_eq!(parse_config_json(r#""a\nb""#), Ok(s("a\nb")));
}

#[test]
fn numbers_out_of_the_subset_are_invalid() {
    assert_eq!(invalid("012"), "number 012 at byte 0 has a leading zero");
    assert_eq!(invalid("[1.5]"), "number at byte 1 has a fraction or exponent; only integers are allowed");
    assert_eq!(invalid("2e3"), "number at byte 0 has a fraction or exponent; only integers are allowed");
    assert_eq!(invalid("18446744073709551616"), "number 18446744073709551616 at byte 0 does not fit 64 bits");
    assert_eq!(
        invalid("-1"),
        "unexpected '-' at byte 0 where a value (object, array, string, null or non-negative integer) belongs"
    );
}

#[test]
fn booleans_are_refused() {
    for text in ["true", "{\"a\":false}"] {
        assert!(invalid(text).contains("where a value (object, array, string, null or non-negative integer) belongs"), "{text}");
    }
}

#[test]
fn trailing_text_and_bad_separators_are_invalid() {
    assert_eq!(invalid("{} x"), "unexpected 'x' at byte 3 after the value");
    assert_eq!(invalid("{\"a\" 1}"), "unexpected '1' at byte 5 after a key");
    assert_eq!(invalid("[1 2]"), "unexpected '2' at byte 3 after an item (',' or ']')");
    assert_eq!(invalid("{\"a\":1 \"b\":2}"), "unexpected '\"' at byte 7 after a member (',' or '}')");
    assert_eq!(invalid("{1:2}"), "unexpected '1' at byte 1 where a key belongs");
    assert_eq!(invalid("nil"), "\"nil\" at byte 0 is not null");
}

#[test]
fn text_ending_early_is_end_of_input() {
    for text in ["", "  ", "{", "{\"a\"", "{\"a\":", "{\"a\":[1,", "[\"ab", "nu", "{\"a\":1"] {
        assert_eq!(parse_config_json(text), Err(JsonError::EndOfInput), "{text:?}");
    }
}
