//! One `{"ref", "actual"}` entry of a not-moved result back to a ref name and its actual value.

use gitio::Oid;

use super::{object_fields, take_optional_oid, take_string, JsonValue};

/// Exactly the keys `ref` and `actual` (object id or null), each once.
pub(crate) fn decode_ref_actual(value: JsonValue) -> Result<(String, Option<Oid>), String> {
    let [refname, actual] = object_fields(value, ["ref", "actual"], "the ref entry")?;
    Ok((take_string(refname, "ref")?, take_optional_oid(actual, "actual")?))
}
