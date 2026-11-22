//! holds_json: one folder's member of disk's --json "holds" array.

use std::path::Path;

use oplog::write_json_string;

use super::{FolderHolds, JoinedAnswer};

/// `{"path":<path>,"tracked":<t>,"untracked":<u>,"joined":true|false|null,"no_joined_answer":null|<why>}` with no
/// spaces; the path is the folder's display text as a JSON string. joined is null when there is no answer, and
/// no_joined_answer says why as a JSON string: "a zone folder", or the reason the joined read failed; it is null
/// when joined is true or false.
pub fn holds_json(path: &Path, holds: &FolderHolds) -> String {
    let mut out = String::from("{\"path\":");
    write_json_string(&mut out, &path.display().to_string());
    let joined = match holds.joined {
        JoinedAnswer::Joined => "true",
        JoinedAnswer::NotJoined => "false",
        JoinedAnswer::ZoneFolder | JoinedAnswer::Unknown(_) => "null",
    };
    out.push_str(&format!(
        ",\"tracked\":{},\"untracked\":{},\"joined\":{joined},\"no_joined_answer\":",
        holds.unsaved.tracked, holds.unsaved.untracked
    ));
    match &holds.joined {
        JoinedAnswer::Joined | JoinedAnswer::NotJoined => out.push_str("null"),
        JoinedAnswer::ZoneFolder => write_json_string(&mut out, "a zone folder"),
        JoinedAnswer::Unknown(reason) => write_json_string(&mut out, reason),
    }
    out.push('}');
    out
}
