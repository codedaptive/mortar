//! join: joins a request ref to its zone. It takes no lock: the caller holds join-<zone> and has run recover.
//! No unit test: it is join_recording with no base known, whose header lists the tests of its decisions and the
//! integration cases; tests/partner_base/recorded_base.rs plain_join_without_known_base_is_not_recorded covers that
//! its rebase op records no base values.

use crate::join_recording::join_recording;
use crate::{JoinAnswer, JoinRunError};
use gitio::Git;
use std::path::Path;

/// Joins `request_ref` (a full ref name) to zone `zone`, whose ref is `refs/heads/zone/<zone>`: reads both tips and
/// their merge base, and decides. A request tip already in the zone answers Joined and moves nothing. A zone tip
/// behind the request moves the zone ref forward by one op, applied with the zone ref protected, so a move that is
/// not a fast-forward is refused. Otherwise the request's commits after the fork are replayed onto the zone tip:
/// clean, the request ref moves to the replayed tip by one op, the zone does not move, and Rebased carries the
/// request ref's old and new tips so the caller can move the request's folder, with no folder_tree (the
/// folder moves from `from`); conflicted, no ref moves and
/// the state is written to `folder`'s git dir as mortar-join.json. Ops name `actor` as who moved the refs and record
/// no base values, since join is not told the base the request was made on. Each op's key carries its op id, so a
/// done record already holding that key means this join moved nothing: it is refused (OpRepeated), never answered
/// as a move.
pub fn join(git: &Git, folder: &Path, zone: &str, request_ref: &str, actor: &str) -> Result<JoinAnswer, JoinRunError> {
    join_recording(git, folder, zone, request_ref, actor, None)
}
