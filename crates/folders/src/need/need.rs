//! Adds a path a folder needs to its cone and to its record's needed list.
//! No unit test: it takes the folders lock, runs git and reads and writes the record file (S-5); need_path_tests,
//! add_needed_tests and cone_folders_tests cover its decisions. Nagatha's integration group is to cover, in a
//! throwaway repository: need adds the path to the folder's cone and to its record's needed_paths; a second need of
//! the same path changes nothing and does not fail; a record written but sparse_add not run is finished by a repeat
//! need; a plain worktree with no record refuses with NotAFolder; a repeated make after need keeps the needed
//! path in the cone. tests/folders/root_files.rs covers a top-level file: recorded, with no sparse_add.

use std::path::Path;

use gitio::Git;
use oplog::PidProbe;

use super::{add_needed, need_path};
use crate::make::{git_cone, read_folder_record, with_folders_lock, write_folder_record, FolderError};
use crate::record::FolderRecord;

/// Adds `path` to the folder at `folder`: cleans it through need_path, which refuses a path gitio's sparse_add
/// would refuse only after the record was written, so a refused path writes nothing. Under the folders lock,
/// keyed `folders:need:<git dir>`: reads the folder's record, refusing a worktree with none as NotAFolder;
/// writes the record with `path` added to needed_paths unless it is already there; then adds `path` to the
/// cone with sparse_add, unless the folder's tree holds it as a top-level file (git_cone, S-126): cone mode
/// already checks those out and git's cone-mode add refuses them, so the record alone carries it. Returns the
/// record as it now is.
///
/// The record is written first, so an interrupted need leaves a record naming the path. A repeat need finds it
/// already needed, leaves the record unchanged without an error, and runs sparse_add again (for a path that is
/// not a top-level file), which is safe to repeat; a repeat make refills the folder to the recorded cone, needed
/// paths included (unless its index has unmerged entries, S-36).
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
        let cone = git_cone(git, folder, &record.base, std::slice::from_ref(&clean))?;
        if !cone.is_empty() {
            git.sparse_add(folder, &cone).map_err(FolderError::Git)?;
        }
        Ok(record)
    })
}
