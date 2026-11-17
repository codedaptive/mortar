//! Parses one JSON value: what the drive reads from `mr ... --json` (the target links no Mortar crate and cli
//! has no JSON library).

use super::{json_error, parse_json_string, JsonValue};

/// One value with whitespace around it; anything after it is an error. Strings take the standard escapes;
/// numbers follow JSON's grammar and keep their text.
pub fn parse_json(text: &str) -> Result<JsonValue, String> {
    let mut at = 0;
    let value = parse_value(text, &mut at)?;
    skip_space(text, &mut at);
    if at < text.len() {
        return Err(json_error(text, at, "trailing text"));
    }
    Ok(value)
}

fn peek(text: &str, at: usize) -> Option<u8> {
    text.as_bytes().get(at).copied()
}

fn skip_space(text: &str, at: &mut usize) {
    while matches!(peek(text, *at), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        *at += 1;
    }
}

fn parse_value(text: &str, at: &mut usize) -> Result<JsonValue, String> {
    skip_space(text, at);
    match peek(text, *at) {
        Some(b'{') => parse_object(text, at),
        Some(b'[') => parse_array(text, at),
        Some(b'"') => parse_json_string(text, at).map(JsonValue::Str),
        Some(b't') => parse_word(text, at, "true", JsonValue::Bool(true)),
        Some(b'f') => parse_word(text, at, "false", JsonValue::Bool(false)),
        Some(b'n') => parse_word(text, at, "null", JsonValue::Null),
        Some(b'-' | b'0'..=b'9') => parse_number(text, at),
        _ => Err(json_error(text, *at, "expected a value")),
    }
}

fn parse_word(text: &str, at: &mut usize, word: &str, value: JsonValue) -> Result<JsonValue, String> {
    if !text[*at..].starts_with(word) {
        return Err(json_error(text, *at, &format!("expected {word}")));
    }
    *at += word.len();
    Ok(value)
}

/// `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`
fn parse_number(text: &str, at: &mut usize) -> Result<JsonValue, String> {
    let start = *at;
    if peek(text, *at) == Some(b'-') {
        *at += 1;
    }
    if peek(text, *at) == Some(b'0') {
        *at += 1;
    } else {
        digits(text, at)?;
    }
    if peek(text, *at) == Some(b'.') {
        *at += 1;
        digits(text, at)?;
    }
    if matches!(peek(text, *at), Some(b'e' | b'E')) {
        *at += 1;
        if matches!(peek(text, *at), Some(b'+' | b'-')) {
            *at += 1;
        }
        digits(text, at)?;
    }
    Ok(JsonValue::Number(text[start..*at].to_string()))
}

/// One or more digits.
fn digits(text: &str, at: &mut usize) -> Result<(), String> {
    let start = *at;
    while matches!(peek(text, *at), Some(b'0'..=b'9')) {
        *at += 1;
    }
    if *at == start {
        return Err(json_error(text, *at, "expected a digit"));
    }
    Ok(())
}

fn parse_array(text: &str, at: &mut usize) -> Result<JsonValue, String> {
    *at += 1;
    let mut items = Vec::new();
    skip_space(text, at);
    if peek(text, *at) == Some(b']') {
        *at += 1;
        return Ok(JsonValue::Array(items));
    }
    loop {
        items.push(parse_value(text, at)?);
        if end_or_comma(text, at, b']')? {
            return Ok(JsonValue::Array(items));
        }
    }
}

fn parse_object(text: &str, at: &mut usize) -> Result<JsonValue, String> {
    *at += 1;
    let mut members = Vec::new();
    skip_space(text, at);
    if peek(text, *at) == Some(b'}') {
        *at += 1;
        return Ok(JsonValue::Object(members));
    }
    loop {
        skip_space(text, at);
        let key = parse_json_string(text, at)?;
        skip_space(text, at);
        if peek(text, *at) != Some(b':') {
            return Err(json_error(text, *at, "expected ':'"));
        }
        *at += 1;
        members.push((key, parse_value(text, at)?));
        if end_or_comma(text, at, b'}')? {
            return Ok(JsonValue::Object(members));
        }
    }
}

/// After an item: true past the closing `close`, false past a `,` (another item follows), else an error.
fn end_or_comma(text: &str, at: &mut usize, close: u8) -> Result<bool, String> {
    skip_space(text, at);
    let ended = match peek(text, *at) {
        Some(b',') => false,
        Some(byte) if byte == close => true,
        _ => return Err(json_error(text, *at, &format!("expected ',' or '{}'", char::from(close)))),
    };
    *at += 1;
    Ok(ended)
}
