//! item_json: One report item as a JSON object.

use oplog::write_json_string;

use super::InstallItem;

/// `{"action":..,"path":..,"note":..}`; the path is "" when the item is about no one path.
pub fn item_json(item: &InstallItem) -> String {
    let mut out = String::from("{\"action\":");
    write_json_string(&mut out, &item.action);
    out.push_str(",\"path\":");
    write_json_string(&mut out, &item.path.to_string_lossy());
    out.push_str(",\"note\":");
    write_json_string(&mut out, &item.note);
    out.push('}');
    out
}
