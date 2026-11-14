//! How an AttemptsRemain refusal reads: the request folder remove kept, every attempt left, and the next step,
//! including the one for an attempt folder whose try branch is gone.

use std::path::Path;

/// The text of AttemptsRemain: the folder at `path`, then `attempts` (branch names or folder paths) joined with ", ".
pub(crate) fn attempts_remain_text(path: &Path, attempts: &[String]) -> String {
    format!(
        "{} was not removed: attempts of its request remain: {}; nothing changed; next step: keep one attempt or \
         drop them all (drop_attempts); for an attempt folder that lost its branch, remake_orphans first (mortar \
         runs it), and with no snapshot of it, copy out anything wanted and remove it by hand",
        path.display(),
        attempts.join(", ")
    )
}
