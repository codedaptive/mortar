//! Gives a zone's standing folder a new list of paths: its record and its cone.
//! No unit test: it takes the folders lock, runs git and reads and writes the record file; zone_paths_record_tests
//! and skip_refill_tests cover its decisions. tests/folders/zone_paths.rs covers it in a throwaway repository:
//! widening and narrowing replace the record's work-order paths, clear its needed paths and set the cone, and a
//! repeat with the same paths changes nothing; a request folder, a plain worktree and a folder with unmerged
//! entries are refused with nothing changed; a modified tracked file and an untracked file under a dropped path
//! stay on disk.

use std::path::Path;

use gitio::Git;
use oplog::PidProbe;

use super::skip_refill::skip_refill;
use super::{git_cone, read_folder_record, with_folders_lock, write_folder_record, zone_paths_record, FolderError};
use crate::plan::sparse_paths;
use crate::record::FolderRecord;

/// Sets the zone folder at `folder` to hold `paths`: the zone's paths, its in-repo dependencies and the root build
/// files, given by the caller and cleaned by sparse_paths, which refuses a bad path as Plan before anything runs.
/// Under the folders lock, keyed `folders:zone-paths:<git dir>`: reads the folder's record, refusing a worktree
/// with none as NotAFolder and a record that is not a zone folder's as DifferentFolder; refuses a folder whose
/// index has unmerged entries as NotSaved; then writes the record with work_order_paths set to the cleaned paths
/// and needed_paths cleared, unless it already says so, and sets the cone to those paths without the top-level
/// files (git_cone; cone mode checks those out anyway). Every refusal before the record is written leaves the
/// record, index and files as they were; a git failure after it leaves the record naming the new paths, and a
/// repeat with the same paths finishes the cone. Returns the record as it now is.
///
/// For zone folders only. Moving the folder to zone/<zone>'s tip is not its job: cli's after-publish move does
/// that.
///
/// The record is written first, so an interrupted call leaves a record naming the new paths, and a repeat with the
/// same paths finishes it: the record is left unchanged and the cone set again, which is safe to repeat.
///
/// A path the new list drops leaves the cone, but git keeps modified or untracked files under it: the folder
/// keeps them on disk, and Mortar deletes nothing.
///
/// The caller runs oplog::recover first, as apply requires: with_folders_lock may apply a takeover op or
/// guard-removal ops.
pub fn set_zone_paths(
    git: &Git,
    folder: &Path,
    paths: &[String],
    pids: &dyn PidProbe,
) -> Result<FolderRecord, FolderError> {
    let cone = sparse_paths(paths, &[]).map_err(FolderError::Plan)?;
    let git_dir = git.git_dir(folder).map_err(FolderError::Git)?;
    let common = git.common_dir().map_err(FolderError::Git)?;
    with_folders_lock(git, &common, &format!("folders:zone-paths:{}", git_dir.display()), pids, || {
        let existing = read_folder_record(&git_dir)?.ok_or_else(|| FolderError::NotAFolder {
            path: folder.to_path_buf(),
            reason: "a worktree with no Mortar folder record".to_string(),
        })?;
        let record = zone_paths_record(folder, &existing, &cone)?;
        if skip_refill(&git.status_paths(folder).map_err(FolderError::Git)?) {
            return Err(FolderError::NotSaved {
                path: folder.to_path_buf(),
            });
        }
        if record != existing {
            write_folder_record(&git_dir, &record)?;
        }
        git.sparse_set(folder, &git_cone(git, folder, &record.base, &record.work_order_paths)?)
            .map_err(FolderError::Git)?;
        Ok(record)
    })
}
