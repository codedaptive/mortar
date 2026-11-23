//! pending_commit_text: the text of the pending marker, as parse_pending_commit reads it back.
//! pending_commit_text_tests covers it.

use super::PendingCommit;

/// The tip and "\n", then each path as displayed, each followed by "\n". A path holding a newline, or one that is
/// not UTF-8, does not read back as it was.
pub fn pending_commit_text(pending: &PendingCommit) -> String {
    let mut text = format!("{}\n", pending.tip);
    for path in &pending.paths {
        text.push_str(&format!("{}\n", path.display()));
    }
    text
}
