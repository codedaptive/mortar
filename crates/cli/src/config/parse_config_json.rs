//! JSON text to a ConfigJsonValue, for the subset the config uses.
//! A stopgap (see config_json_value.rs): its number, null and error rules match oplog's crate-private
//! parse_json, and strings are read by oplog::parse_json_string (S-51) until oplog exports parse_json and
//! JsonValue (S-80, WO-oplog-11).

use oplog::{parse_json_string, JsonError};

use super::ConfigJsonValue;

/// One JSON value with optional whitespace around every token: objects, arrays, strings, null and
/// non-negative integers (no sign, fraction, exponent or leading zero; must fit u64). Booleans are refused.
/// Text that ends before the value is complete is EndOfInput, the mark of a torn write; anything else
/// wrong, trailing text included, is Invalid.
pub(crate) fn parse_config_json(text: &str) -> Result<ConfigJsonValue, JsonError> {
    let (value, end) = value(text, skip_ws(text, 0))?;
    let rest = skip_ws(text, end);
    if rest < text.len() {
        return Err(invalid(text, rest, "after the value"));
    }
    Ok(value)
}

fn value(text: &str, pos: usize) -> Result<(ConfigJsonValue, usize), JsonError> {
    match text.as_bytes().get(pos) {
        None => Err(JsonError::EndOfInput),
        Some(b'{') => object(text, pos + 1),
        Some(b'[') => array(text, pos + 1),
        Some(b'"') => parse_json_string(text, pos + 1).map(|(s, end)| (ConfigJsonValue::String(s), end)),
        Some(b'n') => null(text, pos),
        Some(b'0'..=b'9') => number(text, pos),
        Some(_) => Err(invalid(text, pos, "where a value (object, array, string, null or non-negative integer) belongs")),
    }
}

/// Members after the '{' at `pos - 1`, up to and including the closing '}'.
fn object(text: &str, pos: usize) -> Result<(ConfigJsonValue, usize), JsonError> {
    let mut pairs = Vec::new();
    let mut pos = skip_ws(text, pos);
    if text.as_bytes().get(pos) == Some(&b'}') {
        return Ok((ConfigJsonValue::Object(pairs), pos + 1));
    }
    loop {
        pos = expect(text, pos, b'"', "where a key belongs")?;
        let (key, end) = parse_json_string(text, pos)?;
        pos = expect(text, skip_ws(text, end), b':', "after a key")?;
        let (item, end) = value(text, skip_ws(text, pos))?;
        pairs.push((key, item));
        pos = skip_ws(text, end);
        if text.as_bytes().get(pos) == Some(&b'}') {
            return Ok((ConfigJsonValue::Object(pairs), pos + 1));
        }
        pos = skip_ws(text, expect(text, pos, b',', "after a member (',' or '}')")?);
    }
}

/// Items after the '[' at `pos - 1`, up to and including the closing ']'.
fn array(text: &str, pos: usize) -> Result<(ConfigJsonValue, usize), JsonError> {
    let mut items = Vec::new();
    let mut pos = skip_ws(text, pos);
    if text.as_bytes().get(pos) == Some(&b']') {
        return Ok((ConfigJsonValue::Array(items), pos + 1));
    }
    loop {
        let (item, end) = value(text, pos)?;
        items.push(item);
        pos = skip_ws(text, end);
        if text.as_bytes().get(pos) == Some(&b']') {
            return Ok((ConfigJsonValue::Array(items), pos + 1));
        }
        pos = skip_ws(text, expect(text, pos, b',', "after an item (',' or ']')")?);
    }
}

/// `null`; a text that stops partway through the word is EndOfInput.
fn null(text: &str, pos: usize) -> Result<(ConfigJsonValue, usize), JsonError> {
    let rest = &text[pos..];
    if rest.starts_with("null") {
        return Ok((ConfigJsonValue::Null, pos + 4));
    }
    if "null".starts_with(rest) {
        return Err(JsonError::EndOfInput);
    }
    Err(JsonError::Invalid(format!("{rest:?} at byte {pos} is not null")))
}

fn number(text: &str, pos: usize) -> Result<(ConfigJsonValue, usize), JsonError> {
    let end = text.as_bytes()[pos..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map_or(text.len(), |n| pos + n);
    let digits = &text[pos..end];
    if digits.len() > 1 && digits.starts_with('0') {
        return Err(JsonError::Invalid(format!("number {digits} at byte {pos} has a leading zero")));
    }
    if matches!(text.as_bytes().get(end), Some(b'.' | b'e' | b'E')) {
        return Err(JsonError::Invalid(format!(
            "number at byte {pos} has a fraction or exponent; only integers are allowed"
        )));
    }
    let n = digits.parse().map_err(|_| {
        JsonError::Invalid(format!("number {digits} at byte {pos} does not fit 64 bits"))
    })?;
    Ok((ConfigJsonValue::Number(n), end))
}

/// The byte after `want` at `pos`; EndOfInput when the text stops there.
fn expect(text: &str, pos: usize, want: u8, place: &str) -> Result<usize, JsonError> {
    match text.as_bytes().get(pos) {
        None => Err(JsonError::EndOfInput),
        Some(&b) if b == want => Ok(pos + 1),
        Some(_) => Err(invalid(text, pos, place)),
    }
}

fn skip_ws(text: &str, pos: usize) -> usize {
    text.as_bytes()[pos..]
        .iter()
        .position(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .map_or(text.len(), |n| pos + n)
}

/// "unexpected 'c' at byte N <place>"; `pos` is always a char boundary, since only ASCII is stepped over.
fn invalid(text: &str, pos: usize, place: &str) -> JsonError {
    let found = text[pos..].chars().next().unwrap_or_default();
    JsonError::Invalid(format!("unexpected {found:?} at byte {pos} {place}"))
}
