//! folders_to_move: step 3 of `mr restore`: which request and attempt folders on the refs it moves need moving,
//! which a repeat leaves because their ref has moved on since its done record (ref_moved_on, S-113(3)), and which
//! of those needing a move hold work that is not saved (S-113(2), S-32), on a repeat checked against the snapshot
//! ref's value before the restore (snapshot_value_to_check, S-122(3)).
//! No unit test: it runs git and snapshot's saved_state (S-5, S-104); ref_moved_on_tests cover the moved-on decision,
//! snapshot_value_to_check_tests the snapshot value a check compares against, restore_refusal_tests the refusal
//! decided from its answer, the ZIT crates/cli/tests/run_restore_tests.rs restore's usage refusals, before any lock,
//! and the ZIT crates/cli/tests/run_restore_repeat_tests.rs an unsaved request folder refusing exit 1 with nothing
//! moved, a kill between oplog::restore and the folder move finished by a plain repeat (Repeated, then the move),
//! such a repeat with work added after the kill refusing exit 1 with nothing moved, and a repeat after a later
//! restore leaving the folder and its ref.

use std::path::PathBuf;

use folders::FolderKind;
use gitio::Git;
use oplog::RefMove;
use snapshot::{saved_state, snapshot_ref_name};

use super::{ref_moved_on, snapshot_value_to_check, FolderOnRef};
use crate::config::SnapshotSettings;
use crate::report::{failure_report, Report};
use crate::save::snapshot_target_for;

const VERB: &str = "restore";

/// folders_to_move's answer: the folders needing a move, the paths already in place, the unsaved paths, the paths
/// left alone.
type FoldersToMove<'a> = (Vec<&'a FolderOnRef>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>);

/// For each request or attempt folder in `folders` whose ref has a move in `moves` (zone folders are never checked,
/// S-115(1)), its ref is read once. When `repeated` is true (the moves are a done record's, from restore_moves) and
/// the ref no longer holds the move's new value (ref_moved_on), a later mover has moved it on: the folder is left
/// alone, with no status read and no saved check, so the repeat never reapplies its move (S-113(3)). Otherwise it is
/// already in place when its ref is at the move's new value and its status is clean, so nothing moves and nothing is
/// checked; else it needs moving, and the saved check runs on it, on a first attempt and on a repeat alike
/// (is_unsaved). There is no shortcut for a clean status otherwise: a clean folder whose last save is not its
/// snapshot is checked like any other. A repeat's folder needing a move has its ref at the new value with files not
/// yet moved (a kill between oplog::restore and the folder move); when the restore moved the folder's snapshot ref,
/// the check compares the folder with that ref's old value from the done record, the snapshot the first attempt
/// checked against (S-122(3)). So the crash case with no edits since passes and moves the folder, while work added
/// after the kill refuses. Answers (the folders needing a move, the paths already in place, the unsaved paths, the
/// paths left alone), in `folders`' order. A git or snapshot failure is failure_report.
pub fn folders_to_move<'a>(
    git: &Git,
    folders: &'a [FolderOnRef],
    moves: &[RefMove],
    repeated: bool,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<FoldersToMove<'a>, Report> {
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
        if is_unsaved(git, folder, moves, repeated, settings, actor)? {
            unsaved.push(folder.path.clone());
        }
    }
    Ok((needing, in_place, unsaved, left_alone))
}

/// The saved check for one folder: unsaved when its work is not saved or the cap refused a file (S-32). Saved means
/// the folder's tree from saved_state, with the config's excludes and cap, equals the tree of the snapshot to check:
/// on a repeat whose moves include the folder's snapshot ref, that ref's old value from the done record (none there
/// means no snapshot before the restore, so unsaved; snapshot_value_to_check, S-122(3)); else saved_state's own
/// answer against the ref's present value. The tree equality is the rule snapshot's decide_snapshot applies. A folder
/// record naming no request, or a git or snapshot failure, is failure_report.
fn is_unsaved(
    git: &Git,
    folder: &FolderOnRef,
    moves: &[RefMove],
    repeated: bool,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<bool, Report> {
    let fail = |what: String| failure_report(VERB, &what);
    let Some(target) = snapshot_target_for(&folder.record, actor) else {
        return Err(fail(format!("the folder record of {} names no request", folder.path.display())));
    };
    let state = saved_state(git, &folder.path, &target, &settings.excludes, settings.cap_bytes)
        .map_err(|error| fail(error.to_string()))?;
    let saved = match snapshot_value_to_check(
        repeated,
        &snapshot_ref_name(&target).map_err(|error| fail(error.to_string()))?,
        moves,
    ) {
        Some(RefMove { old: Some(old), .. }) => {
            git.commit_info(old).map_err(|error| fail(error.to_string()))?.tree == state.tree
        }
        Some(RefMove { old: None, .. }) => false,
        None => state.saved,
    };
    Ok(!saved || !state.refused.is_empty())
}
