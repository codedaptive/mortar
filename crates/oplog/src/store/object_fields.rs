//! The members of a JSON object, sorted into the slots of a fixed key list.

use super::JsonValue;

/// One slot per name in `keys`, holding that key's value, or None when the key is absent; the caller
/// decides which keys are required. Refuses a value that is not an object, a key not in `keys`, and a
/// key given twice. `what` names the object in the refusal.
pub(crate) fn object_fields<const N: usize>(
    value: JsonValue,
    keys: [&str; N],
    what: &str,
) -> Result<[Option<JsonValue>; N], String> {
    let JsonValue::Object(pairs) = value else {
        return Err(format!("{what} is not a JSON object"));
    };
    let mut slots: [Option<JsonValue>; N] = std::array::from_fn(|_| None);
    for (name, item) in pairs {
        let Some(slot) = keys.iter().position(|k| *k == name).map(|i| &mut slots[i]) else {
            return Err(format!("{what} has unknown key {name:?}"));
        };
        if slot.replace(item).is_some() {
            return Err(format!("{what} has key {name:?} twice"));
        }
    }
    Ok(slots)
}
