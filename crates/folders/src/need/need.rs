//! Adds a path a folder needs to its cone and to its record's needed list.
//! No unit test: it takes the folders lock, runs git and reads and writes the record file (S-5); need_path_tests
//! and add_needed_tests cover its decisions. Nagatha's integration group is to cover, in a throwaway
//! repository: need adds the path to the folder's cone and to its record's needed_paths; a second need of the
//! same path changes nothing and does not fail; a record written but sparse_add not run is finished by a repeat
//! need; a plain worktree with no record refuses with NotAFolder; a repeated make after need keeps the needed
//! path in the cone.

use std::path::Path;

use gitio::Git;
use oplog::PidProbe;

use super::{add_needed, need_path};
use crate::make::{read_folder_record, with_folders_lock, write_folder_record, FolderError};
use crate::record::FolderRecord;

/// Adds `path` to the folder at `folder`: cleans it through need_path, which refuses a path gitio's sparse_add
/// would refuse only after the record was written, so a refused path writes nothing. Under the folders lock,
/// keyed `folders:need:<git dir>`: reads the folder's record, refusing a worktree with none as NotAFolder;
/// writes the record with `path` added to needed_paths unless it is already there; then adds `path` to the
/// cone with sparse_add. Returns the record as it now is.
///
/// The record is written first, so an interrupted need leaves a record naming the path. A repeat need finds it
/// already needed, leaves the record unchanged without an error, and runs sparse_add again, which is safe to
/// repeat; a repeat make refills the folder to the recorded cone, needed paths included (unless its index has
/// unmerged entries, S-36).
///
/// The caller runs oplog::recover first, as apply requires: with_folders_lock may apply a takeover op or
/// guard-removal ops (S-30).
pub fn need(git: &Git, folder: &Path, path: &str, pids: &dyn PidProbe) -> Result<FolderRecord, FolderError> {
    let clean = need_path(path).map_err(FolderError::Plan)?;
    let git_dir = git.git_dir(folder).map_err(FolderError::Git)?;
    let common = git.common_dir().map_err(FolderError::Git)?;
    with_folders_lock(git, &common, &format!("folders:need:{}", git_dir.display()), pids, || {
        let existing = read_folder_record(&git_dir)?.ok_or_else(|| FolderError::NotAFolder {
            path: folder.to_path_buf(),
            reason: "a worktree with no Mortar folder record".to_string(),
        })?;
        let record = match add_needed(&existing, &clean) {
            Some(new) => {
                write_folder_record(&git_dir, &new)?;
                new
            }
            None => existing,
        };
        git.sparse_add(folder, &[clean.clone()]).map_err(FolderError::Git)?;
        Ok(record)
    })
}
