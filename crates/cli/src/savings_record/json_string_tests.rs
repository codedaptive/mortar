//! json_string: one RFC 8259 string decoded, with the byte after it; Err at the byte that breaks it.

use super::json_string;

#[test]
fn a_plain_string_and_the_byte_after() {
    assert_eq!(json_string(r#"x"abc", "#, 1), Ok(("abc".to_string(), 6)));
    assert_eq!(json_string(r#""""#, 0), Ok((String::new(), 2)));
}

#[test]
fn every_short_escape() {
    assert_eq!(json_string(r#""\"\\\/\b\f\n\r\t""#, 0), Ok(("\"\\/\u{8}\u{c}\n\r\t".to_string(), 18)));
}

#[test]
fn unicode_escapes_and_raw_non_ascii() {
    assert_eq!(json_string(r#""\u00e9\u00E9é""#, 0), Ok(("ééé".to_string(), 16)));
    assert_eq!(json_string(r#""\ud83c\udf31""#, 0), Ok(("🌱".to_string(), 14)));
}

#[test]
fn a_lone_surrogate_is_the_replacement_char() {
    assert_eq!(json_string(r#""\ud83cx""#, 0), Ok(("\u{FFFD}x".to_string(), 9)));
    assert_eq!(json_string(r#""\udf31""#, 0), Ok(("\u{FFFD}".to_string(), 8)));
}

#[test]
fn errors_name_the_byte() {
    assert_eq!(json_string("abc", 0), Err(0));
    assert_eq!(json_string(r#""ab"#, 0), Err(3));
    assert_eq!(json_string(r#""a\x""#, 0), Err(3));
    assert_eq!(json_string(r#""\u12g4""#, 0), Err(3));
    assert_eq!(json_string("\"a\nb\"", 0), Err(2));
}
