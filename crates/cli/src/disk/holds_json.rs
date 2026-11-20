//! holds_json: one folder's member of disk's --json "holds" array.

use std::path::Path;

use oplog::write_json_string;

use super::FolderHolds;

/// `{"path":<path>,"tracked":<t>,"untracked":<u>,"joined":true|false|null}` with no spaces; the path is the folder's
/// display text as a JSON string, and joined is null for a zone folder.
pub fn holds_json(path: &Path, holds: &FolderHolds) -> String {
    let mut out = String::from("{\"path\":");
    write_json_string(&mut out, &path.display().to_string());
    let joined = match holds.joined {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    out.push_str(&format!(
        ",\"tracked\":{},\"untracked\":{},\"joined\":{joined}}}",
        holds.unsaved.tracked, holds.unsaved.untracked
    ));
    out
}
