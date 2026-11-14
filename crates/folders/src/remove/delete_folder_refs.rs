//! Deletes a removed folder's branch and snapshot ref by one folder-remove op, and answers a repeat of it.
//! No unit test: it appends to the op log and runs git (S-5); remove_op_tests covers the op as data,
//! repeat_step_for_tests the repeat decision and ref_moved_from_tests the stale ref. Nagatha's integration group
//! is to cover it through remove's cases.

use std::path::Path;

use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now, ApplyOutcome};

use super::repeat_step_for::repeat_step_for;
use super::{remove_op, RepeatStep};
use crate::make::{ref_moved_from, FolderError};

/// Applies the folder-remove op for `branch` at `tip` and `snap`, after `folder`'s worktree is gone. Repeated means
/// that exact remove was logged before; repeat_step_for decides from where the branch is now: Removed is done;
/// Again deletes it with an op whose key ends in its own op id, as make_branch does; Moved refuses. A Failed op
/// names the ref its record calls stale and what that ref held; a Failed record with no stale ref, or a second op
/// answered Repeated, names the branch and where it is now. Every such refusal is RemovedRefsLeft for `folder`,
/// since the worktree is already removed and both refs are kept.
pub(crate) fn delete_folder_refs(
    git: &Git,
    folder: &Path,
    branch: &str,
    tip: &Oid,
    snap: Option<(&str, &Oid)>,
    actor: &str,
) -> Result<(), FolderError> {
    match delete_refs(git, branch, tip, snap, actor) {
        Err(FolderError::RefMoved { refname, at }) => Err(FolderError::RemovedRefsLeft {
            path: folder.to_path_buf(),
            branch: branch.to_string(),
            moved: refname,
            at,
        }),
        other => other,
    }
}

/// The op and its repeat; a ref not where it was read is RefMoved, which delete_folder_refs renames.
fn delete_refs(git: &Git, branch: &str, tip: &Oid, snap: Option<(&str, &Oid)>, actor: &str) -> Result<(), FolderError> {
    let common = git.common_dir().map_err(FolderError::Git)?;
    let op = remove_op(new_op_id(), actor.to_string(), unix_now(), branch, tip, snap).map_err(FolderError::Git)?;
    let failed = match apply(git, &common, &op).map_err(FolderError::Oplog)? {
        ApplyOutcome::Done(_) => return Ok(()),
        ApplyOutcome::Failed(record) => Some(record),
        ApplyOutcome::Repeated(_) => {
            let now = git.read_ref(branch).map_err(FolderError::Git)?;
            match repeat_step_for(now.as_ref(), tip) {
                RepeatStep::Removed => return Ok(()),
                RepeatStep::Moved => {
                    return Err(FolderError::RefMoved {
                        refname: branch.to_string(),
                        at: now,
                    })
                }
                RepeatStep::Again => {
                    let mut again = remove_op(new_op_id(), actor.to_string(), unix_now(), branch, tip, snap)
                        .map_err(FolderError::Git)?;
                    again.key = format!("{}:{}", again.key, again.op_id);
                    match apply(git, &common, &again).map_err(FolderError::Oplog)? {
                        ApplyOutcome::Done(_) => return Ok(()),
                        ApplyOutcome::Failed(record) => Some(record),
                        ApplyOutcome::Repeated(_) => None,
                    }
                }
            }
        }
    };
    if let Some(err) = failed.as_ref().and_then(ref_moved_from) {
        return Err(err);
    }
    Err(FolderError::RefMoved {
        refname: branch.to_string(),
        at: git.read_ref(branch).map_err(FolderError::Git)?,
    })
}
