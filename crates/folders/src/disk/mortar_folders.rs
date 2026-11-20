//! The repository's worktrees that are Mortar folders.
//! No unit test: it runs git and reads files. The integration group is to cover, in a throwaway
//! repository: a zone folder and a request folder are listed with their records; the main checkout, which has no
//! record, is not; a worktree whose folder was deleted (prunable) is not.

use std::path::PathBuf;

use gitio::Git;

use crate::make::{read_folder_record, FolderError};
use crate::record::FolderRecord;

/// Each worktree of `git`'s repository, in worktree_list's order, that has a Mortar folder record, with that
/// record. Bare and prunable worktrees are skipped. The main checkout has no record, so it is never listed.
pub(crate) fn mortar_folders(git: &Git) -> Result<Vec<(PathBuf, FolderRecord)>, FolderError> {
    let mut folders = Vec::new();
    for worktree in git.worktree_list().map_err(FolderError::Git)? {
        if worktree.bare || worktree.prunable {
            continue;
        }
        let git_dir = git.git_dir(&worktree.path).map_err(FolderError::Git)?;
        if let Some(record) = read_folder_record(&git_dir)? {
            folders.push((worktree.path, record));
        }
    }
    Ok(folders)
}
