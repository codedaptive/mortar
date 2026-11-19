//! publish_push_op: the one place publish's push intent is built.
//! No sibling unit test: every test of it runs gitio's Oid Display and oplog's ChangeId Display through
//! integration_op and the key. Covered by tests/publish_zit (publishing.rs,
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up: the publish-push records and their move;
//! an_open_push_intent_after_the_push_is_closed_done_on_the_repeat and
//! an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again: an intent opened under this key form);
//! its unit case is a repo integration test (RIT): tests/land_rit/owed_ops.rs covers it.
//! push_landing opens the intent with oplog's open_remote_intent and closes it with close_intent; settle_push_intent
//! finds one left open by its key.

use crate::landing::integration_op;
use crate::squash::LandRefs;
use gitio::Oid;
use oplog::{ChangeId, Op};

/// Kind `publish-push`, exactly one move: `refs/heads/<origin_branch>` from `expected` to `new`, the remote move the
/// guard's pre-push check allows only when an open intent lists it with the same old and new. Reason as
/// integration_op's. The key is `publish-push:<remote>:refs/heads/<origin_branch>:<expected>:<new>`, without the op
/// id, so a later publish can find an intent left open for this move. Reusing it is safe: oplog's
/// remote-intent check ignores failed records, so a refused push can be tried again, and once a push of expected to
/// new is done the origin moves to new and only ever forward, so that pair does not come back.
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
    Op {
        key: format!("publish-push:{}:{refname}:{expected}:{new}", refs.remote),
        ..integration_op("publish-push", &refname, expected, new, change_id, op_id, actor, at)
    }
}
