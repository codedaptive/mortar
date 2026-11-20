//! A required object id member, from its object_fields slot.

use gitio::Oid;

use super::{take_string, JsonValue};

/// The object id in `slot`, checked by Oid::parse; refuses what take_string refuses, and text that is not
/// an object id, naming the key and the text.
pub(crate) fn take_oid(slot: Option<JsonValue>, name: &str) -> Result<Oid, String> {
    let text = take_string(slot, name)?;
    Oid::parse(&text).map_err(|_| format!("{name} {text:?} is not a git object id"))
}
