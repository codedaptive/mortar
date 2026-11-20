//! overlap: which open requests of a zone change the same paths, before anyone joins.
//! No unit test: it runs git; pair_overlaps_tests covers its decisions. Integration tests are to cover, in a
//! throwaway repository: two requests changing the same file are listed as a pair with that path; requests changing
//! different files give no pair; a request whose tip is its snapshot commit (unsaved work) is compared by that
//! commit; a rename shows its old and new path; overlap moves no ref and writes nothing (refs and worktree unchanged
//! after); a missing base or tip commit gives JoinError::Git.

use crate::pair_overlaps::pair_overlaps;
use crate::{JoinError, OpenRequest, OverlapPair};
use gitio::Git;

/// For each request, the files changed between its base and its tip (a rename counts as its old and new path);
/// then every pair of requests sharing at least one path, as pair_overlaps orders them. Read only: it moves no
/// ref, writes nothing and takes no lock. The first git failure is returned as JoinError::Git.
pub fn overlap(git: &Git, requests: &[OpenRequest]) -> Result<Vec<OverlapPair>, JoinError> {
    let changed = requests
        .iter()
        .map(|r| Ok((r.name.clone(), git.changed_files(&r.base, &r.tip)?)))
        .collect::<Result<Vec<_>, JoinError>>()?;
    Ok(pair_overlaps(&changed))
}
