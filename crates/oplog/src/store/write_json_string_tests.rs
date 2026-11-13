//! write_json_string: quotes, the escapes JSON requires, and every other char as is.

use super::write_json_string;

fn json(s: &str) -> String {
    let mut out = String::new();
    write_json_string(&mut out, s);
    out
}

#[test]
fn plain_text_is_quoted() {
    assert_eq!(json("refs/heads/main"), "\"refs/heads/main\"");
    assert_eq!(json(""), "\"\"");
}

#[test]
fn quote_and_backslash_are_escaped() {
    assert_eq!(json("a\"b\\c"), r#""a\"b\\c""#);
}

#[test]
fn short_form_escapes() {
    assert_eq!(json("\n\r\t\u{8}\u{c}"), r#""\n\r\t\b\f""#);
}

#[test]
fn other_control_chars_are_u_escapes() {
    assert_eq!(json("\u{0}\u{1}\u{1b}\u{1f}"), r#""\u0000\u0001\u001b\u001f""#);
}

#[test]
fn space_slash_delete_and_non_ascii_are_written_as_is() {
    assert_eq!(json(" /\u{7f}"), "\" /\u{7f}\"");
    assert_eq!(json("é主😀\u{2028}\u{2029}"), "\"é主😀\u{2028}\u{2029}\"");
}

#[test]
fn appends_to_what_is_there() {
    let mut out = String::from("x:");
    write_json_string(&mut out, "y");
    assert_eq!(out, "x:\"y\"");
}
