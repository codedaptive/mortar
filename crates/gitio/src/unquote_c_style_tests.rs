use crate::unquote_c_style::unquote_c_style;

fn unquote(line: &[u8]) -> Vec<u8> {
    unquote_c_style(line).unwrap()
}

#[test]
fn unquoted_lines_are_returned_as_they_are() {
    assert_eq!(unquote(b"d1"), b"d1".to_vec());
    assert_eq!(unquote(b"sp ace"), b"sp ace".to_vec());
    assert_eq!(unquote(b"-dash"), b"-dash".to_vec());
    assert_eq!(unquote(b"caf\xc3\xa9"), b"caf\xc3\xa9".to_vec());
}

#[test]
fn an_unquoted_line_keeps_a_backslash_or_inner_quote() {
    assert_eq!(unquote(b"a\\b\"c"), b"a\\b\"c".to_vec());
}

#[test]
fn captured_quoted_lines_unquote_to_their_names() {
    assert_eq!(unquote(b"\"back\\\\sl\""), b"back\\sl".to_vec());
    assert_eq!(unquote(b"\"caf\\303\\251\""), b"caf\xc3\xa9".to_vec());
    assert_eq!(unquote(b"\"cr\\rx\""), b"cr\rx".to_vec());
    assert_eq!(unquote(b"\"q\\\"uote\""), b"q\"uote".to_vec());
    assert_eq!(unquote(b"\"ta\\tb\""), b"ta\tb".to_vec());
}

#[test]
fn every_letter_escape_is_its_control_byte() {
    assert_eq!(unquote(b"\"\\a\\b\\t\\n\\v\\f\\r\""), vec![7, 8, 9, 10, 11, 12, 13]);
}

#[test]
fn octal_escapes_cover_the_whole_byte_range() {
    assert_eq!(unquote(b"\"\\000\\001\\177\\377\""), vec![0, 1, 127, 255]);
}

#[test]
fn quoted_empty_name_is_empty() {
    assert_eq!(unquote(b"\"\""), Vec::<u8>::new());
}

#[test]
fn missing_closing_quote_is_refused_naming_the_line() {
    let err = unquote_c_style(b"\"ta\\tb").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a C-quoted line: \"\\\"ta\\\\tb\"");
}

#[test]
fn a_lone_quote_is_refused() {
    assert!(unquote_c_style(b"\"").is_err());
}

#[test]
fn a_bare_quote_inside_is_refused() {
    assert!(unquote_c_style(b"\"a\"b\"").is_err());
}

#[test]
fn an_unknown_escape_is_refused() {
    assert!(unquote_c_style(b"\"a\\xb\"").is_err());
}

#[test]
fn a_trailing_backslash_is_refused() {
    assert!(unquote_c_style(b"\"a\\\"").is_err());
}

#[test]
fn octal_with_fewer_than_three_digits_is_refused() {
    assert!(unquote_c_style(b"\"\\30\"").is_err());
    assert!(unquote_c_style(b"\"\\3\"").is_err());
}

#[test]
fn octal_with_a_non_octal_digit_is_refused() {
    assert!(unquote_c_style(b"\"\\308\"").is_err());
}

#[test]
fn octal_above_0o377_is_refused() {
    assert!(unquote_c_style(b"\"\\400\"").is_err());
}
