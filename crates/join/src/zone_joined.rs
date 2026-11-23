//! zone_joined: applies one op moving a zone ref and its request ref to the replayed tip, recording the request's
//! base before and after.
//! No unit test: it applies an op against real refs; zone_request_op_tests, base_values_tests and op_applied_tests
//! cover its parts; tests/partner_base covers it in a throwaway repository, recorded_base.rs
//! zone_join_records_bases_and_survives_undo its values.

use crate::base_values::base_values;
use crate::op_applied::op_applied;
use crate::zone_request_op::zone_request_op;
use crate::{BaseJoinAnswer, JoinRunError, RequestBase};
use gitio::{Git, Oid};
use oplog::{apply_with_values, new_op_id, unix_now};

/// One op moving the zone ref from `zone_tip` and the request ref from `from`, both to `to`, recording
/// base_values(`before`, `zone_tip` with no partners) in its values through apply_with_values.
/// Not apply_protected, which takes no values: the zone move is a fast-forward by construction, since `to` was
/// replayed onto `zone_tip`, and when the op moves the zone ref (the replay left a commit of the request's own; the
/// zone move is left out when `to` is `zone_tip`) its old value pins the zone ref at `zone_tip`, so a zone moved since
/// it was read fails the transaction (OpFailed) with nothing moved.
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
    op_applied(apply_with_values(git, &common, &op, &values).map(|valued| valued.outcome))?;
    Ok(BaseJoinAnswer::ZoneJoined {
        zone_ref,
        request_ref: request_ref.to_string(),
        from: zone_tip,
        request_from: from,
        to,
    })
}
