//! write_conflicted_files: a resolve's first run, writing the conflicted tree's versions of the conflicted files
//! into the folder.
//! No unit test: it runs git and writes real files (S-5). Nagatha's integration tests are to cover, in a throwaway
//! repository: the conflicted versions are written, markers included (WO-join-3 Goal 2); a conflicted path the
//! conflicted tree lacks (rename/rename) is skipped and listed (S-50: "a conflicted path the conflicted tree lacks
//! (rename/rename) is skipped by the first run and listed in the answer").

use crate::JoinRunError;
use gitio::{Git, Oid};
use std::path::{Path, PathBuf};

/// Splits `files` into those `tree` has a version of and those it lacks, in order, writes the first into `folder`
/// (no write when there are none) and returns both: (written, skipped).
pub(crate) fn write_conflicted_files(
    git: &Git,
    folder: &Path,
    tree: &Oid,
    files: Vec<PathBuf>,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), JoinRunError> {
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    for file in files {
        match git.tree_entry(tree, &file).map_err(JoinRunError::Git)? {
            Some(_) => written.push(file),
            None => skipped.push(file),
        }
    }
    if !written.is_empty() {
        git.write_blob_paths(folder, tree, &written).map_err(JoinRunError::Git)?;
    }
    Ok((written, skipped))
}
