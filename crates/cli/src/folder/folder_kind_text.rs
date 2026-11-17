//! A folder record's kind in the cli's words, shared by folder_error_why and disk_report.

use folders::{FolderKind, FolderRecord};

/// "the zone folder of zone <z>", "the request folder of request <r> in zone <z>" or "attempt folder <k> of request
/// <r> in zone <z>".
pub fn folder_kind_text(record: &FolderRecord) -> String {
    let (zone, request) = (&record.zone, record.request.as_deref().unwrap_or("(none)"));
    match record.kind {
        FolderKind::Zone => format!("the zone folder of zone {zone}"),
        FolderKind::Request => format!("the request folder of request {request} in zone {zone}"),
        FolderKind::Attempt(k) => format!("attempt folder {k} of request {request} in zone {zone}"),
    }
}
