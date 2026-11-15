//! folders_to_move: step 3 of `mortar restore`: which request and attempt folders on the refs it moves need moving,
//! which a repeat leaves because their ref has moved on since its done record (ref_moved_on, S-113(3)), and which
//! hold work that is not saved, checked on a first attempt only (S-113(2), S-32).
//! No unit test: it runs git and snapshot's saved_state (S-5, S-104); ref_moved_on_tests cover the moved-on decision,
//! restore_refusal_tests the refusal decided from its answer, the ZIT crates/cli/tests/run_restore_tests.rs restore's
//! usage refusals, before any lock, and the ZIT crates/cli/tests/run_restore_repeat_tests.rs an unsaved request
//! folder refusing exit 1 with nothing moved, a kill between oplog::restore and the folder move finished by a plain
//! repeat (Repeated, then the move), and a repeat after a later restore leaving the folder and its ref.

use std::path::PathBuf;

use folders::FolderKind;
use gitio::Git;
use oplog::RefMove;
use snapshot::saved_state;

use super::{ref_moved_on, FolderOnRef};
use crate::config::SnapshotSettings;
use crate::report::{failure_report, Report};
use crate::save::snapshot_target_for;

const VERB: &str = "restore";

/// For each request or attempt folder in `folders` whose ref has a move in `moves` (zone folders are never checked,
/// S-115(1)), its ref is read once. When `repeated` is true (the moves are a done record's, from restore_moves) and
/// the ref no longer holds the move's new value (ref_moved_on), a later mover has moved it on: the folder is left
/// alone, with no status read and no saved check, so the repeat never reapplies its move (S-113(3)). Otherwise it is
/// already in place when its ref is at the move's new value and its status is clean, so nothing moves and nothing is
/// checked; else it needs moving. On a first attempt (`repeated` false) a folder needing a move is unsaved unless
/// snapshot's saved_state, with the config's excludes and cap as save uses them, says saved with no file refused by
/// the cap (S-32). There is no shortcut for a clean status otherwise: a clean folder whose last save is not its
/// snapshot is checked like any other. On a repeat the saved check is not run and no folder is unsaved: the first
/// attempt made it before its ref moved, and a snapshot ref named in the restore may already hold its restored value,
/// so comparing the folder with it would read saved work as unsaved. A repeat's folder needing a move has its ref at
/// the new value with files not yet moved (a kill between oplog::restore and the folder move). Answers (the folders
/// needing a move, the paths already in place, the unsaved paths, the paths left alone), in `folders`' order. A git
/// or snapshot failure is failure_report.
pub fn folders_to_move<'a>(
    git: &Git,
    folders: &'a [FolderOnRef],
    moves: &[RefMove],
    repeated: bool,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<(Vec<&'a FolderOnRef>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>), Report> {
    let fail = |what: String| failure_report(VERB, &what);
    let (mut needing, mut in_place, mut unsaved, mut left_alone) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for folder in folders.iter().filter(|folder| folder.record.kind != FolderKind::Zone) {
        let Some(new) = moves.iter().find(|m| m.refname == folder.refname).map(|m| &m.new) else {
            continue;
        };
        let current = git.read_ref(&folder.refname).map_err(|error| fail(error.to_string()))?;
        if ref_moved_on(repeated, current.as_ref(), new) {
            left_alone.push(folder.path.clone());
            continue;
        }
        if current.as_ref() == Some(new) {
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
    Ok((needing, in_place, unsaved, left_alone))
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
