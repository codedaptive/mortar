//! Moves a folder from one tip to another: its files and index by gitio, then its branch by one op-log op.
//! No unit test for the git and op-log path: it runs git and appends to the op log (S-5). move_folder_tests covers
//! the unsaved refusal, branch_move_for_tests where the branch stands against the two tips, moved_or_refused_tests
//! a gitio refusal, and move_op_tests the op as data. Nagatha's integration group is to cover, in a throwaway
//! repository: a moved folder over unsaved work is refused; after join's rebase (the branch already at the new tip)
//! the folder's files and index move and no op is logged; with the branch at the old tip, one folder-move op moves
//! it; a local change in the way refuses with nothing logged and the branch unmoved.

use std::path::Path;

use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now, ApplyOutcome};

use super::branch_move_for::branch_move_for;
use super::moved_or_refused::moved_or_refused;
use super::{move_op, BranchMove};
use crate::make::FolderError;

/// Moves `folder` from tip `old` to tip `new`. `saved` is the caller's answer to whether the folder's work is saved;
/// false is NotSaved before any git run. A detached HEAD is NotAFolder; a branch at neither tip is BranchElsewhere,
/// before anything changes. git refusing over local changes is MoveRefused, with nothing logged and the branch not
/// moved. A branch already at `new` (join moved it with its own op) logs nothing; a branch at `old` is moved by one
/// folder-move op. The caller holds the lock and runs oplog::recover first; move_folder takes no lock itself.
pub fn move_folder(
    git: &Git,
    folder: &Path,
    old: &Oid,
    new: &Oid,
    saved: bool,
    actor: &str,
) -> Result<(), FolderError> {
    if !saved {
        return Err(FolderError::NotSaved {
            path: folder.to_path_buf(),
        });
    }
    let (branch, commit) = git.head(folder).map_err(FolderError::Git)?;
    let Some(branch) = branch else {
        return Err(FolderError::NotAFolder {
            path: folder.to_path_buf(),
            reason: "its HEAD names no branch".to_string(),
        });
    };
    let step = branch_move_for(commit.as_ref(), old, new);
    if step == BranchMove::Elsewhere {
        return Err(FolderError::BranchElsewhere {
            branch,
            at: commit,
            old: old.clone(),
            new: new.clone(),
        });
    }
    moved_or_refused(folder, git.folder_move(folder, old, new).map_err(FolderError::Git)?)?;
    match step {
        BranchMove::Move => move_branch(git, &branch, old, new, actor),
        BranchMove::AlreadyMoved | BranchMove::Elsewhere => Ok(()),
    }
}

/// Applies the folder-move op. Repeated means that exact move was logged before and the branch is back at `old`,
/// so the op is applied again under a key ending in its own op id, as make_branch does; anything but Done, either
/// time, is BranchElsewhere naming where the branch is now.
fn move_branch(git: &Git, branch: &str, old: &Oid, new: &Oid, actor: &str) -> Result<(), FolderError> {
    let common = git.common_dir().map_err(FolderError::Git)?;
    let op = move_op(new_op_id(), actor.to_string(), unix_now(), branch, old, new);
    match apply(git, &common, &op).map_err(FolderError::Oplog)? {
        ApplyOutcome::Done(_) => return Ok(()),
        ApplyOutcome::Repeated(_) => {
            let mut again = move_op(new_op_id(), actor.to_string(), unix_now(), branch, old, new);
            again.key = format!("{}:{}", again.key, again.op_id);
            if let ApplyOutcome::Done(_) = apply(git, &common, &again).map_err(FolderError::Oplog)? {
                return Ok(());
            }
        }
        ApplyOutcome::Failed(_) => {}
    }
    Err(FolderError::BranchElsewhere {
        branch: branch.to_string(),
        at: git.read_ref(branch).map_err(FolderError::Git)?,
        old: old.clone(),
        new: new.clone(),
    })
}
