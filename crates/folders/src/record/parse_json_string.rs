//! The body of a JSON string, with every JSON escape.

use super::JsonError;

/// Reads a string whose opening quote sits just before byte `start` of `text`. Returns the string and the
/// byte after its closing quote. A raw control char, an unknown escape or a lone surrogate is Invalid;
/// text that ends inside the string, a `\u` escape or a surrogate pair is EndOfInput.
pub(crate) fn parse_json_string(text: &str, start: usize) -> Result<(String, usize), JsonError> {
    let bytes = text.as_bytes();
    let mut out = String::new();
    let mut run = start;
    let mut pos = start;
    loop {
        match bytes.get(pos) {
            None => return Err(JsonError::EndOfInput),
            Some(b'"') => {
                out.push_str(&text[run..pos]);
                return Ok((out, pos + 1));
            }
            Some(b'\\') => {
                out.push_str(&text[run..pos]);
                let (c, next) = escape(text, pos + 1)?;
                out.push(c);
                pos = next;
                run = next;
            }
            Some(&b) if b < 0x20 => {
                return Err(JsonError::Invalid(format!(
                    "raw control char U+{b:04X} in a string at byte {pos}; it must be escaped"
                )))
            }
            Some(_) => pos += 1,
        }
    }
}

/// The char an escape stands for, `pos` being the byte after the backslash, and the byte after the escape.
fn escape(text: &str, pos: usize) -> Result<(char, usize), JsonError> {
    let c = match text.as_bytes().get(pos) {
        None => return Err(JsonError::EndOfInput),
        Some(b'u') => return unicode_escape(text.as_bytes(), pos + 1),
        Some(b'"') => '"',
        Some(b'\\') => '\\',
        Some(b'/') => '/',
        Some(b'b') => '\u{8}',
        Some(b'f') => '\u{c}',
        Some(b'n') => '\n',
        Some(b'r') => '\r',
        Some(b't') => '\t',
        Some(_) => {
            let found = text[pos..].chars().next().unwrap_or_default();
            return Err(JsonError::Invalid(format!("unknown escape \\{found} at byte {pos}")));
        }
    };
    Ok((c, pos + 1))
}

/// A `\u` escape whose four hex digits start at `pos`; a high surrogate must be followed by `\u` and a low one.
fn unicode_escape(bytes: &[u8], pos: usize) -> Result<(char, usize), JsonError> {
    let (unit, next) = hex4(bytes, pos)?;
    let (code, after) = match unit {
        0xD800..=0xDBFF => {
            match (bytes.get(next), bytes.get(next + 1)) {
                (None, _) | (Some(b'\\'), None) => return Err(JsonError::EndOfInput),
                (Some(b'\\'), Some(b'u')) => {}
                _ => return Err(lone(unit, pos)),
            }
            let (low, after) = hex4(bytes, next + 2)?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(lone(unit, pos));
            }
            (0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00), after)
        }
        0xDC00..=0xDFFF => return Err(lone(unit, pos)),
        other => (other, next),
    };
    // Every code reaching here is a scalar value, so from_u32 always succeeds.
    char::from_u32(code)
        .map(|c| (c, after))
        .ok_or_else(|| lone(unit, pos))
}

/// Four hex digits from `pos` as a number, and the byte after them.
fn hex4(bytes: &[u8], pos: usize) -> Result<(u32, usize), JsonError> {
    let mut unit = 0;
    for at in pos..pos + 4 {
        let b = *bytes.get(at).ok_or(JsonError::EndOfInput)?;
        let digit = char::from(b).to_digit(16).ok_or_else(|| {
            JsonError::Invalid(format!("\\u escape needs 4 hex digits; byte {at} is not one"))
        })?;
        unit = unit * 16 + digit;
    }
    Ok((unit, pos + 4))
}

fn lone(unit: u32, pos: usize) -> JsonError {
    JsonError::Invalid(format!(
        "\\u{unit:04X} at byte {pos} is a surrogate without its pair"
    ))
}
