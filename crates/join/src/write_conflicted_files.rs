//! write_conflicted_files: a resolve's first run, writing the conflicted tree's versions of the conflicted files
//! into the folder.
//! No unit test: it runs git and writes real files (S-5); classify_conflicted_tests covers how each path is sorted.
//! Nagatha's integration tests are to cover, in a throwaway repository: the conflicted versions are written, markers
//! included (WO-join-3 Goal 2); a conflicted path the conflicted tree lacks (rename/rename) is skipped and listed
//! (S-50: "a conflicted path the conflicted tree lacks (rename/rename) is skipped by the first run and listed in the
//! answer"); a conflicted gitlink (submodule) is not written and is listed apart from the absent paths (S-64).

use crate::classify_conflicted::classify_conflicted;
use crate::conflicted_entry::ConflictedEntry;
use crate::JoinRunError;
use gitio::{Git, Oid};
use std::path::{Path, PathBuf};

/// Sorts `files`, in order, into those `tree` has a file version of, those it lacks and those that are gitlinks
/// (submodules), writes the first into `folder` (no write when there are none) and returns all three:
/// (written, absent, gitlinks).
pub(crate) fn write_conflicted_files(
    git: &Git,
    folder: &Path,
    tree: &Oid,
    files: Vec<PathBuf>,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>), JoinRunError> {
    let mut written = Vec::new();
    let mut absent = Vec::new();
    let mut gitlinks = Vec::new();
    for file in files {
        match classify_conflicted(&file, git.tree_entry(tree, &file)).map_err(JoinRunError::Git)? {
            ConflictedEntry::Write => written.push(file),
            ConflictedEntry::Absent => absent.push(file),
            ConflictedEntry::Gitlink => gitlinks.push(file),
        }
    }
    if !written.is_empty() {
        git.write_blob_paths(folder, tree, &written).map_err(JoinRunError::Git)?;
    }
    Ok((written, absent, gitlinks))
}
