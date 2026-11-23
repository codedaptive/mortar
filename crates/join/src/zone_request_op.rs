//! zone_request_op: the op that moves a zone ref and a request ref to one commit together.
//! zone_request_op_tests covers every field, both moves in order, and a key unique to each op id.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `join-zone-request`: the move of `zone_ref` from `zone_from` to `to`, then the move of
/// `request_ref` from `request_from` to `to`, in one op so neither ref moves without the other. The zone
/// move is left out when `zone_from` is `to` (the replay left no commit of the request's own), since a
/// move to where the ref already is changes nothing. The key is
/// `join-zone-request:<zone_ref>:<zone_from>:<request_ref>:<request_from>:<to>:<op_id>`: it carries the op
/// id so each join attempt is its own op, as fast_forward_op's does. The reason names both refs and their three
/// commit ids.
// Eight arguments: each ref's name and old tip, `to`, the op id, the actor and the time; each is one field of the op,
// and a struct made for this one caller would only rename them.
#[allow(clippy::too_many_arguments)]
pub(crate) fn zone_request_op(
    zone_ref: &str,
    zone_from: &Oid,
    request_ref: &str,
    request_from: &Oid,
    to: &Oid,
    op_id: String,
    actor: &str,
    at: u64,
) -> Op {
    let zone_move = RefMove { refname: zone_ref.to_string(), old: Some(zone_from.clone()), new: to.clone() };
    let request_move =
        RefMove { refname: request_ref.to_string(), old: Some(request_from.clone()), new: to.clone() };
    let moves = if zone_from == to { vec![request_move] } else { vec![zone_move, request_move] };
    Op {
        // Before op_id, which the next field moves.
        key: format!("join-zone-request:{zone_ref}:{zone_from}:{request_ref}:{request_from}:{to}:{op_id}"),
        op_id,
        kind: "join-zone-request".to_string(),
        actor: actor.to_string(),
        reason: format!("join moves {zone_ref} from {zone_from} and {request_ref} from {request_from} to {to}"),
        at,
        moves,
    }
}
