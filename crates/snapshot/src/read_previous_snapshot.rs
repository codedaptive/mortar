//! read_previous_snapshot: the commit a request's snapshot ref names, and its tree.
//! No unit test: it only runs git and holds no decision (S-5). Nagatha's integration group should cover, in a
//! throwaway repository: an absent ref reads as None; a ref at a snapshot commit reads as that commit and its tree.

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
