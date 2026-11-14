//! A folder record as one line of JSON.

use super::{write_json_string, FolderKind, FolderRecord};

/// One line of JSON, no trailing newline, keys in the order kind, zone, request, base, work_order_paths,
/// needed_paths, parent; an absent request or parent is null. Every string is escaped, so any record encodes.
pub fn encode_folder_record(record: &FolderRecord) -> String {
    let mut out = String::from("{\"kind\":");
    write_json_string(&mut out, kind_name(record.kind));
    out.push_str(",\"zone\":");
    write_json_string(&mut out, &record.zone);
    out.push_str(",\"request\":");
    optional_string(&mut out, record.request.as_deref());
    out.push_str(",\"base\":");
    write_json_string(&mut out, record.base.as_str());
    out.push_str(",\"work_order_paths\":");
    strings(&mut out, &record.work_order_paths);
    out.push_str(",\"needed_paths\":");
    strings(&mut out, &record.needed_paths);
    out.push_str(",\"parent\":");
    optional_string(&mut out, record.parent.as_deref());
    out.push('}');
    out
}

fn kind_name(kind: FolderKind) -> &'static str {
    match kind {
        FolderKind::Zone => "zone",
        FolderKind::Request => "request",
    }
}

fn optional_string(out: &mut String, value: Option<&str>) {
    match value {
        Some(s) => write_json_string(out, s),
        None => out.push_str("null"),
    }
}

fn strings(out: &mut String, items: &[String]) {
    out.push('[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_json_string(out, item);
    }
    out.push(']');
}
