//! move_restored_folders: step 6 of `mr restore`: each folder that needs it is moved with its restored ref.
//! No unit test: it moves real folders through folders::move_folder; restore_move_refusal_report_tests
//! cover what a refused move reports. The integration tests are to cover, with the mortar binary as a child
//! process in a throwaway repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and
//! GIT_CONFIG_NOSYSTEM=1):
//! - a request folder on req/<z>/<r> is moved to the restored tip.
//!   The integration test crates/cli/tests/log_restore_runs/run_restore_repeat_tests.rs covers a kill between oplog::restore and the folder
//!   move finished by a plain repeat (Repeated, then the move), and a repeat after a later restore leaving the folder
//!   and its ref.

use std::path::PathBuf;

use folders::move_folder;
use gitio::Git;
use oplog::OpRecord;

use super::{restore_move_refusal_report, FolderOnRef};
use crate::report::Report;

/// Moves each folder in `needing` (restored as of op `op_id`) from its ref's old value to its new one as `record`
/// (the restore's done record, fresh or Repeated) moved the ref, so a repeat moves from the right tip. `needing` never
/// holds a folder whose ref has moved on since a repeated record (folders_to_move leaves those, ref_moved_on), so a
/// repeat moves only a folder whose ref still holds the record's new value. The folders were checked saved
/// before the restore (on a repeat, by the first attempt), so move_folder is told saved. A folder whose ref the record
/// does not move stays. The first move refused stops the rest: restore_move_refusal_report, naming the folders
/// already moved. Answers the folders moved.
pub fn move_restored_folders(
    git: &Git,
    needing: &[&FolderOnRef],
    op_id: &str,
    record: &OpRecord,
    actor: &str,
) -> Result<Vec<PathBuf>, Report> {
    let mut moved = Vec::new();
    for folder in needing {
        let Some(m) = record.moves.iter().find(|m| m.refname == folder.refname) else {
            continue;
        };
        let refused = |why: &str| restore_move_refusal_report(op_id, &moved, &folder.path, &folder.refname, why);
        let Some(old) = &m.old else {
            return Err(refused("its ref did not exist before the restore, so there is no tip to move it from"));
        };
        move_folder(git, &folder.path, old, &m.new, true, actor, None)
            .map_err(|error| refused(&error.to_string()))?;
        moved.push(folder.path.clone());
    }
    Ok(moved)
}
