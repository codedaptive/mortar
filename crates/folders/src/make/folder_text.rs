//! How a folder record reads inside a refusal's text, for FolderError's Display. Kept apart from
//! folder_error_display so that file stays under 150 lines; folder_text_tests exercises each phrase.

use crate::record::{FolderKind, FolderRecord};

/// "the zone folder for zone Z", "the request folder for zone Z, request R", "attempt k's folder for zone Z,
/// request R", or, for a record with no request, "a request folder for zone Z" and "attempt k's folder for zone Z".
pub(super) fn folder_text(record: &FolderRecord) -> String {
    match (record.kind, &record.request) {
        (FolderKind::Request, Some(request)) => {
            format!("the request folder for zone {}, request {request}", record.zone)
        }
        (FolderKind::Request, None) => format!("a request folder for zone {}", record.zone),
        (FolderKind::Attempt(k), Some(request)) => {
            format!("attempt {k}'s folder for zone {}, request {request}", record.zone)
        }
        (FolderKind::Attempt(k), None) => format!("attempt {k}'s folder for zone {}", record.zone),
        (FolderKind::Zone, _) => format!("the zone folder for zone {}", record.zone),
    }
}
