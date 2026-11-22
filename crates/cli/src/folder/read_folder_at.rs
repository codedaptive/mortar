//! read_folder_at: the top of the worktree at a path and the folder record Mortar keeps in its git dir, the lookup
//! `mr folder remove` and `mr folder refresh` make before any lock.
//! No sibling test: it runs git and reads the record from disk, and holds no decision of its own. The integration
//! tests crates/cli/tests/folder_runs/run_folder_refresh_tests.rs (a plain directory and a plain git worktree each
//! exit 1 not-a-folder; zone, request and attempt folders read) and run_folder_remove_tests.rs (request, attempt and
//! zone folders read) cover it through the binary.

use std::path::{Path, PathBuf};

use folders::make::read_folder_record;
use folders::FolderRecord;
use gitio::Git;

use super::folder_error_report;
use crate::report::Report;

/// The top of the worktree at `path` and its folder record. Git finding no worktree there is
/// `not_a_folder(path, "git finds no worktree there")`; no git dir for that top, `not_a_folder(top, git's error)`; a
/// worktree with no record, `not_a_folder(top, "the worktree has no Mortar record")`. A record that cannot be read
/// is folder_error_report's for `sub_verb`. Each verb passes its own not_a_folder, since what it says was left
/// unchanged and its next step differ.
pub fn read_folder_at(
    path: &Path,
    sub_verb: &str,
    not_a_folder: fn(&Path, &str) -> Report,
) -> Result<(PathBuf, FolderRecord), Report> {
    let folder = Git::new(path).toplevel().map_err(|_| not_a_folder(path, "git finds no worktree there"))?;
    let git_dir = Git::new(&folder).git_dir(&folder).map_err(|error| not_a_folder(&folder, &error.to_string()))?;
    match read_folder_record(&git_dir) {
        Ok(Some(record)) => Ok((folder, record)),
        Ok(None) => Err(not_a_folder(&folder, "the worktree has no Mortar record")),
        Err(error) => Err(folder_error_report(sub_verb, &error)),
    }
}
