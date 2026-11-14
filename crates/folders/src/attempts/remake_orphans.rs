//! Re-makes a request's orphan attempt folders (their try branch gone) from their trysnap refs, so keep and
//! drop_attempts meet ordinary attempts (S-96 (2), S-98).
//! No unit test: it runs git and appends to the op log (S-5). remake_plan_tests, remade_message_tests,
//! remade_trailers_tests, refill_cone_tests, branch_step_after_tests and tests/folders/attempt_plans.rs
//! (orphan_attempts) cover its decisions. tests/folders/attempts_remake.rs covers, in a throwaway repository: an
//! orphan with a trysnap ref gets its try branch at a new commit holding the snapshot's tree, with the snapshot's
//! first parent as its one parent, the message `<request>: attempt <k> re-made from its snapshot`, the parent's
//! change ID, request and zone trailers when the parent carries them for this request and zone (none otherwise), and
//! its snapshot's author as author and committer; the folder's index at that tree with its own cone applied again,
//! its files untouched, and one folder-branch op logged; an orphan with no trysnap ref, or a parentless trysnap
//! commit, is OrphanWithoutSnapshot naming its folder, with nothing changed; a repeat after completion returns empty
//! and logs nothing; a repeat after a kill before the op re-makes it; no orphans return empty; keep of a re-made
//! attempt moves the request branch to its commit, with no snapshot commit between. attempts_keep.rs and
//! attempts_drop.rs cover keep of another attempt, and a drop, after remake_orphans finishing, and remove of the
//! request folder then going ahead. attempts_remake_refusals.rs covers an orphan whose folder record is gone:
//! NotAFolder naming it, with nothing changed.

use std::path::{Path, PathBuf};

use gitio::{Git, Oid, Person};
use oplog::{read_mortar_trailers, write_mortar_trailers};

use super::{orphan_attempts, remade_message, remade_trailers, remake_plan, Remake, Snap};
use crate::make::{make_branch, read_folder_record, refill_cone, FolderError};
use crate::plan::{attempt_branch, request_branch, TRYSNAP_REF_PREFIX};

/// Re-makes every orphan attempt folder of `request` in `zone` (orphan_attempts) and returns their folders in k order.
/// The caller holds the lock and runs oplog::recover first (S-16, S-25); remake_orphans takes no lock itself. The names
/// must make a request branch (Plan). No orphans: Ok(empty), nothing changed, which is also what a repeat after a
/// completed call finds. Each orphan k's snapshot T is the commit its trysnap ref `<zone>/<request>/<k>` names; an
/// orphan with no such ref, or whose T has no parent, is OrphanWithoutSnapshot for all of them before anything is
/// touched (remake_plan); so is an orphan whose folder has no Mortar folder record (NotAFolder naming it), and a
/// record whose cone is unusable (Plan, refill_cone). Then, in k order: a new commit R holding T's tree with T's
/// first parent P as its one parent, its message remade_message with remade_trailers drawn from P's trailers
/// (unreadable ones count as none); T's author is R's author and committer, so no git identity is needed and R stays
/// traceable to its snapshot. The folder's index is set to R's tree, which drops a sparse folder's skip-worktree
/// bits, so sparse_add applies the folder's own cone (refill_cone of its record) again with no checkout (HEAD is
/// still unborn), its files untouched, and paths outside the cone do not show as deleted; then one folder-branch op
/// (make_branch) makes the try branch at R, which was absent. A kill before that op leaves the folder an orphan, and
/// a repeat writes the same R again (every input is fixed, so its id is the same) and the op then makes the branch
/// at it; after the op the attempt is ordinary, so a repeat does not meet it again.
pub fn remake_orphans(git: &Git, zone: &str, request: &str, actor: &str) -> Result<Vec<PathBuf>, FolderError> {
    request_branch(zone, request).map_err(FolderError::Plan)?;
    let refs = git
        .list_refs(&format!("refs/heads/try/{zone}/{request}/"))
        .map_err(FolderError::Git)?;
    let worktrees = git.worktree_list().map_err(FolderError::Git)?;
    let orphans = orphan_attempts(zone, request, &refs, &worktrees);
    if orphans.is_empty() {
        return Ok(Vec::new());
    }
    let mut snaps = Vec::new();
    let mut authors: Vec<(u32, Person)> = Vec::new();
    for (k, _) in &orphans {
        let snap = match git
            .read_ref(&format!("{TRYSNAP_REF_PREFIX}{zone}/{request}/{k}"))
            .map_err(FolderError::Git)?
        {
            Some(t) => {
                let info = git.commit_info(&t).map_err(FolderError::Git)?;
                let snap = Snap { tree: info.tree.clone(), first_parent: info.parents.first().cloned() };
                authors.push((*k, info.author));
                Some(snap)
            }
            None => None,
        };
        snaps.push((*k, snap));
    }
    let remakes = remake_plan(&orphans, &snaps)?;
    let cones = remakes.iter().map(|remake| folder_cone(git, &remake.folder)).collect::<Result<Vec<_>, _>>()?;
    let common = git.common_dir().map_err(FolderError::Git)?;
    for (remake, cone) in remakes.iter().zip(&cones) {
        let author = authors.iter().find(|(k, _)| *k == remake.k).map(|(_, author)| author);
        let tip = remade_commit(git, zone, request, remake, author)?;
        git.set_index(&remake.folder, &remake.tree).map_err(FolderError::Git)?;
        git.sparse_add(&remake.folder, cone).map_err(FolderError::Git)?;
        let branch = attempt_branch(zone, request, remake.k).map_err(FolderError::Plan)?;
        make_branch(git, actor, &common, &branch, &tip)?;
    }
    Ok(remakes.into_iter().map(|remake| remake.folder).collect())
}

/// The cone of the folder at `folder`, from its record (refill_cone); a worktree with no record is NotAFolder.
fn folder_cone(git: &Git, folder: &Path) -> Result<Vec<String>, FolderError> {
    let record = read_folder_record(&git.git_dir(folder).map_err(FolderError::Git)?)?.ok_or_else(|| {
        FolderError::NotAFolder { path: folder.to_path_buf(), reason: "a worktree with no Mortar folder record".into() }
    })?;
    refill_cone(&record)
}

/// Writes R for `remake`: its tree, its parent alone, the remade message with the trailers drawn from the parent's,
/// and `author` (its snapshot's) as author and committer.
fn remade_commit(
    git: &Git,
    zone: &str,
    request: &str,
    remake: &Remake,
    author: Option<&Person>,
) -> Result<Oid, FolderError> {
    let parent = git.commit_info(&remake.parent).map_err(FolderError::Git)?;
    let trailers = remade_trailers(zone, request, read_mortar_trailers(git, &parent.message).ok().as_ref());
    let message =
        write_mortar_trailers(git, &remade_message(request, remake.k), &trailers).map_err(FolderError::Git)?;
    git.commit_tree(
        &remake.tree,
        std::slice::from_ref(&remake.parent),
        &message,
        author,
        author,
    )
    .map_err(FolderError::Git)
}
