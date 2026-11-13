//! One op log line back to a record, telling a torn write from a bad line.

use super::{
    decode_op_result, decode_ref_move, object_fields, parse_json, take_string, JsonError, JsonValue,
    LineError, OpPhase, OpRecord, RefMove,
};

/// The record on `line`, the strict inverse of encode_op_record: exactly its keys, each once, `result`
/// optional. JSON that stops early is Torn; every other refusal is Malformed with what is wrong.
pub(crate) fn decode_op_line(line: &str) -> Result<OpRecord, LineError> {
    let value = parse_json(line).map_err(|e| match e {
        JsonError::EndOfInput => LineError::Torn,
        JsonError::Invalid(reason) => LineError::Malformed(reason),
    })?;
    record(value).map_err(LineError::Malformed)
}

fn record(value: JsonValue) -> Result<OpRecord, String> {
    let keys = ["op_id", "key", "kind", "actor", "reason", "at", "phase", "moves", "result"];
    let [op_id, key, kind, actor, reason, at, phase, moves, result] =
        object_fields(value, keys, "the record")?;
    Ok(OpRecord {
        op_id: take_string(op_id, "op_id")?,
        key: take_string(key, "key")?,
        kind: take_string(kind, "kind")?,
        actor: take_string(actor, "actor")?,
        reason: take_string(reason, "reason")?,
        at: match at {
            Some(JsonValue::Number(n)) => n,
            Some(_) => return Err("at is not a non-negative integer".to_string()),
            None => return Err("missing key \"at\"".to_string()),
        },
        phase: phase_named(&take_string(phase, "phase")?)?,
        moves: match moves {
            Some(JsonValue::Array(items)) => ref_moves(items)?,
            Some(_) => return Err("moves is not an array".to_string()),
            None => return Err("missing key \"moves\"".to_string()),
        },
        result: result.map(decode_op_result).transpose()?,
    })
}

fn phase_named(name: &str) -> Result<OpPhase, String> {
    match name {
        "intent" => Ok(OpPhase::Intent),
        "done" => Ok(OpPhase::Done),
        "failed" => Ok(OpPhase::Failed),
        other => Err(format!("unknown phase {other:?} (expected intent, done or failed)")),
    }
}

/// Each move decoded; a refusal names the move by its 1-based place in the list.
fn ref_moves(items: Vec<JsonValue>) -> Result<Vec<RefMove>, String> {
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| decode_ref_move(item).map_err(|e| format!("move {}: {e}", i + 1)))
        .collect()
}
