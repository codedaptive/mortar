//! Deletes the refs a sweep picks by one op-log op, then packs the store: closed requests' snapshot refs (attempts'
//! included), joined leftover request branches with their snapshot refs (S-52 (1)), and the caller's extra refs
//! (S-53, S-54).
//! No unit test: it runs git and appends to the op log (S-5); leftover_branches_tests, joined_leftovers_tests,
//! extra_ref_verdict_tests, sweep_pick_tests and closed_snap_refs_tests cover the choice, and sweep_refs_op_tests the
//! op as data. Nagatha's integration group is to cover, in a throwaway repository: a closed request's snapshot ref and
//! its attempts' trysnap refs are deleted and an open request's are kept; a joined req/ branch with no worktree is
//! deleted with its snapshot ref in one folder-sweep-refs op in mortar/oplog.jsonl; a joined branch with a worktree,
//! and one not joined, are kept; a caller's extra ref is deleted, an absent one skipped, and a refs/heads/ or
//! protected one refused with nothing deleted; maintenance runs, also when there is nothing to sweep and no op is
//! logged; a ref moved between its read and the op gives RefMoved with nothing deleted.

use gitio::Git;
use oplog::{apply_protected, new_op_id, unix_now, ApplyOutcome, ProtectedRef};

use super::{joined_leftovers, leftover_branches, sweep_pick, sweep_refs_op};
use crate::make::{ref_moved_from, FolderError};

/// Reads every ref and the branches checked out in any worktree; keeps the leftover_branches results that
/// joined_leftovers chooses from their merge bases with their zone tips (contained, so joined: S-52 (1)); then
/// deletes what sweep_pick picks from them, `extra_refs` and `protected`, by one folder-sweep-refs op through
/// oplog::apply_protected with `protected` (S-54), when it picks any. Then git maintenance runs, whether or not
/// anything was swept. Returns the deleted refnames. A refused extra ref is ExtraRefRefused before any op (S-53,
/// S-54). A Failed op is RefMoved naming the ref its record calls stale, or else the first ref and where it is now;
/// the op deleted nothing. The op's key is unique, so Repeated cannot come back; it is taken as Done. The caller
/// holds the lock and runs oplog::recover first (S-16, S-25); sweep_refs takes no lock itself.
pub fn sweep_refs(
    git: &Git,
    actor: &str,
    extra_refs: &[String],
    protected: &[ProtectedRef],
) -> Result<Vec<String>, FolderError> {
    let refs = git.list_refs("").map_err(FolderError::Git)?;
    let checked_out: Vec<String> =
        git.worktree_list().map_err(FolderError::Git)?.into_iter().filter_map(|w| w.branch).collect();
    let candidates = leftover_branches(&refs, &checked_out)
        .into_iter()
        .map(|leftover| {
            let base = git.merge_base(&leftover.tip, &leftover.zone_tip).map_err(FolderError::Git)?;
            Ok((leftover, base))
        })
        .collect::<Result<Vec<_>, FolderError>>()?;
    let joined = joined_leftovers(candidates);
    let picked = sweep_pick(&refs, &joined, extra_refs, protected)?;
    if let Some((first, _)) = picked.first() {
        let common = git.common_dir().map_err(FolderError::Git)?;
        let op = sweep_refs_op(new_op_id(), actor.to_string(), unix_now(), &picked).map_err(FolderError::Git)?;
        if let ApplyOutcome::Failed(record) =
            apply_protected(git, &common, &op, protected).map_err(FolderError::Oplog)?
        {
            return Err(match ref_moved_from(&record) {
                Some(err) => err,
                None => FolderError::RefMoved {
                    refname: first.clone(),
                    at: git.read_ref(first).map_err(FolderError::Git)?,
                },
            });
        }
    }
    git.maintenance().map_err(FolderError::Git)?;
    Ok(picked.into_iter().map(|(refname, _)| refname).collect())
}
