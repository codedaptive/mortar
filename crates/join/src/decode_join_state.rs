//! decode_join_state: reads the text of mortar-join.json back into a JoinState. decode_join_state_tests covers a
//! hand-written text, an escape encode_join_state never writes, and each refusal, a text missing all_files or
//! folder_tree included.
//! Why a strict reader of one layout, not a general JSON parser: only encode_join_state writes this file, so text
//! in any other layout is not a state join wrote, and resolve must not act on it. Strings are read by oplog's
//! parse_json_string, which takes every JSON escape.

use crate::join_state::JoinState;
use gitio::Oid;
use oplog::{parse_json_string, JsonError};
use std::path::PathBuf;

/// Reads exactly the layout encode_join_state writes: one JSON object with no whitespace, its 12 keys in
/// encode_join_state's order, and at most one trailing newline. Strings may hold any JSON escape; ids must be valid
/// object ids. Anything else is refused with the reason and the byte it was found at.
pub(crate) fn decode_join_state(text: &str) -> Result<JoinState, String> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    let (request_ref, at) = string(body, literal(body, 0, "{\"request_ref\":")?)?;
    let (request_tip, at) = id(body, key(body, at, "request_tip")?)?;
    let (onto, at) = id(body, key(body, at, "onto")?)?;
    let (replayed_tip, at) = id(body, key(body, at, "replayed_tip")?)?;
    let (commit, at) = id(body, key(body, at, "commit")?)?;
    let (tree, at) = id(body, key(body, at, "tree")?)?;
    let (remaining, at) = array(body, key(body, at, "remaining")?, id)?;
    let (files, at) = array(body, key(body, at, "files")?, path)?;
    let (all_files, at) = array(body, key(body, at, "all_files")?, path)?;
    let (files_written, at) = flag(body, key(body, at, "files_written")?)?;
    let (rebased_to, at) = optional_id(body, key(body, at, "rebased_to")?)?;
    let (folder_tree, at) = optional_id(body, key(body, at, "folder_tree")?)?;
    if body.get(at..) != Some("}") {
        return Err(format!("expected the closing brace and the end of the text at byte {at}"));
    }
    Ok(JoinState {
        request_ref,
        request_tip,
        onto,
        replayed_tip,
        commit,
        tree,
        remaining,
        files,
        all_files,
        files_written,
        rebased_to,
        folder_tree,
    })
}

/// The index after `lit`, which must start at byte `at`.
fn literal(body: &str, at: usize, lit: &str) -> Result<usize, String> {
    match body.get(at..) {
        Some(rest) if rest.starts_with(lit) => Ok(at + lit.len()),
        _ => Err(format!("expected {lit} at byte {at}")),
    }
}

/// The index after `,"<name>":`, the separator and key encode_join_state writes before each value but the first.
fn key(body: &str, at: usize, name: &str) -> Result<usize, String> {
    literal(body, at, &format!(",\"{name}\":"))
}

/// The string whose opening quote is at byte `at`, and the index after its closing quote.
fn string(body: &str, at: usize) -> Result<(String, usize), String> {
    parse_json_string(body, literal(body, at, "\"")?).map_err(|error| match error {
        JsonError::EndOfInput => {
            format!("the state file's text could not be read: it ends inside the string starting at byte {at}")
        }
        JsonError::Invalid(why) => format!("the state file's text could not be read: {why}"),
    })
}

fn id(body: &str, at: usize) -> Result<(Oid, usize), String> {
    let (hex, next) = string(body, at)?;
    let oid = Oid::parse(&hex).map_err(|error| format!("at byte {at}: {error}"))?;
    Ok((oid, next))
}

fn path(body: &str, at: usize) -> Result<(PathBuf, usize), String> {
    let (text, next) = string(body, at)?;
    Ok((PathBuf::from(text), next))
}

/// `[]`, or `[` then items separated by `,` then `]`, each item read by `item`.
fn array<T>(
    body: &str,
    at: usize,
    item: fn(&str, usize) -> Result<(T, usize), String>,
) -> Result<(Vec<T>, usize), String> {
    let mut at = literal(body, at, "[")?;
    let mut items = Vec::new();
    if let Ok(end) = literal(body, at, "]") {
        return Ok((items, end));
    }
    loop {
        let (value, next) = item(body, at)?;
        items.push(value);
        if let Ok(end) = literal(body, next, "]") {
            return Ok((items, end));
        }
        at = literal(body, next, ",")?;
    }
}

fn flag(body: &str, at: usize) -> Result<(bool, usize), String> {
    match (literal(body, at, "true"), literal(body, at, "false")) {
        (Ok(next), _) => Ok((true, next)),
        (_, Ok(next)) => Ok((false, next)),
        _ => Err(format!("expected true or false at byte {at}")),
    }
}

fn optional_id(body: &str, at: usize) -> Result<(Option<Oid>, usize), String> {
    match literal(body, at, "null") {
        Ok(next) => Ok((None, next)),
        Err(_) => id(body, at).map(|(oid, next)| (Some(oid), next)),
    }
}
