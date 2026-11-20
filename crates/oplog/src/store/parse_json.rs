//! JSON text to a JsonValue, for the JSON subset Mortar's records use.

use super::{parse_json_string, JsonError, JsonValue};

/// One JSON value with optional whitespace around it: objects, arrays, strings, null, booleans and
/// non-negative integers that fit u64. Negative numbers, fractions, exponents and leading zeros are
/// refused. An object keeps its keys in text order, duplicates included. Text that ends before the
/// value is complete is EndOfInput, the mark of a torn write; anything else wrong, trailing text
/// included, is Invalid.
pub fn parse_json(text: &str) -> Result<JsonValue, JsonError> {
    let (value, end) = value(text, skip_ws(text, 0))?;
    let rest = skip_ws(text, end);
    if rest < text.len() {
        return Err(invalid(text, rest, "after the value"));
    }
    Ok(value)
}

fn value(text: &str, pos: usize) -> Result<(JsonValue, usize), JsonError> {
    match text.as_bytes().get(pos) {
        None => Err(JsonError::EndOfInput),
        Some(b'{') => object(text, pos + 1),
        Some(b'[') => array(text, pos + 1),
        Some(b'"') => parse_json_string(text, pos + 1).map(|(s, end)| (JsonValue::String(s), end)),
        Some(b'n') => literal(text, pos, "null", JsonValue::Null),
        Some(b't') => literal(text, pos, "true", JsonValue::Bool(true)),
        Some(b'f') => literal(text, pos, "false", JsonValue::Bool(false)),
        Some(b'0'..=b'9') => number(text, pos),
        Some(_) => Err(invalid(text, pos, "where a value (object, array, string, null, boolean or non-negative integer) belongs")),
    }
}

/// Members after the '{' at `pos - 1`, up to and including the closing '}'.
fn object(text: &str, pos: usize) -> Result<(JsonValue, usize), JsonError> {
    let mut pairs = Vec::new();
    let mut pos = skip_ws(text, pos);
    if text.as_bytes().get(pos) == Some(&b'}') {
        return Ok((JsonValue::Object(pairs), pos + 1));
    }
    loop {
        pos = expect(text, pos, b'"', "where a key belongs")?;
        let (key, end) = parse_json_string(text, pos)?;
        pos = expect(text, skip_ws(text, end), b':', "after a key")?;
        let (item, end) = value(text, skip_ws(text, pos))?;
        pairs.push((key, item));
        pos = skip_ws(text, end);
        if text.as_bytes().get(pos) == Some(&b'}') {
            return Ok((JsonValue::Object(pairs), pos + 1));
        }
        pos = skip_ws(text, expect(text, pos, b',', "after a member (',' or '}')")?);
    }
}

/// Items after the '[' at `pos - 1`, up to and including the closing ']'.
fn array(text: &str, pos: usize) -> Result<(JsonValue, usize), JsonError> {
    let mut items = Vec::new();
    let mut pos = skip_ws(text, pos);
    if text.as_bytes().get(pos) == Some(&b']') {
        return Ok((JsonValue::Array(items), pos + 1));
    }
    loop {
        let (item, end) = value(text, pos)?;
        items.push(item);
        pos = skip_ws(text, end);
        if text.as_bytes().get(pos) == Some(&b']') {
            return Ok((JsonValue::Array(items), pos + 1));
        }
        pos = skip_ws(text, expect(text, pos, b',', "after an item (',' or ']')")?);
    }
}

/// `word` (null, true or false) as `value`; a text that stops partway through the word is EndOfInput.
fn literal(text: &str, pos: usize, word: &str, value: JsonValue) -> Result<(JsonValue, usize), JsonError> {
    let rest = &text[pos..];
    if rest.starts_with(word) {
        return Ok((value, pos + word.len()));
    }
    if word.starts_with(rest) {
        return Err(JsonError::EndOfInput);
    }
    Err(JsonError::Invalid(format!("{rest:?} at byte {pos} is not {word}")))
}

fn number(text: &str, pos: usize) -> Result<(JsonValue, usize), JsonError> {
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
    Ok((JsonValue::Number(n), end))
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
