//! folders_to_move: step 3 of `mortar restore`: which request and attempt folders on the refs it moves need moving,
//! and which of those hold work that is not saved, checked on a first attempt only (S-113(2), S-32).
//! No unit test: it runs git and snapshot's saved_state (S-5, S-104); restore_refusal_tests cover the refusal decided
//! from its answer, the ZIT crates/cli/tests/run_restore_tests.rs restore's usage refusals, before any lock, and the
//! ZIT crates/cli/tests/run_restore_repeat_tests.rs an unsaved request folder refusing exit 1 with nothing moved
//! and a kill between oplog::restore and the folder move finished by a plain repeat (Repeated, then the move).

use std::path::PathBuf;

use folders::FolderKind;
use gitio::Git;
use oplog::RefMove;
use snapshot::saved_state;

use super::FolderOnRef;
use crate::config::SnapshotSettings;
use crate::report::{failure_report, Report};
use crate::save::snapshot_target_for;

const VERB: &str = "restore";

/// For each request or attempt folder in `folders` whose ref has a move in `moves` (zone folders are never checked,
/// S-115(1)): already in place when its branch is at the move's new value and its status is clean, so nothing moves
/// and nothing is checked; else it needs moving. On a first attempt (`repeated` false) a folder needing a move is
/// unsaved unless snapshot's saved_state, with the config's excludes and cap as save uses them, says saved with no
/// file refused by the cap (S-32). There is no shortcut for a clean status otherwise: a clean folder whose last save
/// is not its snapshot is checked like any other. When `repeated` is true (the moves are a done record's, from
/// restore_moves) the saved check is not run and no folder is unsaved: the first attempt made it before its ref
/// moved, and a snapshot ref named in the restore may already hold its restored value, so comparing the folder with
/// it would read saved work as unsaved. Answers (the folders needing a move, the paths already in place, the unsaved
/// paths), in `folders`' order. A git or snapshot failure is failure_report.
pub fn folders_to_move<'a>(
    git: &Git,
    folders: &'a [FolderOnRef],
    moves: &[RefMove],
    repeated: bool,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<(Vec<&'a FolderOnRef>, Vec<PathBuf>, Vec<PathBuf>), Report> {
    let fail = |what: String| failure_report(VERB, &what);
    let (mut needing, mut in_place, mut unsaved) = (Vec::new(), Vec::new(), Vec::new());
    for folder in folders.iter().filter(|folder| folder.record.kind != FolderKind::Zone) {
        let Some(new) = moves.iter().find(|m| m.refname == folder.refname).map(|m| &m.new) else {
            continue;
        };
        if git.read_ref(&folder.refname).map_err(|error| fail(error.to_string()))?.as_ref() == Some(new) {
            let status = git.status_paths(&folder.path).map_err(|error| fail(error.to_string()))?;
            if status.changed.is_empty() && status.untracked.is_empty() && status.conflicted.is_empty() {
                in_place.push(folder.path.clone());
                continue;
            }
        }
        needing.push(folder);
        if !repeated && is_unsaved(git, folder, settings, actor)? {
            unsaved.push(folder.path.clone());
        }
    }
    Ok((needing, in_place, unsaved))
}

/// The saved check for one folder: unsaved unless saved_state, with the config's excludes and cap, says saved with
/// no file refused by the cap. A folder record naming no request, or a snapshot failure, is failure_report.
fn is_unsaved(git: &Git, folder: &FolderOnRef, settings: &SnapshotSettings, actor: &str) -> Result<bool, Report> {
    let Some(target) = snapshot_target_for(&folder.record, actor) else {
        return Err(failure_report(VERB, &format!("the folder record of {} names no request", folder.path.display())));
    };
    let state = saved_state(git, &folder.path, &target, &settings.excludes, settings.cap_bytes)
        .map_err(|error| failure_report(VERB, &error.to_string()))?;
    Ok(!state.saved || !state.refused.is_empty())
}
