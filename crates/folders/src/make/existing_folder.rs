//! What is at the path where a folder is to be made: nothing, a folder mortar made, or something in the way.
//! No unit test: it reads the worktree list, the file system and a record file (S-5). Nagatha's integration
//! group is to cover, in a throwaway repository: an absent path and an empty directory answer None; a folder
//! made by make_request_folder answers its record, also when named through a symlinked parent (macOS /var);
//! a worktree added by plain git answers NotAFolder "a worktree with no Mortar folder record"; a directory
//! holding a file, and a plain file, answer NotAFolder "exists and is not a worktree".

use std::path::{Path, PathBuf};

use gitio::Git;

use super::{read_folder_record, FolderError};
use crate::record::FolderRecord;

/// The record of the worktree at `path`, or None when nothing is there or the directory is empty. A worktree
/// without a record is never adopted, and anything else at `path` is in the way: both are NotAFolder. Paths are
/// compared after canonicalizing, since git lists real paths (macOS /var is /private/var).
pub fn existing_folder(git: &Git, path: &Path) -> Result<Option<FolderRecord>, FolderError> {
    let wanted = canonical(path);
    let listed = git
        .worktree_list()
        .map_err(FolderError::Git)?
        .iter()
        .any(|w| canonical(&w.path) == wanted);
    if listed {
        let git_dir = git.git_dir(path).map_err(FolderError::Git)?;
        return match read_folder_record(&git_dir)? {
            Some(record) => Ok(Some(record)),
            None => Err(not_a_folder(path, "a worktree with no Mortar folder record")),
        };
    }
    if !path.exists() {
        return Ok(None);
    }
    if path.is_dir() {
        let mut entries = std::fs::read_dir(path).map_err(|source| FolderError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if entries.next().is_none() {
            return Ok(None);
        }
    }
    Err(not_a_folder(path, "exists and is not a worktree"))
}

/// The real path when it exists; else the path as given.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn not_a_folder(path: &Path, reason: &str) -> FolderError {
    FolderError::NotAFolder {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
}
