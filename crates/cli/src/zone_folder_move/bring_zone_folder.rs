//! bring_zone_folder: one zone's folder brought from the commit it stands on to its zone branch's tip. Called by
//! run_after_publish after land's after_publish answered Ok, whether the zone Moved, was Current or Conflicted, and
//! by run_folder_refresh after it set the folder's paths. Composed from gitio and the op log's records only:
//! read_ref, index_tree, commit_info and folder_move (it moves the folder's files and index, never HEAD). No branch
//! op is written, since the branch is already at the tip, and nothing is checked or saved first.
//! The folder stands on the first value, newest first, that zone/<zone> has held (zone_values) whose tree equals its
//! index's tree (find_zone_old); it moves from that tree to the tip's, so git refuses an unstaged edit or untracked
//! file in the way and nothing in the folder changes. An index matching no value holds staged changes, or stands
//! nowhere the branch has been: it is refused, never moved from a guessed commit.
//! An index equal to a tree the zone branch held is taken as the folder standing there. A folder set on purpose to
//! an older zone value V (its index and files at V's tree, as by restoring every path from V) is therefore brought to
//! the tip like a folder lagging behind; nothing is lost, since V's content is a commit the zone branch held.
//! No sibling test: it runs git on a real folder; find_zone_old_tests (the staged-edit loss case as data) and
//! zone_values_tests cover its decisions.
//! The integration tests crates/cli/tests/across_kits/after_publish/moves.rs and refusals.rs cover, with the mortar
//! binary:
//! - the staged-edit loss run: a staged edit at a path the tip changes refuses that folder,
//!   "matches no commit", the edit kept in its index and files, exit 3;
//! - zone b's folder, made before r3 joined, ends with r3's change;
//! - a folder several joins behind is brought to the tip from the commit it stands on;
//! - a folder left on a pre-replay tip by a refused repeat is found through the op log and brought to the tip;
//! - the folders of Current and Conflicted zones are brought to the tip;
//! - a folder already at the tip: nothing moves;
//! - an unstaged edit in the way, and an unmerged index, each refuse that folder with nothing changed;
//! - no branch op is written for a folder move.
//!
//! crates/cli/tests/folder_runs/run_folder_refresh_tests.rs covers it through mr folder refresh: a zone folder
//! brought to the tip after its paths widen, and an unstaged edit in the way and a staged change matching no zone
//! commit each refusing the folder with the work kept.

use std::path::Path;

use gitio::{FolderMoveOutcome, Git, IndexTreeError, Oid};
use oplog::OpRecord;

use super::{find_zone_old, zone_values, FolderRefusal, ZoneFolderOutcome, ZoneOld};

/// `folder`'s outcome: the tip of refs/heads/zone/<zone> is read, then the folder's index's tree (unmerged entries
/// refuse it), then the tip's tree. An index at the tip's tree is AtTip. Else find_zone_old over
/// zone_values(`records`, the zone's ref, the tip), reading each value's tree with commit_info and passing over one it cannot read: NoMatch is refused
/// (NoZoneCommit); From(a value) moves the folder from that value's tree, which is the index's tree, to the tip's
/// tree, refused when git names paths in the way. Any git failure is Failed with its why in the cli's words, an
/// outcome like the others, so after-publish goes on to its other zones whatever this answers.
pub fn bring_zone_folder(git: &Git, records: &[OpRecord], zone: &str, folder: &Path) -> ZoneFolderOutcome {
    let failed = |why: String| ZoneFolderOutcome::Failed { folder: folder.to_path_buf(), why };
    let refused = |why: FolderRefusal| ZoneFolderOutcome::Refused { folder: folder.to_path_buf(), why };
    let zone_ref = format!("refs/heads/zone/{zone}");
    let tip = match git.read_ref(&zone_ref) {
        Ok(Some(tip)) => tip,
        Ok(None) => return failed(format!("{zone_ref} does not exist")),
        Err(_) => return failed(format!("git could not read {zone_ref}")),
    };
    let index_tree = match git.index_tree(folder) {
        Ok(tree) => tree,
        Err(IndexTreeError::Unmerged { paths }) => return refused(FolderRefusal::Unmerged(paths)),
        Err(IndexTreeError::Git(_)) => return failed("git could not read its index".to_string()),
    };
    let Ok(tip_tree) = git.commit_info(&tip).map(|info| info.tree) else {
        return failed(format!("git could not read commit {tip}"));
    };
    let tree_of = |value: &Oid| git.commit_info(value).ok().map(|info| info.tree);
    match find_zone_old(&index_tree, &tip_tree, &zone_values(records, &zone_ref, &tip), tree_of) {
        ZoneOld::AtTip => ZoneFolderOutcome::AtTip { folder: folder.to_path_buf() },
        ZoneOld::NoMatch => refused(FolderRefusal::NoZoneCommit),
        // The value found has the index's tree, so the move is from index_tree.
        ZoneOld::From(_) => match git.folder_move(folder, &index_tree, &tip_tree) {
            Ok(FolderMoveOutcome::Moved) => ZoneFolderOutcome::Moved { folder: folder.to_path_buf() },
            Ok(FolderMoveOutcome::Refused(paths)) => refused(FolderRefusal::InTheWay(paths)),
            Err(_) => failed("git could not move its files and index".to_string()),
        },
    }
}
