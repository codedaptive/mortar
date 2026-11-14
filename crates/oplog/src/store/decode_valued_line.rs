//! One op log line back to a record and its values, telling a torn write from a bad line.

use super::{
    decode_op_result, decode_op_values, decode_ref_move, object_fields, parse_json, take_string,
    JsonError, JsonValue, LineError, OpPhase, OpRecord, RefMove, ValuedRecord,
};

/// The record and values on `line`, the strict inverse of encode_valued_record: exactly its keys, each
/// once, `result` and `values` optional (no `values` key: no values). JSON that stops early is Torn; every
/// other refusal is Malformed with what is wrong.
pub(crate) fn decode_valued_line(line: &str) -> Result<ValuedRecord, LineError> {
    let value = parse_json(line).map_err(|e| match e {
        JsonError::EndOfInput => LineError::Torn,
        JsonError::Invalid(reason) => LineError::Malformed(reason),
    })?;
    valued_record(value).map_err(LineError::Malformed)
}

fn valued_record(value: JsonValue) -> Result<ValuedRecord, String> {
    let keys = [
        "op_id", "key", "kind", "actor", "reason", "at", "phase", "moves", "result", "values",
    ];
    let [op_id, key, kind, actor, reason, at, phase, moves, result, values] =
        object_fields(value, keys, "the record")?;
    let record = OpRecord {
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
    };
    let values = values.map(decode_op_values).transpose()?.unwrap_or_default();
    Ok(ValuedRecord { record, values })
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
