//! A JSON parse error's text. parse_json_tests.rs covers it through parse_json's errors.

/// `<what> at byte <n>`, or `<what> at the end of the text` when the text ran out there.
pub fn json_error(text: &str, at: usize, what: &str) -> String {
    if at < text.len() {
        format!("{what} at byte {at}")
    } else {
        format!("{what} at the end of the text")
    }
}
