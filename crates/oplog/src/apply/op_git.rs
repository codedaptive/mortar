//! The git an op's ref moves run through: the caller's git with MORTAR_OP naming the op.

use gitio::Git;

use super::MORTAR_OP_ENV;

/// A clone of `git` with `(MORTAR_OP, op_id)` appended as the last entry of its env; `git` is not changed.
///
/// apply runs its ref transaction through this. It is also how a caller's push inside an open intent carries
/// MORTAR_OP: write the intent (intent_record, append_op_record), push with op_git's git, then close
/// (close_record). There is no separate push entry point.
pub fn op_git(git: &Git, op_id: &str) -> Git {
    let mut op = git.clone();
    op.env.push((MORTAR_OP_ENV.into(), op_id.into()));
    op
}
