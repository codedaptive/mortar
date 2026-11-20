//! parse_guard_config: guard.json's text back to the branches it names.

use oplog::{parse_json_string, JsonError};

use crate::LandRefs;

/// The three keys guard.json holds, each exactly once.
const KEYS: [&str; 3] = ["origin_branch", "integration_branch", "remote"];

/// Reads a JSON object holding exactly origin_branch, integration_branch and remote with string values, in any
/// order and with JSON whitespace around its tokens (so a trailing newline is fine). Err says why it is refused: not
/// an object, a missing, repeated or unknown key, a value that is not a string, or text after the object. Strict, so
/// a hand-edited or damaged config is reported rather than half-read by the hook that guards the branches.
pub(crate) fn parse_guard_config(text: &str) -> Result<LandRefs, String> {
    let bytes = text.as_bytes();
    let start = skip_ws(bytes, 0);
    if bytes.get(start) != Some(&b'{') {
        return Err("it is not a JSON object".to_string());
    }
    let (values, end) = members(text, skip_ws(bytes, start + 1))?;
    let rest = skip_ws(bytes, end);
    if rest < bytes.len() {
        return Err(format!("text after the object at byte {rest}"));
    }
    let [origin_branch, integration_branch, remote] = values;
    let take = |value: Option<String>, key: &str| value.ok_or_else(|| format!("key \"{key}\" is missing"));
    Ok(LandRefs {
        origin_branch: take(origin_branch, KEYS[0])?,
        integration_branch: take(integration_branch, KEYS[1])?,
        remote: take(remote, KEYS[2])?,
    })
}

/// The object's members from `pos` (after `{` and its whitespace) through the closing `}`: each key's value in KEYS
/// order, and the byte after the `}`.
fn members(text: &str, mut pos: usize) -> Result<([Option<String>; 3], usize), String> {
    let bytes = text.as_bytes();
    let mut values: [Option<String>; 3] = Default::default();
    if bytes.get(pos) == Some(&b'}') {
        return Ok((values, pos + 1));
    }
    loop {
        let (key, end) = string_at(text, pos).map_err(|why| format!("a key at byte {pos}: {why}"))?;
        let slot = KEYS.iter().position(|k| *k == key).ok_or_else(|| format!("unknown key {key:?}"))?;
        if values[slot].is_some() {
            return Err(format!("key {key:?} appears twice"));
        }
        pos = skip_ws(bytes, end);
        if bytes.get(pos) != Some(&b':') {
            return Err(format!("no ':' after key {key:?} at byte {pos}"));
        }
        let (value, end) = string_at(text, skip_ws(bytes, pos + 1))
            .map_err(|why| format!("the value of {key:?} is not a string: {why}"))?;
        values[slot] = Some(value);
        pos = skip_ws(bytes, end);
        match bytes.get(pos) {
            Some(b'}') => return Ok((values, pos + 1)),
            Some(b',') => pos = skip_ws(bytes, pos + 1),
            _ => return Err(format!("no ',' or '}}' after a member at byte {pos}")),
        }
    }
}

/// The JSON string starting at `pos` and the byte after it, or why there is none there.
fn string_at(text: &str, pos: usize) -> Result<(String, usize), String> {
    if text.as_bytes().get(pos) != Some(&b'"') {
        return Err(format!("no string at byte {pos}"));
    }
    parse_json_string(text, pos + 1).map_err(|e| match e {
        JsonError::EndOfInput => "the text ends inside a string".to_string(),
        JsonError::Invalid(why) => why,
    })
}

/// The first byte from `pos` that is not JSON whitespace (space, tab, newline, carriage return).
fn skip_ws(bytes: &[u8], pos: usize) -> usize {
    pos + bytes[pos.min(bytes.len())..].iter().take_while(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r')).count()
}
