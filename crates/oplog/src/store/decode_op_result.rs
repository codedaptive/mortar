//! One parsed result object back to an OpResult.

use super::{object_fields, take_optional_oid, take_string, JsonValue, OpResult};

/// `outcome` "moved" or "not-moved" alone, or "stale" with `ref` and `actual` (object id or null).
/// Any other outcome name, and `ref` or `actual` beside an outcome that has none, is refused.
pub(crate) fn decode_op_result(value: JsonValue) -> Result<OpResult, String> {
    let [outcome, refname, actual] = object_fields(value, ["outcome", "ref", "actual"], "the result")?;
    let outcome = take_string(outcome, "outcome")?;
    if outcome == "stale" {
        return Ok(OpResult::Stale {
            refname: take_string(refname, "ref")?,
            actual: take_optional_oid(actual, "actual")?,
        });
    }
    let result = match outcome.as_str() {
        "moved" => OpResult::Moved,
        "not-moved" => OpResult::NotMoved,
        other => return Err(format!("unknown outcome {other:?} (expected moved, stale or not-moved)")),
    };
    if refname.is_some() || actual.is_some() {
        return Err(format!("outcome {outcome} takes no ref or actual"));
    }
    Ok(result)
}
