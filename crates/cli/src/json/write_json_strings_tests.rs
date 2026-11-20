//! write_json_strings: a bracketed, comma-separated array of quoted strings, appended.

use super::write_json_strings;

fn written(items: &[&str]) -> String {
    let mut out = String::from(">");
    write_json_strings(&mut out, &items.iter().map(|s| s.to_string()).collect::<Vec<_>>());
    out
}

#[test]
fn empty_is_brackets() {
    assert_eq!(written(&[]), ">[]");
}

#[test]
fn items_are_quoted_and_escaped_in_order() {
    assert_eq!(written(&["b", "a\"x", "", "t\tu"]), r#">["b","a\"x","","t\tu"]"#);
}
