//! One parsed move object back to a RefMove.

use super::{object_fields, take_oid, take_optional_oid, take_string, JsonValue, RefMove};

/// Exactly the keys `ref`, `old` (object id or null) and `new` (object id), each once.
pub(crate) fn decode_ref_move(value: JsonValue) -> Result<RefMove, String> {
    let [refname, old, new] = object_fields(value, ["ref", "old", "new"], "the move")?;
    Ok(RefMove {
        refname: take_string(refname, "ref")?,
        old: take_optional_oid(old, "old")?,
        new: take_oid(new, "new")?,
    })
}
