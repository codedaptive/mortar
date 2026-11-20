//! One top-level boolean member of a JSON object.
//! oplog::parse_json refuses `true` and `false`, so it cannot read the user's `{"savings_line": false}`: this
//! checks the whole text as RFC 8259 JSON and keeps only the booleans of the top-level members.

use super::json_string;

/// Err(reason) unless `text` is exactly one JSON object, with optional whitespace around it. Ok(Some(b)) is the
/// value of the last top-level member named `key` (escapes decoded) whose value is a boolean; Ok(None) when no
/// top-level member of that name holds a boolean. Members of nested objects do not count.
pub fn top_level_bool(text: &str, key: &str) -> Result<Option<bool>, String> {
    let start = skip(text, 0);
    if text.as_bytes().get(start) != Some(&b'{') {
        return Err(fail(start));
    }
    let (found, end) = object(text, start, Some(key))?;
    match skip(text, end) {
        end if end == text.len() => Ok(found),
        end => Err(fail(end)),
    }
}

fn fail(at: usize) -> String {
    format!("not a JSON object: unexpected text at byte {at}")
}

/// The first byte at or after `at` that is not JSON whitespace.
fn skip(text: &str, at: usize) -> usize {
    let rest = text.as_bytes().get(at..).unwrap_or_default();
    at + rest.iter().take_while(|c| matches!(c, b' ' | b'\t' | b'\n' | b'\r')).count()
}

fn expect(text: &str, at: usize, byte: u8) -> Result<usize, String> {
    match text.as_bytes().get(at) {
        Some(c) if *c == byte => Ok(skip(text, at + 1)),
        _ => Err(fail(at)),
    }
}

/// The value at `at`: Some(b) when it is a boolean, and the byte after it.
fn value(text: &str, at: usize) -> Result<(Option<bool>, usize), String> {
    let literal = |word: &str, b: Option<bool>| match text.get(at..).is_some_and(|rest| rest.starts_with(word)) {
        true => Ok((b, at + word.len())),
        false => Err(fail(at)),
    };
    match text.as_bytes().get(at) {
        Some(b'{') => object(text, at, None).map(|(_, end)| (None, end)),
        Some(b'[') => array(text, at).map(|end| (None, end)),
        Some(b'"') => json_string(text, at).map(|(_, end)| (None, end)).map_err(fail),
        Some(b't') => literal("true", Some(true)),
        Some(b'f') => literal("false", Some(false)),
        Some(b'n') => literal("null", None),
        Some(b'-' | b'0'..=b'9') => number(text.as_bytes(), at).map(|end| (None, end)),
        _ => Err(fail(at)),
    }
}

/// The object opening at `at`: the boolean of its last member named `key` (None for nested objects), and the byte
/// after its closing brace.
fn object(text: &str, at: usize, key: Option<&str>) -> Result<(Option<bool>, usize), String> {
    let mut at = skip(text, at + 1);
    let mut found = None;
    if text.as_bytes().get(at) == Some(&b'}') {
        return Ok((found, at + 1));
    }
    loop {
        let (name, after) = json_string(text, at).map_err(fail)?;
        let (member, after) = value(text, expect(text, skip(text, after), b':')?)?;
        if key == Some(name.as_str()) && member.is_some() {
            found = member;
        }
        at = skip(text, after);
        match text.as_bytes().get(at) {
            Some(b',') => at = skip(text, at + 1),
            Some(b'}') => return Ok((found, at + 1)),
            _ => return Err(fail(at)),
        }
    }
}

fn array(text: &str, at: usize) -> Result<usize, String> {
    let mut at = skip(text, at + 1);
    if text.as_bytes().get(at) == Some(&b']') {
        return Ok(at + 1);
    }
    loop {
        at = skip(text, value(text, at)?.1);
        match text.as_bytes().get(at) {
            Some(b',') => at = skip(text, at + 1),
            Some(b']') => return Ok(at + 1),
            _ => return Err(fail(at)),
        }
    }
}

/// The number at `at`: an optional minus, 0 or digits without a leading 0, an optional fraction, an optional
/// exponent; the byte after it.
fn number(bytes: &[u8], at: usize) -> Result<usize, String> {
    let at = at + usize::from(bytes.get(at) == Some(&b'-'));
    let mut at = match bytes.get(at) {
        Some(b'0') => at + 1,
        Some(b'1'..=b'9') => digits(bytes, at)?,
        _ => return Err(fail(at)),
    };
    if bytes.get(at) == Some(&b'.') {
        at = digits(bytes, at + 1)?;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at = digits(bytes, at + 1 + usize::from(matches!(bytes.get(at + 1), Some(b'+' | b'-'))))?;
    }
    Ok(at)
}

/// The byte after the one or more digits starting at `at`.
fn digits(bytes: &[u8], at: usize) -> Result<usize, String> {
    match bytes.get(at..).unwrap_or_default().iter().take_while(|c| c.is_ascii_digit()).count() {
        0 => Err(fail(at)),
        n => Ok(at + n),
    }
}
