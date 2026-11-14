//! classify_conflicted: what a resolve's first run does with one conflicted path, from gitio's typed tree_entry
//! answer for it. classify_conflicted_tests covers the rules that run no gitio code; the rest are RITs owed to
//! Nagatha (S-101, WO-join-3cr).

use crate::conflicted_entry::ConflictedEntry;
use gitio::{EntryKind, GitError, Oid, TreeEntryError};
use std::path::Path;

/// A blob at `file` is Write, no entry is Absent and a NotABlob refusal of kind Commit naming `file` is Gitlink.
/// Any other NotABlob (a folder, another type or another path) is returned as the GitError gitio's From gives it,
/// and a Git refusal as its GitError unchanged.
pub(crate) fn classify_conflicted(
    file: &Path,
    entry: Result<Option<Oid>, TreeEntryError>,
) -> Result<ConflictedEntry, GitError> {
    match entry {
        Ok(Some(_)) => Ok(ConflictedEntry::Write),
        Ok(None) => Ok(ConflictedEntry::Absent),
        Err(TreeEntryError::NotABlob { path, kind: EntryKind::Commit }) if path == file => {
            Ok(ConflictedEntry::Gitlink)
        }
        Err(error) => Err(GitError::from(error)),
    }
}
