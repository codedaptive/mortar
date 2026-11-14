//! snapshot_formatted: run the caller's formatter over a folder's changed files, then save the folder's files as a
//! commit on the request's snapshot ref.
//! No unit test: it runs git and the formatter and appends to the op log, and holds no decision of its own (S-5);
//! decide_snapshot, snapshot_ref_name, snapshot_parents, snapshot_trailers, snapshot_op, snapshot_from_apply,
//! formatter_paths, formatter_argv, decide_formatter_wait and decide_formatter_result are its decisions and have their
//! tests.
//! Nagatha's integration group should cover, in a throwaway repository: a first snapshot writes a commit with
//! parents [HEAD], message "snapshot" with Mortar-Request and Mortar-Zone trailers, moves
//! refs/mortar/snap/<zone>/<request> to it and leaves a done op record; a second snapshot of an unchanged folder
//! writes nothing and returns the same commit with written false; a snapshot after a change chains parents
//! [HEAD, previous snapshot]; a ref moved by someone else between the read and the apply gives RefMoved and is left
//! as found; a new 6 MB file over DEFAULT_SIZE_CAP is left out of the commit's tree and listed in refused, on a
//! written and on an unchanged answer.
//! The formatter cases are run_formatter's header's: tests/snapshot/formatter.rs, formatter_paths.rs and
//! formatter_timeout.rs cover them, except the S-37 process-group cases still to be written.

use crate::decide_snapshot::decide_snapshot;
use crate::folder_tree::folder_tree;
use crate::read_previous_snapshot::read_previous_snapshot;
use crate::run_formatter::run_formatter;
use crate::snapshot_decision::SnapshotDecision;
use crate::snapshot_from_apply::snapshot_from_apply;
use crate::snapshot_op::snapshot_op;
use crate::snapshot_parents::snapshot_parents;
use crate::snapshot_ref_name::snapshot_ref_name;
use crate::snapshot_trailers::snapshot_trailers;
use crate::{Formatter, SnapshotError, SnapshotOutcome, SnapshotTarget};
use gitio::Git;
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// Runs `formatter` over `folder`'s changed files, then saves `folder`'s files, minus the built-in excludes,
/// `extra_excludes` and each new file (not in HEAD) over `size_cap` bytes, listed in refused, as a commit on
/// `refs/mortar/snap/<zone>/<request>` (S-19), with parents [HEAD] for the request's first snapshot and
/// [HEAD, previous snapshot] after that. `git` gives the binary and environment; every run is in `folder`, which
/// must be the folder's top.
///
/// The formatter runs over run_formatter's files (the changed then untracked regular files the snapshot would save
/// differently from HEAD, not conflicted), so the snapshot saves the formatted files; the run is in `formatted`.
/// `formatter` is taken as the caller has it configured: None runs nothing, not even git status, and gives
/// formatted None (run_formatter returns first); no file to format gives formatted None too. A formatter that
/// fails, cannot start or runs past its timeout (killed and reaped) is listed in formatted and does not stop the
/// snapshot. The folder's index is untouched
/// (gitio's snapshot_tree builds the tree in a temp index); its files change only by what the formatter writes.
///
/// A tree equal to the last snapshot's writes nothing and returns that snapshot with written false; that is the
/// only written-false answer. Otherwise the ref moves through one op log entry and the answer is written true; a
/// ref not at the value read gives RefMoved, and an op key the log already holds as done gives OpRepeated; in both
/// nothing moved.
///
/// Callers wanting the default cap pass DEFAULT_SIZE_CAP. The caller runs oplog::recover and holds the lock around
/// this call, as every command does (S-16).
pub fn snapshot_formatted(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    extra_excludes: &[String],
    size_cap: u64,
    formatter: Option<&Formatter>,
) -> Result<SnapshotOutcome, SnapshotError> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let refname = snapshot_ref_name(target)?;
    let formatted = run_formatter(&git, folder, formatter, extra_excludes, size_cap)?;
    let (head, tree, refused) = folder_tree(&git, folder, extra_excludes, size_cap)?;
    let previous = read_previous_snapshot(&git, &refname)?;
    if let SnapshotDecision::Unchanged(commit) = decide_snapshot(&tree, previous.as_ref()) {
        return Ok(SnapshotOutcome { commit, tree, written: false, refused, formatted });
    }
    let previous_commit = previous.as_ref().map(|p| &p.commit);
    let message = git
        .with_trailers("snapshot\n", &snapshot_trailers(target))
        .map_err(SnapshotError::Git)?;
    let commit = git
        .commit_tree(&tree, &snapshot_parents(&head, previous_commit), &message, None, None)
        .map_err(SnapshotError::Git)?;
    let common = git.common_dir().map_err(SnapshotError::Git)?;
    let op = snapshot_op(&refname, previous_commit, &commit, &target.actor, new_op_id(), unix_now());
    snapshot_from_apply(apply(&git, &common, &op).map_err(SnapshotError::Oplog)?, commit, tree, refused, formatted)
}
