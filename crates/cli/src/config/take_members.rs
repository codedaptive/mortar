//! An object's members sorted into one slot per expected key.

use oplog::JsonValue;

/// One slot per name in `keys`, holding that key's value, or None when the key is absent (the take_*
/// helpers refuse a missing key by name). A value that is not an object, an unknown key and a key given
/// twice are refused, naming `place`.
pub(crate) fn take_members<const N: usize>(
    value: JsonValue,
    keys: [&str; N],
    place: &str,
) -> Result<[Option<JsonValue>; N], String> {
    let JsonValue::Object(members) = value else {
        return Err(format!("{place} is not an object"));
    };
    let mut slots: [Option<JsonValue>; N] = std::array::from_fn(|_| None);
    for (name, value) in members {
        let Some(i) = keys.iter().position(|k| *k == name) else {
            return Err(format!("unknown key {name:?} in {place}"));
        };
        if slots[i].replace(value).is_some() {
            return Err(format!("key {name:?} appears twice in {place}"));
        }
    }
    Ok(slots)
}
