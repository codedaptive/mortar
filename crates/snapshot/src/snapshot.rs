//! snapshot: save a folder's files as a commit on the request's snapshot ref.
//! No unit test: it runs git and appends to the op log, and holds no decision of its own (S-5); decide_snapshot,
//! snapshot_ref_name, snapshot_parents, snapshot_trailers, snapshot_op and snapshot_from_apply are its decisions
//! and have their tests.
//! Nagatha's integration group should cover, in a throwaway repository: a first snapshot writes a commit with
//! parents [HEAD], message "snapshot" with Mortar-Request and Mortar-Zone trailers, moves
//! refs/mortar/snap/<zone>/<request> to it and leaves a done op record; a second snapshot of an unchanged folder
//! writes nothing and returns the same commit with written false; a snapshot after a change chains parents
//! [HEAD, previous snapshot]; a ref moved by someone else between the read and the apply gives RefMoved and is left
//! as found; a new 6 MB file over DEFAULT_SIZE_CAP is left out of the commit's tree and listed in refused, on a
//! written and on an unchanged answer.

use crate::decide_snapshot::decide_snapshot;
use crate::folder_tree::folder_tree;
use crate::read_previous_snapshot::read_previous_snapshot;
use crate::snapshot_decision::SnapshotDecision;
use crate::snapshot_from_apply::snapshot_from_apply;
use crate::snapshot_op::snapshot_op;
use crate::snapshot_parents::snapshot_parents;
use crate::snapshot_ref_name::snapshot_ref_name;
use crate::snapshot_trailers::snapshot_trailers;
use crate::{SnapshotError, SnapshotOutcome, SnapshotTarget};
use gitio::Git;
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// Saves `folder`'s files, minus the built-in excludes, `extra_excludes` and each new file (not in HEAD) over
/// `size_cap` bytes, listed in refused, as a commit on `refs/mortar/snap/<zone>/<request>` (S-19), with parents
/// [HEAD] for the request's first snapshot and [HEAD, previous snapshot] after that. `git` gives the binary and
/// environment; every run is in `folder`, which must be the folder's top. The folder's own index and files are
/// untouched: gitio's snapshot_tree builds the tree in a temp index.
///
/// A tree equal to the last snapshot's writes nothing and returns that snapshot with written false; that is the
/// only written-false answer. Otherwise the ref moves through one op log entry and the answer is written true; a
/// ref not at the value read gives RefMoved, and an op key the log already holds as done gives OpRepeated; in both
/// nothing moved.
///
/// Callers wanting the default cap pass DEFAULT_SIZE_CAP. The caller runs oplog::recover and holds the lock around
/// this call, as every command does (S-16).
pub fn snapshot(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SnapshotOutcome, SnapshotError> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let refname = snapshot_ref_name(target)?;
    let (head, tree, refused) = folder_tree(&git, folder, extra_excludes, size_cap)?;
    let previous = read_previous_snapshot(&git, &refname)?;
    if let SnapshotDecision::Unchanged(commit) = decide_snapshot(&tree, previous.as_ref()) {
        return Ok(SnapshotOutcome { commit, tree, written: false, refused });
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
    snapshot_from_apply(apply(&git, &common, &op).map_err(SnapshotError::Oplog)?, commit, tree, refused)
}
