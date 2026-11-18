//! decode_record: Install's record read back from install.json.

use std::path::PathBuf;

use oplog::{parse_json, JsonError, JsonValue};

use super::RecordItem;

/// The items, or why the text is not a record, naming the item index. The text is one object whose one key is
/// `items`, an array; each item has a known `kind` and exactly that kind's keys, each once, each a string.
pub fn decode_record(text: &str) -> Result<Vec<RecordItem>, String> {
    let value = parse_json(text).map_err(|error| match error {
        JsonError::EndOfInput => "it ends before its JSON is complete".to_string(),
        JsonError::Invalid(why) => format!("it is not JSON: {why}"),
    })?;
    let JsonValue::Object(pairs) = value else {
        return Err("it is not an object".to_string());
    };
    let [(key, JsonValue::Array(items))] = pairs.as_slice() else {
        return Err("it is not an object with one key, \"items\", an array".to_string());
    };
    if key != "items" {
        return Err(format!("its one key is \"{key}\", not \"items\""));
    }
    items.iter().enumerate().map(|(index, item)| decode_item(index, item)).collect()
}

/// One item of the array, at `index`.
fn decode_item(index: usize, item: &JsonValue) -> Result<RecordItem, String> {
    let JsonValue::Object(pairs) = item else {
        return Err(format!("item {index} is not an object"));
    };
    let fields = string_fields(index, pairs)?;
    let field = |key: &str| {
        fields
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| *value)
            .ok_or_else(|| format!("item {index} has no \"{key}\""))
    };
    let kind = field("kind")?;
    let keys: &[&str] = match kind {
        "file" => &["kind", "path", "hash"],
        "link" => &["kind", "path", "target"],
        "dir" => &["kind", "path"],
        other => return Err(format!("item {index} has an unknown kind \"{other}\"")),
    };
    if let Some((extra, _)) = fields.iter().find(|(name, _)| !keys.contains(name)) {
        return Err(format!("item {index} has an unexpected key \"{extra}\""));
    }
    let path = PathBuf::from(field("path")?);
    // `kind` is one of the three here: the match above returned for any other.
    Ok(match kind {
        "file" => RecordItem::File { path, hash: field("hash")?.to_string() },
        "link" => RecordItem::Link { path, target: PathBuf::from(field("target")?) },
        _ => RecordItem::Dir { path },
    })
}

/// The item's members as (key, text), refusing a key given twice or a value that is not a string.
fn string_fields(index: usize, pairs: &[(String, JsonValue)]) -> Result<Vec<(&str, &str)>, String> {
    let mut fields: Vec<(&str, &str)> = Vec::with_capacity(pairs.len());
    for (key, value) in pairs {
        if fields.iter().any(|(name, _)| name == key) {
            return Err(format!("item {index} has \"{key}\" twice"));
        }
        let JsonValue::String(text) = value else {
            return Err(format!("item {index}: \"{key}\" is not a string"));
        };
        fields.push((key, text));
    }
    Ok(fields)
}
