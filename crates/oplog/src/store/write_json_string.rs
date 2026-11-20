//! A string as quoted JSON text.

use std::fmt::Write;

/// Appends `s` to `out` in double quotes. Escapes '"', '\' and every char below U+0020 (\n \r \t \b \f
/// in their short forms, the rest as \u00XX); every other char, non-ASCII included, is written as is,
/// so a record stays on one line whatever its strings hold.
pub fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            // Writing to a String cannot fail.
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
