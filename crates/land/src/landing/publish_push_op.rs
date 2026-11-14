//! publish_push_op: the one place publish's push intent is built (SPEC-land 04-publish.md, S-60, S-103).
//! No sibling unit test (S-104): every test of it runs gitio's Oid Display and oplog's ChangeId Display through
//! integration_op (S-101). Covered by tests/publish_zit (publishing.rs,
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up: the publish-push records and their move); its unit
//! case is one of Nagatha's RITs (owed).
//! The intent is an ordinary op that push_landing writes and closes itself.

use crate::landing::integration_op;
use crate::squash::LandRefs;
use gitio::Oid;
use oplog::{ChangeId, Op};

/// Kind `publish-push`, exactly one move: `refs/heads/<origin_branch>` from `expected` to `new`, the remote move the
/// guard's pre-push check allows only when an open intent lists it with the same old and new (S-60). Key and reason
/// as integration_op's, so the key carries the op id.
pub(crate) fn publish_push_op(
    refs: &LandRefs,
    expected: &Oid,
    new: &Oid,
    change_id: &ChangeId,
    op_id: String,
    actor: &str,
    at: u64,
) -> Op {
    let refname = format!("refs/heads/{}", refs.origin_branch);
    integration_op("publish-push", &refname, expected, new, change_id, op_id, actor, at)
}
