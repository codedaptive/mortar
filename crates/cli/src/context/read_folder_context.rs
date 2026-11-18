//! Reads the Mortar folder holding a directory: its worktree, git dirs and record (S-18).
//! No unit test: it runs git and reads a real file (S-5); context_report_tests covers what each refusal says.
//! Nagatha's integration group is to cover, with the mortar binary as a child process in a throwaway repository
//! and the first agent verb (mr join): run outside any git repository it is refused, exit 1, outcome
//! "not-a-folder"; run in a worktree with no Mortar record (the main checkout) it is refused, exit 1, naming the
//! worktree's top; run in a subfolder of a request folder it finds that folder (toplevel, its own git dir, the
//! shared common dir) and its record; a record file holding text that is not a record is refused, exit 1, naming
//! the file.

use std::path::Path;

use folders::make::read_folder_record;
use gitio::Git;

use super::{repository_error, ContextError, FolderContext};

/// The folder whose worktree holds `cwd`. Git refusing toplevel with an exit code means `cwd` is in no
/// repository (NotARepository); with none, git itself failed (Git). A worktree without a record is NotAFolder.
pub fn read_folder_context(cwd: &Path) -> Result<FolderContext, ContextError> {
    let git = Git::new(cwd);
    let toplevel = git.toplevel().map_err(|error| repository_error(cwd, error))?;
    let git_dir = git.git_dir(&toplevel).map_err(ContextError::Git)?;
    let common = git.common_dir().map_err(ContextError::Git)?;
    match read_folder_record(&git_dir).map_err(ContextError::Record)? {
        Some(record) => Ok(FolderContext { toplevel, git_dir, common, record }),
        None => Err(ContextError::NotAFolder { toplevel }),
    }
}
