//! help_entry_json: one verb of the `--json` help object's "verbs" array (SPEC-cli 08, "The calls").

use oplog::write_json_string;

use super::{usage_forms, HelpEntry, HelpGroup};
use crate::json::write_json_strings;

/// `{"name":..,"group":..,"line":..,"usage":[..]}` on one line: the entry's name, its group's key, its line, and
/// its usage one form per item.
pub fn help_entry_json(group: &HelpGroup, entry: &HelpEntry) -> String {
    let mut out = String::from("{\"name\":");
    write_json_string(&mut out, entry.name);
    out.push_str(",\"group\":");
    write_json_string(&mut out, group.key);
    out.push_str(",\"line\":");
    write_json_string(&mut out, entry.line);
    out.push_str(",\"usage\":");
    write_json_strings(&mut out, &usage_forms(entry.name));
    out.push('}');
    out
}
