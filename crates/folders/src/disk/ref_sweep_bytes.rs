//! How many refs sweep_refs would delete now, and the bytes only they hold, for the disk report's sweep figure.
//! No unit test: it runs git (S-5); the whole choice of refs is sweep's own: joined_leftovers picks the contained
//! branches and sweep_pick the refs, covered by joined_leftovers_tests, sweep_pick_tests, leftover_branches_tests,
//! closed_snap_refs_tests and extra_ref_verdict_tests. Nagatha's integration group is to cover, in a throwaway
//! repository: a closed request's snapshot ref with objects of its own counts them, an open request's does not; a
//! closed request's attempt snapshot ref (refs/mortar/trysnap/<zone>/<request>/<k>) is counted, an open request's is
//! not; a joined req/ branch with no worktree and its snapshot ref are counted; a req/ branch not contained in its
//! zone, and one checked out in a worktree, are not counted; an accepted extra ref is counted, an absent one skipped,
//! a refs/heads/ or protected one refused with ExtraRefRefused; nothing to sweep gives (0, 0); the count matches the
//! refs sweep_refs then deletes on the same repository.

use gitio::{Git, Oid};
use oplog::ProtectedRef;

use crate::make::FolderError;
use crate::sweep::{joined_leftovers, leftover_branches, sweep_pick};

/// Picks the refs sweep_refs would delete from `refs` (list_refs' shape), `extra_refs` and `protected`, the way
/// sweep_refs does: the leftover_branches not checked out in any worktree whose tip is its merge base with its zone tip
/// (joined, S-52 (1); joined_leftovers), then sweep_pick. Returns how many it picked and Git::disk_usage_only over
/// them, an upper bound (S-75). A refused extra ref is ExtraRefRefused, as sweep_refs would refuse it (S-53, S-54).
pub(crate) fn ref_sweep_bytes(
    git: &Git,
    refs: &[(Oid, String)],
    extra_refs: &[String],
    protected: &[ProtectedRef],
) -> Result<(usize, u64), FolderError> {
    let checked_out: Vec<String> =
        git.worktree_list().map_err(FolderError::Git)?.into_iter().filter_map(|w| w.branch).collect();
    let candidates = leftover_branches(refs, &checked_out)
        .into_iter()
        .map(|leftover| {
            let base = git.merge_base(&leftover.tip, &leftover.zone_tip).map_err(FolderError::Git)?;
            Ok((leftover, base))
        })
        .collect::<Result<Vec<_>, FolderError>>()?;
    let refnames: Vec<String> = sweep_pick(refs, &joined_leftovers(candidates), extra_refs, protected)?
        .into_iter()
        .map(|(refname, _)| refname)
        .collect();
    Ok((refnames.len(), git.disk_usage_only(&refnames).map_err(FolderError::Git)?))
}
