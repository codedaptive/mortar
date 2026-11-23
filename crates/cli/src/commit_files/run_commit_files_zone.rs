//! run_commit_files_zone: `mr commit-files` in a zone folder, named paths committed onto zone/<zone>'s tip by
//! snapshot's commit_paths and the zone branch moved to that commit by one op, under the join-<zone> lock.
//! The folder must stand on the tip before commit_paths runs, since commit_paths writes blobs and sets the folder's
//! index; the named paths are judged by commit_paths alone, so a zone folder gets snapshot's refusals as the
//! integration folder does. Before commit_paths, the pending marker (mortar-commit-files-pending in the folder's git
//! dir) names the tip and the paths, so a repeat after a stop between commit_paths' index write and the op can tell
//! its own staged paths from a lagging folder's (read_pending_commit).
//! No sibling test: it takes a real lock and runs git, snapshot, the op log and the marker file;
//! zone_off_tip_refusal_tests, stopped_run_refusal_tests, moved_since_read_refusal_tests,
//! commit_paths_error_report_tests, zone_commit_op_tests and commit_files_report_tests cover its decisions. The
//! integration test crates/cli/tests/folder_runs/run_commit_files_tests.rs covers it through the binary: only the
//! named paths committed, the unnamed edit kept in the folder, one op moving zone/<zone> as the actor; a repeat
//! answers same with the ref unmoved; a folder behind its zone's tip refused, next mr folder refresh; a repeat after a
//! stopped run commits once and removes the marker (commit_files_zone_repeat_after_a_stopped_run_commits_once); a
//! lagging folder with a named path staged refused with the tip unchanged
//! (commit_files_zone_lagging_with_a_staged_named_path_is_refused).

use std::fs;
use std::path::PathBuf;

use gitio::{Git, IndexTreeError, Oid};
use oplog::{read_op_log, ApplyOutcome, KillPidProbe, LockName, RefMove};
use snapshot::{CommitPathsError, CommitPathsOutcome};

use super::read_pending_commit::read_pending_commit;
use super::write_pending_commit::write_pending_commit;
use super::{
    commit_files_report, commit_paths_error_report, moved_since_read_refusal, stopped_run_refusal, zone_commit_op,
    zone_off_tip_refusal, PendingCommit,
};
use crate::context::FolderContext;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;
use crate::zone_folder_move::{find_zone_old, zone_values, ZoneOld};

const VERB: &str = "commit-files";
/// The pending marker's file name in the zone folder's own git dir.
const MARKER: &str = "mortar-commit-files-pending";

/// Under join-<zone> after recover (locked_act), keyed by --key or op_key("commit-files", the folder) for the lock
/// holder: the tip checked (tip_for_commit), then the commit written and the branch moved (commit_and_move).
pub(super) fn run_commit_files_zone(
    context: &ShellContext,
    folder: &FolderContext,
    paths: &[PathBuf],
    message: &str,
) -> Report {
    let git = Git::new(&folder.toplevel);
    let zone = folder.record.zone.as_str();
    let key = op_key(context.args.key.as_deref(), VERB, &folder.toplevel, &[]);
    locked_act(&git, &folder.common, &LockName::Join(zone.to_string()), &key, &KillPidProbe, VERB, || {
        let (tip, index_at_tip) = tip_for_commit(&git, folder, paths)?;
        commit_and_move(context, &git, folder, &tip, index_at_tip, paths, message)
    })
    .unwrap_or_else(|report| report)
}

/// zone/<zone>'s tip, once the folder is found to stand on it, and whether the index's tree is the tip's. The
/// folder's index tree (unmerged entries or git failing: failure_report), the tip's tree and the op log's values of
/// the zone branch give find_zone_old's answer. AtTip goes on; From is zone_off_tip_refusal's. NoMatch with a marker
/// that counts (read_pending_commit) goes on when the marker's tip is the tip and is stopped_run_refusal's when the
/// branch has moved since; without one it is zone_off_tip_refusal's. No commit, ref or index entry is written;
/// reading the index's tree may write tree objects.
fn tip_for_commit(git: &Git, folder: &FolderContext, paths: &[PathBuf]) -> Result<(Oid, bool), Report> {
    let (zone, top, marker) = (folder.record.zone.as_str(), folder.toplevel.as_path(), folder.git_dir.join(MARKER));
    let failed = |what: String| failure_report(VERB, &format!("{what}; no commit was written and no ref moved"));
    let zone_ref = format!("refs/heads/zone/{zone}");
    let tip = match git.read_ref(&zone_ref) {
        Ok(Some(tip)) => tip,
        Ok(None) => return Err(failed(format!("{zone_ref} does not exist"))),
        Err(_) => return Err(failed(format!("git could not read {zone_ref}"))),
    };
    let shown = top.display();
    let index_tree = git.index_tree(top).map_err(|error| match error {
        IndexTreeError::Unmerged { .. } => failed(format!("the zone folder {shown}'s index holds unmerged paths")),
        IndexTreeError::Git(_) => failed(format!("git could not read the zone folder {shown}'s index")),
    })?;
    let tip_tree = git.commit_info(&tip).map_err(|_| failed(format!("git could not read commit {tip}")))?.tree;
    let records = read_op_log(&folder.common).map_err(|_| failed("the op log could not be read".to_string()))?.records;
    let tree_of = |value: &Oid| git.commit_info(value).ok().map(|info| info.tree);
    let old = find_zone_old(&index_tree, &tip_tree, &zone_values(&records, &zone_ref, &tip), tree_of);
    let staged_on = (old == ZoneOld::NoMatch).then(|| read_pending_commit(git, &marker, &index_tree, paths)).flatten();
    match (staged_on, zone_off_tip_refusal(zone, top, &tip, &old)) {
        (Some(staged_on), _) if staged_on == tip => Ok((tip, index_tree == tip_tree)),
        (Some(staged_on), _) => Err(stopped_run_refusal(zone, top, &staged_on, &tip, paths)),
        (None, Some(refusal)) => Err(refusal),
        (None, None) => Ok((tip, index_tree == tip_tree)),
    }
}

/// The marker written (write_pending_commit), then snapshot's commit_paths onto `tip` (refused or failed:
/// commit_paths_error_report; Same: commit_files_report with nothing moved). Committed: zone_commit_op keyed by --key
/// or op_key("commit-files", the folder, the move), applied with the protected refs. Done or Repeated is
/// commit_files_report to the commit the record moved zone/<zone> to; Failed is moved_since_read_refusal; the op log
/// failing is failure_report.
/// The marker is removed after Same with the index at the tip's tree (`index_at_tip`), a commit_paths error other
/// than IndexNotSet (no commit written) and Done or Repeated. It is kept after Same with the index elsewhere (a
/// stopped run's paths still staged, the files reverted, so a repeat finds it and answers same again), IndexNotSet, a
/// Failed op or an op log error, where the index may hold the named paths and a repeat must tell. A failed removal is
/// ignored: a marker left behind counts only while the index differs from its tip at most at its paths, and after
/// Done or Repeated the next run is AtTip, which reads no marker, and overwrites it.
fn commit_and_move(
    context: &ShellContext,
    git: &Git,
    folder: &FolderContext,
    tip: &Oid,
    index_at_tip: bool,
    paths: &[PathBuf],
    message: &str,
) -> Result<Report, Report> {
    let (zone, top, marker) = (folder.record.zone.as_str(), folder.toplevel.as_path(), &folder.git_dir.join(MARKER));
    let zone_ref = format!("refs/heads/zone/{zone}");
    let next = format!("mr squash {zone} --actor <name> carries it in the zone's next squash");
    write_pending_commit(marker, &PendingCommit { tip: tip.clone(), paths: paths.to_vec() })?;
    let commit = match snapshot::commit_paths(git, top, tip, paths, message) {
        Ok(CommitPathsOutcome::Committed { commit, .. }) => commit,
        Ok(CommitPathsOutcome::Same) => {
            if index_at_tip {
                let _ = fs::remove_file(marker);
            }
            return Ok(commit_files_report(&zone_ref, tip, None, paths, next));
        }
        Err(error @ CommitPathsError::IndexNotSet { .. }) => return Err(commit_paths_error_report(VERB, &error)),
        Err(error) => {
            let _ = fs::remove_file(marker);
            return Err(commit_paths_error_report(VERB, &error));
        }
    };
    let one = RefMove { refname: zone_ref.clone(), old: Some(tip.clone()), new: commit.clone() };
    let key = op_key(context.args.key.as_deref(), VERB, top, &[one]);
    let op = zone_commit_op(zone, tip, &commit, &key, &context.actor, oplog::new_op_id(), oplog::unix_now());
    match oplog::apply_protected(git, &folder.common, &op, &context.protected) {
        Ok(ApplyOutcome::Done(record) | ApplyOutcome::Repeated(record)) => {
            let _ = fs::remove_file(marker);
            let moved = record.moves.iter().find(|one| one.refname == zone_ref).map_or(&commit, |one| &one.new);
            Ok(commit_files_report(&zone_ref, tip, Some(moved), paths, next))
        }
        Ok(ApplyOutcome::Failed(_)) => Err(moved_since_read_refusal(zone, top, tip, paths)),
        Err(_) => Err(failure_report(
            VERB,
            &format!(
                "the op log could not move {zone_ref} to {commit}; the commit was written and the folder's index set \
                 for the named paths"
            ),
        )),
    }
}
