//! JSON text to the members of a folder record's object, for the subset the record uses.

use super::{parse_json_string, JsonError, RecordJsonValue};

/// The members of the one JSON object in `text`, in text order with repeats kept, so the decoder can refuse
/// a repeated key. Whitespace is allowed around every token. A member's value is a string, null or an array
/// of strings; anything else (numbers, booleans, nested objects) is Invalid, naming the key unless the value
/// starts with 'n'. Text that ends
/// before the object is complete is EndOfInput, the mark of a torn write; anything else wrong, a value that
/// is not an object and trailing text included, is Invalid.
pub(crate) fn parse_record_json(text: &str) -> Result<Vec<(String, RecordJsonValue)>, JsonError> {
    let start = skip_ws(text, 0);
    let pos = expect(text, start, b'{', "where the record's '{' belongs: the record is not a JSON object")?;
    let (members, end) = object(text, pos)?;
    let rest = skip_ws(text, end);
    if rest < text.len() {
        return Err(invalid(text, rest, "after the record"));
    }
    Ok(members)
}

/// Members after the '{' at `pos - 1`, and the byte after the closing '}'.
fn object(text: &str, pos: usize) -> Result<(Vec<(String, RecordJsonValue)>, usize), JsonError> {
    let mut members = Vec::new();
    let mut pos = skip_ws(text, pos);
    if text.as_bytes().get(pos) == Some(&b'}') {
        return Ok((members, pos + 1));
    }
    loop {
        pos = expect(text, pos, b'"', "where a key belongs")?;
        let (key, end) = parse_json_string(text, pos)?;
        pos = expect(text, skip_ws(text, end), b':', "after a key")?;
        let (item, end) = value(text, skip_ws(text, pos), &key)?;
        members.push((key, item));
        pos = skip_ws(text, end);
        if text.as_bytes().get(pos) == Some(&b'}') {
            return Ok((members, pos + 1));
        }
        pos = skip_ws(text, expect(text, pos, b',', "after a member (',' or '}')")?);
    }
}

/// The value of member `key` at `pos`, and the byte after it.
fn value(text: &str, pos: usize, key: &str) -> Result<(RecordJsonValue, usize), JsonError> {
    match text.as_bytes().get(pos) {
        None => Err(JsonError::EndOfInput),
        Some(b'"') => parse_json_string(text, pos + 1).map(|(s, end)| (RecordJsonValue::String(s), end)),
        Some(b'[') => strings(text, pos + 1, key),
        Some(b'n') => null(text, pos),
        Some(_) => Err(invalid(
            text,
            pos,
            &format!("as the value of {key:?}, which must be a string, null or an array of strings"),
        )),
    }
}

/// String items after the '[' at `pos - 1`, and the byte after the closing ']'.
fn strings(text: &str, pos: usize, key: &str) -> Result<(RecordJsonValue, usize), JsonError> {
    let mut items = Vec::new();
    let mut pos = skip_ws(text, pos);
    if text.as_bytes().get(pos) == Some(&b']') {
        return Ok((RecordJsonValue::Strings(items), pos + 1));
    }
    loop {
        pos = expect(text, pos, b'"', &format!("in the array {key:?}, where a string belongs"))?;
        let (item, end) = parse_json_string(text, pos)?;
        items.push(item);
        pos = skip_ws(text, end);
        if text.as_bytes().get(pos) == Some(&b']') {
            return Ok((RecordJsonValue::Strings(items), pos + 1));
        }
        pos = skip_ws(text, expect(text, pos, b',', "after an item (',' or ']')")?);
    }
}

/// `null`; a text that stops partway through the word is EndOfInput.
fn null(text: &str, pos: usize) -> Result<(RecordJsonValue, usize), JsonError> {
    let rest = &text[pos..];
    if rest.starts_with("null") {
        return Ok((RecordJsonValue::Null, pos + 4));
    }
    if "null".starts_with(rest) {
        return Err(JsonError::EndOfInput);
    }
    Err(invalid(text, pos, "where null was begun"))
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
