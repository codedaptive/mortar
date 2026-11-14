//! classify_conflicted: what a resolve's first run does with one conflicted path, from gitio's tree_entry answer for
//! it. classify_conflicted_tests covers each rule.
//! gitio's GitError has no refusal kinds yet (S-65), so a gitlink is told apart by gitio's exact refusal message, as
//! tree_refusal does: tree_entry refuses any entry that is not a blob with no exit code, no `args` and
//! `not a blob: <path> is a <type>` in `stderr`, the path and the type each in their `{:?}` form (gitio's
//! read_tree_entry).

use crate::conflicted_entry::ConflictedEntry;
use gitio::{GitError, Oid};
use std::path::Path;

/// A blob at `file` is Write and no entry is Absent. A refusal with no exit code, no `args` and exactly
/// `not a blob: <file> is a "commit"` is Gitlink. Every other error, another type, another path, or a failed git run
/// included, is returned unchanged.
pub(crate) fn classify_conflicted(
    file: &Path,
    entry: Result<Option<Oid>, GitError>,
) -> Result<ConflictedEntry, GitError> {
    match entry {
        Ok(Some(_)) => Ok(ConflictedEntry::Write),
        Ok(None) => Ok(ConflictedEntry::Absent),
        Err(error)
            if error.code.is_none()
                && error.args.is_empty()
                && error.stderr == format!("not a blob: {file:?} is a \"commit\"") =>
        {
            Ok(ConflictedEntry::Gitlink)
        }
        Err(error) => Err(error),
    }
}
