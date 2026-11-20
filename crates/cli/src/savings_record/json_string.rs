//! One JSON string, decoded: the string reader top_level_bool uses for member names and string values.

/// The RFC 8259 string whose opening quote is at byte `at` of `text`, its escapes decoded, and the byte after its
/// closing quote. Err(byte) at the first byte that breaks the grammar: no opening quote, a raw control character,
/// a bad escape, or no closing quote.
pub fn json_string(text: &str, at: usize) -> Result<(String, usize), usize> {
    let bytes = text.as_bytes();
    if bytes.get(at) != Some(&b'"') {
        return Err(at);
    }
    let mut out = String::new();
    let mut at = at + 1;
    loop {
        let start = at;
        at += bytes.get(at..).unwrap_or_default().iter().take_while(|c| **c != b'"' && **c != b'\\' && **c >= 0x20).count();
        // The run stops at an ASCII byte or the end, both char boundaries.
        out.push_str(&text[start..at]);
        match bytes.get(at) {
            Some(b'"') => return Ok((out, at + 1)),
            Some(b'\\') => at = escape(bytes, at + 1, &mut out)?,
            _ => return Err(at),
        }
    }
}

fn escape(bytes: &[u8], at: usize, out: &mut String) -> Result<usize, usize> {
    let c = match bytes.get(at) {
        Some(b'"') => '"',
        Some(b'\\') => '\\',
        Some(b'/') => '/',
        Some(b'b') => '\u{8}',
        Some(b'f') => '\u{c}',
        Some(b'n') => '\n',
        Some(b'r') => '\r',
        Some(b't') => '\t',
        Some(b'u') => return unicode(bytes, at + 1, out),
        _ => return Err(at),
    };
    out.push(c);
    Ok(at + 1)
}

/// The \uXXXX escape whose hex digits start at `at`, joined with a following low surrogate escape when it is a high
/// surrogate. A lone surrogate is valid JSON but no char: it decodes as U+FFFD.
fn unicode(bytes: &[u8], at: usize, out: &mut String) -> Result<usize, usize> {
    let high = hex4(bytes, at).ok_or(at)?;
    let low = match (0xD800..0xDC00).contains(&high) && bytes.get(at + 4..at + 6) == Some(b"\\u") {
        true => hex4(bytes, at + 6).filter(|low| (0xDC00..0xE000).contains(low)),
        false => None,
    };
    let (code, end) = match low {
        Some(low) => (0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00), at + 10),
        None => (high, at + 4),
    };
    out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
    Ok(end)
}

fn hex4(bytes: &[u8], at: usize) -> Option<u32> {
    bytes.get(at..at + 4)?.iter().try_fold(0, |acc, c| char::from(*c).to_digit(16).map(|d| acc * 16 + d))
}
