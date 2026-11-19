//! The Mortar folder the current directory is in, as an agent verb sees it.
//! A bare type; read_folder_context makes it and has no unit test, so the validation tests listed
//! there are to exercise it.

use std::path::PathBuf;

use folders::FolderRecord;

/// `toplevel` is the worktree's top, `git_dir` its own git dir (where the record is), `common` the repository's
/// shared git dir (where the config, the op log and the locks are), `record` the folder's Mortar record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderContext {
    pub toplevel: PathBuf,
    pub git_dir: PathBuf,
    pub common: PathBuf,
    pub record: FolderRecord,
}
