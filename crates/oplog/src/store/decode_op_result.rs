//! One parsed result object back to an OpResult.

use gitio::Oid;

use super::{decode_ref_actual, object_fields, take_optional_oid, take_string, JsonValue, OpResult};

/// `outcome` "moved" alone, "stale" with `ref` and `actual` (object id or null), or "not-moved" with
/// `refs`, an array of `{"ref", "actual"}` entries (possibly empty). Any other outcome name, and a key
/// beside an outcome that has none, is refused.
pub(crate) fn decode_op_result(value: JsonValue) -> Result<OpResult, String> {
    let [outcome, refname, actual, refs] =
        object_fields(value, ["outcome", "ref", "actual", "refs"], "the result")?;
    let outcome = take_string(outcome, "outcome")?;
    let has_ref_or_actual = refname.is_some() || actual.is_some();
    match outcome.as_str() {
        "moved" if has_ref_or_actual || refs.is_some() => {
            Err("outcome moved takes no ref, actual or refs".to_string())
        }
        "moved" => Ok(OpResult::Moved),
        "stale" if refs.is_some() => Err("outcome stale takes no refs".to_string()),
        "stale" => Ok(OpResult::Stale {
            refname: take_string(refname, "ref")?,
            actual: take_optional_oid(actual, "actual")?,
        }),
        "not-moved" if has_ref_or_actual => Err("outcome not-moved takes no ref or actual".to_string()),
        "not-moved" => Ok(OpResult::NotMoved { refs: ref_actuals(refs)? }),
        other => Err(format!("unknown outcome {other:?} (expected moved, stale or not-moved)")),
    }
}

/// Each entry decoded; a refusal names the entry by its 1-based place in the list.
fn ref_actuals(slot: Option<JsonValue>) -> Result<Vec<(String, Option<Oid>)>, String> {
    let items = match slot {
        Some(JsonValue::Array(items)) => items,
        Some(_) => return Err("refs is not an array".to_string()),
        None => return Err("missing key \"refs\"".to_string()),
    };
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| decode_ref_actual(item).map_err(|e| format!("ref entry {}: {e}", i + 1)))
        .collect()
}
