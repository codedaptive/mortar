//! A JSON string at a position in the text. parse_json_tests.rs covers it through parse_json.

use super::json_error;

/// Reads the string starting at the `"` at `*at`, with the standard escapes (`\uXXXX` included, a surrogate pair
/// joined into one character), and moves `*at` past its closing `"`.
pub fn parse_json_string(text: &str, at: &mut usize) -> Result<String, String> {
    if text.as_bytes().get(*at) != Some(&b'"') {
        return Err(json_error(text, *at, "expected '\"'"));
    }
    *at += 1;
    let mut out = String::new();
    loop {
        let c = text[*at..].chars().next().ok_or_else(|| json_error(text, *at, "unterminated string"))?;
        *at += c.len_utf8();
        match c {
            '"' => return Ok(out),
            '\\' => out.push(escape(text, at)?),
            c if u32::from(c) < 0x20 => return Err(json_error(text, *at - 1, "control character in a string")),
            c => out.push(c),
        }
    }
}

fn escape(text: &str, at: &mut usize) -> Result<char, String> {
    let byte = text.as_bytes().get(*at).copied().ok_or_else(|| json_error(text, *at, "unterminated escape"))?;
    *at += 1;
    match byte {
        b'"' => Ok('"'),
        b'\\' => Ok('\\'),
        b'/' => Ok('/'),
        b'b' => Ok('\u{8}'),
        b'f' => Ok('\u{c}'),
        b'n' => Ok('\n'),
        b'r' => Ok('\r'),
        b't' => Ok('\t'),
        b'u' => unicode(text, at),
        _ => Err(json_error(text, *at - 1, "unknown escape")),
    }
}

/// A high surrogate (D800-DBFF) must be followed by `\u` and a low one (DC00-DFFF); either alone is an error.
fn unicode(text: &str, at: &mut usize) -> Result<char, String> {
    let high = hex4(text, at)?;
    if !(0xD800..0xDC00).contains(&high) {
        return char::from_u32(high).ok_or_else(|| json_error(text, *at, "lone low surrogate"));
    }
    if !text[*at..].starts_with("\\u") {
        return Err(json_error(text, *at, "lone high surrogate"));
    }
    *at += 2;
    let low = hex4(text, at)?;
    if !(0xDC00..0xE000).contains(&low) {
        return Err(json_error(text, *at, "lone high surrogate"));
    }
    char::from_u32(0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00))
        .ok_or_else(|| json_error(text, *at, "bad surrogate pair"))
}

fn hex4(text: &str, at: &mut usize) -> Result<u32, String> {
    let digits = text
        .get(*at..*at + 4)
        .filter(|digits| digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| json_error(text, *at, "expected four hex digits"))?;
    *at += 4;
    u32::from_str_radix(digits, 16).map_err(|error| json_error(text, *at, &error.to_string()))
}
