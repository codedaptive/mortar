//! rebased: applies one op moving a request ref onto its new partner merge, recording the request's base before and
//! after.
//! No unit test: it applies an op against real refs; rebase_op_tests, base_values_tests and op_applied_tests cover
//! its parts; tests/partner_base covers it in a throwaway repository, recorded_base.rs
//! partner_rebase_records_bases_and_survives_undo its values.

use crate::base_values::base_values;
use crate::op_applied::op_applied;
use crate::rebase_op::rebase_op;
use crate::{BaseJoinAnswer, JoinRunError, RequestBase};
use gitio::{Git, Oid};
use oplog::{apply_with_values, new_op_id, unix_now};

/// One op moving the request ref from `from` to `to`, replayed onto the new merge `new.base`, recording
/// base_values(`before`, `new`) in its values through apply_with_values.
pub(crate) fn rebased(
    git: &Git,
    request_ref: &str,
    from: Oid,
    to: Oid,
    before: &RequestBase,
    new: RequestBase,
    actor: &str,
) -> Result<BaseJoinAnswer, JoinRunError> {
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let op = rebase_op(request_ref, &from, &to, &new.base, new_op_id(), actor, unix_now());
    op_applied(apply_with_values(git, &common, &op, &base_values(before, &new)).map(|valued| valued.outcome))?;
    Ok(BaseJoinAnswer::Rebased { request_ref: request_ref.to_string(), from, to, base: new })
}
