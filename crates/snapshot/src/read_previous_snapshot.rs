//! read_previous_snapshot: the commit a request's snapshot ref names, and its tree.
//! No unit test: it only runs git and holds no decision (S-5). Nagatha's integration tests cover it through snapshot
//! and is_saved, in a throwaway repository:
//! - an absent ref reads as None: is_saved is false before any snapshot, and the first snapshot's parents are [HEAD]
//!   and its op-log move has no old value:
//!   tests/snapshot/lifecycle.rs::first_snapshot_saves_real_tree_parent_trailers_ref_and_attributed_op;
//! - a ref at a snapshot commit reads as that commit and its tree: an unchanged snapshot returns the first
//!   snapshot's commit and tree, and the next snapshot's parents are [HEAD, first]:
//!   lifecycle.rs::unchanged_writes_nothing_then_changed_snapshot_chains_head_and_previous.

use crate::previous_snapshot::PreviousSnapshot;
use crate::SnapshotError;
use gitio::Git;

/// None when `refname` does not exist; otherwise the commit it names and that commit's tree.
pub(crate) fn read_previous_snapshot(git: &Git, refname: &str) -> Result<Option<PreviousSnapshot>, SnapshotError> {
    let Some(commit) = git.read_ref(refname).map_err(SnapshotError::Git)? else {
        return Ok(None);
    };
    let tree = git.commit_info(&commit).map_err(SnapshotError::Git)?.tree;
    Ok(Some(PreviousSnapshot { commit, tree }))
}
