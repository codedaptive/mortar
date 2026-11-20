//! overlap_for: one named open request's overlaps with each other open request.
//! No unit test: it runs git; pair_overlaps_for_tests covers its decisions. Its integration cases are
//! RITs still to be written: the named request overlapping another gives that pair with the shared path; no overlap
//! gives an empty answer; a name not among the requests gives an empty answer; overlap_for moves no ref and writes
//! nothing (refs and worktree unchanged after); a missing base or tip commit gives JoinError::Git.

use crate::pair_overlaps_for::pair_overlaps_for;
use crate::{JoinError, OpenRequest, OverlapPair};
use gitio::Git;

/// For each request, the files changed between its base and its tip (a rename counts as its old and new path);
/// then the pairs holding the request named `name` that share at least one path, as pair_overlaps_for orders
/// them. Every request's files are read, even when `name` is not among them, so a git failure is never hidden.
/// The caller picks each request's base. Read only: it moves no ref, writes nothing and takes no lock.
/// The first git failure is returned as JoinError::Git.
pub fn overlap_for(git: &Git, requests: &[OpenRequest], name: &str) -> Result<Vec<OverlapPair>, JoinError> {
    let changed = requests
        .iter()
        .map(|r| Ok((r.name.clone(), git.changed_files(&r.base, &r.tip)?)))
        .collect::<Result<Vec<_>, JoinError>>()?;
    Ok(pair_overlaps_for(&changed, name))
}
