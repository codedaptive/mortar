//! A required member that is an object id or null, from its object_fields slot.

use gitio::Oid;

use super::{take_oid, JsonValue};

/// None for JSON null; otherwise what take_oid gives. The key itself must be present.
pub(crate) fn take_optional_oid(slot: Option<JsonValue>, name: &str) -> Result<Option<Oid>, String> {
    match slot {
        Some(JsonValue::Null) => Ok(None),
        other => take_oid(other, name).map(Some),
    }
}
