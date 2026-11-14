//! A record's `values` member back to its name/value pairs.

use super::{object_fields, take_string, JsonValue};

/// The pairs in array order. Refuses a value that is not an array, and an item that is not an object
/// with exactly the string members `name` and `value`, each once; a refusal names the item by its
/// 1-based place in the array.
pub(crate) fn decode_op_values(value: JsonValue) -> Result<Vec<(String, String)>, String> {
    let JsonValue::Array(items) = value else {
        return Err("values is not an array".to_string());
    };
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| pair(item).map_err(|e| format!("value {}: {e}", i + 1)))
        .collect()
}

fn pair(item: JsonValue) -> Result<(String, String), String> {
    let [name, value] = object_fields(item, ["name", "value"], "the item")?;
    Ok((take_string(name, "name")?, take_string(value, "value")?))
}
