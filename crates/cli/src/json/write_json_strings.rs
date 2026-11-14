//! An array of strings as JSON text.

use oplog::write_json_string;

/// Appends `items` to `out` as a JSON array, each item quoted by oplog::write_json_string, no spaces.
pub(crate) fn write_json_strings(out: &mut String, items: &[String]) {
    out.push('[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_json_string(out, item);
    }
    out.push(']');
}
