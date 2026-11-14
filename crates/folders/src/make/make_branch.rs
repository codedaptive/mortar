//! Makes a folder's branch at a commit through the op log, and makes it again when it was deleted after.
//! No unit test: it appends to the op log and runs git (S-5); branch_op_tests and branch_step_after_tests cover
//! its decisions. Nagatha's integration group is to cover, in a throwaway repository: a new branch is made at
//! the commit and its op is in mortar/oplog.jsonl; a second call at the same commit moves nothing and logs no
//! new op; a branch deleted after it was made is made again by an op whose key ends in its op id; a branch
//! already at another commit refuses with BranchExists.

use std::path::Path;

use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now, ApplyOutcome};

use super::branch_step_after::branch_step_after;
use super::{BranchStep, FolderError};
use crate::plan::branch_op;

/// Makes `branch` at `tip` in `git`'s repository through the op log, each op logged under `actor`, as
/// branch_step_after decides from the outcome. A repeat is answered from the log alone, so only then is the branch
/// read. A branch deleted after it was made is made again by an op whose key ends in its own op id, so the log does
/// not answer it as a repeat; anything but Done for that op is BranchExists. The caller runs oplog::recover first,
/// as apply requires.
pub(crate) fn make_branch(git: &Git, actor: &str, common: &Path, branch: &str, tip: &Oid) -> Result<(), FolderError> {
    let op = branch_op(new_op_id(), actor.to_string(), unix_now(), branch, tip);
    let outcome = apply(git, common, &op).map_err(FolderError::Oplog)?;
    let ref_now = match outcome {
        ApplyOutcome::Repeated(_) => git.read_ref(&format!("refs/heads/{branch}")).map_err(FolderError::Git)?,
        ApplyOutcome::Done(_) | ApplyOutcome::Failed(_) => None,
    };
    let exists = || FolderError::BranchExists {
        branch: branch.to_string(),
    };
    match branch_step_after(&outcome, ref_now.as_ref(), tip) {
        BranchStep::Ready => Ok(()),
        BranchStep::Exists => Err(exists()),
        BranchStep::Remake => {
            let mut again = branch_op(new_op_id(), actor.to_string(), unix_now(), branch, tip);
            again.key = format!("{}:{}", again.key, again.op_id);
            match apply(git, common, &again).map_err(FolderError::Oplog)? {
                ApplyOutcome::Done(_) => Ok(()),
                ApplyOutcome::Repeated(_) | ApplyOutcome::Failed(_) => Err(exists()),
            }
        }
    }
}
