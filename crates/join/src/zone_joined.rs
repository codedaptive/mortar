//! zone_joined: applies one op moving a zone ref and its request ref to the replayed tip, recording the request's
//! base before and after.
//! No unit test: it applies an op against real refs; zone_request_op_tests, base_values_tests and op_applied_tests
//! cover its parts; tests/partner_base covers it in a throwaway repository, recorded_base.rs
//! zone_join_records_bases_and_survives_undo its values, racing_zone_move.rs
//! racing_zone_move_is_protected_before_logging a zone moved sideways since it was read.

use crate::base_values::base_values;
use crate::op_applied::op_applied;
use crate::zone_request_op::zone_request_op;
use crate::{BaseJoinAnswer, JoinRunError, RequestBase};
use gitio::{Git, Oid};
use oplog::{apply_protected_with_values, new_op_id, unix_now, ProtectedRef};

/// One op moving the zone ref from `zone_tip` and the request ref from `from`, both to `to`, recording
/// base_values(`before`, `zone_tip` with no partners) in its values through apply_protected_with_values, the zone
/// ref protected by "join". A zone moved since it was read to a commit `to` does not contain (sideways, or forward
/// past `zone_tip`) is refused as Oplog(Protected) before anything is written to the log, and nothing moves. One
/// moved back to an ancestor of `zone_tip` passes the guard as a fast-forward and fails the transaction on its stale
/// old value (OpFailed), nothing moved. When `to` is `zone_tip` the op leaves the zone move out: nothing to protect.
// Eight arguments: the zone ref and its tip, the request ref and its old tip, `to`, the base before, the actor and
// git; each is one part of the op or its values, and a struct made for this one caller would only rename them.
#[allow(clippy::too_many_arguments)]
pub(crate) fn zone_joined(
    git: &Git,
    zone_ref: String,
    zone_tip: Oid,
    request_ref: &str,
    from: Oid,
    to: Oid,
    before: &RequestBase,
    actor: &str,
) -> Result<BaseJoinAnswer, JoinRunError> {
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let op = zone_request_op(&zone_ref, &zone_tip, request_ref, &from, &to, new_op_id(), actor, unix_now());
    let values = base_values(before, &RequestBase { base: zone_tip.clone(), partners: vec![] });
    op_applied(
        apply_protected_with_values(
            git,
            &common,
            &op,
            &[ProtectedRef { refname: zone_ref.clone(), by: "join".to_string() }],
            &values,
        )
        .map(|valued| valued.outcome),
    )?;
    Ok(BaseJoinAnswer::ZoneJoined {
        zone_ref,
        request_ref: request_ref.to_string(),
        from: zone_tip,
        request_from: from,
        to,
    })
}
